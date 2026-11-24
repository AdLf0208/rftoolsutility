#!/usr/bin/env bash
# Focused, fail-capable slice-10 leg: the destination analyzer.
# Reuses the exact helpers of tools/e2e-teleporter.sh (game built with 0.2.9).
# The artefact has no redstone emitter on the analyzer, so the assertion reads the
# harness's own analyzer readout (the brief's permitted fallback), not a wire's power.
export LC_ALL=C
[ -z "${RFU_DEBUG:-}" ] || set -x

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/rftoolsutility
CTL=${CTL:-}
GAME=${GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/rftoolsutility-e2e.sock}
SCRATCH_ROOT=${E2E_TMP:-/tmp/opencode}
SELFTEST=${SELFTEST:-0}
SELFTEST_AT=${SELFTEST_AT:-analyzer}
CONTROL=${CONTROL:-0}
KEEP=${KEEP:-0}
WORLD=RFToolsUtilityE2E
X0=200

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -n "$CTL" ] || { echo "set CTL= to the horizon-ctl of this pin (launcher-game.sh env)" >&2; exit 1; }
[ -n "$GAME" ] || { echo "set GAME= to a game built with this package" >&2; exit 1; }
if [ "$CONTROL" = 1 ]; then
    [ -n "${CONTROL_GAME:-}" ] || { echo "set CONTROL_GAME= to the unmodded Nightly game" >&2; exit 1; }
    GAME=$CONTROL_GAME
fi
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d "$SCRATCH_ROOT/rftoolsutility-home.XXXXXX")
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

block_name() { ctl block "$1" "$2" "$3" | jq -r '.block // empty'; }

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
set_energy() { control energy "$@"; }
set_dial() { control dial "$@"; }

analyzer() {
    NONCE=$((NONCE + 1))
    control analyzer "$1" "$2" "$3" "$NONCE"
    local line="$REPLY"
    A_SIGNAL=$(echo "$line" | sed -n 's/.*signal=\([0-9]*\).*/\1/p')
    A_STATUS=$(echo "$line" | sed -n 's/.*status=\([^ ]*\).*/\1/p')
    A_DIAL=$(echo "$line" | sed -n 's/.*dial=\([^ ]*\).*/\1/p')
    [ -n "$A_SIGNAL" ] && [ -n "$A_STATUS" ] || fail "the analyzer line did not parse: $line"
}
expect_signal() {
    [ "$A_SIGNAL" = "$1" ] || fail "the analyzer reads signal=$A_SIGNAL, expected $1 (status=$A_STATUS)"
    [ "$A_STATUS" = "$2" ] || fail "the analyzer reads status=$A_STATUS, expected $2"
}

TRANSMITTER_BLOCK=rftoolsutility:matter_transmitter
RECEIVER_BLOCK=rftoolsutility:matter_receiver
ANALYZER_BLOCK=rftoolsutility:destination_analyzer
TX_X=203; TX_Y=101; TX_Z=8
RX_X=203; RX_Y=101; RX_Z=12
AX_X=203; AX_Y=101; AX_Z=7
NONCE=0

launch
arena

echo "== AJ: a transmitter, a receiver and a passive analyzer are placed and read back =="
ctl command "gamemode creative" >/dev/null
ctl wait 6 >/dev/null
place_block "$TRANSMITTER_BLOCK" "$TX_X" "$TX_Y" "$TX_Z"
place_block "$RECEIVER_BLOCK" "$RX_X" "$RX_Y" "$RX_Z"
place_block "$ANALYZER_BLOCK" "$AX_X" "$AX_Y" "$AX_Z"
ctl wait 8 >/dev/null
[ "$(block_name "$AX_X" "$AX_Y" "$AX_Z")" = "$ANALYZER_BLOCK" ] || fail "the analyzer block is not there"

echo "== AJ1: dialed to an underpowered receiver the analyzer reads a low signal =="
set_dial "$TX_X" "$TX_Y" "$TX_Z" minecraft:the_end "$RX_X" "$RX_Y" "$RX_Z"
ctl wait 40 >/dev/null
analyzer "$AX_X" "$AX_Y" "$AX_Z"
echo "  underpowered: $REPLY"
expect_signal 0 warn

echo "== AJ2: charging the receiver to 5,000 FE raises the signal to 15 =="
set_energy "$RX_X" "$RX_Y" "$RX_Z" 5000
ctl wait 40 >/dev/null
analyzer "$AX_X" "$AX_Y" "$AX_Z"
echo "  charged: $REPLY"
if [ "${SELFTEST:-0}" = 1 ] && [ "${SELFTEST_AT:-}" = analyzer ]; then
    if [ "$A_SIGNAL" = 15 ]; then
        fail "SELFTEST analyzer: the charged receiver reads signal=$A_SIGNAL, expected it to stay low (the inverted assertion proves the charge case can fail)"
    fi
fi
expect_signal 15 ok

echo "== AJ3: a block above the receiver drops the signal back to 0 =="
ctl command "execute in minecraft:the_end run setblock $RX_X $((RX_Y + 1)) $RX_Z minecraft:stone" >/dev/null
# The analyzer re-checks every 20 ticks (game_time % 20 == 0); an 8-tick wait can read the
# previous check's 15. Wait a full period so the assertion sees the re-check, not setup latency.
ctl wait 40 >/dev/null
analyzer "$AX_X" "$AX_Y" "$AX_Z"
echo "  blocked: $REPLY"
expect_signal 0 warn

echo "PASS: a passive destination analyzer next to a dialed transmitter reads signal=0 (warn) while its receiver is underpowered, signal=15 (ok) after 5,000 FE, and signal=0 again when a block is placed above the receiver; SELFTEST_AT=analyzer inverts the charge case and fails."
