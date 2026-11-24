# Validation — RFTools Utility slice 1 (the tank)

> **Current state (2026-11-18): the SDK package.** The table below is the retired
> `.hnmod` bundle's ladder at pin `1d63e0a5`; the format's own ladder — `cargo test`,
> `cargo fmt --check`, `cargo clippy -D warnings`, `cargo horizon-mod build`, `check`,
> `fits` — is in the last section and in `notes.md` § "Migration to the SDK format".
> What the in-game phases below prove is unchanged: the scenario is the same one
> `tools/e2e.sh` runs today.

Pin `1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`, mixins
`5a64af938ec86b00d53d16a408aabc0038fb3e57` (schema 6). Every build/test/e2e step
ran inside `ports/tools/build-slot.sh` with `CARGO_INCREMENTAL=0`; no bare cargo,
no `--target-dir`. Logs are in `ports/_shared/evidence/rftoolsutility/`.

| # | Step | Command (inside the lease) | Evidence | Result |
|---|---|---|---|---|
| 1 | red | `tools/red-leg.sh`: every runtime body stubbed with `todo!()`, `cargo test --locked --offline` | `test-red.log` | `test result: FAILED. 0 passed; 18 failed`, `cargo_exit=101` |
| 2 | green | `cargo test --locked --offline` (standalone crate) | `test-green.log` | `test result: ok. 18 passed; 0 failed` |
| 3 | fmt | `cargo fmt --all -- --check` + `rustfmt --edition 2024 --check mixins/rftoolsutility/*.rs` | `fmt.log`, `fmt-helper.log` (first run), `fmt-clippy-final.log` (after the last edit) | exit 0, no diff |
| 4 | clippy | `cargo clippy --all-targets --locked --offline -- -D warnings` | `clippy.log`, `fmt-clippy-final.log` (after the last edit) | exit 0, 0 warnings |
| 5 | stage | `python3 tools/horizon.py stage` | `stage.log` | `stage_changed: true` (2 modules, 9 injections) |
| 6 | check | `python3 tools/horizon.py check` | `horizon-check.log` | `cargo_exit: 0`, **0 warnings** |
| 7 | embedded tests | `python3 tools/horizon.py test` | `horizon-test.log`, `horizon-test-cargo.log` | `18 passed; 0 failed; 1240 filtered out` |
| 8 | fast build | `python3 tools/horizon.py build` | `horizon-build.log` | `cargo_exit: 0`, **0 warnings**, `Finished fast profile in 39m 59s`, binary 110,338,304 B sha256 `cb5a13464deac56b…` |
| 9 | e2e | `build-slot.sh run --step e2e -- tools/evidence.sh` (build + all legs in one lease) | `e2e.log` | **PASS**, `EXIT:0` |
| 10 | e2e selftest | `SELFTEST=1 tools/e2e.sh` | `e2e-selftest.log` | **FAIL: the tank holds 1000 mB, expected 0**, `EXIT:1` |
| 11 | e2e, one file | `EMBEDDED=1 CONTROL=1 tools/e2e.sh` (no `--dev-mod`, unusable-`.horizon-cache` control) | `e2e-embedded.log` | **PASS**, `EXIT:0` |
| 12 | `.hmod` load | `tools/hmod-load.sh` (the shared unmodded game, `--dev-mod <hmod>`, then the same ids without it) | `hmod-load.log` | **PASS**, `EXIT:0` |
| 13 | packages | `tools/package.sh` | `package-hmod.log`, `package-hnmod.log` | see below |
| 14 | log checker | `python3 ports/tools/check-e2e-logs.py rftoolsutility --strict` | — | `logs=3 ok=3 missing-exit=0 bad-exit=0`, exit 0 |
| 15 | skills (notes) | `python3 ports/tools/check-skills.py --notes ports/rftoolsutility` | — | `Skills line present`, `SKILLS GATE: PASS` |
| 16 | upstream gate | `python3 ports/tools/upstream_ready.py rftoolsutility` | — | **exit 1**: `status is pending` (the orchestrator records it) and `add-on whose required parent(s) rftoolsbase are not in the registry` — a publishing condition, not a port defect; the artifact-freshness and e2e-log reasons it reported earlier are cleared |

## Red → green

The red run's 18 failures are the runtime's own tests failing at `todo!()`
(`not yet implemented`), one per behaviour: the level curve at
0/1/3,999/4,000/8,000/12,000/31,999/32,000/64,000, the empty tank, one fluid at a
time, a clamped fill, a zero/negative fill, a drain up to what is held, the
emptied tank forgetting its fluid, a water and a lava bucket filling, an empty
bucket draining water and lava, an empty tank with an empty bucket, less than a
bucket, a bucket that does not fit, a bucket of another fluid, a full water tank,
a non-container (stone, a milk bucket, another tank), and a fill-then-drain round
trip. The green run is the same 18 passing.

## In-game (The End arena, headless, one lease)

`e2e.log` phases, each `PASS`-asserted on parsed JSON, not on the absence of an
error:

| Phase | What was asserted | Observed |
|---|---|---|
| A | a tank placed from its item is empty | `empty 0/32000 mB (level 0)`, block state `level=0` |
| B | a water bucket fills exactly 1,000 mB | `minecraft:water 1000/32000 (level 1)`, hand `minecraft:bucket`, block `level=1`, **client hotbar** `[{count:1,item:minecraft:bucket,slot:0}]` |
| C | an empty bucket drains exactly 1,000 mB | `empty 0/32000 (level 0)`, hand `minecraft:water_bucket`, block `level=0` |
| D | 31,500 mB + one bucket does not fit | still `31500/32000 (level 8)`, the bucket stayed in the hand |
| E | the capacity is 32,000 mB | `/rftoolsutility tank … water 40000` → `32000/32000`, a full tank takes no bucket |
| F | a lava bucket into a water tank is refused | still `minecraft:water 1000/32000`, the lava bucket stayed |
| G | an empty bucket on an empty tank does nothing | `empty 0/32000`, hand `minecraft:bucket` |
| H | *the same click fills a vanilla cauldron* (differential) | `minecraft:water_cauldron`, hand `minecraft:bucket` |
| I | the tank keeps its fluid across save + quit + relaunch | `minecraft:water 5000/32000 (level 2)` after the relaunch, block state `level=2` |

`e2e-embedded.log` runs the same scenario with **no `--dev-mod`** (the content
comes from inside the `.hnmod`): `PASS` + `EXIT:0`, and its control leg — the
same binary with `.horizon-cache` made unusable — shows `no tank item, the block
stayed minecraft:air` and the cache file untouched, so the leg is not vacuous.
The helper wrote `.horizon-cache/rftoolsutility-embedded-content.hmod`
(**17,525 B, identical to `content/rftoolsutility.hmod`**).

`e2e-selftest.log` flips phase B's assertion (`expect_amount 0` after a real
fill): `FAIL: the tank holds 1000 mB, expected 0` + `EXIT:1`.

`hmod-load.log` loads `content/rftoolsutility.hmod` on the **shared unmodded
1d63e0a5 game** with `--dev-mod`: the tank gives and places with `level=0`; the
second launch without it refuses the item and leaves `minecraft:air`.

## Packages

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `content/rftoolsutility.hmod` | 17,525 B | `42f6030b72bd591f7cc4b914a2b3942f90c4b3c46a527dd2fe05a9debe7e69a9` | 38 |
| `rftoolsutility.hnmod` | 26,219 B | `98ad2799fc18a6288893be4274adacd227098fed3feb6b57923cd8026125b30d` | 16 |

`cargo horizon-mod check` on the `.hmod`: `rftoolsutility 0.1.0 (mod, API 0.2,
both) … no code: content only, nothing to run`. The `.hnmod` holds
`horizon-native.json`, `mixins/rftoolsutility/` (8 helpers, `runtime.rs`,
`support.rs`, the plan), `LICENSE`, `NOTICE.md` and the embedded
`data/rftoolsutility-content.hmod` (byte-identical; `tools/package.py` refuses to
pack a stale copy).

The `.hmod` rebuilt byte-identically (`42f6030b…`, 17,525 B) in two separate
`tools/package.sh` runs hours apart, and the in-game legs' comparison
(`cmp` against `content/rftoolsutility.hmod`) is part of the embedded leg.

## The SDK package (2026-11-18, `0b14f8a`)

One lease per step, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no `--target-dir`, no bare cargo
outside the registry. Logs: `../_shared/evidence/rftoolsutility/sdk-0b14f8a/`.

| # | Step | Command (inside the lease) | Result |
|---|---|---|---|
| 1 | runtime only | `cargo test` with `src/runtime.rs` copied byte-identically and no glue | `18 passed; 0 failed`, `cmp` clean against the retired file |
| 2 | red glue | the four pure helpers stubbed with `todo!()`, `cargo test` | `FAILED. 19 passed; 6 failed`, each at `not yet implemented` |
| 3 | green | `cargo test` | `25 passed; 0 failed` (18 runtime + 7 glue) |
| 4 | fmt | `cargo fmt --check` | clean |
| 5 | clippy | `cargo clippy --all-targets --offline -- -D warnings` | `Finished` with 0 warnings, 0 errors |
| 6 | package | `rm -f Cargo.lock && cargo horizon-mod build` | `Built ./target/horizon-mod/rftoolsutility.hmod` |
| 7 | registry check | `cargo horizon-mod check rftoolsutility.hmod` | `rftoolsutility 0.2.0 (native-mod, format native-1/0b14f8ac…/e59da909…, for 26.3)`, **37,485 B**, SHA-256 `7121e48f…0a5a`, **EXIT 0** |
| 8 | fits the pin | `cargo horizon-mod fits rftoolsutility.hmod --build nightly` | `RFTools Utility: / 0b14f8a  made for`, **EXIT 0** |
| 9 | reproducible | `cargo horizon-mod build` twice from the final sources | same size, same SHA-256; no `Cargo.lock` in the archive |
| 10 | runtime inside the package | `unzip -p rftoolsutility.hmod src/runtime.rs \| sha256sum` | `4350997c…755b` = the retired file, byte for byte |
| 11 | surfaces used | `cargo horizon-mod check … --uses` (`uses.json`, 309 entries) | `hook:use_item_on`, `hook:block_entity_tick`, `hook:block_entity_ticks`, `hook:register_commands`, `hook:run_mod_command`, the `BlockEntityRef`/`Registrar`/`LevelRef` handles |
| 12 | skills | `python3 ports/tools/check-skills.py --notes ports/rftoolsutility` | `Skills line present`, `SKILLS GATE: PASS` |

**Not run here:** the in-game leg. It needs a launcher-built game with this package
(`horizon-launcher native-build`), which is not installed; `tools/e2e.sh` keeps the 0.1.0
scenario and now takes `GAME=`. The behaviour evidence is the 0.1.0 runs above plus the mapping
proved in the migration.

## Slice 2 — contents on the item (2026-11-21, pin `0a875da`, package 0.2.1)

One lease per step, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no `--target-dir`. Logs:
`../_shared/evidence/rftoolsutility/{sdk-test,sdk-clippy,sdk-check,sdk-fits,sdk-fits-other}.log`
(rows 2–8, each printing the `rftoolsutility.hmod` sha256 and size it ran against) and
`../_shared/evidence/rftoolsutility-e2e/` (the game legs). The 0.2.0 and 0.1.0 logs are under
`../_shared/evidence/rftoolsutility/retired-0.2.0/`.

| # | Step | Command (inside the lease) | Result |
|---|---|---|---|
| 1 | red, unit | the three new `runtime` rules stubbed `todo!()`, `cargo test` | `FAILED. 25 passed; 3 failed`, each at `not yet implemented` |
| 2 | green, unit | `cargo test` | `31 passed; 0 failed` (21 runtime + 10 glue) |
| 3 | fmt | `cargo fmt --check` | clean |
| 4 | clippy | `cargo clippy --all-targets --offline -- -D warnings` | `Finished`, 0 warnings |
| 5 | package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | `cmp` identical; 43,884 B, SHA-256 `3e00917a…07e3f`, 43 entries, no `Cargo.lock` |
| 6 | registry check | `cargo horizon-mod check rftoolsutility.hmod` | `rftoolsutility 0.2.1 (native-mod, format native-1/0a875da…/e59da909…, for 26.3)` **EXIT 0** |
| 7 | fits the pin | `cargo horizon-mod fits rftoolsutility.hmod --build 0a875da` | `0a875da  made for` **EXIT 0** (a tautology, reported as the brief asks) |
| 8 | fits another build | `cargo horizon-mod fits rftoolsutility.hmod --build 0b14f8a` | `0b14f8a  fits` **EXIT 0** |
| 9 | gate | `python3 ports/tools/verify-sdk-package.py rftoolsutility` | **1/1** (`runtime byte-identical`, `no packed Cargo.lock`, 43 entries) |
| 10 | in game, PASS | `launcher-game.sh build` then `build-slot.sh run --step e2e -- tools/e2e.sh` | **PASS**, `EXIT:0` (`e2e-slice2.log`) |
| 11 | in game, c/f | `SELFTEST=1 SELFTEST_AT=restore` | **FAIL: the tank holds 5000 mB, expected 0**, `EXIT:1` |
| 12 | in game, c/f | `SELFTEST=1` (fill flip) | **FAIL: the tank holds 1000 mB, expected 0**, `EXIT:1` |
| 13 | in game, control | `CONTROL=1` (the unmodded Nightly) | **FAIL: the tank item did not register**, `EXIT:1` |
| 14 | log gate | `python3 ports/tools/check-e2e-logs.py rftoolsutility-e2e --strict` | `logs=5 ok=5 missing-exit=0 bad-exit=0`, exit 0 |
| 15 | skills | `python3 ports/tools/check-skills.py --notes ports/rftoolsutility` | `Skills line present`, `SKILLS GATE: PASS` |

The in-game phases and what each asserts are the table in `notes.md` § "In game". The
`fits --build 0a875da` run is the package's own pin (a tautology); `0b14f8a` is a different,
earlier Nightly, where the package still fits — the two lines the brief asks for are both here.

## Slice 3 — the crafter (2026-11-21, pin `0a875da`, package 0.2.2)

One lease per step, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no `--target-dir`. Logs:
`../_shared/evidence/rftoolsutility/{slice3-red,slice3-red-craftmode,slice3-green,sdk-test,sdk-clippy,sdk-check,sdk-fits,sdk-fits-other}.log`
(each `sdk-*` log prints the `rftoolsutility.hmod` sha256 and size it ran against) and
`../_shared/evidence/rftoolsutility-e2e/` (the game legs).

| # | Step | Command (inside the lease) | Result |
|---|---|---|---|
| 1 | red, unit | the nine new `runtime` rules stubbed `todo!()`, `cargo test` | `FAILED. 31 passed; 9 failed`, each at `not yet implemented` (`slice3-red.log`) |
| 2 | red, modes | the two `CraftMode` rules stubbed, `cargo test` | `FAILED. 40 passed; 2 failed` at `result_range` (`slice3-red-craftmode.log`) |
| 3 | green, unit | `cargo test` after the two in-game corrections below | **48 passed; 0 failed** (21 tank runtime + 12 crafter runtime + 10 tank glue + 5 crafter glue) |
| 4 | fmt | `cargo fmt --check` | clean |
| 5 | clippy | `cargo clippy --all-targets --offline -- -D warnings` | `Finished`, 0 warnings |
| 6 | package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | `cmp` identical; **63,478 B**, SHA-256 `ba50851d…463eb`, 52 entries, no `Cargo.lock` |
| 7 | registry check | `cargo horizon-mod check rftoolsutility.hmod` | `rftoolsutility 0.2.2 (native-mod, format native-1/0a875da…/e59da909…, for 26.3)` **EXIT 0** |
| 8 | fits the pin | `cargo horizon-mod fits rftoolsutility.hmod --build 0a875da` | `0a875da  made for` **EXIT 0** (a tautology, reported as the brief asks) |
| 9 | fits another build | `cargo horizon-mod fits rftoolsutility.hmod --build 0b14f8a` | `0b14f8a  fits` **EXIT 0** |
| 10 | gate | `python3 ports/tools/verify-sdk-package.py rftoolsutility` | **1/1** (`48 passed`, `no packed Cargo.lock`, 52 entries, skills PASS) |
| 11 | in game, PASS | `launcher-game.sh build` (`Compiling rftoolsutility (1 of 1)`), then `build-slot.sh run --step e2e -- tools/e2e.sh` | **PASS**, `EXIT:0` — phases A–V (`e2e-slice3.log`) |
| 12 | in game, c/f | `SELFTEST=1 SELFTEST_AT=craft` | **FAIL: the crafter holds 4700 FE, expected 5000**, `EXIT:1` |
| 13 | in game, control | `CONTROL=1` (the unmodded Nightly) | **FAIL: the tank item did not register**, `EXIT:1` |
| 14 | log gate | `python3 ports/tools/check-e2e-logs.py rftoolsutility-e2e --strict` | `logs=8 ok=8 missing-exit=0 bad-exit=0`, exit 0 |
| 15 | skills | `python3 ports/tools/check-skills.py --notes ports/rftoolsutility` | `SKILLS GATE: PASS` |

Two rules changed *after* meeting the game, each red before the fix (both in
`notes.md` § "Red → green"): one buffer slot feeds several cells (upstream's `split(1)` loop,
which chained crafting needs), and a cycle costs 100 FE whatever it crafted (`rf * i` counts
cycles). The first in-game failure this slice produced was a real defect of the glue — `plan`
and `placements` number their own slice, and the glue used those indices as world slots, so the
product was written into the ghost slots; it is fixed and pinned by
`the_ghost_grid_shows_one_item_per_filled_cell`.

## Slice 4 — the spawner (2026-11-21, pin `0a875da`, package 0.2.3)

One lease per step, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no `--target-dir`. Logs:
`../_shared/evidence/rftoolsutility/{slice4-red,slice4-runtime-green,slice4-glue-green,sdk-test,sdk-clippy,sdk-check,sdk-fits,sdk-fits-other,sdk-package-gate}.log`
(each `sdk-*` log prints the `rftoolsutility.hmod` sha256 and size it ran against) and
`../_shared/evidence/rftoolsutility-e2e/e2e-slice4*.log` (the game legs). The 0.2.2 logs are
under `../_shared/evidence/rftoolsutility/retired-0.2.2/`.

| # | Step | Command (inside the lease) | Result |
|---|---|---|---|
| 1 | red, unit | the four new `runtime` rules stubbed `todo!()`, `cargo test` | `FAILED. 48 passed; 4 failed`, each at `not yet implemented` (`slice4-red.log`) |
| 2 | green, runtime | `cargo test` after the four bodies | **52 passed; 0 failed** (`slice4-runtime-green.log`) |
| 3 | green, glue | `cargo test` with the six spawner glue tests | **58 passed; 0 failed** (`slice4-glue-green.log`, `sdk-test.log`) |
| 4 | fmt | `cargo fmt --check` | clean |
| 5 | clippy | `cargo clippy --all-targets --offline -- -D warnings` | `Finished`, 0 warnings (`sdk-clippy.log`) |
| 6 | package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice from the final sources | `cmp` identical; **103,488 B**, SHA-256 `d12c3646abde71c541adc707add6a044a934a77ce375a301d4de1a46e52c27b7`, 134 entries, no `Cargo.lock` |
| 7 | registry check | `cargo horizon-mod check rftoolsutility.hmod` | `rftoolsutility 0.2.3 (native-mod, format native-1/0a875da…/e59da909…, for 26.3)` **EXIT 0** (`sdk-check.log`) |
| 8 | fits the pin | `cargo horizon-mod fits rftoolsutility.hmod --build 0a875da` | `0a875da  made for` **EXIT 0** (a tautology, reported as the brief asks; `sdk-fits.log`) |
| 9 | fits another build | `cargo horizon-mod fits rftoolsutility.hmod --build 0b14f8a` | `0b14f8a  fits` **EXIT 0** (`sdk-fits-other.log`) |
| 10 | gate | `python3 ports/tools/verify-sdk-package.py rftoolsutility` | **1/1** (58 passed, 0 clippy warnings, `no packed Cargo.lock`, 134 entries, skills PASS) |
| 11 | native compile | `launcher-game.sh build rftoolsutility` | `Compiling rftoolsutility (1 of 1)` — the mod compiled against the pin, then exported |
| 12 | in game, PASS | `build-slot.sh run --step e2e -- tools/e2e.sh` | **PASS**, `EXIT:0` — phases A–AB (`e2e-slice4.log`) |
| 13 | in game, c/f | `SELFTEST=1 SELFTEST_AT=spawn` | **FAIL: the spawner holds 0 FE, expected 800**, `EXIT:1` |
| 14 | in game, control | `CONTROL=1` (the unmodded Nightly) | **FAIL: the tank item did not register**, `EXIT:1` |
| 15 | log gate | `python3 ports/tools/check-e2e-logs.py rftoolsutility-e2e --strict` | `logs=11 ok=11 missing-exit=0 bad-exit=0`, exit 0 |
| 16 | coverage | `python3 ports/tools/check-coverage.py rftoolsutility --no-upstream` | `families=12 tested=7 declared=5 missing=0` (the whole-jar inventory still lists the unported families, as a `ported-partial` port should) |
| 17 | skills | `python3 ports/tools/check-skills.py --notes ports/rftoolsutility` | `SKILLS GATE: PASS` |

The package's identical builds are `8c836e61…` (pre-fix source), `a55d4e5f…` (after the reply
fix) and `d12c3646…` (the final source); the in-game legs above ran on a game the launcher built
from that final source (`native-0a875da-878276d6ae1e`), which it compiled itself
(`Compiling rftoolsutility (1 of 1)`). What each new phase asserts is the table in `notes.md`
§ "Slice 4 — the spawner", and the two traps the slice cost (a control reply's leading slash,
an entity blocking `ctl place`) are recorded there.

## Slice 5 — the crafter tiers (2026-11-21, pin `0a875da`, package 0.2.4)

One lease per step, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no `--target-dir`, no bare cargo.
Logs: `../_shared/evidence/rftoolsutility/{slice5-red,slice5-green,slice5-clippy,slice5-build-1,
slice5-build-2,slice5-check,slice5-fits,slice5-fits-other,slice5-package-gate}.log` (the `sdk-*`
logs are copies of these for the gate), and `../_shared/evidence/rftoolsutility-e2e/
{e2e-slice5-red,e2e-slice5,e2e-slice5-selftest-tiers,e2e-slice5-control,slice5-native-build}.log`.
The 0.2.3 logs are under `../_shared/evidence/rftoolsutility/retired-0.2.3/`.

| # | Step | Command (inside the lease) | Result |
|---|---|---|---|
| 1 | red, unit | the tier rule stubbed `todo!()` (3 runtime tests + 1 glue test), `cargo test` | `FAILED. 58 passed; 4 failed`, each at `not yet implemented` (`slice5-red.log`) |
| 2 | green | `cargo test` after the implementation | **62 passed; 0 failed** (`slice5-green.log`, `sdk-test.log`) |
| 3 | fmt | `cargo fmt --check` | clean |
| 4 | clippy | `cargo clippy --all-targets --offline -- -D warnings` | `Finished`, 0 warnings (`slice5-clippy.log`, `sdk-clippy.log`) |
| 5 | package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice from the final sources | `cmp` identical; **111,935 B**, SHA-256 `4cd3abdb4786ff498bfd2d45bfb8d50a444f7f89d18d396a93ebd9c2e427534f`, 152 entries, no `Cargo.lock` |
| 6 | registry check | `cargo horizon-mod check rftoolsutility.hmod` | `rftoolsutility 0.2.4 (native-mod, format native-1/0a875da…/e59da909…, for 26.3)` **CHECK_EXIT:0** (`slice5-check.log`) |
| 7 | fits the pin | `cargo horizon-mod fits rftoolsutility.hmod --build 0a875da` | `0a875da  made for` **FITS_EXIT:0** (a tautology, reported as the brief asks; `slice5-fits.log`, sha `4cd3abdb…`) |
| 8 | fits another build | `cargo horizon-mod fits rftoolsutility.hmod --build 0b14f8a` | `0b14f8a  fits` **FITS_EXIT:0** (`slice5-fits-other.log`, sha `4cd3abdb…`) |
| 9 | gate | `python3 ports/tools/verify-sdk-package.py rftoolsutility` | **1/1** (62 passed, 0 clippy warnings, `no packed Cargo.lock`, 152 entries, skills PASS; `slice5-package-gate.log`) |
| 10 | native compile | `launcher-game.sh build rftoolsutility --port ports/rftoolsutility` (in a slot) | `Compiling rftoolsutility (1 of 1)` — the mod compiled against the pin, then exported (`slice5-native-build.log`) |
| 11 | in game, red | the new script on the **0.2.3** package | **FAIL: ctl place rftoolsutility:crafter3 … was refused three times** (`no rftoolsutility:crafter3 in the inventory`), `EXIT:1` (`e2e-slice5-red.log`) |
| 12 | in game, PASS | `build-slot.sh run --step e2e -- tools/e2e.sh` | **PASS**, `EXIT:0` — phases A–AF (`e2e-slice5.log`) |
| 13 | in game, c/f | `SELFTEST=1 SELFTEST_AT=tiers` | **FAIL: the crafter holds 0 FE, expected 800**, `EXIT:1` (`e2e-slice5-selftest-tiers.log`) |
| 14 | in game, control | `CONTROL=1` (the unmodded Nightly) | **FAIL: the tank item did not register**, `EXIT:1` (`e2e-slice5-control.log`) |
| 15 | log gate | `python3 ports/tools/check-e2e-logs.py rftoolsutility-e2e --strict` | `logs=15 ok=15 missing-exit=0 bad-exit=0`, exit 0 |
| 16 | coverage | `python3 ports/tools/check-coverage.py rftoolsutility --no-upstream` | `families=14 tested=8 declared=6 missing_families=0 entries=92 named=92 missing=0`, exit 0 |
| 17 | skills | `python3 ports/tools/check-skills.py --notes ports/rftoolsutility` | `SKILLS GATE: PASS` |

The game the legs ran on was built by the launcher from this source
(`native-0a875da-c2f7cf370efc`, the 0.2.4 package installed) and compiled by it
(`Compiling rftoolsutility (1 of 1)`). What each new phase asserts is the table in `notes.md`
§ "Slice 5 — the crafter tiers", and the trap the slice cost (`$CRAFTER` being a prefix of
`$CRAFTER2` for the coverage tool's variable expansion) is recorded there.

## Slice 6 — the tick's own dimension (2026-11-21, pin `0a875da`, package 0.2.5)

The reviewer's correctness bug on PR #1643: the crafter and spawner resolved their tick's level by
scanning for a level holding a matching block at the position, so the same machine at the same
`x,y,z` in two dimensions made the tick act in the first dimension. Fix: `ticking_level` = the level
in place (`LevelRef::in_place`); both helpers deleted. No unit test was added (the rule needs the
game's level list; the fail-capable regression is the in-game leg below). Logs:
`../_shared/evidence/rftoolsutility/slice6-*.log` and `../_shared/evidence/rftoolsutility-e2e/e2e-slice6*.log`.

| # | Step | Command (inside the lease) | Result |
|---|---|---|---|
| 1 | red, in game | `tools/e2e.sh` (new phase AG) on the **0.2.4** export | **FAIL: the End's spawner spawned 0 zombie(s) at 202,101,19 in its own dimension, expected 1**, `EXIT:1` (`e2e-slice6-red.log`) |
| 2 | unit | `cargo test --offline` | `62 passed; 0 failed` (`slice6-test.log`, `sdk-test.log`) |
| 3 | fmt + clippy | `cargo fmt -- --check` + `cargo clippy --all-targets --offline -- -D warnings` | `FMT_CLEAN`, `CLIPPY_CLEAN`, 0 warnings (`slice6-clippy.log`) |
| 4 | package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | `cmp` identical; **112,059 B, SHA-256 `213695fc…`**, 152 entries, no `Cargo.lock` |
| 5 | registry check | `cargo horizon-mod check rftoolsutility.hmod` | `rftoolsutility 0.2.5 (native-mod, format native-1/0a875da…/e59da909…, for 26.3)`, `EXIT:0` |
| 6 | fits the pin | `sha256sum` + `cargo horizon-mod fits … --build 0a875da` | `213695fc…  rftoolsutility.hmod`, `0a875da  made for`, `EXIT:0` |
| 7 | fits another build | `sha256sum` + `cargo horizon-mod fits … --build 0b14f8a` | `213695fc…  rftoolsutility.hmod`, `0b14f8a  fits`, `EXIT:0` |
| 8 | gate | `python3 ports/tools/verify-sdk-package.py rftoolsutility` | **1/1** (`runtime byte-identical`, `no packed Cargo.lock`, 62 tests, 0 warnings) |
| 9 | native build | `ports/tools/launcher-game.sh build rftoolsutility` | `Compiling rftoolsutility (1 of 1)`, `built native-0a875da-2d419ffa479a` (`slice6-native-build.log`) |
| 10 | in game, PASS | `build-slot.sh run --step e2e -- tools/e2e.sh` | **PASS**, `EXIT:0`, phases A–AG (`e2e-slice6.log`) |
| 11 | in game, control | `CONTROL=1` (the unmodded Nightly) | **FAIL: the tank item did not register**, `EXIT:1` |
| 12 | in game, counterfactual | `SELFTEST=1 SELFTEST_AT=dimension` | **FAIL: the End's spawner spawned 1 zombie(s) in its own dimension, expected 0**, `EXIT:1` |
| 13 | in game, the earlier legs | `SELFTEST_AT=fill|restore|craft|spawn|tiers` | each `FAIL:` + `EXIT:1` |
| 14 | log checker | `python3 ports/tools/check-e2e-logs.py rftoolsutility-e2e --strict` | `logs=24 ok=24 missing-exit=0 bad-exit=0`, `EXIT:0` |
| 15 | skills | `python3 ports/tools/check-skills.py --notes ports/rftoolsutility` | `Skills line present`, `SKILLS GATE: PASS` |

**The two-dimension assertion (phase AG), verbatim from the green run:**

```
  the fed End crafter: 36=minecraft:oak_planksx12, 0 FE left
  the unfed Overworld twin: '-' in, '-' out, '-', 0 FE
  zombies at 202,101,19: the End has 1, the Overworld twin has 0; the End spawner reads 0 FE and spawned=1
  the End spawner acted in the End; the Overworld twin stayed empty
```

and the same three lines in the red run (0.2.4):

```
  the fed End crafter: 36=minecraft:oak_planksx12, 0 FE left
  the unfed Overworld twin: '-' in, '-' out, '-', 0 FE
  zombies at 202,101,19: the End has 0, the Overworld twin has 0; the End spawner reads 800 FE and spawned=0
FAIL: the End's spawner spawned 0 zombie(s) at 202,101,19 in its own dimension, expected 1: []
```

**Harness fixes this slice needed** (recorded in `notes.md`, no assertion weakened): `held_item_line`
now reads `/data get entity @s Inventory` instead of `SelectedItem` (a machine broken with a buffer
drops its contents, `ctl collect` fills the hotbar, and the machine's item lands in the main
inventory); `place_kept` copies the carried stack into hotbar slot 8 with
`item replace entity @s hotbar.8 from entity @s container.<slot>`.

## Slice 7 — the spawner's matter gate (2026-11-22, pin `0a875da`, package 0.2.6)

The spawner now gates on the three matter amounts its own recipe names and spends them with the
energy (`SpawnerTileEntity.tickServer`), keeps them across a break and carries them on its item.
The stores are upstream's three floats in the machine's data — **not** three fluid tanks: the
spawner module has no fluid at all (`SpawnerData` is `matter0/1/2` floats and
`addMatter(ItemStack, int, float)` maps a beamed item to one of them), and the reason is recorded in
`notes.md` § "Slice 7" § "The brief said three fluid tanks; the artefact says three floats".

One lease per step, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no `--target-dir`, no bare cargo. Logs:
`../_shared/evidence/rftoolsutility/{slice7-red,slice7-clippy,slice7-build-1,slice7-build-2,
sdk-test,sdk-clippy,sdk-check,sdk-fits,sdk-fits-other,slice7-package-gate}.log` and
`../_shared/evidence/rftoolsutility-e2e/e2e-slice7*.log` + `slice7-native-build.log`.

| # | Step | Command (inside the lease) | Result |
|---|---|---|---|
| 1 | red, unit | the five new `runtime` rules stubbed `todo!()`, `cargo test` | `FAILED. 62 passed; 5 failed`, each at `not yet implemented` (`slice7-red.log`) |
| 2 | green | `cargo test` | **69 passed; 0 failed** (`sdk-test.log`) |
| 3 | fmt | `cargo fmt --check` | clean |
| 4 | clippy | `cargo clippy --all-targets --offline -- -D warnings` | `Finished`, 0 warnings (`slice7-clippy.log`, `sdk-clippy.log`) |
| 5 | package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | `cmp` identical; **115,039 B**, SHA-256 `f476e137…bbcb`, 152 entries, no `Cargo.lock` |
| 6 | registry check | `cargo horizon-mod check rftoolsutility.hmod` | `rftoolsutility 0.2.6 (native-mod, format native-1/0a875da…/e59da909…, for 26.3)` **CHECK_EXIT:0** |
| 7 | fits the pin | `sha256sum` + `cargo horizon-mod fits … --build 0a875da` | `f476e137…  rftoolsutility.hmod`, `0a875da  made for`, **FITS_EXIT:0** |
| 8 | fits another build | `sha256sum` + `cargo horizon-mod fits … --build 0b14f8a` | `f476e137…  rftoolsutility.hmod`, `0b14f8a  fits`, **FITS_EXIT:0** |
| 9 | gate | `python3 ports/tools/verify-sdk-package.py rftoolsutility` | **1/1** (115,039 B, `runtime byte-identical`, 69 tests, 0 clippy warnings, no packed lock, 152 entries, skills PASS) |
| 10 | native build | `ports/tools/launcher-game.sh build rftoolsutility --port ports/rftoolsutility` | `Compiling rftoolsutility (1 of 1)`, `native-0a875da-2600432bf5cd` (`slice7-native-build.log`) |
| 11 | in game, red | the new script on the **0.2.5** export | **FAIL: the spawner line did not parse** (0.2.5 has no `matter=` field), `EXIT:1` (`e2e-slice7-red.log`) |
| 12 | in game, PASS | `build-slot.sh run --step e2e -- bash ports/rftoolsutility/tools/e2e.sh` | **PASS**, `EXIT:0`, phases A–AG (`e2e-slice7.log`) |
| 13 | in game, c/f | `SELFTEST=1 SELFTEST_AT=gate` | **FAIL: the spawner holds 800 FE, expected 0**, `EXIT:1` (`e2e-slice7-selftest-gate.log`) |
| 14 | in game, c/f | `SELFTEST=1 SELFTEST_AT=spawn` | **FAIL: the spawner holds 0 FE, expected 800**, `EXIT:1` (`e2e-slice7-selftest-spawn.log`) |
| 15 | in game, c/f | `SELFTEST=1 SELFTEST_AT=dimension` | **FAIL: the End's spawner spawned 1 zombie(s) in its own dimension, expected 0**, `EXIT:1` (`e2e-slice7-selftest-dimension.log`) |
| 16 | in game, the earlier legs | `SELFTEST_AT=fill|restore|craft|tiers` | each `FAIL:` + `EXIT:1` (B, K, Q, AC inverted) |
| 17 | in game, control | `CONTROL=1` (the unmodded Nightly) | **FAIL: the tank item did not register**, `EXIT:1` (`e2e-slice7-control.log`) |
| 18 | log gate | `python3 ports/tools/check-e2e-logs.py rftoolsutility-e2e --strict` | `logs=34 ok=34 missing-exit=0 bad-exit=0`, exit 0 |
| 19 | coverage | `python3 ports/tools/check-coverage.py rftoolsutility --no-upstream` | `families=15 tested=9 declared=6 missing_families=0 entries=92 named=92 missing=0`, exit 0 |
| 20 | skills | `python3 ports/tools/check-skills.py --notes ports/rftoolsutility` | `SKILLS GATE: PASS` |

**The gate's own transcript (from `e2e-slice7.log`):**

```
== Y1: 800 FE with no matter spawns nothing and pays nothing (the matter gate) ==
  Spawner at [202, 101, 3]: energy=800/200000 mob=minecraft:zombie cost=800 needs=0.1,0.2,20 matter=0,0,0 spawned=0 nonce=33
  800 FE and no matter: 0 zombie(s), 800 FE kept, matter=0,0,0, spawned=0
== Y2: one matter type one unit short still spawns nothing ==
  Spawner at [202, 101, 3]: energy=800/200000 mob=minecraft:zombie cost=800 needs=0.1,0.2,20 matter=0.1,0.2,19.9 spawned=0 nonce=35
== Z: exactly the recipe's matter and its 800 FE spawn one zombie and spend both ==
  Spawner at [202, 101, 3]: energy=0/200000 mob=minecraft:zombie cost=800 needs=0.1,0.2,20 matter=0,0,0 spawned=1 nonce=38
== Z2: each store is short by exactly its own amount, not by one of them ==
  Spawner at [202, 101, 3]: energy=0/200000 mob=minecraft:zombie cost=800 needs=0.1,0.2,20 matter=1,1,1 spawned=2 nonce=39
```

The two in-game defects this slice's first green run caught are in `notes.md` § "Measured traps"
(a readout taken after a feed is a spawn; a float leftover must be binary-exact to be asserted).

## Slice 8 — the matter beamer (2026-11-22, pin `0a875da`, package 0.2.7)

The beamer fills the spawner's three matter stores from the materials its recipe's
`item1..3.object` names: one block a send for 2,000 FE, every 20 ticks, behind a redstone signal
and a command-linked spawner. One lease per step, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no
`--target-dir`. Logs: `../_shared/evidence/rftoolsutility/{slice8-red,slice8-runtime-green,
slice8-glue-red,slice8-green,slice8-clippy,slice8-build-1,slice8-build-2,sdk-test,sdk-clippy,
sdk-check,sdk-fits,sdk-fits-other}.log` and `../_shared/evidence/rftoolsutility-e2e/e2e-slice8*.log`
+ `slice8-native-build.log`.

| # | Step | Command (inside the lease) | Result |
|---|---|---|---|
| 1 | red, runtime | the eight new `runtime` rules stubbed `todo!()`, `cargo test` | `FAILED. 69 passed; 8 failed`, each at `not yet implemented` (`slice8-red.log`) |
| 2 | green, runtime | `cargo test` | **77 passed; 0 failed** (`slice8-runtime-green.log`) |
| 3 | red, glue | the object parser, the destination codec and the readout stubbed `todo!()`, `cargo test` | `FAILED. 75 passed; 5 failed` (`slice8-glue-red.log`) |
| 4 | green | `cargo test` | **80 passed; 0 failed** (52 runtime + 28 glue; `slice8-green.log`, `sdk-test.log`) |
| 5 | fmt + clippy | `cargo fmt --check` + `cargo clippy --all-targets --offline -- -D warnings` | clean, 0 warnings (`slice8-clippy.log`, `sdk-clippy.log`) |
| 6 | package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | `cmp` identical; **129,078 B**, SHA-256 `2f4706bc8c2e35c148df11762cc2219ddca249fbefa23e625341e93c1d22adaf`, 167 entries, no `Cargo.lock` |
| 7 | registry check | `cargo horizon-mod check rftoolsutility.hmod` | `rftoolsutility 0.2.7 (native-mod, format native-1/0a875da…/e59da909…, for 26.3)`, **CHECK_EXIT:0** (`sdk-check.log`) |
| 8 | fits the pin | `sha256sum` + `cargo horizon-mod fits … --build 0a875da` | `2f4706bc…  rftoolsutility.hmod`, `0a875da  made for`, **FITS_EXIT:0** (`sdk-fits.log`) |
| 9 | fits another build | `sha256sum` + `cargo horizon-mod fits … --build 0b14f8a` | `2f4706bc…  rftoolsutility.hmod`, `0b14f8a  fits`, **FITS_EXIT:0** (`sdk-fits-other.log`) |
| 10 | gate | `python3 ports/tools/verify-sdk-package.py rftoolsutility` | **1/1** (129,078 B, `runtime byte-identical`, 80 tests, 0 clippy warnings, no packed lock, 167 entries, skills PASS) |
| 11 | native build | `ports/tools/launcher-game.sh build rftoolsutility --port ports/rftoolsutility` (after removing the previous export) | `Compiling rftoolsutility (1 of 1)`, `native-0a875da-129559a0b8f3` (`slice8-native-build.log`) |
| 12 | in game, PASS | `build-slot.sh run --step e2e -- bash ports/rftoolsutility/tools/e2e.sh` | **PASS**, `EXIT:0`, phases A–AH (`e2e-slice8.log`) |
| 13 | in game, c/f | `SELFTEST=1 SELFTEST_AT=beamer` | **FAIL: the spawner's matter reads '2,0,0', expected '0,0,0'**, `EXIT:1` (`e2e-slice8-selftest-beamer.log`) |
| 14 | in game, control | `CONTROL=1` (the unmodded Nightly) | **FAIL: the tank item did not register**, `EXIT:1` (`e2e-slice8-control.log`) |
| 15 | log gate | `python3 ports/tools/check-e2e-logs.py rftoolsutility-e2e --strict` | `logs=37 ok=37 missing-exit=0 bad-exit=0`, `EXIT:0` (the 34 earlier legs still green) |
| 16 | coverage | `python3 ports/tools/check-coverage.py rftoolsutility --no-upstream` | `EXIT:0`; the `matter_beamer` row is `tested` (`notes.md` § "Coverage after slice 8") |
| 17 | skills | `python3 ports/tools/check-skills.py --notes ports/rftoolsutility` | `Skills line present`, `SKILLS GATE: PASS` |

**The AH transcript (from `e2e-slice8.log`):**

```
== AH2: item1's material feeds the first store; 4,000 FE is two sends of 2,000 ==
  Beamer at [200, 101, -12]: energy=0/200000 dest=[200, 101, -16] slot=minecraft:rotten_fleshx6 sent=2 lit=true nonce=65
  Spawner at [200, 101, -16]: energy=0/200000 mob=minecraft:zombie cost=800 needs=0.1,0.2,20 matter=2,0,0 spawned=0 nonce=66
== AH4: the living item (item3) carries its yield tag's factor: beef is 1.5 a block ==
  Beamer at [200, 101, -12]: energy=0/200000 dest=[200, 101, -16] slot=minecraft:beefx6 sent=6 lit=true nonce=69
  Spawner at [200, 101, -16]: energy=0/200000 mob=minecraft:zombie cost=800 needs=0.1,0.2,20 matter=2,2,3 spawned=0 nonce=70
== AH5: a material the recipe does not take is refused without spending anything ==
  Beamer at [200, 101, -12]: energy=4000/200000 dest=[200, 101, -16] slot=minecraft:cobblestonex8 sent=6 lit=true nonce=71
```

The two in-game traps this slice's first run caught are in `notes.md` § "Measured traps": a material
loaded while the buffer is charged is itself a send (AH6/AH8's order), and `ctl collect` walks (the
drop is picked up by teleporting onto the item entity's own `pos`). The `fits --build 0b14f8a` false
fail on a variant named `Item` — and its fix — is recorded there too.

**Not a red leg on the previous package.** A 0.2.6 red run of the same script would fail at the
first `place_block rftoolsutility:matter_beamer` ("the item did not register") because the block does
not exist there; the unit red (rows 1 and 3), the control leg (row 14) and the in-phase differentials
(AH5–AH8) are this slice's fail-capable evidence, and no claim depends on a 0.2.6 run.

## Slice 9 — the teleporter family (0.2.8, forwarded to AdLf-PC)

Every compile/ladder step below went through `build-slot.sh run --job G12-rftoolsutility --port
ports/rftoolsutility --step <…>`, which printed `build-slot: forwarding <step> for
ports/rftoolsutility to the remote runner`; the remote host is `AdLf-PC`
(`deepseek@192.168.1.115`). `BUILD_LOCAL=1` was never set. The game leg stayed local (the remote
game/ctl pair is linked on this box's glibc); the in-game slots were entered through
`build-slot.sh run --step e2e` and the lease was held inside the turn.

| # | check | command | result |
|---|---|---|---|
| 1 | red, runtime | the five teleporter rules stubbed `todo!()` behind their tests, `cargo test` | `FAILED. 0 passed; 1 failed; 84 filtered out`, `a_cross_dimension_dial_costs_one_hundred_thousand_and_fifty_ticks … panicked at src/runtime.rs:837:5: not yet implemented` |
| 2 | green | `cargo test` | **85 passed; 0 failed** (`sdk-test.log`) |
| 3 | fmt + clippy | `cargo fmt --check`, `cargo clippy --all-targets --offline -- -D warnings` | clean, 0 warnings (`sdk-clippy.log`) |
| 4 | package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | `cmp` identical; **147,852 B**, SHA-256 `60b0204474fda391b15947f1c44a8812ada6710dd9281fa7ed9836e98fbf317d`, no `Cargo.lock` (`build-a.sha`, `build-b.sha`) |
| 5 | registry check | `cargo horizon-mod check rftoolsutility.hmod` | `rftoolsutility 0.2.8 (native-mod, format native-1/0a875da…/e59da909…, for 26.3)`, **CHECK_EXIT:0** (`sdk-check.log`) |
| 6 | fits the pin | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, **FITS_EXIT:0** (`sdk-fits.log`) |
| 7 | fits another build | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, **FITS_EXIT:0** (`sdk-fits-other.log`) |
| 8 | gate | `python3 ports/tools/verify-sdk-package.py rftoolsutility` | **1/1** (package matches the tree, R9, skills gate) |
| 9 | native build | `ports/tools/launcher-game.sh build rftoolsutility` | `Compiling rftoolsutility (1 of 1)`, `native-0a875da-8356e2011e25` |
| 10 | in game, PASS | `build-slot.sh run --step e2e -- bash ports/rftoolsutility/tools/e2e-teleporter.sh` | **PASS**, `EXIT:0`, AI–AI2 (`e2e-slice9-teleporter.log`): undialed transmitter moves nobody; dialed one moves the player from `203.5,102.0,8.5` to `203.5,102.0,12.5`; readout `energy=186600/200000 … state=cooldown` |
| 11 | in game, control | `CONTROL=1` (the unmodded Nightly) | **FAIL: Unknown item 'rftoolsutility:matter_transmitter'**, `EXIT:1` (`e2e-slice9-teleporter-control.log`) |
| 12 | in game, c/f | `SELFTEST=1 SELFTEST_AT=teleport` | **FAIL: the player left the beam for `203.5,102.0,12.5`, expected it to stay at z=8.5**, `EXIT:1` (`e2e-slice9-teleporter-selftest.log`) |
| 13 | log gate | `check-e2e-logs.py rftoolsutility-e2e --strict` | `logs=41 ok=41 missing-exit=0 bad-exit=0`, `EXIT:0` |

**Traps handled:** the manifest was bumped first and the recompile confirmed
(`Compiling rftoolsutility (1 of 1)`); no `native-*` tag carried this mod in the launcher's version
store (verified before the build), so there was no stale build to reuse; `E2E_TMP` came from
`launcher-game.sh env` (`/home/adlf/.cache/horizon-e2e/scratch`), not `/tmp`; `sdk-test.log` was
regenerated for 0.2.8; the docs (`README.md`, packed) were edited before the final build and the
gate confirmed `package matches the tree`; R9 holds (SDK path dependency only, no `[workspace]`, no
packed `Cargo.lock`); no local type or variant is named `Block`/`Item`, so neither `fits` run
false-failed.

## Slice 10 — the destination analyzer (0.2.9, forwarded to AdLf-PC)

All Rust ladder steps went through `build-slot.sh run --job W1-rftoolsutility --port
ports/rftoolsutility --step test|lint|fmt|build|check -- …`, each printing `build-slot: forwarding
… to the remote runner`; `BUILD_LOCAL=1` was never set. The launcher's native build cannot run on
the far box (`libxkbcommon-x11.so.0` missing), so it ran locally in a `--step native` lease; the
game/ctl legs stayed local on this box's glibc. No lease was backgrounded.

| # | check | command | result |
|---|---|---|---|
| 1 | red, runtime | five analyzer rules stubbed `todo!()` behind their tests, `cargo test` | `FAILED. 85 passed; 5 failed`, each at `not yet implemented` (`slice10-red.log`) |
| 2 | red, glue | `analyzer_line` stubbed `todo!()` behind its test | `FAILED. 0 passed; 1 failed; 90 filtered out` (`slice10-glue-red.log`) |
| 3 | green | `cargo test --offline` | **91 passed; 0 failed** (`sdk-test.log`, regenerated for 0.2.9 after the last forwarded run) |
| 4 | fmt + clippy | `cargo fmt --check`, `cargo clippy --all-targets --offline -- -D warnings` | clean, 0 warnings (`slice10-fmt.log`, `sdk-clippy.log`) |
| 5 | package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | `cmp` identical; **155,241 B**, sha `54791b79…3cf5`, 203 entries, no `Cargo.lock` (`slice10-build-a.sha`, `-b.sha`) |
| 6 | registry check | `cargo horizon-mod check rftoolsutility.hmod` | `rftoolsutility 0.2.9 (native-mod, format native-1/0a875da…/e59da909…, for 26.3)`, **CHECK_EXIT:0** (`sdk-check.log`) |
| 7 | fits the pin | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, **FITS_EXIT:0** (`sdk-fits.log`) |
| 8 | fits another build | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, **FITS_EXIT:0** (`sdk-fits-other.log`) |
| 9 | gate | `python3 ports/tools/verify-sdk-package.py rftoolsutility` | **1/1** (package matches the tree, R9, skills gate; `slice10-package-gate.log`) |
| 10 | native build | `ports/tools/launcher-game.sh build rftoolsutility` | `Compiling rftoolsutility (1 of 1)`, `native-0a875da-c1cea11ff937` (`slice10-native-build.log`) |
| 11 | in game, PASS | `build-slot.sh run --step native -- bash ports/rftoolsutility/tools/e2e-analyzer.sh` | **PASS**, `EXIT:0` — AJ–AJ3 (`e2e-slice10-analyzer.log`): signal 0 → 15 → 0 as the receiver is charged and then blocked |
| 12 | in game, c/f | `SELFTEST=1 SELFTEST_AT=analyzer` | **FAIL: SELFTEST analyzer: the charged receiver reads signal=15, expected it to stay low**, `EXIT:1` (`e2e-slice10-analyzer-selftest.log`) |
| 13 | in game, control | `CONTROL=1` (the unmodded Nightly) | **FAIL: no rftoolsutility:matter_transmitter in the inventory**, `EXIT:1` (`e2e-slice10-analyzer-control.log`) |
| 14 | no regression | the same game, `tools/e2e-teleporter.sh` | **PASS**, `EXIT:0` — AI–AI2: dial, arrival at `203.5,102.0,12.5`, readout `energy=186600/200000 … state=cooldown status=none` (`e2e-slice10-teleporter.log`) |
| 15 | log gate | `check-e2e-logs.py rftoolsutility-e2e --strict` | `logs=46 ok=46 missing-exit=0 bad-exit=0`, `EXIT:0` |

**Traps handled:** the two stale `native-0a875da-…` tags carrying 0.2.8 were removed before the
build so the launcher actually recompiled (`Compiling rftoolsutility (1 of 1)`); `E2E_TMP` and all
four path exports came from `launcher-game.sh env`, never `/tmp`; `cargo horizon-mod build` writes
into `target/horizon-mod/`, so the delivered port-root copy was installed and `cmp`-checked after
the two builds; the forwarded pull-back overwrote a locally-teed `sdk-test.log` with the old
85-pass run, so the final 91-pass log was staged outside the pulled-back tree and copied in; the
manifest and packed README were edited before the final build and the gate confirmed `package
matches the tree`; R9 holds (SDK path dep only, no `[workspace]`, no packed lock); no local type or
variant is named `Block`/`Item`, so neither `fits` run false-failed. The full A–AH leg's late
teleporter phase hit two harness flakes (a missing placement face after the earlier breaks, then
a chat-reply race); `e2e-slice9.log` shows the same late-phase failure on 0.2.8, and the focused
teleporter leg is green on these bytes.
