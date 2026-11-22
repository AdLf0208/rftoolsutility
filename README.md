# RFTools Utility for Horizon — slices 1–8: the tank, the crafter tiers, the spawner and the matter beamer

A clean-room rewrite of [RFTools Utility](https://github.com/McJtyMods/RFToolsUtility)
1.21-7.0.13 for [Horizon](https://github.com/horizon-rust/horizon) (Minecraft
Java 26.3). MIT; the original copyright notice travels with the port (see
[`LICENSE`](LICENSE) and [`LICENSE-NOTICE.md`](LICENSE-NOTICE.md)).

**Status: slices 1–8, the tank, the crafter tiers, the spawner's matter gate and the matter
beamer.** One flat SDK package (`rftoolsutility.hmod`, mod API 0.2) carries the blocks, the
items, the models and textures, and the behaviour — see [`notes.md`](notes.md) § "Slice 8 — the
matter beamer", § "Slice 7 — the spawner's matter gate", § "Slice 6 — the tick's own dimension",
§ "Slice 5 — the crafter tiers", § "Slice 4 — the spawner", § "Slice 3 — the crafter" and
§ "Slice 2 — the tank's contents travel with its item".

## What works

- `rftoolsutility:tank` — a **32,000 mB** tank (32 buckets; the mod's own manual
  and `maxCapacity`'s default), one fluid at a time, every face a fluid
  capability so pipes fill and drain it.
- **Right-click with a bucket**: a water or lava bucket pours exactly 1,000 mB
  in and leaves an empty bucket in the hand; an empty bucket takes exactly
  1,000 mB out and leaves a filled one. A full tank, a tank of another fluid, or
  a tank holding less than a whole bucket does nothing. Any other registered
  fluid container works too, through the engine's own fluid-container path.
- **The contents travel with the block**: breaking a filled tank drops a tank
  item that carries the fluid (`minecraft:block_entity_data`), and placing that
  item restores the tank; an empty tank's item carries nothing, and an item
  edited to more than the capacity places a clamped tank.
- **The fill level is visible**: the block's own `level` state (0…8) picks the
  jar's nine `tank0`…`tank8` side textures, and the server tick keeps it on the
  tank's contents however they changed.
- `rftoolsutility:crafter1`, `crafter2`, `crafter3` — one **crafting machine** in the
  jar's three tiers: a 41-slot container (10 ghost, 26 input, 4 output, the filter slot),
  a **50,000 FE** buffer filling at 500 a call, and a server tick that runs the taught 3x3
  crafting recipes from the input buffer for 100 FE a cycle. A crafter keeps **2, 4 or 8**
  recipes by tier (`CrafterBaseTE.createTier1/2/3`), crafted in order, and `mode int` puts a
  result back in the input buffer so the next recipe can use it (**chained crafting**: a log
  becomes planks, the planks a crafting table, for 100 FE). `keep` leaves the last item of
  each slot; `speed fast` runs five cycles a tick. Hoppers reach the input and output slots on
  every face, and a broken crafter's item carries its recipes, buffer and energy — each tier
  under its own block entity id.
- `rftoolsutility:spawner` — a **200,000 FE** buffer filling at 2,000 a call, and a
  server tick that spawns the mob the machine's setting names, on the block's facing
  side, for **that mob's own carried recipe's** RF cost: `minecraft:zombie` 800 FE,
  `minecraft:blaze` 1,000 FE. The mob is set from the jar's **70** `rftoolsutility:spawner`
  recipes — a crafting kind this package registers, so the files are in force — and an id
  no recipe names is refused. A spawn also needs **that recipe's three matter amounts** in the
  machine's three stores (`maxMatterStorage` 6,400 each; the **matter beamer** below fills them,
  and `/rftoolsutility matter` is the harness's direct route): 800 FE with empty stores spawns
  nothing and keeps its FE, one
  type one unit short is a refusal, and a spawn spends each store by its own amount. A new mob
  clears the stores, and a broken spawner's item carries the mob, its numbers and its matter.
- `rftoolsutility:matter_beamer` — the machine that **feeds** a spawner's stores: one material
  slot, a **200,000 FE** buffer filling at 1,000 a call, a redstone signal to run, and a send every
  20 ticks. A send moves **one block** of the material and costs **2,000 FE**
  (`beamRfPerObject`), and adds `blocks × factor` to the store that the spawner's recipe's
  `item1..3` names: an item or a tag yields 1, the recipe's empty (living) form takes the
  `living/*` tags' **1.5 / 1.0 / 0.5**. A material the recipe does not take, an unpowered beamer
  and one without the FE all send nothing and spend nothing. The destination is linked with
  `/rftoolsutility link` (upstream's wrench, within `maxBeamDistance` 8 and only to a spawner;
  linking it to itself unlinks it), the block's `lit` state shows a send, and a broken beamer's
  item carries its destination, material and energy.
- Commands (the harness, as upstream has no commands): `/rftoolsutility info
  <pos> [nonce]` and `tank <pos> <fluid> <mb>` for the tank; `/rftoolsutility
  crafter <pos> [nonce]`, `recipe <pos> <index> <id>`, `buffer <pos> <slot> <item>
  <count>`, `energy <pos> <amount>` and `setting <pos> speed|mode|keep <value>`
  for the crafter (and `energy`/`buffer` for the beamer too); `/rftoolsutility spawner <pos>
  [nonce]`, `mob <pos> <id>`, `matter <pos> <m0> <m1> <m2>` and `recipes [nonce]` for the
  spawner, its stores and its table; `/rftoolsutility beamer <pos> [nonce]` and
  `link <pos> <dest>` for the beamer and its destination.

## What does not (yet)

Teleporters, screens and their 30 modules, the logic blocks, the environmental
controller and its modules, the tank's GUI and filter slot, the
fluid quad inside its window, the crafter's GUI, the spawner's GUI, the syringe that sets the
mob, six-way facing, the item tooltips, the wrench item (a command links the beamer instead) and
the infusing power discount. The full list, with the reason for each, is in
[`notes.md`](notes.md).

## Building and checking

```sh
BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sdk-rfu --port ports/rftoolsutility \
    --step build -- bash -c 'cd ports/rftoolsutility && rm -f Cargo.lock && cargo horizon-mod build'
BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sdk-rfu --port ports/rftoolsutility \
    --step check -- bash -c 'cd ports/rftoolsutility && cargo horizon-mod check rftoolsutility.hmod &&
        cargo horizon-mod fits rftoolsutility.hmod --build 0a875da'
```

The in-game scenario (`tools/e2e.sh`) needs a game built with this package by the
launcher: `ports/tools/launcher-game.sh build rftoolsutility` compiles the mod
into the Nightly at the pinned commit and exports `GAME=`/`CTL=` for the leg.

See [`notes.md`](notes.md) for the hooks, the numbers and their sources, and
[`FINDINGS.md`](FINDINGS.md) for the upstream analysis.
