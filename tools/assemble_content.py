#!/usr/bin/env python3
"""Assemble the slice-1 `assets/` and `data/` folders from the pack's RFTools Utility jar.

The jar is the ATM10 file (`ports/_shared/licence-cache/rftoolsutility-1.21-7.0.13.jar`,
SHA-256 `615642b1…d3ff`), kept as the licence and provenance evidence. Nothing
is invented: the tank's nine fill textures, its inventory model, item model,
loot table, recipe, advancement and lang lines come out of that jar; the three
machine base textures come out of the *RFToolsBase* jar (the tank's own models
name `rftoolsbase:block/base/…`, and this port must stand alone), copied into
this namespace and re-pointed. Only spellings 26.3 needs are rewritten: a
recipe ingredient `{"item": X}` becomes `"X"` and `{"tag": T}` becomes `"#T"`,
the item model becomes a 26.3 item definition, and the block model is nine
per-level cubes because the pin has no custom model loader.

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
]
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
# The crafter's own cube model, re-pointed the same way.
CRAFTER_MODEL = "assets/rftoolsutility/models/block/crafter1.json"

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
    "items": [f"{MOD}:tank", f"{MOD}:crafter1"],
}

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

# The jar's recipes name `c:` convention tags the prepared 26.3 root does not
# ship (it ships `minecraft` tags only), and an unknown tag is an empty
# ingredient. These are the smallest honest memberships, the same ones this
# campaign's `rftoolsbase` port declares for the same two tags.
CONVENTION_TAGS = {
    "ingots/iron": ["minecraft:iron_ingot"],
    "glass_blocks": ["minecraft:glass", "minecraft:tinted_glass"],
}

# The jar's lang lines for the tank and the crafter, verbatim.
LANG_KEYS = [
    "itemGroup.rftoolsutility",
    "block.rftoolsutility.tank",
    "block.rftoolsutility.crafter1",
    "message.rftoolsutility.shiftmessage",
    "message.rftoolsutility.tank.header",
    "message.rftoolsutility.tank.contents",
]



def upgrade_recipe(recipe: dict) -> dict:
    """The carry pipeline's 1.21.1 → 26.3 ingredient upgrade."""
    for key, value in list(recipe.get("key", {}).items()):
        if isinstance(value, dict) and "tag" in value:
            recipe["key"][key] = f"#{value['tag']}"
        elif isinstance(value, dict) and "item" in value:
            recipe["key"][key] = value["item"]
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
                raw = (json.dumps(upgrade_recipe(json.loads(raw)), indent=2, sort_keys=True) + "\n").encode()
            elif "loot_table/blocks/" in name:
                raw = (json.dumps(upgrade_loot(json.loads(raw)), indent=2, sort_keys=True) + "\n").encode()
            target.write_bytes(raw)
        inventory = json.loads(jar.read(INVENTORY_MODEL))
        crafter_model = json.loads(jar.read(CRAFTER_MODEL))
    inventory["textures"] = {
        key: value.replace("rftoolsbase:", f"{MOD}:")
        for key, value in inventory["textures"].items()
    }
    dump(assets / "models" / "block" / "tank_inventory.json", inventory)
    crafter_model["textures"] = {
        key: value.replace("rftoolsbase:", f"{MOD}:")
        for key, value in crafter_model["textures"].items()
    }
    dump(assets / "models" / "block" / "crafter1.json", crafter_model)

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
    dump(
        assets / "items" / "crafter1.json",
        {"model": {"type": "minecraft:model", "model": f"{MOD}:item/crafter1"}},
    )
    dump(
        assets / "blockstates" / "crafter1.json",
        {
            "variants": {
                f"facing={facing}": {"model": f"{MOD}:block/crafter1", **rotation}
                for facing, rotation in CRAFTER_FACINGS.items()
            }
        },
    )
    dump(assets / "lang" / "en_us.json", {key: lang[key] for key in LANG_KEYS})

    dump(data_dir / "horizon" / "block" / "tank.json", BLOCK)
    dump(data_dir / "horizon" / "block" / "crafter1.json", CRAFTER_BLOCK)
    dump(data_dir / "horizon" / "creative_tab" / f"{MOD}.json", TAB)
    for tag, values in CONVENTION_TAGS.items():
        dump(CONTENT / "data" / "c" / "tags" / "item" / f"{tag}.json", {"values": values})
    dump(
        CONTENT / "data" / "minecraft" / "tags" / "block" / "mineable" / "pickaxe.json",
        {"values": [f"{MOD}:tank", f"{MOD}:crafter1"]},
    )
    dump(
        CONTENT / "data" / "minecraft" / "tags" / "block" / "needs_iron_tool.json",
        {"values": [f"{MOD}:tank", f"{MOD}:crafter1"]},
    )

    print(f"assembled assets/ and data/ under {CONTENT} from {JAR.name}")


if __name__ == "__main__":
    main()
