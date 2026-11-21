#!/usr/bin/env python3
"""Parse the `/data get entity <dropped item> Item` reply for the tank's contents.

The engine answers vanilla's line:

    Entity <uuid> has the following entity data: {count:1,id:"rftoolsutility:tank",
    components:{"minecraft:block_entity_data":{id:"rftoolsutility:tank",
    fluids:[{Amount:5000,Fluid:"minecraft:water"}]}}}

This prints the shell assignments the e2e evaluates, and exits 1 when the reply is
not what was asked for:

    ok=1 component=present carried=5000 carried_fluid=minecraft:water

Usage: parse_drop.py "<reply line>" none|<millibuckets>
"""

import re
import sys

COMPONENT = "minecraft:block_entity_data"
TYPE = "rftoolsutility:tank"
WATER = "minecraft:water"


def main(argv):
    if len(argv) != 3:
        print("usage: parse_drop.py '<data get reply>' none|<mB>", file=sys.stderr)
        return 2
    line, expect = argv[1], argv[2].strip()
    present = COMPONENT in line
    carried = None
    fluid = None
    if present:
        after = line.split(COMPONENT, 1)[1]
        amount = re.search(r"Amount:\s*(-?\d+)", after)
        carried = int(amount.group(1)) if amount else None
        name = re.search(r'Fluid:\s*"([^"]+)"', after)
        fluid = name.group(1) if name else None
    if expect == "none":
        ok = not present
    else:
        try:
            wanted = int(expect)
        except ValueError:
            print(f"bad expectation {expect!r}", file=sys.stderr)
            return 2
        ok = present and carried == wanted and fluid == WATER and TYPE in line
    print(f"ok={1 if ok else 0}")
    print(f"component={'present' if present else 'absent'}")
    print(f"carried={carried if carried is not None else 'none'}")
    print(f"carried_fluid={fluid if fluid else 'none'}")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
