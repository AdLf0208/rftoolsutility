#!/usr/bin/env bash
# Focused, fail-capable slice-14 leg: the counter's rising-edge count and its latched redstone
# output. A vanilla redstone lamp beside the counter is the assertion; a vanilla lever on the
# north input side supplies the rising edges, and a vanilla redstone block powers the same lamp
# first as the positive control. The counter's own harness readout carries game-time stamps of the
# last edge (`pulse`) and the fire that reached the limit (`fire`), so the "only on the limit-th
# pulse" claim is asserted from the replies' own ticks, never wall time or an absence of errors.
# Reuses the launcher-built game/ctl/resource exports (E2E_TMP, never /tmp).
export LC_ALL=C
[ -z "${RFU_DEBUG:-}" ] || set -x

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/rftoolsutility
CTL=${CTL:-}
GAME=${GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/rftoolsutility-counter.sock}
SCRATCH_ROOT=${E2E_TMP:-/tmp/opencode}
SELFTEST=${SELFTEST:-0}
SELFTEST_AT=${SELFTEST_AT:-counter}
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

SCRATCH=$(mktemp -d "$SCRATCH_ROOT/rftoolsutility-counter.XXXXXX")
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

# A unique nonce pins the reply even under chat load; parse the reply's own fields.
counter_read() {
    NONCE=$((NONCE + 1))
    ctl chat "/rftoolsutility counter $SX $SY $SZ $NONCE" >/dev/null
    ctl wait 1 >/dev/null
    REPLY=$(ctl messages 6 | jq -r '.chat[]' | grep -m1 "nonce=$NONCE" || true)
    [ -n "$REPLY" ] || fail "no counter reply for nonce $NONCE"
    echo "  $REPLY" >&2
    Q_LIMIT=$(echo "$REPLY" | sed -n 's/.*limit=\([0-9-]*\).*/\1/p')
    Q_CURRENT=$(echo "$REPLY" | sed -n 's/.*current=\([0-9-]*\).*/\1/p')
    Q_OUT=$(echo "$REPLY" | sed -n 's/.*output=\([0-9]*\).*/\1/p')
    Q_PULSES=$(echo "$REPLY" | sed -n 's/.*pulses=\([0-9-]*\).*/\1/p')
    Q_PULSE=$(echo "$REPLY" | sed -n 's/.*pulse=\([0-9-]*\).*/\1/p')
    Q_FIRE=$(echo "$REPLY" | sed -n 's/.*fire=\([0-9-]*\).*/\1/p')
    Q_LAMP=$(echo "$REPLY" | sed -n 's/.*lamp=\([a-z]*\).*/\1/p')
    [ -n "$Q_LIMIT" ] && [ -n "$Q_CURRENT" ] && [ -n "$Q_OUT" ] && [ -n "$Q_PULSES" ] \
        && [ -n "$Q_PULSE" ] && [ -n "$Q_FIRE" ] && [ -n "$Q_LAMP" ] \
        || fail "the counter line did not parse: $REPLY"
}

# One clean rising edge of a vanilla lever on the north input side: false -> true -> false.
pulse_lever() {
    ctl command "execute in minecraft:the_end run setblock $LX_LEV $LY_LEV $LZ_LEV minecraft:lever[face=wall,facing=south,powered=true]" >/dev/null
    ctl wait 8 >/dev/null
    ctl command "execute in minecraft:the_end run setblock $LX_LEV $LY_LEV $LZ_LEV minecraft:lever[face=wall,facing=south,powered=false]" >/dev/null
    ctl wait 8 >/dev/null
}

expect_state() { # limit current output pulses
    [ "$Q_LIMIT" = "$1" ] || fail "the counter reads limit=$Q_LIMIT, expected $1"
    [ "$Q_CURRENT" = "$2" ] || fail "the counter reads current=$Q_CURRENT, expected $2"
    [ "$Q_OUT" = "$3" ] || fail "the counter reads output=$Q_OUT, expected $3"
    [ "$Q_PULSES" = "$4" ] || fail "the counter reads pulses=$Q_PULSES, expected $4"
}

SX=$((X0 + 1)); SY=101; SZ=0                  # the counter
LX=$((X0 + 2)); LY=$SY; LZ=0                   # the vanilla redstone lamp beside it
PX=$((X0 + 3)); PY=$SY; PZ=0                   # the vanilla redstone block (positive control)
LX_LEV=$SX; LY_LEV=$SY; LZ_LEV=$((SZ - 1))     # a vanilla wall lever on the north input side
NONCE=0
LIMIT=4

launch
arena

echo "== AN: a counter and a vanilla redstone lamp are placed and read back =="
ctl command "gamemode creative" >/dev/null
ctl wait 6 >/dev/null
place_block "rftoolsutility:counter" "$SX" "$SY" "$SZ"
ctl command "execute in minecraft:the_end run setblock $LX $LY $LZ minecraft:redstone_lamp" >/dev/null
ctl wait 8 >/dev/null
[ "$(block_name "$SX" "$SY" "$SZ")" = "rftoolsutility:counter" ] || fail "the counter block is not there"
[ "$(block_name "$LX" "$LY" "$LZ")" = "minecraft:redstone_lamp" ] || fail "the lamp block is not there"

echo "== AN-control: a vanilla redstone block lights the same lamp (control for the assertion) =="
ctl command "execute in minecraft:the_end run setblock $PX $PY $PZ minecraft:redstone_block" >/dev/null
ctl wait 20 >/dev/null
echo "  lamp with a vanilla redstone block: $(state_at $LX $LY $LZ)"
lamp_lit "$LX" "$LY" "$LZ" || fail "a vanilla redstone block did not light the lamp (the control cannot prove anything)"
ctl command "execute in minecraft:the_end run setblock $PX $PY $PZ minecraft:air" >/dev/null
ctl wait 20 >/dev/null
echo "  lamp after removing it: $(state_at $LX $LY $LZ)"
if lamp_lit "$LX" "$LY" "$LZ"; then fail "the lamp stayed lit after the redstone block was removed"; fi

echo "== AN0: the GUI's limit setting is command-driven and starts fresh =="
ctl chat "/rftoolsutility counter-set $SX $SY $SZ $LIMIT north" >/dev/null
ctl wait 20 >/dev/null
counter_read
expect_state "$LIMIT" 0 0 0
[ "$Q_FIRE" = "-1" ] || fail "a freshly set counter has a fire stamp: fire=$Q_FIRE"
[ "$Q_LAMP" = "dark" ] || fail "a freshly set counter's lamp is lit: lamp=$Q_LAMP"
if lamp_lit "$LX" "$LY" "$LZ"; then fail "a freshly set counter's lamp is lit: $(state_at $LX $LY $LZ)"; fi

echo "== AN1: a no-pulse window stays dark (differential) =="
ctl wait 40 >/dev/null
counter_read
expect_state "$LIMIT" 0 0 0
[ "$Q_PULSES" = 0 ] || fail "the counter counted with no input pulse (pulses=$Q_PULSES)"
[ "$Q_FIRE" = "-1" ] || fail "the counter fired with no input pulse (fire=$Q_FIRE)"
if lamp_lit "$LX" "$LY" "$LZ"; then fail "the counter lit its lamp with no input pulse"; fi

echo "== AN2-AN4: the first limit-1 pulses count but stay dark =="
for n in 1 2 3; do
    pulse_lever
    counter_read
    expect_state "$LIMIT" "$n" 0 "$n"
    [ "$Q_LAMP" = "dark" ] || fail "pulse $n lit the lamp before the limit: lamp=$Q_LAMP"
    if lamp_lit "$LX" "$LY" "$LZ"; then fail "pulse $n lit the lamp before the limit"; fi
    LAST_PULSE=$Q_PULSE
done

echo "== AN5: the limit-th pulse fires, lights the lamp and resets the count =="
pulse_lever
counter_read
expect_state "$LIMIT" 0 15 "$LIMIT"
echo "  last edge before the fire at tick $LAST_PULSE; fire at tick $Q_FIRE (edge tick $Q_PULSE)"
[ "$Q_FIRE" -ge 0 ] || fail "no fire was stamped: fire=$Q_FIRE"
[ "$Q_PULSE" -ge 0 ] || fail "no rising edge was stamped: pulse=$Q_PULSE"
[ "$Q_FIRE" -eq "$Q_PULSE" ] || fail "the fire was not on the limit-th edge tick: fire=$Q_FIRE pulse=$Q_PULSE"
[ "$Q_FIRE" -gt "$LAST_PULSE" ] || fail "the fire was not after the previous edge: fire=$Q_FIRE last=$LAST_PULSE"
if [ "$SELFTEST" = 1 ] && [ "$SELFTEST_AT" = counter ]; then
    # Invert the lamp observation at the fire: the correct counter lights it, so this must fail.
    [ "$Q_LAMP" = "dark" ] || fail "SELFTEST counter: the limit-th pulse lit the lamp, expected it to stay dark (the lamp assertion can fail)"
else
    [ "$Q_LAMP" = "lit" ] || fail "the limit-th pulse did not light the lamp: lamp=$Q_LAMP"
fi

echo "== AN5b: the output is a level latch, not a one-tick pulse =="
BASE_FIRE=$Q_FIRE
ctl wait 40 >/dev/null
counter_read
expect_state "$LIMIT" 0 15 "$LIMIT"
[ "$Q_FIRE" = "$BASE_FIRE" ] || fail "the counter re-fired with no new pulse: fire=$Q_FIRE baseline=$BASE_FIRE"
[ "$Q_LAMP" = "lit" ] || fail "the latched lamp went dark with no new pulse: lamp=$Q_LAMP"
lamp_lit "$LX" "$LY" "$LZ" || fail "the lamp is dark after the latched output: $(state_at $LX $LY $LZ)"

echo "== AN6: the next pulse clears the latched signal and starts a new count =="
pulse_lever
counter_read
expect_state "$LIMIT" 1 0 "$((LIMIT + 1))"
[ "$Q_LAMP" = "dark" ] || fail "the next pulse did not clear the lamp: lamp=$Q_LAMP"
if lamp_lit "$LX" "$LY" "$LZ"; then fail "the lamp stayed lit after the clearing pulse"; fi

echo "== AN7: an unknown block id is refused =="
if refusal=$(ctl give rftoolsutility:not_a_counter 1 2>&1); then rc=0; else rc=$?; fi
echo "  give: rc=$rc $refusal"
[ "$rc" -ne 0 ] || [[ "$refusal" == *"Unknown"* ]] || fail "an unknown item id was not refused: $refusal"

echo "PASS: a counter counts rising edges from a vanilla lever and lights a vanilla lamp only on the limit-th pulse, latching the signal until the next pulse clears it (all read from the replies' own game-time stamps); the redstone-block control, the no-pulse window, the pre-limit pulses, the latch window and SELFTEST_AT=counter all assert parsed values and the inverted lamp fails."
