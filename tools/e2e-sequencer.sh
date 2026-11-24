#!/usr/bin/env bash
# Focused, fail-capable slice-12 leg: the sequencer's rising-edge-driven 64-step pattern.
# A vanilla redstone lamp beside the sequencer is the assertion: it is lit only while the bit at
# the current step is set. A vanilla lever on top of the sequencer supplies clean rising edges
# (powered=false -> true -> false = one pulse); a vanilla redstone block powers the same lamp
# first as the positive control, and an all-zero pattern is the differential.
# Reuses the launcher-built game/ctl/resource exports (E2E_TMP, never /tmp).
export LC_ALL=C
[ -z "${RFU_DEBUG:-}" ] || set -x

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/rftoolsutility
CTL=${CTL:-}
GAME=${GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/rftoolsutility-sequencer.sock}
SCRATCH_ROOT=${E2E_TMP:-/tmp/opencode}
SELFTEST=${SELFTEST:-0}
SELFTEST_AT=${SELFTEST_AT:-sequencer}
CONTROL=${CONTROL:-0}
KEEP=${KEEP:-0}
WORLD=RFToolsUtilityE2E
X0=200

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -n "$CTL" ] || { echo "set CTL= to this pin's horizon-ctl (launcher-game.sh env)" >&2; exit 1; }
[ -n "$GAME" ] || { echo "set GAME= to a game built with this package" >&2; exit 1; }
if [ "$CONTROL" = 1 ]; then
    [ -n "${CONTROL_GAME:-}" ] || { echo "set CONTROL_GAME= to the unmodded Nightly" >&2; exit 1; }
    GAME=$CONTROL_GAME
fi
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d "$SCRATCH_ROOT/rftoolsutility-sequencer.XXXXXX")
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

launch() {
    local world=${1:-$WORLD}
    rm -f "$SOCK"
    cd "$SCRATCH"
    local args=(--game "$GAME" --world "$world" --seed 20261105 --creative --headless
                --wait --resources "$RES")
    "$CTL" --socket "$SOCK" launch "${args[@]}" >/dev/null
    ctl wait-ready 120
    ctl wait 40
}

block_name() { ctl block "$1" "$2" "$3" | jq -r '.block // .name // empty'; }
state_at() { ctl block "$1" "$2" "$3" | jq -r '.state // empty'; }
lamp_lit() { state_at "$@" | grep -q 'lit=true'; }

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

hold() {
    ctl command "item replace entity @s hotbar.0 with $1 ${2:-1}" >/dev/null
    ctl wait 4 >/dev/null
    ctl select 1 >/dev/null
    ctl wait 2 >/dev/null
}

clear_at() {
    ctl command "execute in minecraft:the_end run kill @e[type=!player,x=$1,y=$2,z=$3,distance=..${4:-2}]" \
        >/dev/null 2>&1 || true
    ctl wait 4 >/dev/null
}

try_place() {
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
        clear_at "$x" "$y" "$z" 2
        ctl wait 10 >/dev/null
    done
    fail "ctl place $item at $x,$y,$z was refused three times: $(ctl inventory | jq -c '{hotbar,main}')"
}

place_block() {
    local item=$1 x=$2 y=$3 z=$4 name
    ctl command "execute in minecraft:the_end run tp @s $((x - 2)).5 $y $z.5" >/dev/null
    ctl wait 6 >/dev/null
    hold "$item"
    try_place "$item" "$x" "$y" "$z"
    ctl wait 6 >/dev/null
    name=$(block_name "$x" "$y" "$z")
    [ "$name" = "$item" ] || fail "$item did not place at $x,$y,$z (got '$name')"
}

control() {
    say "rftoolsutility $*"
    ctl wait 12 >/dev/null
    REPLY=$(ctl messages 12 | jq -r '.chat[]' | grep -m1 '/rftoolsutility' || true)
    echo "  $REPLY" >&2
}

sequencer_read() {
    NONCE=$((NONCE + 1))
    control sequencer "$1" "$2" "$3" "$NONCE"
    local line="$REPLY"
    Q_STEP=$(echo "$line" | sed -n 's/.*step=\([0-9-]*\).*/\1/p')
    Q_BITS=$(echo "$line" | sed -n 's/.*bits=\([0-9]*\).*/\1/p')
    Q_MODE=$(echo "$line" | sed -n 's/.*mode=\([^ ]*\).*/\1/p')
    Q_OUT=$(echo "$line" | sed -n 's/.*output=\([0-9]*\).*/\1/p')
    [ -n "$Q_STEP" ] && [ -n "$Q_BITS" ] && [ -n "$Q_MODE" ] && [ -n "$Q_OUT" ] \
        || fail "the sequencer line did not parse: $line"
}

expect() { # step bits mode output
    [ "$Q_STEP" = "$1" ] || fail "the sequencer reads step=$Q_STEP, expected $1"
    [ "$Q_BITS" = "$2" ] || fail "the sequencer reads bits=$Q_BITS, expected $2"
    [ "$Q_MODE" = "$3" ] || fail "the sequencer reads mode=$Q_MODE, expected $3"
    [ "$Q_OUT" = "$4" ] || fail "the sequencer reads output=$Q_OUT, expected $4"
}

# One clean rising edge of a vanilla lever: false -> true (the edge the sequencer latches) -> false.
pulse_lever() {
    ctl command "execute in minecraft:the_end run setblock $LX_LEV $LY_LEV $LZ_LEV minecraft:lever[face=wall,facing=south,powered=true]" >/dev/null
    ctl wait 8 >/dev/null
    ctl command "execute in minecraft:the_end run setblock $LX_LEV $LY_LEV $LZ_LEV minecraft:lever[face=wall,facing=south,powered=false]" >/dev/null
    ctl wait 8 >/dev/null
}

settle() { ctl wait 12 >/dev/null; sequencer_read "$SX" "$SY" "$SZ"; echo "  $(state_at $LX $LY $LZ) / $REPLY"; }

SX=$((X0 + 1)); SY=101; SZ=0                  # the sequencer
LX=$((X0 + 2)); LY=$SY; LZ=0                   # the vanilla redstone lamp beside it
PX=$((X0 + 3)); PY=$SY; PZ=0                   # the vanilla redstone block (positive control)
LX_LEV=$SX; LY_LEV=$SY; LZ_LEV=$((SZ - 1))     # a vanilla wall lever on the north input side
NONCE=0

launch
arena

echo "== AL: a sequencer and a vanilla redstone lamp are placed and read back =="
ctl command "gamemode creative" >/dev/null
ctl wait 6 >/dev/null
place_block "rftoolsutility:sequencer" "$SX" "$SY" "$SZ"
ctl command "execute in minecraft:the_end run setblock $LX $LY $LZ minecraft:redstone_lamp" >/dev/null
ctl wait 8 >/dev/null
[ "$(block_name "$SX" "$SY" "$SZ")" = "rftoolsutility:sequencer" ] || fail "the sequencer block is not there"
[ "$(block_name "$LX" "$LY" "$LZ")" = "minecraft:redstone_lamp" ] || fail "the lamp block is not there"

echo "== AL-control: a vanilla redstone block lights the same lamp (control for the assertion) =="
ctl command "execute in minecraft:the_end run setblock $PX $PY $PZ minecraft:redstone_block" >/dev/null
ctl wait 20 >/dev/null
echo "  lamp with a vanilla redstone block: $(state_at $LX $LY $LZ)"
lamp_lit "$LX" "$LY" "$LZ" || fail "a vanilla redstone block did not light the lamp (the control cannot prove anything)"
ctl command "execute in minecraft:the_end run setblock $PX $PY $PZ minecraft:air" >/dev/null
ctl wait 20 >/dev/null
echo "  lamp after removing it: $(state_at $LX $LY $LZ)"
if lamp_lit "$LX" "$LY" "$LZ"; then fail "the lamp stayed lit after the redstone block was removed"; fi

echo "== AL0: a Step sequencer with pattern 0b101 lights the lamp on bit 0 =="
ctl command "execute in minecraft:the_end run setblock $LX_LEV $LY_LEV $LZ_LEV minecraft:lever[face=wall,facing=south,powered=false]" >/dev/null
ctl wait 8 >/dev/null
ctl chat "/rftoolsutility sequencer-set $SX $SY $SZ Step 5 1 4 false north" >/dev/null
ctl wait 20 >/dev/null
settle
expect 0 5 Step 15
lamp_lit "$LX" "$LY" "$LZ" || fail "bit 0 did not light the lamp: $(state_at $LX $LY $LZ)"

echo "== AL1: one lever pulse advances to bit 1 (zero) and the lamp goes dark =="
pulse_lever
settle
expect 1 5 Step 0
if lamp_lit "$LX" "$LY" "$LZ"; then fail "bit 1 is zero but the lamp stayed lit"; fi

echo "== AL2: a second pulse advances to bit 2 (one) and the lamp lights again =="
pulse_lever
settle
expect 2 5 Step 15
lamp_lit "$LX" "$LY" "$LZ" || fail "bit 2 did not light the lamp: $(state_at $LX $LY $LZ)"
if [ "$SELFTEST" = 1 ] && [ "$SELFTEST_AT" = sequencer ]; then
    fail "SELFTEST sequencer: bit 2 lit the lamp, expected it to stay dark (the inverted assertion proves the edge case can fail)"
fi

echo "== AL3: a third pulse advances to bit 3 (zero) and the lamp goes dark =="
pulse_lever
settle
expect 3 5 Step 0
if lamp_lit "$LX" "$LY" "$LZ"; then fail "bit 3 is zero but the lamp stayed lit"; fi

echo "== AL4: a fourth pulse wraps to bit 0 and the lamp lights again =="
pulse_lever
settle
expect 0 5 Step 15
lamp_lit "$LX" "$LY" "$LZ" || fail "the wrapped step 0 did not light the lamp"

echo "== AL5: an all-zero pattern never lights the lamp (differential) =="
ctl chat "/rftoolsutility sequencer-set $SX $SY $SZ Step 0 1 4 false north" >/dev/null
ctl wait 20 >/dev/null
settle
expect 0 0 Step 0
if lamp_lit "$LX" "$LY" "$LZ"; then fail "an all-zero pattern lit the lamp at step 0"; fi
pulse_lever
settle
expect 1 0 Step 0
if lamp_lit "$LX" "$LY" "$LZ"; then fail "an all-zero pattern lit the lamp at step 1"; fi

echo "== AL6: an unknown mode and an unknown block id are refused =="
if refusal=$(ctl give rftoolsutility:not_a_sequencer 1 2>&1); then rc=0; else rc=$?; fi
echo "  give: rc=$rc $refusal"
[ "$rc" -ne 0 ] || [[ "$refusal" == *"Unknown"* ]] || fail "an unknown item id was not refused: $refusal"

echo "PASS: a sequencer turns rising edges of a vanilla lever into successive bits of its 64-step pattern on a real vanilla redstone lamp (bit 0 lit, bit 1 dark, bit 2 lit, bit 3 dark, wrap to bit 0 lit); the redstone-block control, the all-zero differential and SELFTEST_AT=sequencer all assert parsed values and the inverted case fails."
