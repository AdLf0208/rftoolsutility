#!/usr/bin/env python3
"""Parse the `/rftoolsutility crafter <pos> [nonce]` line into shell variables.

The line is the harness's contract, printed by `crafter_readout` in `src/lib.rs`:

    Crafter at [200, 101, 0]: energy=5000/50000 speed=Slow mode=Ext keep=All
    recipes=minecraft:oak_planks in=10=minecraft:oak_logx3 out=36=minecraft:oak_planksx12 nonce=1

Fields: `energy=<held>/<capacity>`, `speed=Slow|Fast`, `mode=Ext|Int`, `keep=All|Keep`,
`recipes=<id>[+<id>…]` or `-`, `in=`/`out=` a `;`-separated list of `<slot>=<item>x<count>`
or `-`. Prints `C_ENERGY=… C_CAP=… C_SPEED=… C_MODE=… C_KEEP=… C_RECIPES=… C_IN=… C_OUT=…
C_NONCE=… ok=1` for a line that parses (`ok=0` otherwise), which the scenario evals.
"""

from __future__ import annotations

import re
import shlex
import sys

PATTERNS = {
    "C_ENERGY": r"energy=(\d+)/(\d+)",
    "C_SPEED": r"speed=(\w+)",
    "C_MODE": r"mode=(\w+)",
    "C_KEEP": r"keep=(\w+)",
    "C_RECIPES": r"recipes=(\S+)",
    "C_IN": r"in=(\S+)",
    "C_OUT": r"out=(\S+)",
    "C_NONCE": r"nonce=(\S+)",
}


def main() -> int:
    if len(sys.argv) < 2:
        print("ok=0")
        return 2
    line = sys.argv[1]
    fields: dict[str, str] = {}
    match = re.search(PATTERNS["C_ENERGY"], line)
    if not match:
        print("ok=0")
        return 1
    fields["C_ENERGY"] = match.group(1)
    fields["C_CAP"] = match.group(2)
    for name in ("C_SPEED", "C_MODE", "C_KEEP", "C_RECIPES", "C_IN", "C_OUT", "C_NONCE"):
        found = re.search(PATTERNS[name], line)
        if not found:
            print("ok=0")
            return 1
        fields[name] = found.group(1)
    print(" ".join(f"{name}={shlex.quote(value)}" for name, value in fields.items()) + " ok=1")
    return 0


if __name__ == "__main__":
    sys.exit(main())
