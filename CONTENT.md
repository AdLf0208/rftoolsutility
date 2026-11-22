# RFTools Utility slices 1–8 — the carried content

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
| `assets/rftoolsutility/textures/block/machinecrafter1.png` | the crafter tier 1's front face (`CrafterModule`'s `p.frontBasedModel("crafter1", …)`) |
| `assets/rftoolsutility/models/item/crafter1.json` | the crafter tier 1's item model (`parent: block/crafter1`) |
| `assets/rftoolsutility/textures/block/machinecrafter2.png`, `machinecrafter3.png` | the crafter tiers 2 and 3 front faces (`frontBasedModel("crafter2"/"crafter3", …)`) |
| `assets/rftoolsutility/models/item/crafter2.json`, `crafter3.json` | the tier item models (`parent: block/crafter2`/`crafter3`) |
| `data/rftoolsutility/recipe/crafter2.json`, `crafter3.json` | the **inner** `minecraft:crafting_shaped` of the jar's `mcjtylib:copy_components` wrapper: redstone torch / crafting table + the previous tier + crafting table / redstone torch |
| `data/rftoolsutility/advancement/recipes/misc/crafter2.json`, `crafter3.json` | the tiers' recipe unlocks (the previous tier as the criterion) |
| `data/rftoolsutility/loot_table/blocks/crafter2.json`, `crafter3.json` | the tiers' drops |
| `assets/rftoolsutility/models/item/spawner.json` | the spawner's item model (`parent: block/spawner`) |
| `assets/rftoolsutility/textures/block/machinespawner.png` | the spawner's front face (`SpawnerModule`'s `frontBasedModel("spawner", …)`) |
| `data/rftoolsutility/recipe/spawner.json` | redstone/rotten flesh/ender pearl/blaze rod/bone around a machine frame |
| `data/rftoolsutility/advancement/recipes/misc/spawner.json` | the recipe unlock |
| `data/rftoolsutility/loot_table/blocks/spawner.json` | the block's drop |
| `data/rftoolsutility/recipe/minecraft_*.json` (**70**) | the jar's mob recipes, **byte for byte** — their type is the port's own crafting kind; the spawner reads their `entity`, `power` and `item1..3.amount`, and the matter beamer matches its material against their `item1..3.object` (see below) |
| `data/rftoolsutility/tags/item/living/{living,highyield,averageyield,lowyield}.json` | the jar's own yield tags, **byte for byte**: a recipe's empty (`living?`) `object` takes its factor from them (1.5 / 1.0 / 0.5) |
| `assets/rftoolsutility/blockstates/matter_beamer.json` | the jar's two variants, `lit=false` → `block/matter_beamer_off`, `lit=true` → `block/matter_beamer_on` |
| `assets/rftoolsutility/models/block/matter_beamer_off.json`, `_on.json` | the jar's `cube_all` cubes, each drawing its own `machinebeameroff`/`machinebeamer` texture |
| `assets/rftoolsutility/models/item/matter_beamer.json` | the item model (`parent: block/matter_beamer_on`) |
| `assets/rftoolsutility/textures/block/machinebeamer.png`, `machinebeameroff.png` | the beamer's lit and dark faces |
| `data/rftoolsutility/recipe/matter_beamer.json` | redstone/glowstone/redstone around a machine frame |
| `data/rftoolsutility/advancement/recipes/misc/matter_beamer.json` | the recipe unlock |
| `data/rftoolsutility/loot_table/blocks/matter_beamer.json` | the block's drop |
| `data/rftoolsutility/recipe/crafter1.json` | redstone torch / crafting table + machine frame + crafting table / redstone torch |
| `data/rftoolsutility/advancement/recipes/misc/crafter1.json` | the recipe unlock |
| `data/rftoolsutility/loot_table/blocks/crafter1.json` | the block's drop |
| `assets/rftoolsutility/models/block/tank_inventory.json` | upstream's datagen inventory cube (front `tank0`, machine sides) |
| `assets/rftoolsutility/models/item/tank.json` | the item model (`parent: rftoolsutility:block/tank_inventory`) |
| `data/rftoolsutility/recipe/tank.json` | 3 glass / bucket + machine frame + bucket / 3 iron ingots |
| `data/rftoolsutility/advancement/recipes/misc/tank.json` | the recipe unlock |
| `data/rftoolsutility/loot_table/blocks/tank.json` | the block's drop |
| `assets/rftoolsutility/lang/en_us.json` (12 keys) | the blocks' names, the tab title, upstream's tooltip lines (including the tiers' "up to 4/8 recipes at once") |

## Patched, because 1.21.1 → 26.3 differs

| File | Patch |
|---|---|
| the recipe | `{"item": X}` → `"X"`, `{"tag": T}` → `"#T"` (26.3's ingredient form; the 1.21.1 object form is an unknown ingredient) |
| `loot_table/blocks/{tank,crafter1,crafter2,crafter3,spawner,matter_beamer}.json` | the entry's `copy_components` function is removed (it names mcjtylib's `item_fluid` components, which this port does not register — see the gap list); `copy_name` stays, and **no** `condition` key is added: the jar's pool has none, and 26.3 reads a pool's condition as singular and drops an absent one unconditionally |
| `tank_inventory.json` | `rftoolsbase:block/base/…` → `rftoolsutility:block/base/…` (the base textures travel in this package) |
| `models/block/crafter1.json`, `crafter2.json`, `crafter3.json`, `models/block/spawner.json` | the same re-point of the jar's own cube models (`rftoolsbase:block/base/…` → `rftoolsutility:block/base/…`) |
| `recipe/crafter2.json`, `crafter3.json` | the `mcjtylib:copy_components` wrapper is **unwrapped** to its inner `minecraft:crafting_shaped` recipe (the component copy needs mcjtylib's recipe type, which this port does not register — see `notes.md` § Slice 5, deviations) |
| the 70 mob recipes | **nothing**: the 1.21.1 `item1..3.object` ingredient forms (an item, a tag, and `item2`'s *list*) are never parsed as ingredients here, so the files travel exactly as the jar ships them |
| the block model | the jar's `block/tank.json` is a custom model loader (`rftoolsutility:tankloader`), which the pin cannot run: it becomes nine cubes (`block/tank0.json` … `tank8.json`), each drawing `tank<level>` on the four sides and the machine top and bottom — the same six faces `TankBakedModel` draws |

## Written by the assembler

| File | What |
|---|---|
| `assets/rftoolsutility/blockstates/tank.json` | nine variants, `level=0` … `level=8` |
| `assets/rftoolsutility/blockstates/{crafter1,crafter2,crafter3}.json`, `blockstates/spawner.json` | the jar's rotations for the four horizontal `facing` values (the front is the model's north face) |
| `assets/rftoolsutility/items/{crafter1,crafter2,crafter3}.json`, `items/spawner.json`, `items/matter_beamer.json` | the 26.3 item definitions (`minecraft:model` → `rftoolsutility:item/<name>`) |
| `assets/rftoolsutility/items/tank.json` | the 26.3 item definition (`minecraft:model` → `rftoolsutility:item/tank`) |
| `data/rftoolsutility/horizon/block/tank.json` | the block: `minecraft:iron_block` template, hardness 2.0 / blast 2.0, correct tool, metal sound, full cube, and the declared `level` state (9 values, default 0) |
| `data/rftoolsutility/horizon/block/{crafter1,crafter2,crafter3}.json` | the blocks: `minecraft:white_glazed_terracotta` template (a full cube whose four-way `facing` placement rule is measured on a content block), hardness 2.0 / blast 2.0, correct tool, metal sound |
| `data/rftoolsutility/horizon/block/spawner.json` | the block: `minecraft:white_glazed_terracotta` template (the crafter's measured four-way `facing`), the same iron properties |
| `data/rftoolsutility/horizon/block/matter_beamer.json` | the block: `minecraft:iron_block` template (upstream's `RotationType.NONE`), the same iron properties, and the declared `lit` state (`false`/`true`, default `false`) the jar's blockstate keys its two models on |
| `data/rftoolsutility/horizon/creative_tab/rftoolsutility.json` | `itemGroup.rftoolsutility`, the tank, the three crafter tiers, the spawner and the matter beamer |
| `data/c/tags/item/{ingots/iron,glass_blocks,bones,rods/blaze,ender_pearls}.json` | the convention tags the recipes name (the prepared 26.3 root ships none) |
| `data/minecraft/tags/block/{mineable/pickaxe,needs_iron_tool}.json` | the tank, the three crafter tiers, the spawner and the matter beamer (upstream's `ironPickaxeTags()`) |

## What the content cannot do alone

- **The recipes need `rftoolsbase:machine_frame`**, an item the `rftoolsbase`
  port's content registers. A game with only this port has both recipes loaded but
  uncraftable (an unknown ingredient is an empty one), so the e2e gives the tank
  and the crafter instead of crafting them. With both ports installed the recipes
  work as upstream's do.
- **The advancement's `frame` criterion** names the same item; its
  `has_the_recipe` criterion still unlocks the recipe.
- **A tier upgrade does not carry the machine's data.** The jar's `crafter2`/`crafter3` recipes
  are `mcjtylib:copy_components` wrappers that copy the old machine's taught recipes, buffer and
  energy onto the crafted one. This port carries the inner shaped recipe only, so the crafted
  machine starts empty (the gap is mcjtylib's recipe type — see `notes.md` § Slice 5).
- **The 70 mob recipes are not grid recipes.** Their `type` (`rftoolsutility:spawner`) is the
  port's own crafting kind: `matches` is false and `assemble` is empty, as upstream's own
  `SpawnerRecipe` answers, and the spawner block reads their `entity`, `power` and `item1..3`
  amounts (the matter beamer matches its material against their `item1..3.object`). They are in
  force only because `src/lib.rs` registers the kind.
- **The `living/*` tags name `c:` convention tags the prepared 26.3 root does not ship**
  (`c:crops`, `c:foods`, `c:eggs`, `c:mushrooms`, `c:seeds`). Those memberships are the pack's;
  this port declares only the five `c:` tags its own recipes name, so with this port alone a
  living item yields through the tags the vanilla root does ship (`#minecraft:fishes`,
  `#minecraft:leaves`, `#minecraft:saplings`, `#minecraft:flowers`) and the ids the files list
  directly (`minecraft:beef`, `minecraft:apple`, `minecraft:cactus`, …) — which is what the e2e
  proves (`minecraft:beef` → 1.5).
- **The fluid drawn inside the tank's window** is upstream's renderer
  (`TankBakedModel`'s inner quads, tinted by the fluid's still texture). The pin
  has no block-entity renderer in this slice, so the fill is shown by the nine
  side textures only — the level is real, the inner fluid quad is a gap.
