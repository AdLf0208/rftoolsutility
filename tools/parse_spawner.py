#!/usr/bin/env python3
"""Parse the `/rftoolsutility spawner <pos> [nonce]` line into shell variables.

The line is the harness's contract, printed by `spawner_readout` in `src/lib.rs`:

    Spawner at [200, 101, 3]: energy=0/200000 mob=minecraft:zombie cost=800
    needs=0.1,0.2,20 matter=0.5,0.5,0.5 spawned=0 nonce=2

Fields: `energy=<held>/<capacity>`, `mob=<id>|none`, `cost=<FE>|-` (the recipe's `power`),
`needs=<a>,<b>,<c>|-` (the recipe's `item1..3` amounts), `matter=<a>,<b>,<c>` (the three
stores, `x`/`y`/`z` formatted as for `needs` but never absent) and `spawned=<count>`.
Prints `S_ENERGY=… S_CAP=… S_MOB=… S_COST=… S_NEEDS=… S_MATTER=… S_SPAWNED=… S_NONCE=…
ok=1` for a line that parses (`ok=0` otherwise), which the scenario evals.
"""

from __future__ import annotations

import re
import shlex
import sys

PATTERNS = {
    "S_ENERGY": r"energy=(\d+)/(\d+)",
    "S_MOB": r"mob=(\S+)",
    "S_COST": r"cost=(\S+)",
    "S_NEEDS": r"needs=(\S+)",
    "S_MATTER": r"matter=(\S+)",
    "S_SPAWNED": r"spawned=(\d+)",
    "S_NONCE": r"nonce=(\S+)",
}


def main() -> int:
    if len(sys.argv) < 2:
        print("ok=0")
        return 2
    line = sys.argv[1]
    fields: dict[str, str] = {}
    match = re.search(PATTERNS["S_ENERGY"], line)
    if not match:
        print("ok=0")
        return 1
    fields["S_ENERGY"] = match.group(1)
    fields["S_CAP"] = match.group(2)
    for name in ("S_MOB", "S_COST", "S_NEEDS", "S_MATTER", "S_SPAWNED", "S_NONCE"):
        found = re.search(PATTERNS[name], line)
        if not found:
            print("ok=0")
            return 1
        fields[name] = found.group(1)
    print(" ".join(f"{name}={shlex.quote(value)}" for name, value in fields.items()) + " ok=1")
    return 0


if __name__ == "__main__":
    sys.exit(main())
