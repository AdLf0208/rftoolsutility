# RFTools Utility for Horizon — slice 1: the tank

A clean-room rewrite of [RFTools Utility](https://github.com/McJtyMods/RFToolsUtility)
1.21-7.0.13 for [Horizon](https://github.com/horizon-rust/horizon) (Minecraft
Java 26.3). MIT; the original copyright notice travels with the port (see
[`LICENSE`](LICENSE) and [`LICENSE-NOTICE.md`](LICENSE-NOTICE.md)).

**Status: slice 1, the tank.** One flat SDK package
(`rftoolsutility.hmod`, mod API 0.2) carries the block, the item, the models and
textures, and the behaviour — see [`notes.md`](notes.md) §
"Migration to the SDK format".

## What works

- `rftoolsutility:tank` — a **32,000 mB** tank (32 buckets; the mod's own manual
  and `maxCapacity`'s default), one fluid at a time, every face a fluid
  capability so pipes fill and drain it.
- **Right-click with a bucket**: a water or lava bucket pours exactly 1,000 mB
  in and leaves an empty bucket in the hand; an empty bucket takes exactly
  1,000 mB out and leaves a filled one. A full tank, a tank of another fluid, or
  a tank holding less than a whole bucket does nothing. Any other registered
  fluid container works too, through the engine's own fluid-container path.
- **The fill level is visible**: the block's own `level` state (0…8) picks the
  jar's nine `tank0`…`tank8` side textures, and the server tick keeps it on the
  tank's contents however they changed.
- `/rftoolsutility info <pos> [nonce]` prints the fluid, the amount, the
  capacity, the level and the held item; `/rftoolsutility tank <pos> <fluid>
  <mb>` sets the contents (an operator harness control, like
  `/rftoolsbase energy`).

## What does not (yet)

Teleporters, screens and their 30 modules, the logic blocks, the environmental
controller and its modules, the spawner, the three crafters, the tank's GUI and
filter slot, and the tank's contents on its item. The full list, with the reason
for each, is in [`notes.md`](notes.md).

## Building and checking

```sh
BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sdk-rfu --port ports/rftoolsutility \
    --step build -- bash -c 'cd ports/rftoolsutility && rm -f Cargo.lock && cargo horizon-mod build'
BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sdk-rfu --port ports/rftoolsutility \
    --step check -- bash -c 'cd ports/rftoolsutility && cargo horizon-mod check rftoolsutility.hmod &&
        cargo horizon-mod fits rftoolsutility.hmod --build nightly'
```

The in-game scenario (`tools/e2e.sh`) needs a game built with this package by
the launcher (`horizon-launcher native-build …`), which the campaign box does not
have; it takes `GAME=` and `CTL=`.

See [`notes.md`](notes.md) for the hooks, the numbers and their sources, and
[`FINDINGS.md`](FINDINGS.md) for the upstream analysis.
