# RFTools Utility slice 1 — the carried content

The `assets/` and `data/` at the package root are generated from the pack jars by
[`tools/assemble_content.py`](tools/assemble_content.py); rerun it to rebuild
(they ship inside the SDK package `rftoolsutility.hmod`, beside `src/` and
`horizon-mod.toml`).
Two jars are read:

- `ports/_shared/licence-cache/rftoolsutility-1.21-7.0.13.jar`, SHA-256
  `615642b1…d3ff` — the mod itself (its fill textures, models, recipe, loot
  table, advancement, lang);
- `ports/_shared/licence-cache/rftoolsbase-1.21-6.0.11.jar`, SHA-256
  `5195ba53…4b00` — the three machine base textures the tank's own models name
  (`rftoolsbase:block/base/…`). The port must stand alone (one `.hmod`, no
  second content package), so they are copied into this namespace and the
  models re-pointed. Both jars are MIT and this campaign's own ports.

Nothing is invented: the nine fill textures, the inventory model, the item
model, the recipe, the loot table, the advancement and the lang lines come out
of the RFTools Utility jar, re-spelled only where 26.3 differs.

## Carried from the jar, byte for byte

| File | Why |
|---|---|
| `assets/rftoolsutility/textures/block/tank0.png` … `tank8.png` | the nine fill-level side textures (`TankBakedModel.getSideTexture`) |
| `assets/rftoolsutility/models/block/tank_inventory.json` | upstream's datagen inventory cube (front `tank0`, machine sides) |
| `assets/rftoolsutility/models/item/tank.json` | the item model (`parent: rftoolsutility:block/tank_inventory`) |
| `data/rftoolsutility/recipe/tank.json` | 3 glass / bucket + machine frame + bucket / 3 iron ingots |
| `data/rftoolsutility/advancement/recipes/misc/tank.json` | the recipe unlock |
| `data/rftoolsutility/loot_table/blocks/tank.json` | the block's drop |
| `assets/rftoolsutility/lang/en_us.json` (5 keys) | the block name, the tab title, upstream's tooltip lines |

## Patched, because 1.21.1 → 26.3 differs

| File | Patch |
|---|---|
| the recipe | `{"item": X}` → `"X"`, `{"tag": T}` → `"#T"` (26.3's ingredient form; the 1.21.1 object form is an unknown ingredient) |
| `loot_table/blocks/tank.json` | the entry's `copy_components` function is removed (it names mcjtylib's `item_fluid` components, which this port does not register — see the gap list); `copy_name` stays, and **no** `condition` key is added: the jar's pool has none, and 26.3 reads a pool's condition as singular and drops an absent one unconditionally |
| `tank_inventory.json` | `rftoolsbase:block/base/…` → `rftoolsutility:block/base/…` (the base textures travel in this package) |
| the block model | the jar's `block/tank.json` is a custom model loader (`rftoolsutility:tankloader`), which the pin cannot run: it becomes nine cubes (`block/tank0.json` … `tank8.json`), each drawing `tank<level>` on the four sides and the machine top and bottom — the same six faces `TankBakedModel` draws |

## Written by the assembler

| File | What |
|---|---|
| `assets/rftoolsutility/blockstates/tank.json` | nine variants, `level=0` … `level=8` |
| `assets/rftoolsutility/items/tank.json` | the 26.3 item definition (`minecraft:model` → `rftoolsutility:item/tank`) |
| `data/rftoolsutility/horizon/block/tank.json` | the block: `minecraft:iron_block` template, hardness 2.0 / blast 2.0, correct tool, metal sound, full cube, and the declared `level` state (9 values, default 0) |
| `data/rftoolsutility/horizon/creative_tab/rftoolsutility.json` | `itemGroup.rftoolsutility`, the tank |
| `data/c/tags/item/{ingots/iron,glass_blocks}.json` | the convention tags the recipe names (the prepared 26.3 root ships none) |
| `data/minecraft/tags/block/{mineable/pickaxe,needs_iron_tool}.json` | the tank (upstream's `ironPickaxeTags()`) |

## What the content cannot do alone

- **The recipe needs `rftoolsbase:machine_frame`**, an item the `rftoolsbase`
  port's content registers. A game with only this port has the recipe loaded but
  uncraftable (an unknown ingredient is an empty one), so the e2e gives the tank
  instead of crafting it. With both ports installed the recipe works as
  upstream's does.
- **The advancement's `frame` criterion** names the same item; its
  `has_the_recipe` criterion still unlocks the recipe.
- **The fluid drawn inside the tank's window** is upstream's renderer
  (`TankBakedModel`'s inner quads, tinted by the fluid's still texture). The pin
  has no block-entity renderer in this slice, so the fill is shown by the nine
  side textures only — the level is real, the inner fluid quad is a gap.
