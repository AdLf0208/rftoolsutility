#!/usr/bin/env python3
"""Parse the `/rftoolsutility beamer <pos> [nonce]` line into shell variables.

The line is the harness's contract, printed by `beamer_readout` in `src/lib.rs`:

    Beamer at [200, 101, -12]: energy=4000/200000 dest=[200, 101, -16]
    slot=minecraft:rotten_fleshx8 sent=2 lit=true nonce=9

Fields: `energy=<held>/<capacity>`, `dest=[x, y, z]|none` (the linked spawner),
`slot=<item>x<count>|-` (the material slot) and `sent=<count>` (the sends the machine has
made), plus the block's own `lit` state. `B_DEST` is printed without its spaces, so the
scenario compares it as `[200,101,-16]`.
Prints `B_ENERGY=… B_CAP=… B_DEST=… B_SLOT=… B_SENT=… B_LIT=… B_NONCE=… ok=1` for a line
that parses (`ok=0` otherwise), which the scenario evals.
"""

from __future__ import annotations

import re
import shlex
import sys

PATTERNS = {
    "B_ENERGY": r"energy=(\d+)/(\d+)",
    "B_DEST": r"dest=(\[[^\]]*\]|\S+)",
    "B_SLOT": r"slot=(\S+)",
    "B_SENT": r"sent=(\d+)",
    "B_LIT": r"lit=(\w+)",
    "B_NONCE": r"nonce=(\S+)",
}


def main() -> int:
    if len(sys.argv) < 2:
        print("ok=0")
        return 2
    line = sys.argv[1]
    fields: dict[str, str] = {}
    match = re.search(PATTERNS["B_ENERGY"], line)
    if not match:
        print("ok=0")
        return 1
    fields["B_ENERGY"] = match.group(1)
    fields["B_CAP"] = match.group(2)
    for name in ("B_DEST", "B_SLOT", "B_SENT", "B_LIT", "B_NONCE"):
        found = re.search(PATTERNS[name], line)
        if not found:
            print("ok=0")
            return 1
        fields[name] = found.group(1)
    # A block position reads `[200, 101, -16]`; the space-free form is what a shell compares.
    fields["B_DEST"] = fields["B_DEST"].replace(" ", "")
    print(" ".join(f"{name}={shlex.quote(value)}" for name, value in fields.items()) + " ok=1")
    return 0


if __name__ == "__main__":
    sys.exit(main())
