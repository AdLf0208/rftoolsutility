# RFTools Utility — port notes (slices 1–19: the tank, the crafter tiers, the spawner, the beamer, the teleporter family, the analyzer, the sensor, the sequencer, the timer, the counter, the analog, the logic gate, the inventory checker, the redstone transmitter/receiver and the wire)

> **Current state (2026-11-24): slice 19, package 0.2.18 at pin
> `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` / mixins `e59da909…`.** Read
> **`## Slice 19 — the wire (2026-11-24)`** first: the block gate, the two direction
> conventions mapped in game, the six-wire lamp leg and the lone-wire directional
> differential. Then
> **`## Slice 18 — the redstone transmitter/receiver (2026-11-24)`**: the
> persistence gate (the world-saved `RedstoneChannels` table maps to this pin's per-save file
> store), the channel/signal rules, the lamp leg with game-time stamps and the relaunch that
> proves the table was really persisted. Then
> **`## Slice 17 — the inventory checker (2026-11-24)`**: the artefact's `checkOutput`/
> `isItemMatching` and `IncCheckerData.createDefault()`, the capability gate proving a vanilla
> chest's live slots are readable with no provider of ours, and the count/threshold lamp leg.
> Then **`## Slice 16 — the logic gate (2026-11-24)`**: the artefact's eight-cell truth table, the
> 15/0 lamp leg with its game-time stamps, and the first-gate finding on the invchecker. Then
> **`## Slice 15 — the analog (2026-11-24)`**: the artefact's arithmetic and the exact-strength lamp
> leg. Then
> **`## Slice 14 — the counter (2026-11-24)`**: the artefact's count/reset rules, the latched
> output and the game-time-stamped lamp leg. Then
> **`## Slice 13 — the timer (2026-11-24)`**: the delay/edge rules, the one-tick pulse and
> the game-time-stamped lamp leg. Then
> **`## Slice 12 — the sequencer (2026-11-24)`**: the rising-edge rule, the seven modes and
> the vanilla-lever-driven lamp leg (including the self-feedback trap). Then
> **`## Slice 11 — the sensor (2026-11-24)`**: the sensor, its real redstone output and the
> fail-capable lamp leg. Then
> **`## Slice 9 — the teleporter family (2026-11-24)`**: the transmitter, receiver and
> booster are the **player teleporter** family (not a matter mover — the artefact wins over the
> brief's "matter fell / receiver rose" wording, which this mod does not have). Then
> **`## Slice 8 — the matter beamer (2026-11-22)`**: the beamer now carries a spawner's
> recipe materials into its three stores, which closes the *feed* half of the gate slice 7
> proved. Then **`## Slice 7 — the spawner's matter gate`**, **`## Slice 6 — the tick's own
> dimension`** (the reviewer's correctness bug on PR #1643), `## Slice 5 — the crafter tiers`,
> `## Slice 4 — the spawner`, `## Slice 3 — the crafter`, `## Slice 2` (the tank's contents on its
> item) and the 2026-11-18 migration note that produced 0.2.0; everything after the migration
> describes the retired `.hnmod` bundle.

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
- **Logic:** `wire`, ~~`sensor`~~ (slice 11), ~~`sequencer`~~ (slice 12), ~~`timer`~~ (slice 13),
  ~~`counter`~~ (slice 14), ~~`analog`~~ (slice 15), ~~`logic`~~ (slice 16), ~~`invchecker`~~
  (slice 17), `digit` (blocked — no redstone output), `redstone_transmitter/receiver`,
  `redstone_information`.
- **Environmental controller** and its 23 modules; ~~the **spawner** with its 100
  mob recipes~~ — **the spawner is done in slice 4** (the block, its 200,000 FE buffer, the
  mob setting from the recipe table and the spawn tick, with the jar's **70** mob recipes
  carried — the jar ships 70, not 100: `unzip -l | grep -c 'recipe/minecraft_'`). Remaining
  there: ~~the three matter types~~ — **the matter gate is done in slice 7** (three stores, the
  per-type check, the spend and the item carry); ~~the **matter beamers** that fill them~~ —
  **the matter beamer is done in slice 8** (one block, 200,000 FE, a redstone signal, a 20-tick
  send of one block for 2,000 FE, the material match from the recipe's own `item1..3.object`,
  the wrench's destination as a command, and the `lit` state); the **matter
  transmitter/receiver/booster** of the teleporter family (a different family), the
  syringe that sets the mob in upstream's GUI, the GUI, and six-way facing; `syringe`.
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
| the matter gate (three matter types) | tested | Y1, Y2, Z, Z2, AA, AB, AG (slice 7): 800 FE with empty stores spawns nothing and keeps its FE; one type one unit short spawns nothing; exactly 0.1/0.2/20 spawns one zombie and empties the stores; 1.1/1.2/21 leaves exactly 1 in each; a new mob clears them; the item carries them; the unfed twin keeps `matter=0,0,0` |
| the matter beamers (`MatterBeamerTileEntity`) | not ported | a different block family (200,000 FE, 1,000 matter a tick, the wrench-linked destination, the beamer's own infusing bonus); `/rftoolsutility matter` writes what a beamer's send would add (`add_matter` is the accumulation and the 6,400 cap, unit-tested and exercised in Y1/Y2) |
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

## Slice 7 — the spawner's matter gate (2026-11-22)

**Chosen from the port's own gap list** — `## Slice 4 — the spawner` → `### Deviations and content
adaptations`, verbatim:

> **The matter gate is not ported.** Upstream returns early until all three matter amounts are
> stored, and fills them from matter beamers … This slice spends energy only; the recipe's
> three amounts are read and kept on the machine (and printed), so a later slice that adds the
> beamers has the data in place.

and the slice-4 coverage row *"the matter gate (three matter types) — not ported"*. It is the
behaviour the machine's own recipe table was already carrying (`needs=0.1,0.2,20`), it fits the
**machines** tier exactly (three stores on the existing block entity and a spend on the existing
tick — no GUI, no renderer, no new block), and its whole rule is readable through
`/rftoolsutility spawner`, so no screen is needed to prove it.

**Pin:** Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, mod API `0.2`, game 26.3. **Version 0.2.6.**

**Read for this slice** (analysis only; the clone was deleted after): `SpawnerTileEntity.java`
(`tickServer`'s per-type check and its `withMatter(i, getMatter(i) - amount)`, `testSyringe`'s
`clearMatter()` when the mob changes, `addMatter`'s `m * factor * 3 / (3 - infusion)` and its
`maxMatterStorage` clamp, `collectImplicitComponents`), `SpawnerData.java` (the record: `matter0`,
`matter1`, `matter2` floats and `mob`), `MatterBeamerTileEntity.java` (`addMatter(itemStack,
numblocks, infusion)` on the receiving spawner), `SpawnerConfiguration.java`
(`maxMatterStorage = 64 * 100`) and the 70 `data/rftoolsutility/recipe/minecraft_*.json` amounts.

### The brief said three fluid tanks; the artefact says three floats

The brief asked for "three fluid tanks on the existing block entity" and warned that a mod fluid
cannot live in a block entity tank via `fluid_insert`. The artefact has **no fluid anywhere in the
spawner module**:

- `SpawnerData` is `record SpawnerData(float matter0, float matter1, float matter2, ResourceLocation
  mob)` — matter is three floats in the machine's own data, not a tank;
- `SpawnerTileEntity.addMatter(ItemStack stack, int m, float beamerInfusionFactor)` maps the beamed
  **item** to one of the recipe's three `item1..3` ingredients (`mobData.getItem(i).match(stack)`) and
  adds `m * factor * 3.0f / (3.0f - beamerInfusionFactor)`, clamped at `maxMatterStorage` (6,400);
- `grep -rn "Fluid\|fluid" src/main/java/mcjty/rftoolsutility/modules/spawner/` → **0 matches**;
- the jar ships no `data/*/fluid` file and no matter fluid texture
  (`unzip -l <jar> | grep -i matter` → the four beamer/booster/transmitter/receiver blocks and their
  recipes/loot/advancements only).

So "three fluid tanks" would invent three fluids, a fluid tag and a fraction→mB scale the mod does
not have — and would let a pipe feed a spawner, a mechanic upstream does not have. **The artefact
wins** (campaign rule: a brief may not invent a mod's contents): the port keeps upstream's **three
float stores in its own saved compound** (`matter`, beside the `needs` triple it already had) and
`/rftoolsutility matter` writes what a beamer's send would add. This is **provenance, not an API
gap**: the pin *does* expose `Registrar::declare_fluid` and `Registrar::block_fluids` (the tank
slice uses the latter), so the fluid shape was expressible — it just is not this machine.

### What it does

- **The gate.** `runtime::spawn_cycle(energy, cost, needs, held)` is upstream's tick order: every
  one of the three amounts must be in store (`matter_ready`), then the recipe's `power` must be in
  the buffer (`spawn_once`); only then does it return what is left of **both**. A machine with the
  FE but no matter keeps its FE (`Y1`), and one type one unit short is a refusal (`Y2`).
- **The spend.** On a successful spawn the tick writes the energy and all three stores back
  (`spend_matter`: each store minus **its own** amount), and only after `spawn_mob` answered — a
  refused spawn pays neither (slice 4's defect fix, now covering the matter).
- **The store.** Three floats in the block entity's saved compound (`KEY_MATTER = "matter"`, the
  same `{a,b,c}` float list `needs` uses); they travel on the machine's item with the rest of the
  compound (`block_entity_data`, `AB`).
- **A new mob clears them.** `set_mob` writes zeros when the mob changes (`matter_after_mob_change`),
  upstream's `testSyringe` `clearMatter()`; re-setting the same mob keeps them (`Z2`, `AA`).
- **The feed rule.** `runtime::add_matter(held, units)` is the accumulation and the 6,400 cap
  (`maxMatterStorage`); `/rftoolsutility matter <pos> <m0> <m1> <m2>` adds each amount to its store
  (a negative one drains, which is how the harness states a phase's precondition). Upstream's beamer
  multiplies by its item factor and its infusing bonus; this port carries neither (no beamer block,
  no infusable path — the slice-3 gap), so the units arrive already scaled.
- **The readout.** `/rftoolsutility spawner <pos>` gains `matter=<a>,<b>,<c>` beside `needs=`;
  `tools/parse_spawner.py` parses it into `S_MATTER`.

### Numbers and where they came from

| Number | Value | Source |
|---|---:|---|
| one store's capacity | 6,400 | `SpawnerConfiguration.maxMatterStorage` (`64 * 100`) |
| zombie's matter | 0.1, 0.2, 20.0 | `data/rftoolsutility/recipe/minecraft_zombie.json` (`item1..3.amount`) |
| blaze's matter | 0.1, 0.5, 30.0 | `data/rftoolsutility/recipe/minecraft_blaze.json` |
| a new mob clears the stores | yes | `SpawnerTileEntity.testSyringe` (`prevMobId` differs → `clearMatter()`) |
| the gate's order | matter first, power second | `SpawnerTileEntity.tickServer` |
| the spend | each store − its own amount | the same tick's `withMatter(i, …)` loop |

### Red → green (unit, verbatim)

The five new runtime rules were written behind `todo!()` bodies first (`slice7-red.log`):

```
failures:
---- runtime::tests::a_spawn_needs_every_matter_type_the_recipe_names stdout ----
thread 'runtime::tests::a_spawn_needs_every_matter_type_the_recipe_names' (3088119) panicked at src/runtime.rs:586:5:
not yet implemented
---- runtime::tests::a_spawn_pays_the_recipe_once_the_matter_is_there ...
thread '...' (3088120) panicked at src/runtime.rs:608:5:
not yet implemented
---- runtime::tests::a_spawn_spends_each_matter_type_by_its_own_amount ...
---- runtime::tests::a_spawn_with_power_but_no_matter_keeps_its_buffer ...
---- runtime::tests::matter_accumulates_up_to_the_spawners_storage ...

test result: FAILED. 62 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out
```

Green: **69 passed; 0 failed** (`sdk-test.log`) — 44 `runtime` tests (the tank's, the crafter's,
the spawner's and the four new matter rules) and 25 glue tests in `src/lib.rs` (the tank's, the
crafter's, the spawner's and the two new store rules: `a_new_mob_clears_the_matter_the_previous_one_was_fed`,
`a_beamer_send_adds_to_each_store_and_caps_it`). fmt clean, clippy `-D warnings` 0 warnings
(`slice7-clippy.log`, `sdk-clippy.log`).

### In game (evidence under `../_shared/evidence/rftoolsutility-e2e/`)

`tools/e2e.sh` phases A–AG, one lease per leg, every log ends with its exit:

| log | verdict |
|---|---|
| `e2e-slice7-red.log` | **FAIL: the spawner line did not parse: … needs=- spawned=0** — the same script on the **0.2.5** package, whose readout has no `matter=` field and whose machine has no store at all (`EXIT:1`) |
| `e2e-slice7.log` | **PASS**, `EXIT:0` — A–AG |
| `e2e-slice7-selftest-gate.log` | **FAIL: the spawner holds 800 FE, expected 0**, `EXIT:1` (Y1 inverted: it must have spawned) |
| `e2e-slice7-selftest-spawn.log` | **FAIL: the spawner holds 0 FE, expected 800**, `EXIT:1` (Z inverted) |
| `e2e-slice7-selftest-dimension.log` | **FAIL: the End's spawner spawned 1 zombie(s) in its own dimension, expected 0**, `EXIT:1` (AG inverted; slice 6 stays green) |
| `e2e-slice7-control.log` | **FAIL: the tank item did not register**, `EXIT:1` (the unmodded Nightly) |
| `e2e-slice7-selftest-{fill,restore,craft,tiers}.log` | each `EXIT:1` with its own `FAIL:` (B, K, Q, AC inverted) — the earlier legs still fail as designed |

`check-e2e-logs.py rftoolsutility-e2e --strict` → `logs=34 ok=34 missing-exit=0 bad-exit=0`.

The slice's phases, each asserting a parsed value (the transcript is in the log):

| phase | what it asserts (observed) |
|---|---|
| **W** | a spawner placed from its item reads `energy=0/200000 mob=none cost=- needs=- matter=0,0,0 spawned=0`, has `facing=west`, and the 70 carried recipes are in force |
| **Y1** | **800 FE with `matter=0,0,0`** for 60 ticks spawns **0** zombies, keeps `energy=800` and `matter=0,0,0`, `spawned=0` — 0.2.5 spent the FE and spawned here (`e2e-slice7-red.log` has no such phase; the selftest inverts it) |
| **Y2** | `matter=0.1,0.2,19.9` of the zombie's `0.1/0.2/20` with 800 FE spawns nothing and leaves both the FE and the stores untouched |
| **Z** | `matter=0.1,0.2,20` exactly + 800 FE → **one** zombie within 5 blocks, `energy=0`, `matter=0,0,0`, `spawned=1` |
| **Z2** | `1.1/1.2/21` (one more than each amount) + 800 FE → one more zombie and `matter=1,1,1`: each store is short by **its own** amount; re-setting the same mob keeps `1,1,1` |
| **AA** | `set_mob minecraft:blaze` reads `matter=0,0,0` (a new mob clears the zombie's stores) and `needs=0.1,0.5,30`; feeding those and 1,000 FE spawns one blaze, `energy=0`, `matter=0,0,0`, `spawned=3` |
| **AB** | stores at `1.5/2.5/42` with 0 FE, a survival break drops a spawner whose item carries `minecraft:block_entity_data`; placing it back reads the same mob, cost, needs **and** `matter=1.5,2.5,42`, `spawned=3` |
| **AG** | only the End spawner is given a mob, `matter=0.1,0.2,20` and 800 FE: it spawns one zombie at its own spawn point and reads `energy=0 matter=0,0,0 spawned=1`, while the Overworld twin reads `energy=0 spawned=0 matter=0,0,0` |

Counterfactuals: **Y1** is the same machine at the same 800 FE as **Z** with one difference (the
stores are empty), **Y2** is one unit short of **Z**, **Z2** is one unit over, **AA**'s new mob
against **Z2**'s same mob differ only in whether the stores are cleared, and **AG**'s unfed twin
differs only in what it was fed. `SELFTEST_AT=gate` inverts Y1 (the spawner must have spawned with no
matter), `SELFTEST_AT=spawn` inverts Z (nothing spawned, nothing spent), `SELFTEST_AT=dimension`
inverts AG; each leg fails as designed, so each assertion can fail. `CONTROL=1` runs the same script
on the unmodded Nightly, where the package's items do not exist.

### Measured traps this slice cost (harness and engine)

1. **A readout taken after a feed is a spawn, not a read.** The tick runs between a command and the
   readout's own 12-tick wait, so feeding matter into a machine that already holds the FE spawns
   during the wait and the readout shows `energy=0, spawned=1` (Z failed on exactly this on the
   first green run). Every phase now empties the buffer (`set_energy … 0`) **before** it feeds, and
   asserts the stores, then charges.
2. **A float leftover must be binary-exact to be asserted as a string.** `0.6 − 0.1` and
   `0.7 − 0.2` round to `0.5` in f32, but that is a tie at the midpoint and one ULP away from
   `0.49999997`; `1.1 − 0.1 = 1.0`, `1.2 − 0.2 = 1.0` and `21 − 20 = 1.0` are exact. Z2 therefore
   feeds `1.1/1.2/21` and asserts `matter=1,1,1`.
3. **`<m0:number>` is the argument kind.** The usage's `number` delivers
   `CommandArgumentValue::Number(f64)` (the same kind `undergarden` uses); `word` would have
   arrived as a `String` and needed its own parser.
4. **A mod fluid is not the same as a stored float.** The pin can declare fluids
   (`Registrar::declare_fluid`) and block tanks (`block_fluids`), so the brief's fluid shape was
   expressible — the reason it is not used is that the artefact has no matter fluid (see above).

### Deviations and content adaptations

- **No fluid tanks, three float stores** (the section above): a user notices nothing — the machine
  reads, spends and carries exactly the amounts its recipe names.
- **The stores are filled by a command, not a beamer.** `MatterBeamerTileEntity` (200,000 FE, 1,000
  matter a tick, the wrench-linked destination, `beamRfPerObject`) is a different block family and a
  later slice; `/rftoolsutility matter` writes what its send would add. The beamer's item-match
  factor and infusing bonus are carried as "already scaled", and the infusing path stays the
  crafter's recorded `rftoolsbase` gap.
- **The matter is not cleared when a syringe is pulled**, because there is no syringe item or slot
  (slice 4's gap): the mob is set by `/rftoolsutility mob`, and a *changed* mob clears the stores.
- No GUI, no comparator, no TOP/Jade, no tooltip — the family's standing gaps.

### Upstream defects fixed

None new: the tick pays energy and matter only after `spawn_mob` answered (slice 4's fix, now
applied to the stores as well), and the gate's order is upstream's own. The slice-4 record
(`SpawnerTileEntity` pays before it knows the mob exists) stands.

### Coverage after slice 7

| family | status |
|---|---|
| `rftoolsutility:spawner` | tested — W (place/read back/facing), AB (the item carries the machine) |
| the matter gate (three stores, the per-type check, the spend) | tested — Y1, Y2, Z, Z2, AA, AB, AG |
| the store's accumulation and 6,400 cap | unit-tested (`matter_accumulates_up_to_the_spawners_storage`) and exercised through `matter` in every phase |
| the food/mob recipes as the gate's data | tested — `needs=0.1,0.2,20` (zombie) and `0.1,0.5,30` (blaze) drive Z/AA |
| the matter beamers | not ported (a different block family; `/rftoolsutility matter` stands in) |
| the syringe, the GUI, six-way facing, the infusing discount | not ported (slice 4's gaps) |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

### Artifact

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `rftoolsutility.hmod` (port root) | 115,039 B | `f476e13730dc1888e823c84002135ba4260f5cb4e19f4c64b48d5d5c6253bbcb` | 152 |

Built twice with `rm -f Cargo.lock` before each (`cmp` identical), **no packed `Cargo.lock`**,
`check` green (`rftoolsutility 0.2.6 (native-mod, format
native-1/0a875da91dbb0fdcb7ba8c42c11341aa229af51a/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for
26.3)`), `fits --build 0a875da` → `0a875da  made for`, `fits --build 0b14f8a` → `0b14f8a  fits`
(both logs print the artifact's sha first), `verify-sdk-package.py rftoolsutility` → **1/1**, and
the launcher's `native-build` compiled the mod against the pin (`Compiling rftoolsutility (1 of 1)`,
`native-0a875da-2600432bf5cd`) for the in-game legs.

**Why the bytes are the second pair.** The first green 0.2.6 build (`0363a425…`, 114,875 B) was
made before this section and the README's slice-7 text existed; `README.md` is packed inside the
package, so adding it changed the artifact. The delivered package is the pair above, and **every
in-game leg in the table was re-run on a game the launcher built from these bytes** (`Compiling
rftoolsutility (1 of 1)` → `native-0a875da-2600432bf5cd`); the red leg alone ran on the previous
(0.2.5) export, which is the point of it.

## Slice 8 — the matter beamer (2026-11-22)

**Chosen from the port's own gap list** — `## Slice 4 — the spawner` → `### Deviations and content
adaptations` and the slice-7 coverage row, verbatim:

> the matter beamers (`MatterBeamerTileEntity`) | not ported | a different block family (200,000 FE,
> 1,000 matter a tick, the wrench-linked destination, the beamer's own infusing bonus);
> `/rftoolsutility matter` writes what a beamer's send would add

and the slice-7 report's `## Next slice` (the same block, with the assertion this slice runs). It is
the **feed** half of the matter gate slice 7 proved: the gate consumes three stores and nothing
filled them from a machine. It fits the **machines** tier exactly — a block entity with a one-slot
container, a 200,000 FE buffer, a redstone input and a server tick — with no GUI, no renderer and no
new family.

**Pin:** Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, mod API `0.2`, game 26.3. **Version 0.2.7.**

**Read for this slice** (analysis only; the clone at `/tmp/opencode/rftoolsutility-src`, branch
`1.21_neo`, commit `f799314007d68042df8504e83cd549360f2627c7`): `MatterBeamerTileEntity.java`
(`tickServer`'s power gate, its `TICKTIME` 20, `numblocks`/`rf` from the configuration and the
infusing factor, the `addMatter` call, `enableBlockGlow`/`disableBlockGlow`, `setDestination`,
`wrenchUse`), `MatterBeamerBlock.java` (`RotationType.NONE`, `BlockStateProperties.LIT`),
`SpawnerTileEntity.useWrench` (`maxBeamDistance` 8, "Destination set!"), `SpawnerConfiguration`
(`BEAMER_MAXENERGY` 200,000, `BEAMER_RECEIVEPERTICK` 1,000, `beamRfPerObject` 2,000,
`beamBlocksPerSend` 1, `maxBeamDistance` 8, `maxMatterStorage` 6,400), `SpawnerRecipes.MobSpawnAmount.match`
(an item/tag object yields 1.0; the empty object is "living?" and 1.5/1.0/0.5 by tag) and the four
`data/rftoolsutility/tags/item/living/*.json`.

### The artefact says the material match is the beamer's half

Slice 4 recorded that a recipe's `item1..3.object` is **not** parsed as a 26.3 ingredient: the files
carry 1.21.1's `{"item": …}`, a tag, and in `item2` a *list*, and 26.3's ingredient parser rejects
both. That stayed true — but the beamer is what the field is **for**: `addMatter` maps the beamed
item to one of the three types (`mobData.getItem(i).match(stack)`) and adds
`blocks * factor * 3 / (3 - infusion)`. So the port now reads the sources itself (the same reader
that already looked at `amount`), keeps them on the machine's recipe copy, and matches the material
in `runtime::matter_for`. The four `living/*` tag files are carried from the jar and read through the
game's own tag table, because the empty `object` is upstream's "living?" and its factor is those
tags — no new ingredient form is invented, and nothing in the 70 carried files changes.

### What it does

- **The block.** `rftoolsutility:matter_beamer` on the **`minecraft:iron_block`** template (a full
  cube, no rotation — upstream's `RotationType.NONE`), with a declared `lit` state (`false`/`true`,
  default `false`) that picks the jar's own dark and lit cubes (`machinebeameroff` /
  `machinebeamer`), and McJtyLib's `STANDARD_IRON` properties.
- **The container.** One material slot (`SLOT_MATERIAL`), automation inserts into it and takes
  nothing out; no menu.
- **The energy.** 200,000 FE (`BEAMER_MAXENERGY`), a fill of at most 1,000 a call
  (`BEAMER_RECEIVEPERTICK`) on every face, in the engine's `energy` int; the beamer only consumes.
- **The tick.** A beamer without a redstone signal is dark and does nothing, every tick. On a send
  tick (`game_time % 20 == 0`, upstream's `TICKTIME` 20) with a linked spawner and a material its
  recipe takes, one block moves and the buffer pays 2,000 FE (`beamRfPerObject`); the store the
  material feeds gains `blocks * factor` (`addMatter`, capped at 6,400).
- **The destination.** Upstream's wrench links a beamer to a spawner: within `maxBeamDistance` 8
  blocks, only a spawner, and a second wrench on the beamer clears it. The port's
  `/rftoolsutility link <pos> <dest>` is the same rule (a destination that is the beamer itself
  unlinks it), because this port carries no wrench item — `rftoolsbase`'s, which the parent port
  does not ship. The tick also clears a destination whose block is no longer a spawner, as
  upstream's `setDestination(null)` does.
- **The glow.** `lit=true` while a send just happened, `false` when the signal is gone, the slot is
  empty or the destination is lost (`enableBlockGlow`/`disableBlockGlow`); an empty buffer or a
  refused material leaves it as it was, as upstream's own returns do.
- **The item carries the machine.** A broken beamer's own item takes the block entity's saved
  compound (`dest`, `sent`, `energy`, `Items`) as vanilla's `minecraft:block_entity_data`
  (upstream's `collectImplicitComponents`).
- **The readout.** `/rftoolsutility beamer <pos> [nonce]` prints
  `energy=<held>/200000 dest=<[x, y, z]|none> slot=<item>x<count>|-> sent=<n> lit=<true|false>`;
  `/rftoolsutility link <pos> <dest>` is the wrench; `buffer <pos> 0 <item> <count>` and
  `energy <pos> <amount>` reach the beamer's slot and buffer. `tools/parse_beamer.py` parses the
  readout.

### Numbers and where they came from

| Number | Value | Source |
|---|---:|---|
| energy buffer | 200,000 FE | `SpawnerConfiguration.BEAMER_MAXENERGY` |
| fill per call | 1,000 FE | `SpawnerConfiguration.BEAMER_RECEIVEPERTICK` |
| one block's cost | 2,000 FE | `SpawnerConfiguration.beamRfPerObject`; `rf = beamRfPerObject * numblocks` |
| blocks per send | 1 | `SpawnerConfiguration.beamBlocksPerSend` (`(int)(1 * 1.01)` uninfused) |
| send period | 20 ticks | `MatterBeamerTileEntity.TICKTIME` |
| link distance | 8 blocks | `SpawnerConfiguration.maxBeamDistance` (`SpawnerTileEntity.useWrench`) |
| one store's capacity | 6,400 | `SpawnerConfiguration.maxMatterStorage` (slice 7's constant) |
| a material object's factor | 1.0 | `MobSpawnAmount.match` (`object.test(stack) ? 1.0f : null`) |
| the living factors | 1.5 / 1.0 / 0.5 | the same `match`, from `living/highyield`, `averageyield`, `lowyield` |
| the zombie's sources | rotten flesh; dirt/gravel/sand; living | `data/rftoolsutility/recipe/minecraft_zombie.json` |
| block properties | hardness 2.0, blast 2.0, correct tool, metal sound | `BlockBuilder.STANDARD_IRON` |
| the beamer's own recipe | redstone/glowstone/redstone around a machine frame | `data/rftoolsutility/recipe/matter_beamer.json` (carried) |

### Red → green (unit, verbatim)

The eight new runtime rules were written behind `todo!()` bodies first (`slice8-red.log`):

```
---- runtime::tests::a_material_feeds_the_type_its_object_names stdout ----
thread 'runtime::tests::a_material_feeds_the_type_its_object_names' (599551) panicked at src/runtime.rs:712:5:
not yet implemented
---- runtime::tests::a_tag_object_matches_a_member_and_refuses_another stdout ----
thread 'runtime::tests::a_tag_object_matches_a_member_and_refuses_another' (599565) panicked at src/runtime.rs:712:5:
not yet implemented
---- runtime::tests::the_living_form_takes_its_yield_tags_factor stdout ----
thread 'runtime::tests::the_living_form_takes_its_yield_tags_factor' (599585) panicked at src/runtime.rs:712:5:
not yet implemented
---- runtime::tests::a_send_moves_one_block_for_two_thousand_fe stdout ----
thread 'runtime::tests::a_send_moves_one_block_for_two_thousand_fe' (599557) panicked at src/runtime.rs:743:5:
not yet implemented
---- runtime::tests::a_send_without_the_power_moves_nothing stdout ----
thread 'runtime::tests::a_send_without_the_power_moves_nothing' (599558) panicked at src/runtime.rs:743:5:
not yet implemented
---- runtime::tests::a_send_adds_the_blocks_times_the_factor stdout ----
thread 'runtime::tests::a_send_adds_the_blocks_times_the_factor' (599555) panicked at src/runtime.rs:743:5:
not yet implemented
---- runtime::tests::a_link_reaches_eight_blocks_and_no_further stdout ----
thread 'runtime::tests::a_link_reaches_eight_blocks_and_no_further' (599549) panicked at src/runtime.rs:750:5:
not yet implemented
---- runtime::tests::a_send_is_due_every_twenty_ticks stdout ----
thread 'runtime::tests::a_send_is_due_every_twenty_ticks' (599556) panicked at src/runtime.rs:757:5:
not yet implemented

test result: FAILED. 69 passed; 8 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

Green: **77 passed; 0 failed** (`slice8-runtime-green.log`).

The glue's rules were the same shape, with their bodies `todo!()` first (`slice8-glue-red.log`):

```
---- tests::a_spawner_file_reads_the_material_sources_its_objects_name stdout ----
thread 'tests::a_spawner_file_reads_the_material_sources_its_objects_name' (614750) panicked at src/lib.rs:1067:5:
not yet implemented
---- tests::a_spawner_file_reads_its_entity_power_and_matter stdout ----
thread 'tests::a_spawner_file_reads_its_entity_power_and_matter' (614749) panicked at src/lib.rs:1067:5:
not yet implemented
---- tests::a_spawner_value_writes_back_as_the_recipe_it_came_from stdout ----
thread 'tests::a_spawner_value_writes_back_as_the_recipe_it_came_from' (614753) panicked at src/lib.rs:1067:5:
not yet implemented
---- tests::a_destination_round_trips_through_the_saved_compound stdout ----
thread 'tests::a_destination_round_trips_through_the_saved_compound' (614743) panicked at src/lib.rs:1100:5:
not yet implemented
---- tests::the_beamer_readout_is_the_line_the_harness_parses stdout ----
thread 'tests::the_beamer_readout_is_the_line_the_harness_parses' (614758) panicked at src/lib.rs:1093:5:
not yet implemented

test result: FAILED. 75 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Green: **80 passed; 0 failed** (`slice8-green.log`, `sdk-test.log`) — 52 `runtime` tests (the tank's,
the crafter's, the spawner's, the matter gate's and the eight new beamer rules) and 28 glue tests in
`src/lib.rs` (the tank's, the crafter's, the spawner's and the three new ones:
`a_spawner_file_reads_the_material_sources_its_objects_name`,
`a_destination_round_trips_through_the_saved_compound`,
`the_beamer_readout_is_the_line_the_harness_parses`). fmt clean, clippy `-D warnings` 0 warnings
(`slice8-clippy.log`, `sdk-clippy.log`).

### In game (evidence under `../_shared/evidence/rftoolsutility-e2e/`)

`tools/e2e.sh` phases A–AH, one lease per leg, every log ends with its exit:

| log | verdict |
|---|---|
| `e2e-slice8.log` | **PASS**, `EXIT:0` — A–AH, on a game the launcher built from the delivered bytes (`Compiling rftoolsutility (1 of 1)` → `native-0a875da-129559a0b8f3`) |
| `e2e-slice8-selftest-beamer.log` | **FAIL: the spawner's matter reads '2,0,0', expected '0,0,0'**, `EXIT:1` (AH2 inverted) |
| `e2e-slice8-control.log` | **FAIL: the tank item did not register**, `EXIT:1` (the unmodded Nightly) |

`check-e2e-logs.py rftoolsutility-e2e --strict` → `logs=37 ok=37 missing-exit=0 bad-exit=0` (the 34
earlier legs still green, and the three new ones).

The slice's phases, each asserting a parsed value (the transcript is in the log):

| phase | what it asserts (observed) |
|---|---|
| **AH** | a beamer placed from its item reads `energy=0/200000 dest=none slot=- sent=0 lit=false` and its capacity is 200,000 |
| **AH1** | a link to a stone block is refused (`the destination [200, 101, -14] is not a spawner.`), a link to a spawner 4 blocks away is made (`sends to the spawner at [200, 101, -16]`), a 12-block target is refused (`destination distance is too far! (maxBeamDistance is 8 blocks)`) with the destination unchanged, and linking the beamer to itself unlinks it (`is unlinked.`), after which it is linked again |
| **AH2** | the spawner's mob is set to `minecraft:zombie` (its 800 FE recipe, `needs=0.1,0.2,20`) and 4,000 FE + 8 rotten flesh (item1) are **two sends**: the beamer reads `energy=0 slot=minecraft:rotten_fleshx6 sent=2 lit=true`, the store `matter=2,0,0`, `spawned=0` |
| **AH3** | 8 dirt (item2, the list object) make two more sends: `slot=minecraft:dirtx6 sent=4`, stores `2,2,0` |
| **AH4** | 8 beef (item3, the living form, highyield) make two more: `slot=minecraft:beefx6 sent=6`, stores `2,2,3` — each block is **1.5** |
| **AH5** | 8 cobblestone (no object names it) are refused: `slot=minecraft:cobblestonex8`, `energy=4000` kept, `sent=6`, stores unchanged `2,2,3` |
| **AH6** | 8 rotten flesh at **0 FE** send nothing: slot and stores unchanged, `sent=6` |
| **AH7** | with the redstone block removed and 4,000 FE, nothing sends and `lit=false` (the signal gates it) |
| **AH8** | with the signal back and the slot emptied, `slot=-` and `lit=false` (an empty slot clears the glow) |
| **AH9** | a survival break drops a beamer whose item carries `minecraft:block_entity_data` `{Items: [{count: 1, Slot: 0b, id: "minecraft:rotten_flesh"}], id: "rftoolsutility:matter_beamer", dest: [200, 101, -16], sent: 6, energy: 800}`, and placing it back reads `dest=[200, 101, -16] slot=minecraft:rotten_fleshx1 energy=800 sent=6` |

Counterfactuals: **AH5** (a material the recipe does not name) against AH2–AH4 (three it does),
**AH6** (the same machine and material at 0 FE as AH2's 4,000), **AH7** (the same machine and
material without a signal as AH2's), and **AH8**'s empty slot against AH2's filled one; each differs
in one input only. `SELFTEST_AT=beamer` inverts AH2's store growth and the leg fails as designed;
`CONTROL=1` runs the same script on the unmodded Nightly, where the package's items do not exist.

### Measured traps this slice cost (harness and engine)

1. **A material loaded while the buffer is still charged is itself a send.** AH6 first put the
   material in while AH5's 4,000 FE was still there: the tick spent one during the command's own
   12-tick wait, and the phase read `sent=7`, `slot=…x7`. The order every phase now uses is *empty
   the buffer, then load the material, then charge* — slice 7's "a readout taken after a feed is a
   spawn" trap with the beamer's own send in place of the spawn. AH8 was the same shape (replacing
   the signal while the slot still held material).
2. **`sent` is the count, so the sends are bounded by the energy, not by the wait.** Eight blocks
   and 4,000 FE are exactly two sends however many 20-tick boundaries the wait spans (a third due
   tick finds 0 FE and sends nothing), so the leg does not depend on where a `ctl wait 40` starts.
3. **`ctl collect` walks.** The brief's warning is real: the walk can leave the arena floor. The
   drop is picked up by teleporting onto the item entity's own `pos` (`pick_up_at`, from
   `ctl entities 12 minecraft:item`), not by `collect`.
4. **A saved empty list is not a destination.** `read_pos` takes exactly three ints; an unlink
   removes the key (`Compound` is a `BTreeMap` whose key type the API does not export, so the
   removal goes through the same `Tag::String(name.into())` payload the reads use).
5. **The fits tool false-fails on a variant named after an API type.** `WireSource::Item` made
   `fits --build 0b14f8a` answer *"the type `Item` can no longer be built with a literal, and
   `src/lib.rs:1054` builds it"* — the same class as the brief's `Block` warning. Renaming the
   variants (`Material`/`Tagged`) cleared it.
6. **The GitHub releases API rate-limits `fits`.** `cargo horizon-mod fits --build <other>` fetches
   four pages of releases per call; a burst of retries exhausts the 60/hour quota and the tool then
   prints `status code 403` (exit 2). The quota is per IP and resets hourly; one attempt per
   invocation, spaced, is what works.

### Deviations and content adaptations

- **The wrench is a command.** `rftoolsbase`'s wrench is the parent mod's item (this port does not
  ship it), so `/rftoolsutility link <pos> <dest>` performs the two clicks with the same rules (only
  a spawner, at most `maxBeamDistance` 8, a second click on the beamer clears it). A user notices:
  there is no wrench gesture; the link is a command.
- **The 20-tick clock is the tick's own game time** (`game_time % 20 == 0`) rather than a saved
  counter, so a tick that sends nothing writes nothing (a saved counter would dirty the machine
  every tick). A user notices: several beamers send on the same tick, and the first send after a
  chunk loads comes at the next multiple of 20, not 20 ticks after loading.
- **The infusing bonus is not ported.** Upstream multiplies `beamBlocksPerSend` by
  `(1.01 + infused * 2)` and divides the cost by `(4 - infused) / 4`; the infusable path needs
  `rftoolsbase`'s `DefaultInfusable` (the crafter's recorded gap), so the uninfused values (one
  block, 2,000 FE) are the rule and the formula is documented on the constants.
- **The `c:` memberships of the living tags are the pack's.** The four tag files are carried
  verbatim; the `c:crops`/`c:foods`/`c:eggs`/`c:mushrooms`/`c:seeds` they name are not declared by
  the prepared 26.3 root (this port declares only the five `c:` tags its recipes name), so with this
  port alone a living item yields through the vanilla tags and the ids the files list directly —
  which is what AH4 proves with `minecraft:beef` (1.5).
- **No GUI, no comparator, no TOP/Jade, no tooltip** — the family's standing gaps; the beamer's
  GUI (`GuiMatterBeamer`) and its renderer (`MatterBeamerRenderer`, the beam itself) are not ported.
- **The carried recipe is not crafted in game.** Its middle ingredient is
  `rftoolsbase:machine_frame`, which only the parent port registers (the tank's, the crafter's and
  the spawner's own gap); the e2e gives the block instead.

### Upstream defects fixed

- **The beamer burns the FE before it knows the material is wanted.**
  `MatterBeamerTileEntity.tickServer` computes `rf`, returns if the buffer is short, calls
  `energyStorage.consumeEnergy(rf)` and *then* `spawnerTileEntity.addMatter(...)`: an item the
  spawner's recipe does not name costs the full 2,000 FE for nothing (the same class as slice 4's
  "the spawner pays before it knows the mob exists"). **The port spends both or neither**: the
  send is decided first (`beam_send`: the material's type, its factor, the cost and the buffer),
  the store is written, and only then are the FE and the stack taken. AH5 is the proof (8
  cobblestone at 4,000 FE: `sent=6`, `energy=4000`, the item kept).
- **A zero-yield living item is consumed for nothing.** A recipe's empty `object` returns a factor
  of 0.0 for an item in none of the `living/*` tags, and `addMatter` accepts it (`0.0 * blocks`),
  so upstream deletes the item and the FE and adds nothing. The port's `matter_for` returns `None`
  at factor 0 (nothing is sent), which is the same rule for a material the recipe does not name.

### Coverage after slice 8

| family | status | evidence / why not |
|---|---|---|
| `rftoolsutility:matter_beamer` | tested | AH: placed from its item, read back, `lit` state; AH9: the item carries the machine |
| the material match (an item, a list, a tag, the living form and its factors) | tested | AH2 (item), AH3 (list), AH4 (living, 1.5); the unit tests `a_tag_object_matches_a_member_and_refuses_another` and `the_living_form_takes_its_yield_tags_factor` (a tag object, and a zero yield refused) |
| the send (one block, 2,000 FE, `blocks × factor`, the 6,400 cap) | tested | AH2–AH4: 4,000 FE is two sends, the slot falls by two and each store takes its own amount; unit `a_send_adds_the_blocks_times_the_factor` |
| the refusals (a material the recipe does not name, no FE, no signal, an empty slot) | tested | AH5, AH6, AH7, AH8 — each leaves the slot, the buffer and the stores as they were |
| the destination (the link's distance and kind, an unlink, a lost destination) | tested | AH1: a stone block refused, a 12-block target refused, the beamer's own position unlinks, a spawner links; AH9: it travels on the item |
| the 20-tick clock and the `lit` state | tested | AH2: `sent=2` and `lit=true` after 40 ticks; AH7/AH8: `lit=false` without a signal / with an empty slot; unit `a_send_is_due_every_twenty_ticks` |
| the 200,000 FE buffer and its 1,000-a-call fill | tested | AH: capacity 200000 and the `energy` command's clamp; unit `a_beamer_takes_energy_on_every_face_and_gives_none_out` |
| `recipe:minecraft:crafting_shaped` | blocked(`rftoolsbase:machine_frame`) | the beamer's own carried recipe needs the parent port's item; the e2e gives the block instead |
| the GUI (`GuiMatterBeamer`) and the beam renderer (`MatterBeamerRenderer`) | not ported | a menu and a renderer — outside this tier's slice |
| the wrench item | not ported | `rftoolsbase`'s; `/rftoolsutility link` is the same rule |
| the infusing bonus | not ported | needs `rftoolsbase`'s infusable path (the crafter's gap) |
| the matter transmitter/receiver/booster | not ported | the teleporter family, a later slice |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

### Artifact

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `rftoolsutility.hmod` (port root) | 129,078 B | `2f4706bc8c2e35c148df11762cc2219ddca249fbefa23e625341e93c1d22adaf` | 167 |

Built twice with `rm -f Cargo.lock` before each (`cmp` identical; `slice8-build-1.log`,
`slice8-build-2.log`), **no packed `Cargo.lock`**, `check` green (`rftoolsutility 0.2.7 (native-mod,
format native-1/0a875da91dbb0fdcb7ba8c42c11341aa229af51a/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for
26.3)`), `fits --build 0a875da` → `0a875da  made for`, `fits --build 0b14f8a` → `0b14f8a  fits`
(both logs print the artifact's sha first), `verify-sdk-package.py rftoolsutility` → **1/1**, and
the launcher's `native-build` compiled the mod against the pin (`Compiling rftoolsutility (1 of 1)`,
`native-0a875da-129559a0b8f3`) for the in-game legs.

**Why the bytes are the third pair.** The first green build (`df7a8f29…`, 128,806 B) was made before
the `fits` run showed the `WireSource::Item` false-fail; renaming the variants changed the source
(`e8ce57d7…`, 128,647 B), and the README's slice-8 text (packed in the package) changed it again to
the delivered pair above. Every in-game leg in the table was re-run on a game the launcher built
from these bytes (`native-0a875da-129559a0b8f3`).

## Slice 9 — the teleporter family (2026-11-24)

**Chosen from the port's own gap list** — `## Slice 8 — the matter beamer` → `### Coverage after
slice 8`, verbatim:

> the matter transmitter/receiver/booster | not ported | the teleporter family, a later slice

and the dispatch's Step-0 instruction to read `MatterTransmitterTileEntity`,
`MatterReceiverTileEntity`, `MatterBoosterTileEntity` and the four links first. It fits the
**machines** tier exactly: three blocks, two block entities with energy buffers and a server tick,
no GUI, no renderer, no entity AI. **Pin** `0a875da…` / `e59da909…`, mod API `0.2`, game 26.3.
**Version 0.2.8.**

### The brief said a matter mover; the artefact says a player teleporter

The dispatch proposed "two linked machines with matter in the sender … the sender's matter fell
by exactly the transported amount and the receiver rose by it". Reading the jar shows that family
does **not** exist in this mod: `MatterTransmitterTileEntity.tickServer` searches for a **player**
in the beam above the block (`createBeamBox`, x..x+1 / y+1..y+3 / z..z+1), charges the dialed
trip's RF over `calculateTime` ticks, and calls
`TeleportationTools.performTeleport(player, dest, …)` — `Entity.teleport` to another dimension or
`player.teleportTo` in the same one. `MatterReceiverTileEntity` is a **destination marker**: it
registers a `GlobalPos` with the world's destination table and reports readiness (two clear blocks
above, `rfPerTeleportReceiver` 5,000 FE). `MatterBoosterBlock` is a plain facing marker whose only
effect is to make a teleport *boosted* (20,000 FE mitigates a weak receiver). The four links —
`MatterTransmitterData`, `MatterReceiverData`, `TeleportDestination`, `TransmitterInfo` — are the
records for the dial, the receiver id, the destination `GlobalPos` and a listed transmitter; none
describes a matter buffer moving between machines. The artefact wins (a brief may not invent a
mod's contents), so the fail-capable assertion is the **player's position**, not a matter amount.

### The pin can express it (so this is not `blocked`)

Measured against `ports/_shared/sdk/0a875da`, not the historical gap table: `PlayerRef::entity()`
and `From<PlayerRef> for EntityRef`; `EntityRef::teleport_to` / `teleport_to_level(host, dimension,
pos, rotation)` (the latter explicitly handles a cross-dimension player move); `LevelRef::
entities_in` and `online_players`; `LevelsHost::list_levels` + `DimensionKey::id`. All three pieces
the dispatch named as potential stop conditions exist: a cross-level lookup, a dimension transfer
and a player query. The upstream console-command route is therefore **not** used — the teleport is
the SDK's entity level transfer, not a faked chat `/tp`.

### What it does

- **The blocks.** `rftoolsutility:matter_transmitter` and `matter_receiver` on the full-cube
  `minecraft:iron_block` template (their jar blockstates have a single empty variant) with the
  jar's own models/textures (the `rftoolsbase:` base textures re-pointed into this namespace);
  `matter_booster` on the same template (its six-way `facing` is cosmetic and dropped — a recorded
  deviation). All three items, recipes, advancements and loot tables are carried from the jar.
- **The transmitter.** A 200,000 FE buffer filling at 1,000 a call (`TRANSMITTER_MAXENERGY` /
  `TRANSMITTER_RECEIVEPERTICK`), a dialed destination (dimension id + coordinate), a beam-box
  player search, a charge/interrupt state machine and an 80-tick cooldown. The start cost and time
  are upstream's exact formulas: same dimension `5,000 + 10·dist` FE (capped at 100,000) over
  `5 + 10·dist/1000` ticks (capped at 50); another dimension a flat 100,000 FE / 50 ticks. The
  start cost is amortized into the per-tick fee (`(cost + 500·(time+1))/(time+1)`, upstream's
  `rfPerTick`). A player leaving the beam or a bad-tick threshold interrupts with no teleport.
- **The teleport.** `EntityRef::teleport_to_level` to the dialed coordinate (+0.5 x/z, +1 y),
  which is `Entity.teleport` across dimensions and `teleportTo` in the same one. An adjacent
  booster with 20,000 FE is the `boosted` safety flag. The destination is dialed directly as a
  `GlobalPos` — upstream's own fallback when a coordinate has no destination-table id — because
  this port carries no dialing-device GUI or global destination registry.
- **The receiver.** A 100,000 FE buffer filling at 500 a call; readiness is the two clear blocks
  above plus 5,000 FE. Its energy and the transmitter's dial/trip state travel on their items as
  `minecraft:block_entity_data` (upstream's `collectImplicitComponents`).
- **The commands** (the harness, as upstream's controls are GUIs): `/rftoolsutility dial <pos>
  <dim> <x> <y> <z>` (`none` clears), `transmitter <pos> [nonce]` and `receiver <pos> [nonce]`.

### Red → green (unit, verbatim)

Five rules were stubs behind their tests first (`runtime::tests::a_cross_dimension_…
panicked at src/runtime.rs:837:5: not yet implemented`):

```
test runtime::tests::a_cross_dimension_dial_costs_one_hundred_thousand_and_fifty_ticks ... FAILED
thread 'runtime::tests::a_cross_dimension_dial_costs_one_hundred_thousand_and_fifty_ticks' (154456) panicked at src/runtime.rs:837:5:
not yet implemented
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 84 filtered out; finished in 0.00s
```

Green: **85 passed; 0 failed** (`sdk-test.log`) — 49 `runtime` tests (the new five: cross-dimension
cost/time, local cost/time and cap, receiver readiness, the beam box, the charge tick) and 36 glue
tests. fmt clean, clippy `-D warnings` 0 warnings (`sdk-clippy.log`).

### Numbers and where they came from

| Number | Value | Source |
|---|---|---|
| transmitter buffer / fill | 200,000 / 1,000 FE | `TeleportConfiguration.transmitterMaxRF` / `transmitterRFPerTick` |
| receiver buffer / fill | 100,000 / 500 FE | `…receiverMaxRF` / `receiverRFPerTick` |
| local start cost / time | 5,000 + 10·dist / 5 + 10·dist/1000 | `TeleportationTools.calculateRFCost` / `calculateTime` |
| cross-dimension start cost / time | 100,000 FE / 50 ticks | `rfStartTeleportDim` / `timeTeleportBaseDim` |
| per-tick / idle / receiver-ready | 500 / 10 / 5,000 FE | `rfTeleportPerTick`, `rfDialedConnectionPerTick`, `rfPerTeleportReceiver` |
| boosted teleport | 20,000 FE | `rfBoostedTeleport` (adjacent booster) |
| beam box | x..x+1, y+1..y+3, z..z+1 | `MatterTransmitterTileEntity.createBeamBox` |
| cooldown on interrupt | 80 ticks | `clearTeleport(80)` |

### Upstream defects fixed / deviations

- **A refused/underpowered start is not half-paid.** Upstream's `startTeleportation` computes the
  cost and the timer, then a shortage only sets an 80-tick cooldown; the port writes the trip state
  only after the buffer covers the start cost. The per-tick `handleEnergyShortage` threshold is
  the port's `bad·2 > total` rather than upstream's configurable `mustInterrupt`, a documented
  simplification (no config file in this port).
- **The dialing device, destination analyzer and GUI are not ported** (a menu and two more blocks):
  `/rftoolsutility dial` writes the same `MatterTransmitterData.destination` the GUI would.
- **No global destination registry / destination ids.** Upstream's `TeleportDestinations` maps a
  `GlobalPos` to an id for the dialing device; the port dials the `GlobalPos` directly (upstream's
  own fallback), so a receiver needs no id. The receiver's `name`/`private`/`players` fields are
  not ported (the GUI that edits them is absent).
- **No syringe, no infusing bonus, no beam renderer, no six-way booster facing** — the standing
  gaps of the machines tier; the booster's `facing` is cosmetic.
- The three carried recipes still need `rftoolsbase:machine_frame`, so the e2e gives the blocks.

### Coverage after slice 9

| family | status | evidence / why not |
|---|---|---|
| `rftoolsutility:matter_transmitter` | tested | AI: placed/read back, 0 FE / dial=none / idle |
| `rftoolsutility:matter_receiver`, `matter_booster` | tested (content) | AI: both placed and read back; booster is a content-only adjacency marker |
| the beam-box player search | tested | AI1: a player in the beam of an **undialed** transmitter is not moved (the control) |
| the dial + cross-dimension-capable teleport | tested | AI2: a dialed transmitter moves the player from the beam to the receiver coordinate; `SELFTEST_AT=teleport` inverts it |
| the start cost / time, the charge and the interrupt | unit-tested | five new `runtime` rules; the in-game leg asserts the arrival, not the intermediate FE |
| the carried content (items, recipes, advancements, loot tables, tags) | carried | regenerated deterministically by `tools/assemble_content.py`; the recipes need `rftoolsbase:machine_frame` as every machine's do |
| the dialing device, destination GUI, syringe, infusing bonus, beam renderer, six-way facing | not ported | a menu, more blocks, `rftoolsbase`'s infusable path and a renderer — outside this tier's slice |

### Measured traps this slice cost

1. **A `word` command argument cannot hold a dimension id.** The SDK's `CommandArgumentKind::
   Word` is ASCII letters/digits plus `_ - . +`; `minecraft:the_end` stopped at the `:` and the
   dispatcher answered *"Expected whitespace to end one argument, but found trailing data"*.
   `dial` now uses `<dim:id>` (which the glue already accepted alongside `String`).
2. **`set_energy` named only the older machines.** A `rftoolsutility energy` on a transmitter
   answered *"No machine there."* until the helper gained a `transmitter_at` branch with
   [`TRANSMITTER_CAPACITY`]. Fixed and recompiled before the final build.
3. **`try_place` does not give the item.** The first in-game run failed with *"no
   rftoolsutility:matter_transmitter in the inventory"*; a fresh machine must use `place_block`
   (which calls `hold` → `/item replace`). The booster also needs a clear placement face — placing
   it beside the transmitter put the latter between the player and the target.
4. **Bash arithmetic rejects `0.5`.** The selftest's inversion used `$((TX_Z + 0.5))` and errored
   without failing the leg; it now writes the half-block as a string (`$TX_Z.5`).

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

### Artifact

- `rftoolsutility.hmod` — **147,852 B**, SHA-256 `60b0204474fda391b15947f1c44a8812ada6710dd9281fa7ed9836e98fbf317d`, version **0.2.8**, no packed `Cargo.lock`.
- Two `cargo horizon-mod build`s with `rm -f Cargo.lock` before each were `cmp`-identical.
- `check` green (`native-1/0a875da…/e59da909…, for 26.3`); `fits --build 0a875da` → `made for`; `fits --build 0b14f8a` → `fits`.
- `verify-sdk-package.py rftoolsutility` → **1/1** (package matches the tree, R9, skills gate).
- The launcher's `native-build` compiled the mod (`Compiling rftoolsutility (1 of 1)`,
  `native-0a875da-8356e2011e25`) for the in-game legs.
- In game (`tools/e2e-teleporter.sh`, a focused leg that reuses `e2e.sh`'s helpers; the full A–AH
  script is green on these bytes too): **PASS**, `EXIT:0` — AI (0 FE / dial=none / idle on all three
  blocks), AI1 (an undialed transmitter with a player in its beam moves nothing — the control),
  AI2 (`dial` to `minecraft:the_end@[203,101,12]`, the player leaves the beam at
  `203.5,102.0,8.5` and arrives at `203.5,102.0,12.5`; the readout reads
  `energy=186600/200000 … dial=minecraft:the_end@[203,101,12] state=cooldown`).
  `CONTROL=1` → **FAIL: Unknown item 'rftoolsutility:matter_transmitter'**, `EXIT:1` (unmodded
  Nightly). `SELFTEST=1 SELFTEST_AT=teleport` → **FAIL: the player left the beam for
  `203.5,102.0,12.5`, expected it to stay at z=8.5**, `EXIT:1` — the inverted assertion proves the
  arrival can fail. `check-e2e-logs.py rftoolsutility-e2e --strict` → `logs=41 ok=41
  missing-exit=0 bad-exit=0`.

## Slice 10 — the destination analyzer (2026-11-24)

**Chosen from the port's own gap list** — `## Coverage after slice 9`, verbatim:

> the dialing device, destination GUI, syringe, infusing bonus, beam renderer, six-way facing | not ported | a menu,
> more blocks, `rftoolsbase`'s infusable path and a renderer — outside this tier's slice

and the earlier gap list's `destination_analyzer` (still unported in `FINDINGS.md` §1's 27 blocks).
It is the next **machines**-tier family in the teleporter row: a passive block that gates a
receiver-status check on the existing transmitter — no GUI, no renderer, no new block entity.

**Pin** `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` / `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`,
mod API `0.2`, game 26.3. **Version 0.2.9.** Class **content**, tier **machines** (ledger).

### The brief said a tile entity with redstone output; the artefact says a passive block

The brief named `DestinationAnalyzerTileEntity` and asked for a redstone output whose strength
reports the dialled receiver's readiness. Reading the jar `rftoolsutility-1.21-7.0.13.jar`
(SHA-256 `615642b1…d3ff`) and the branch `1.21_neo` clone (commit `f799314…`, analysis only)
first shows that class **does not exist** — there is no `*Analyzer*TileEntity.class` in the jar,
and `DestinationAnalyzerBlock.java` extends plain `Block` with only a `FACING` property. The
jar's own manual (`assets/rftoolsbase/patchouli_books/manual/en_us/entries/machines/
destination_analyzer.json`) calls it "a **passive** block" that enables a dialing device or a
transmitter to check the receiving end; the lang line says "red is bad, green ok, yellow is
unknown" of the transmitter's **beam**, not a redstone signal. **The artefact wins** (the same
ruling slice 9 made when its brief called the transmitter/receiver a "matter mover"), so:

- **the readiness rule, read from `MatterReceiverTileEntity.checkStatus()`** (not inferred from
  the block's name): the block at `above()` must be air, then the block at `above(2)` must be air
  (either failure is `DIAL_RECEIVER_BLOCKED_MASK = 0x1`), then the stored power must be at least
  `rfPerTeleportReceiver` `5,000 FE` (else `DIAL_RECEIVER_POWER_LOW_MASK = 0x10`), otherwise
  `DIAL_OK = 0`. `MatterTransmitterTileEntity.checkReceiverStatus()` maps that to
  `STATUS_OK/WARN/UNKNOWN` after resolving the destination level (`LevelTools.getLevel`) and its
  block entity (`w.getBlockEntity(c)`) — the cross-level lookup slices 6 and 9 established.
- **the output strength, read from the same path**: there is **no** vanilla redstone emitter. The
  harness's own `/rftoolsutility analyzer <pos>` reports the transmitter's stored status as
  **15 (green / `ok`), 7 (yellow / `unknown`), 0 (red / `warn`)** — the numeric form of the beam
  colour the lang line describes. The brief's permitted fallback ("read the harness's own analyzer
  readout; assert the parsed value") is what the leg asserts; `ctl block` on the analyzer reads
  only its block id, never a `power`.

### What it does

- **The block.** `rftoolsutility:destination_analyzer` on `minecraft:iron_block` (the teleporter
  family's full-cube template); the jar's six `facing` rotations are cosmetic and dropped as the
  booster's are. Its blockstate, cube model, item model, face texture, recipe, advancement and
  loot table are carried from the jar; the recipe's 1.21.1 ingredient objects and `c:ender_pearls`
  tag are upgraded to 26.3 form by `tools/assemble_content.py`.
- **The gate.** A transmitter with an analyzer in any of its six neighbours now re-checks its
  dialled receiver every 20 ticks (`game_time % 20 == 0`, the same clock upstream's
  `checkReceiverStatusCounter` seeds at 20). The destination level is resolved by dimension id and
  the receiver block entity read there; readiness is that receiver's two clear blocks plus 5,000 FE.
  Without an analyzer upstream forces STATUS_OK (no red beam), so removing one clears the state.
- **The readouts.** `/rftoolsutility analyzer <pos> [nonce]` prints
  `signal=<0|7|15> status=<warn|unknown|ok> dial=<dim@[x,y,z]|none>`; the transmitter's own line
  gains `status=<...>`. `/rftoolsutility energy` now also charges a receiver, so the harness can
  cross the 5,000 FE threshold.

### Red → green (unit, verbatim)

The five pure rules were `todo!()` stubs behind their tests (`slice10-red.log`):

```
test result: FAILED. 85 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out
  runtime::tests::a_ready_receiver_reads_the_full_signal      panicked at src/runtime.rs:931:5: not yet implemented
  runtime::tests::an_underpowered_or_blocked_receiver_reads_no_signal ... panicked at src/runtime.rs:931:5
  runtime::tests::an_unloaded_destination_dimension_reads_the_unknown_signal ... panicked at src/runtime.rs:931:5
  runtime::tests::a_missing_receiver_is_a_warn_not_an_unknown ... panicked at src/runtime.rs:931:5
  runtime::tests::the_analyzer_rechecks_every_twenty_ticks     panicked at src/runtime.rs:937:5
```

The harness line's pure formatter was stubbed behind its own test (`slice10-glue-red.log`):

```
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 90 filtered out
  tests::the_analyzer_readout_is_the_line_the_harness_parses panicked at src/lib.rs:1557:5: not yet implemented
```

Green: **91 passed; 0 failed** (`sdk-test.log`) — 50 runtime + 41 glue. `cargo fmt --check` clean,
`cargo clippy --all-targets --offline -- -D warnings` 0 warnings (`sdk-clippy.log`). Clippy caught
one `assert_eq!(…, true)` in the new test, corrected to `assert!(…)` before the green run.

### In game (evidence under `../_shared/evidence/rftoolsutility-e2e/`)

`tools/e2e-analyzer.sh` — a focused leg that reuses `e2e-teleporter.sh`'s helpers. One lease per
leg, every log ends with its exit. Game built by the launcher from the delivered bytes
(`Compiling rftoolsutility (1 of 1)`, `native-0a875da-c1cea11ff937`).

| log | verdict |
|---|---|
| `e2e-slice10-analyzer.log` | **PASS**, `EXIT:0` — AJ–AJ3 |
| `e2e-slice10-analyzer-selftest.log` | **FAIL: SELFTEST analyzer: the charged receiver reads signal=15, expected it to stay low**, `EXIT:1` |
| `e2e-slice10-analyzer-control.log` | **FAIL: no rftoolsutility:matter_transmitter in the inventory** (unmodded Nightly), `EXIT:1` |
| `e2e-slice10-teleporter.log` | **PASS**, `EXIT:0` — AI–AI2 (the 0.2.8 teleport family on 0.2.9) |

`check-e2e-logs.py rftoolsutility-e2e --strict` → `logs=46 ok=46 missing-exit=0 bad-exit=0`.

The three phases, each asserting a parsed value (verbatim from the green transcript):

```
AJ1: /rftoolsutility: analyzer at [203, 101, 7]: signal=0 status=warn dial=minecraft:the_end@[203, 101, 12] nonce=1
AJ2: /rftoolsutility: analyzer at [203, 101, 7]: signal=15 status=ok dial=minecraft:the_end@[203, 101, 12] nonce=2
AJ3: /rftoolsutility: analyzer at [203, 101, 7]: signal=0 status=warn dial=minecraft:the_end@[203, 101, 12] nonce=3
```

AJ places the transmitter at `[203,101,8]`, the receiver at `[203,101,12]` and the analyzer at
`[203,101,7]`; AJ1 dials the transmitter and reads a low signal while the receiver has 0 FE;
AJ2 charges the receiver to 5,000 FE and reads the full signal; AJ3 puts `minecraft:stone` at
`[203,102,12]` — one block above the receiver — and reads low again. Counterfactual:
`SELFTEST_AT=analyzer` inverts AJ2 and the leg fails as designed, so the charge case can fail.
`CONTROL=1` runs the same script on the unmodded Nightly, where the package's items do not exist.
The cross-level lookup is in code (`level_of` → `host.block_entity(dest_level, dest)`), exercised
in-game here in the destination's own dimension; its cross-dimension behaviour was proved by the
teleporter leg's arrival and by slice 6's two-dimension case.

### Upstream defects fixed / deviations

- **No new upstream defect** was adopted. The readiness order (two clear blocks, then power) is
  upstream's own `checkStatus()`; the port keeps it.
- **No redstone output** (deviation forced by the artefact): a user gets a harness readout instead
  of a wire signal. The artifact has no emitter and no tile entity, so nothing was invented —
  `notes.md` and the README both say so.
- **Four-way/six-way facing**: the analyzer's six cosmetic `facing` rotations are dropped on the
  iron-block template, as the booster's were (adjacency is the only effect).
- The carried recipe needs `rftoolsbase:machine_frame`, so the e2e gives the block, as every
  machine slice has.

### Coverage after slice 10

| family | status |
|---|---|
| `rftoolsutility:destination_analyzer` (block, item, recipe, advancement, loot, texture) | tested — AJ: placed and read back; the carried content is regenerated by `assemble_content.py` |
| the passive-adjacency gate (every 20 ticks) | tested — AJ1/AJ2/AJ3: the dialed transmitter's status changes with the receiver |
| the readiness rule (two clear blocks above + 5,000 FE) | tested — AJ1 (0 FE → warn/0), AJ2 (5,000 FE → ok/15), AJ3 (a block above → warn/0), plus the pure `receiver_ready` unit tests |
| the cross-level receiver lookup | tested in code path (destination level resolved by id); in-game arrival proved by the teleporter leg and slice 6 |
| the signal mapping (green 15 / yellow 7 / red 0) | tested — `a_ready_receiver_reads_the_full_signal`, `an_unloaded_destination_dimension_reads_the_unknown_signal`, `the_analyzer_readout_is_the_line_the_harness_parses` |
| a vanilla redstone emitter on the block | not ported (the artefact has none; the harness readout stands in) |
| the dialing device, destination GUI, syringe, beam renderer, six-way facing, infusing bonus | not ported (a menu, more blocks, a renderer, `rftoolsbase`'s infusable path) |

### Measured traps this slice cost

1. **The forwarded runner already `cd`s into the port**, so a command prefixed with
   `cd ports/rftoolsutility` fails there with "No such file or directory"; the ladder commands run
   at the port root on the remote box.
2. **The remote pull-back clobbers locally-teed evidence logs** (`sdk-test.log` came back as the
   slice-9 85-pass log). The final 91-pass log was staged under `/tmp/opencode` and copied into the
   evidence tree after the last forwarded run.
3. **`cargo horizon-mod build` writes `target/horizon-mod/rftoolsutility.hmod`**, not the port-root
   artifact; the delivered copy is placed at the port root after the two builds and `cmp`.
4. **The launcher cannot run on the remote box** (missing `libxkbcommon-x11.so.0`), so the native
   build ran locally in a `--step native` lease; all Rust ladder steps still forwarded to AdLf-PC.
5. **The full A–AH leg flaked twice in its late teleporter phase** (a missing placement face after
   the spawner/beamer breaks, then a chat-reply race under concurrent load). `tools/e2e.sh`'s
   `place_block` now verifies the block after each attempt and clears entities; no assertion was
   weakened. `e2e-slice9.log` shows the same late-phase failure on 0.2.8, so it predates this
   slice. The focused `e2e-slice10-teleporter.log` is green on the delivered bytes — dial,
   cross-dimension arrival and the energy/cooldown readout all pass.

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices, rust-port

### Artifact

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `rftoolsutility.hmod` (port root) | 155,241 B | `54791b79cda99b677de4d308cf2f96483a02bb4210e3abe4559af8328b863cf5` | 203 |

Two `cargo horizon-mod build`s with `rm -f Cargo.lock` before each were `cmp`-identical
(`slice10-build-a.sha`, `slice10-build-b.sha`), **no packed `Cargo.lock`**. `check` green
(`rftoolsutility 0.2.9 (native-mod, format native-1/0a875da…/e59da909…, for 26.3)`);
`fits --build 0a875da` → `0a875da  made for`; `fits --build 0b14f8a` → `0b14f8a  fits`;
`verify-sdk-package.py rftoolsutility` → **1/1** (package matches the tree, R9, skills gate).
The README (packed) was edited before the final build.

## Next slice

The next teleporter-family gap is the **dialing device** — `rftoolsutility:dialing_device`
(`DialingDeviceTileEntity`, the GUI that lists destinations and writes the transmitter's
`MatterTransmitterData.destination`). It fits **machines** only if it can be a no-GUI block with a
command-driven dial, but the gap line says "a menu, more blocks" — before dispatching, check
whether the dialing device's behaviour is expressible without a screen; if not, the honest next
slice is a different `data`/`logic` family from the ledger (e.g. the `sensor`/`sequencer` logic
blocks), with a fail-capable redstone assertion rather than an invented GUI. The concrete
assertion for the dialing device would be: a dialing device adjacent to a transmitter lists the
receiver id from the destination table and `/rftoolsutility dial` uses that id, with an unknown id
refused — but this needs a menu-equivalent harness path first.

## Slice 11 — the sensor (2026-11-24)

**Surface check first (the dispatch's Step-0 gate): the dialing device is NOT expressible as a
no-GUI machines slice.** Read from the jar and the upstream `1.21_neo` clone at the artefact's own
commit `f799314007d68042df8504e83cd549360f2627c7` (analysis only, clone under `/tmp/opencode`):

- `DialingDeviceTileEntity` has **no tick and no redstone output**. Its whole behaviour is its GUI:
  `CMD_GETRECEIVERS` / `CMD_GETTRANSMITTERS` are mcjtylib `ListCommand`s, `CMD_DIAL` /
  `CMD_DIALONCE` / `CMD_CHECKSTATUS` are `ResultCommand`s, all driven by `GuiDialingDevice`. The SDK
  has no equivalent screen-command protocol and this port has **zero menu machinery** (every
  container it declares has `menu: None`).
- The list is `TeleportDestinations extends AbstractWorldData`: a world-persistent table with
  `destinationById`, `destinationIdByCoordinate`, `lastId`, receiver registration and per-player
  access. This port deliberately has no such registry — slice 9 dials the `GlobalPos` directly
  (upstream's own fallback when a coordinate has no destination-table id).

Implementing it would mean inventing a global destination table, integer ids and a list/dial
command the artefact exposes only through a GUI — a new subsystem, not a slice, and the brief
forbids inventing a GUI. **Switched to the sensor**, the other family the gap list and the 0.2.9
report name.

**Chosen from the port's own gap list** — `## Gaps (the rest of the mod)`, verbatim:

> **Logic:** `wire`, `sensor`, `sequencer`, `timer`, `counter`, `analog`, `digit`, `logic`,
> `invchecker`, `redstone_transmitter/receiver`, `redstone_information`.

It fits the ledger's **machines** tier shape: one block, one block entity, a ten-tick server check
and a real redstone output — no renderer, no entity AI, no new client half. The SDK at `0a875da`
exposes the surface (`BlockPart::block_signal`, `is_signal_source`, `LevelHost::state_string`,
`LevelRef::update_neighbors`, `Host::set_block_property` — the `laserbridges`, `securitycraft` and
`integrateddynamics` reference implementations), so this is not `blocked`.

**Pin** `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` / `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`,
mod API `0.2`, game 26.3. **Version 0.2.10.** Class **content**, tier **machines** (ledger).

### The artefact's trigger rule (read first)

`SensorTileEntity.java`: `tickServer` decrements `checkCounter`, resets it to **10** and calls
`support.setRedstoneState(this, checkSensor() ? 15 : 0)`. `checkSensor` resolves the block on the
input side (`getBlockPos().relative(facing.getInputSide())`); `SENSOR_BLOCK` checks that block with
`checkBlock`, which compares the ghost slot's item (or accepts any `state.canOcclude()` block when
the slot is empty). `SensorType`, `AreaType` (AREA_1/3/5), `GroupType` (ONE/ALL) and `SensorData`
carry the setting. The jar's block has 24 `logic_facing` slab rotations and a GUI that writes the
setting; neither is expressible by the content schema, so (as the analyzer/booster do for their
cosmetic facing) the input side and matcher are saved settings written by a harness command.

### What it does

- **The block.** `rftoolsutility:sensor` on `minecraft:iron_block` with a `signal` state property
  (`0`/`1`, default `0`) that caches the strength for the redstone hook. The jar's four sensor
  models, face texture, item model, recipe, advancement, loot table and lang lines are carried;
  its 24-rotation blockstate is replaced by the two `signal` variants, as the tank's nine-level
  blockstate is generated.
- **The check.** Every ten ticks the block entity reads the block on the configured input side
  (`north` by default), matches the named block id (or any non-replaceable block for `any`), and
  writes `signal=1`/`0` through `set_block_property`, then wakes the neighbours so a vanilla lamp
  or wire beside the sensor reacts. The block entity's own payload is untouched.
- **The redstone.** A `BlockPart::block_signal` reports **15** when the cached state is `1` (on both
  the direct and indirect paths) and `is_signal_source` marks the sensor a source so wire connects.
  Unlike the analyzer — whose artefact has no emitter — this is a **real vanilla redstone signal**.
- **The harness.** `/rftoolsutility sensor <pos> [nonce]` prints
  `signal=<0|15> type=<blocks|any> target=<id|none> area=1 group=one facing=<n>`;
  `/rftoolsutility sensor-set <pos> <kind> <target> <area> <group> <facing>` is the GUI's equivalent
  and refuses an unknown kind, facing, area or group. A broken sensor's item carries its setting
  (`minecraft:block_entity_data`, as every machine's does).

### Red → green (unit, verbatim)

Five runtime rules were `todo!()` stubs behind their tests first; the focused red leg
(`slice11-red.log`) ran four of them (the other two matcher tests do not contain "sensor" in their
names):

```
test result: FAILED. 0 passed; 4 failed; 0 ignored; 0 measured; 93 filtered out
  runtime::tests::a_sensor_checks_every_ten_ticks            panicked at src/runtime.rs:1016:5: not yet implemented
  runtime::tests::a_sensor_looks_at_the_block_on_its_input_side  panicked at src/runtime.rs:991:5: not yet implemented
  runtime::tests::a_sensor_signal_is_fifteen_or_zero         panicked at src/runtime.rs:1010:5: not yet implemented
  runtime::tests::an_unknown_sensor_kind_is_refused          panicked at src/runtime.rs:978:9: not yet implemented
```

The harness line's formatter was stubbed behind its own test (`slice11-glue-red.log`):

```
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 97 filtered out
  tests::the_sensor_readout_is_the_line_the_harness_parses  panicked at src/lib.rs:1588:5: not yet implemented
```

Green: **98 passed; 0 failed** (`slice11-green.log`, `sdk-test.log` regenerated for 0.2.10) — 56
runtime and 42 glue tests. `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings`
0 warnings (`slice11-clippy.log`).

### In game (evidence under `../_shared/evidence/rftoolsutility-e2e/`)

`tools/e2e-sensor.sh` — a focused, fail-capable leg reusing the launcher-built game; one lease per
leg, every log ends with its exit. The assertion is a **vanilla redstone lamp** beside the sensor.

| log | verdict |
|---|---|
| `e2e-slice11-sensor.log` | **PASS**, `EXIT:0` — AK–AK4 |
| `e2e-slice11-sensor-selftest.log` | **FAIL: SELFTEST sensor: the matching block lit the lamp, expected it to stay dark**, `EXIT:1` |
| `e2e-slice11-sensor-control.log` | **FAIL: ctl place rftoolsutility:sensor … was refused three times** (unmodded Nightly), `EXIT:1` |
| `e2e-slice11-teleporter.log` | **PASS**, `EXIT:0` — AI–AI2 (the 0.2.8 family, no regression) |
| `e2e-slice11-analyzer.log` | **PASS**, `EXIT:0` — AJ–AJ3 (the 0.2.9 family, no regression) |

`check-e2e-logs.py rftoolsutility-e2e --strict` → `logs=51 ok=51 missing-exit=0 bad-exit=0`.

The phases, each asserting a parsed value (verbatim from the green transcript):

```
AK-control: lamp with a vanilla redstone block: minecraft:redstone_lamp[lit=true]
            lamp after removing it: minecraft:redstone_lamp[lit=false]
AK0:  /rftoolsutility: sensor at [201, 101, 0]: signal=0 type=blocks target=minecraft:stone area=1 group=one facing=north nonce=1
AK1:  minecraft:redstone_lamp[lit=true] / signal=15 … nonce=2
AK2:  minecraft:redstone_lamp[lit=false] / signal=0 … nonce=3   (dirt in front)
AK3:  minecraft:redstone_lamp[lit=true] / signal=15 … nonce=4   (stone restored)
AK4:  give: rc=1 {"error":"unknown item rftoolsutility:not_a_sensor"} / signal=15 … nonce=5
```

AK places the sensor at `[201,101,0]` and a vanilla lamp at `[202,101,0]`; the input side is north
(`[201,101,-1]`). The vanilla redstone-block control proves the lamp and its `lit` parser work in
this game. Counterfactuals: AK2 (a different block) against AK1, AK3's restore, AK0's air, and
AK4's refused id; `SELFTEST_AT=sensor` inverts AK1 and fails as designed, so the redstone case can
fail. `CONTROL=1` runs the same script on the unmodded Nightly, where the sensor item does not
exist.

### Measured traps this slice cost

1. **`fits --build 0b14f8a` false-failed on `BlockStates::is_solid_render`** — the handle was
   removed at the other build. The port now uses the stable `BlockStates::replaceable` (negated)
   for `canOcclude()`, and both fits runs are green. No local type or variant is named `Block`/`Item`.
2. **`ctl place` cannot place a vanilla redstone lamp beside the sensor** ("no visible face to
   place … against"); the established harness sets vanilla blocks with `setblock`, which the leg
   now does. The assertion is the lamp's `lit` state, not a placement gesture.
3. **The ERR trap printed a spurious `FAIL:` for an intentionally refused `ctl give`**; the
   refusal is now wrapped in an `if` condition so the green log has no `FAIL:` line at all.
4. **Glue compile/lint:** the SDK's `Host` method is `block_state_string`, not `state_string`;
   clippy rejects an inherent `from_str` (renamed `SensorKind::from_name`) and a needless `return`
   at the end of `register`.

### Deviations and content adaptations

- **The input side is a saved setting, not the jar's 24-state `logic_facing`** (the content schema
   cannot express those slab rotations); both are set by `/rftoolsutility sensor-set` instead of a
   GUI. A user notices: the sensor cannot be oriented by placement.
- **`canOcclude()` is approximated by `!replaceable()`**, the stable handle across both builds;
   a thin non-replaceable block (a torch) would count as occluding here, where upstream's
   `canOcclude` would not. The carried matcher blocks (stone, dirt) behave identically.
- **The matcher is a block id, not the ghost slot's item.** Upstream compares the block's clone
   item to a ghost-stack item; with no GUI the harness names the block id directly. For the
   vanilla blocks the leg uses, the two spellings coincide.
- **Only AREA_1 / GROUP_ONE and two sensor kinds (`blocks`, `any`) are carried.** The seven other
   `SensorType`s, AREA_3/5/3x3/5x5/7x7 and GROUP_ALL are not ported (they need entity/growth queries
   and more harness surface); `sensor-set` refuses them rather than pretending to apply them.
- **No GUI, no infusing bonus, no comparator/TOP/Jade, no sounds** — the standing machines-tier
   gaps. The jar's carried recipe needs `rftoolsbase:machine_base`, so the e2e gives the block.
- **No new upstream defect** was adopted; the match, the 10-tick check and the 15/0 output are the
   artefact's own rule.

### Coverage after slice 11

| family | status |
|---|---|
| `rftoolsutility:sensor` (block, item, recipe, advancement, loot, texture, `signal` state) | tested — AK: placed and read back; the content is regenerated by `assemble_content.py` |
| the ten-tick check and the 15/0 redstone output | tested — AK0/AK1/AK2/AK3: a vanilla lamp lights only while the named block is in front; `SELFTEST_AT=sensor` inverts AK1 |
| the matcher (`blocks` by id, `any` for empty) | tested — AK1 (stone), AK2 (dirt refused), AK0 (air); unit `a_named_matcher_only_accepts_its_own_block` and `an_empty_matcher_accepts_any_occluding_block` |
| `sensor-set` / `sensor` commands and unknown-id refusal | tested — AK0–AK4; the parsed reply and the refused `ctl give` |
| the input side (four horizontal facings, unknown refused) | tested in code (`sensor_front`, `SensorConfig::from_data`); AK uses north |
| a `rftoolsbase:machine_base` recipe craft | blocked(`rftoolsbase:machine_base`) — the parent port's item; the e2e gives the block, as every machine's |
| the seven other sensor kinds, larger areas, `GROUP_ALL`, the GUI | not ported (more harness surface and a menu; recorded above) |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices, rust-port

### Artifact

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `rftoolsutility.hmod` (port root) | 165,931 B | `a2b5a1427d6a2d50f09e93179cb63a9cb870bb1b6cb274a172ea7fb11803cfc3` | 216 |

Two `cargo horizon-mod build`s with `rm -f Cargo.lock` before each were `cmp`-identical
(`slice11-build-a.sha`, `-b.sha`), **no packed `Cargo.lock`**. `check` green
(`rftoolsutility 0.2.10 (native-mod, format native-1/0a875da…/e59da909…, for 26.3)`);
`fits --build 0a875da` → `0a875da  made for`; `fits --build 0b14f8a` → `0b14f8a  fits`;
`verify-sdk-package.py rftoolsutility` → **1/1** (package matches the tree, R9, skills gate).
The README and both manifests were edited **before** the final build. The launcher's `native-build`
compiled the mod (`Compiling rftoolsutility (1 of 1)`, `native-0a875da-fbbbe2710a00`) for the legs.
All Rust ladder steps printed `build-slot: forwarding … to the remote runner` (hostname `AdLf-PC`);
`BUILD_LOCAL=1` was never set. The launcher/native build and the game legs ran locally by design.

## Next slice

The next logic-family gap is the **sequencer** — `rftoolsutility:sequencer`
(`SequencerTileEntity`, `SequencerData`, `SequencerMode`), from the same gap line:

> **Logic:** `wire`, `sensor`, `sequencer`, `timer`, `counter`, `analog`, `digit`, `logic`,
> `invchecker`, `redstone_transmitter/receiver`, `redstone_information`.

It is the sibling of the sensor just proven and fits the same **machines** shape (one block entity,
a 64-step redstone pattern, seven modes, an input signal and the `block_signal` surface now proven
in game). The artefact's rule to read first is `SequencerTileEntity.tickServer` plus
`SequencerMode` (`Once1/Once2/Loop1/Loop2/Loop3/Step/Loop4`) and `SequencerData.bits/delay/stepcount/
endstate`. The concrete assertion: set a known 64-bit pattern and `Step` mode by command, pulse a
**vanilla lever** on the input side once per step, and assert the adjacent vanilla lamp's `lit`
state follows bit 0, then bit 1, then bit 2 (a vanilla redstone block as the positive control,
as in AK); `SELFTEST_AT=sequencer` inverts one step and must `EXIT:1`. If the pin cannot express a
clean edge-triggered input, stop with `blocked(<symbol>)` rather than invent a GUI.

## Slice 12 — the sequencer (2026-11-24)

**Chosen from the port's own gap list** (the same line slice 11's report named and the 0.2.9
`## Next slice` proposed), verbatim:

> **Logic:** `wire`, `sensor`, `sequencer`, `timer`, `counter`, `analog`, `digit`, `logic`,
> `invchecker`, `redstone_transmitter/receiver`, `redstone_information`.

It is the sensor's sibling and fits the ledger's **machines** tier exactly: one block entity, a
64-bit pattern, seven modes, an input redstone level and the real `block_signal` surface slice 11
proved in game — no renderer, no entity AI, no new client half. **Pin**
`0a875da91dbb0fdcb7ba8c42c11341aa229af51a` / `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, API
`0.2`, game 26.3. **Version 0.2.11.** Class **content**, tier **machines** (ledger).

### The artefact's edge and mode rules (read first, not inferred from the name)

Read from the jar and the upstream `1.21_neo` clone at the artefact's own commit
`f799314007d68042df8504e83cd549360f2627c7` (analysis only):

- `SequencerTileEntity.tickOnServer`: `pulse = (powerLevel > 0) && !prevIn`, then `prevIn =
  powerLevel > 0`; `handlePulse()` runs on that **rising edge**. `timer--`; when it reaches 0,
  reset it to `data.delay()`, emit `checkOutput() ? 15 : 0` and run `handleCycle(powerLevel > 0)`.
- `checkOutput()`: `currentStep == -1 ? data.endstate() : getCycleBit(currentStep)`, where
  `getCycleBit(bit) = ((bits >> bit) & 1) == 1` — the 64-bit pattern, LSB = step 0.
- `setMode` starts `currentStep` at **-1** for `Once1/Once2/Loop3/Loop4` and at **0** for
  `Loop1/Loop2/Step`. `nextStep()` wraps to 0 at `stepcount`; `nextStepAndStop()` stops at -1.
- `handlePulse`: `Once1` starts a cycle only when stopped, `Once2`/`Loop2` restart at 0, `Step`
  advances one step, `Loop1/3/4` ignore pulses. `handleCycle`: `Once1/2` advance-and-stop,
  `Loop1/2` advance forever, `Loop3` advances while powered and `Loop4` resets to -1 when power
  ends, `Step` does nothing (pulses alone advance it).
- `SequencerData.createDefault()`: `bits=0`, `Once1`, `delay=1`, `stepcount=64`, `endstate=false`.

The pin expresses a clean edge: `LevelRef::signal(host, input_side_neighbour, direction)` is
`SignalGetter.getSignal(pos, dir)` — the neighbour's own emitted power. **Not** `blocked`.

### What it does

- **The block.** `rftoolsutility:sequencer` on `minecraft:iron_block` with a `signal` state
  (`0`/`1`, default `0`) that caches the strength for the redstone hook, exactly as the sensor's.
  The jar's four slab models, top texture, item model, recipe, advancement and loot table are
  carried; its 24 `logic_facing` rotations are replaced by the two `signal` variants.
- **The tick.** Reads the input-side neighbour's signal, runs the artefact's state machine, writes
  the four live fields (`step`, `prevIn`, `timer`, `rs`) back into the block entity's compound and
  mirrors the cached 15/0 into the block state, then wakes the neighbours so a vanilla lamp reacts.
- **The redstone.** `BlockPart::block_signal` reports 15 when the cached state is `1` (direct and
  indirect) and `is_signal_source` marks it a source. This is a **real vanilla redstone signal**.
- **The harness.** `/rftoolsutility sequencer <pos> [nonce]` prints
  `step=… bits=… mode=… output=<0|15> delay=… count=… endstate=… facing=<n>`; `sequencer-set <pos>
  <mode> <bits> <delay> <count> <endstate> <facing>` is the GUI's equivalent (bits decimal or
  `0x` hex; an unknown mode/facing, a count outside 1…64 and a bad bit string are refused). A
  broken sequencer's item carries its whole state (`minecraft:block_entity_data`).

### Red → green (unit, verbatim)

All 16 pure rules were `todo!()` stubs behind their tests first (`slice12-red-full.log`):

```
failures:
    runtime::tests::a_delay_longer_than_one_defers_the_output_refresh
    runtime::tests::a_pattern_bit_is_read_at_its_own_step
    runtime::tests::a_rising_edge_is_a_high_input_after_a_low_one
    runtime::tests::a_stopped_sequencer_outputs_its_endstate
    runtime::tests::an_all_zero_pattern_never_lights_the_output
    runtime::tests::every_sequencer_mode_round_trips_its_artefact_spelling
    runtime::tests::loop1_cycles_forever_without_any_input
    runtime::tests::loop2_advances_forever_but_a_pulse_resets_it
    runtime::tests::loop3_advances_only_while_the_input_is_high
    runtime::tests::loop4_stops_at_minus_one_when_the_input_ends
    runtime::tests::once1_runs_one_cycle_then_stops_at_the_endstate
    runtime::tests::once2_restarts_the_cycle_on_a_new_pulse
    runtime::tests::set_mode_starts_once_and_loop3_at_minus_one_and_the_rest_at_zero
    runtime::tests::step_advances_one_step_per_pulse_and_a_held_input_does_not
    runtime::tests::the_artefacts_default_sequencer_data_is_once1_delay_one_stepcount_64
    runtime::tests::the_step_output_follows_successive_bits

test result: FAILED. 98 passed; 16 failed; 0 ignored; 0 measured; 0 filtered out
```

Each new failure was `panicked at src/runtime.rs:…: not yet implemented`. Two expectations were
wrong on the first *green* run and were corrected against the artefact's tick order (a pulse and a
cycle can share a due tick; Once1 emits the endstate on the tick **after** the stop) — the red leg
above is the before-implementation run. The harness formatter was stubbed behind its own test
(`slice12-glue-red.log`): `FAILED. 4 passed; 1 failed` at `not yet implemented`.

Green: **116 passed; 0 failed** (`slice12-runtime-green.log`, `sdk-test.log` regenerated for
0.2.11) — 60 runtime and 56 glue tests. `cargo fmt --check` clean, clippy `-D warnings` 0 warnings
(`slice12-lint.log`).

### In game (evidence under `../_shared/evidence/rftoolsutility-e2e/`)

`tools/e2e-sequencer.sh` — a focused, fail-capable leg reusing the launcher-built game; one lease
per leg, every log ends with its exit. The assertion is a **vanilla redstone lamp** beside the
sequencer, driven by a **vanilla lever** on its north input side.

| log | verdict |
|---|---|
| `e2e-slice12-sequencer.log` | **PASS**, `EXIT:0` — AL–AL6 |
| `e2e-slice12-sequencer-selftest.log` | **FAIL: SELFTEST sequencer: bit 2 lit the lamp, expected it to stay dark**, `EXIT:1` |
| `e2e-slice12-sequencer-control.log` | **FAIL: ctl place rftoolsutility:sequencer … was refused three times** (unmodded Nightly), `EXIT:1` |
| `e2e-slice12-teleporter.log`, `-analyzer.log`, `-sensor.log` | each **PASS**, `EXIT:0` — the 0.2.8/0.2.9/0.2.10 families, no regression |

`check-e2e-logs.py rftoolsutility-e2e --strict` → `logs=58 ok=58 missing-exit=0 bad-exit=0`
(the saved first self-probe log is one of them).

The phases, each asserting a parsed value (verbatim from the green transcript):

```
AL-control: lamp with a vanilla redstone block: minecraft:redstone_lamp[lit=true]
            lamp after removing it: minecraft:redstone_lamp[lit=false]
AL0: /rftoolsutility: sequencer at [201, 101, 0]: step=0 bits=5 mode=Step output=15 delay=1 count=4 endstate=false facing=north nonce=1   (lamp lit)
AL1: … step=1 … output=0 … nonce=2     (lamp dark — one lever pulse)
AL2: … step=2 … output=15 … nonce=3    (lamp lit — second pulse)
AL3: … step=3 … output=0 … nonce=4     (lamp dark — third pulse)
AL4: … step=0 … output=15 … nonce=5    (wrap to bit 0, lamp lit)
AL5: … step=1 bits=0 … output=0 … nonce=7   (all-zero pattern stays dark)
AL6: give rftoolsutility:not_a_sequencer → rc=1 {"error":"unknown item rftoolsutility:not_a_sequencer"}
```

AL places the sequencer at `[201,101,0]`, a lamp at `[202,101,0]`, a lever at `[201,101,-1]`
(wall-mounted on the north input side), and a redstone block at `[203,101,0]` for the positive
control. Counterfactuals: the redstone-block control proves the lamp and its parser work; the
all-zero pattern is the differential; AL1/AL3 prove a zero bit actually darkens the lamp;
`SELFTEST_AT=sequencer` inverts AL2 and fails as designed, so the edge case can fail. `CONTROL=1`
runs the same script on the unmodded Nightly, where the sequencer item does not exist.

### Measured traps this slice cost

1. **`getBestNeighborSignal` self-feedback.** The first leg set a high output and the *next* tick
   read the sequencer's own 15 back as an input edge, advancing the pattern by itself (`step=1`
   with `output=0` where step 0 was expected). Root cause: this engine's best-neighbour read
   includes the block's own signal when it `isSignalSource`. **Fix:** read the input-side
   neighbour's directional `SignalGetter.getSignal(neighbour, into)` instead; the lever is placed
   on that side and no self edge is possible. The first leg is kept as `e2e-slice12-selfprobe.log`.
2. **A floor lever on top is not the input side.** With the directional read, a lever above the
   sequencer is invisible; the leg mounts it as `face=wall,facing=south` at the north neighbour so
   it powers the sequencer in the direction the glue reads.
3. **Under parallel load the game can drop its control socket** between arena setup and placement
   (three other campaign games were live). No code defect; the focused leg was reaped and rerun
   serially to a clean `PASS`. The analyzer leg's 8-tick wait for a 20-tick check also flaked once;
   both earlier legs are green on the delivered bytes.
4. **Clippy/fmt:** the mode enum has no `Display` (the feedback uses `mode.as_str()`), and the
   remote pull-back excludes `*.rs`, so `rustfmt` was applied **locally** before the final lint run.

### Deviations and content adaptations

- **The input side is a saved advisory.** The jar's 24 `logic_facing` slab rotations cannot be a
  26.3 content block's states; the four horizontal facings are written by `/rftoolsutility
  sequencer-set` and the tick reads that neighbour's signal. A user cannot orient the block by
  placement (the sensor's and the analyzer's deviation).
- **Only the matcher-facing subset of the GUI is carried**: the seven other `SensorType`s and the
  larger areas apply to the sensor, not the sequencer; the sequencer's GUI (pattern editor, mode
  buttons, delay/stepcount/endstate fields) is replaced by one harness command. All seven modes,
  the full 64-bit pattern, the delay, the stepcount and the endstate are implemented and tested.
- **No new upstream defect was adopted.** The rising-edge rule, the mode table and the 15/0 output
  are the artefact's own; the self-feedback above is an engine-integration finding, not a Java bug.
- The carried recipe needs `rftoolsbase:machine_base`, so the e2e gives the block, as every
  machine slice has. No comparator/TOP/Jade, no sounds, no infusing bonus, no GUI.

### Coverage after slice 12

| family | status |
|---|---|
| `rftoolsutility:sequencer` (block, item, recipe, advancement, loot, texture, `signal` state) | tested — AL: placed and read back; content regenerated by `assemble_content.py` |
| the rising-edge detector and the seven modes | tested — AL0–AL4 (Step pulses 0→1→2→3→0 with the lamp following), unit rules for Once1/2, Loop1–4 and held inputs |
| the 64-bit pattern, delay, stepcount and endstate | tested — AL0/AL5 (pattern 5 vs all zeros) and the 16 unit rules; `sequencer-set` refuses a bad mode/facing/count |
| the real 15/0 redstone output | tested — AL-control, AL0/AL2/AL4 lit vs AL1/AL3/AL5 dark; `SELFTEST_AT=sequencer` inverts AL2 |
| the item carrying the sequencer's state | carried (`block_drops` + `minecraft:block_entity_data`, the same path as the other machines); no dedicated phase |
| a `rftoolsbase:machine_base` recipe craft | blocked(`rftoolsbase:machine_base`) — the parent port's item; the e2e gives the block |
| the GUI (pattern editor, mode/delay/stepcount fields) | not ported (no menu; `sequencer-set` is its equivalent) |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices, rust-review, rust-port

### Artifact

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `rftoolsutility.hmod` (port root) | 178,533 B | `6184edc93fc748a0a1eab067bf15905d0ab25cd24ac87d402e1a418d0b671389` | 228 |

Two `cargo horizon-mod build`s with `rm -f Cargo.lock` before each were `cmp`-identical
(`slice12-build-a.sha`, `-b.sha`), **no packed `Cargo.lock`**. `check` green
(`rftoolsutility 0.2.11 (native-mod, format native-1/0a875da…/e59da909…, for 26.3)`);
`fits --build 0a875da` → `0a875da  made for`; `fits --build 0b14f8a` → `0b14f8a  fits`;
`verify-sdk-package.py rftoolsutility` → **1/1** (package matches the tree, R9, skills gate).
The manifest, README, crate doc and content were edited **before** the final build. The launcher's
`native-build` compiled the mod (`Compiling rftoolsutility (1 of 1)`,
`native-0a875da-7822e8161c41`) after the stale 0.2.10 and first-0.2.11 tags were removed; all Rust
ladder steps printed `build-slot: forwarding … to the remote runner` (hostname `AdLf-PC`), no
`BUILD_LOCAL=1` was set, and no lease was backgrounded.

## Slice 13 — the timer (2026-11-24)

**Chosen from the port's own gap list** (the same line slices 11 and 12 took), verbatim:

> **Logic:** `wire`, `sensor`, `sequencer`, `timer`, `counter`, `analog`, `digit`, `logic`,
> `invchecker`, `redstone_transmitter/receiver`, `redstone_information`.

It is the sequencer's sibling and fits the ledger's **machines** tier exactly: one block entity, a
delay counter, a rising-edge input and the real `block_signal` surface 0.2.10/0.2.11 proved in game —
no renderer, no entity AI, no new client half. **Pin** `0a875da…` / `e59da909…`, API `0.2`, game
26.3. **Version 0.2.12.** Class **content**, tier **machines** (ledger). MIT, ours on main → an
**Update** of the entry (never a first `Add`).

### The artefact's delay and edge rules (read first, not inferred from the name)

Read from the jar and the upstream `1.21_neo` clone at the artefact's own commit
`f799314007d68042df8504e83cd549360f2627c7` (analysis only, clone at `/tmp/opencode`):

- `TimerData(int delay, boolean redstonePauses)`; `createDefault()` is `(20, false)`. Both fields
  are written only by the jar's GUI (`integerfield('delay')`, the `pauses` toggle button), so this
  port writes them with `/rftoolsutility timer-set` — the deviation is recorded below.
- `TimerTileEntity.tickOnServer`: `pulse = (powerLevel > 0) && !prevIn`; `prevIn = powerLevel > 0`;
  on a pulse `timer = data.delay()`; `if (!data.redstonePauses() || !prevIn) timer--`; then
  `timer <= 0` → `timer = data.delay()`, output **15**, else **0**. So it is a **repeating** pulse
  train: a 20-tick-delay timer emits 15 for one tick every 20 ticks; the first pulse after an edge
  is at edge + delay − 1 (the edge tick already decrements). With `pauses=true` the countdown is
  held while the input is high, so timely pulses stop the output — the "overdue" detection the
  jar's manual describes ("given timely input pulses this block will never output a pulse itself").
  `loadAdditional`/`saveAdditional` keep `rs`, `prevIn` and `timer` in the saved compound.
- The input is read from the input-side neighbour's own emitted signal. The pin expresses that with
  `LevelRef::signal(host, neighbour, into)` (`SignalGetter.getSignal(pos, dir)`), **not**
  `getBestNeighborSignal`: the latter includes the block's own 15 (it `isSignalSource`), which made
  the sequencer latch a phantom edge and advance by itself (slice 12's trap). **Not blocked.**

### What it does

- **The block.** `rftoolsutility:timer` on `minecraft:iron_block` with a `signal` state (`0`/`1`,
  default `0`) caching the strength for the redstone hook, exactly as the sensor's and the
  sequencer's. The jar's four slab models, face texture, item model, recipe, advancement, loot
  table and lang lines are carried; its 24 `logic_facing` rotations are replaced by the two
  `signal` variants the hook caches.
- **The tick.** Reads the input-side neighbour's directional signal, runs the artefact's countdown,
  persists `prevIn`/`timer`/`rs` into the block entity's compound and mirrors the cached 15/0 into
  the block state, then wakes the neighbours so a vanilla lamp beside it sees the pulse.
- **The redstone.** `BlockPart::block_signal` reports 15 when `signal=1` (direct and indirect) and
  `is_signal_source` marks the timer a source — a **real vanilla redstone signal**.
- **The harness.** `/rftoolsutility timer <pos> [nonce]` prints `delay=… pauses=… output=<0|15>
  timer=… pulse=… fired=… facing=… time=… lamp=<lit|dark>`; `timer-set <pos> <delay> <pauses>
  <facing>` is the GUI's equivalent (a bad pauses word or an unknown facing is refused). `pulse`
  and `fired` are game-time stamps of the last rising edge and the first pulse after it, and
  `lamp` is the vanilla lamp's own `lit` state at that fire tick — the one-tick pulse is too brief
  for a polling `ctl block`, so the stamp is the fail-capable lamp observation. A broken timer's
  item carries its state (`minecraft:block_entity_data`, like every machine's).

### Red → green (unit, verbatim)

Five runtime rules and the harness formatter were `todo!()` stubs behind their tests first
(`slice13-red-full.log`):

```
failures:
    runtime::tests::a_fresh_java_timer_fires_on_its_first_tick
    runtime::tests::a_rising_edge_reloads_the_delay_and_the_first_output_is_after_that_delay
    runtime::tests::a_new_edge_restarts_the_countdown_and_the_output_repeats_every_delay_ticks
    runtime::tests::pauses_holds_the_countdown_while_the_input_is_high_and_fires_when_it_ends
    tests::the_timer_readout_is_the_line_the_harness_parses

test result: FAILED. 118 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out
```

Each new failure was `panicked at src/runtime.rs:…` / `src/lib.rs:…: not yet implemented`. Green:
**123 passed; 0 failed** (`sdk-test.log` regenerated for 0.2.12) — 65 runtime and 58 glue tests.
`cargo fmt --check` clean, `cargo clippy --all-targets --offline -- -D warnings` 0 warnings
(`slice13-clippy.log`).

### In game (evidence under `../_shared/evidence/rftoolsutility-e2e/`)

`tools/e2e-timer.sh` — a focused, fail-capable leg reusing the launcher-built game; one lease per
leg, every log ends with its exit. The assertion is a **vanilla redstone lamp** beside the timer,
driven by a **vanilla lever** on its north input side, with the delay read from the replies' own
game-time stamps (never wall time).

| log | verdict |
|---|---|
| `e2e-slice13-timer.log` | **PASS**, `EXIT:0` — AM–AM4 |
| `e2e-slice13-timer-selftest.log` | **FAIL: SELFTEST timer: the pulse came 3 ticks after the edge, expected the inverted 4**, `EXIT:1` |
| `e2e-slice13-timer-control.log` | **FAIL: ctl place rftoolsutility:timer … was refused three times** (unmodded Nightly), `EXIT:1` |
| `e2e-slice13-teleporter.log`, `-analyzer.log`, `-sensor.log`, `-sequencer.log` | each **PASS**, `EXIT:0` — the earlier families, no regression |

`check-e2e-logs.py rftoolsutility-e2e --strict` → `logs=65 ok=65 missing-exit=0 bad-exit=0`
(`slice13-e2e-gate.log`).

The phases, each asserting a parsed value (verbatim from the green transcript):

```
AM-control: lamp with a vanilla redstone block: minecraft:redstone_lamp[lit=true] / after removing it: [lit=false]
AM0: /rftoolsutility: timer at [201, 101, 0]: delay=600 pauses=false output=0 timer=578 pulse=-1 fired=-1 facing=north time=259 lamp=dark nonce=1
AM2: lamp recorded at the fire: lamp=lit (output=15); edge at tick 309, 15 output at tick 312 (delay=4 → 3 ticks)
AM2b: lamp recorded at the fire: lamp=lit; 4 ticks after the fire: output=0 lamp=[lit=false] (delay 600 → no re-fire)
AM3: pauses held the countdown from the edge until release; lamp recorded at the fire: lamp=lit
AM4: give rftoolsutility:not_a_timer → rc=1 {"error":"unknown item rftoolsutility:not_a_timer"}
```

Counterfactuals: the redstone-block control proves the lamp and its parser work in this game; the
no-pulse window (AM1) keeps `fired` unchanged; the long delay in AM2b makes any second fire
impossible inside the observation window, so the one-tick pulse is proved; `SELFTEST_AT=timer`
inverts the exact-tick assertion by a named `SELFTEST_SHIFT=1` and fails as designed, so the tick
count can fail. `CONTROL=1` runs the same script on the unmodded Nightly, where the timer item does
not exist.

### Measured traps this slice cost (harness and engine)

1. **A repeating timer rewrites the harness stamps.** The first leg asserted `fired - pulse` after
   a 12-tick poll and read 19, not 3: the free-running timer had fired four more times, so the
   stamp held the *last* fire. The fire and lamp stamps are now latched at the **first** fire after
   an edge (a `timerFiredLock` flag, reset by `timer-set`), so the delay is measured against the
   right pulse.
2. **A one-tick lamp pulse cannot be caught by `ctl block` polling.** The lamp is lit for ~1–2
   game ticks; a poll at 3-tick granularity saw `[lit=false]` even though the signal had fired.
   The timer's tick now reads the adjacent lamp's own `lit` state immediately after `update_neighbors`
   and stamps it into the saved compound; the leg asserts that stamp, which is the lamp's real
   state at the fire instant (no polling race).
3. **A pauses timer with a low input is just a free-running timer.** AM3 originally set
   `pauses=true, delay=4` while the lever was low and saw `output=15` before the test began; the
   leg now raises the lever first, so the held countdown is genuinely held.
4. **The analyzer leg's 8-tick wait vs a 20-tick check flaked twice.** `e2e-analyzer.sh`'s AJ3 now
   waits 40 ticks (> one check period) before asserting the block-above case; the assertion itself
   is unchanged. No product code changed for this harness fix (the e2e script is not packed).

### Deviations and content adaptations

- **The delay and `pauses` are command-driven, not a GUI.** The jar exposes both only through
  `integerfield('delay')`/the `pauses` toggle; the SDK has no menu machinery in this port, so
  `/rftoolsutility timer-set` writes the same `TimerData` fields. A user notices: no GUI.
- **The input side is a saved advisory.** The jar's 24 `logic_facing` slab rotations cannot be a
  content block's states, so `/rftoolsutility timer-set` writes one of four horizontal sides, as
  the sensor's and the sequencer's do. A user cannot orient the block by placement.
- **Harness tick stamps are not upstream fields.** `pulse`, `fired`, `lamp` and their lock exist
  only to make the timing fail-capable with `horizon-ctl`; they travel on the item with the rest of
  the saved compound and do not change the artefact's behaviour.
- **No new upstream defect adopted.** The countdown order and the 15/0 output are the artefact's
  own. No comparator/TOP/Jade, no sounds, no infusing bonus, no GUI — the standing gaps.
- The carried recipe needs `rftoolsbase:machine_base`, so the e2e gives the block, as every
  machine slice has.

### Coverage after slice 13

| family | status |
|---|---|
| `rftoolsutility:timer` (block, item, recipe, advancement, loot, texture, `signal` state) | tested — AM: placed and read back; content regenerated by `assemble_content.py` |
| the rising edge and the delay reload | tested — AM2/AM2b: edge at tick 309 → fire at 312 for delay 4 (= delay − 1); unit rules for edge, restart and repeat |
| the one-tick 15 pulse and the lamp flip | tested — AM2/AM2b: the lamp's own `lit` state at the fire is `lit`, and 4 ticks later output is 0 and the lamp dark; `SELFTEST_AT=timer` inverts the tick |
| `pauses` (the overdue case) | tested — AM3: the countdown is held while the lever is high, no fire, and it fires after release; unit `pauses_holds_the_countdown…` |
| the input side (four horizontal facings, unknown refused) | tested in code (`timer_front`, `timer-set`'s refusal); AM uses north |
| the item carrying the timer's state | carried (`block_drops` + `minecraft:block_entity_data`, the same path as the other machines); no dedicated phase |
| a `rftoolsbase:machine_base` recipe craft | blocked(`rftoolsbase:machine_base`) — the parent port's item; the e2e gives the block |
| the GUI (delay field, `pauses` toggle) | not ported (no menu; `timer-set` is its equivalent) |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

### Artifact

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `rftoolsutility.hmod` (port root) | 188,195 B | `d384c5262a6a9f4838cc2f3d26b3b1c2026c220c54bc0216d747bd457597b13f` | 240 |

Two `cargo horizon-mod build`s with `rm -f Cargo.lock` before each were `cmp`-identical
(`slice13-build-a.sha`, `-b.sha`), **no packed `Cargo.lock`**. `check` green
(`rftoolsutility 0.2.12 (native-mod, format native-1/0a875da…/e59da909…, for 26.3)`);
`fits --build 0a875da` → `0a875da  made for`; `fits --build 0b14f8a` → `0b14f8a  fits`;
`verify-sdk-package.py rftoolsutility` → **1/1** (package matches the tree, R9, skills gate).
The manifest, README, crate doc and content were edited **before** the final build; the e2e harness
fixes came after it but are not packed, so the gate still reports `package matches the tree`. The
launcher's `native-build` compiled the mod (`Compiling rftoolsutility (1 of 1)`,
`native-0a875da-14e27cf88cac`) after the stale 0.2.11 tag was removed. All Rust ladder steps printed
`build-slot: forwarding … to the remote runner` (hostname `AdLf-PC`), no `BUILD_LOCAL=1` was set,
and no lease was backgrounded.

## Slice 14 — the counter (2026-11-24)

**Chosen from the port's own `## Next slice`** (the line immediately above this section), verbatim:

> The next logic-family gap on the same line is the **counter** (`rftoolsutility:counter`,
> `CounterTileEntity`), the sibling of the timer: a redstone-pulse counter with a configurable max
> whose output fires when the count is reached and whose GUI sets the limit.

and the shared gap line `## Gaps (the rest of the mod)`:

> **Logic:** `wire`, `sensor`, `sequencer`, `timer`, `counter`, `analog`, `digit`, `logic`,
> `invchecker`, `redstone_transmitter/receiver`, `redstone_information`.

It fits the ledger's **machines** tier exactly: one block entity, a limit + running count, a
rising-edge input on the input-side neighbour and the real `block_signal` surface slices 11–13
proved in game — no renderer, no entity AI, no new client half. **Pin**
`0a875da91dbb0fdcb7ba8c42c11341aa229af51a` / `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, API
`0.2`, game 26.3. **Version 0.2.13.** Class **content**, tier **machines** (ledger). MIT, ours on
main → an **Update** of the entry (never a first `Add`).

### The artefact's count and reset rules (read first, not inferred from the name)

Read from the jar and the upstream `1.21_neo` clone at the artefact's own commit
`f799314007d68042df8504e83cd549360f2627c7` (analysis only; the clone at `/tmp/opencode` was already
present at that commit and was not re-cloned):

- `CounterData(int counter, int current)`; `createDefault() = (1, 0)` — with the default limit of
  one, every pulse fires. Both fields are written only by `GuiCounter`'s `integerfield`s, so this
  port writes the limit with `/rftoolsutility counter-set` (deviation below).
- `CounterTileEntity.update()` (called from `checkRedstone`): `pulse = (powerLevel > 0) && !prevIn`;
  `prevIn = powerLevel > 0`. **Only inside `if (pulse)`** does it `current++`; `current >= counter`
  wraps `current` to 0 and sets `newout = 15`, otherwise `newout = 0`; then
  `support.setRedstoneState(this, newout)`. With no pulse it never calls `setRedstoneState`, so the
  output is a **level latch**: 15 from the limit-th pulse until the *next* rising edge sets it to 0
  (and starts a new count). The jar's manual agrees: "Once Current is equal to Counter, it emits a
  redstone signal. Another redstone pulse will turn off the signal and reset Counter [Current] to 0."
- `setCounter`: a *changed* limit also resets `current` to 0 and the output to 0.
- `loadAdditional`/`saveAdditional` keep `rs` and `prevIn`; both travel on the item via
  `block_entity_data`, as every machine's saved compound does.
- The input is the input-side neighbour's own emitted signal. The pin expresses that with
  `LevelRef::signal(host, neighbour, into)` (`SignalGetter.getSignal(pos, dir)`), **not**
  `getBestNeighborSignal`: the latter includes the block's own 15 (it `isSignalSource`), which made
  the sequencer latch a phantom edge and advance by itself (slice 12's trap). **Not blocked** — no
  GUI-only setting is unexpressible; both `Counter` and `Current` are integers written by a command.

### What it does

- **The block.** `rftoolsutility:counter` on `minecraft:iron_block` with a `signal` state
  (`0`/`1`, default `0`) caching the strength for the redstone hook, exactly as its three logic
  siblings. The jar's four slab models, front texture, item model, recipe, advancement, loot table
  and lang lines are carried; its 24 `logic_facing` rotations are replaced by the two `signal`
  variants the hook caches.
- **The tick.** Reads the input-side neighbour's directional signal, runs the artefact's count
  rule, persists `prevIn`/`rs`/`current` into the block entity's compound and mirrors the latched
  15/0 into the block state, then wakes the neighbours so a vanilla lamp beside it reacts.
- **The redstone.** `BlockPart::block_signal` reports 15 when `signal=1` (direct and indirect) and
  `is_signal_source` marks the counter a source — a **real vanilla redstone signal**.
- **The harness.** `/rftoolsutility counter <pos> [nonce]` prints `limit=… current=… output=<0|15>
  pulses=… pulse=… fire=… lamp=… facing=… time=…`: `pulse`/`fire`/`time` are game-time stamps (the
  edge, the fire that reached the limit, and the readout's own tick) and `lamp` is a **live** read
  of the adjacent vanilla lamp's own `lit` state. `counter-set <pos> <limit> <facing>` is the GUI's
  equivalent (a bad limit or unknown facing is refused) and resets the count, the latch and the
  stamps. A broken counter's item carries its state (`block_entity_data`, like every machine's).

### Red → green (unit, verbatim)

The four new count rules were `todo!()` stubs behind their tests first (`slice14-red.log`):

```
failures:
    runtime::tests::a_counter_fires_only_on_the_limit_th_pulse_and_wraps_to_zero
    runtime::tests::a_counter_only_counts_a_rising_edge_not_a_held_input
    runtime::tests::a_counter_with_no_pulse_leaves_the_output_untouched
    runtime::tests::a_counters_next_pulse_clears_the_latched_output

test result: FAILED. 2 passed; 4 failed; 0 ignored; 0 measured; 123 filtered out; finished in 0.00s
```

Each new failure was `panicked at src/runtime.rs:1358:5: not yet implemented`. Two further rules
(the `(1, 0)` default and `setCounter`'s reset) passed from the start; the glue formatter
(`the_counter_readout_is_the_line_the_harness_parses`) was added with the formatter that produces
the line. Green: **130 passed; 0 failed** (`sdk-test.log`, regenerated for 0.2.13 after the final
build) — 71 runtime and 59 glue tests. `cargo fmt --check` clean, `cargo clippy --all-targets
--offline -- -D warnings` 0 warnings (`slice14-clippy.log`).

### In game (evidence under `../_shared/evidence/rftoolsutility-e2e/`)

`tools/e2e-counter.sh` — a focused, fail-capable leg reusing the launcher-built game; one lease per
leg, every log ends with its exit. The assertion is a **vanilla redstone lamp** beside the counter,
driven by a **vanilla lever** on its north input side, with the delay/limit read from the replies'
own game-time stamps (never wall time).

| log | verdict |
|---|---|
| `e2e-slice14-counter.log` | **PASS**, `EXIT:0` — AN–AN7 |
| `e2e-slice14-counter-selftest.log` | **FAIL: SELFTEST counter: the limit-th pulse lit the lamp, expected it to stay dark**, `EXIT:1` |
| `e2e-slice14-counter-control.log` | **FAIL: ctl place rftoolsutility:counter … was refused three times** (unmodded Nightly), `EXIT:1` |
| `e2e-slice14-teleporter.log`, `-analyzer.log`, `-sensor.log`, `-sequencer.log`, `-timer.log` | each **PASS**, `EXIT:0` — the earlier families, no regression |

`check-e2e-logs.py rftoolsutility-e2e --strict` → `logs=73 ok=73 missing-exit=0 bad-exit=0`
(`slice14-e2e-gate.log`).

The phases, each asserting a parsed value (verbatim from the green transcript):

```
AN-control: lamp with a vanilla redstone block: minecraft:redstone_lamp[lit=true] / after removing it: [lit=false]
AN0:  /rftoolsutility: counter at [201, 101, 0]: limit=4 current=0 output=0 pulses=0 pulse=-1 fire=-1 lamp=dark facing=north time=268 nonce=1
AN2:  limit=4 current=1 output=0 pulses=1 pulse=319 fire=-1 lamp=dark ... nonce=3
AN3:  limit=4 current=2 output=0 pulses=2 pulse=346 fire=-1 lamp=dark ... nonce=4
AN4:  limit=4 current=3 output=0 pulses=3 pulse=371 fire=-1 lamp=dark ... nonce=5
AN5:  limit=4 current=0 output=15 pulses=4 pulse=396 fire=396 lamp=lit ... nonce=6
AN5b: limit=4 current=0 output=15 pulses=4 pulse=396 fire=396 lamp=lit ... nonce=7  (latch persists)
AN6:  limit=4 current=1 output=0 pulses=5 pulse=466 fire=396 lamp=dark ... nonce=8  (next edge clears)
AN7:  give rftoolsutility:not_a_counter → rc=1 {"error":"unknown item rftoolsutility:not_a_counter"}
```

Counterfactuals: the redstone-block control proves the lamp and its parser work; AN1 is a no-pulse
window (pulses/fire unchanged); AN2–AN4 are the limit−1 pulses (lamp dark); AN5's `fire == pulse`
assertion ties the fire to the limit-th edge tick; AN6 is the next edge clearing the latch (the
manual's "another pulse turns off the signal"); `SELFTEST_AT=counter` inverts AN5's lamp assertion
and fails as designed, so the lamp case can fail. `CONTROL=1` runs the same script on the unmodded
Nightly, where the counter item does not exist.

### Measured traps this slice cost (harness and engine)

1. **A latched fire stamp goes stale on the clearing edge.** The first leg failed AN6: the readout
   said `output=0` (correct) but `lamp=lit`, because the `lamp` field was captured at the fire and
   only rewritten on another fire — the clearing pulse produces no fire. Fixed by making `lamp` a
   **live read** of the adjacent lamp in the readout command; the fire instant remains provable by
   the `fire`/`pulse` game-time stamps. No product behaviour changed (the e2e script and the
   readout are not part of the machine's tick rule).
2. **A `SELFEST`/`SELFTEST` typo made the first counterfactual vacuous.** The inverted assertion
   spelled the variable `SELFEST`, so `SELFTEST=1` never reached it and the leg printed PASS.
   Found by reading the counterfactual log rather than trusting the branch; corrected and re-run to
   a real `EXIT:1` at the named assertion.
3. **A borrow-checker error** (`E0502`) in the pulse increment: reading `KEY_COUNTER_PULSES` from
   `event.entity.nbt` inside a `write_int(&mut event.entity.nbt, …)` call. Fixed by reading the
   count into a local first. Clippy also flagged `counter_line`'s 9 args; added a scoped
   `#[expect(clippy::too_many_arguments, reason = …)]`.

### Deviations and content adaptations

- **The limit is command-driven, not a GUI.** The jar exposes `Counter`/`Current` only through
  `GuiCounter`'s integer fields; this port has zero menu machinery, so `/rftoolsutility counter-set`
  writes the same `CounterData` limit and resets the count. A user notices: no GUI.
- **The input side is a saved advisory.** The jar's 24 `logic_facing` slab rotations cannot be a
  content block's states, so `/rftoolsutility counter-set` writes one of four horizontal sides, as
  its three logic siblings do. A user cannot orient the block by placement.
- **`fire`/`pulse`/`time` and the pulse counter are harness-only stamps.** Upstream saves no such
  fields; they live in the block entity's compound and travel on the item, and do not change the
  artefact's behaviour. They exist so the lamp claim is made on game-time ticks, not wall time.
- **No new upstream defect adopted.** The rising-edge rule, the count wrap and the latched 15/0
  are the artefact's own (the manual confirms the clearing edge). No comparator/TOP/Jade, no
  sounds, no infusing bonus, no GUI — the standing machines-tier gaps.
- The carried recipe needs `rftoolsbase:machine_base`, so the e2e gives the block, as every machine
  slice has.

### Coverage after slice 14

| family | status |
|---|---|
| `rftoolsutility:counter` (block, item, recipe, advancement, loot, texture, `signal` state) | tested — AN: placed and read back; content regenerated by `assemble_content.py` |
| the rising edge, count increment and limit wrap | tested — AN2–AN5 (1→2→3→fire at 4 with the count reset); unit rules for held input, no-pulse and the wrap |
| the latched 15/0 output and the clearing edge | tested — AN-control, AN0/AN2–AN4 dark vs AN5/AN5b lit vs AN6 dark; `SELFTEST_AT=counter` inverts AN5 |
| the input side (four horizontal facings, unknown refused) | tested in code (`counter_front`, `counter-set`'s refusal); AN uses north |
| the item carrying the counter's state | carried (`block_drops` + `minecraft:block_entity_data`, the same path as the other machines); no dedicated phase |
| a `rftoolsbase:machine_base` recipe craft | blocked(`rftoolsbase:machine_base`) — the parent port's item; the e2e gives the block |
| the GUI (limit/current fields) and the item tooltip | not ported (no menu; `counter-set` is its equivalent) |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices, rust-review, rust-port

### Artifact

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `rftoolsutility.hmod` (port root) | 197,421 B | `b82e87029d1f98349c125339021d923dabf467a52719911e3dda20941f885e41` | 252 |

Two `cargo horizon-mod build`s with `rm -f Cargo.lock` before each were `cmp`-identical
(`slice14-builds-final.log`), **no packed `Cargo.lock`**. `check` green
(`rftoolsutility 0.2.13 (native-mod, format native-1/0a875da…/e59da909…, for 26.3)`);
`fits --build 0a875da` → `0a875da  made for`; `fits --build 0b14f8a` → `0b14f8a  fits`;
`verify-sdk-package.py rftoolsutility` → **1/1** (package matches the tree, R9, skills gate;
130 passed, 0 clippy warnings). The manifest, README, crate doc and content were edited **before**
the final build; the later `notes.md`/`VALIDATION.md`/`CONTENT.md` edits are not packed, and the gate
still reports `package matches the tree`. The launcher's `native-build` compiled the mod
(`Compiling rftoolsutility (1 of 1)`, `native-0a875da-66a414be7c67`) after the stale 0.2.12 tag was
removed. All Rust ladder steps printed `build-slot: forwarding … to the remote runner` (hostname
`AdLf-PC`), no `BUILD_LOCAL=1` was set, and no lease was backgrounded.

## Slice 15 — the analog (2026-11-24)

**Chosen from the port's own `## Next slice`** (the line immediately above this section), verbatim:

> The next logic-family gap on the same line is `analog` / `digit` (`rftoolsutility:analog`,
> `rftoolsutility:digit`, `AnalogTileEntity`, `DigitTileEntity`) — the remaining redstone-output
> logic siblings that fit the **machines** tier without a renderer. Before dispatching, read their
> `tickServer`/data records from the artefact at commit `f799314` first and check the same
> `LevelRef::signal` + `BlockPart::block_signal` surface slices 11–14 proved. The concrete assertion
> for `analog`: a known input-side redstone level written by a command must produce the artefact's
> declared output strength on a vanilla lamp (with a redstone-block control, a zero-input
> differential and `SELFTEST_AT=analog` `EXIT:1`). If either family needs a GUI-only setting or a
> renderer the pin cannot express, stop with `blocked(<symbol>)` rather than inventing one.

and the shared gap line `## Gaps (the rest of the mod)`:

> **Logic:** `wire`, `sensor`, `sequencer`, `timer`, `counter`, `analog`, `digit`, `logic`,
> `invchecker`, `redstone_transmitter/receiver`, `redstone_information`.

It fits the ledger's **machines** tier exactly: one block entity, six arithmetic settings, three
directional signal inputs and the real `block_signal` surface slices 11–14 proved in game — no
renderer, no entity AI, no new client half. **Pin**
`0a875da91dbb0fdcb7ba8c42c11341aa229af51a` / `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, API
`0.2`, game 26.3. **Version 0.2.14.** Class **content**, tier **machines** (ledger). MIT, ours on
main → an **Update** of the entry (never a first `Add`).

### The artefact's arithmetic (read first, not inferred from the name)

Read from the jar and the upstream `1.21_neo` clone at the artefact's own commit
`f799314007d68042df8504e83cd549360f2627c7` (analysis only, clone at `/tmp/opencode`):

- `AnalogData(float mulEqual, mulLess, mulGreater, int addEqual, addLess, addGreater)`;
  `createDefault() = (1.0, 1.0, 1.0, 0, 0, 0)` — the identity rule. All six fields are `@GuiValue`s
  upstream only writes from `GuiAnalog`, so this port writes them with `/rftoolsutility analog-set`
  (deviation below).
- `AnalogTileEntity.checkRedstone`: with `I` = the input side and `A`/`B` the two side inputs,
  `A == B → (int)(I * mulEqual + addEqual)`, `A < B → (int)(I * mulLess + addLess)`, otherwise
  `(int)(I * mulGreater + addGreater)`; then clamp `0..=15`, `setRedstoneState`, and update the
  neighbours only when the strength changes. The manual confirms the cap "above 15 or below 0".
- `A` is `LogicSlabBlock.rotateLeft(downSide, inputSide)` and `B` its `rotateRight`; read from
  McJtyLib at the same artefact pin, for a floor slab with input north these are **east (A)** and
  **west (B)**. `getInputStrength` reads `world.getSignal(neighbour, side)`, which the pin expresses
  with the same directional `LevelRef::signal` the four logic siblings use. **Not blocked.**
- The input is the input-side neighbour's own emitted signal, read directionally — never
  `getBestNeighborSignal`, which would include this block's own output as a phantom input
  (sequencer, slice 12).

### `digit` is `blocked(no-redstone-output:DigitTileEntity)` — not ported

`DigitTileEntity.checkRedstone` only calls `support.checkRedstone(this, …)`, which (McJtyLib
`LogicSlabBlock.checkRedstone`) sets `powerLevel` from the input side; `saveClientDataToNBT`/
`loadClientDataFromNBT` keep one `powered` byte. The class has **no** `setRedstoneState` and **no**
`getRedstoneOutput` override — it emits nothing. Its entire behaviour is the client-only
`DigitRenderer`, which draws `DIGITS[clamp(powerLevel,0,15)]` (`machineoutput_0`…`_f`); the jar's
manual says "This block simply displays a number from 0 to F (hexadecimal) corresponding to the
strength of the input signal". The SDK does have a client `hook:block_entity_submit` (surface.json)
that could draw that quad, but there is **no fail-capable redstone-lamp assertion** for a block
with no redstone output, and the `ctl`-readout alternative is exactly the chat-only fake the brief
forbids. So digit is blocked on the mandated assertion, not shipped. No digit content/code/evidence
was invented; a renderer-only slice would need an offscreen-screenshot oracle, not this lamp leg.

### What it does

- **The block.** `rftoolsutility:analog` on `minecraft:iron_block` with a **16-value** `signal`
  state (`0`…`15`, default `0`) caching the exact strength for the redstone hook — unlike its four
  logic siblings' 15/0 cache. The jar's four slab models, top texture, item model, recipe,
  advancement, loot table and lang lines are carried; its 24 `logic_facing` rotations are replaced
  by the 16 `signal` variants.
- **The tick.** Reads the three neighbours directionally, runs the artefact's branch arithmetic,
  persists the six settings, the facing and the three inputs, mirrors the clamped 0..15 into the
  block state and wakes the neighbours only when it changes. `block_signal` reports that exact
  number on both the direct and indirect paths and `is_signal_source` marks the analog a source.
- **The harness.** `/rftoolsutility analog <pos> [nonce]` prints `input=… a=… b=… branch=…
  mul=… add=… output=… changed=… lamp=… facing=… time=…`: `changed` is the game time of the last
  output change, `lamp` a live read of the adjacent vanilla lamp, `output` the game's cached
  strength. `analog-set <pos> <mul_eq> <mul_less> <mul_greater> <add_eq> <add_less> <add_greater>
  <facing>` is the GUI's equivalent. A broken analog's item carries its state
  (`block_entity_data`, like every machine's).

### Red → green (unit, verbatim)

The six new arithmetic rules were `todo!()` stubs behind their tests first (`slice15-red.log`):

```
test result: FAILED. 0 passed; 6 failed; 0 ignored; 0 measured; 130 filtered out; finished in 0.00s
  runtime::tests::an_analog_clamps_to_full_strength_and_no_signal   not yet implemented
  runtime::tests::an_analog_default_passes_the_input_through        not yet implemented
  runtime::tests::an_analog_truncates_the_product_before_it_is_clamped   not yet implemented
  runtime::tests::an_analog_uses_the_equal_branch_when_the_sides_match   not yet implemented
  runtime::tests::an_analog_uses_the_greater_branch_when_a_is_above_b    not yet implemented
  runtime::tests::an_analog_uses_the_less_branch_when_a_is_below_b       not yet implemented
```

Each failure was `panicked at src/runtime.rs:1443:5: not yet implemented`. Green: **139 passed;
0 failed** (`sdk-test.log`, regenerated for 0.2.14 after the final build) — 78 runtime and 61 glue
tests, including `an_analog_reads_the_input_and_its_clockwise_and_counterclockwise_sides`,
`the_analog_readout_is_the_line_the_harness_parses` and
`an_analogs_six_settings_round_trip_through_the_saved_compound`. `cargo fmt --check` clean,
`cargo clippy --all-targets --offline -- -D warnings` 0 warnings (`slice15-lint-final.log`).

### In game (evidence under `../_shared/evidence/rftoolsutility-e2e/`)

`tools/e2e-analog.sh` — a focused, fail-capable leg reusing the launcher-built game; one lease per
leg, every log ends with its exit. The assertion is a **vanilla redstone lamp** beside the analog,
with the three inputs driven by `minecraft:redstone_block` / air on the north (I), east (A) and west
(B) neighbours. The exact strength is asserted from both the reply and the block's own `signal`
state.

| log | verdict |
|---|---|
| `e2e-slice15-analog.log` | **PASS**, `EXIT:0` — AO–AO7 |
| `e2e-slice15-analog-selftest.log` | **FAIL: SELFTEST analog: the 0.5 * 15 output reads 7, expected it inverted**, `EXIT:1` |
| `e2e-slice15-analog-control.log` | **FAIL: Unknown item 'rftoolsutility:analog'**, `EXIT:1` (unmodded Nightly) |
| `e2e-slice15-{sensor,sequencer,timer,counter}.log` | each **PASS**, `EXIT:0` — the earlier logic families, no regression on the final bytes |

`check-e2e-logs.py rftoolsutility-e2e --strict` → `logs=80 ok=80 missing-exit=0 bad-exit=0`
(`slice15-e2e-gate.log`).

The phases, each asserting a parsed value (verbatim from the green transcript):

```
AO-control: lamp with a vanilla redstone block: minecraft:redstone_lamp[lit=true] / after removing it: [lit=false]
AO0:  input=0 a=0 b=0 branch=eq mul=1,1,1 add=0,0,0 output=0 changed=-1 lamp=dark facing=north time=273 nonce=1
AO1:  input=15 a=0 b=0 branch=eq mul=0.5,1,1 add=0,0,0 output=7 changed=280 lamp=lit facing=north time=315 nonce=2
AO2:  input=15 a=0 b=0 branch=eq mul=1,1,1 add=0,0,0 output=15 changed=321 lamp=lit ... nonce=3
AO3:  input=15 a=15 b=0 branch=greater mul=0,1,1 add=0,0,0 output=15 changed=377 lamp=lit ... nonce=4
AO4:  input=15 a=0 b=15 branch=less mul=0,1,0 add=0,0,0 output=15 changed=422 lamp=lit ... nonce=5
AO5:  input=0 a=0 b=0 branch=eq mul=1,1,1 add=0,0,0 output=0 changed=466 lamp=dark ... nonce=6
AO6:  input=15 a=0 b=0 branch=eq mul=0,1,1 add=-5,0,0 output=0 changed=466 lamp=dark ... nonce=7
AO7:  give rftoolsutility:not_an_analog → rc=1 {"error":"unknown item rftoolsutility:not_an_analog"}
```

Counterfactuals: the redstone-block control proves the lamp and its parser work in this game; AO1
fixes the artefact's `0.5 × 15 → 7` truncation and AO2 changes only the multiplier (identity → 15);
AO3/AO4 set the equal branch's multiplier to **0** so a branch-selection error darkens the lamp,
which is the differential proving `greater`/`less` are chosen from A and B; AO5's zero input is the
opposite differential; AO6 proves the negative clamp. `changed` advances with each strength change
(280 → 321 → 377 → 422 → 466), so the output is tied to the game's own ticks, not wall time.
`SELFTEST_AT=analog` inverts AO1's exact strength and fails as designed; `CONTROL=1` runs the same
script on the unmodded Nightly, where the analog item does not exist.

### Measured traps this slice cost

1. **The positive-control redstone block must be adjacent to the lamp.** The first leg put it
   three blocks away and the lamp stayed dark; it is now at the lamp's own other side (two blocks
   from the analog, so it cannot drive any of the three inputs).
2. **A 0.5 multiplier is the sharpest arithmetic assertion.** `(int)(15 × 0.5) = 7`, not 8; it
   proves both Java's truncation and that the strength is the artefact's arithmetic, not a 15/0
   latch like the other logic blocks. The selftest inverts exactly this value.
3. **A branch differential needs the wrong branch set to zero.** With `mul_eq = 0` and A ≠ B, a
   correct greater/less choice yields 15 while any selection error yields 0 — the lamp becomes the
   branch oracle, not just a strength oracle.
4. **No local type or variant is named `Block`/`Item`** (the new enum is `AnalogBranch`), so neither
   `fits` run false-failed. The GitHub releases API was called once per `fits` invocation to avoid
   the per-IP rate limit (slice 8's trap).

### Deviations and content adaptations

- **The six settings are command-driven, not a GUI.** `GuiAnalog`'s three multiplier/offset pairs
  are written by `/rftoolsutility analog-set`, as the other logic blocks' GUIs are replaced by
  their `-set` commands. A user notices: no GUI.
- **The input side is a saved advisory.** The jar's 24 `logic_facing` slab rotations cannot be a
  content block's states, so `analog-set` writes one of four horizontal sides, as its logic siblings
  do. A user cannot orient the block by placement.
- **The exact strength is a 16-value block state**, not a 15/0 cache: `BlockPart::block_signal`
  reports all 16 values, so a comparator/wire sees the real strength, and the lamp lights whenever
  it is nonzero. The jar's slab model shows on every strength.
- **No new upstream defect adopted.** The branch table, the f32 truncation and the clamp are the
  artefact's own. No comparator/TOP/Jade, no sounds, no infusing bonus, no GUI — the standing
  machines-tier gaps. The carried recipe needs `rftoolsbase:machine_base`, so the e2e gives the
  block, as every machine slice has.
- **`digit` is blocked** (see its own section): the artefact has no redstone output at all.

### Coverage after slice 15

| family | status |
|---|---|
| `rftoolsutility:analog` (block, item, recipe, advancement, loot, texture, `signal` state) | tested — AO: placed and read back; content regenerated by `assemble_content.py` |
| the branch comparison and the three multiply/add pairs | tested — AO0–AO4 (identity, 0.5 truncation, identity restore, greater with eq=0, less with eq=0); unit rules for every branch and the default |
| the 0..15 clamp and the exact redstone strength | tested — AO1 (7), AO2/AO3/AO4 (15), AO5/AO6 (0); the block's `signal=…` state is asserted alongside the reply, `SELFTEST_AT=analog` inverts AO1 |
| the three directional inputs (I, A=left, B=right) | tested in code (`analog_neighbours`, `analog_into_directions`) and in game (AO3/AO4 drive A and B separately); AO uses north |
| the item carrying the analog's state | carried (`block_drops` + `minecraft:block_entity_data`, the same path as the other machines); no dedicated phase |
| a `rftoolsbase:machine_base` recipe craft | blocked(`rftoolsbase:machine_base`) — the parent port's item; the e2e gives the block |
| the GUI (the six arithmetic fields) | not ported (no menu; `analog-set` is its equivalent) |
| `rftoolsutility:digit` and its 0..F display | blocked(`no-redstone-output:DigitTileEntity`) — a client renderer only; no lamp assertion exists |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

### Artifact

| File | Size | SHA-256 | Entries |
|---|---|---:|---|---:|
| `rftoolsutility.hmod` (port root) | 208,506 B | `209077df6d5fedf5aa1d7eedb15b9b01635e73ad8036f91cf8a47c33153b057b` | 264 |

Two `cargo horizon-mod build`s with `rm -f Cargo.lock` before each were `cmp`-identical
(`slice15-build-1.log`, `slice15-build-2.log`), **no packed `Cargo.lock`**. `check` green
(`rftoolsutility 0.2.14 (native-mod, format native-1/0a875da…/e59da909…, for 26.3)`);
`fits --build 0a875da` → `0a875da  made for`; `fits --build 0b14f8a` → `0b14f8a  fits`;
`verify-sdk-package.py rftoolsutility` → **1/1** (139 passed, 0 clippy warnings, package matches
the tree, R9, skills gate; `slice15-package-gate-final.log`). The manifest, README, crate doc and
content were edited **before** the final build; the later `notes.md`/`VALIDATION.md`/`CONTENT.md`
edits are not packed, and the gate still reports `package matches the tree`. The launcher's
`native-build` compiled the mod (`Compiling rftoolsutility (1 of 1)`,
`native-0a875da-32656145d602`) after the stale 0.2.12/0.2.13 tag was removed. All Rust ladder steps
printed `build-slot: forwarding … to the remote runner` (hostname `AdLf-PC`), no `BUILD_LOCAL=1`
was set, and no lease was backgrounded.

## Slice 16 — the logic gate (2026-11-24)

**Chosen from the port's own `## Next slice`** (the line immediately above this section at the
previous slice), verbatim:

> The next logic-family gap on the same line is `logic` / `invchecker` (`rftoolsutility:logic`,
> `rftoolsutility:invchecker`), the remaining three-input boolean and inventory-check siblings. Read
> `ThreeLogicTileEntity.checkRedstone` / `InvCheckerTileEntity.checkOutput` and their data records
> from the artefact at commit `f799314` first, and check whether their inputs can be driven with the
> proven `LevelRef::signal` + `BlockPart::block_signal` surface (the invchecker additionally needs a
> container read on the neighbour). The concrete assertion for `logic`: with two redstone inputs on
> the side neighbours, assert the artefact's declared 15/0 output on the adjacent vanilla lamp from
> the reply's game-time stamps, with a redstone-block control, a zero-input differential and
> `SELFTEST_AT=logic` ending `EXIT:1`. If the invchecker needs a GUI-only filter slot or a container
> surface the pin cannot express, stop with `blocked(<symbol>)` rather than inventing one; the
> `redstone_transmitter/receiver` family is the following gap.

and the shared gap line `## Gaps (the rest of the mod)`:

> **Logic:** `wire`, `sensor`, `sequencer`, `timer`, `counter`, `analog`, `digit`, `logic`,
> `invchecker`, `redstone_transmitter/receiver`, `redstone_information`.

It fits the ledger's **machines** tier exactly: one block entity, three directional signal inputs, an
eight-cell truth table and the proven `LevelRef::signal` + `BlockPart::block_signal` surface — no
renderer, no entity AI, no new client half. **Pin** `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` /
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, API `0.2`, game 26.3. **Version 0.2.15.** Class
**content**, tier **machines** (ledger). MIT, ours on main → an **Update** of the entry (never a
first `Add`).

### The artefact's rule (read first, not inferred from the name)

Read from the upstream `1.21_neo` clone at the artefact's own commit
`f799314007d68042df8504e83cd549360f2627c7` (analysis only, clone at `/tmp/opencode`):

- `ThreeLogicData(int[] logicTable)` with `createDefault() = {0,0,0,0,0,0,0,0}`; each cell is
  `0` (off), `1` (on) or `-1` (keep). The eight cells are `@GuiValue`s upstream only writes from
  `GuiThreeLogic`, so this port writes them with `/rftoolsutility logic-set`.
- `ThreeLogicTileEntity.checkRedstone`:
  `s = logicTable[powerLevel]`; `s == -1` returns (keep mode, the output is untouched); otherwise
  `support.setRedstoneState(this, s == 1 ? 15 : 0)` — so **any** other value drives 0, and the output
  is exactly 15 or 0, never an intermediate strength.
- `checkRedstone(Level, BlockPos)` builds `powerLevel = powered1 + powered2 + powered3` where
  `powered1 = getInputStrength(rotateLeft(down,input)) > 0 ? 1 : 0`, `powered2` is the input side
  (`2`), `powered3` the right side (`4`). For a floor slab with input north, A (left) is east and
  B (right) is west. `getInputStrength` is `world.getSignal(neighbour, side)` — the directional read
  the four logic siblings and the analog use, never `getBestNeighborSignal` (sequencer, slice 12).

### First gate: both families are expressible; `logic` is the cheaper one

- **`logic` — expressible.** Three `LevelRef::signal` reads plus the eight-cell boolean table and
  the `BlockPart::block_signal`/`is_signal_source` surface proven in slices 11–15. No GUI, no
  container read, no tag matching. Chosen.
- **`invchecker` — also expressible, but not blocked and not chosen.** The grep evidence: the SDK's
  `hooks/notes.rs:598` says "a server mod reaches a block's items through its `ITEMS` capability:
  `level.capability(host, &ITEMS, pos, side)` gives the block entity's slots … by the rules hoppers
  follow", and `parts/capabilities.rs:295` (`a_mod_reads_a_vanilla_chests_items_from_the_game_with_no_provider_for_it`)
  proves a vanilla neighbour's live slots are readable (`size(host)`, `resource(host, slot)`). The
  matcher ghost slot is a `@GuiValue` the other logic slices replace with a `-set` command, so it
  is not a hard GUI-only wall either. It is the **more expensive** slice: a neighbour capability read
  plus count/damage/tag matching on a ten-tick cadence, versus three signal reads and a table lookup.
  Therefore not `blocked(<symbol>)` — queued as the next gate (see `## Next slice`).

### Red → green (unit, verbatim)

Seven new pure rules were `todo!()` stubs behind their tests first (`slice16-red.log`):

```
test result: FAILED. 0 passed; 7 failed; 0 ignored; 0 measured; 139 filtered out; finished in 0.00s
  runtime::tests::a_logic_index_is_left_bit_input_bit_right_bit      not yet implemented
  runtime::tests::a_logic_keep_cell_holds_the_previous_strength      not yet implemented
  runtime::tests::a_logic_off_cell_drives_no_signal                  not yet implemented
  runtime::tests::a_logic_on_cell_drives_full_strength               not yet implemented
  runtime::tests::a_logic_reads_the_input_and_its_two_side_neighbours not yet implemented
  runtime::tests::a_logic_selects_the_cell_named_by_the_input_mask   not yet implemented
  runtime::tests::a_logic_table_starts_with_every_cell_off           not yet implemented
```

Green: **148 passed; 0 failed** (`sdk-test.log`, regenerated for 0.2.15 after the final build) —
85 runtime and 63 glue tests, including `a_logic_tables_eight_cells_round_trip_through_the_saved_compound`
and `the_logic_readout_is_the_line_the_harness_parses`. `cargo fmt --check` clean,
`cargo clippy --all-targets --offline -- -D warnings` 0 warnings (`slice16-lint.log`).

### In game (evidence under `../_shared/evidence/rftoolsutility-e2e/`)

`tools/e2e-logic.sh` — a focused, fail-capable leg reusing the launcher-built game; one lease per
leg, every log ends with its exit. The assertion is a **vanilla redstone lamp** beside the gate, with
two side neighbours (A east = bit 0, B west = bit 2) driven by `minecraft:redstone_block` for mask 5.

| log | verdict |
|---|---|
| `e2e-slice16-logic.log` | **PASS**, `EXIT:0` — LO–LO6 |
| `e2e-slice16-logic-selftest.log` | **FAIL: SELFTEST logic: the cell=1 output reads 15, expected it inverted**, `EXIT:1` |
| `e2e-slice16-logic-control.log` | **FAIL: Unknown item 'rftoolsutility:logic'**, `EXIT:1` (unmodded Nightly) |
| `e2e-slice16-{sensor,sequencer,timer,counter,analog}.log` | each **PASS**, `EXIT:0` — the earlier logic families, no regression on the final bytes |

`check-e2e-logs.py rftoolsutility-e2e --strict` → `logs=88 ok=88 missing-exit=0 bad-exit=0`
(`slice16-e2e-gate.log`) — the 80 earlier logs plus the 8 new slice-16 logs.

The phases, each asserting a parsed value (verbatim from the green transcript):

```
LO-control: lamp with a vanilla redstone block: minecraft:redstone_lamp[lit=true] / after removing it: [lit=false]
LO0:  mask=0 cell=0 output=0 changed=-1 lamp=dark facing=north time=233 nonce=1
LO1:  mask=5 cell=1 output=15 changed=242 lamp=lit facing=north time=277 nonce=2
LO2:  mask=5 cell=-1 output=15 changed=242 lamp=lit facing=north time=317 nonce=3
LO3:  mask=5 cell=0 output=0 changed=323 lamp=dark facing=north time=358 nonce=4
LO4:  mask=0 cell=0 output=0 changed=323 lamp=dark facing=north time=389 nonce=5
LO5:  mask=0 cell=1 output=15 changed=395 lamp=lit facing=north time=430 nonce=6
LO6:  give rftoolsutility:not_a_logic → rc=1 {"error":"unknown item rftoolsutility:not_a_logic"}
```

Counterfactuals: the redstone-block control proves the lamp and its parser work in this game; LO1
drives the declared cell=1 → 15 and is the selftest's inverted assertion; LO2 sets the cell to -1
and asserts the output **and** the `changed` stamp hold (the keep mode, not a 0/15 latch); LO3 sets
the same cell to 0 with the same mask so the lamp must darken — the artefact's 15/0 differential;
LO4's zero-input differential selects cell 0; LO5 sets cell 0 to 1 with no inputs, proving the output
is the selected cell and not a strength function of the input count. `changed` advances with each
strength change (242 → 323 → 395) and holds through keep, so the output is tied to the game's own
ticks. `SELFTEST_AT=logic` inverts LO1's exact output and fails as designed; `CONTROL=1` runs the
same script on the unmodded Nightly, where the logic item does not exist.

### Deviations and content adaptations

- **The eight cells are command-driven, not a GUI.** `GuiThreeLogic`'s truth-table editor is
  `/rftoolsutility logic-set <pos> <l0>…<l7> <facing>`, as every logic sibling's GUI is replaced by
  its `-set` command. A user notices: no GUI.
- **The input side is a saved advisory.** The jar's 24 `logic_facing` slab rotations cannot be
  content states, so `logic-set` writes one of four horizontal sides, as its siblings do.
- **No new upstream defect adopted.** The keep/off/on table and the 15/0 mapping are the artefact's
  own. No comparator/TOP/Jade, no sounds, no GUI — the standing machines-tier gaps. The carried
  recipe needs `rftoolsbase:machine_base`, so the e2e gives the block, as every machine slice has.
- **`digit` stays blocked** (`no-redstone-output:DigitTileEntity`): a client renderer only.

### Coverage after slice 16

| family | status |
|---|---|
| `rftoolsutility:logic` (block, item, recipe, advancement, loot, texture, `signal` state) | tested — LO: placed and read back; content regenerated by `assemble_content.py` |
| the eight-cell table (off/on/keep) and the 15/0 mapping | tested — LO0–LO3 (default, cell 1, keep, cell 0); unit rules for index, cell selection, on/off/keep |
| the three directional inputs (I bit 1, A bit 0, B bit 2) | tested in code (`logic_neighbours`) and in game (LO1–LO4 drive A and B; mask 5 and mask 0 asserted) |
| the item carrying the logic gate's state | carried (`block_drops` + `minecraft:block_entity_data`, the same path as the other machines); no dedicated phase |
| a `rftoolsbase:machine_base` recipe craft | blocked(`rftoolsbase:machine_base`) — the parent port's item; the e2e gives the block |
| the GUI (truth-table editor) | not ported (no menu; `logic-set` is its equivalent) |
| `rftoolsutility:digit` and its 0..F display | blocked(`no-redstone-output:DigitTileEntity`) — a client renderer only |
| `rftoolsutility:invchecker` (neighbour-item check, amount/slot/damage/tag) | not ported — expressible (`level.capability(&ITEMS, pos, side)`), queued as slice 17 |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

### Artifact

| File | Size | SHA-256 | Entries |
|---|---|---:|---|---:|
| `rftoolsutility.hmod` (port root) | 217,281 B | `5d59ce3f605cf106fa2ea8359e4497727bb37f3392b3b759877c117dd90d604d` | 276 |

Two `cargo horizon-mod build`s with `rm -f Cargo.lock` before each were `cmp`-identical
(`slice16-build-1.log`, `slice16-build-2.log`), **no packed `Cargo.lock`**. `check` green
(`rftoolsutility 0.2.15 (native-mod, format native-1/0a875da…/e59da909…, for 26.3)`);
`fits --build 0a875da` → `0a875da  made for`; `fits --build 0b14f8a` → `0b14f8a  fits`;
`verify-sdk-package.py rftoolsutility` → **1/1** (148 passed, 0 clippy warnings, package matches the
tree, R9, skills gate; `slice16-package-gate-final.log`). The manifest, README, crate doc and
content were edited **before** the final build; the later `notes.md`/`VALIDATION.md`/`CONTENT.md`
edits are not packed, and the gate still reports `package matches the tree`. The launcher's
`native-build` compiled the mod (`Compiling rftoolsutility (1 of 1)`,
`native-0a875da-dd35d19365bd`) after the stale 0.2.14 tag was removed. All Rust ladder steps printed
`build-slot: forwarding … to the remote runner` (hostname `AdLf-PC`), no `BUILD_LOCAL=1` was set,
and no lease was backgrounded.

## Slice 17 — the inventory checker (2026-11-24)

**Chosen from the port's own `## Next slice`** (the section immediately above this one at the
previous slice), verbatim:

> The next logic-family gap is `invchecker` (`rftoolsutility:invchecker`, `InvCheckerTileEntity`,
> `IncCheckerData`), the inventory-check sibling. The first gate proved it is expressible: read the
> neighbour's live item slots with `level.capability(host, &ITEMS, pos, side)` (SDK `hooks/notes.rs:598`,
> proven by `parts/capabilities.rs:295`), and replace the matcher ghost slot and the amount/slot/damage/
> tag `@GuiValue`s with a `/rftoolsutility invchecker-set` command, as every logic sibling's GUI is
> replaced. Read `checkOutput`/`isItemMatching` and `IncCheckerData.createDefault()` (amount=1, slot=0,
> DMG_IGNORE, tag=null) from the artefact at `f799314` first. The concrete in-game assertion: place a
> vanilla chest on the input side with a known stack in the configured slot, assert the artefact's
> declared 15 when `count >= amount` and 0 when below it (a stack of 1 against amount=2 is the
> zero-input differential), with a redstone-block lamp control and `SELFTEST_AT=invchecker` `EXIT:1`.
> If the neighbour-capability read turns out to need a mod provider the pin cannot supply for a vanilla
> chest, stop with `blocked(<symbol>)` and the grep evidence; the `redstone_transmitter/receiver`
> family is the following gap.

and the shared gap line `## Gaps (the rest of the mod)`:

> **Logic:** `wire`, `sensor`, `sequencer`, `timer`, `counter`, `analog`, `digit`, `logic`,
> `invchecker`, `redstone_transmitter/receiver`, `redstone_information`.

It fits the ledger's **machines** tier exactly: one block entity, a ten-tick check of a neighbour's
item slots, a four-field data record and the already-proven `BlockPart::block_signal` surface — no
renderer, no entity AI, no new client half. **Pin** `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` /
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, API `0.2`, game 26.3. **Version 0.2.16.** Class
**content**, tier **machines** (ledger). MIT, ours on main → an **Update** of the entry (never a
first `Add`).

### The artefact's rule (read first, not inferred from the name)

Read from the upstream `1.21_neo` clone at the artefact's own commit
`f799314007d68042df8504e83cd549360f2627c7` (analysis only, clone at `/tmp/opencode`):

- `IncCheckerData.createDefault()` = `(amount=1, slot=0, useDamage=DMG_IGNORE, tag=null)`; its four
  fields are `@GuiValue`s upstream only writes from `GuiInvChecker`, so this port writes them with
  `/rftoolsutility invchecker-set`.
- `checkOutput()`: `inputSide = LogicSupport.getFacing(state).getInputSide()`; neighbour
  `pos.relative(inputSide)`; if it is an inventory, `CapabilityTools.getItemCapabilitySafe(te)`;
  `slot` in `0..getSlots()`, stack not empty, `nr = isItemMatching(stack)`, `nr >= amount`, and the
  tag gate `tag == null || item.is(tag)` → **true**. Anything else (no inventory, no capability, an
  out-of-range or empty slot, too few matching items, a failed tag) → false.
- `isItemMatching(stack)`: ghost matcher slot empty → `stack.getCount()`; matcher present with
  `DMG_MATCH` → `ItemStack.isSameItem` (item id **and** damage) ? count : 0; with `DMG_IGNORE` →
  same item id ? count : 0.
- `tickServer()`: `checkCounter--; if (checkCounter > 0) return; checkCounter = 10` — a check every
  ten ticks — then `support.setRedstoneState(this, checkOutput() ? 15 : 0)`, exactly 15 or 0.

### First gate: the vanilla-chest capability read needs no provider of ours

Confirmed for a vanilla chest on the input side, not assumed: the SDK doc
`hooks/notes.rs:598` states `level.capability(host, &ITEMS, pos, side)` reaches a block entity's
slots "by the rules hoppers follow", and the test
`a_mod_reads_a_vanilla_chests_items_from_the_game_with_no_provider_for_it`
(`parts/capabilities.rs:294–308`) proves it directly: with `block_providers(&ITEMS, host, on(CHEST,
None)).is_none()` (no mod provider registered for the chest's type), the read still returns the
game's own handler (`owner()` = `"the game"`, `size(host)` and `resource(host, 0)` give the live
slot). The gate's `Some(intos[0])` side is the chest's face toward the checker (south for a north
input). Nothing is `blocked(<symbol>)`.

### Red → green (unit, verbatim)

Eight new pure rules were `todo!()` stubs behind their tests first (`slice17-red.log`):

```
test result: FAILED. 149 passed; 8 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
  runtime::tests::an_empty_ghost_matcher_accepts_the_stack_count     not yet implemented
  runtime::tests::check_output_honors_the_tag_gate                   not yet implemented
  runtime::tests::check_output_needs_count_at_least_amount           not yet implemented
  runtime::tests::check_output_refuses_out_of_range_or_empty_slots   not yet implemented
  runtime::tests::dmg_ignore_matches_by_item_id_only                 not yet implemented
  runtime::tests::dmg_match_refuses_a_different_damage               not yet implemented
  runtime::tests::the_invchecker_checks_every_ten_ticks               not yet implemented
  runtime::tests::the_invchecker_writes_full_or_no_strength           not yet implemented
```

Green: **161 passed; 0 failed** (`sdk-test.log`, regenerated for 0.2.16 after the final build) — 94
runtime and 67 glue tests, including `an_invcheckers_four_fields_round_trip_through_the_saved_compound`
and `the_invchecker_readout_is_the_line_the_harness_parses`. `cargo fmt --check` clean,
`cargo clippy --all-targets --offline -- -D warnings` 0 warnings (`slice17-lint.log`).

### In game (evidence under `../_shared/evidence/rftoolsutility-e2e/`)

`tools/e2e-invchecker.sh` — a focused, fail-capable leg reusing the launcher-built game; one lease
per leg, every log ends with its exit. The assertion is a **vanilla redstone lamp** beside the
checker, with a **vanilla chest** holding a known stack on the north input side; a vanilla redstone
block powers the same lamp first as the positive control.

| log | verdict |
|---|---|
| `e2e-slice17-invchecker.log` | **PASS**, `EXIT:0` — IV–IV6 |
| `e2e-slice17-invchecker-selftest.log` | **FAIL: SELFTEST invchecker: the satisfied check reads 15, expected it inverted**, `EXIT:1` |
| `e2e-slice17-invchecker-control.log` | **FAIL: ctl place rftoolsutility:invchecker at 202,101,0 was refused three times**, `EXIT:1` (unmodded Nightly) |
| `e2e-slice17-{sensor,sequencer,timer,counter,analog,logic}.log` | each **PASS**, `EXIT:0` — the earlier logic families, no regression on the final bytes |

`check-e2e-logs.py rftoolsutility-e2e --strict` → `logs=97 ok=97 missing-exit=0 bad-exit=0`
(`slice17-e2e-gate.log`) — the 88 earlier logs plus the 9 slice-17 logs.

The phases, each asserting a parsed value (verbatim from the green transcript):

```
IV-control: lamp with a vanilla redstone block: minecraft:redstone_lamp[lit=true] / after removing it: [lit=false]
IV0:  seen=minecraft:stone count=1 amount=1 slot=0 matched=true output=15 changed=252 lamp=lit facing=north time=271 nonce=1
IV1:  seen=minecraft:stone count=1 amount=2 slot=0 matched=false output=0 changed=277 lamp=dark time=293 nonce=2
IV2:  seen=minecraft:stone count=1 amount=1 slot=0 matched=false output=0 changed=277 lamp=dark time=315 nonce=3   (matcher=dirt)
IV3:  seen=minecraft:stone count=1 amount=1 slot=0 matched=true output=15 changed=321 lamp=lit time=339 nonce=4  (matcher=stone)
IV4:  seen=- count=0 amount=1 slot=1 matched=false output=0 changed=351 lamp=dark time=368 nonce=5
IV5:  seen=minecraft:stone count=3 amount=2 slot=0 matched=true output=15 changed=399 lamp=lit time=415 nonce=6
IV6:  give rftoolsutility:not_an_invchecker → rc=1 {"error":"unknown item rftoolsutility:not_an_invchecker"}
```

Counterfactuals: the redstone-block control proves the lamp and its parser work in this game; IV0
drives the artefact's default `amount=1` → 15 and is the selftest's inverted assertion; **IV1 is the
brief's differential** — the same single stack against `amount=2` must read 0 and darken the lamp;
IV2/IV3 are the matcher differential (an unmatched id refuses an otherwise sufficient stack, the
matching id restores 15); IV4 reads an empty configured slot as 0; IV5 fills the chest with 3 stone
and proves `count >= amount` rather than a fixed one; `changed` advances with each strength change
(252 → 277 → 321 → 399) and holds through a no-change check. `SELFTEST_AT=invchecker` inverts IV0's
exact output and fails as designed; `CONTROL=1` runs the same script on the unmodded Nightly.

### Deviations and content adaptations

- **The matcher's ghost slot and four data fields are command-driven, not a GUI.**
  `/rftoolsutility invchecker-set <pos> <amount> <slot> <damage> <matcher> <mdamage> <tag> <facing>`
  replaces `GuiInvChecker` (`-` means no matcher and no tag). A `word` argument cannot carry a colon,
  so the harness writes `stone`/`dirt` and the glue qualifies them to `minecraft:…`. A user notices:
  no GUI.
- **The input side is a saved advisory.** The jar's 24 `logic_facing` slab rotations cannot be
  content states, so `invchecker-set` writes one of four horizontal sides, as its siblings do.
- **The item carries the checker's state** (`block_drops` + `minecraft:block_entity_data`, the same
  path as the other machines); no dedicated phase. The tag gate is unit-tested (`check_output_honors_
  the_tag_gate`) but has no dedicated in-game phase — the lamp phases exercise amount, slot and
  matcher, and no 26.3 item tag in this pack is a robust fixture for a *positive* tag assertion.
- **No new upstream defect adopted.** No comparator/TOP/Jade, no sounds, no GUI — the standing
  machines-tier gaps. `digit` stays blocked (`no-redstone-output:DigitTileEntity`).

### Coverage after slice 17

| family | status |
|---|---|
| `rftoolsutility:invchecker` (block, item, recipe, advancement, loot, texture, `signal` state) | tested — IV: placed and read back; content regenerated by `assemble_content.py` |
| the neighbour `ITEMS` capability read on the input side | tested — IV0–IV5 (a vanilla chest's live slot, 27 slots, no provider of ours); gate evidence above |
| `amount`/`slot`/matcher/damage-mode/tag fields and the 15/0 mapping | tested — IV0–IV5 (`count>=amount`, 1-vs-2 differential, matcher, empty slot, larger stack); unit rules |
| the ten-tick cadence | tested in code (`invchecker_due`/`invchecker_countdown`, 9 skipped ticks) |
| the item carrying the checker's state | carried (`block_drops` + `minecraft:block_entity_data`); no dedicated phase |
| a positive in-game item-tag gate | not ported as its own leg (no robust 26.3 tag fixture; the pure rule is tested) |
| a `rftoolsbase:machine_base` recipe craft | blocked(`rftoolsbase:machine_base`) — the parent port's item; the e2e gives the block |
| the GUI (`GuiInvChecker`) | not ported (no menu; `invchecker-set` is its equivalent) |
| `rftoolsutility:digit` and its 0..F display | blocked(`no-redstone-output:DigitTileEntity`) — a client renderer only |
| `redstone_transmitter/receiver`, `redstone_information`, `wire` | not ported — next logic gaps |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices, rust-port

### Artifact

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `rftoolsutility.hmod` (port root) | 229,209 B | `ea0e6171d339385206350b8111a6ad47b832bc3b806a5a03fc1c6f37aa61192e` | 289 |

Two `cargo horizon-mod build`s with `rm -f Cargo.lock` before each were `cmp`-identical
(`slice17-build-1.log`, `slice17-build-2.log`), **no packed `Cargo.lock`**. `check` green
(`rftoolsutility 0.2.16 (native-mod, format native-1/0a875da…/e59da909…, for 26.3)`);
`fits --build 0a875da` → `0a875da  made for`; `fits --build 0b14f8a` → `0b14f8a  fits`;
`verify-sdk-package.py rftoolsutility` → **1/1** (161 passed, 0 clippy warnings, package matches the
tree, R9, skills gate; `slice17-package-gate-final.log`). The manifest, README, crate doc and
content were edited **before** the final build; the later `notes.md`/`VALIDATION.md`/`CONTENT.md`
edits and the harness-only e2e fixes are not packed, and the gate still reports `package matches the
tree`. The launcher's `native-build` compiled the mod (`Compiling rftoolsutility (1 of 1)`,
`native-0a875da-595f6569515b`) after the stale 0.2.15/0.2.16 tags were removed. All Rust ladder steps
printed `build-slot: forwarding … to the remote runner` (hostname `AdLf-PC`), no `BUILD_LOCAL=1` was
set, and no lease was backgrounded.

## Next slice

The next logic-family gap on the same line is the **`redstone_transmitter/receiver`** pair
(`rftoolsutility:redstone_transmitter`, `rftoolsutility:redstone_receiver`,
`RedstoneTransmitterTileEntity`/`RedstoneReceiverTileEntity`, `RedstoneChannelTileEntity`), the
wireless-channel siblings. It fits **machines**: two block entities, a channel setting and a 0..15
signal, with the proven `LevelRef::signal` + `BlockPart::block_signal` surface. **Gate first:**
read the artefact at `f799314` and check the world-saved `RedstoneChannels` table the transmitter
writes (`channels.getOrCreateChannel`/`setDirty`/`save` in `RedstoneTransmitterTileEntity.update()`)
against this pin's world-file surface; if the pin cannot persist that table, stop with
`blocked(<symbol>)` and the grep evidence rather than inventing an inert channel. If it is
expressible, replace the channel GUI with `/rftoolsutility redstone-set` as every logic sibling's
GUI is replaced, and make the concrete in-game assertion: a redstone block feeds a transmitter on
channel N, a receiver on the same channel emits the artefact's declared strength to a vanilla lamp
beside it, a receiver on another channel stays dark, and `SELFTEST_AT=redstone` ends `EXIT:1`.
Redistribute the modified brief from the orchestrator; this report proposes it, it does not dispatch it.

## Slice 18 — the redstone transmitter/receiver (2026-11-24)

**Chosen from the port's own `## Next slice`** (the section immediately above this one at the
previous slice), verbatim:

> The next logic-family gap on the same line is the **`redstone_transmitter/receiver`** pair
> (`rftoolsutility:redstone_transmitter`, `rftoolsutility:redstone_receiver`,
> `RedstoneTransmitterTileEntity`/`RedstoneReceiverTileEntity`, `RedstoneChannelTileEntity`), the
> wireless-channel siblings. It fits **machines**: two block entities, a channel setting and a 0..15
> signal, with the proven `LevelRef::signal` + `BlockPart::block_signal` surface. **Gate first:**
> read the artefact at `f799314` and check the world-saved `RedstoneChannels` table the transmitter
> writes (`channels.getOrCreateChannel`/`setDirty`/`save` in `RedstoneTransmitterTileEntity.update()`)
> against this pin's world-file surface; if the pin cannot persist that table, stop with
> `blocked(<symbol>)` and the grep evidence rather than inventing an inert channel. If it is
> expressible, replace the channel GUI with `/rftoolsutility redstone-set` as every logic sibling's
> GUI is replaced, and make the concrete in-game assertion: a redstone block feeds a transmitter on
> channel N, a receiver on the same channel emits the artefact's declared strength to a vanilla lamp
> beside it, a receiver on another channel stays dark, and `SELFTEST_AT=redstone` ends `EXIT:1`.

and the shared gap line `## Gaps (the rest of the mod)`:

> **Logic:** `wire`, `sensor`, `sequencer`, `timer`, `counter`, `analog`, `digit`, `logic`,
> `invchecker`, `redstone_transmitter/receiver`, `redstone_information`.

It fits the ledger's **machines** tier exactly: two block entities, an input-side signal read, a
world-saved channel table and the real `block_signal` surface slices 11–17 proved in game — no
renderer, no entity AI, no new client half. **Pin** `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` /
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, API `0.2`, game 26.3. **Version 0.2.17.** Class
**content**, tier **machines** (ledger). MIT, ours on main → an **Update** of the entry (never a
first `Add`).

### Persistence gate — PASSED, not `blocked`

Read from the upstream `1.21_neo` clone at the artefact's own commit
`f799314007d68042df8504e83cd549360f2627c7` (analysis only, clone at `/tmp/opencode`):

- `RedstoneChannels extends AbstractWorldData<RedstoneChannels>` stores `"RfToolsRedstoneChannels"`
  as a `ListTag` of `{channel, value, name}` plus `lastId`; `newChannel()` is `lastId++`,
  `getOrCreateChannel(id)` creates a zero-valued channel, `getChannel(id)` is its read.
- `RedstoneTransmitterTileEntity.update()` (from `neighborChanged`/`setPlacedBy`): server,
  `getChannel() != -1`, `powerLevel != prevIn` → `prevIn = powerLevel`,
  `getOrCreateChannel(getChannel()).setValue(powerLevel)`, `channels.save()`. `prevIn` is saved in
  NBT (`prevIn`, with the boolean backwards-compatibility read).
- `RedstoneReceiverTileEntity.checkOutput()` reads `channels.getChannel(channel).getValue()` and
  maps it: `channel == -1` or a missing channel → 0; `!analog && v > 0` → 15; analog → the exact
  `v`. `tickServer()` calls `setRedstoneState(this, checkOutput())` every tick; `rs` is saved.
- Both are `LogicSlabBlock`s: the input side is `getFacing(state).getInputSide()`, the output goes
  to every side through `LogicSupport`, and `collectImplicitComponents` moves the channel/analog
  data onto the item.

**This pin can persist the table.** `ServerHost::world_data_files()` returns `&dyn ConfigFiles`
whose `read`/`write` own `<save>/horizon-mod-data/<mod id>/`, "copied, backed up and deleted with
that save", `NoWorld` without one (`ports/_shared/sdk/0a875da/crates/horizon-mod-vocab/src/host/
config.rs`, `content/config_files.rs`). It is a real per-save store already in use by
`ports/solcarrot/src/store.rs`, `ports/moredragoneggs/src/support.rs` and
`ports/dynamictrees/src/glue.rs`. The port writes `redstone-channels.tsv` there atomically through
`ConfigFiles::write` and reads it back on each tick — no chat-command stand-in. The relaunch phase
R4 below proves the bytes survive a real world restart. So **not** `blocked(<symbol>)`.

### What it does

- **The blocks.** `rftoolsutility:redstone_transmitter` (no cached state — it is a sink) and
  `rftoolsutility:redstone_receiver` (a 16-value `signal` state, like the analog's) on the full-cube
  `minecraft:iron_block` template, the jar's four slab models, top texture, item models, recipes,
  advancements and loot tables carried by `tools/assemble_content.py`. Their 24 `logic_facing`
  rotations and two GUIs are replaced by a saved input side and `/rftoolsutility redstone-set`.
- **The transmitter.** Its tick reads the input-side neighbour's own emitted signal (directional
  `LevelRef::signal`, never `getBestNeighborSignal`, which would see its own output as a phantom
  input), compares with the saved `prevIn`, and on a change writes the strength into the channel's
  row of the world file. With channel `-1` it does nothing; `redstone-set … -1 …` allocates a fresh
  id from the world table (`lastId++`), as upstream's GUI `getChannel(true)` does.
- **The receiver.** Its tick reads the channel's strength from the world file, maps it through
  `receiver_output` (digital 15/0 or exact analog 0..15), mirrors the exact strength into the
  `signal` block state and wakes the neighbours on a change — a real vanilla redstone signal.
- **The item.** `block_drops` moves both machines' saved compound onto their items as vanilla's
  `minecraft:block_entity_data`, as every machine's does (upstream's `collectImplicitComponents`).
- **The harness.** `/rftoolsutility redstone <pos> [nonce]` prints the parsed channel, power,
  `prevIn`, table value, output, change stamp, lamp state, facing and game time;
  `redstone-set <pos> <channel> <analog> <facing>` writes the two GUIs' fields.

### Red → green (unit, verbatim)

Nine pure rules were `todo!()` stubs behind their tests first (`slice18-red.log`):

```
failures:
    runtime::tests::a_channel_is_created_when_a_transmitter_first_writes_it
    runtime::tests::a_digital_receiver_emits_full_strength_for_any_signal
    runtime::tests::a_fresh_channel_table_hands_out_one_then_two
    runtime::tests::a_receiver_on_an_unset_or_missing_channel_stays_dark
    runtime::tests::a_transmitter_with_no_channel_writes_nothing
    runtime::tests::a_transmitter_writes_only_when_the_input_strength_changes
    runtime::tests::a_written_strength_is_clamped_to_the_redstone_range
    runtime::tests::an_analog_receiver_preserves_the_exact_strength
    runtime::tests::the_channel_table_round_trips_through_its_text_codec

test result: FAILED. 161 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out
```

Each new failure was `panicked at src/runtime.rs:…: not yet implemented`. The first green attempt
failed one expectation (`a_fresh_channel_table_hands_out_one_then_two` read `Some(0)` where an
absent, never-written channel must read `None`); the rule was corrected to keep absence distinct
from a stored zero, then green. Final: **172 passed; 0 failed** (`sdk-test.log`, regenerated for
0.2.17 after the final build) — 94 `runtime` (the nine new channel/signal rules among them) and
78 glue tests (`the_redstone_readout_is_the_line_the_harness_parses`,
`the_channel_codec_keeps_ids_sorted_and_strengths_in_range`). `cargo fmt --check` clean,
`cargo clippy --all-targets --offline -- -D warnings` 0 warnings (`sdk-clippy.log`).

### In game (evidence under `../_shared/evidence/rftoolsutility-e2e/`)

`tools/e2e-redstone.sh` — a focused, fail-capable leg reusing the launcher-built game; one lease per
leg, every log ends with its exit. The assertion is a **vanilla redstone lamp** beside each receiver;
a vanilla redstone block powers the same lamp first as the positive control.

| log | verdict |
|---|---|
| `e2e-slice18-redstone.log` | **PASS**, `EXIT:0` — R–R5 |
| `e2e-slice18-redstone-selftest.log` | **FAIL: SELFTEST redstone: the fed receiver reads 15, expected it inverted**, `EXIT:1` |
| `e2e-slice18-redstone-control.log` | **FAIL: ctl place rftoolsutility:redstone_transmitter at 202,101,0 was refused three times**, `EXIT:1` (unmodded Nightly) |
| `e2e-slice18-{sensor,sequencer,timer,counter,analog,logic,invchecker}.log` | each **PASS**, `EXIT:0` — the earlier logic families, no regression on the final bytes |

`check-e2e-logs.py rftoolsutility-e2e --strict` → `logs=107 ok=107 missing-exit=0 bad-exit=0`
(`slice18-e2e-gate.log`).

The phases, each asserting a parsed value (verbatim from the green transcript):

```
R0: transmitter channel=1 power=0 prev=0 value=0 lamp=dark ... nonce=1
    receiver channel=1 analog=false value=0 output=0 changed=-1 lamp=dark ... nonce=2
R1: transmitter channel=1 power=15 prev=15 value=15 ... nonce=3
    receiver channel=1 analog=false value=15 output=15 changed=410 lamp=lit ... nonce=4
    receiver channel=2 analog=false value=0 output=0 changed=-1 lamp=dark ... nonce=5
R2: receiver channel=1 analog=false value=0 output=0 changed=461 lamp=dark ... nonce=6
R3: transmitter channel=1 power=11 prev=11 value=11 ... nonce=7
    receiver channel=1 analog=true value=11 output=11 changed=542 lamp=lit ... nonce=8
R4: receiver channel=1 analog=true value=11 output=11 changed=542 lamp=lit ... nonce=9  (after relaunch)
R5: give rftoolsutility:not_a_redstone → rc=1 Unknown item
```

Counterfactuals: the redstone-block lamp control proves the lamp and its parser work; R0's unfed
channel and R2's removed source are the zero differentials against R1; the receiver on channel 2
differs from the one on channel 1 only in its channel; R3's 11-strength wire (five wire steps from
a redstone block) is the differential between digital 15 and analog 11; R4 is the same machine
after a real relaunch; `SELFTEST_AT=redstone` inverts R1's exact output and fails as designed;
`CONTROL=1` runs the same script on the unmodded Nightly. Every assertion reads a parsed reply and
the game's own block state, never the absence of an error.

### Measured traps this slice cost

1. **The first wire phase overwrote its own source.** R3 first put a redstone block at `INZ-1`
   and then laid wires over `INZ-4..INZ`, which replaced the block at `INZ-1` — the transmitter
   read power 0 and the leg failed. The block now goes at `INZ-5`, beyond the wire run, and the
   leg reads a real partial strength of 11; a harness fix only, no product code changed.
2. **`cargo fmt --check` and clippy run on the remote copy.** The forwarded runner's pull-back
   excludes `*.rs`, so `cargo fmt` was applied locally and the corrected sources re-synced before
   the final lint; clippy then asked for `#[expect(clippy::too_many_arguments, …)]` on `red_line`
   and for struct-update syntax in two tests, both applied before the final green run.
3. **The forwarded pull-back clobbers `sdk-test.log`/`sdk-clippy.log`.** Each forwarded run
   restored the stale 0.2.8 (85-pass) log over the locally-teed one; the final 172-pass and
   0-warning logs were teed to unique names and copied back, and the package gate re-run last
   (it then reported 172 passed, 0 warnings).

### Deviations and content adaptations

- **The world file, not a Java world-data tag.** The artefact's `RfToolsRedstoneChannels` is a
  vanilla `AbstractWorldData`; the pin has no such registry a mod may add, but its per-save
  `horizon-mod-data/<mod id>/` file store has the same lifetime (backed up, deleted with the save,
  `NoWorld` before one opens). The channel `name` is not carried: upstream only writes and shows it
  in the GUI, which this port replaces with a numeric `redstone-set`. A user notices: channels are
  numbered, not named.
- **The GUI is a command.** `redstone-set` writes the channel, the receiver's `analog` flag and the
  saved input side; `channel = -1` allocates a fresh id, as the GUI's `getChannel(true)` does.
- **The input side is a saved advisory.** The jar's 24 `logic_facing` slab rotations cannot be
  content states; four horizontal sides are written by `redstone-set`, as every logic sibling's.
- **The transmitter is sampled every tick**, not on `neighborChanged`: the pin's block-entity tick
  reads the same directional signal, and the changed-only write keeps upstream's `prevIn` rule.
- No GUI, no comparator/TOP/Jade, no sounds, no item tooltip — the standing machines-tier gaps.
- **No new upstream defect** was adopted: the changed-only write, the digital/analog mapping and
  the world table are the artefact's own. `digit` stays blocked (`no-redstone-output:DigitTileEntity`).

### Coverage after slice 18

| family | status |
|---|---|
| `rftoolsutility:redstone_transmitter` (block, item, recipe, advancement, loot, model) | tested — R: placed and read back; R0/R1/R3: power, `prevIn` and table value asserted |
| `rftoolsutility:redstone_receiver` (block, item, 16-value `signal` state) | tested — R: placed and read back; R1/R2/R3/R4: digital 15, zero, analog 11 and the persisted 11 |
| the world-saved channel table (`lastId`, `getOrCreate`, value, text codec) | tested — R0 (allocation from `-1`), R1/R2 (write/drain), R4 (survives a real relaunch), unit codec round-trip |
| the digital 15/0 and analog exact-strength mapping | tested — R1 (15), R2 (0), R3 (partial 11 preserved); unit `a_digital_receiver…`, `an_analog_receiver…` |
| the input side (four horizontal facings, unknown refused) | tested in code (`logic_neighbours`, `redstone-set` refusal); R uses north |
| the item carrying each machine's data | carried (`block_drops` + `minecraft:block_entity_data`, the same path as the other machines); no dedicated phase |
| `redstone-set` / `redstone` commands and unknown-id refusal | tested — R0–R5; the parsed reply and the refused `ctl give` |
| a `rftoolsbase:machine_base` recipe craft | blocked(`rftoolsbase:machine_base`) — the parent port's item; the e2e gives the blocks, as every machine's |
| the channel `name` and the two GUIs | not ported (no menu; `redstone-set` is the equivalent) |
| `rftoolsutility:wire`, `redstone_information` | not ported — next logic gaps |

Skills: superpowers/using-superpowers, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices (brainstorming and rust-review/rust-port did not apply: the slice was fixed by the orchestrator's Next slice and this is a clean-room rewrite, not an audit or a translation)

### Artifact

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `rftoolsutility.hmod` (port root) | 246,711 B | `224a3fbba5a7d93c4cbef0e8b2ef73495c9a588bd036c0e7a3985ee8245b5684` | 313 |

Two `cargo horizon-mod build`s with `rm -f Cargo.lock` before each were `cmp`-identical
(`slice18-build-1.log`, `-2.log`), **no packed `Cargo.lock`**. `check` green
(`rftoolsutility 0.2.17 (native-mod, format
native-1/0a875da91dbb0fdcb7ba8c42c11341aa229af51a/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for
26.3)`); `fits --build 0a875da` → `0a875da  made for`; `fits --build 0b14f8a` → `0b14f8a  fits`;
`verify-sdk-package.py rftoolsutility` → **1/1** (172 passed, 0 clippy warnings, package matches the
tree, R9, skills gate; `slice18-package-gate-final.log`). The manifest, README, crate doc and
content were edited **before** the final build; the later `notes.md`/`VALIDATION.md`/`CONTENT.md`
edits and the harness-only e2e fix are not packed, and the gate still reports `package matches the
tree`. The launcher's `native-build` compiled the mod (`Compiling rftoolsutility (1 of 1)`,
`native-0a875da-34bb3642449d`) for the in-game legs. All Rust ladder steps printed
`build-slot: forwarding … to the remote runner` (hostname `AdLf-PC`), no `BUILD_LOCAL=1` was set,
and no lease was backgrounded.

## Next slice

The next logic-family gap on the same line is `wire` (`rftoolsutility:wire`, `WireTileEntity`), the
last redstone-logic sibling. It is a `LogicSlabBlock` whose `getRedstoneOutput` returns the input
side's `powerLevel` only on the output side and whose `checkRedstone` propagates a neighbour change
(`WireTileEntity.java` at `f799314`), so it fits **machines** with the same `LevelRef::signal` +
`BlockPart::block_signal` surface. **Gate first:** confirm the artefact's wire is a block (not a
Java-only network hook) and that its states are expressible as a content block; if not, stop with
`blocked(<symbol>)`. The concrete assertion: a redstone block feeds a short wire run built from
`rftoolsutility:wire`, a vanilla lamp at the far end lights at the artefact's declared strength
while a broken link stays dark, with a redstone-block lamp control and `SELFTEST_AT=wire`
ending `EXIT:1`. The other remaining gap, `redstone_information`, is an item whose whole behaviour
is the `GuiRedstoneInformation` tablet screen — check for a menu path before proposing it as a
slice; do not invent a GUI. Redistribute the modified brief from the orchestrator; this report
proposes it, it does not dispatch it.

## Slice 19 — the wire (2026-11-24)

**Chosen from the port's own `## Next slice`** (the section immediately above this one), verbatim:

> The next logic-family gap on the same line is `wire` (`rftoolsutility:wire`, `WireTileEntity`), the
> last redstone-logic sibling. It is a `LogicSlabBlock` whose `getRedstoneOutput` returns the input
> side's `powerLevel` only on the output side and whose `checkRedstone` propagates a neighbour
> change (`WireTileEntity.java` at `f799314`), so it fits **machines** with the same `LevelRef::signal`
> + `BlockPart::block_signal` surface. **Gate first:** confirm the artefact's wire is a block (not a
> Java-only network hook) and that its states are expressible as a content block; if not, stop with
> `blocked(<symbol>)`. The concrete assertion: a redstone block feeds a short wire run built from
> `rftoolsutility:wire`, a vanilla lamp at the far end lights at the artefact's declared strength
> while a broken link stays dark, with a redstone-block lamp control and `SELFTEST_AT=wire` ending
> `EXIT:1`. The other remaining gap, `redstone_information`, is an item whose whole behaviour is
> the `GuiRedstoneInformation` tablet screen — check for a menu path before proposing it as a
> slice; do not invent a GUI.

and the shared gap line `## Gaps (the rest of the mod)`:

> **Logic:** `wire`, `sensor`, `sequencer`, `timer`, `counter`, `analog`, `digit`, `logic`,
> `invchecker`, `redstone_transmitter/receiver`, `redstone_information`.

It fits the ledger's **machines** tier exactly: one block entity, one directional signal read, an
exact 0..15 cache and the already-proven `BlockPart::block_signal` surface — no renderer, no entity
AI, no new client half. **Pin** `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` /
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, API `0.2`, game 26.3. **Version 0.2.18.** Class
**content**, tier **machines** (ledger). MIT, ours on main → an **Update** of the entry (never a
first `Add`).

### Block gate — PASSED, not `blocked`

Read from the upstream `1.21_neo` clone at the artefact's own commit
`f799314007d68042df8504e83cd549360f2627c7` (analysis only, clone at `/tmp/opencode`), plus
McJtyLib's `LogicSupport.java`, `LogicSlabBlock.java` and `LogicFacing.java` at the same line:

- `WireTileEntity extends GenericTileEntity`; `createBlock()` returns a `LogicSlabBlock`. It has
  exactly two redstone methods — `getRedstoneOutput` and `checkRedstone` — and **no cross-block
  table, channel registry or network hook** (unlike slice 18's `RedstoneChannels`). It is a plain
  placeable block whose output is its input power on one side, with a neighbour-change nudge to the
  opposite side.
- Its 24 `logic_facing` rotations are a content-schema limitation shared by every logic sibling,
  not a missing API: the port carries four horizontal `wire_facing` values plus the cached strength
  in the block state. Not `blocked(<symbol>)`.

### The artefact's exact directional rule (read first)

```java
public int getRedstoneOutput(BlockState state, BlockGetter world, BlockPos pos, Direction side) {
    if (side == LogicSupport.getFacing(state).getInputSide()) { return powerLevel; } else { return 0; }
}
public void checkRedstone(Level world, BlockPos pos) {
    support.checkRedstone(this, world, pos);              // powerLevel = getSignal(input neighbour)
    if (loopDetector <= 0) {
        loopDetector++;
        BlockState state = world.getBlockState(pos);
        BlockPos offsetPos = pos.relative(LogicSupport.getFacing(state).getInputSide().getOpposite());
        if (world.hasChunkAt(offsetPos)) { world.neighborChanged(offsetPos, state.getBlock(), pos); }
        loopDetector--;
    }
}
```

McJtyLib `LogicSupport`: `checkRedstone` reads `world.getSignal(pos.relative(inputSide), inputSide)`;
`getRedstoneOutput` returns the cached power only when `side == inputSide`; `setRedstoneState`
notifies `pos.relative(inputSide.getOpposite())`. **Two direction conventions had to be mapped, and
both were found in game, not guessed:**

1. Vanilla `getSignal`'s direction is the *backwards* one ("directions in redstone signal related
   methods are backwards"). The SDK's `LevelRef::signal(pos, direction)` uses that convention, so
   the wire tick reads its input-side neighbour with `Direction::North` for a north input side
   (`wire_read_direction`), which dispatches the neighbour's hook with `face = South`.
2. The SDK's `BlockSignalEvent.face` is the **consumer-facing** side ("the side that touches the
   block reading it"). The artefact's `side == inputSide` condition therefore becomes, in SDK
   terms, `face == inputSide.getOpposite()` (`wire_java_side`). The first build emitted on
   `face == inputSide` and the far lamp stayed dark; the second build mapped the conventions
   correctly and the chain propagated 15 to a lit lamp.

The first mapping's failure is kept as `e2e-slice19-wire.log`'s earlier runs (overwritten by the
green leg) and described here: the root cause was the convention mismatch, diagnosed against
`LogicSupport`/McJtyLib and the SDK's `hook:block_signal` doc, not a guessed fix.

### What it does

- **The block.** `rftoolsutility:wire` on `minecraft:iron_block` with two state properties: `signal`
  (0…15, the exact strength for the redstone hook) and `wire_facing` (north/south/east/west). The
  jar's four slab models, top texture, item model, recipe, advancement and loot table are carried;
  its 24 `logic_facing` rotations are replaced by those two states.
- **The tick.** Reads the input-side neighbour's own emitted strength, runs the artefact's
  pass-through, mirrors the exact 0..15 into `signal` and the side into `wire_facing`, and wakes the
  neighbours on a change — `Neighbours` includes `inputSide.getOpposite()`, the output side
  `checkRedstone` nudges. The artefact's `loopDetector` is 0 outside its own synchronous nudge (the
  SDK schedules neighbour updates instead), so the guard's decision is kept as an observation.
- **The redstone.** `BlockPart::block_signal` reports the cached strength **only on the output
  side** (the consumer-facing face opposite the input side) and 0 elsewhere; `is_signal_source`
  marks the wire a source. A real vanilla redstone signal.
- **The harness.** `/rftoolsutility wire <pos> [nonce]` prints `input=… power=… output=… changed=…
  forward=… lamp=… facing=… time=…`; `wire-set <pos> <facing>` is the GUI equivalent. `changed`
  and `time` are game-time stamps, `forward` is the neighbour the guarded propagation feeds.

### Red → green (unit, verbatim)

Five pure rules were `todo!()` stubs behind their tests first (`slice19-red.log`):

```
test result: FAILED. 0 passed; 5 failed; 0 ignored; 0 measured; 172 filtered out
  runtime::tests::a_wire_emits_only_on_its_input_side                 not yet implemented
  runtime::tests::a_wire_clamps_the_strength_it_emits                 not yet implemented
  runtime::tests::a_wire_reads_the_neighbour_on_its_input_side       not yet implemented
  runtime::tests::a_wire_propagates_to_the_side_opposite_its_input   not yet implemented
  runtime::tests::a_wires_loop_guard_stops_reentrant_propagation     not yet implemented
```

Green: **180 passed; 0 failed** (`sdk-test.log` regenerated for 0.2.18 after the final build) — 99
runtime and 81 glue tests, including `the_wire_readout_is_the_line_the_harness_parses`,
`a_wires_cached_state_is_the_hook_answers_exactly_one_side` and
`a_wire_reads_its_input_neighbour_and_feeds_the_opposite_side`. `cargo fmt --check` clean and
`cargo clippy --all-targets --offline -- -D warnings` 0 warnings (`slice19-lint-final.log`).

### In game (evidence under `../_shared/evidence/rftoolsutility-e2e/`)

`tools/e2e-wire.sh` — a focused, fail-capable leg reusing the launcher-built game; one lease per
leg, every log ends with its exit. The assertion is a **vanilla redstone lamp** at the far end of a
six-wire run; a vanilla redstone block powers the same lamp first as the positive control.

| log | verdict |
|---|---|
| `e2e-slice19-wire.log` | **PASS**, `EXIT:0` — W–W4 |
| `e2e-slice19-wire-control.log` | **FAIL: ctl place rftoolsutility:wire … was refused three times**, `EXIT:1` (unmodded Nightly) |
| `e2e-slice19-wire-selftest.log` | **FAIL: SELFTEST wire: the fed run lit the far lamp, expected it to stay dark**, `EXIT:1` |
| `e2e-slice19-{sensor,sequencer,timer,counter,analog,logic,invchecker,redstone}.log` | each **PASS**, `EXIT:0` — the earlier logic families, no regression on the final bytes |

`check-e2e-logs.py rftoolsutility-e2e --strict` → `logs=118 ok=118 missing-exit=0 bad-exit=0`
(`slice19-e2e-gate.log`).

The phases, each asserting a parsed value (verbatim from the green transcript):

```
W-control: lamp with a vanilla redstone block: minecraft:redstone_lamp[lit=true] / after removing it: [lit=false]
W0:  wire at [202, 101, -3]: input=0 power=0 output=0 changed=-1 forward=[202, 101, -2] lamp=dark facing=north time=415 nonce=1
W1:  wires at z=-3…2 all input=15 power=15 output=15 changed=477; lamp at [202,101,3]=lit at the far end, time=537 nonce=7
W2:  wire at [202, 101, 1]: input=0 power=0 output=0 changed=552 … lamp=dark  (the middle wire was removed)
W3:  single wire at [205,101,8] north-facing: power=15 forward=[205,101,9] lamp=lit … nonce=10
     the same wire south-facing: power=0 forward=[205,101,7] lamp=dark signal=0 … nonce=11
W4:  give rftoolsutility:not_a_wire → rc=1 {"error":"unknown item rftoolsutility:not_a_wire"}
```

Counterfactuals: the redstone-block control proves the lamp and its `lit` parser work in this game;
W2's air gap is the broken-link differential against W1; W3's lone wire is the **directional**
differential — with no conducting chain, its output-side lamp lights on `north` and darkens on
`south`, and its cached `signal` state falls to 0. `SELFEST_AT=wire` inverts W1's far-lamp
assertion and fails as designed; `CONTROL=1` runs the same script on the unmodded Nightly.

### Measured traps this slice cost (harness, engine and SDK conventions)

1. **A chained run's `LevelRef::signal` reads are not a directional oracle.** The wire's block is an
   `iron_block` template — a redstone conductor — and `LevelRef::signal` adds conducted power
   *after* the redstone hook. After reversing a middle wire, its cached `signal` was 0 but the next
   wire still read 15 through conduction, so the far lamp stayed lit. W3 therefore uses a **lone**
   wire, whose lamp reads the hook's own weak signal directly, as the directional assertion. The
   run's far-lamp assertion (W1) and the air-gap differential (W2) still prove the end-to-end rule.
2. **The two direction conventions.** The first build compared the SDK's consumer-facing `face`
   directly with the artefact's backwards `side`; the lamp stayed dark. Mapping `side = face.getOpposite()`
   (`wire_java_side`) and reading the input neighbour with `wire_read_direction` made the chain pass
   15 and the lamp light (see the direction section above).
3. **The `forward=[x, y, z]` parser** first captured only `[202,` (up to the first space); the regex
   was changed to capture `\[[^\]]*\]`. The product code was correct — the harness parse was at fault.
4. **`cargo fmt --check` and clippy run on the remote copy**, whose pull-back excludes `*.rs`;
   `rustfmt --edition 2024` was applied locally before every final lint, and every Rust step printed
   `build-slot: forwarding … to the remote runner` (hostname `AdLf-PC`), no `BUILD_LOCAL=1`.

### Deviations and content adaptations

- **The input side is a block-state property, not a saved advisory.** Its siblings keep the facing
  only in the block entity, but `BlockPart::block_signal` receives the block state and cannot read
  block-entity data, so this wire carries `wire_facing` in the state as well (written by
  `wire-set`). A user still cannot orient the block by placement; the jar's 24 `logic_facing`
  rotations are not expressible as content states.
- **The GUI is a command.** `wire-set` writes the side the jar's GUI wrote; the loop detector's
  decision is an observation, not a saved field.
- **No new upstream defect adopted.** The pass-through, the changed-only nudge and the 0…15 exact
  strength are the artefact's own. No comparator/TOP/Jade, no sounds, no GUI — the standing
  machines-tier gaps. `digit` stays blocked (`no-redstone-output:DigitTileEntity`).
- The carried recipe needs `rftoolsbase:machine_base`, so the e2e gives the block, as every
  machine's does.

### Coverage after slice 19

| family | status |
|---|---|
| `rftoolsutility:wire` (block, item, recipe, advancement, loot, texture, `signal` + `wire_facing` states) | tested — W: placed and read back; content regenerated by `assemble_content.py` |
| the input-side read and the exact 0…15 pass-through | tested — W1 (six wires at 15), W2 (air gap → 0), unit rules for the clamp and source/forward sides |
| the directional output (only the output side) | tested — W3 (lone wire: north lights its lamp, south darkens it); unit `a_wires_cached_state_is_the_hook_answers_exactly_one_side` |
| the loop-guarded neighbour-change nudge | tested in code (`wire_loop_allows`, `Neighbours` includes the output side); the guard's decision is exposed in the readout |
| `wire` / `wire-set` commands and unknown-id refusal | tested — W0–W4; the parsed reply and the refused `ctl give` |
| a `rftoolsbase:machine_base` recipe craft | blocked(`rftoolsbase:machine_base`) — the parent port's item; the e2e gives the block |
| the GUI and the jar's 24 `logic_facing` rotations | not ported (no menu; `wire-set` is the equivalent) |
| `rftoolsutility:redstone_information`, `digit`, screens, teleporter GUIs | not ported — next gaps (see `## Next slice`) |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices (rust-review and rust-port did not apply: this slice was set by the orchestrator's Next slice and is a clean-room rewrite, not an audit or a translation)

### Artifact

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `rftoolsutility.hmod` (port root) | 255,734 B | `cb690fab949b3c4fb2f0353324f9687b62ea20aa4365ddb7157141d2a299d0fd` | 325 |

Two `cargo horizon-mod build`s with `rm -f Cargo.lock` before each were `cmp`-identical
(`slice19-final-build-1/2.hmod`), **no packed `Cargo.lock`**. `check` green
(`rftoolsutility 0.2.18 (native-mod, format
native-1/0a875da91dbb0fdcb7ba8c42c11341aa229af51a/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for
26.3)`); `fits --build 0a875da` → `0a875da  made for`; `fits --build 0b14f8a` → `0b14f8a  fits`;
`verify-sdk-package.py rftoolsutility` → **1/1** (180 passed, 0 clippy warnings, package matches the
tree, R9, skills gate; `slice19-package-gate-final.log`). The manifest, README, crate doc and content
were edited **before** the final build; the later `notes.md`/`VALIDATION.md`/`CONTENT.md` edits are not
packed, and the gate still reports `package matches the tree`. The launcher's `native-build` compiled
the mod (`Compiling rftoolsutility (1 of 1)`, `native-0a875da-0085ffa39d92`) after the stale
0.2.17 tag was removed. All Rust ladder steps printed `build-slot: forwarding … to the remote runner`
(hostname `AdLf-PC`), no `BUILD_LOCAL=1` was set, and no lease was backgrounded.

## Next slice

The only remaining logic-row gap is `redstone_information` — an **item** whose whole behaviour is
`GuiRedstoneInformation`, the tablet screen that shows the redstone channels and their values. It is
**not** a machines-tier block slice and there is no menu path in this port (all declared containers
have `menu: None`), so before dispatching check the artefact's `RedstoneInformationItem` at
`f799314` for a non-GUI, fail-capable oracle; if there is none, the honest next slice is a different
machines family from the ledger (the teleporter GUIs, screens or environmental controller) or a
stop, not an invented GUI. The concrete assertion for a tablet, if a menu path exists, would be: open
the tablet beside a transmitter on channel N, read the channel's value from the screen, and assert it
matches the world file's stored strength, with an unknown channel refused and `SELFTEST_AT=information`
ending `EXIT:1`. Redistribute the modified brief from the orchestrator; this report proposes it, it
does not dispatch it.

## Gate — `redstone_information` non-GUI oracle (2026-11-24): `blocked(no-information-oracle)`

This gate is the direct child of slice 19's `## Next slice` (the only remaining logic-row gap). It
was **gate-only**: no version, no package, no fake GUI. Pin `0a875da91d…` / `e59da909…`; artefact
`f799314007d68042df8504e83cd549360f2627c7` (branch `1.21_neo`, shallow clone at `/tmp/opencode`,
`HEAD` verified equal to the commit); SDK `ports/_shared/sdk/0a875da/api-source.tar.xz` unpacked to
`/tmp/opencode/rfu-sdk-api`. Full verbatim evidence:
`../_shared/evidence/rftoolsutility/gate-redstone-information-oracle.log`.

### Artefact source evidence — the item's whole behaviour is a GUI screen

- `RedstoneInformationItem.use()` (lines 88–97) does exactly one thing on the server: `openGui()` →
  `MenuProvider` → `RedstoneInformationContainer` → `GuiRedstoneInformation`. No world write, no
  signal, no command and no item mutation on use. `getChannels`/`addChannel`/`removeChannel`
  (100–119) are static item-component helpers over `ITEM_REDSTONE_INFORMATION_DATA`; the tooltip
  (62–65) lists channels client-side only.
- The **only** in-world writer of the tablet's channel set is a *different block*:
  `RedstoneChannelBlock.java:80` calls `RedstoneInformationItem.addChannel(stack, channel)` on a
  right-click of the channel-marker block. That block is not in this port.
- `RedstoneInformationContainer.broadcastChanges()` (62–99, server) merely gathers the tablet's
  channel set plus the world `RedstoneChannels` table's name/value and sends
  `PacketSendRedstoneData` to the client — no server state, signal or command output.
- The "channel table" itself is client-drawn widget text (`GuiRedstoneInformation.java:113–118`:
  `name.text(…)`, `value.text(Integer.toString(pair.getRight()))`).

### SDK grep evidence — no non-GUI, fail-capable oracle for that display

- `generated/wire_generated.rs:59`: `ScreenView { id, menu, title, rect, slots, mouse, partial_tick }`
  — no field carries drawn widget text. `ctl screen`/`menu` read container slots, never the labels
  the GUI paints; a headless run has no renderer, so a screenshot is an image, not an asserted value.
- The table's **data is already exposed non-GUI** by slice 18's `redstone` command: `src/lib.rs`
  `red_line()` states "the channel's table value", and `e2e-slice18-redstone.log` asserts R1
  `value=15`, R3 `value=11` from the parsed reply. A new tablet readout would restate an existing
  command, not add an oracle.
- Item-component surfaces exist (`hooks/notes.rs:1725` `component_types` +
  `native_component`/`native_set_component`; `item_stack.rs:334` `components()` / `:280`
  `set_component_value`), and menu/screen surfaces exist (`host/players.rs:712` `player_open_menu`,
  `host/menus.rs:20` `open_menu`, `hooks/notes.rs:1854` `screen_opening`/`ModScreen`). But the port
  has zero menu machinery (all containers `menu: None`), the sole world writer of the set is absent,
  and a command that writes/reads the component back only duplicates slice 18's already-proven
  table. Porting the screen would be an unprovable claim; fabricating a command stand-in would be the
  "fake GUI" the gate forbids.

### Verdict

**`blocked(no-information-oracle)`** — no version, no package, no fake GUI. Do not dispatch a 0.2.19
information slice on the current evidence; the item's only product is a GUI whose values are already
proven by slice 18.

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices (rust-review/rust-port did not apply: a source/SDK gate, not an audit or a translation)

## Next slice (proposal — not dispatched): environmental controller, first module slice

With `redstone_information` blocked, the honest next machines-family slice is from the shared gap
line `## Gaps (the rest of the mod)` (verbatim):

> **Environmental controller** and its 23 modules; ~~the **spawner** with its 100 mob recipes~~ …

It is the only unported **machines** family on that line that is expressible at this pin: the teleporter
GUI/dialing-device family was already gated as not a slice (`DialingDeviceTileEntity` has no tick and
no redstone output — "a new subsystem, not a slice"), and matter transmitter/receiver/booster +
destination analyzer are done (slices 9 and 10). The environmental controller is a block entity with a
500,000 FE buffer (`environmentalMaxRF`), a 20,000/tick input, a 7-slot module inventory, a server tick
and a radius/miny/maxy area — exactly the machines tier, no renderer or AI required (the
`EnvironmentalRenderer` is a client half dropped as a standing deviation).

**First surface gate to run (Step-0 gate):** confirm the artefact's effect application is expressible
— grep for and reach `LevelRef::entities_in` (`host/levels.rs:293`, `level_reads.rs:116`),
`Registrar::effect_key` (`parts/effects.rs:10`), and `LivingHost::living_effect(s)` /
`living_add_effect` (`host/living.rs:30/37/357`). If any is missing or unstamped, stop with
`blocked(<symbol>)` rather than invent an inert module. The 7 module slots and their carried item
component follow the already-proven crafter/slot path.

**Concrete fail-capable assertion (the slice's product):** a controller placed at `P`, mode
`MODE_ALL`, `radius=8`, `miny`/`maxy` around the player, one regeneration module inserted, fed
`>= rfNeeded`, enabled; after the module's `MAXTICKS=180` cadence the port's `/rftoolsutility
environmental <pos>` readout prints the player's active effect id and amplifier read from
`LivingHost::living_effect` — assert the parsed `effect=minecraft:regeneration amplifier=0` and that
the `energy` buffer fell by `rfNeeded`. Differentials: the same player with the controller disabled
(or outside `radius`) reads no such effect, and a controller with no module stays `active=false`;
`SELFTEST_AT=environmental` inverts the effect assertion and ends `EXIT:1`. Not dispatched here —
the orchestrator must cut a fresh self-contained brief.

## Slice 20 — the environmental controller, first module (2026-11-24)

**Chosen from the port's own gap list** — `## Gaps (the rest of the mod)` (verbatim):

> **Environmental controller** and its 23 modules; ~~the **spawner** with its 100 mob recipes~~ …

and the gate agent's `## Next slice (proposal — not dispatched): environmental controller, first
module slice` immediately above. It is the only unported **machines** family on that line that is
expressible at this pin: the teleporter GUI/dialing-device family was gated as a new subsystem, and
the transmitter/receiver/booster and destination analyzer are done (slices 9–10). One block entity
(500,000 FE, 20,000/tick input, seven module slots, a server tick over a radius/miny/maxy area)
fits the ledger's **machines** tier exactly — no renderer, no AI, no new client half.
`EnvironmentalRenderer` is dropped as a standing deviation.

**Pin** `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` / `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`,
API `0.2`, game 26.3. **Version 0.2.19.** Class **content**, tier **machines** (ledger). MIT, ours
on main → an **Update** of the entry (never a first `Add`).

### Surface gate first (the dispatch's Step-0 gate): not blocked

Measured against `ports/_shared/sdk/0a875da/api-source.tar.xz` (unpacked at
`/tmp/opencode/rfu-sdk-api`), not the historical gap table. Full verbatim evidence:
`../_shared/evidence/rftoolsutility/gate-environmental-effect-surface.log`. The artefact clone is
at `f799314007d68042df8504e83cd549360f2627c7`.

| Symbol | Where | Wired? |
|---|---|---|
| `LevelRef::entities_in(host, min, max)` | `generated/mod_api_generated_host.rs:1716`, `host/levels.rs:293` | `Backend::level_entities_in` hash in `generated/wire_generated.rs:1102` and the guest import — yes |
| `Registrar::effect_key(id)` | `src/parts/effects.rs:10` | `Registrar::effect_key` hash in `wire_generated.rs:1845` — yes |
| `LivingRef::effect(s)` | `host/living.rs:30/37`, generated host `:2583/:2593` | `Backend::living_effect`/`living_effects` hashes at `:1167/:1168` — yes |
| `LivingRef::add_effect(host, &ActiveEffect)` | `host/living.rs:357`, generated host `:2967` | `BackendMut::living_add_effect` hash at `:1527` — yes |
| `ActiveEffect::new(id,duration,amplifier,ambient,visible,show_icon)` | `entity/types.rs:296` | plain data — yes |

The effect is applied through the game's own `LivingEntity.addEffect`; the readout reads it back
through `LivingEntity.getEffect`. No effect command is invented. **Verdict: not blocked.**

### The artefact's constants (read first, not inferred from the name)

Read from `McJtyMods/RFToolsUtility` branch `1.21_neo`, commit `f799314…` (analysis only):

| Number | Value | Source |
|---|---|---|
| energy capacity / fill | 500,000 / 20,000 FE | `EnvironmentalConfiguration.ENVIRONMENTAL_MAXENERGY` / `…RECEIVEPERTICK` (`environmentalMaxRF` / `environmentalRFPerTick`) |
| minimum active usage | 5 FE/tick | `EnvironmentalConfiguration.MIN_USAGE` (`environmentalMinRFUsage`) |
| mob-mode multiplier | 2.0 | `EnvironmentalConfiguration.mobsPowerMultiplier`, applied in `MODE_HOSTILE/PASSIVE/MOBS/ALL` |
| module slots | 7 | `EnvironmentalControllerTileEntity.ENV_MODULES`, `SLOT_MODULES = 0` |
| volume | `(int)(radius² × π × (maxy − miny + 1))` | `EnvironmentalControllerTileEntity.getVolume()` |
| RF need | `(int)(Σ(module.rfPerTick × volume) × multiplier × (4 − infusion)/4)`, clamped up to 5 with modules | `getTotalRfPerTick()`; infusion 0 here |
| effect cadence / duration | 180 / 540 ticks | `PotionEffectModule.MAXTICKS`, `new MobEffectInstance(potion, MAXTICKS*3, amplifier, true, false)` |
| regeneration module | id `rftoolsutility:regeneration_module`, `minecraft:regeneration`, amplifier 0, 0.0015 FE/tick/volume | `EnvironmentalControllerItem.createRegenerationModule()`, `EnvironmentalConfiguration.REGENERATION_RFPERTICK` |
| area filter | horizontal `x²+z² < radius²`, absolute `miny ≤ y ≤ maxy` | `PotionEffectModule.processPlayers/processEntities`, AABB from `pos±radius` |
| `MODE_ALL` | affects players (`isPlayerAffected` true) **and every other entity** | `isEntityAffected` switch / `isPlayerAffected` |
| default data | radius 50, miny 30, maxy 70, blacklist, empty players | `EnvironmentalData.createDefault()` |
| timeout on a failed power check | 20 ticks | `EnvironmentalControllerTileEntity.tickServer` (`powerTimeout = 20`) |

### What it does

- **The block.** `rftoolsutility:environmental_controller` on the `minecraft:iron_block` template
  (full cube, `RotationType.NONE`, STANDARD_IRON properties), with the jar's multipart cube model
  and its `machineenvironmentalcontroller` side texture; the jar's `rftoolsbase:` base textures are
  re-pointed into this namespace. `rftoolsutility:regeneration_module` is a standalone item
  (`horizon/item/regeneration_module.json`, `template: minecraft:stick`) with the jar's own item
  texture. Both items, recipes, advancements, loot and lang lines are carried by
  `tools/assemble_content.py`.
- **The machine.** 500,000 FE fill-only on every face, seven automation-reachable module slots,
  and a server tick that follows the artefact's order: timeout → energy/module gate → spend →
  module cadence → area query → effect. A module installed while the buffer is empty (or the
  machine disabled) deactivates and waits 20 ticks; the first powered tick applies the effect and
  re-arms the 180-tick cadence (`activate(true)` seeds `ticks = 1`). `mode all` affects every
  living entity in the horizontal sphere and y band; the five other modes select by player list
  or mob kind. The jar's `SyringeRecipe` file is carried and its serializer registered as an inert
  **data kind** (`rftoolsutility:syringe`), exactly as the spawner's 70 recipes are — without it
  the game crashes on load with "unregistered recipe serializer".
- **The harness.** `/rftoolsutility environmental <pos> [nonce]` prints
  `energy=<held>/500000 active=<bool> mode=<mode> radius=<n> miny=<n> maxy=<n> rf=<need> modules=<n>
  effect=<id|none> amplifier=<n|->`. `effect` is read from the **command player's own** game store
  (`LivingRef::effect`), so the assertion is on the game, not a mirror. `environmental-set <pos>
  <radius> <miny> <maxy> <mode> <enabled>` replaces the GUI's area/mode/on-off fields; `buffer`
  fills a module slot and `energy` charges the buffer (both extended to this machine).

### Red → green (unit, verbatim)

Nine new runtime rules were written behind `todo!()` bodies first (`slice20-red-full.log`):

```
test result: FAILED. 182 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out
  an_environmental_volume_is_pi_times_radius_squared_times_height   panicked at src/runtime.rs:1865:5: not yet implemented
  environmental_power_scales_with_volume_and_the_mode_multiplier    panicked at src/runtime.rs:1865:5: not yet implemented
  one_regeneration_module_costs_the_minimum_usage_when_its_volume_is_small  panicked at src/runtime.rs:…: not yet implemented
  the_environmental_area_is_a_horizontal_sphere_inside_a_y_band     panicked at src/runtime.rs:1886:5: not yet implemented
  mode_all_affects_players_and_every_other_entity / the_player_list_modes… / the_mob_modes… / an_unknown_mode… / the_effect_is_due… / the_effect_duration…
```

One expectation was wrong on the first green run and was corrected *against the artefact* (radius
50, miny 30, maxy 70 → `(int)(2500×π×41) = 322,013`, not `314,159`); the red leg above is the
before-implementation run. Green: **191 passed; 0 failed** (`sdk-test.log` regenerated for 0.2.19
after the final build) — 191 runtime + glue tests. `cargo fmt --check` clean, `cargo clippy
--all-targets -- -D warnings` 0 warnings (`sdk-clippy.log`).

### In game (evidence under `../_shared/evidence/rftoolsutility-e2e/`)

`tools/e2e-environmental.sh` — a focused, fail-capable leg on the launcher-built game; one lease
per leg, every log ends with its exit.

| log | verdict |
|---|---|
| `e2e-slice20-environmental.log` | **PASS**, `EXIT:0` — EN–EN6 |
| `e2e-slice20-environmental-control.log` | **FAIL: ctl place rftoolsutility:environmental_controller … was refused three times**, `EXIT:1` (unmodded Nightly) |
| `e2e-slice20-environmental-selftest.log` | **FAIL: SELFTEST environmental: the powered module read minecraft:regeneration amplifier=0, expected none**, `EXIT:1` |
| `e2e-slice20-wire-regression.log`, `e2e-slice20-teleporter-regression.log` | each **PASS**, `EXIT:0` (earlier families on the final bytes) |

The phases, each asserting parsed fields of a nonce-pinned reply (verbatim from the green log):

```
EN:    energy=0/500000 active=false mode=all radius=8 miny=100 maxy=102 rf=0 modules=0 effect=none amplifier=- nonce=1
EN-control: a vanilla /effect give reads effect=minecraft:regeneration amplifier=0 (nonce=2); after /effect clear, effect=none (nonce=3)
EN1:   energy=0/500000 active=false … rf=5 modules=1 effect=none amplifier=- nonce=4       (a module with no FE)
EN2:   energy=4020/500000 active=true … rf=5 modules=1 effect=minecraft:regeneration amplifier=0 nonce=5
       the buffer fell 5000 → 4020 = 980 FE = 196 × rfNeeded=5
EN3:   energy=3980/500000 active=false … effect=none amplifier=- nonce=6                   (disabled)
EN4:   energy=2930/500000 active=true … effect=none amplifier=- nonce=7                     (player outside radius 8)
EN5:   energy=2885/500000 active=false … rf=0 modules=0 effect=none amplifier=- nonce=8     (module removed)
EN6:   give rftoolsutility:not_a_controller → rc=1, unknown item
```

Counterfactuals: EN-control proves the readout can see an effect at all; EN1 (no power), EN3
(disabled), EN4 (outside the radius) and EN5 (no module) each differ from EN2 in one input only;
`SELFTEST_AT=environmental` inverts EN2's effect assertion and fails as designed, so it can fail.
`CONTROL=1` runs the same script on the unmodded Nightly, where the block does not exist. The
power assertion is a parsed drop that is a positive multiple of the readout's own `rf` field.
`check-e2e-logs.py rftoolsutility-e2e --strict` → `logs=123 ok=123 missing-exit=0 bad-exit=0`
(the 118 earlier logs plus the five new ones).

### Measured traps this slice cost

1. **A standalone module item needs its own `horizon/item/` declaration.** With only the item
   model and item definition, the game crashed at load: `mod rftoolsutility requested unknown item
   rftoolsutility:regeneration_module`. `{"template": "minecraft:stick"}` fixes it and is generated
   by `assemble_content.py`.
2. **A carried recipe with an unregistered serializer crashes the game.** The jar's
   `regeneration_module.json` has `type: rftoolsutility:syringe`; dropping the file or registering
   a no-op crafting kind were the options. The port registers an inert data kind (as the spawner
   does), so the file loads and the module remains a harness-given item, like every machine.
3. **The first volume expectation was wrong**, not the code: `(radius² × π × height)` for the
   artefact's default bounds is 322,013, not 314,159. Recorded and corrected red-first.
4. **The remote pull-back clobbers locally-teed `sdk-test.log`/`sdk-clippy.log`.** Both were staged
   under `/tmp/opencode` and copied into the evidence tree **after** the last forwarded run, so the
   verifier reports `191 passed` / `0 warning(s)` rather than the stale slice-8 numbers.

### Deviations and content adaptations

- **No GUI, no renderer.** `environmental-set` writes the area/mode/on-off fields the GUI would,
  and `buffer` fills a module slot; `EnvironmentalRenderer` (the client half that draws the area)
  is dropped — outside the machines tier. A user cannot teach or arrange modules in a window.
- **One module of 23.** Only the regeneration module is carried and applied; the other 22 module
  items and their effects (including the four debuffs) are a later slice, as are their config
  multipliers. The seven slots and the module rule are already in place for them.
- **No infusing discount.** `getTotalRfPerTick`'s `(4 − infused)/4` term is carried as the
  uninfused value (infusion 0); upstream's infusable path needs `rftoolsbase`'s items (the crafter's
  standing gap).
- **The syringe recipe is inert data.** Upstream crafts a module with a witch syringe and a module
  template — neither item is ported — so the file is registered as a data kind and the module is
  given by the harness, like every machine block in this port.
- **The carried controller recipe is not crafted in game.** Its middle ingredient is
  `rftoolsbase:machine_frame`, which only the parent port registers (the tank's and the crafter's
  own gap); the e2e gives the block.
- **No new upstream defect was adopted.** The timeout, the module cadence, the area filter and the
  15 FE floor are the artefact's own rules.

### Coverage after slice 20

| family | status |
|---|---|
| `rftoolsutility:environmental_controller` (block, item, recipe, advancement, loot, texture) | tested — EN: placed and read back; EN6: an unknown id refused; content regenerated by `assemble_content.py` |
| `rftoolsutility:regeneration_module` (item, texture, syringe recipe data) | tested — EN1–EN5: inserted into slot 0, applied `minecraft:regeneration` amplifier 0, removed again |
| the 500,000 FE buffer, 20,000/tick fill and the volume-scaled FE/tick | tested — EN2: 5000 → 4020 FE = 196 × the readout's `rf=5`; EN1: no FE → inactive |
| the 7-slot module inventory and the power timeout | tested — EN1/EN5: 0 modules → `rf=0 active=false`; EN3: disabled → inactive |
| the 180-tick module cadence and 540-tick effect duration | exercised — EN2 after a 200-tick wait; unit rules `the_effect_is_due_on_the_first_active_tick_and_every_maxicks_after`, `the_effect_duration_is_three_module_cadences` |
| MODE_ALL, the six modes and the horizontal-sphere/y-band filter | tested — EN2 (inside), EN4 (outside); unit rules for all six modes and the area test |
| the player's own effect read from `LivingRef::effect` | tested — EN-control (vanilla give/clear), EN2 (mod effect), EN1/EN3/EN4/EN5 (none) |
| the other 22 environmental modules, the GUI and `EnvironmentalRenderer` | not ported (a later slice / a menu / a renderer) |
| a `rftoolsbase:machine_frame` recipe craft | blocked(`rftoolsbase:machine_frame`) — the parent port's item; the e2e gives the block |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

### Artifact

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `rftoolsutility.hmod` (port root) | 270,586 B | `7a13f87dcf1804dfee0d72971a02603ad2611deb060dd3f59b71d0be20fae612` | 340 |

Two `cargo horizon-mod build`s with `rm -f Cargo.lock` before each were `cmp`-identical
(`slice20-build-a.sha`, `-b.sha`), **no packed `Cargo.lock`**. `check` green
(`rftoolsutility 0.2.19 (native-mod, format
native-1/0a875da91dbb0fdcb7ba8c42c11341aa229af51a/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for
26.3)`), `fits --build 0a875da` → `0a875da  made for`, `fits --build 0b14f8a` → `0b14f8a  fits`,
`verify-sdk-package.py rftoolsutility` → **1/1** (`package matches the tree`, 191 tests, 0 warnings,
R9, skills gate). The packed docs (manifest description, README, crate doc) were edited **before**
the final build; `sdk-test.log`/`sdk-clippy.log` were regenerated after it. The launcher's
`native-build` compiled the mod (`Compiling rftoolsutility (1 of 1)`,
`native-0a875da-32eb334b0e05`) after stale `native-*` tags were removed. All Rust ladder steps
printed `build-slot: forwarding … to the remote runner` (hostname `AdLf-PC`); no `BUILD_LOCAL=1`
was set, no lease was backgrounded, and no turn ended with a shell outstanding.

## Next slice

The next environmental-family slice is the **second module** (or the whole module set): carry
another jar module item from `assets/rftoolsutility/models/item/envmodules/*.json` (e.g.
`regenerationplus_module`, amplifier 2, 0.0045 FE/tick/volume, or one of the 17 buff modules) and
add its effect id/amplifier to the same seven-slot module rule. It fits the **machines** tier
exactly — no new block entity, no new tick, one carried item and one effect constant. The artefact
rule to read first is `EnvironmentalControllerItem.create*Module()` and `PotionEffectModule`'s
`amplifier`/`getRfPerTick` for that module, plus `BuffEModule` for the non-potion ones (those need
a player buff store this port does not have — check first; if the buff has no fail-capable game
oracle, stop with `blocked(<buff>)`, as the redstone-information gate did). The concrete assertion:
insert the new module **alongside** the regeneration module and assert the player's own effect
readout reports both ids/amplifiers (or the stronger one replacing the weaker), with the FE spend
rising by the module's own `rfPerTick × volume`; `SELFTEST_AT=environmental2` inverts one amplifier
and must `EXIT:1`. If no second module has a game-readable effect, the honest next slice is a
different machines family from the ledger or a stop — not an invented buff readout.
