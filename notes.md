# RFTools Utility — port notes (slices 1–6: the tank, the crafter tiers and the spawner)

> **Current state (2026-11-21): slice 6, package 0.2.5 at pin
> `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` / mixins `e59da909…`.** Read
> **`## Slice 6 — the tick's own dimension (2026-11-21)`** first: it fixes the correctness bug a
> reviewer found on PR #1643 (a tick resolved its level by scanning for a matching block, so two
> dimensions holding the same machine at the same position confused it), with its red, its fix and
> its two-dimension evidence. Then **`## Slice 5 — the crafter tiers (2026-11-21)`** (the crafter
> family), `## Slice 4 — the spawner`, `## Slice 3 — the crafter`, `## Slice 2` (the tank's
> contents on its item) and the 2026-11-18 migration note that produced 0.2.0; everything after the
> migration describes the retired `.hnmod` bundle.

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
- **Crafters** — **the whole family is done**: `crafter1` in slice 3, `crafter2`/`crafter3` (4
  and 8 recipe slots, the same block entity) in slice 5. Remaining there: the GUI and its
  ghost-grid editor, the filter module slot, the infusing power discount and `ExtC`, the
  tier-upgrade's component copy (`mcjtylib:copy_components`), and `creative_screen`.
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
- **Environmental controller** and its 23 modules; ~~the **spawner** with its 100
  mob recipes~~ — **the spawner is done in slice 4** (the block, its 200,000 FE buffer, the
  mob setting from the recipe table and the spawn tick, with the jar's **70** mob recipes
  carried — the jar ships 70, not 100: `unzip -l | grep -c 'recipe/minecraft_'`). Remaining
  there: the three matter types and the matter beamers that fill them, the syringe that
  sets the mob in upstream's GUI, the GUI, and six-way facing; `syringe`.
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

## Slice 3 — the crafter (2026-11-21)

**Chosen from the port's own gap list** — `## Gaps (the rest of the mod)`, verbatim:

> **Crafters** (`crafter1/2/3`, 2/4/8 recipes, chained crafting, the infusing power discount) and
> `creative_screen`.

and `FINDINGS.md` §3 ("This is a machines mod with several families … four single machines:
`tank`, `crafter1/2/3`, `environmental_controller`, `creative_screen`"). It is the largest
remaining **machines** family after the tank: a block entity with a 41-slot container, a 50,000 FE
buffer and a server tick that runs crafting recipes — no renderer, no dimension, no client half.
Its whole rule is readable through `horizon-ctl` (`/rftoolsutility crafter` prints the energy,
the taught recipes, the input buffer and the output buffer), so it needs no screen to prove.

**Pin:** Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, mod API `0.2`, game 26.3. **Version 0.2.2.**

**Read for this slice** (analysis only; the clone at `/tmp/opencode/rftoolsutility-src`, branch
`1.21_neo`, commit `f799314007d68042df8504e83cd549360f2627c7`): `modules/crafter/data/CraftingRecipe.java`
(the 3x3 grid, `findRecipe`, `getRemainingItems`, `CraftMode`/`KeepMode`/`SpeedMode`),
`modules/crafter/blocks/CrafterBaseTE.java` (`tickServer`, `craftOneCycle`, `craftOneItem`,
`testAndConsume`, `placeResult`, `setSelected`), `CrafterContainer.java` (the slot layout),
`CrafterConfiguration.java` and `CrafterModule.java` (registration, block properties, recipe).

### What it does

- **The block.** `rftoolsutility:crafter1` on the **`minecraft:white_glazed_terracotta`**
  template (a full cube with a four-way `facing` placement rule, no block entity of its own, no
  menu), with the jar's own front texture on its north face and the machine base top/bottom/sides.
- **The container.** 41 slots, upstream's own layout: 10 ghost slots (the 3x3 recipe grid and its
  result, slots 0–9), 26 input buffer slots (10–35), 4 output buffer slots (36–39) and the filter
  module slot (40). Hoppers reach the 26 input slots (insert) and the 4 output slots (extract) on
  every face; the ghost slots and the filter slot are outside automation, as in
  `CrafterContainer`'s `ghost()` slots.
- **The energy.** 50,000 FE (`crafterMaxRF`), a fill of at most 500 a call (`crafterRFPerTick`) on
  every face, in the engine's `energy` int.
- **The craft loop.** Each server tick runs `steps(speed, energy)` **cycles**; a cycle walks the
  taught recipes in order and crafts one of each that the buffer can supply
  (`craftOneCycle`). One cycle costs 100 FE (`rfPerOperation`) whatever it crafted, so a log
  becomes four planks and then a crafting table for 100 FE in `int` mode — upstream's chained
  crafting. Slow mode runs 1 cycle a tick, fast mode 5.
- **The recipe matching.** A taught recipe is the engine's own crafting recipe: the harness names
  it (`/rftoolsutility recipe <pos> <index> <id>`) and the port resolves its 3x3 grid from
  `Host::recipes_of_type("minecraft:crafting")` — the pattern row by row, each cell's expanded
  ingredient ids (so a `#minecraft:planks` cell accepts any plank) — writes one item per cell into
  the ghost slots, and **refuses** a grid the engine does not match back to the same recipe.
  Each operation then takes one item per cell out of the input buffer (one slot may feed several
  cells, as upstream's `split(1)` loop does) and lets `LevelRef::recipe_for` assemble the work
  grid, which is what makes tags, remainders and dynamic outputs the engine's own.
- **Where results go.** `Ext` (default): the output buffer, the four slots automation takes from.
  `Int`: the input buffer, where the next recipe of the same cycle can use it — the chaining.
- **Keep-one.** `Keep` leaves the last item of each slot it takes from (`KeepMode.KEEP`).
- **The item carries the machine.** A broken crafter's own item takes the block entity's saved
  compound as vanilla's `minecraft:block_entity_data` (upstream's
  `collectImplicitComponents`), so a placed item brings back the taught recipes, the buffer and
  the energy.
- **The readout.** `/rftoolsutility crafter <pos> [nonce]` prints
  `energy=<held>/<cap> speed= mode= keep= recipes= in= out=`; `tools/parse_crafter.py` parses it.

### Numbers and where they came from

| Number | Value | Source |
|---|---:|---|
| energy buffer | 50,000 FE | `CrafterConfiguration.MAXENERGY` (`defineInRange("crafterMaxRF", 50000, …)`) |
| fill per call | 500 FE | `CrafterConfiguration.RECEIVEPERTICK` (`defineInRange("crafterRFPerTick", 500, …)`) |
| one cycle | 100 FE | `CrafterConfiguration.rfPerOperation` (`defineInRange("rfPerOperation", 100, …)`) |
| fast mode | 5 cycles a tick | `CrafterConfiguration.speedOperations` (`defineInRange("speedOperations", 5, …)`) |
| recipes per crafter | 2 (tier 1) | `CrafterBaseTE.createTier1` (`supportedRecipes = 2`) |
| slots | 10 ghost + 26 in + 4 out + 1 filter = 41 | `CrafterContainer.CONTAINER_FACTORY` (`10 + BUFFER_SIZE + BUFFEROUT_SIZE + 1`) |
| block properties | hardness 2.0, blast 2.0, correct tool, metal sound | `BlockBuilder.STANDARD_IRON` + `ironPickaxeTags()`, the same row the tank carries |
| recipe | redstone torch / crafting table + machine frame + crafting table / redstone torch | `data/rftoolsutility/recipe/crafter1.json` (carried) |
| facing | four horizontal directions | `orientedBlock` in `CrafterModule`'s datagen; see the content adaptation below |

### Red → green (unit, verbatim)

The new family's rules were written as tests over `todo!()` bodies first:

```
test result: FAILED. 31 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out
  runtime::tests::a_cell_accepts_any_item_of_its_ingredient            panicked at src/runtime.rs:317:5: not yet implemented
  runtime::tests::a_crafter_burns_one_operation_in_slow_mode_and_five_in_fast ...
  runtime::tests::a_full_output_buffer_refuses_the_operation ...
  runtime::tests::a_slot_feeds_one_cell_only ...
  runtime::tests::an_empty_grid_has_no_operation ...
  runtime::tests::energy_pays_for_no_more_operations_than_it_covers ...
  runtime::tests::keeping_one_leaves_the_last_item_behind ...
  runtime::tests::one_log_crafts_from_the_buffer ...
  runtime::tests::the_output_merges_up_to_the_stack_limit ...
```

(the log is `../_shared/evidence/rftoolsutility/slice3-red.log`; a second red leg added the craft
modes: `FAILED. 40 passed; 2 failed` at `result_range`, `slice3-red-craftmode.log`). Green:
**48 passed; 0 failed** (21 tank runtime + 12 crafter runtime + 10 tank glue + 5 crafter glue),
`slice3-green.log`. Two rules were corrected *after* meeting the game, each red first:

1. **One slot feeds several cells** (`one_slot_can_feed_several_cells`). Upstream's
   `testAndConsume` splits one item per cell out of a slot while it still holds more than `keep`,
   so four planks in one slot feed a 2x2 recipe; the first implementation marked a slot used once
   and would have refused every chained craft.
2. **A cycle costs 100 FE whatever it crafted** (`spend`: upstream's `rf * i` multiplies the
   *cycles*, not the crafts).

### In game (evidence under `../_shared/evidence/rftoolsutility-e2e/`)

`tools/e2e.sh` phases A–U, one lease per leg, every log ends with its exit:

| log | verdict |
|---|---|
| `e2e-slice3.log` | **PASS**, `EXIT:0` — A–V |
| `e2e-slice3-selftest-craft.log` | **FAIL: the crafter holds 4700 FE, expected 5000**, `EXIT:1` |
| `e2e-slice3-control.log` | **FAIL: the tank item did not register**, `EXIT:1` (the unmodded Nightly) |

`check-e2e-logs.py rftoolsutility-e2e --strict` → `logs=8 ok=8 missing-exit=0 bad-exit=0`.

The slice's phases, each asserting a parsed value:

| phase | what it asserts (observed) |
|---|---|
| **N** | a crafter placed from its item reads `energy=0/50000 … recipes=- in=- out=-` and has the template's `facing` state (`crafter1[facing=west]`) |
| **O** | `recipe … 0 minecraft:oak_planks` answers `crafts minecraft:oak_planks` and the readout lists it; a **fifth** slot answers `this crafter has 2 recipe slots` and a **smelting** id answers `no crafting recipe`, with the taught list unchanged |
| **P** | with 0 FE and three logs nothing is crafted (`in=10=minecraft:oak_logx3`, `out=-`) |
| **Q** | 5,000 FE + three logs → `in=-`, `out=36=minecraft:oak_planksx12`, `energy=4700` (three 100 FE cycles) |
| **R** | three cobblestones stay put and cost nothing (`energy=1000` unchanged) |
| **S** | four full output slots refuse the operation: two logs stay in the buffer, `energy=1000` unchanged (nothing consumed, nothing spent) |
| **T** | `mode int`, `speed slow`: four oak logs become four crafting tables **inside the machine** (`in=12=minecraft:crafting_tablex4`), `out=-`, `energy=4600` (four cycles — the planks were never inserted) |
| **U** | a survival break drops a crafter whose item carries `minecraft:block_entity_data`; placing it back restores both taught recipes, 4,600 FE and the four tables |
| **V** | the loot table's `minecraft:needs_iron_tool` gate: a **wooden** pickaxe breaks the crafter and drops nothing, an **iron** pickaxe breaks the same block and the item comes back (the control) |

Counterfactuals: `SELFTEST_AT=craft` inverts Q (the machine must have made nothing and spent
nothing) and the leg fails as designed, so Q can fail; `CONTROL=1` runs the same script on the
unmodded Nightly, where the package's items do not exist; and V is itself a differential (the
wooden and the iron pickaxe on the same block).

### Measured traps this slice cost (engine and harness)

1. **A slice's slot numbers are relative to the slice the runtime was given.** `plan` numbers the
   input buffer 0..25 and `placements` numbers the output slice 0..3; the glue used them as world
   slots, so the work grid was read out of the *ghost* slots and the planks were written into
   slot 0..3 — the machine crafted (the ghost cell happened to hold a log) and its product
   vanished. Both helpers now document "an index into the slice" and the glue adds its base.
2. **`ctl messages` lists the newest chat line first**, so a reply picker must take the *first*
   match, not the last; and a shell wrapper that hard-codes five positional parameters silently
   drops the sixth (`buffer <pos> <slot> <item> <count>`), which parses as "incomplete command"
   with the error caret at the item.
3. **`ctl place` can be refused by an entity standing on the target** ("the block did not change
   (blocked by an entity, or the item cannot be placed)"). The tank's doctored-item phase hit it
   once in six runs; `try_place` retries the click twice, and still asserts the block that ends
   up there.
4. **A command's feedback does not reach the chat if the command never parses** — which is why
   every control phase asserts the *readout* after it (a nonce-bearing line), not the control's
   own answer alone.

### Deviations and content adaptations

- **Four-way facing, not six.** Upstream's `orientedBlock` faces six directions; the content
  schema takes a block's states and its placement rule from its vanilla template, and the
  full-cube, block-entity-free, no-menu template whose horizontal `facing` placement is
  *measured* on a content block is `minecraft:white_glazed_terracotta` (bibliocraft's four
  blocks). A user notices: a crafter cannot be turned to face up or down.
- **No GUI, so the ghost grid is written by the harness.** Upstream's player teaches a recipe in
  the GUI's ghost slots and picks the active recipe with a button; this slice has no menu.
  `/rftoolsutility recipe` writes the same grid from the engine's recipe table, and both recipe
  slots can be taught, which is the tier-1 mechanic ("up to 2 recipes, chained"). A player cannot
  teach a recipe yet — the gap below.
- **`ExtC` (upstream's third craft mode) is not ported.** `Ext` puts the result and the
  remainders in the output buffer, `Int` puts both in the input buffer; `ExtC` differs only in
  sending *remainders* to the input buffer while the result goes out, and it is a GUI choice.
- **The filter module slot exists but is unusable.** `CrafterContainer.SLOT_FILTER_MODULE` takes
  only `rftoolsbase:filter_module`, an item this port does not ship; the slot is declared (41
  slots, upstream's layout) and outside automation, and nothing reads it.
- **No tooltip, no comparator, no TOP/Jade, no sounds of its own** — the same gaps the tank
  carries; `horizon-ctl` has no tooltip readout to prove `Contents:`/`Recipes:` lines with.
- **The infusing power discount is not ported.** Upstream's crafter is infusable
  (`rf * (2 - infused/256) / 2`); that needs `rftoolsbase`'s infusable item path (the
  `rftoolsbase:infused` component and `DefaultInfusable`), which this port does not carry.
- **Remaining crafter content:** ~~`crafter2` and `crafter3`~~ **— done in slice 5** (the same
  block entity with 4 and 8 recipe slots, which differ only in `supportedRecipes` and their
  carried content). What is still open: the GUI; the
  filter module; and the tier-specific recipes (`crafter2` needs a `crafter1`).

### Coverage of the crafter family (after slice 3)

| family | state |
|---|---|
| `crafter1` block, item, facing state, creative tab | tested (N: place, read back, the `facing` state) |
| `minecraft:mineable/pickaxe` + `needs_iron_tool` tags, and the loot table | tested (V: a wooden pickaxe drops nothing, an iron one drops the block; U reads the drop) |
| the carried recipe and its advancement | not crafted in game: the recipe's middle ingredient is `rftoolsbase:machine_frame`, which only the parent port registers (the tank's own gap); the e2e gives the block instead |
| the 50,000 FE buffer and its 500-a-call fill | tested (Q, T, U: the energy read back after every cycle) |
| taught recipe from the engine's table, grid validation, two slots | tested (O) |
| the craft loop: consume per cell, assemble, place, merge | tested (Q, T) |
| chained crafting (`mode int`) | tested (T) |
| `keep` mode | unit-tested (`keeping_one_leaves_the_last_item_behind`); no harness phase (a GUI setting) |
| a refused operation (wrong input, full output) | tested (R, S) |
| contents, energy and recipes on the item, restored on placement | tested (U) |
| `crafter2`, `crafter3` (4 and 8 recipes) | tested (slice 5: AC, AD, AE — see `## Slice 5`) |
| the GUI (`GuiCrafter`), the ghost-grid editor, the active-recipe button | not ported (no menu in this slice; the harness writes the grid) |
| the filter module slot and `FilterModuleItem` | not ported (needs `rftoolsbase:filter_module`) |
| the infusing power discount | not ported (needs `rftoolsbase`'s infusable path) |
| `ExtC` craft mode | not ported (a GUI-only third mode) |
| the item's `Contents:`/`Recipes:` tooltip | not ported (no `ctl` tooltip readout; would be unprovable) |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

### Artifact

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `rftoolsutility.hmod` (port root) | 63,478 B | `ba50851d3aa5c9a37929750be8acc7f34724a48652cd2dbf8856f546834463eb` | 52 |

Built twice with `rm -f Cargo.lock` before each (`cmp` identical), **no packed `Cargo.lock`**,
`check` green (`native-1/0a875da…/e59da909…, for 26.3`), `fits --build 0a875da` → `made for`,
`fits --build 0b14f8a` → `fits`, `verify-sdk-package.py rftoolsutility` → **1/1**.

## Slice 4 — the spawner (2026-11-21)

**Chosen from the port's own gap list** — `## Gaps (the rest of the mod)`, verbatim:

> **Environmental controller** and its 23 modules; the **spawner** with its 100
> mob recipes; `syringe`.

and `FINDINGS.md` §3 ("the spawner and its 100 mob recipes"). It is the last single machine in
the `machines` tier with a loop that closes without a screen: a block entity with an energy
buffer, a mob setting, a server tick that spawns the mob, and the jar's own mob recipes as the
family's data. Its whole rule is readable through `horizon-ctl` (`/rftoolsutility spawner`
prints the energy, the mob and the recipe's numbers; `/rftoolsutility recipes` counts the
table), so no menu is needed.

**Pin:** Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, mod API `0.2`, game 26.3. **Version 0.2.3.**

**Read for this slice** (analysis only; the clone was deleted after): `modules/spawner/blocks/
SpawnerTileEntity.java` (`tickServer`, `testSyringe`, `addMatter`, `collectImplicitComponents`),
`modules/spawner/recipes/{SpawnerRecipe,SpawnerRecipes,SpawnerRecipeSerializer}.java` (the
custom recipe type, the entity-keyed table, `MobSpawnAmount`), `modules/spawner/
SpawnerConfiguration.java` (`SPAWNER_MAXENERGY` 200,000, `SPAWNER_RECEIVEPERTICK` 2,000,
`maxMatterStorage` 6,400), `modules/spawner/data/SpawnerData.java` and `SpawnerModule.java`
(registration, `orientedBlock`, the loot table's `ITEM_SPAWNER_DATA`), and
`DataGenHelper.getDefaultMobData` (the 70 mobs).

**The count is 70, not the campaign's 100** — `unzip -l <jar> | grep -c 'recipe/minecraft_'`
says 70, every one with `"type": "rftoolsutility:spawner"`, `entity`, `power` and `item1..3`;
the jar's datagen list is the same 70. The artefact wins, and all 70 are carried.

### What it does

- **The block.** `rftoolsutility:spawner` on the **`minecraft:white_glazed_terracotta`**
  template (the crafter's measured full-cube, no-block-entity, no-menu template with a
  four-way horizontal `facing`), the jar's own front texture on the north face and the machine
  base top/bottom/sides, on McJtyLib's `STANDARD_IRON` properties.
- **The energy.** 200,000 FE (`SpawnerConfiguration.SPAWNER_MAXENERGY`), at most 2,000 a call
  (`SPAWNER_RECEIVEPERTICK`) on every face, in the engine's `energy` int — the same shape the
  crafter uses. The spawner only consumes (`max_extract: 0`).
- **The mob setting, from the recipe table.** The jar's 70 `rftoolsutility:spawner` recipes are
  registered as a **crafting kind** (`CraftingRecipeKind` + `Registrar::
  register_crafting_recipe_serializer`), because the pin has no separate recipe-serializer
  route a mod may take: `register_crafting_recipe_serializer(name, kind)` files `<mod>:<name>`
  under `minecraft:crafting`, and the files' `type` names exactly that id. Upstream's recipe is
  a data table, so the kind answers `matches` → false and `assemble` → none, as its own
  `SpawnerRecipe` does; the spawner block is what reads the values. `/rftoolsutility mob <pos>
  <id>` resolves the mob's recipe and keeps its `power` and its three `item1..3` amounts on the
  machine (upstream's `MobData`, read once per table load); an id no recipe names is refused.
- **The spawn tick.** With a mob and its cost in the buffer, the tick pays the cost once and
  calls `LevelRef::spawn_mob(host, mob, pos, SpawnReason::Spawner)` — `EntityType.spawn` with
  the `SPAWNER` reason, which finalizes the mob as vanilla's own spawner does — on the block's
  facing side, centred on x/z (upstream's `getNormal()` offset and `moveTo(sx + 0.5, sy, sz +
  0.5)`). A refused spawn pays nothing.
- **The item carries the machine.** A broken spawner's own item takes the block entity's saved
  compound as vanilla's `minecraft:block_entity_data`, so the mob, its numbers and the energy
  come back on placement (upstream's `collectImplicitComponents`).
- **The readout.** `/rftoolsutility spawner <pos> [nonce]` prints
  `energy=<held>/200000 mob=<id|none> cost=<FE|-> needs=<a,b,c|-> spawned=<n>`;
  `/rftoolsutility recipes [nonce]` prints the table's size and two of its costs. `tools/
  parse_spawner.py` parses the first.

### Numbers and where they came from

| Number | Value | Source |
|---|---:|---|
| energy buffer | 200,000 FE | `SpawnerConfiguration.SPAWNER_MAXENERGY` |
| fill per call | 2,000 FE | `SpawnerConfiguration.SPAWNER_RECEIVEPERTICK` |
| matter storage (not ported) | 6,400 | `SpawnerConfiguration.maxMatterStorage` (`64 * 100`) |
| zombie's spawn cost | 800 FE | `data/rftoolsutility/recipe/minecraft_zombie.json` (`power`) |
| blaze's spawn cost | 1,000 FE | `data/rftoolsutility/recipe/minecraft_blaze.json` (`power`) |
| zombie's matter amounts | 0.1, 0.2, 20.0 | the same file's `item1..3.amount` |
| blaze's matter amounts | 0.1, 0.5, 30.0 | the same |
| mob recipes | **70** | `unzip -l` lists 70 `data/rftoolsutility/recipe/minecraft_*.json` |
| block properties | hardness 2.0, blast 2.0, correct tool, metal sound | `BlockBuilder.STANDARD_IRON`, the tank's and the crafter's row |
| the spawner's own recipe | redstone/rotten flesh/ender pearl/blaze rod/bone around a machine frame | `data/rftoolsutility/recipe/spawner.json` (carried) |
| facing | four horizontal directions | upstream's `orientedBlock` is six-way; the content template gives four (the crafter's deviation) |

### Red → green (unit, verbatim)

The four new runtime rules were written behind `todo!()` bodies first (`slice4-red.log`):

```
failures:
---- runtime::tests::a_machine_with_no_known_cost_spawns_nothing stdout ----
thread 'runtime::tests::a_machine_with_no_known_cost_spawns_nothing' (3062707) panicked at src/runtime.rs:501:5:
not yet implemented
---- runtime::tests::a_recipe_is_found_by_the_mob_it_spawns stdout ----
thread 'runtime::tests::a_recipe_is_found_by_the_mob_it_spawns' (3062710) panicked at src/runtime.rs:490:5:
not yet implemented
---- runtime::tests::a_spawn_costs_the_recipes_own_power_once stdout ----
thread 'runtime::tests::a_spawn_costs_the_recipes_own_power_once' (3062711) panicked at src/runtime.rs:501:5:
not yet implemented
---- runtime::tests::a_spawn_lands_on_the_neighbour_the_facing_names stdout ----
thread 'runtime::tests::a_spawn_lands_on_the_neighbour_the_facing_names' (3062712) panicked at src/runtime.rs:511:5:
not yet implemented

test result: FAILED. 48 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Green: **52 passed; 0 failed** (`slice4-runtime-green.log`), then **58 passed; 0 failed** with the
six glue tests (`slice4-glue-green.log`, `sdk-test.log`): 25 runtime tests (21 tank + 4 spawner),
27 glue tests (10 tank + 5 crafter + 6 spawner) and 6 crafter runtime tests. fmt clean, clippy
`-D warnings` 0 warnings (`sdk-clippy.log`).

One test expectation was wrong on the first green run — the south neighbour of `[200,101,0]` is
`z = 1`, centred `1.5`, and the test asserted `0.5`; the implementation was right and the
expectation was corrected (the red leg above is the before-implementation run).

### In game (evidence under `../_shared/evidence/rftoolsutility-e2e/`)

`tools/e2e.sh` phases A–AB, one lease per leg, every log ends with its exit:

| log | verdict |
|---|---|
| `e2e-slice4.log` | **PASS**, `EXIT:0` — A–AB |
| `e2e-slice4-selftest-spawn.log` | **FAIL: the spawner holds 0 FE, expected 800**, `EXIT:1` |
| `e2e-slice4-control.log` | **FAIL: the tank item did not register**, `EXIT:1` (the unmodded Nightly) |

`check-e2e-logs.py rftoolsutility-e2e --strict` → `logs=11 ok=11 missing-exit=0 bad-exit=0`.

The slice's phases, each asserting a parsed value (the transcript is in the log):

| phase | what it asserts (observed) |
|---|---|
| **W** | a spawner placed from its item reads `energy=0/200000 mob=none cost=- needs=- spawned=0`, has `facing=west`, and `/rftoolsutility recipes` answers `/rftoolsutility: 70 spawner recipes, zombie=800 blaze=1000 nonce=28` |
| **X** | `minecraft:not_a_mob` is refused (`no spawner recipe for minecraft:not_a_mob (70 recipes are loaded).`) and changes nothing; `minecraft:zombie` is set from its own recipe (`spawns minecraft:zombie for 800 FE.`), and the readout then reads `cost=800 needs=0.1,0.2,20` |
| **Y** | at **0 FE** 60 ticks spawn nothing: `spawned=0`, `energy=0`, and no zombie within 8 blocks (the counterfactual for Z) |
| **Z** | 800 FE spawns **exactly one** `minecraft:zombie` with `kind:"mob"` within 5 blocks of the spawner, and the readout goes `energy=800` → `energy=0`, `spawned=1` — the recipe's own 800 FE |
| **AA** | `minecraft:blaze` costs its own recipe's 1,000 FE (`needs=0.1,0.5,30`), and 1,000 FE spawns exactly one blaze, leaving 0 and `spawned=2` |
| **AB** | a survival break drops a spawner whose item carries `minecraft:block_entity_data` `{needs: [0.1f, 0.5f, 30.0f], spawned: 2, mob: "minecraft:blaze", cost: 1000, id: "rftoolsutility:spawner", energy: 0}`, and placing it back reads the same mob, cost and count |

Counterfactuals: **Y** is the same machine at 0 FE (no mob, no spend) against **Z**'s 800 FE (one
mob, 800 spent), and **X**'s two ids differ only in whether the recipe table names them;
`SELFTEST_AT=spawn` inverts Z and the leg fails as designed, so Z can fail; `CONTROL=1` runs the
same script on the unmodded Nightly, where the package's items do not exist.

### Measured traps this slice cost (engine and harness)

1. **A control's reply must start with `/rftoolsutility`.** `control()` picks its answer with
   `grep -m1 '/rftoolsutility'`; the new `recipes` line began `rftoolsutility: …` and the phase
   read an empty `REPLY` while the game log showed the right line. The readout now spells the
   slash like every other control reply.
2. **`ctl place` is refused by an entity standing on the target** ("the block did not change
   (blocked by an entity, or the item cannot be placed)"). It cost the slice-3 crafter phase on
   the *second* run of this slice: the End's endermen wander onto the arena floor over a
   ten-minute leg. `try_place` now clears the two blocks around the target (`kill @e[type=!player,
   x=…,y=…,z=…,distance=..2]`) between attempts; the assertion still reads the block that ends up
   there.
3. **A zombie or a blaze beside a survival player is a failed leg.** A mob spawns one block from
   the block, and the player stands two blocks west: the spawner phases run in creative (no
   damage, and mobs do not target a creative player), and AB switches back to survival only for
   its break.
4. **The recipe's `item1..3.object` is an ingredient form this port must not parse.** The files
   carry 1.21.1's `{"item": …}`, a tag, and in `item2` a *list* of items; 26.3's ingredient
   parser rejects both forms. This port reads only each `item`'s `amount`, so the files travel
   **verbatim** and no rewrite applies — which is why the family is carried byte for byte.
5. **The kind is what makes the files in force.** Without
   `register_crafting_recipe_serializer("spawner", …)` the 70 files name a type nothing claims;
   with it, `/rftoolsutility recipes` counts all 70 in game (W), and the same call is what
   `fits`/`check` see.
6. **`LevelRef::spawn_mob` works from a block-entity tick.** The pin documents `EntityType.spawn`
   with a `SpawnReason`; a spawner's own tick is the same shape as vanilla's spawner block, and Z
   proved it in game (one zombie per 800 FE, none at 0).

### Deviations and content adaptations

- **The matter gate is not ported.** Upstream returns early until all three matter amounts are
  stored, and fills them from matter beamers (`MatterBeamerTileEntity`, 200,000 FE and 1,000 a
  tick each) and from the syringe's `SpawnerData`. This slice spends energy only; the recipe's
  three amounts are read and kept on the machine (and printed), so a later slice that adds the
  beamers has the data in place.
- **The mob is set by a command, not a syringe slot.** Upstream's `SLOT_SYRINGE` takes a syringe
  whose `SyringeData` names a mob at the configured injection level; this slice has no syringe
  item and no menu, so `/rftoolsutility mob` writes the same `mob` the syringe would.
- **Four-way facing, not six.** The content schema takes a block's states and placement rule
  from its vanilla template; the spawner borrows the crafter's measured four-way template, so a
  spawner cannot be turned up or down (upstream's `orientedBlock` has six).
- **No GUI, no filter, no comparator, no TOP/Jade, no tooltip** — the same gaps the tank and the
  crafter carry.
- **The carried recipe is not crafted in game.** Its middle ingredient is
  `rftoolsbase:machine_frame`, which only the parent port registers (the tank's and the
  crafter's own gap); the e2e gives the block instead. Its `c:bones`, `c:rods/blaze` and
  `c:ender_pearls` tags are declared by the assembler with the honest vanilla memberships.

### Upstream defects fixed

- **The spawner pays before it knows it can spawn.** `SpawnerTileEntity.tickServer` checks the
  matter, then `energyStorage.consumeEnergy(rf)`, clears the three matter amounts with
  `setData(...)`, and *then* resolves the entity type:
  `EntityType<?> type = Tools.getEntity(mobId); if (type == null) { Logging.logError(…); return; }`
  — a recipe whose mob this game does not have (a pack's recipe for an absent mod's mob, or a
  mob removed by an update) costs the full RF and all three matter amounts for a spawn that never
  happens. **The port pays only after `spawn_mob` answered** (`spawner_tick`: `None` leaves the
  buffer and the count untouched), so a failed spawn costs nothing. All 70 carried recipes name
  `minecraft:` mobs, so no carried content behaves differently; a pack's own recipe does. Worth a
  small upstream report; not filed (the orchestrator decides).
- **Not a defect, recorded as a deviation:** `SpawnerRecipes.loadRecipes` caches the whole table
  in a `static` map and only loads it when the map is empty, so a `/reload` that adds or changes
  a spawner recipe is not seen until the world restarts. The port reads the table when the mob is
  set, which is the same "read once, keep on the machine" shape with the same staleness.

### Coverage of the spawner family (after slice 4)

| Family | Status | Evidence / why not |
|---|---|---|
| `rftoolsutility:spawner` | tested | W: placed, read back, `facing` state; AB: the item carries the machine |
| `block_entity` | tested | A–AB (the tank's, the crafter's and the spawner's); AC/AD/AE/AF: the three crafter tiers, each under its own type id |
| `loot_table` | tested | V: the iron-tool gate; J, U, AB and AF: the drop and the data it carries (all three crafter tiers share the table shape and the tag, proven by V and AF) |
| `recipe:rftoolsutility:spawner` | tested | W: all 70 in force; X/Y/Z/AA: zombie 800 and blaze 1000 exercised end to end. The other 68 are carried and decoded by the same reader; the count in W is what proves they loaded, and a row per file would prove nothing the count does not |
| `recipe:minecraft:crafting_shaped` | blocked(`rftoolsbase:machine_frame`) | the tank's, the crafter's and the spawner's own recipes need the parent port's item; the e2e gives the blocks instead |
| `tag:item` | not ported | the jar's `rftoolsutility:living/*` yield tags belong to the matter system; the `c:` tags the recipes name are declared by the assembler |
| `command` | tested | all ten subcommands are called and their parsed replies asserted (A–AB) |
| the matter gate (three matter types) | not ported | the beamers and the syringe are a different machine family; the recipe amounts are carried and printed (`needs=`) |
| the syringe item and its injection levels | not ported | no item and no menu in this slice; `/rftoolsutility mob` writes the same setting |
| the GUI (`GuiSpawner`) | not ported | no menu in this slice |
| six-way facing | not ported | four-way template (the crafter's deviation) |
| the infusing power discount | not ported | needs `rftoolsbase`'s infusable path (the crafter's gap) |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

### Artifact

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `rftoolsutility.hmod` (port root) | 103,488 B | `d12c3646abde71c541adc707add6a044a934a77ce375a301d4de1a46e52c27b7` | 134 |

Built twice with `rm -f Cargo.lock` before each (`cmp` identical: `8c836e61…` for the pre-fix
source, `a55d4e5f…` after the reply fix and `d12c3646…` for the final, documented source), **no packed `Cargo.lock`**, `check` green
(`native-1/0a875da…/e59da909…, for 26.3`), `fits --build 0a875da` → `made for`,
`fits --build 0b14f8a` → `fits`, `verify-sdk-package.py rftoolsutility` → **1/1**, and the
launcher's `native-build` compiled the mod against the pin (`Compiling rftoolsutility (1 of 1)`)
for the in-game legs.

## Slice 5 — the crafter tiers (2026-11-21)

**Chosen from the port's own gap list** — `## Gaps (the rest of the mod)` and the slice-3
coverage row, verbatim:

> **Crafters** — **`crafter1` done in slice 3** (the block, its 41-slot container, its 50,000 FE
> buffer, the craft loop with chained crafting, keep-one, and its item carrying the machine).
> Remaining: `crafter2`/`crafter3` (4 and 8 recipe slots, the same block entity), …

and the slice-3 artefact's own conclusion (`## Next slice` in the 0.2.3 report): two carried
blocks with items/recipes/advancements/loot and **one `supportedRecipes` value each**
(`CrafterBaseTE.createTier2/3`), read by the loop slice 3 already proved. It closes the crafter
family's content and fits the **machines** tier exactly — no new block entity, container, energy
buffer or renderer; one carried block and one number per tier.

**Pin:** Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, mod API `0.2`, game 26.3. **Version 0.2.4.**

**Read for this slice** (analysis only): the jar's `assets/rftoolsutility/{blockstates,models,
textures}/**crafter2|3*`, its `data/rftoolsutility/{recipe,advancement,loot_table}/**crafter2|3*`,
the lang lines (`message.rftoolsutility.crafter2/3.header`: "can handle up to 4/8 recipes at
once"), and from the branch `1.21_neo` clone at commit `f799314007d68042df8504e83cd549360f2627c7`:
`CrafterModule` (`CRAFTER2`/`CRAFTER3` registration, `frontBasedModel`, the tier recipes) and
`CrafterBaseTE.createTier1/2/3` (`new CrafterBaseTE(…, 2 / 4 / 8)`).

### What it does

- **The blocks.** `rftoolsutility:crafter2` and `rftoolsutility:crafter3` on the crafter1's own
  measured template (`minecraft:white_glazed_terracotta`: full cube, four horizontal facings,
  no block entity of its own, no menu), their own jar front textures (`machinecrafter2/3`) on the
  north face and the machine base top/bottom/sides, on McJtyLib's `STANDARD_IRON` properties.
- **The same machine, one number different.** Each tier declares its own block entity type
  (`rftoolsutility:crafter2/3`), the same 41-slot container, the same 50,000 FE buffer and the
  same craft loop; the only tier-specific value is `supportedRecipes` — **2, 4 or 8** — and it is
  now `runtime::CrafterTier::recipes()`, read by `register` (the type it declares), by
  `crafter_at` (which tier a position holds), by `teach` (the slot bound and the refusal's count)
  and by `block_drops` (each tier's own item and `block_entity_data` id).
- **The carried content.** Each tier brings its item model, item definition, blockstate
  (the jar's four horizontal rotations), block model, front texture, block declaration, creative
  tab entry, `mineable/pickaxe` + `needs_iron_tool` membership, recipe, advancement and loot
  table. The recipes are the jar's `mcjtylib:copy_components` wrapper **unwrapped** to its inner
  `minecraft:crafting_shaped` (the wrapper's component copy is a deviation below).
- **The readout and the refusal.** `/rftoolsutility crafter <pos>` is the same line for every
  tier; `/rftoolsutility recipe <pos> <index> <id>` refuses a slot past the tier's own count with
  that count — `this crafter has 8 recipe slots` on a crafter3, `… 4 …` on a crafter2, `… 2 …` on
  a crafter1 (`fn recipe_slot`, pinned by a glue test).

### Numbers and where they came from

| Number | Value | Source |
|---|---:|---|
| recipes per tier | 2 / 4 / 8 | `CrafterBaseTE.createTier1/2/3` (`supportedRecipes`) and the jar's lang lines "can handle up to 2/4/8 recipes at once" |
| energy buffer | 50,000 FE | `CrafterConfiguration.MAXENERGY` (the same machine) |
| fill per call | 500 FE | `CrafterConfiguration.RECEIVERTICK` — unchanged from slice 3 |
| one cycle | 100 FE | `CrafterConfiguration.rfPerOperation` — unchanged |
| slots | 41 per tier | `CrafterContainer.CONTAINER_FACTORY` — the same container |
| block properties | hardness 2.0, blast 2.0, correct tool, metal sound | `BlockBuilder.STANDARD_IRON` + `ironPickaxeTags()`, the crafter1 row |
| facing | four horizontal directions | `orientedBlock` is six-way upstream; the template gives four (slice 3's recorded deviation) |
| tier recipes | crafter2 = craft table + `rftoolsutility:crafter1` + table; crafter3 = the same with `crafter2` | the jar's `recipe/crafter2.json` / `crafter3.json` inner shaped recipe |

### Red → green (unit, verbatim)

The tier rule was written as tests over `todo!()` bodies first (`slice5-red.log`):

```
---- runtime::tests::a_crafter_tier_is_found_by_its_own_block stdout ----
thread 'runtime::tests::a_crafter_tier_is_found_by_its_own_block' (634970) panicked at src/runtime.rs:303:9:
not yet implemented
---- runtime::tests::each_crafter_tier_keeps_its_own_number_of_recipes stdout ----
thread 'runtime::tests::each_crafter_tier_keeps_its_own_number_of_recipes' (634994) panicked at src/runtime.rs:290:9:
not yet implemented
---- runtime::tests::every_crafter_tier_has_its_own_identity_and_slot stdout ----
thread 'runtime::tests::every_crafter_tier_has_its_own_identity_and_slot' (634996) panicked at src/runtime.rs:296:9:
not yet implemented
---- tests::a_slot_past_the_tiers_count_is_refused_with_that_count stdout ----
thread 'tests::a_slot_past_the_tiers_count_is_refused_with_that_count' (635007) panicked at src/lib.rs:1777:5:
not yet implemented

test result: FAILED. 58 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Green: **62 passed; 0 failed** (`slice5-green.log`, `sdk-test.log`) — 39 `runtime` tests
(the tank's, the crafter's including the three new tier tests, and the spawner's) and 23 glue
tests in `src/lib.rs` (the tank's, the crafter's — including the new
`a_slot_past_the_tiers_count_is_refused_with_that_count` — and the spawner's). fmt clean, clippy `-D warnings` 0 warnings
(`slice5-clippy.log`, `sdk-clippy.log`).

### In game (evidence under `../_shared/evidence/rftoolsutility-e2e/`)

`tools/e2e.sh` phases A–AF, one lease per leg, every log ends with its exit:

| log | verdict |
|---|---|
| `e2e-slice5-red.log` | **FAIL: ctl place rftoolsutility:crafter3 … was refused three times** (the same script on the 0.2.3 package, where the tier does not exist), `EXIT:1` — the in-game assertion's own red leg |
| `e2e-slice5.log` | **PASS**, `EXIT:0` — A–AF |
| `e2e-slice5-selftest-tiers.log` | **FAIL: the crafter holds 0 FE, expected 800**, `EXIT:1` |
| `e2e-slice5-control.log` | **FAIL: the tank item did not register**, `EXIT:1` (the unmodded Nightly) |

`check-e2e-logs.py rftoolsutility-e2e --strict` → `logs=15 ok=15 missing-exit=0 bad-exit=0`.

The slice's phases, each asserting a parsed value (the transcript is in the log):

| phase | what it asserts (observed) |
|---|---|
| **AC** | a crafter3 placed from its item reads `recipes=-`, `energy=0` and no buffer; eight recipes are taught (each answering `slot N … crafts <id>`) and the readout lists all eight; a **ninth** slot answers `this crafter has 8 recipe slots.`; stocked for eight cycles and charged exactly **800 FE**, the readout ends `energy=0` with `crafting_tablex8`, `chestx8`, `stickx32`, `furnacex8`, `sugarx8`, `paperx24`, `breadx8` and `gold_nuggetx72` — exactly eight 100 FE cycles, all eight recipes in each |
| **AD** | a crafter2 takes four recipes and a **fifth** answers `this crafter has 4 recipe slots.`, the readout still listing four |
| **AE** | the **counterfactual**: the same eight teaches on a **crafter1** — slots 2…7 refused with `2 recipe slots` — the same stock and the same 800 FE end at `energy=0` with only `crafting_tablex8` and `chestx8`; none of `stick`/`furnace`/`sugar`/`paper`/`bread`/`gold_nugget` exists |
| **AF** | a survival break of the taught crafter3 drops an item whose `minecraft:block_entity_data` carries all eight `id: "…"` recipes (and the buffer: 8 tables, 32 sticks, 8 chests, 8 furnaces, 8 sugar, 24 paper, 8 bread, 72 nuggets); placing it back reads the same eight recipes and the same buffer |

Counterfactuals: **AE** is the same script on another tier (two products against eight); `AC`'s
ninth slot against `AD`'s fifth and `AE`'s third differ only in the tier's count;
`SELFTEST_AT=tiers` inverts AC (the machine must have crafted nothing and spent nothing) and the
leg fails as designed, so AC can fail; `CONTROL=1` runs the same script on the unmodded Nightly,
where the package's items do not exist.

### Measured traps and notes this slice cost (harness)

1. **`$CRAFTER` is a prefix of `$CRAFTER2`.** `check-coverage.py` expands `NAME=literal`
   assignments before matching ids, replacing `$CRAFTER` first, so `"$CRAFTER2"` became
   `rftoolsutility:crafter12` and both new blocks read as *unnamed* (`MISSING: 4 entries`).
   Renaming the variables to `TIER2_BLOCK`/`TIER3_BLOCK` fixed the tool's view; the ids
   themselves were correct in game all along.
2. **The craft-count stock has to match the cycle count exactly.** The stock is eight cycles'
   worth (112 planks across two slots, 64 cobblestone, 32 sugar cane, 24 wheat, 8 gold ingots)
   and the charge is exactly 800 FE: a recipe that fails to craft ends that tick's cycle early, so
   the buffer cannot reach 0 FE unless all eight recipes crafted in all eight cycles.
3. **Products are no recipe's ingredient in this script.** That is deliberate — with `mode int`
   a product that another recipe accepts (planks) would be consumed by a later cycle and the
   counts would drift; the eight recipes here produce tables, chests, sticks, furnaces, sugar,
   paper, bread and gold nuggets, none of which any of the eight consumes.
4. **The recipe ids are the 26.3 ids, not the 1.21.1 file names' guesses:** sugar is
   `minecraft:sugar_from_sugar_cane` (there is no `minecraft:sugar` recipe), and the furnace
   accepts `minecraft:cobblestone` through `#minecraft:stone_crafting_materials` — both read from
   the prepared 26.3 root before the run.

### Deviations and content adaptations

- **The tier upgrade does not keep the old machine's data.** The jar's `crafter2`/`crafter3`
  recipes are `mcjtylib:copy_components` — an outer wrapper that copies the crafted result's
  components from the tier-1 (or tier-2) machine in the grid onto the new item, so an upgraded
  machine keeps its taught recipes, buffer and energy. This port does not register mcjtylib's
  recipe type, so the wrapper is unwrapped to its inner `minecraft:crafting_shaped` recipe: the
  grid, the result and the unlock are the jar's, and the crafted machine starts empty. A user
  notices: upgrading a taught crafter1 to a crafter2 loses what it was taught. Recorded as a gap
  (`mcjtylib:copy_components`), not a defect — upstream's own behaviour needs the library mod.
- **Four-way facing, not six** — the crafter1's template deviation, shared by both tiers.
- **No GUI, no filter module, no `ExtC`, no tooltip, no infusing discount** — the crafter1 gaps;
  the tiers add no new one.
- The carried recipes name `rftoolsutility:crafter1`/`crafter2` (this port's own blocks), so with
  this port alone the chain is craftable as soon as a player has the ingredients — the tier
  recipes do **not** need `rftoolsbase:machine_frame` (only the crafter1's own recipe does).

### Upstream defects fixed

None new in this slice: the tier's behaviour is the same block entity with a different
`supportedRecipes`, and the port reproduces it exactly (the jar's own lines say 2/4/8). The
slice-3 and slice-4 defect records stand.

### Coverage of the crafter family (after slice 5)

| Family | Status | Evidence / why not |
|---|---|---|
| `rftoolsutility:crafter1` | tested | N: placed, read back, `facing` state; U: the item carries the machine |
| `rftoolsutility:crafter2` | tested | AD: four recipes, a fifth refused with 4; the block, item and `facing` read back |
| `rftoolsutility:crafter3` | tested | AC: eight recipes and eight cycles; AF: the drop carries all eight and places back with them |
| `recipe:rftoolsutility:crafter1|2|3` | blocked(`rftoolsbase:machine_frame` / `mcjtylib:copy_components`) | crafter1's own recipe needs the parent port's machine frame; the tier recipes are carried unwrapped and craftable, but no phase crafts them (the e2e gives the blocks, as every slice has) |
| the craft loop, chained crafting, keep-one, the 50,000 FE buffer | tested | Q, R, S, T (slice 3); the tier's `supportedRecipes` by AC/AD/AE |
| the GUI, the ghost-grid editor, the filter module, `ExtC`, the infusing discount, the item tooltip | not ported | the crafter1 gaps (a menu, `rftoolsbase`'s items and paths, a `ctl` tooltip readout) |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

### Artifact

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `rftoolsutility.hmod` (port root) | 111,935 B | `4cd3abdb4786ff498bfd2d45bfb8d50a444f7f89d18d396a93ebd9c2e427534f` | 152 |

Built twice with `rm -f Cargo.lock` before each (`cmp` identical), **no packed `Cargo.lock`**,
`check` green (`native-1/0a875da…/e59da909…, for 26.3`), `fits --build 0a875da` → `made for`,
`fits --build 0b14f8a` → `fits`, `verify-sdk-package.py rftoolsutility` → **1/1**, and the
launcher's `native-build` compiled the mod against the pin (`Compiling rftoolsutility (1 of 1)`)
for the in-game legs.

## Slice 6 — the tick's own dimension (2026-11-21)

**Chosen from the reviewer's comment on PR #1643** (`levzzz5154`), verbatim:

> Use the ticking machine's actual dimension. [The crafter and inherited spawner resolve a level by
> scanning for any matching block at the coordinates](https://github.com/AdLf0208/rftoolsutility/blob/rftoolsutility-0.2.4/src/lib.rs#L447-L465).
> With two dimensions containing the same machine at the same position, the tick can act on the
> first dimension rather than its own; `spawner_tick` spends this block's energy and spawns in the
> chosen level. Resolve from the tick's server context and check duplicate coordinates across two
> levels.

**Pin:** Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, mod API `0.2`, game 26.3. **Version 0.2.5.**

### What it was (root cause, measured in game — not read from the Java)

`level_of(host, pos, block)` and `crafter_level_of(host, pos)` walked `host.levels().list_levels()`
and took the **first** level whose block at `pos` had the machine's id (or any crafter tier's id).
`list_levels` answers "in vanilla dimension order", so the Overworld always came before the End:
with the same machine at the same `x,y,z` in both, the End machine's tick acted on the Overworld's
level. Both ticks paid from *this* block entity's saved compound (the tick's `event.entity` is the
right one) and then used the *wrong* level for everything else.

Two-dimension leg on 0.2.4 (phase AG, `e2e-slice6-red.log`), the twin unfed in the Overworld:

| family | what the wrong level did | observed on 0.2.4 |
|---|---|---|
| spawner | `facing_of` + `level.spawn_mob` on the Overworld | **nothing** — `LevelRef::spawn_mob` refuses "a level not in place", so the tick returned `Unchanged`: the End spawner kept its 800 FE, `spawned=0`, and no zombie existed in either dimension |
| crafter | `level.recipe_for` on the Overworld | crafted normally (recipes are server-wide, and the lookup tolerated the parked level): 3 logs → 12 planks for 300 FE, so the wrong level was **harmless for the crafter at this API** — but it was the same wrong choice |

The reviewer's wording ("spawns in the chosen level") is bounded by the API: a parked level refuses
the spawn instead. The user-visible defect is the same class and sharper — **a spawner silently
stops spawning (and stops paying) when an identical machine stands at the same coordinates in
another loaded dimension.**

### The fix

`ticking_level(host)` — the level the server ticks now (`LevelRef::in_place`), which is the tick's
own dimension whatever else is loaded and whatever stands at the same coordinates there. It is the
same resolution `bigreactors`, `computercraft` and `actuallyadditions` use for a tick that is told a
position and not a level. `crafter_tick` and `spawner_tick` now call it; `level_of` and
`crafter_level_of` are gone. The tick path has no block-matching level loop left:

```
$ grep -n "level_of\|list_levels" ports/rftoolsutility/src/lib.rs
457:        .list_levels()          # ticking_level: .find(|level| level.in_place(host))
1523:            .list_levels()      # the command path: `.find(|level| level.dimension() == invocation.dimension)`
```

The only `list_levels` in the tick path is `ticking_level`'s; the second is the command path, which
resolves by the invocation's own dimension and was never a scan for a block.

### Red → green (in game, verbatim)

Red, the same scenario on the 0.2.4 package (`e2e-slice6-red.log`, `EXIT:1`):

```
  the fed End crafter: 36=minecraft:oak_planksx12, 0 FE left
  the unfed Overworld twin: '-' in, '-' out, '-', 0 FE
  zombies at 202,101,19: the End has 0, the Overworld twin has 0; the End spawner reads 800 FE and spawned=0
FAIL: the End's spawner spawned 0 zombie(s) at 202,101,19 in its own dimension, expected 1: []
```

Green, the same script on 0.2.5 (`e2e-slice6.log`, `EXIT:0`):

```
  the fed End crafter: 36=minecraft:oak_planksx12, 0 FE left
  the unfed Overworld twin: '-' in, '-' out, '-', 0 FE
  zombies at 202,101,19: the End has 1, the Overworld twin has 0; the End spawner reads 0 FE and spawned=1
  the End spawner acted in the End; the Overworld twin stayed empty
```

**No new unit test, deliberately.** The rule needs the game's level list; a hand-written
`ServerHost` double would have to answer `list_levels`/`in_place` itself — it would test the mock,
not the game. The fail-capable regression is the in-game leg above, plus
`SELFTEST_AT=dimension` (below), which is the campaign's TDD pipeline for a glue rule: the rule is
exercised red against the previous package and green after.

Unit suite unchanged at **62 passed; 0 failed** (`slice6-test.log`), fmt clean, clippy
`-D warnings` 0 warnings (`slice6-clippy.log`).

### In game (evidence under `../_shared/evidence/rftoolsutility-e2e/`)

| log | verdict |
|---|---|
| `e2e-slice6-red.log` | **FAIL: the End's spawner spawned 0 zombie(s) at 202,101,19 in its own dimension, expected 1**, `EXIT:1` — the 0.2.4 package, the failing assertion |
| `e2e-slice6.log` | **PASS**, `EXIT:0` — phases A–AG |
| `e2e-slice6-control.log` | **FAIL: the tank item did not register**, `EXIT:1` (the unmodded Nightly) |
| `e2e-slice6-selftest-dimension.log` | **FAIL: the End's spawner spawned 1 zombie(s) in its own dimension, expected 0**, `EXIT:1` (AG inverted) |
| `e2e-slice6-fill.log`, `-restore.log`, `-selftest-craft.log`, `-selftest-spawn.log`, `-selftest-tiers.log` | each `EXIT:1` with its own `FAIL:` (B, K, Q, Z, AC inverted) — the earlier legs still fail as designed |

`check-e2e-logs.py rftoolsutility-e2e --strict` → `logs=24 ok=24 missing-exit=0 bad-exit=0`.

**Phase AG** (the two-dimension regression, the exact scenario the reviewer asked for):

- the crafter and the spawner are placed at `202,101,16` and `202,101,18` **in the End** (the
  arena floor) and at the **same `x,y,z` in the Overworld**, inside a sealed stone room carved out
  of a filled cube (the Overworld at `y=101` can be ocean or mountain); both chunks are
  `forceload`ed so both dimensions keep ticking while the player reads one of them;
- **only the End pair is fed**: the End crafter is taught `minecraft:oak_planks`, stocked with 3
  oak logs and charged 300 FE; the End spawner is set to `minecraft:zombie` from its own 800 FE
  recipe and charged 800 FE;
- asserted: the End crafter reads `energy=0/50000 … recipes=minecraft:oak_planks in=- out=36=minecraft:oak_planksx12`
  (its buffer fell, its output appeared); the Overworld twin reads `energy=0`, `recipes=-`,
  `in=-`, `out=-`; the End holds **1** zombie at `202,101,19` with `energy=0`, `spawned=1`; the
  Overworld twin holds **0**;
- read back by hopping into each dimension (`ctl block`, `ctl entities` and the mod's own commands
  read the player's level) — the transcript above is from that read; `SELFTEST_AT=dimension`
  inverts the End count and fails as designed.

### Measured traps this slice cost

1. **`LevelRef::spawn_mob` refuses "a level not in place".** That is why the 0.2.4 spawner did
   nothing rather than spawning next door; it also means the fix cannot be "spawn in the level I
   picked" — the only correct level for a spawn from a tick is the in-place one.
2. **`ctl place --item` places from the hotbar only.** A machine broken while its buffer holds
   items drops those beside it, so `ctl collect` can fill the hotbar and leave the machine's own
   item in the main inventory — AF failed on exactly that (its `/data get … SelectedItem` then read
   the iron pickaxe it had broken with). Two harness fixes: `held_item_line` reads
   `data get entity @s Inventory` (the item's components ride with it in any slot), and
   `place_kept` copies the carried stack into the hotbar with
   `item replace entity @s hotbar.8 from entity @s container.<slot>` (`container.<n>` is a
   player's raw inventory slot, and `from` copies, which is all a placement needs). An
   `open-inventory`/`click-slot` route was tried first and one `ctl` call in it failed under load;
   the command route has no screen dependency.
3. **The End chunk must stay forceloaded while the player reads the other dimension**, or the
   machine under test stops ticking and the read is meaningless. The End arena is loaded by the
   player only, so AG forceloads it explicitly.

### Coverage of the tick's level (after slice 6)

| Family | Status | Evidence / why not |
|---|---|---|
| `block_entity` | tested | AG (slice 6): the fed End crafter (3 logs → 12 planks, 300 FE) and the fed End spawner (800 FE → 1 zombie at `202,101,19`) both act at their own `x,y,z` with identical, unfed twins in the Overworld — the twins keep `energy=0`, `recipes=-`, `in=-`, `out=-` and 0 zombies; `SELFTEST_AT=dimension` inverts the End count and fails. The tick resolves its level from `LevelRef::in_place`, never by scanning for a block. The tank's tick never resolved a level (it writes at `event.pos`), and phases A–I/J–M are still green |

Skills: superpowers/using-superpowers, systematic-debugging, test-driven-development, verification-before-completion; BeautifulRust/rust-best-practices

### Artifact

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `rftoolsutility.hmod` (port root) | 112,059 B | `213695fc2d98491ee0a99dfd38ceaa4bb3ee9b591d41fa7921a19d9fca2ee4fb` | 152 |

Built twice with `rm -f Cargo.lock` before each (`cmp` identical), **no packed `Cargo.lock`**,
`check` green (`rftoolsutility 0.2.5 (native-mod, format
native-1/0a875da91dbb0fdcb7ba8c42c11341aa229af51a/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for
26.3)`), `fits --build 0a875da` → `0a875da  made for`, `fits --build 0b14f8a` → `0b14f8a  fits`
(both logs print the artifact's sha first), `verify-sdk-package.py rftoolsutility` → **1/1**, and
the launcher's `native-build` compiled the mod against the pin (`Compiling rftoolsutility (1 of 1)`,
`built native-0a875da-2d419ffa479a`) for the in-game legs.
