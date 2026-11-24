# RFTools Utility for Horizon — slices 1–19: the tank, the crafter tiers, the spawner, the matter beamer, the teleporter family, the sensor, the sequencer, the timer, the counter, the analog, the logic gate, the inventory checker, the redstone transmitter/receiver and the wire

A clean-room rewrite of [RFTools Utility](https://github.com/McJtyMods/RFToolsUtility)
1.21-7.0.13 for [Horizon](https://github.com/horizon-rust/horizon) (Minecraft
Java 26.3). MIT; the original copyright notice travels with the port (see
[`LICENSE`](LICENSE) and [`LICENSE-NOTICE.md`](LICENSE-NOTICE.md)).

**Status: slices 1–19, the tank, the crafter tiers, the spawner's matter gate, the matter
beamer, the matter transmitter / receiver / booster, the destination analyzer, the sensor,
the sequencer, the timer, the counter, the analog, the logic gate, the inventory checker, the
redstone transmitter / receiver and the wire.**
One flat SDK package (`rftoolsutility.hmod`, mod API 0.2) carries the blocks, the items, the
models and textures, and the behaviour — see [`notes.md`](notes.md) § "Slice 19 — the wire", § "Slice 18 — the redstone transmitter/receiver", § "Slice 17 — the inventory checker",
§ "Slice 16 — the logic gate",
§ "Slice 15 — the analog",
§ "Slice 14 — the counter",
§ "Slice 13 — the timer",
§ "Slice 12 — the sequencer",
§ "Slice 11 — the sensor",
§ "Slice 10 — the destination analyzer", § "Slice 9 — the teleporter family",
§ "Slice 8 — the matter beamer", § "Slice 7 — the spawner's matter gate", § "Slice 6 — the tick's own dimension",
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
- `rftoolsutility:matter_transmitter`, `matter_receiver`, `matter_booster` — the
  **teleporter family**. Despite the name, these teleport **players**, not items or fluid: the
  transmitter's beam finds a player standing above it, charges the dialed trip's RF over its
  ticks (a local trip is `5,000 + 10·distance` FE over `5 + distance/100` ticks, capped at a
  cross-dimension `100,000` FE / `50` ticks), and moves the player to the dialed coordinate —
  across dimensions through the SDK's entity level transfer. A player leaving the beam or a
  power failure interrupts the trip with a cooldown; a receiver needs two clear blocks above and
  5,000 FE. An adjacent matter booster plus 20,000 FE mitigates a weak receiver. All three
  blocks and their carried content ship; the dialing device, destination GUI, syringe and beam
  renderer do not.
- `rftoolsutility:destination_analyzer` — a **passive** block (no block entity and no vanilla
  redstone output: that is what the jar's `DestinationAnalyzerBlock` and its manual say).
  Next to a dialed transmitter it enables the transmitter's receiver check, every 20 ticks: the
  destination level is resolved by dimension id and its receiver block entity read there, then
  readiness is that receiver's two clear blocks above plus **5,000 FE**. `/rftoolsutility
  analyzer <pos>` reports it as a harness signal — **15** (green, ready), **7** (yellow,
  destination dimension not loaded) or **0** (red, blocked or underpowered) — the numeric form
  of the beam colour the jar's lang line describes. Removing the analyzer forces the ready
  state again, as upstream does.
- `rftoolsutility:sensor` — a **block** whose ten-tick check reads the block on its input side
   and emits a **real redstone signal** through the SDK's `block_signal` hook: **15** when the
   named block matches (`blocks <id>`), otherwise **0**. With the matcher empty (`any`) it
   accepts any occluding block, upstream's empty-ghost-slot rule. `/rftoolsutility sensor-set
   <pos> <kind> <target> <area> <group> <facing>` is the GUI's equivalent (only AREA_1 and
   GROUP_ONE are carried); an unknown type/facing is refused. The jar's 24 cosmetic slab
   rotations are dropped, so the input side is a saved setting, and a broken sensor's item
   carries that setting and the last signal.
- `rftoolsutility:sequencer` — a **64-step pattern** driven by **rising edges** of an input
   redstone signal (`pulse = powered && !prevIn`, the artefact's own rule) and emitted as a real
   15/0 signal through the same hook as the sensor. `Step` advances one step per pulse, `Once1/2`
   run one cycle, `Loop1/2` cycle forever, `Loop3/4` follow the signal level and `Loop4` resets
   when it ends. `checkOutput()` reads the pattern bit at the current step, or the `endstate` when
   stopped at step −1; the checker runs every `delay` ticks (default 1 over 64 steps). A vanilla
   lever on the input side supplies clean edges: the tick reads that neighbour's directional
   `SignalGetter.getSignal`, not `getBestNeighborSignal` (which includes the block's own 15 and
   would latch a phantom edge). `/rftoolsutility sequencer-set <pos> <mode> <bits> <delay>
   <count> <endstate> <facing>` is the GUI's equivalent (bits in decimal or `0x` hex), and a
   broken sequencer's item carries its whole state.
- `rftoolsutility:timer` — a **repeating redstone pulse** machine (`TimerTileEntity`). Its
  `delay` (20 ticks by default) and `pauses` toggle are written by the jar's GUI, so here
  `/rftoolsutility timer-set <pos> <delay> <pauses> <facing>` sets them. A rising edge on the
  input side (`powered && !prevIn`, read directionally as the sequencer's is) reloads the
  countdown; when it reaches zero the block emits **15 for exactly one tick** and restarts, so
  without a new edge it pulses every `delay` ticks — the jar's manual calls it "a simple gate
  that will repeatedly send a redstone pulse". With `pauses=true` the countdown is held while
  the input stays high, so timely pulses stop the output (overdue detection). `/rftoolsutility
  timer <pos> [nonce]` prints the parsed `delay`, `pauses`, `output`, `timer`, the game-time
  stamps of the last edge and fire, and the input side; the in-game leg uses those stamps to
  assert a pulse exactly `delay - 1` game ticks after an edge. A broken timer's item carries
  its state, as every machine's does.
- `rftoolsutility:counter` — a **pulse counter** (`CounterTileEntity`). A rising edge on the
  input side bumps its running count; when it reaches the command-set limit the count wraps to 0
  and the block **latches 15 until the next pulse** turns it off (the jar's manual: "another
  redstone pulse will turn off the signal and reset Current to 0"). The GUI's `Counter`/`Current`
  fields are written by `/rftoolsutility counter-set <pos> <limit> <facing>`; the readout
  `/rftoolsutility counter <pos> [nonce]` prints `limit=… current=… output=<0|15> pulses=… pulse=…
  fire=… lamp=… facing=… time=…`, so a leg can assert that a vanilla lamp lights only on the
  limit-th lever pulse from the replies' own game-time stamps (`fire` is the game tick of the limit
  edge, `pulse` the last edge's tick). A broken counter's item carries its state.

- `rftoolsutility:analog` — an **arithmetic redstone block** (`AnalogTileEntity`, `AnalogData`).
   It reads its input side (I) and the two side inputs (A = left/clockwise, B = right/counterclockwise)
   and emits `(int)(I * mul + add)` from the branch chosen by `A == B`, `A < B` or `A > B`, clamped
   to **0..15** (the artefact's `createDefault()` is the identity `1,1,1 / 0,0,0`). The six GUI
   fields are written by `/rftoolsutility analog-set <pos> <mul_eq> <mul_less> <mul_greater>
   <add_eq> <add_less> <add_greater> <facing>`; the readout `/rftoolsutility analog <pos> [nonce]`
   prints `input=… a=… b=… branch=… mul=… add=… output=… changed=… lamp=… facing=… time=…`, so the
   leg can assert the exact declared strength and its game-time stamp from a vanilla lamp beside it.
- `rftoolsutility:logic` — a **three-input logic gate** (`ThreeLogicTileEntity`, `ThreeLogicData`).
   It reads its left (bit 0), input (bit 1) and right (bit 2) side neighbours' own emitted signals,
   sums them into a 0…7 mask, and drives the mask's cell of an eight-cell truth table as **15** for
   `1`, **0** for `0`, or the previous output for `-1` (keep) — the artefact's `s == 1 ? 15 : 0` rule.
   The eight GUI cells are written by `/rftoolsutility logic-set <pos> <l0>…<l7> <facing>`; the
   readout `/rftoolsutility logic <pos> [nonce]` prints `mask=… cell=… output=… changed=… lamp=…
   facing=… time=…`, so a leg can assert the declared 15/0 from a vanilla lamp and the reply's own
   game-time stamps. As with every logic sibling, the jar's 24 cosmetic slab rotations are dropped
   and the input side is a saved setting.
- `rftoolsutility:invchecker` — an **inventory checker** (`InvCheckerTileEntity`, `IncCheckerData`).
   Every ten ticks it reads the live item slots of the neighbour on its input side through the
   game's `ITEMS` capability (`level.capability(&ITEMS, pos, side)`, no provider of ours needed) and,
   when the configured slot holds at least the configured count of a matching item, drives **15**,
   else **0**. `IncCheckerData.createDefault()` is `amount=1, slot=0, DMG_IGNORE, tag=null`; an
   optional ghost matcher (item + damage mode) and an item tag gate are applied as in
   `isItemMatching`/`checkOutput`. The GUI fields are written by
   `/rftoolsutility invchecker-set <pos> <amount> <slot> <damage> <matcher> <mdamage> <tag> <facing>`
   and the readout `/rftoolsutility invchecker <pos> [nonce]` prints `seen=… count=… amount=… slot=…
   matched=… output=… changed=… lamp=… facing=… time=…`, so a leg can assert the count/threshold
   from a vanilla chest, a vanilla lamp and the reply's own game-time stamps.
- `rftoolsutility:redstone_transmitter` and `rftoolsutility:redstone_receiver` — the wireless
   channel pair. The transmitter reads the input-side neighbour's redstone strength and writes it
   into a numbered channel in the save's own `<world>/horizon-mod-data/rftoolsutility/
   redstone-channels.tsv` file (the pin's equivalent of the artefact's world-saved
   `RedstoneChannels` table); the receiver reads that channel every tick and drives **15** in
   digital mode (the default) or the exact **0..15** strength in analog mode. `/rftoolsutility
   redstone-set <pos> <channel> <analog> <facing>` replaces the two GUIs (`channel = -1` allocates
   a fresh id), and `/rftoolsutility redstone <pos> [nonce]` prints the parsed channel, power,
   table value, output, lamp and game time — so a leg can prove a lamp beside a receiver on the
   fed channel lights while one on another channel stays dark, across a relaunch.
- Commands (the harness, as upstream has no commands): `/rftoolsutility info
  <pos> [nonce]` and `tank <pos> <fluid> <mb>` for the tank; `/rftoolsutility
  crafter <pos> [nonce]`, `recipe <pos> <index> <id>`, `buffer <pos> <slot> <item>
  <count>`, `energy <pos> <amount>` and `setting <pos> speed|mode|keep <value>`
  for the crafter (and `energy`/`buffer` for the beamer too); `/rftoolsutility spawner <pos>
  [nonce]`, `mob <pos> <id>`, `matter <pos> <m0> <m1> <m2>` and `recipes [nonce]` for the
  spawner, its stores and its table; `/rftoolsutility beamer <pos> [nonce]` and
  `link <pos> <dest>` for the beamer and its destination; `/rftoolsutility dial <pos> <dim> <x>
  <y> <z>`, `transmitter <pos> [nonce]`, `receiver <pos> [nonce]` and `analyzer <pos> [nonce]`
  for the teleporter family (the analyzer prints `signal=<0|7|15> status=<warn|unknown|ok>`);
  `/rftoolsutility sensor <pos> [nonce]` and `sensor-set <pos> <kind> <target> <area> <group>
  <facing>` for the sensor (the readout prints `signal=<0|15> type=… target=… facing=…`).

  The sequencer's commands are `/rftoolsutility sequencer <pos> [nonce]` and `sequencer-set
  <pos> <mode> <bits> <delay> <count> <endstate> <facing>`; its readout prints `step=… bits=…
  mode=… output=<0|15> …`.

   The timer's commands are `/rftoolsutility timer <pos> [nonce]` and `timer-set <pos> <delay>
   <pauses> <facing>`; its readout prints `delay=… pauses=… output=… timer=… pulse=… fired=…
   facing=… time=…` (the last three are game-time stamps, so a leg can count ticks).

   The counter's commands are `/rftoolsutility counter <pos> [nonce]` and `counter-set <pos>
   <limit> <facing>`; its readout prints `limit=… current=… output=… pulses=… pulse=… fire=…
   lamp=… facing=… time=…` (`pulse`/`fire`/`time` are game-time stamps).

   The analog's commands are `/rftoolsutility analog <pos> [nonce]` and `analog-set <pos> <mul_eq>
   <mul_less> <mul_greater> <add_eq> <add_less> <add_greater> <facing>`; its readout prints
   `input=… a=… b=… branch=… output=… changed=… lamp=… facing=… time=…` (`changed`/`time` are
   game-time stamps, `output` is the exact 0..15 strength the redstone hook reports).

   The logic gate's commands are `/rftoolsutility logic <pos> [nonce]` and `logic-set <pos> <l0>
   <l1> <l2> <l3> <l4> <l5> <l6> <l7> <facing>`; its readout prints `mask=… cell=… output=…
   changed=… lamp=… facing=… time=…` (`changed`/`time` are game-time stamps, `output` is the 15/0
   the redstone hook reports).

   The inventory checker's commands are `/rftoolsutility invchecker <pos> [nonce]` and
   `invchecker-set <pos> <amount> <slot> <damage> <matcher> <mdamage> <tag> <facing>`; its readout
   prints `seen=… count=… amount=… slot=… matched=… output=… changed=… lamp=… facing=… time=…`
   (`changed`/`time` are game-time stamps, `output` is the 15/0 the redstone hook reports).

   The redstone pair's commands are `/rftoolsutility redstone <pos> [nonce]` and
   `redstone-set <pos> <channel> <analog> <facing>`; the readout prints `channel=… power=…
   prev=… value=… output=… changed=… lamp=… facing=… time=…` (`changed`/`time` are game-time
   stamps, `output` is the exact strength the redstone hook reports).

   The wire's commands are `/rftoolsutility wire <pos> [nonce]` and `wire-set <pos> <facing>`; its
   readout prints `input=… power=… output=… changed=… forward=… lamp=… facing=… time=…` (`changed`
   is a game-time stamp, `forward` is the neighbour the loop-guarded propagation feeds, and
   `output` is the exact strength the redstone hook reports **only on the input side**).

## What does not (yet)

The dialing device, destination GUI, screens and their 30 modules, the rest of the logic
blocks (the sensor, sequencer, timer, counter, analog, logic gate, inventory checker, redstone transmitter/receiver and wire are ported), the environmental controller and its modules, the tank's GUI and filter slot, the
fluid quad inside its window, the crafter's GUI, the spawner's GUI, the syringe that sets the
mob, six-way facing, the item tooltips, the wrench item (a command links the beamer instead),
the infusing power discount and the analyzer's vanilla redstone output (the artefact has none —
the harness reads the signal instead). The `digit` block is **not ported**: its whole behaviour is
a client sprite renderer (`DigitRenderer`, `machineoutput_0`…`_f`) with no redstone output at all,
so there is no fail-capable lamp assertion and a chat readout would be a fake. The full list, with
the reason for each, is in
[`notes.md`](notes.md).

## Building and checking

```sh
ports/tools/build-slot.sh run --job W1-rftoolsutility --port ports/rftoolsutility \
    --step build -- bash -c 'rm -f Cargo.lock && CARGO_INCREMENTAL=0 cargo horizon-mod build'
ports/tools/build-slot.sh run --job W1-rftoolsutility --port ports/rftoolsutility \
    --step check -- bash -c 'cargo horizon-mod check rftoolsutility.hmod &&
        cargo horizon-mod fits rftoolsutility.hmod --build 0a875da'
```

The in-game scenario (`tools/e2e.sh`) needs a game built with this package by the
launcher: `ports/tools/launcher-game.sh build rftoolsutility` compiles the mod
into the Nightly at the pinned commit and exports `GAME=`/`CTL=` for the leg.

See [`notes.md`](notes.md) for the hooks, the numbers and their sources, and
[`FINDINGS.md`](FINDINGS.md) for the upstream analysis.
