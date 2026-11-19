#!/usr/bin/env python3
"""Parse `/rftoolsutility info`'s line into shell assignments.

`Tank at [200, 101, 0]: water 1000/32000 mB (level 1), held minecraft:bucket x1,
t=264 nonce=1`

Prints `FLUID=… AMOUNT=… CAPACITY=… LEVEL=… HELD=… HELD_COUNT=… T=…` (or
`ok=0`), for the e2e to `eval`.
"""

from __future__ import annotations

import re
import sys

LINE = re.compile(
    r"^Tank at \[.*?\]: (?P<fluid>\S+) (?P<amount>-?\d+)/(?P<capacity>\d+) mB "
    r"\(level (?P<level>-?\d+)\), held (?P<held>.*?), t=(?P<t>\d+)(?: nonce=\d+)?$"
)
STACK = re.compile(r"(?P<item>\S+) x(?P<count>\d+)")


def main() -> None:
    line = sys.argv[1] if len(sys.argv) > 1 else ""
    match = LINE.match(line)
    if not match:
        print("ok=0")
        return
    print("ok=1")
    print(f"FLUID={match['fluid']}")
    print(f"AMOUNT={match['amount']}")
    print(f"CAPACITY={match['capacity']}")
    print(f"LEVEL={match['level']}")
    print(f"T={match['t']}")
    print(f"HELD={match['held']!r}")
    held = STACK.match(match["held"])
    print(f"HELD_ITEM={held['item'] if held else match['held']!r}")
    print(f"HELD_COUNT={held['count'] if held else 0}")


if __name__ == "__main__":
    main()
