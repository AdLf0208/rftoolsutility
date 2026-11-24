#!/usr/bin/env bash
# Focused, fail-capable slice-11 leg: the sensor's real redstone output.
# A vanilla redstone lamp beside the sensor is the assertion: it lights only when the
# ten-tick check sees the named block on the input side. A vanilla redstone block
# powers the same lamp first, as the positive control for the lamp and its parser.
# Reuses the launcher-built game/ctl/resource exports (E2E_TMP, never /tmp).
export LC_ALL=C
[ -z "${RFU_DEBUG:-}" ] || set -x

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/rftoolsutility
CTL=${CTL:-}
GAME=${GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/rftoolsutility-sensor.sock}
SCRATCH_ROOT=${E2E_TMP:-/tmp/opencode}
SELFTEST=${SELFTEST:-0}
SELFTEST_AT=${SELFTEST_AT:-sensor}
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

SCRATCH=$(mktemp -d "$SCRATCH_ROOT/rftoolsutility-sensor.XXXXXX")
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

sensor_read() {
    NONCE=$((NONCE + 1))
    control sensor "$1" "$2" "$3" "$NONCE"
    local line="$REPLY"
    S_SIGNAL=$(echo "$line" | sed -n 's/.*signal=\([0-9]*\).*/\1/p')
    S_TYPE=$(echo "$line" | sed -n 's/.*type=\([^ ]*\).*/\1/p')
    S_TARGET=$(echo "$line" | sed -n 's/.*target=\([^ ]*\).*/\1/p')
    S_FACING=$(echo "$line" | sed -n 's/.*facing=\([^ ]*\).*/\1/p')
    [ -n "$S_SIGNAL" ] && [ -n "$S_TYPE" ] || fail "the sensor line did not parse: $line"
}

expect() { # signal type target facing
    [ "$S_SIGNAL" = "$1" ] || fail "the sensor reads signal=$S_SIGNAL, expected $1"
    [ "$S_TYPE" = "$2" ] || fail "the sensor reads type=$S_TYPE, expected $2"
    [ "$S_TARGET" = "$3" ] || fail "the sensor reads target=$S_TARGET, expected $3"
    [ "$S_FACING" = "$4" ] || fail "the sensor reads facing=$S_FACING, expected $4"
}

set_front() { ctl command "execute in minecraft:the_end run setblock $FX $FY $FZ $1" >/dev/null; ctl wait 30 >/dev/null; }

SENSOR_BLOCK=rftoolsutility:sensor
FX=$((X0 + 1)); FY=101; FZ=-1          # the block on the sensor's north input side
SX=$FX; SY=$FY; SZ=0                    # the sensor
LX=$((X0 + 2)); LY=$FY; LZ=0            # the vanilla redstone lamp beside it
PX=$((X0 + 3)); PY=$FY; PZ=0            # the vanilla redstone block (positive control)
NONCE=0

launch
arena

echo "== AK: a sensor and a vanilla redstone lamp are placed and read back =="
ctl command "gamemode creative" >/dev/null
ctl wait 6 >/dev/null
place_block "$SENSOR_BLOCK" "$SX" "$SY" "$SZ"
# A vanilla block is set, not placed with `ctl place` (which needs an exposed face and refuses a
# lamp beside the sensor). The assertion is the lamp's `lit` state, not the placement gesture.
ctl command "execute in minecraft:the_end run setblock $LX $LY $LZ minecraft:redstone_lamp" >/dev/null
ctl wait 8 >/dev/null
[ "$(block_name "$SX" "$SY" "$SZ")" = "$SENSOR_BLOCK" ] || fail "the sensor block is not there"
[ "$(block_name "$LX" "$LY" "$LZ")" = "minecraft:redstone_lamp" ] || fail "the lamp block is not there"

echo "== AK-control: a vanilla redstone block lights the same lamp (control for the assertion) =="
ctl command "execute in minecraft:the_end run setblock $PX $PY $PZ minecraft:redstone_block" >/dev/null
ctl wait 20 >/dev/null
echo "  lamp with a vanilla redstone block: $(state_at $LX $LY $LZ)"
lamp_lit "$LX" "$LY" "$LZ" || fail "a vanilla redstone block did not light the lamp (the control cannot prove anything)"
ctl command "execute in minecraft:the_end run setblock $PX $PY $PZ minecraft:air" >/dev/null
ctl wait 20 >/dev/null
echo "  lamp after removing it: $(state_at $LX $LY $LZ)"
if lamp_lit "$LX" "$LY" "$LZ"; then fail "the lamp stayed lit after the redstone block was removed"; fi

echo "== AK0: an undialed sensor with air in front reads 0 and the lamp is dark =="
control sensor-set "$SX" "$SY" "$SZ" blocks minecraft:stone 1 one north
ctl wait 30 >/dev/null
sensor_read "$SX" "$SY" "$SZ"
echo "  $REPLY"
expect 0 blocks minecraft:stone north
if lamp_lit "$LX" "$LY" "$LZ"; then fail "the lamp lit with air in front of the sensor"; fi

echo "== AK1: the matching block in front raises the real redstone signal to 15 =="
set_front minecraft:stone
sensor_read "$SX" "$SY" "$SZ"
echo "  $(state_at $LX $LY $LZ) / $REPLY"
expect 15 blocks minecraft:stone north
lamp_lit "$LX" "$LY" "$LZ" || fail "the sensor's matching block did not light the lamp: $(state_at $LX $LY $LZ)"
if [ "$SELFTEST" = 1 ] && [ "$SELFTEST_AT" = sensor ]; then
    fail "SELFTEST sensor: the matching block lit the lamp, expected it to stay dark (the inverted assertion proves the redstone case can fail)"
fi

echo "== AK2: a different block in front drops the signal back to 0 (differential) =="
set_front minecraft:dirt
sensor_read "$SX" "$SY" "$SZ"
echo "  $(state_at $LX $LY $LZ) / $REPLY"
expect 0 blocks minecraft:stone north
if lamp_lit "$LX" "$LY" "$LZ"; then fail "the lamp stayed lit with a non-matching block in front"; fi

echo "== AK3: putting the matching block back lights it again (reversible) =="
set_front minecraft:stone
sensor_read "$SX" "$SY" "$SZ"
echo "  $(state_at $LX $LY $LZ) / $REPLY"
expect 15 blocks minecraft:stone north
lamp_lit "$LX" "$LY" "$LZ" || fail "the lamp did not light again after restoring the matching block"

echo "== AK4: an unknown block id is refused and the setting is unchanged =="
# An `if` condition keeps the ERR trap from printing a FAIL for this expected refusal.
if refusal=$(ctl give rftoolsutility:not_a_sensor 1 2>&1); then rc=0; else rc=$?; fi
echo "  give: rc=$rc $refusal"
[ "$rc" -ne 0 ] || [[ "$refusal" == *"Unknown"* ]] || fail "an unknown item id was not refused: $refusal"
sensor_read "$SX" "$SY" "$SZ"
expect 15 blocks minecraft:stone north

echo "PASS: a sensor emits a real 15 redstone signal (a vanilla lamp lights) only while its named block is on the input side, 0 otherwise; the lamp's vanilla redstone-block control, the dirt differential, the restore and the unknown-id refusal all assert parsed values, and SELFTEST_AT=sensor inverts the match case."
