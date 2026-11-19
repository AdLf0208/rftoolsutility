# RFTools Utility — findings (jar inventory, then the tank)

Ported from **RFTools Utility 1.21-7.0.13** (`McJtyMods/RFToolsUtility`, MIT), the
exact ATM10 jar `ports/_shared/licence-cache/rftoolsutility-1.21-7.0.13.jar`
(SHA-256 `615642b1…d3ff`, 1,051 entries, 3,032,989 B uncompressed). Upstream
behaviour was read from branch `1.21_neo`, commit
`f799314007d68042df8504e83cd549360f2627c7` (`version=1.21-7.0.13` in
`gradle.properties`), analysis only: this is a clean-room rewrite.

**Licence as read from the jar:** `META-INF/neoforge.mods.toml` line 3 says
`license="MIT License https://github.com/McJtyMods/RFToolsUtility/blob/1.16/LICENCE"`.
The jar carries no `LICENCE` file of its own, so the text was taken from that
repository path: the MIT licence, `Copyright (c) 2019 McJtyMods` (kept in
[`LICENSE`](LICENSE)). **MIT → rank 0 (permissive).** Local-only until the
maintainer approves publishing (ARCHITECTURE.md §8).

## 1. The jar, counted (not read from the name)

`unzip -l` of the 1,051 entries:

| Group | Count | What |
|---|---:|---|
| `data/rftoolsutility/recipe/` | 137 | 1 per block/item + **100 spawner-entity recipes** (`minecraft_<mob>.json`) |
| `assets/rftoolsutility/models/item/` | 98 | the mod's items, plus `_0`…`_8` animation frames (porters, syringes) |
| `assets/rftoolsutility/models/block/` | 65 | 27 blocks + logic/wire/screen variants |
| `assets/rftoolsutility/blockstates/` | 28 | 27 blocks + `screen_hitblock` |
| `data/rftoolsutility/loot_table/blocks/` | 27 | one per block |
| `assets/rftoolsutility/textures/item/` | 29 + 23 `envmodules` + 15 `modules` | module icons |
| `mcjty/rftoolsutility/**` | — | classes only (never read: clean-room) |

The 27 blocks: `analog`, `counter`, `crafter1/2/3`, `creative_screen`,
`destination_analyzer`, `dialing_device`, `digit`, `environmental_controller`,
`invchecker`, `logic`, `matter_beamer/booster/receiver/transmitter`,
`redstone_receiver/transmitter`, `screen`, `screen_controller`,
`screen_hitblock`, `sensor`, `sequencer`, `simple_dialer`, `spawner`, `tank`,
`timer`, `wire`.

The 99 items include the block items above plus: `tablet_screen`,
`tablet_redstone`, `screen_link`, `storage_control_module`, the 30 screen
modules (`text_module`, `counter_module`, `energy_module`, `fluid_module`,
`inventory_module`, `redstone_module`, `button_module`, `clock_module`,
`computer_module`, `machineinformation_module`, …), the environmental modules
(`featherfalling`, `flight`, `glowing`, `haste`, `luck`, `nightvision`,
`noteleport`, `peaceful`, `poison`, `regeneration`, `saturation`, `slowness`,
`speed`, `waterbreathing`, `weakness`, `blindness`, `+plus` variants),
`module_template`, `moduleplus_template`, `syringe`, `charged_porter`,
`advanced_charged_porter`, `teleport_probe`, `redstone_information`.

**This is a machines mod with several families** — teleporters (porter, matter
transmitter/receiver/beamer/booster, dialing device, simple dialer, destination
analyzer), screens (screen + controller + hitblock + 30 modules + tablet),
logic (wire, sensor, sequencer, timer, counter, analog, digit, logic,
invchecker, redstone transmitter/receiver), environmental controller + 23
modules, spawner (+100 mob recipes), and four single machines: `tank`,
`crafter1/2/3`, `environmental_controller`, `creative_screen`. Slice 1 takes
**one** of them; the rest are the gap list.

## 2. The chosen machine: the tank

Chosen because its loop closes **without a GUI** — the block's own use fills and
drains it with buckets — and every number it uses is in the jar or its source.

### Numbers (each with where it came from)

| Number | Value | Source |
|---|---:|---|
| capacity | **32,000 mB** (32 buckets) | `TankConfiguration.MAXCAPACITY` `defineInRange("maxCapacity", 32000, 0, Integer.MAX_VALUE)`; the jar's own manual says "can (by default) hold up to 32 buckets" (`assets/rftoolsbase/patchouli_books/manual/en_us/entries/machines/tank.json`) |
| one bucket | 1,000 mB | NeoForge/vanilla: a bucket is 1,000 mB (`FluidUtil` container move) |
| fill level | 0 when empty, else `min(8, 8*amount/capacity + 1)` | `TankTE.computeLevel` |
| filter slot | 1 slot, bucket items only | `TankTE.CONTAINER_FACTORY` (`new ContainerFactory(1).slot(specific(s -> s.getItem() instanceof BucketItem).in().out(), SLOT_FILTER, 151, 10)`) |
| filter behaviour | a filled bucket in slot 0 locks the tank to that fluid | `TankTE.isFluidValid` + `updateFilterFluid`; manual "Filter" page |
| block properties | hardness 2.0, blast 2.0, correct tool, metal sound | `BaseBlock`/`BlockBuilder` defaults, the same ones `rftoolsbase`'s Machine Infuser carries (our own port's record) |
| model | top `machinebase:machinetop`, bottom `machinebottom`, **all four sides `tank0`…`tank8` by fill level** | `TankBakedModel.getQuads` + `TankTE.AMOUNT` |
| recipe | 3 glass / bucket + machine frame + bucket / 3 iron ingots | `data/rftoolsutility/recipe/tank.json` (carried) |
| loot | the block item, `copy_name` + `copy_components` of `mcjtylib:item_energy/inventory/fluids` | `data/rftoolsutility/loot_table/blocks/tank.json` |

### Behaviour (from `TankTE`, `TankModule`, `TankBakedModel`, `CustomTank`)

1. **The block entity.** `TankTE extends GenericTileEntity`: an item handler of 1
   slot (filter), a fluid handler `CustomTank(MAXCAPACITY)` declared as
   `CapType.FLUIDS` (all sides) and `CapType.ITEMS_AUTOMATION`, and a
   `CapType.CONTAINER` menu ("Tank"). Saved keys: `level` (the render level) and
   `tank` (the fluid handler), `items` (the filter slot).
2. **`onBlockActivated` (server side only).**
   `FluidUtil.tryEmptyContainerAndStow(heldItem, fluidHandler, null, MAX_VALUE, player, true)`
   first: a filled bucket (or any fluid container) in the hand pours its fluid
   into the tank and the held item becomes the empty container. Then
   `tryFillContainerAndStow(...)`: an empty bucket in the hand takes one 1,000 mB
   bucket out of the tank and becomes the filled bucket. Either success returns
   `SUCCESS` (the hand is set to the result); otherwise `PASS`, which for a
   `BaseBlock` with a container capability opens the tank's GUI.
3. **One fluid at a time.** `CustomTank` is a NeoForge `FluidTank`: it holds one
   fluid id; a fill of a different fluid into a non-empty tank is refused, a fill
   of the same fluid is clamped to the capacity, and `drain` gives what is asked
   up to what it holds.
4. **The filter.** Slot 0 takes bucket items only; its contents set
   `filterFluid`, and `isFluidValid` then refuses every other fluid. A filter is
   only re-read in the slot's `onUpdate`, so a filter bucket that is **loaded
   from NBT** (a restart) is not applied until the slot changes again — see
   §4.
5. **The render level.** `computeLevel` maps the amount to 0…8; the block's
   model loader picks `tank<level>` for the four sides and draws the fluid as an
   inner quad (tinted by the fluid's own still texture) inside the tank's window.
6. **The item carries the contents.** `collectImplicitComponents` /
   `applyImplicitComponents` move `mcjtylib:item_fluids` between the block entity
   and its item, and the loot table copies it on break, so a broken tank keeps
   its fluid. Its tooltip is `Contents: <amount>mb <fluid> (32000 mb)`.
7. **Pipes.** The `CapType.FLUIDS` capability is every side, fill and drain, so
   `native_fluid_insert`/`native_fluid_extract` are the same door a pipe uses.

### What the pin can express

| Upstream | Pin | Note |
|---|---|---|
| `EntityBlock.newBlockEntity` | `block_entity` (API 15, any template since API 129) | |
| `getTicker` | `block_entity_ticks` + `block_entity_tick` (15/17) | the tick mirrors the fill level into the block state |
| `Capabilities.FluidHandler.BLOCK` | `block_entity_fluids` (API 54) | contents live in the entity's saved `fluids` compound |
| `FluidUtil.tryEmpty/FillContainerAndStow` | `right_click_block` (API 21) | the pin has **no** pre-block-use hook, so the click is claimed there; a host's bucket never reaches `bucket_empty` (API 179) — the same finding `enderstorage` recorded |
| the block state's render level | a declared `level` state property 0…8 + `native_set_block_property` (API 18) | the jar's own `tank0`…`tank8` textures become 9 block models; the fluid quad inside the window is a renderer gap |
| the `CapType.CONTAINER` menu | none | the slice has no GUI: a non-bucket click does nothing |
| `mcjtylib:item_fluids` on the item | not in slice 1 | `loot_drops` (API 75) + a component type + `place_block` would carry it; deferred to slice 2 |
| `isFluidValid` (filter) | `native_block_entity_accepts_fluid` is **not stamped** at this pin | the filter slot is a gap; nothing fakes it |

## 3. The rest of the mod (gap list)

Teleporters (`charged_porter`, `advanced_charged_porter`, `teleport_probe`,
`matter_transmitter/receiver/beamer/booster`, `dialing_device`,
`simple_dialer`, `destination_analyzer`, and the `playerprops` teleport
registry); screens (`screen`, `screen_controller`, `screen_hitblock`,
`tablet_screen`, `tablet_redstone`, `screen_link`, `storage_control_module`,
the 30 screen modules); logic (`wire`, `sensor`, `sequencer`, `timer`,
`counter`, `analog`, `digit`, `logic`, `invchecker`,
`redstone_transmitter/receiver`, `redstone_information`); the environmental
controller and its 23 modules; the spawner and its 100 mob recipes; the three
crafters; `creative_screen`.

## 4. Upstream defects and deviations (see `notes.md` for the ruling)

- **The filter is lost on load.** `TankTE.loadAdditional` loads `items` but never
  calls `updateFilterFluid`, which only runs in the handler's `onUpdate`. After a
  restart a tank with a water bucket in the filter slot accepts lava again until
  the slot is touched. **Not reproduced, not fixed: the port has no filter slot
  at all** (the pin cannot refuse a fluid — see §2), so the port cannot carry the
  bug. Recorded here so the slice-2 filter implementation recomputes the filter
  on load.
- **The render level is saved in NBT.** `loadAdditional` reads `level` back into
  `amount`, which is a client-side display cache; on the server the tick's
  `computeLevel` overwrites it. The port derives the level from the tank every
  tick instead, so a stale `level` cannot survive.
- **`onBlockActivated` PASSes on the client.** The server does the work; the
  port's `right_click_block` is a server hook, so the same shape.
