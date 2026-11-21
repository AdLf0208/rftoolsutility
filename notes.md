# RFTools Utility — port notes (slices 1–2: the tank)

> **Current state (2026-11-21): slice 2, package 0.2.1 at pin
> `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` / mixins `e59da909…`.** Read
> **`## Slice 2 — the tank's contents travel with its item (2026-11-21)`** first; it is the
> current behaviour, its evidence and the four engine traps it cost. The 2026-11-18 note below is
> the format migration that produced 0.2.0, and everything after it describes the retired bundle.

> **Format (2026-11-18): the port is a flat SDK `.hmod`.** Horizon's mod-API merge (#16941,
> launcher ≥ 0.2.70) retired the `.hnmod` container; this port's submission (PR **#904**) was
> closed by the 2026-11-18 format purge, and the port is now one package —
> `horizon-mod.toml` + `Cargo.toml` + `src/lib.rs` (`impl mod_api::Hooks`) + `src/runtime.rs` +
> `assets/` + `data/` — at Horizon **`0b14f8acfd97fa2c750e42795249dcc981178ca4`** / horizon-mixins
> **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, mod API **`0.2`**, version **0.2.0**.
> Read **`## Migration to the SDK format`** first: it is the current state, with the helper→hook
> table, the traps and the shipped `.hmod`. Everything above it describes the retired bundle (pin
> `1d63e0a5`, `mixins/`, the staging ladder) and is kept as the record, not as instructions — none
> of `mixins/<id>/`, `compatibility.json`, `rust-toolchain.toml`, `tools/horizon.py`,
> `tools/package.py` or the two 0.1.0 packages exists any more.

Ported from **RFTools Utility 1.21-7.0.13** (`McJtyMods/RFToolsUtility`, **MIT**),
the exact ATM10 NeoForge 1.21.1 jar
`ports/_shared/licence-cache/rftoolsutility-1.21-7.0.13.jar` (SHA-256
`615642b184ba1b9228d7cd942a5d700228a47133419d61a2037b79bae618d3ff`, 1,051
entries). Upstream behaviour was read from branch `1.21_neo`, commit
`f799314007d68042df8504e83cd549360f2627c7` (`version=1.21-7.0.13`) — analysis
only, this is a clean-room rewrite. What was read and the numbers extracted:
[`FINDINGS.md`](FINDINGS.md).

**Licence: MIT → rank 0 (permissive).** Local-only until the maintainer approves
publishing (ARCHITECTURE.md §8). The jar's `neoforge.mods.toml` declares
`license="MIT License …/LICENCE"` and carries no licence file of its own, so the
text comes from that repository path (`Copyright (c) 2019 McJtyMods`), kept in
[`LICENSE`](LICENSE).

Pin **`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`**, mixins
**`5a64af938ec86b00d53d16a408aabc0038fb3e57`** (schema 6).

Status: **ported-partial** — slice 1 (the tank: the block, the item, the fluid
tank, the bucket loop, the fill level and the harness readout) and slice 2 (the
tank's contents travel with its item, and a doctored item is clamped on placement)
are implemented and verified in-game at the current pin; A–M of the e2e are green.
The gap list below is the rest of the mod.

## Class + tier and the approach

- **Class:** `content`; **tier:** `machines` (`ports/ledger.csv`,
  `ports/_shared/source-map/atm10-sources.csv`). `precheck-mod.py` also calls it
  an **add-on** (it declares `rftoolsbase` required); the parent is this
  campaign's own `rftoolsbase` port, `ported-partial`, so the job is workable.
- **Approach** (`horizon/docs/modpacks.md`, "What packs need and what runs
  today"): a **machines** shape — a block entity with a server tick, a fluid
  capability and the carried content. Not a data-only block.
- **Carry-over vs agent port:** the nine fill textures, the inventory model, the
  item model, the recipe, the loot table, the advancement and the lang lines are
  carried from the jar; the block entity, the fluid table, the tick, the bucket
  loop and the commands are the agent port ([`CONTENT.md`](CONTENT.md)).
- **The brief's constraint:** one machine whose loop closes **without a GUI**.
  The tank was chosen from the jar inventory over the crafters because its whole
  behaviour is a right-click with a bucket, and every number it uses is in the
  jar or its own source.

## What the slice does

- **The block.** `rftoolsutility:tank` on the **`minecraft:iron_block`**
  template: full cube, no template states, standard-iron properties (hardness
  2.0 / blast 2.0, correct tool, metal sound), plus a declared `level` state
  (0…8) that picks the jar's own fill texture.
- **The tank.** 32,000 mB (`TankConfiguration.MAXCAPACITY`'s default; the mod's
  manual says "up to 32 buckets"), one fluid at a time, declared through
  `block_entity_fluids` (API 54) with every face filling and draining — the same
  door pipes use. The contents live in the block entity's saved `fluids`
  compound.
- **The bucket loop.** `right_click_block` (API 21): a water or lava bucket pours
  exactly 1,000 mB in and the hand becomes an empty bucket; an empty bucket takes
  exactly 1,000 mB out and the hand becomes the filled one. A full tank, another
  fluid, or less than a bucket is refused without pouring anything.
- **The fill level.** The server tick reads the tank and writes the block's
  `level` state when it differs (`native_set_block_property`), so hand, pipe,
  save/load and command writes all show up.
- **The readout.** `/rftoolsutility info <pos> [nonce]` and
  `/rftoolsutility tank <pos> <fluid> <mb>`.

## Numbers and where they came from

| Number | Value | Source |
|---|---:|---|
| capacity | 32,000 mB | `TankConfiguration.MAXCAPACITY` (`defineInRange("maxCapacity", 32000, …)`) and the jar's own manual page |
| one bucket | 1,000 mB | vanilla/NeoForge: a bucket is 1,000 mB |
| fill level | `0` empty, else `min(8, 8*amount/32000 + 1)` | `TankTE.computeLevel` |
| fill textures | `tank0` … `tank8` | `TankBakedModel.getSideTexture` |
| tank faces | all six, fill and drain | `@Cap(type = CapType.FLUIDS)` on `TankTE.fluidHandler` |
| block properties | hardness 2.0, blast 2.0, correct tool, metal sound | McJtyLib `BlockBuilder.STANDARD_IRON`, the same row `rftoolsbase`'s machine carries |
| recipe | 3 glass / bucket + machine frame + bucket / 3 iron | `data/rftoolsutility/recipe/tank.json` |

## Hooks (the retired bundle, kept as history)

The SDK package's mapping is in `## Migration to the SDK format`; this table is what the 0.1.0
bundle injected, and every behaviour it carried is still implemented (through Parts and host
handles now).

| Hook (API) | Helper | What it does |
|---|---|---|
| `block_entity` (15) | `block_entity.rs` | the tank's block entity |
| `block_entity_ticks` (15) | `block_entity_ticks.rs` | ticks on the server |
| `block_entity_fluids` (54) | `block_entity_fluids.rs` | one 32,000 mB tank, every face in and out |
| `block_entity_tick` (17) | `block_entity_tick.rs` | mirrors the contents into the block's `level` |
| `right_click_block` (21) | `right_click_block.rs` | the bucket loop |
| `server_tick` (4) | `server_tick.rs` | re-sends a hand a bucket swap changed |
| `register_commands` (8) | `register_commands.rs` | `/rftoolsutility info|tank` |
| `perform_command` (8) | `perform_command.rs` | their bodies |
| `content_root` (raw target) | `embedded_content.rs` | writes the bundle's own content out and loads it |

9 injections (8 named hook points + one raw `content_root` target), 2 modules,
**0 unstamped**.

## Pin facts that shaped this slice

1. **A mod block entity needs no block-entity template since API 129**
   (`native_hooks.rs`: "before 129 the block's template had to have a block
   entity of its own"). The tank therefore sits on `minecraft:iron_block` — no
   template states and no menu on use — instead of a menued template whose own
   use would compete with the bucket click.
2. **`right_click_block` is the only bucket path for the world's own player.**
   `bucket_empty` (API 179) only reaches a LAN guest (`game.rs:7207`, "a game's
   own player is not asked"), the same finding `enderstorage` recorded; the pin
   also has no pre-block-use hook, so the click must be claimed in
   `right_click_block` and cancelled.
3. **A held-stack write does not reach the pin's integrated client on its own.**
   The swap is queued and re-sent one tick later through the hub's
   `send_inventory` (the `enderstorage` fix, reproduced): without it the player's
   client keeps showing the old bucket.
4. **The tank's contents are the engine's, in the block entity's saved
   compound** (`fluids`, API 54). The relaunch leg of the e2e is what proves
   persistence rather than a reading of the docs.
5. **The fill level is a block state, not a renderer.** The pin has no
   block-entity renderer in this slice, so the jar's nine side textures become
   nine block models and the level becomes a `level` property — the same data the
   jar's `TankTE.AMOUNT` fed its model loader.

## Deviations

- **A refused bucket click is cancelled, not passed on.** Upstream returns
  `PASS` when neither direction worked, which lets vanilla's bucket empty itself
  on the ground beside a full tank. The port claims every fluid-container click
  on the tank, so a full tank, another fluid or less than a bucket does nothing.
  A user notices: no water poured next to a full tank.
- **No GUI, no filter slot, no fluid quad inside the window.** The slice has no
  menu (`TankTE`'s `CapType.CONTAINER`), so a non-bucket click does nothing; the
  filter slot needs `native_block_entity_accepts_fluid`, which is not stamped at
  this pin; the inner fluid quad is a renderer gap (the nine side textures are
  carried).
- **The item does not carry the tank's contents.** Upstream moves
  `mcjtylib:item_fluids` between the block entity and its item (and the loot
  table copies it on break). The pin can express it — `loot_drops` (API 75) plus
  a component type and `place_block` — but it is slice-2 work, not an API gap.
- **The recipe needs the `rftoolsbase` port.** Its middle ingredient is
  `rftoolsbase:machine_frame`, which this port's content cannot declare (a
  content mod declares only its own namespace), so a game with only this port
  has the recipe loaded but uncraftable. The e2e gives the tank instead.

## Upstream defects fixed

None is *fixed* in this slice, because the one defect found lives in the filter
this slice does not implement. Recorded so slice 2 does not carry it:

- **The filter is lost on load.** `TankTE.loadAdditional` loads `items` but never
  calls `updateFilterFluid`; that only runs in the handler's `onUpdate`, so after
  a restart a tank with a water bucket in the filter slot accepts lava again
  until the slot is touched. Evidence: `TankTE.java`, `loadAdditional` (loads
  `items` only) against `GenericItemHandler.create(...).onUpdate((slot, stack) ->
  updateFilterFluid(stack))`. **What the port does instead:** nothing yet — there
  is no filter. **What a user would notice once it lands:** a filter that
  survives a restart. Worth a small upstream report; not filed (the orchestrator
  decides).
- Not a defect, recorded as a deviation instead: `loadAdditional` also reads the
  *render* level back into `amount`; the port derives the level from the tank
  every tick, so a stale saved level cannot survive.

## Gaps (the rest of the mod)

- **Tank:** the GUI (`GuiTank`), the filter slot and `isFluidValid`, the inner
  fluid quad, and the item's `Contents: …` tooltip. ~~Its carried fluid and the
  block's loot copy~~ — **done in slice 2**: the tank's saved data moves on the
  item as vanilla's `minecraft:block_entity_data` and comes back on placement.
- **Crafters** (`crafter1/2/3`, 2/4/8 recipes, chained crafting, the infusing
  power discount) and `creative_screen`.
- **Teleporters:** `charged_porter`, `advanced_charged_porter` (9 animation
  frames each), `teleport_probe`, `matter_transmitter/receiver/beamer/booster`,
  `dialing_device`, `simple_dialer`, `destination_analyzer`, and the
  `playerprops` teleport registry.
- **Screens:** `screen`, `screen_controller`, `screen_hitblock`, `tablet_screen`,
  `tablet_redstone`, `screen_link`, `storage_control_module` and the 30 screen
  modules.
- **Logic:** `wire`, `sensor`, `sequencer`, `timer`, `counter`, `analog`,
  `digit`, `logic`, `invchecker`, `redstone_transmitter/receiver`,
  `redstone_information`.
- **Environmental controller** and its 23 modules; the **spawner** with its 100
  mob recipes; `syringe`.
- **LAN guests:** the bucket swap is re-sent for the world's own player (the
  window client); a LAN guest's resend goes through the same hub record but is
  untested here.
- No config file (upstream's `maxCapacity` is baked at 32,000), no
  sounds/particles, no comparator output, no TOP/Jade integration.

## Verification (the retired bundle, 0.1.0 — history)

The current ladder is in `## Migration to the SDK format`; this section records what the retired
`.hnmod` proved, and the in-game phases are the behaviour evidence the SDK package keeps (the
scenario in `tools/e2e.sh` is the same one).

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices, rust-port

Every command, its output and the in-game phases are in
[`VALIDATION.md`](VALIDATION.md); the raw logs are in
`../_shared/evidence/rftoolsutility/`. In one line: 18 runtime tests red
(`todo!()`) → 18 green, 18 embedded (`game::rftoolsutility::tests`), fmt and
clippy clean, `check`/`build` `cargo_exit 0` with **0 warnings**, and the e2e in
The End passed with a control and a `SELFTEST=1` leg that fails as designed.

Two deliverables, both in the port at 0.1.0 (both retired; see the migration section):

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `content/rftoolsutility.hmod` | 17,525 B | `42f6030b…69a9` | 38 |
| `rftoolsutility.hnmod` (one file: code + content + notices) | 26,219 B | `98ad2799…b30d` | 16 |

Gates: `check-e2e-logs.py --strict` 3/3 exit 0; `check-skills.py --notes` PASS
(`SKILLS GATE: PASS`); `upstream_ready.py` exit 1 for two reasons that are not
this port's code — the ledger status is still `pending` (the orchestrator records
it) and the required parent `rftoolsbase` is not in the upstream registry yet, so
a submission would be closed exactly like `refinedstorage_quartz_arsenal` #498.

## Migration to the SDK format (2026-11-18, SDK `0b14f8a`)

Horizon's mod-API merge (#16941, launcher ≥ 0.2.70, PR #17992) retired `.hnmod`, and the registry
closed this port's submission — **PR #904** — with *"republish this as a native `.hmod` with the SDK
from the next Nightly"*. The port is now one flat package. Nothing about the tank's behaviour
changed: the pure logic is the same file, and each retired hook now has a typed replacement at the
same vanilla call site.

- **Pin:** Horizon **`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (nightly 2026-11-18), horizon-mixins
  **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, mod API **`0.2`**.
- **Version:** **0.2.0** (the retired bundle was 0.1.0; a resubmission is always a new version).
- **Class + tier** unchanged: class **content**, tier **machines** — a block entity with a fluid
  capability and a server tick, over the carried content.
- **`src/runtime.rs` is byte-identical** to the retired `mixins/rftoolsutility/runtime.rs`: sha256
  `4350997c1f8196a27ffefa1d2c2761083bf7f21eb545e96743c6c100c291755b`, 405 lines, `cmp` clean before
  the old tree was moved to evidence and again against the file **inside the built package**.

### File map

| Path | What it is |
|---|---|
| `horizon-mod.toml` | the only manifest and identity: `id = "rftoolsutility"`, `version = "0.2.0"`, `api = "0.2"`, `side = "both"`, `capabilities = ["registries"]`, `[native]` `game = "26.3"` + the two exact commits |
| `Cargo.toml` | one crate, version 0.2.0; the only dependency is `horizon-mod-api` from `.horizon/sdk/` |
| `src/lib.rs` | the one `#[mod_api::horizon_mod] impl Hooks` (`register`, `register_commands`, `run_mod_command`), `TankPart` (`BlockPart` + `BlockEntityPart`), the NBT↔[`runtime::Tank`] glue, the readout, and 7 glue tests |
| `src/runtime.rs` | the pure logic, byte-identical to the retired file, with its 18 tests |
| `assets/rftoolsutility/…` | the carried blockstate (9 levels), 9 block models + the inventory model, the item model, 9 fill textures + 3 machine-base textures, lang |
| `data/rftoolsutility/…`, `data/minecraft/…`, `data/c/…` | the block and creative-tab declarations, the loot table, the recipe and its advancement, the pickaxe/needs-iron tags, the `c:` ingredient tags |
| `rftoolsutility.hmod` | the delivered package (port root) |
| `LICENSE`, `LICENSE-NOTICE.md`, `README.md` | packed inside the package (the MIT notice condition) |
| `tools/e2e.sh`, `tools/parse_info.py` | the in-game scenario and its line parser (both format-independent; `--dev-mod` and the embedded-content legs are gone) |
| `tools/assemble_content.py` | regenerates `assets/` + `data/` from the two pack jars, deterministically |
| `.gitignore` | `/target`, `/.horizon`, `/Cargo.lock` |

Moved to `../_shared/evidence/rftoolsutility/retired-hnmod-0.1.0/`: `mixins/rftoolsutility/`
(`horizon-mixin.toml`, `runtime.rs`, `support.rs`, the seven helpers, the embedded content),
`rftoolsutility.hnmod` (`98ad2799…b30d`, 26,219 B), `content/rftoolsutility.hmod`
(`42f6030b…69a9`, 17,525 B), `content/horizon-mod.toml`, `compatibility.json`,
`rust-toolchain.toml`, the old `Cargo.toml`/`Cargo.lock`, and the staging/packaging tools
(`horizon.py`, `package.py`, `build.sh`, `ladder.sh`, `package.sh`, `evidence.sh`, `finish.sh`,
`hmod-load.sh`, `red-leg.sh`, `add-notices.py`).

### Helper → hook table

| Retired injection / helper | SDK 0.2 replacement | Where |
|---|---|---|
| `block_entity` (15) + `block_entity.rs` | `Registrar::block_entity_type("tank").blocks(&[TANK]).build()` + `block_entity_part`; a declared type starts with an empty compound, so no initial-data method is needed | `Mod::register` |
| `block_entity_ticks` (15) + `block_entity_ticks.rs` | `BlockEntityPart::block_entity_ticks` sets `event.ticks = true` | `TankPart` |
| `block_entity_tick` (17) + `block_entity_tick.rs` | `BlockEntityPart::block_entity_tick`: read the tank from `event.entity.nbt`, mirror it with `ServerHost::set_block_property`, answer `BlockEntityTickReply::Unchanged` (this part writes no payload) | `TankPart`, `mirror_level` |
| `block_entity_fluids` (54) + `block_entity_fluids.rs` | `Registrar::block_fluids(&block, ModFluidTanks { tanks: [32,000], faces: all six, inputs: [0], outputs: [0] })` | `Mod::register` |
| `right_click_block` (21) + `right_click_block.rs` | `BlockPart::use_item_on` on the tank; the runtime's `bucket_click` decides pass/refuse/move, `is_fluid_container` (the engine's `ITEM_FLUIDS` at `ItemPlace::PlayerHand`) catches a container the runtime does not model, and `fluid_util::interact_with_fluid_handler` performs the move (the engine's own `FluidUtil`, which swaps the hand and plays the sound itself) | `TankPart` |
| `server_tick` (4) + `server_tick.rs` (the one-tick-delayed hand resend) | **gone**: the SDK's fluid-container path writes the hand through `ItemPlace::PlayerHand`, which syncs, so the `Sync` state machine and the hub resend are deleted | — |
| `register_commands` (8) | `Hooks::register_commands` pushes `Command::new("rftoolsutility info <pos:block_pos> [nonce:word]")` and an op-level-2 `tank <pos:block_pos> <fluid:id> <mb:integer>` | `Mod::register_commands` |
| `perform_command` (8) + `perform_command.rs` | `Hooks::run_mod_command` with typed `CommandArgumentValue`s (`BlockPos`, `Id`, `Integer`, `String`), feedback through `host.commands().feedback` | `Mod::run_mod_command` |
| `content_root` + `embedded_content.rs` (the AE2 pattern) | **gone**: the flat package carries `assets/` and `data/` itself | — |
| `support.rs` static `SYNC` + `Game` calls | host handles: `PlayersHost::player`, `PlayerRef::level`, `BlockEntitiesHost::block_entity`, `BlockEntityRef::data`/`replace_data`, `BlockStatesServerHost::set_block_property`, `LivingHost::living` + `LivingRef::hand_item`, `LevelRef::block`/`game_time`, `LevelsHost::list_levels` | `src/lib.rs` |

The runtime is untouched: `CAPACITY`, `BUCKET`, `MAX_LEVEL`, `fill_level`, `Tank`,
`held_fluid`, `fluid_container` and `bucket_click` are called exactly as before. The new glue only
converts between the SDK's types and the runtime's (`fluids` NBT ↔ `runtime::Tank`, `u8` level ↔
the string property, `ItemStack` ↔ `(id, count)`).

Skills: superpowers/using-superpowers, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

### Build and check (verbatim)

`build-slot.sh` speaks the old staging tool, so each command runs **directly in the port** (inside
a lease, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no `--target-dir`):

```sh
BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sdk-rfu --port ports/rftoolsutility --step test \
    -- bash -c 'cd ports/rftoolsutility && CARGO_INCREMENTAL=0 cargo test --offline'
# → test result: ok. 25 passed; 0 failed   (18 runtime + 7 glue)

BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sdk-rfu --port ports/rftoolsutility --step lint \
    -- bash -c 'cd ports/rftoolsutility && cargo fmt --check &&
        CARGO_INCREMENTAL=0 cargo clippy --all-targets --offline -- -D warnings'
# → Finished `dev` profile … (0 warnings, 0 errors)

BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sdk-rfu --port ports/rftoolsutility --step build \
    -- bash -c 'cd ports/rftoolsutility && rm -f Cargo.lock && CARGO_INCREMENTAL=0 cargo horizon-mod build'
# → SDK 0b14f8a (build 0b14f8acfd97fa2c750e42795249dcc981178ca4) in ./.horizon/sdk
# → Built ./target/horizon-mod/rftoolsutility.hmod

BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sdk-rfu --port ports/rftoolsutility --step check \
    -- bash -c 'cd ports/rftoolsutility && cargo horizon-mod check rftoolsutility.hmod;
        cargo horizon-mod fits rftoolsutility.hmod --build nightly'
# → rftoolsutility 0.2.0 (native-mod, format native-1/0b14f8ac…/e59da909…, for 26.3)
# → 37485 bytes, SHA-256 7121e48f…, unsigned                 CHECK_EXIT:0
# → RFTools Utility: / 0b14f8a  made for                     FITS_EXIT:0
```

The package is **37,485 B**, SHA-256 `7121e48f95642d568cd0561181c9cafbd4d458eb94fa4f5d0febdb1089710a5a`,
43 entries (manifest, `Cargo.toml`, `LICENSE`, `LICENSE-NOTICE.md`, `README.md`, `src/lib.rs`,
`src/runtime.rs`, 35 `assets/`+`data/` files, `uses.json`), **no `Cargo.lock`**, and **built twice
with identical bytes**. Its `uses.json` (309 entries) names `hook:use_item_on`,
`hook:block_entity_tick`, `hook:block_entity_ticks`, `hook:register_commands`, `hook:run_mod_command`
and the `BlockEntityRef`/`Registrar` handles the glue uses, so the checker read the Parts as intended.

TDD: the four pure glue helpers (`tank_of`, `write_tank`, `readout`, `answer`) were written as
`todo!()` stubs behind 6 tests first — `test result: FAILED. 19 passed; 6 failed`, each at
`not yet implemented` — then implemented (plus a 7th test for the level string the tick writes).
The byte-identical runtime's 18 tests were green **before any glue existed**.

### Traps this migration cost

1. **`--dev-mod <port root>` would run the content without the code.** The tank's block entity and
   its tank live in the native half, so a dev-loaded folder (or the old `content/` path) shows a
   tank that holds nothing. `tools/e2e.sh` now takes `GAME=` from a launcher-built game and no
   `--dev-mod`; the header says so.
2. **`build-slot.sh` does not `cd` into the port.** `--port` is a label; wrap every command as
   `bash -c 'cd ports/<id> && …'` or cargo cannot find the manifest.
3. **The port must not name a `rust-version`/toolchain.** The scaffold declares none; the pinned
   commit (rust 1.99.1) decides. The retired `rust-toolchain.toml` (1.95.0) is gone with the bundle.
4. **No packed `Cargo.lock`.** `cargo test`/`clippy` write one; packed, it changes the artifact
   (imfast: 7,570 B → 23,133 B) and pins versions the SDK was not built against. `rm -f Cargo.lock`
   immediately before `cargo horizon-mod build`, and `/Cargo.lock` in `.gitignore`.
5. **Host *writes* are handle methods, not host-area methods.** `set_block_property` is a
   `ServerHost` area method, but `game_time` is on the **handle** (`LevelRef::game_time(host)`) —
   `host.level_game_time` does not exist (it is a backend trait's method, not the host's).
6. **`Compound` is `BTreeMap<JavaString, Tag>` and `JavaString` is not exported.** Read a key by
   passing the payload of `Tag::String(name.into())` to `map.get`, as `rangedpumps` does; or go
   through `Tag::get(&str)` on a `Tag::Compound`.
7. **`block_entity_tick` answers `Changed` by default.** A tick that writes no payload (this one
   writes only a block property) must set `BlockEntityTickReply::Unchanged`, or every tick dirties
   the chunk and bumps its revision.
8. **A block entity is out of the world during its own tick.** Read the tank from
   `event.entity.nbt`, never through the fluid capability (`native_fluid_tanks` before the
   migration, the same rule now); for commands, `BlockEntityRef::data`.
9. **`#[non_exhaustive]` events/enums need a wildcard.** `match` on `CommandArgumentValue` and on
   the literal path keeps a `_ =>` arm; `InteractionResult` is `#[non_exhaustive]` too, so only
   construct the variants, never match exhaustively on it.
10. **`cargo fmt --all --check` reports the SDK's files** (`.horizon/sdk` is a path dependency):
    use `cargo fmt --check`.
11. **The tool packs; it does not compile against the game.** `check`/`fits` validate the package,
    `clippy`/`cargo test` compile the glue against the pinned SDK API crate; the only real native
    build is `horizon-launcher native-build`, which this box does not have (see the gaps).
12. **`cargo horizon-mod check` with no file checks wasm** (`can't find crate for 'core'`): always
    pass the built package.

### Gaps after the migration

1. ~~**No in-game run at `0b14f8a`.**~~ **Closed 2026-11-21**: the launcher builds the mod at the
   current pin `0a875da` and `tools/e2e.sh` ran in game (A–M green); see `## Slice 2` and
   `../_shared/evidence/rftoolsutility-e2e/`. (History:) `horizon-launcher` was not installed and there was no game build
   at this pin (the shared binaries are at `1d63e0a5`/`41f822b6`), so the behaviour evidence is the
   0.1.0 e2e (`../_shared/evidence/rftoolsutility/e2e.log`, `e2e-selftest.log`,
   `check-e2e-logs.py --strict` 3/3) plus the *mapping* proved here: each retired hook to a typed
   Part/host call at the same vanilla site, the type-checked glue and the carried rules. The next
   slice is "build the game with the launcher and run `tools/e2e.sh` with `GAME=`".
2. ~~**The tank's contents are not on its item, and the block does not copy them out**~~
   **Closed 2026-11-21 by slice 2**: `block_drops` puts the tank's saved data on its own item as
   `minecraft:block_entity_data`, and vanilla placement restores it (the item's tooltip is still
   a gap — see `## Slice 2`).
3. **The GUI, the filter slot and the inner fluid quad** (inherited): the filter needs an
   accepts-fluid decision on the tank; the quad is a renderer.
4. **No config file** (upstream's `maxCapacity` is baked at 32,000), no comparator output, no
   TOP/Jade integration, no sounds of its own (the engine's bucket sound now plays, which the
   retired bundle did not).
5. **A LAN guest's hand swap** is untested here: the engine's `ItemPlace::PlayerHand` write is the
   supported path for every player (the retired bundle only re-sent the window client's).
6. **The `t=` field of the readout is the level's game time** (`LevelRef::game_time`); it is
   informational — the harness matches its reply by the `nonce`, as before.

### Changelog for the resubmission

> **Supersedes #904**, closed because Horizon retired `.hnmod` (#17992). Changes: one flat SDK
> package with `api = "0.2"` and `[native]` pinned to `0b14f8a` / `e59da90`; the code is now typed
> `impl mod_api::Hooks` methods and Parts — a `BlockEntityPart` on `rftoolsutility:tank` (the block
> entity, its server tick and its one 32,000 mB tank through `Registrar::block_fluids`) and a
> `BlockPart::use_item_on` that hands a bucket to the engine's own fluid-container path instead of
> a hand-rolled swap plus a one-tick resend; the content ships inside the package (`assets/`,
> `data/`) so the embedded-content injection is gone; `src/runtime.rs` and its 18 tests are
> unchanged. Version 0.2.0.

## Slice 2 — the tank's contents travel with its item (2026-11-21)

**Chosen from the port's own gap list** — `## Gaps after the migration` item 2, verbatim:

> **The tank's contents are not on its item, and the block does not copy them out** (inherited):
> `loot_drops` plus an item component would express it; slice-2 work, not an API gap.

and `FINDINGS.md` §2.6 ("The item carries the contents … so a broken tank keeps its fluid"). It
fits the **machines** tier: a block entity's saved data, an item component and the carried
content — no renderer, no new family.

**Pin:** Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, mod API `0.2`, game 26.3. **Version 0.2.1.**

**Read this slice** (analysis only; clone deleted after): `McJtyMods/RFToolsUtility` branch
`1.21_neo`, commit `f799314007d68042df8504e83cd549360f2627c7` —
`modules/tank/blocks/TankTE.java` (`collectImplicitComponents`/`applyImplicitComponents`,
`getFluidString`'s tooltip, the filter), `modules/crafter/*` (checked for the next slice's shape,
not ported) and `data/rftoolsutility/loot_table/blocks/tank.json`.

### What it does

- **Break:** `Hooks::block_drops` (server, `BlockDropsEvent`) reads the tank's own saved compound
  (`level.block_entity_data(entity)` — the data a player's break keeps for the event) and, when
  the tank holds anything (`runtime::carried`), puts it on the tank's own drop as
  `minecraft:block_entity_data` tagged `id = "rftoolsutility:tank"` (`TypedEntityData`, which
  vanilla's placement requires). This is upstream's `collectImplicitComponents` moving the fluid
  handler onto the item, and its loot table's `copy_components`.
- **Place:** vanilla's `BlockItem.updateCustomBlockEntityTag` applies that component to the new
  block entity, so the contents come back with no code. `TankItemPart::on_block_placed_by`
  (`ItemPart`) clamps the result to the capacity — upstream's `FluidTank.setFluid` clamp — and
  mirrors the fill level at once, so a doctored item cannot place a 99,999 mB tank.
- **An empty tank's item carries nothing** (`carried` is `None`), so a broken empty tank is a
  plain tank item.

`src/runtime.rs` gained `carried` and `clamped` (pure, tested) and its `insert`/`extract` are now
the glue's writers as well as the engine model.

### Red → green (unit, verbatim)

The three new rules were stubbed `todo!()` behind their tests first:

```
test result: FAILED. 25 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out
  a_broken_tank_carries_exactly_what_it_holds      panicked at src/runtime.rs:204:5: not yet implemented
  an_empty_tank_carries_nothing_on_its_item        panicked at src/runtime.rs:204:5: not yet implemented
  a_carried_tank_is_restored_at_most_the_capacity  panicked at src/runtime.rs:212:5: not yet implemented
```

Green: **31 passed; 0 failed** (21 runtime + 10 glue). The glue tests pin the payload
(`the_carried_payload_is_the_vanilla_block_entity_data`: the component's `id` plus the tank's own
`fluids` shape), the empty case, the container the click names, and the harness command's clamp.

### In game (evidence under `../_shared/evidence/rftoolsutility-e2e/`)

The scenario is `tools/e2e.sh`; the four legs are one lease each, and every log ends with its exit.
The package gates' logs are `../_shared/evidence/rftoolsutility/sdk-{test,clippy,check,fits,fits-other}.log`,
each printing the `rftoolsutility.hmod` sha256 and size it ran against; the 0.2.0/0.1.0 logs were
moved to `../_shared/evidence/rftoolsutility/retired-0.2.0/`.

| log | verdict |
|---|---|
| `e2e-slice2.log` | **PASS**, `EXIT:0` — phases A–M |
| `e2e-slice2-selftest-restore.log` | **FAIL: the tank holds 5000 mB, expected 0**, `EXIT:1` |
| `e2e-slice2-selftest-fill.log` | **FAIL: the tank holds 1000 mB, expected 0**, `EXIT:1` |
| `e2e-slice2-control.log` | **FAIL: the tank item did not register**, `EXIT:1` (the unmodded Nightly) |

`check-e2e-logs.py rftoolsutility-e2e --strict` → `logs=5 ok=5 missing-exit=0 bad-exit=0`.

The new phases, each asserting a parsed value:

| phase | what it asserts (observed) |
|---|---|
| **J** | a survival break of a 5,000 mB water tank drops a tank whose in-hand `SelectedItem` has `minecraft:block_entity_data` `{id: "rftoolsutility:tank", fluids: [{Amount: 5000, Fluid: "minecraft:water"}]}` |
| **K** | that item, placed again, is `minecraft:water 5000/32000` at block `level=2` |
| **L** | an **empty** tank's item carries **no** block-entity data (`component=absent`) and places back empty — the same three assertions inverted |
| **M** | an item doctored to 99,999 mB (`give … [minecraft:block_entity_data={…Amount:99999}]`) places a tank clamped to `32000/32000`, level 8 |

Counterfactual: `SELFTEST_AT=restore` inverts K's assertion and the leg fails as designed, so K
can fail. J is the component read-back the parity rule asks for; L is its differential.

### Four engine traps this slice cost (measured, not read)

1. **The engine's own `FluidUtil` path at this pin does not fill a container.** A tank with
   1,000 mB drains into an empty bucket (the tank empties) but the hand stays an empty bucket —
   the fluid is lost; a water cauldron behaves the same way (`use_on_cauldron` is not wired into
   the use path at 0b14f8a). The port therefore performs the bucket move itself from the
   runtime's numbers (`bucket_click` + `insert`/`extract`) and keeps `fluid_util` only for a
   container the runtime does not model.
2. **`ItemPlace::PlayerHand.exchange` and `player.set_inventory_slot` are undone when the click's
   own transaction ends** — the write reads back correctly inside the part and is gone two ticks
   later. The hand swap is queued and applied in `server_tick` (the retired bundle's one-tick
   resend), one entry per hand, dropped when the hand no longer holds the item the click was made
   with.
3. **The item's own `useOn` still runs after a block part answered**, so a refused click (or a
   fill) would also empty the bucket on the ground beside the tank. The click is taken in
   `Hooks::use_item_on_block` at `UsePhase::Block` (cancel + `Success`), and on the client the same
   hook cancels a bucket click so the integrated client predicts nothing.
4. **Creative keeps a container stack** (`ItemPlace`'s creative rule): the 0.1.0 e2e's bucket
   phases only hold in survival, and a survival break is required for any drop at all — the e2e
   switches to survival after the arena. The port's own hand write swaps in both modes, as
   upstream's `setItemInHand` does.
5. **`<fluid:id>` is delivered as `CommandArgumentValue::String`, not `Id`** (`id` is a free
   identifier; `resource:<registry>` delivers `Id`) — the harness command's `tank` usage was
   silently unparsable since 0.2.0. Fixed by accepting either, and pinned by
   `the_tank_command_replaces_and_clamps`.

### Upstream defects fixed / deviations

- **No new upstream defect** is adopted: upstream's own `setItemInHand` swap is what the port now
  does, so slice 1's creative-mode divergence is gone.
- **Deviation kept:** a refused container click is *consumed and does nothing* (upstream answers
  `PASS`, which empties the bucket beside the tank). A user notices: no water poured next to a
  full tank.
- **Deviation recorded, not ported:** the item's `Contents: <amount>mb <fluid> (32000 mb)`
  **tooltip**. The component is carried and read back (J), but `horizon-ctl` has no tooltip
  readout and a headless leg cannot assert a client-drawn line, so shipping a tooltip here would
  be an unprovable claim. The tank's own `level` state is the visible half.

### Coverage of the tank family (after slice 2)

| family | state |
|---|---|
| block, item, creative tab, tags, recipe, advancement, loot table | tested (A: place/read-back; the recipe itself needs `rftoolsbase:machine_frame`, see below) |
| the 32,000 mB one-fluid tank + fluid capability | tested (A–G, I) |
| bucket click (fill/drain/refuse), capacity, levels | tested (B–G, I) |
| save/load of the tank | tested (I, relaunch) |
| contents on the item, loot copy, placement restore | tested (J, K, L) |
| capacity clamp on a carried item | tested (M) |
| item tooltip `Contents: …` | not ported (no `ctl` tooltip readout; would be unprovable) |
| GUI (`GuiTank`), filter slot + `isFluidValid` | not ported (needs a menu; the filter needs `native_block_entity_accepts_fluid`, not stamped) |
| inner fluid quad | not ported (a renderer — outside the machines tier) |
| LAN guest's hand swap | not ported as its own leg (the engine's inventory sync is the only path; untested) |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

### Artifact

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `rftoolsutility.hmod` (port root) | 43,884 B | `3e00917a88b3d284c2933c3a3a2a98b170a226bea8ed10970880eed3f6d07e3f` | 43 |

Built twice with `rm -f Cargo.lock` before each (`cmp` identical), **no packed `Cargo.lock`**,
`check` green (`native-1/0a875da…/e59da909…, for 26.3`), `fits --build 0a875da` → `made for`,
`fits --build 0b14f8a` → `fits`, `verify-sdk-package.py rftoolsutility` → **1/1**.
