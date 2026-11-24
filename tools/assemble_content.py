#!/usr/bin/env python3
"""Assemble the slices' `assets/` and `data/` folders from the pack's RFTools Utility jar.

The jar is the ATM10 file (`ports/_shared/licence-cache/rftoolsutility-1.21-7.0.13.jar`,
SHA-256 `615642b1…d3ff`), kept as the licence and provenance evidence. Nothing
is invented: the tank's nine fill textures, its inventory model, item model,
loot table, recipe, advancement and lang lines come out of that jar; the three
machine base textures come out of the *RFToolsBase* jar (the tank's own models
name `rftoolsbase:block/base/…`, and this port must stand alone), copied into
this namespace and re-pointed. Only spellings 26.3 needs are rewritten: a
recipe ingredient `{"item": X}` becomes `"X"` and `{"tag": T}` becomes `"#T"`,
the item model becomes a 26.3 item definition, and the block model is nine
per-level cubes because the pin has no custom model loader. The crafter tier 2/3
recipes are the jar's `mcjtylib:copy_components` wrapper unwrapped to its inner
shaped recipe (`unwrap_recipe`).

The block/item/creative-tab declarations and the `c:` tags the recipe names are
written here, because a carry has no oracle for a code-registered block and no
convention tags without one.

Rerun it to rebuild `assets/` and `data/` (it deletes its own generated folders first).
The flat SDK package carries them at its root, beside `horizon-mod.toml`; the manifest
itself is hand-maintained, because it is also the code package's identity:

    python3 tools/assemble_content.py
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import shutil
import zipfile

ROOT = Path(__file__).resolve().parents[1]
JAR = ROOT.parent / "_shared/licence-cache/rftoolsutility-1.21-7.0.13.jar"
BASE_JAR = ROOT.parent / "_shared/licence-cache/rftoolsbase-1.21-6.0.11.jar"
CONTENT = ROOT
MOD = "rftoolsutility"
# The pack's jar, as SHA-256 (the licence cache's file).
JAR_SHA256 = "615642b184ba1b9228d7cd942a5d700228a47133419d61a2037b79bae618d3ff"
# RFToolsBase 1.21-6.0.11, whose `block/base` textures the tank's models name.
BASE_JAR_SHA256 = "5195ba530e6cf9ba61c9954a3297679e6d29aa1b6182e27ae14ea43463dd4b00"

# Carried byte-for-byte from the RFTools Utility jar.
CARRIED_ASSETS = [
    "assets/rftoolsutility/models/item/tank.json",
    "assets/rftoolsutility/textures/block/tank0.png",
    "assets/rftoolsutility/textures/block/tank1.png",
    "assets/rftoolsutility/textures/block/tank2.png",
    "assets/rftoolsutility/textures/block/tank3.png",
    "assets/rftoolsutility/textures/block/tank4.png",
    "assets/rftoolsutility/textures/block/tank5.png",
    "assets/rftoolsutility/textures/block/tank6.png",
    "assets/rftoolsutility/textures/block/tank7.png",
    "assets/rftoolsutility/textures/block/tank8.png",
    "data/rftoolsutility/advancement/recipes/misc/tank.json",
    "data/rftoolsutility/recipe/tank.json",
    "data/rftoolsutility/loot_table/blocks/tank.json",
    # Slice 3: the crafter tier 1 (its own front texture, item model, recipe, advancement,
    # loot table and block model).
    "assets/rftoolsutility/models/item/crafter1.json",
    "assets/rftoolsutility/textures/block/machinecrafter1.png",
    "data/rftoolsutility/advancement/recipes/misc/crafter1.json",
    "data/rftoolsutility/recipe/crafter1.json",
    "data/rftoolsutility/loot_table/blocks/crafter1.json",
    # Slice 5: the crafter tiers 2 and 3, the same block entity with four and eight recipe
    # slots. Their recipes are the jar's `mcjtylib:copy_components` wrapper around a plain
    # `minecraft:crafting_shaped`; the wrapper is unwrapped below (see `unwrap_recipe`).
    "assets/rftoolsutility/models/item/crafter2.json",
    "assets/rftoolsutility/models/item/crafter3.json",
    "assets/rftoolsutility/textures/block/machinecrafter2.png",
    "assets/rftoolsutility/textures/block/machinecrafter3.png",
    "data/rftoolsutility/advancement/recipes/misc/crafter2.json",
    "data/rftoolsutility/advancement/recipes/misc/crafter3.json",
    "data/rftoolsutility/recipe/crafter2.json",
    "data/rftoolsutility/recipe/crafter3.json",
    "data/rftoolsutility/loot_table/blocks/crafter2.json",
    "data/rftoolsutility/loot_table/blocks/crafter3.json",
    # Slice 4: the spawner (its front texture, item model, recipe, advancement and loot table;
    # its block model is re-pointed below, like the crafter's).
    "assets/rftoolsutility/models/item/spawner.json",
    "assets/rftoolsutility/textures/block/machinespawner.png",
    "data/rftoolsutility/advancement/recipes/misc/spawner.json",
    "data/rftoolsutility/recipe/spawner.json",
    "data/rftoolsutility/loot_table/blocks/spawner.json",
    # Slice 8: the matter beamer. Its blockstate keys the jar's two models on `lit`, its models
    # are the jar's own `cube_all` cubes, and its item model points at the lit cube.
    "assets/rftoolsutility/blockstates/matter_beamer.json",
    "assets/rftoolsutility/models/block/matter_beamer_off.json",
    "assets/rftoolsutility/models/block/matter_beamer_on.json",
    "assets/rftoolsutility/models/item/matter_beamer.json",
    "assets/rftoolsutility/textures/block/machinebeamer.png",
    "assets/rftoolsutility/textures/block/machinebeameroff.png",
    "data/rftoolsutility/advancement/recipes/misc/matter_beamer.json",
    "data/rftoolsutility/recipe/matter_beamer.json",
    "data/rftoolsutility/loot_table/blocks/matter_beamer.json",
    # Slice 9: the teleporter family — matter transmitter, receiver and booster. Their
    # blockstates/models are the jar's own; the transmitter/receiver block models name
    # `rftoolsbase:` base textures, re-pointed below like the crafter's.
    "assets/rftoolsutility/blockstates/matter_transmitter.json",
    "assets/rftoolsutility/blockstates/matter_receiver.json",
    "assets/rftoolsutility/blockstates/matter_booster.json",
    "assets/rftoolsutility/models/block/matter_transmitter.json",
    "assets/rftoolsutility/models/block/matter_receiver.json",
    "assets/rftoolsutility/models/block/matter_booster.json",
    "assets/rftoolsutility/models/item/matter_transmitter.json",
    "assets/rftoolsutility/models/item/matter_receiver.json",
    "assets/rftoolsutility/models/item/matter_booster.json",
    "assets/rftoolsutility/textures/block/machinetransmitter.png",
    "assets/rftoolsutility/textures/block/machinereceiver.png",
    "assets/rftoolsutility/textures/block/machinematterbooster.png",
    "data/rftoolsutility/advancement/recipes/misc/matter_transmitter.json",
    "data/rftoolsutility/advancement/recipes/misc/matter_receiver.json",
    "data/rftoolsutility/advancement/recipes/misc/matter_booster.json",
    "data/rftoolsutility/recipe/matter_transmitter.json",
    "data/rftoolsutility/recipe/matter_receiver.json",
    "data/rftoolsutility/recipe/matter_booster.json",
    "data/rftoolsutility/loot_table/blocks/matter_transmitter.json",
    "data/rftoolsutility/loot_table/blocks/matter_receiver.json",
    "data/rftoolsutility/loot_table/blocks/matter_booster.json",
    # Slice 10: the destination analyzer — a passive block. Its blockstate's six `facing`
    # rotations are cosmetic (the effect is adjacency), so the iron-block template drops them
    # as the booster's does. The recipe's `c:ender_pearls` tag and 1.21.1 item objects are
    # upgraded to 26.3's ingredient form below.
    "assets/rftoolsutility/blockstates/destination_analyzer.json",
    "assets/rftoolsutility/models/block/destination_analyzer.json",
    "assets/rftoolsutility/models/item/destination_analyzer.json",
    "assets/rftoolsutility/textures/block/machinedestinationanalyzer.png",
    "data/rftoolsutility/advancement/recipes/misc/destination_analyzer.json",
    "data/rftoolsutility/recipe/destination_analyzer.json",
    "data/rftoolsutility/loot_table/blocks/destination_analyzer.json",
    # Slice 11: the sensor. Its four models and its face texture are the jar's own; the jar's
    # 24 `logic_facing` slab rotations cannot be a content block's states, so the blockstate the
    # pin reads is generated below with the `signal` property the native code caches, exactly as
    # the tank's nine-level blockstate is generated.
    "assets/rftoolsutility/models/block/sensor_0.json",
    "assets/rftoolsutility/models/block/sensor_1.json",
    "assets/rftoolsutility/models/block/sensor_2.json",
    "assets/rftoolsutility/models/block/sensor_3.json",
    "assets/rftoolsutility/models/item/sensor.json",
    "assets/rftoolsutility/textures/block/logic/machinesensor.png",
    "data/rftoolsutility/advancement/recipes/misc/sensor.json",
    "data/rftoolsutility/recipe/sensor.json",
    "data/rftoolsutility/loot_table/blocks/sensor.json",
]
# Slice 8: the jar's `living/*` item tags, byte for byte. A spawner recipe's empty `object` is
# upstream's "living?" form, and these tags are the yield a living item carries
# (`SpawnerRecipes.MobSpawnAmount.match`); the beamer reads them through the game's tag table.
CARRIED_TAGS = [
    "data/rftoolsutility/tags/item/living/living.json",
    "data/rftoolsutility/tags/item/living/highyield.json",
    "data/rftoolsutility/tags/item/living/averageyield.json",
    "data/rftoolsutility/tags/item/living/lowyield.json",
]
# Slice 4: the jar's mob recipes (`SpawnerModule`'s datagen), carried **byte for byte**. Their
# `type` is `rftoolsutility:spawner`, a type the port's own crafting kind reads; the 1.21.1
# `item1..3.object` forms are upstream's matter *ingredients* and are never parsed as
# ingredients here (only the `amount` of each is read), so 26.3's ingredient form does not apply
# and rewriting them would change the file the licence evidence points at.
MOB_RECIPE_PREFIX = "data/rftoolsutility/recipe/minecraft_"
# The tank's own block models name RFToolsBase's machine base textures; the port
# ships them under its own namespace so the one `.hnmod` needs no other content.
BASE_TEXTURES = [
    "assets/rftoolsbase/textures/block/base/machinebottom.png",
    "assets/rftoolsbase/textures/block/base/machineside.png",
    "assets/rftoolsbase/textures/block/base/machinetop.png",
]
# The jar's inventory model, re-pointed at this namespace's copy of the base
# textures (it names `rftoolsbase:block/base/…`).
INVENTORY_MODEL = "assets/rftoolsutility/models/block/tank_inventory.json"
# The crafter's own cube models, re-pointed the same way (one per tier).
CRAFTER_MODELS = [
    "assets/rftoolsutility/models/block/crafter1.json",
    "assets/rftoolsutility/models/block/crafter2.json",
    "assets/rftoolsutility/models/block/crafter3.json",
]
# The spawner's own cube model, re-pointed the same way.
SPAWNER_MODEL = "assets/rftoolsutility/models/block/spawner.json"
# Slice 9: the transmitter's and receiver's cube models name `rftoolsbase:` base textures.
TELEPORTER_MODELS = [
    "assets/rftoolsutility/models/block/matter_transmitter.json",
    "assets/rftoolsutility/models/block/matter_receiver.json",
    "assets/rftoolsutility/models/block/matter_booster.json",
    # Slice 10: the analyzer's cube names `rftoolsbase:` machine base textures.
    "assets/rftoolsutility/models/block/destination_analyzer.json",
    # Slice 11: the four sensor cube models name `rftoolsbase:` machine base textures.
    "assets/rftoolsutility/models/block/sensor_0.json",
    "assets/rftoolsutility/models/block/sensor_1.json",
    "assets/rftoolsutility/models/block/sensor_2.json",
    "assets/rftoolsutility/models/block/sensor_3.json",
]
# The teleporter family's three blocks (McJtyLib `STANDARD_IRON`; the booster is
# `.strength(2, 6)`). The transmitter and receiver carry a block entity; the booster is a plain
# facing marker (its six-way `facing` is cosmetic — the effect is adjacency — so the full-cube
# iron-block template drops the rotation).
TELEPORTER_BLOCK = {
    "template": "minecraft:iron_block",
    "properties": {
        "hardness": 2.0,
        "explosion_resistance": 2.0,
        "requires_tool": True,
        "sounds": "minecraft:iron_block",
        "occludes": True,
    },
    "item": True,
}

# `TankTE.createBlock` on McJtyLib's `BlockBuilder.STANDARD_IRON`: strength 2.0,
# correct tool for drops, metal sound. `minecraft:iron_block` is a full cube
# with no state of its own; the tank adds the fill level the jar's model loader
# read from the block entity. Since API 129 a mod block entity needs no
# block-entity template.
BLOCK = {
    "template": "minecraft:iron_block",
    "properties": {
        "hardness": 2.0,
        "explosion_resistance": 2.0,
        "requires_tool": True,
        "sounds": "minecraft:iron_block",
        "occludes": True,
    },
    "state_properties": {
        "level": {"type": "enum", "values": [str(level) for level in range(9)]}
    },
    "default_state": {"level": "0"},
}

# Upstream's own item group (`itemGroup.rftoolsutility`), this slice's items.
TAB = {
    "title": "itemGroup.rftoolsutility",
    "icon": f"{MOD}:tank",
    "items": [
        f"{MOD}:tank",
        f"{MOD}:crafter1",
        f"{MOD}:crafter2",
        f"{MOD}:crafter3",
        f"{MOD}:spawner",
        f"{MOD}:matter_beamer",
        f"{MOD}:matter_transmitter",
        f"{MOD}:matter_receiver",
        f"{MOD}:matter_booster",
        f"{MOD}:destination_analyzer",
        f"{MOD}:sensor",
    ],
}

# The crafter's three tiers (`CrafterModule.CRAFTER1..3`): the same block, container and block
# entity with 2, 4 or 8 recipe slots. The jar's lang lines say the count ("can handle up to
# 2/4/8 recipes at once"), which is where `runtime::CrafterTier::recipes` reads it from.
CRAFTER_TIERS = ["crafter1", "crafter2", "crafter3"]

# `CrafterBlock` on McJtyLib's `BlockBuilder.STANDARD_IRON` with `ironPickaxeTags()`: strength
# 2.0, correct tool for drops, metal sound. Upstream orients the block in six directions
# (`orientedBlock`); the content schema takes a block's states and placement rule from its
# vanilla template, and the full-cube, block-entity-free, no-menu template whose horizontal
# `facing` placement is measured on a content block is `minecraft:white_glazed_terracotta`
# (`bibliocraft`'s four blocks), so the crafter faces the four horizontal directions. The
# jar's own blockstate keeps its rotations for those four.
CRAFTER_BLOCK = {
    "template": "minecraft:white_glazed_terracotta",
    "properties": {
        "hardness": 2.0,
        "explosion_resistance": 2.0,
        "requires_tool": True,
        "sounds": "minecraft:iron_block",
        "occludes": True,
    },
    "item": True,
}

# The jar's blockstate rotations for the four horizontal facings (its `orientedBlock`): the
# front is the model's north face.
CRAFTER_FACINGS = {
    "north": {},
    "east": {"y": 90},
    "south": {"y": 180},
    "west": {"y": 270},
}

# `SpawnerTileEntity.createBlock` on McJtyLib's `BlockBuilder.STANDARD_IRON`: the tank's and the
# crafter's own properties. Upstream orients the block in six directions
# (`orientedBlock`), and the content schema takes a block's states and placement rule from its
# vanilla template, so the spawner borrows the crafter's measured full-cube, no-block-entity,
# no-menu template with a four-way horizontal `facing`.
SPAWNER_BLOCK = {
    "template": "minecraft:white_glazed_terracotta",
    "properties": {
        "hardness": 2.0,
        "explosion_resistance": 2.0,
        "requires_tool": True,
        "sounds": "minecraft:iron_block",
        "occludes": True,
    },
    "item": True,
}

# The jar's own blockstate rotations (`spawner.json`), kept for the four horizontal facings.
SPAWNER_FACINGS = CRAFTER_FACINGS

# `MatterBeamerBlock` on McJtyLib's `BlockBuilder.STANDARD_IRON` with `.infusable()`: the same
# properties every machine carries, no rotation (`RotationType.NONE`) and the one state upstream
# adds, `BlockStateProperties.LIT` — the jar's blockstate keys its two models on it. The template
# is the tank's full cube; a mod block entity needs no block-entity template since API 129.
BEAMER_BLOCK = {
    "template": "minecraft:iron_block",
    "properties": {
        "hardness": 2.0,
        "explosion_resistance": 2.0,
        "requires_tool": True,
        "sounds": "minecraft:iron_block",
        "occludes": True,
    },
    "state_properties": {
        "lit": {"type": "enum", "values": ["false", "true"]}
    },
    "default_state": {"lit": "false"},
    "item": True,
}

# `SensorTileEntity.createBlock` on McJtyLib's `BlockBuilder.STANDARD_IRON`. The jar's slab has 24
# `logic_facing` states, which the content schema cannot take from a vanilla template; like the
# analyzer's cosmetic facing, the input side is a saved setting written by the harness command.
# The one state the port needs is `signal`, the cache the redstone hook reads (`BlockPart::
# block_signal` asks very often and must not read the world).
SENSOR_BLOCK = {
    "template": "minecraft:iron_block",
    "properties": {
        "hardness": 2.0,
        "explosion_resistance": 2.0,
        "requires_tool": True,
        "sounds": "minecraft:iron_block",
        "occludes": True,
    },
    "state_properties": {
        "signal": {"type": "enum", "values": ["0", "1"]}
    },
    "default_state": {"signal": "0"},
    "item": True,
}

# The jar's recipes name `c:` convention tags the prepared 26.3 root does not
# ship (it ships `minecraft` tags only), and an unknown tag is an empty
# ingredient. These are the smallest honest memberships, the same ones this
# campaign's `rftoolsbase` port declares for the same two tags.
CONVENTION_TAGS = {
    "ingots/iron": ["minecraft:iron_ingot"],
    "glass_blocks": ["minecraft:glass", "minecraft:tinted_glass"],
    # Slice 4, the spawner's own recipe (`data/rftoolsutility/recipe/spawner.json`).
    "bones": ["minecraft:bone"],
    "rods/blaze": ["minecraft:blaze_rod"],
    "ender_pearls": ["minecraft:ender_pearl"],
    # Slice 11, the sensor's own recipe (`data/rftoolsutility/recipe/sensor.json`).
    "gems/quartz": ["minecraft:quartz"],
}

# The jar's lang lines for the tank, the crafter tiers, the spawner and the matter beamer,
# verbatim.
LANG_KEYS = [
    "itemGroup.rftoolsutility",
    "block.rftoolsutility.tank",
    "block.rftoolsutility.crafter1",
    "block.rftoolsutility.crafter2",
    "block.rftoolsutility.crafter3",
    "block.rftoolsutility.spawner",
    "block.rftoolsutility.matter_beamer",
    "block.rftoolsutility.matter_transmitter",
    "block.rftoolsutility.matter_receiver",
    "block.rftoolsutility.matter_booster",
    "block.rftoolsutility.destination_analyzer",
    "block.rftoolsutility.sensor",
    "message.rftoolsutility.sensor.header",
    "message.rftoolsutility.shiftmessage",
    "message.rftoolsutility.tank.header",
    "message.rftoolsutility.tank.contents",
    "message.rftoolsutility.crafter1.header",
    "message.rftoolsutility.crafter2.header",
    "message.rftoolsutility.crafter3.header",
    "message.rftoolsutility.spawner.header",
    "message.rftoolsutility.spawner.gold",
    "message.rftoolsutility.matter_beamer.header",
    "message.rftoolsutility.matter_beamer.gold",
    "message.rftoolsutility.matter_transmitter.header",
    "message.rftoolsutility.matter_transmitter.gold",
    "message.rftoolsutility.matter_receiver.header",
    "message.rftoolsutility.matter_receiver.gold",
]



def upgrade_recipe(recipe: dict) -> dict:
    """The carry pipeline's 1.21.1 → 26.3 ingredient upgrade."""
    for key, value in list(recipe.get("key", {}).items()):
        if isinstance(value, dict) and "tag" in value:
            recipe["key"][key] = f"#{value['tag']}"
        elif isinstance(value, dict) and "item" in value:
            recipe["key"][key] = value["item"]
    return recipe


def unwrap_recipe(recipe: dict) -> dict:
    """The recipe a carried one becomes.

    The jar's `crafter2`/`crafter3` recipes are `mcjtylib:copy_components`: a wrapper whose inner
    `recipe` is the plain `minecraft:crafting_shaped` one and whose component copy (the tier-1
    machine's taught recipes, buffer and energy travelling onto the upgraded machine) needs
    mcjtylib's recipe type, which this port does not register. The wrapper is unwrapped to the
    inner recipe — the same grid, the same result, the same unlock — so a crafted tier-2/3
    machine starts empty (recorded as a deviation in `notes.md`).
    """
    if recipe.get("type") == "mcjtylib:copy_components":
        return recipe["recipe"]
    return recipe


def upgrade_loot(loot: dict) -> dict:
    """The 26.3 loot table: one pool, and no `condition` key at all.

    The jar's own table carries no conditions, and 26.3 reads a pool's condition
    as singular (`pool["condition"]`) and drops the pool unconditionally when it
    is absent — the same behaviour as the 1.21.1 file. The `copy_components`
    function names mcjtylib's `item_fluids`, a component this port does not
    register (the tank's contents on the item is a recorded gap), so it is
    dropped; `copy_name` stays.
    """
    for pool in loot.get("pools", []):
        pool.pop("conditions", None)
        for entry in pool.get("entries", []):
            functions = [
                function
                for function in entry.get("functions", [])
                if function.get("function") != "minecraft:copy_components"
            ]
            if functions:
                entry["functions"] = functions
            else:
                entry.pop("functions", None)
    return loot


def block_model(level: int) -> dict:
    """One of the nine cubes: `TankBakedModel` draws `tank<level>` on the four
    sides and the machine base top and bottom."""
    side = f"{MOD}:block/tank{level}"
    return {
        "parent": "block/cube",
        "textures": {
            "particle": side,
            "down": f"{MOD}:block/base/machinebottom",
            "up": f"{MOD}:block/base/machinetop",
            "north": side,
            "south": side,
            "east": side,
            "west": side,
        },
    }


def dump(path: Path, value: dict | list) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def main() -> None:
    data = JAR.read_bytes()
    assert hashlib.sha256(data).hexdigest() == JAR_SHA256, "the jar is not the pack's file"

    assets = CONTENT / "assets" / MOD
    data_dir = CONTENT / "data" / MOD
    for folder in (assets, data_dir, CONTENT / "data" / "c", CONTENT / "data" / "minecraft"):
        shutil.rmtree(folder, ignore_errors=True)

    with zipfile.ZipFile(JAR) as jar:
        lang = json.loads(jar.read(f"assets/{MOD}/lang/en_us.json"))
        for name in CARRIED_ASSETS:
            target = CONTENT / name
            target.parent.mkdir(parents=True, exist_ok=True)
            raw = jar.read(name)
            if "/recipe/" in name and name.endswith(".json"):
                raw = (
                    json.dumps(
                        upgrade_recipe(unwrap_recipe(json.loads(raw))),
                        indent=2,
                        sort_keys=True,
                    )
                    + "\n"
                ).encode()
            elif "loot_table/blocks/" in name:
                raw = (json.dumps(upgrade_loot(json.loads(raw)), indent=2, sort_keys=True) + "\n").encode()
            target.write_bytes(raw)
        inventory = json.loads(jar.read(INVENTORY_MODEL))
        crafter_models = [(name, json.loads(jar.read(name))) for name in CRAFTER_MODELS]
        spawner_model = json.loads(jar.read(SPAWNER_MODEL))
        teleporter_models = [(name, json.loads(jar.read(name))) for name in TELEPORTER_MODELS]
        # Slice 4: the 70 mob recipes, byte for byte (see `MOB_RECIPE_PREFIX`).
        mob_recipes = [
            name for name in jar.namelist() if name.startswith(MOB_RECIPE_PREFIX) and name.endswith(".json")
        ]
        for name in sorted(mob_recipes):
            target = CONTENT / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(jar.read(name))
        # Slice 8: the `living/*` tags, byte for byte.
        for name in CARRIED_TAGS:
            target = CONTENT / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(jar.read(name))
    inventory["textures"] = {
        key: value.replace("rftoolsbase:", f"{MOD}:")
        for key, value in inventory["textures"].items()
    }
    dump(assets / "models" / "block" / "tank_inventory.json", inventory)
    for name, model in crafter_models:
        model["textures"] = {
            key: value.replace("rftoolsbase:", f"{MOD}:")
            for key, value in model["textures"].items()
        }
        dump(CONTENT / name, model)
    spawner_model["textures"] = {
        key: value.replace("rftoolsbase:", f"{MOD}:")
        for key, value in spawner_model["textures"].items()
    }
    dump(assets / "models" / "block" / "spawner.json", spawner_model)
    for name, model in teleporter_models:
        model["textures"] = {
            key: value.replace("rftoolsbase:", f"{MOD}:")
            for key, value in model["textures"].items()
        }
        dump(CONTENT / name, model)

    with zipfile.ZipFile(BASE_JAR) as base:
        base_data = BASE_JAR.read_bytes()
        assert hashlib.sha256(base_data).hexdigest() == BASE_JAR_SHA256, "the base jar is not the pack's file"
        for name in BASE_TEXTURES:
            target = assets / "textures" / "block" / "base" / Path(name).name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(base.read(name))

    for level in range(9):
        dump(assets / "models" / "block" / f"tank{level}.json", block_model(level))
    dump(
        assets / "blockstates" / "tank.json",
        {"variants": {f"level={level}": {"model": f"{MOD}:block/tank{level}"} for level in range(9)}},
    )
    dump(
        assets / "items" / "tank.json",
        {"model": {"type": "minecraft:model", "model": f"{MOD}:item/tank"}},
    )
    for tier in CRAFTER_TIERS:
        dump(
            assets / "items" / f"{tier}.json",
            {"model": {"type": "minecraft:model", "model": f"{MOD}:item/{tier}"}},
        )
        dump(
            assets / "blockstates" / f"{tier}.json",
            {
                "variants": {
                    f"facing={facing}": {"model": f"{MOD}:block/{tier}", **rotation}
                    for facing, rotation in CRAFTER_FACINGS.items()
                }
            },
        )
    dump(
        assets / "items" / "spawner.json",
        {"model": {"type": "minecraft:model", "model": f"{MOD}:item/spawner"}},
    )
    dump(
        assets / "blockstates" / "spawner.json",
        {
            "variants": {
                f"facing={facing}": {"model": f"{MOD}:block/spawner", **rotation}
                for facing, rotation in SPAWNER_FACINGS.items()
            }
        },
    )
    # Slice 8: the beamer's item definition (its blockstate and models are the jar's own).
    dump(
        assets / "items" / "matter_beamer.json",
        {"model": {"type": "minecraft:model", "model": f"{MOD}:item/matter_beamer"}},
    )
    # Slice 9/10: the teleporter family's item definitions (their blockstates/models are the jar's).
    for name in ("matter_transmitter", "matter_receiver", "matter_booster", "destination_analyzer"):
        dump(
            assets / "items" / f"{name}.json",
            {"model": {"type": "minecraft:model", "model": f"{MOD}:item/{name}"}},
        )
    # Slice 11: the sensor's jar blockstate is all 24 `logic_facing` slab rotations, which this
    # content block cannot express; the pin reads the generated `signal` variants instead. The
    # four jar models are carried (the front-facing one shows on both strengths).
    dump(
        assets / "items" / "sensor.json",
        {"model": {"type": "minecraft:model", "model": f"{MOD}:item/sensor"}},
    )
    dump(
        assets / "blockstates" / "sensor.json",
        {"variants": {f"signal={value}": {"model": f"{MOD}:block/sensor_0"} for value in ("0", "1")}},
    )
    dump(assets / "lang" / "en_us.json", {key: lang[key] for key in LANG_KEYS})

    dump(data_dir / "horizon" / "block" / "tank.json", BLOCK)
    for tier in CRAFTER_TIERS:
        dump(data_dir / "horizon" / "block" / f"{tier}.json", CRAFTER_BLOCK)
    dump(data_dir / "horizon" / "block" / "spawner.json", SPAWNER_BLOCK)
    dump(data_dir / "horizon" / "block" / "matter_beamer.json", BEAMER_BLOCK)
    for name in ("matter_transmitter", "matter_receiver", "matter_booster", "destination_analyzer"):
        dump(data_dir / "horizon" / "block" / f"{name}.json", TELEPORTER_BLOCK)
    dump(data_dir / "horizon" / "block" / "sensor.json", SENSOR_BLOCK)
    dump(data_dir / "horizon" / "creative_tab" / f"{MOD}.json", TAB)
    for tag, values in CONVENTION_TAGS.items():
        dump(CONTENT / "data" / "c" / "tags" / "item" / f"{tag}.json", {"values": values})
    dump(
        CONTENT / "data" / "minecraft" / "tags" / "block" / "mineable" / "pickaxe.json",
        {
            "values": [
                f"{MOD}:tank",
                *[f"{MOD}:{tier}" for tier in CRAFTER_TIERS],
                f"{MOD}:spawner",
                f"{MOD}:matter_beamer",
                f"{MOD}:matter_transmitter",
                f"{MOD}:matter_receiver",
                f"{MOD}:matter_booster",
                f"{MOD}:destination_analyzer",
                f"{MOD}:sensor",
            ]
        },
    )
    dump(
        CONTENT / "data" / "minecraft" / "tags" / "block" / "needs_iron_tool.json",
        {
            "values": [
                f"{MOD}:tank",
                *[f"{MOD}:{tier}" for tier in CRAFTER_TIERS],
                f"{MOD}:spawner",
                f"{MOD}:matter_beamer",
                f"{MOD}:matter_transmitter",
                f"{MOD}:matter_receiver",
                f"{MOD}:matter_booster",
                f"{MOD}:destination_analyzer",
                f"{MOD}:sensor",
            ]
        },
    )

    print(
        f"assembled assets/ and data/ under {CONTENT} from {JAR.name} "
        f"({len(mob_recipes)} mob recipes)"
    )


if __name__ == "__main__":
    main()
