#!/usr/bin/env bash
# In-game check for RFTools Utility: the tank fills and drains a whole bucket at
# a time, holds 32,000 mB of one fluid, shows its contents as the jar's own fill
# level, and its contents travel on the block's item when it is broken (slice 2).
#
# Every mod command is typed with `ctl chat "/cmd"`: the chat path reaches the
# command dispatcher the SDK's `register_commands`/`run_mod_command` register into
# (the retired bundle needed chat to reach `perform_command`).
#
# The End, in the void at x=200 (away from the central island and the ambient
# item entities of a generated level). The tank stands at x=202 on the stone
# floor; the cauldron of the differential at x=202, z=-8.
#
# GAME must be a game built with THIS package (flat SDK `.hmod`, mod API 0.2):
# `horizon-launcher native-build <port> --instance <i> --trust-native-code`, or
# the instance the launcher installed it into. `--dev-mod` is not used: the
# package carries its own assets/ and data/, and a dev-loaded folder would run
# the content without the native part (a tank with no block entity).
#
# Default run:
#  A  a tank placed from its item is empty: 0/32,000 mB, level 0
#  B  a water bucket fills exactly 1,000 mB, the hand becomes an empty bucket,
#     and the block's own state rises to level 1
#  C  an empty bucket drains exactly 1,000 mB back, the hand becomes a water
#     bucket, and the level falls to 0
#  D  at 31,500 mB one more bucket does not fit (500 mB of room), and the level
#     is 8
#  E  /rftoolsutility tank water 40,000 clamps to the 32,000 mB capacity
#  F  a lava bucket into a water tank does nothing (no fluid, no hand swap)
#  G  an empty bucket on an empty tank does nothing (no water bucket appears)
#  H  a vanilla cauldron takes the same water bucket (the harness's bucket click
#     works, so the tank's refusals above are the tank's)
#  I  the tank keeps its fluid across save + quit + relaunch, and the tick puts
#     the level back on the block
#  J  a broken tank's item carries the tank's contents: a survival break of a
#     tank holding 5,000 mB of water drops one tank whose Item tag has
#     `minecraft:block_entity_data` with that fluid and amount
#  K  that item, picked up and placed again, is a tank holding 5,000 mB of water
#     at level 2 (the contents came back)
#  L  an empty tank's item carries no block-entity data, and placing it gives an
#     empty tank (the same three assertions, inverted)
#  M  an item doctored to 99,999 mB places a tank clamped to the 32,000 mB
#     capacity (upstream's `FluidTank.setFluid` clamp), level 8
#  N  a crafter placed from its item is empty, knows no recipe, holds no FE and
#     has the template's horizontal facing state
#  O  a recipe is taught from the engine's own crafting table and an id the
#     table does not hold (or a slot past the machine's two) is refused
#  P  with no energy nothing is crafted
#  Q  5,000 FE and three oak logs become 12 oak planks for exactly 300 FE
#  R  an input the recipe does not take stays put and costs no energy
#  S  a full output buffer refuses the operation: no item consumed, no energy
#  T  chained crafting in `mode int`: four logs become four crafting tables
#     inside the machine, for 400 FE (no planks are ever inserted; upstream
#     charges per cycle, and a cycle runs each taught recipe once)
#  U  a broken crafter keeps its taught recipes, its buffer and its energy on its
#     own item, and placing it back restores them
#  V  the crafter's loot table is gated by `minecraft:needs_iron_tool`: a wooden
#     pickaxe breaks it and drops nothing, an iron pickaxe drops it (the control)
#
# SELFTEST=1 flips B's assertion (the tank must still read 0 mB), proving the
# scenario can fail: it prints FAIL and exits non-zero.
# SELFTEST=1 SELFTEST_AT=restore flips K's assertion instead (the restored tank
# must read 0 mB), which is slice 2's own fail-capable leg.
# SELFTEST=1 SELFTEST_AT=craft flips Q's assertion (the crafter must have made
# nothing and spent nothing), which is slice 3's own fail-capable leg.
#
# CONTROL=1 runs the same scenario on the Nightly WITHOUT this package
# (CONTROL_GAME, exported by launcher-game.sh): the tank item is not there, so A
# fails and the log says what it proved.
set -Eeuo pipefail
export LC_ALL=C
[ -z "${RFU_DEBUG:-}" ] || set -x

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/rftoolsutility
# Both binaries come from the campaign pin (0a875da): GAME from `launcher-game.sh build`,
# CTL from the same export.
CTL=${CTL:-}
GAME=${GAME:-}   # a launcher-built game carrying this package (see the header)
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/rftoolsutility-e2e.sock}
SELFTEST=${SELFTEST:-0}
SELFTEST_AT=${SELFTEST_AT:-fill}
CONTROL=${CONTROL:-0}
KEEP=${KEEP:-0}
WORLD=RFToolsUtilityE2E
X0=200
SX=$((X0 + 2))
EXPECT_CAPACITY=32000
BUCKET=1000
BLOCK=rftoolsutility:tank
WATER_BUCKET=minecraft:water_bucket
LAVA_BUCKET=minecraft:lava_bucket
EMPTY_BUCKET=minecraft:bucket
CAULDRON=minecraft:cauldron
CRAFTER=rftoolsutility:crafter1

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -n "$CTL" ] || { echo "set CTL= to the horizon-ctl of this pin (launcher-game.sh env)" >&2; exit 1; }
[ -n "$GAME" ] || { echo "set GAME= to a game built with this package: horizon-launcher native-build $PORT --instance <i> --trust-native-code" >&2; exit 1; }
if [ "$CONTROL" = 1 ]; then
    [ -n "${CONTROL_GAME:-}" ] || { echo "set CONTROL_GAME= to the unmodded Nightly game" >&2; exit 1; }
    GAME=$CONTROL_GAME
fi
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/rftoolsutility-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
say() { ctl chat "/$1" >/dev/null; }
fail() { echo "FAIL: $*" >&2; ctl logs --lines 60 >&2 || true; exit 1; }
on_error() { echo "FAIL: line $1: '$2' exited $?" >&2; ctl logs --lines 40 >&2 || true; }
trap 'on_error "$LINENO" "$BASH_COMMAND"' ERR

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    if [ "$KEEP" = 1 ]; then
        echo "kept scratch: $SCRATCH" >&2
    else
        rm -rf "$SCRATCH" 2>/dev/null || true
    fi
    echo "EXIT:$status"
}
trap cleanup EXIT

launch() { # launch [WORLD]
    local world=${1:-$WORLD}
    rm -f "$SOCK"
    cd "$SCRATCH"
    local args=(--game "$GAME" --world "$world" --seed 20261105 --creative --headless
                --wait --resources "$RES")
    "$CTL" --socket "$SOCK" launch "${args[@]}" >/dev/null
    ctl wait-ready 120
    ctl wait 40
}

# `ctl block` answers {pos, block, state, loaded}; the state is a string such as
# `rftoolsutility:tank[level=1]`.
block_name() { ctl block "$1" "$2" "$3" | jq -r '.block // empty'; }
block_level() { ctl block "$1" "$2" "$3" | jq -r '.state // empty' | sed -n 's/.*\[level=\([0-9]*\)\].*/\1/p'; }

arena() {
    ctl command "execute in minecraft:the_end run tp @s $X0.5 104 0.5" >/dev/null
    local loaded=false
    for _ in $(seq 1 120); do
        ctl wait 5 >/dev/null
        if [ "$(ctl block "$X0" 100 0 | jq -r '.loaded // false')" = true ]; then
            loaded=true
            break
        fi
    done
    [ "$loaded" = true ] || fail "the End arena chunk never loaded"
    ctl command "execute in minecraft:the_end run fill $((X0 - 2)) 100 -20 $((X0 + 4)) 100 20 minecraft:stone" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "execute in minecraft:the_end run fill $((X0 - 2)) 101 -20 $((X0 + 4)) 108 20 minecraft:air" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "execute in minecraft:the_end run kill @e[type=!player]" >/dev/null
    ctl command "execute in minecraft:the_end run tp @s $X0.5 101 0.5" >/dev/null
    ctl wait 20
    [ "$(block_name "$X0" 100 0)" = "minecraft:stone" ] || fail "the arena floor is not stone"
    [ "$(block_name "$X0" 101 0)" = "minecraft:air" ] || fail "the arena air above the floor is not air"
}

# `/rftoolsutility info X Y Z`: the line, then the parsed fields.
NONCE=0
info() { # info X Y Z
    NONCE=$((NONCE + 1))
    say "rftoolsutility info $1 $2 $3 $NONCE"
    ctl wait 12 >/dev/null
    INFO_LINE=$(ctl messages 20 | jq -r '.chat[]' | grep -m1 "nonce=$NONCE" || true)
    [ -n "$INFO_LINE" ] || fail "no fresh /rftoolsutility info reply for nonce $NONCE: $(ctl messages 8 | jq -c '.chat')"
    eval "$(python3 "$PORT/tools/parse_info.py" "$INFO_LINE")"
    [ "${ok:-0}" = 1 ] || fail "the info line did not parse: $INFO_LINE"
    echo "  $INFO_LINE" >&2
    echo "  parsed: fluid=$FLUID amount=$AMOUNT/$CAPACITY level=$LEVEL held=$HELD_ITEM x$HELD_COUNT" >&2
}

expect_amount() { [ "$AMOUNT" = "$1" ] || fail "the tank holds $AMOUNT mB, expected $1"; }
expect_level() { [ "$LEVEL" = "$1" ] || fail "the block's level is $LEVEL, expected $1"; }
expect_held() { [ "$HELD_ITEM" = "$1" ] || fail "the hand holds $HELD_ITEM, expected $1"; }
expect_block_level() { # expect_block_level X Y Z LEVEL
    local got
    got=$(block_level "$1" "$2" "$3")
    [ "$got" = "$4" ] || fail "the block state at $1,$2,$3 has level '$got', expected $4"
}

# Hold one item in the first hotbar slot and select it.
hold() { # hold ITEM [COUNT]
    ctl command "item replace entity @s hotbar.0 with $1 ${2:-1}" >/dev/null
    ctl wait 4 >/dev/null
    ctl select 1 >/dev/null
    ctl wait 2 >/dev/null
}

# Click ITEM from the hand down onto the floor under X Y Z. Retries the click twice: the
# client's selection and aim settle a tick or two after a teleport, and a `place` that lands
# before that is refused by the server (seen once, on the tank's doctored-item phase).
try_place() { # try_place ITEM X Y Z
    local item=$1 x=$2 y=$3 z=$4 attempt reply=""
    for attempt in 1 2 3; do
        ctl select "$item" >/dev/null 2>&1 || true
        ctl wait 3 >/dev/null
        ctl look-at-block "$x" $((y - 1)) "$z" up >/dev/null 2>&1 || true
        ctl wait 1 >/dev/null
        if reply=$(ctl place "$x" "$y" "$z" --item "$item" 2>&1); then
            return 0
        fi
        echo "  place attempt $attempt refused: $(tr -d '\n' <<<"$reply" | cut -c1-240)" >&2
        ctl wait 10 >/dev/null
    done
    fail "ctl place $item at $x,$y,$z was refused three times: $(ctl inventory | jq -c '{hotbar,main}')"
}

# Place ITEM from the player's hand at X Y Z (the player stands two blocks west).
place_block() { # place_block ITEM X Y Z
    local item=$1 x=$2 y=$3 z=$4
    ctl command "execute in minecraft:the_end run tp @s $((x - 2)).5 $y $z.5" >/dev/null
    ctl wait 6 >/dev/null
    hold "$item"
    try_place "$item" "$x" "$y" "$z"
    ctl wait 6 >/dev/null
    local name
    name=$(block_name "$x" "$y" "$z")
    [ "$name" = "$item" ] || fail "$item did not place at $x,$y,$z (got '$name')"
}

place_tank() { # place_tank X Y Z
    place_block "$BLOCK" "$1" "$2" "$3"
}

# Right-click the tank from two blocks away, holding ITEM.
click_with() { # click_with X Y Z ITEM
    local x=$1 y=$2 z=$3
    ctl command "execute in minecraft:the_end run tp @s $((x - 2)).5 101 $z.5" >/dev/null
    ctl wait 6 >/dev/null
    hold "$4"
    ctl interact "$x" "$y" "$z" >/dev/null
    ctl wait 12 >/dev/null
}

set_tank() { # set_tank X Y Z FLUID MB
    say "rftoolsutility tank $1 $2 $3 $4 $5"
    ctl wait 12 >/dev/null
    ctl messages 8 | jq -r '.chat[]' | grep -m1 'Tank at' >&2 || true
}

# A survival break: the drop path (block_drops) does not fire for a creative
# break, so the contents-on-the-item phases switch the player to survival and
# hold the tank's own tool.
break_block() { # break_block X Y Z TOOL
    local x=$1 y=$2 z=$3
    ctl command "execute in minecraft:the_end run tp @s $((x - 2)).5 101 $z.5" >/dev/null
    ctl wait 6 >/dev/null
    hold "$4"
    ctl break "$x" "$y" "$z" >/dev/null || fail "ctl break at $x,$y,$z failed"
    ctl wait 10 >/dev/null
    local got
    got=$(block_name "$x" "$y" "$z")
    [ "$got" = "minecraft:air" ] || fail "the block at $x,$y,$z is '$got' after the break, expected air"
}

# The chat line `/data get entity @s SelectedItem` replies for the tank in hand:
# its own tag, components included. (A selector over dropped items would also
# catch the arena's ambient ender pearls.)
held_item_line() {
    ctl select "${1:-$BLOCK}" >/dev/null
    ctl wait 4 >/dev/null
    ctl command "data get entity @s SelectedItem" >/dev/null
    ctl wait 6 >/dev/null
    ctl messages 8 | jq -r '.chat[]' | grep -m1 'has the following entity data' || true
}

# Collect the drop (`ctl collect` walks to it), then stand on its block as a fallback.
pick_up() { # pick_up X Y Z [ITEM]
    local item=${4:-$BLOCK}
    ctl collect 8 >/dev/null 2>&1 || true
    ctl wait 10 >/dev/null
    if ! ctl inventory | jq -e "[.hotbar[], .main[] | select(.item == \"$item\")] | length >= 1" >/dev/null; then
        ctl command "execute in minecraft:the_end run tp @s $1.5 $2 $3.5" >/dev/null
        ctl wait 20 >/dev/null
    fi
    ctl inventory | jq -e "[.hotbar[], .main[] | select(.item == \"$item\")] | length >= 1" >/dev/null \
        || fail "the broken block's $item item was not picked up: $(ctl inventory | jq -c '{hotbar,main}')"
}

# Place the tank item the player already carries. Never `/item replace` here:
# that hands a fresh, empty tank and proves nothing about the carried one.
place_kept() { # place_kept X Y Z [ITEM]
    local x=$1 y=$2 z=$3 item=${4:-$BLOCK}
    ctl command "execute in minecraft:the_end run tp @s $((x - 2)).5 101 $z.5" >/dev/null
    ctl wait 6 >/dev/null
    try_place "$item" "$x" "$y" "$z"
    ctl wait 6 >/dev/null
    [ "$(block_name "$x" "$y" "$z")" = "$item" ] || fail "the carried $item did not place at $x,$y,$z"
}


# ---------------------------------------------------------------------------
# The crafter (slice 3): readouts and the harness's controls
# ---------------------------------------------------------------------------

# `/rftoolsutility crafter X Y Z`, parsed into C_ENERGY/C_CAP/C_SPEED/C_MODE/C_KEEP/
# C_RECIPES/C_IN/C_OUT.
crafter() { # crafter X Y Z
    NONCE=$((NONCE + 1))
    say "rftoolsutility crafter $1 $2 $3 $NONCE"
    ctl wait 12 >/dev/null
    local line
    line=$(ctl messages 20 | jq -r '.chat[]' | grep -m1 "nonce=$NONCE" || true)
    [ -n "$line" ] || fail "no fresh /rftoolsutility crafter reply for nonce $NONCE: $(ctl messages 8 | jq -c '.chat')"
    eval "$(python3 "$PORT/tools/parse_crafter.py" "$line")"
    [ "${ok:-0}" = 1 ] || fail "the crafter line did not parse: $line"
    echo "  $line" >&2
}

# Run a crafter control and keep its own reply (the newest `/rftoolsutility:` line, after a
# 12-tick wait); the caller asserts on that text and on the readout that follows.
control() { # control ARGS…
    say "rftoolsutility $*"
    ctl wait 12 >/dev/null
    # `ctl messages` lists the newest line first, so `-m1` is this command's own reply.
    REPLY=$(ctl messages 12 | jq -r '.chat[]' | grep -m1 '/rftoolsutility' || true)
    echo "  $REPLY" >&2
}

teach() { control recipe "$@"; }
set_slot() { control buffer "$@"; }
set_energy() { control energy "$@"; }
set_setting() { control setting "$@"; }

expect_energy() { [ "$C_ENERGY" = "$1" ] || fail "the crafter holds $C_ENERGY FE, expected $1"; }
expect_recipes() { [ "$C_RECIPES" = "$1" ] || fail "the crafter's recipes read '$C_RECIPES', expected '$1'"; }
expect_in() { [ "$C_IN" = "$1" ] || fail "the crafter's buffer reads '$C_IN', expected '$1'"; }
expect_out() { [ "$C_OUT" = "$1" ] || fail "the crafter's output reads '$C_OUT', expected '$1'"; }
# The total number of ITEM in the crafter's input buffer, across however many slots (and
# however they merged): `in_total minecraft:crafting_table`.
in_total() { # in_total ITEM
    printf '%s' "$C_IN" | tr ';' '\n' | sed -n "s/.*=$1x\([0-9]*\)$/\1/p" \
        | awk '{sum += $1} END {print sum + 0}'
}
in_has() { [[ "$C_IN" == *"$1"* ]] || fail "the crafter's buffer '$C_IN' does not hold '$1'"; }

echo "== launch (The End arena, the package's own content and code) =="
launch
arena

# Survival from here: the engine's own `FluidUtil` path (the SDK's
# `fluid_util::interact_with_fluid_handler`) keeps a creative player's container
# stack, so the bucket swap a survival player sees would never happen. Vanilla's
# creative rule is the deviation recorded in notes.md; the mod's mechanic is the
# survival one, and the drop phases need survival too (a creative break drops
# nothing).
ctl command "gamemode survival" >/dev/null
ctl wait 4 >/dev/null

ctl command "item replace entity @s hotbar.0 with $BLOCK 1" >/dev/null
ctl wait 4 >/dev/null
ctl inventory | jq -e "[.hotbar[], .main[] | select(.item == \"$BLOCK\")] | length >= 1" >/dev/null \
    || fail "the tank item did not register: $(ctl inventory | jq -c '{hotbar,main}')"
echo "  items: $(ctl inventory | jq -c '[.hotbar[], .main[] | select(.item != null) | .item | select(startswith("rftoolsutility:"))] | unique')"

echo "== A: a tank placed from its item is empty =="
place_tank "$SX" 101 0
info "$SX" 101 0
[ "$CAPACITY" = "$EXPECT_CAPACITY" ] || fail "the tank's capacity reads $CAPACITY, expected $EXPECT_CAPACITY"
[ "$AMOUNT" = 0 ] || fail "a fresh tank holds $AMOUNT mB"
[ "$FLUID" = empty ] || fail "a fresh tank holds '$FLUID'"
[ "$LEVEL" = 0 ] || fail "a fresh tank's level is $LEVEL"
expect_block_level "$SX" 101 0 0
echo "  block $(block_name "$SX" 101 0), $AMOUNT/$CAPACITY mB, level $LEVEL"

echo "== B: a water bucket fills exactly 1,000 mB and the hand becomes an empty bucket =="
click_with "$SX" 101 0 "$WATER_BUCKET"
info "$SX" 101 0
if [ "$SELFTEST" = 1 ] && [ "$SELFTEST_AT" = fill ]; then
    echo "(selftest: asserting the tank is still empty)"
    expect_amount 0
else
    expect_amount "$BUCKET"
fi
[ "$FLUID" = "minecraft:water" ] || fail "the tank holds '$FLUID', expected minecraft:water"
expect_held "$EMPTY_BUCKET"
expect_level 1
expect_block_level "$SX" 101 0 1
echo "  one water bucket: $AMOUNT mB of $FLUID, level $LEVEL, hand $HELD_ITEM"
ctl inventory | jq -e "[.hotbar[] | select(.item == \"$EMPTY_BUCKET\")] | length == 1" >/dev/null \
    || fail "the client's hotbar does not show the empty bucket: $(ctl inventory | jq -c '.hotbar')"
echo "  client hotbar: $(ctl inventory | jq -c '[.hotbar[] | select(.item != null)]')"

echo "== C: an empty bucket drains exactly 1,000 mB back =="
click_with "$SX" 101 0 "$EMPTY_BUCKET"
info "$SX" 101 0
expect_amount 0
[ "$FLUID" = empty ] || fail "an emptied tank holds '$FLUID'"
expect_held "$WATER_BUCKET"
expect_level 0
expect_block_level "$SX" 101 0 0
echo "  one drain: $AMOUNT mB, level $LEVEL, hand $HELD_ITEM"

echo "== D: at 31,500 mB one more bucket does not fit =="
set_tank "$SX" 101 0 minecraft:water 31500
info "$SX" 101 0
expect_amount 31500
expect_level 8
click_with "$SX" 101 0 "$WATER_BUCKET"
info "$SX" 101 0
expect_amount 31500
expect_held "$WATER_BUCKET"
echo "  500 mB of room: still $AMOUNT mB, the bucket stayed in the hand"

echo "== E: the capacity is 32,000 mB =="
set_tank "$SX" 101 0 minecraft:water 40000
info "$SX" 101 0
expect_amount "$EXPECT_CAPACITY"
expect_level 8
click_with "$SX" 101 0 "$WATER_BUCKET"
info "$SX" 101 0
expect_amount "$EXPECT_CAPACITY"
echo "  40,000 mB asked, $AMOUNT mB held; a full tank takes no bucket"

echo "== F: a lava bucket into a water tank does nothing =="
set_tank "$SX" 101 0 minecraft:water 1000
info "$SX" 101 0
expect_amount 1000
click_with "$SX" 101 0 "$LAVA_BUCKET"
info "$SX" 101 0
expect_amount 1000
[ "$FLUID" = "minecraft:water" ] || fail "the tank now holds '$FLUID'"
expect_held "$LAVA_BUCKET"
echo "  one fluid at a time: $AMOUNT mB of $FLUID, the lava bucket stayed"

echo "== G: an empty bucket on an empty tank does nothing =="
set_tank "$SX" 101 0 minecraft:water 0
click_with "$SX" 101 0 "$EMPTY_BUCKET"
info "$SX" 101 0
expect_amount 0
expect_held "$EMPTY_BUCKET"
expect_block_level "$SX" 101 0 0
echo "  empty tank, empty bucket: $AMOUNT mB, hand $HELD_ITEM"

echo "== H: a vanilla cauldron takes the same water bucket =="
ctl command "execute in minecraft:the_end run setblock $SX 101 -8 $CAULDRON" >/dev/null
ctl wait 6 >/dev/null
[ "$(block_name "$SX" 101 -8)" = "$CAULDRON" ] || fail "the cauldron did not place"
click_with "$SX" 101 -8 "$WATER_BUCKET"
CAULDRON_NOW=$(block_name "$SX" 101 -8)
echo "  the cauldron at $SX,101,-8 is now '$CAULDRON_NOW'" >&2
[ "$CAULDRON_NOW" = "minecraft:water_cauldron" ] \
    || fail "the cauldron became '$CAULDRON_NOW', expected minecraft:water_cauldron"
ctl inventory | jq -e "[.hotbar[] | select(.item == \"$EMPTY_BUCKET\")] | length >= 1" >/dev/null \
    || fail "the cauldron did not empty the bucket: $(ctl inventory | jq -c '.hotbar')"
echo "  the same click emptied the bucket into $CAULDRON_NOW"

echo "== I: the tank keeps its fluid across save and relaunch =="
set_tank "$SX" 101 0 minecraft:water 5000
info "$SX" 101 0
expect_amount 5000
ctl save >/dev/null
ctl wait 10 >/dev/null
ctl quit >/dev/null 2>&1 || true
sleep 2
launch
# No arena() here: its air fill would take the tank with it.
ctl command "execute in minecraft:the_end run tp @s $((SX - 2)).5 101 0.5" >/dev/null
ctl wait 40 >/dev/null
[ "$(block_name "$SX" 101 0)" = "$BLOCK" ] || fail "the tank did not survive the relaunch"
info "$SX" 101 0
expect_amount 5000
[ "$FLUID" = "minecraft:water" ] || fail "after the relaunch the tank holds '$FLUID'"
expect_block_level "$SX" 101 0 2
echo "  after save + relaunch: $AMOUNT mB of $FLUID, level $(block_level "$SX" 101 0)"

echo "== J: a broken tank carries its contents on its item =="
ctl command "gamemode survival" >/dev/null
ctl command "clear @s" >/dev/null
ctl wait 6 >/dev/null
place_tank "$SX" 101 -6
set_tank "$SX" 101 -6 minecraft:water 5000
info "$SX" 101 -6
expect_amount 5000
expect_block_level "$SX" 101 -6 2
break_block "$SX" 101 -6 minecraft:iron_pickaxe
pick_up "$SX" 101 -6
DROP_LINE=$(held_item_line)
[ -n "$DROP_LINE" ] || fail "no /data get reply for the picked-up tank: $(ctl messages 8 | jq -c '.chat')"
echo "  $DROP_LINE" >&2
eval "$(python3 "$PORT/tools/parse_drop.py" "$DROP_LINE" 5000)" \
    || fail "the dropped tank does not carry 5,000 mB of water: $DROP_LINE"
[ "${ok:-0}" = 1 ] || fail "the drop's block-entity data did not parse: $DROP_LINE"
echo "  the picked-up tank carries $carried mB of $carried_fluid ($component)"

echo "== K: the item puts the contents back when it is placed =="
place_kept "$SX" 101 6
info "$SX" 101 6
if [ "$SELFTEST" = 1 ] && [ "$SELFTEST_AT" = restore ]; then
    echo "(selftest: asserting the restored tank is empty)"
    expect_amount 0
fi
expect_amount 5000
[ "$FLUID" = "minecraft:water" ] || fail "the restored tank holds '$FLUID', expected minecraft:water"
expect_level 2
expect_block_level "$SX" 101 6 2
echo "  placed from its own item: $AMOUNT mB of $FLUID, level $LEVEL"

echo "== L: an empty tank's item carries nothing (the same check, inverted) =="
place_tank "$SX" 101 -12
info "$SX" 101 -12
expect_amount 0
break_block "$SX" 101 -12 minecraft:iron_pickaxe
pick_up "$SX" 101 -12
DROP_LINE=$(held_item_line)
[ -n "$DROP_LINE" ] || fail "no /data get reply for the picked-up empty tank: $(ctl messages 8 | jq -c '.chat')"
echo "  $DROP_LINE" >&2
eval "$(python3 "$PORT/tools/parse_drop.py" "$DROP_LINE" none)" \
    || fail "an empty tank's item carries block-entity data: $DROP_LINE"
[ "${ok:-0}" = 1 ] || fail "the empty drop did not parse: $DROP_LINE"
place_kept "$SX" 101 -12
info "$SX" 101 -12
expect_amount 0
expect_level 0
echo "  an empty tank's item: $component, placed back as $AMOUNT mB"

echo "== M: a doctored item places a tank clamped to 32,000 mB =="
ctl command "give @s rftoolsutility:tank[minecraft:block_entity_data={id:\"rftoolsutility:tank\",fluids:[{Fluid:\"minecraft:water\",Amount:99999}]}]" >/dev/null
ctl wait 8 >/dev/null
place_kept "$SX" 101 12
info "$SX" 101 12
expect_amount "$EXPECT_CAPACITY"
expect_level 8
expect_block_level "$SX" 101 12 8
echo "  99,999 mB asked by the item, $AMOUNT mB in the placed tank (level $LEVEL)"


echo "== N: a crafter placed from its item is empty and knows no recipe =="
place_block "$CRAFTER" "$SX" 101 -18
crafter "$SX" 101 -18
[ "$C_CAP" = 50000 ] || fail "the crafter's capacity reads $C_CAP, expected 50000"
expect_energy 0
expect_recipes -
expect_in -
expect_out -
CRAFTER_STATE=$(ctl block "$SX" 101 -18 | jq -r '.state // empty')
echo "  block state $CRAFTER_STATE" >&2
case "$CRAFTER_STATE" in
    *facing=north*|*facing=south*|*facing=east*|*facing=west*) ;;
    *) fail "the crafter has no horizontal facing state: $CRAFTER_STATE" ;;
esac

echo "== O: a recipe is taught from the engine's table, and a bad one is refused =="
teach "$SX" 101 -18 0 minecraft:oak_planks
[[ "$REPLY" == *"crafts minecraft:oak_planks"* ]] || fail "teaching oak_planks did not answer: $REPLY"
crafter "$SX" 101 -18
expect_recipes minecraft:oak_planks
# A slot past this machine's two, and an id the crafting table does not hold: both refused.
teach "$SX" 101 -18 5 minecraft:oak_planks
[[ "$REPLY" == *"2 recipe slots"* ]] || fail "a fifth recipe slot was not refused: $REPLY"
teach "$SX" 101 -18 0 minecraft:iron_ingot_from_smelting
[[ "$REPLY" == *"no crafting recipe"* ]] || fail "a smelting recipe was not refused: $REPLY"
crafter "$SX" 101 -18
expect_recipes minecraft:oak_planks
echo "  the crafter still knows only $C_RECIPES"

echo "== P: with no energy nothing is crafted =="
set_slot "$SX" 101 -18 10 minecraft:oak_log 3
ctl wait 30 >/dev/null
crafter "$SX" 101 -18
expect_energy 0
expect_in 10=minecraft:oak_logx3
expect_out -
echo "  three logs, no FE: $C_IN, output '$C_OUT'"

echo "== Q: 5,000 FE and three logs become twelve planks for 300 FE =="
set_energy "$SX" 101 -18 5000
ctl wait 40 >/dev/null
crafter "$SX" 101 -18
if [ "$SELFTEST" = 1 ] && [ "$SELFTEST_AT" = craft ]; then
    echo "(selftest: asserting the crafter made nothing and spent nothing)"
    expect_energy 5000
    expect_in 10=minecraft:oak_logx3
    expect_out -
else
    expect_energy 4700
    expect_in -
    expect_out 36=minecraft:oak_planksx12
fi
echo "  three logs crafted: $C_OUT, $C_ENERGY FE left"

echo "== R: an input the recipe does not take is left alone and costs nothing =="
set_slot "$SX" 101 -18 10 minecraft:cobblestone 3
set_energy "$SX" 101 -18 1000
ctl wait 30 >/dev/null
crafter "$SX" 101 -18
expect_energy 1000
expect_in 10=minecraft:cobblestonex3
echo "  cobblestone is no plank: $C_IN, $C_ENERGY FE untouched"

echo "== S: a full output refuses the operation (nothing consumed, no energy) =="
# Setup runs at 0 FE so nothing crafts while the machine is being filled.
set_energy "$SX" 101 -18 0
for slot in 36 37 38 39; do
    set_slot "$SX" 101 -18 "$slot" minecraft:wooden_pickaxe 1
done
set_slot "$SX" 101 -18 10 minecraft:oak_log 2
set_energy "$SX" 101 -18 1000
ctl wait 30 >/dev/null
crafter "$SX" 101 -18
expect_energy 1000
expect_in 10=minecraft:oak_logx2
[ "$C_OUT" = "36=minecraft:wooden_pickaxex1;37=minecraft:wooden_pickaxex1;38=minecraft:wooden_pickaxex1;39=minecraft:wooden_pickaxex1" ] \
    || fail "the four output slots did not stay as they were: $C_OUT"
echo "  no room for planks: $C_ENERGY FE untouched, the logs stay"

echo "== T: chained crafting in int mode turns four logs into four crafting tables =="
# The machine is emptied and reconfigured at 0 FE, so only the four logs below are crafted.
set_energy "$SX" 101 -18 0
for slot in 36 37 38 39; do
    set_slot "$SX" 101 -18 "$slot" minecraft:air 0
done
set_slot "$SX" 101 -18 10 minecraft:air 0
set_setting "$SX" 101 -18 mode int
set_setting "$SX" 101 -18 speed slow
teach "$SX" 101 -18 1 minecraft:crafting_table
[[ "$REPLY" == *"crafts minecraft:crafting_table"* ]] || fail "teaching the crafting table did not answer: $REPLY"
set_slot "$SX" 101 -18 10 minecraft:oak_log 4
set_energy "$SX" 101 -18 5000
ctl wait 60 >/dev/null
crafter "$SX" 101 -18
# Upstream charges per *cycle* (`rf * i`), and a cycle runs each taught recipe once: four
# cycles, two crafts each, is 400 FE.
expect_energy 4600
expect_out -
[ "$(in_total minecraft:crafting_table)" = 4 ] \
    || fail "four logs did not become four crafting tables inside the machine: $C_IN"
echo "  four logs ($C_ENERGY FE left) chained into four tables in the buffer: $C_IN"

echo "== U: a broken crafter keeps its recipes, buffer and energy on its item =="
ctl command "gamemode survival" >/dev/null
ctl wait 20 >/dev/null
ctl command "clear @s" >/dev/null
ctl wait 6 >/dev/null
break_block "$SX" 101 -18 minecraft:iron_pickaxe
# A survival break needs the player to be in reach; the drop is the crafter's own item.
pick_up "$SX" 101 -18 "$CRAFTER"
DROP_LINE=$(held_item_line "$CRAFTER")
[ -n "$DROP_LINE" ] || fail "no /data get reply for the picked-up crafter: $(ctl messages 8 | jq -c '.chat')"
[[ "$DROP_LINE" == *"minecraft:block_entity_data"* ]] \
    || fail "the dropped crafter carries no block-entity data: $DROP_LINE"
echo "  $DROP_LINE" >&2
place_kept "$SX" 101 -18 "$CRAFTER"
crafter "$SX" 101 -18
expect_recipes "minecraft:oak_planks+minecraft:crafting_table"
expect_energy 4600
[ "$(in_total minecraft:crafting_table)" = 4 ] \
    || fail "the placed crafter lost its crafting tables: $C_IN"
echo "  the placed crafter still knows $C_RECIPES, holds $C_ENERGY FE and $C_IN"


echo "== V: the crafter drops only for an iron-tier pickaxe (the needs_iron_tool tag) =="
ctl command "gamemode survival" >/dev/null
ctl wait 20 >/dev/null
ctl command "clear @s" >/dev/null
ctl wait 6 >/dev/null
place_block "$CRAFTER" "$SX" 101 18
break_block "$SX" 101 18 minecraft:wooden_pickaxe
ctl collect 8 >/dev/null 2>&1 || true
ctl wait 10 >/dev/null
if ctl inventory | jq -e "[.hotbar[], .main[] | select(.item == \"$CRAFTER\")] | length >= 1" >/dev/null; then
    fail "a wooden pickaxe broke the crafter and it dropped: the block is not needs_iron_tool"
fi
echo "  a wooden pickaxe broke the crafter and dropped nothing"
# The control: the same block, the same break, with a tool the tag accepts.
place_block "$CRAFTER" "$SX" 101 18
break_block "$SX" 101 18 minecraft:iron_pickaxe
pick_up "$SX" 101 18 "$CRAFTER"
echo "  an iron pickaxe broke the same block and the item came back"

echo "PASS: a fresh tank is empty (level 0); a water bucket fills exactly 1,000 mB, swaps the hand to an empty bucket and raises the block's level to 1; an empty bucket drains exactly 1,000 mB back and restores the water bucket and level 0; at 31,500 mB one more bucket does not fit; the capacity is exactly 32,000 mB; a lava bucket is refused by a water tank; an empty bucket on an empty tank does nothing; the same click fills a vanilla cauldron (so the click path works); 5,000 mB of water survive save + relaunch with the level re-mirrored to 2; a broken 5,000 mB tank drops one tank whose item carries minecraft:block_entity_data with that water, and placing that item restores a 5,000 mB tank at level 2; an empty tank's item carries no block-entity data and places back empty; an item doctored to 99,999 mB places a tank clamped to the 32,000 mB capacity; a crafter placed from its item is empty and knows no recipe; a recipe is taught from the engine's crafting table while a fifth slot and a smelting recipe are refused; no energy crafts nothing; 5,000 FE and three logs become twelve planks for exactly 300 FE; an input the recipe does not take is left alone; a full output buffer refuses the operation without consuming or spending; chained crafting in int mode turns four logs into four crafting tables for 400 FE; a broken crafter keeps its recipes, buffer and energy on its own item, which placing back restores; its loot is gated by the iron-tool tag, so a wooden pickaxe drops nothing and an iron one drops it"
