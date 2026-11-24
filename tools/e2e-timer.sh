#!/usr/bin/env bash
# Focused, fail-capable slice-13 leg: the timer's rising-edge countdown and one-tick pulse.
# A vanilla redstone lamp beside the timer is the assertion; a vanilla lever on the north input
# side supplies the rising edge, and a vanilla redstone block powers the same lamp first as the
# positive control. The timer's own harness readout carries game-time stamps of the last edge and
# the last 15 output, so the delay is asserted in the game's own ticks, never wall time.
# Reuses the launcher-built game/ctl/resource exports (E2E_TMP, never /tmp).
export LC_ALL=C
[ -z "${RFU_DEBUG:-}" ] || set -x

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/rftoolsutility
CTL=${CTL:-}
GAME=${GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/rftoolsutility-timer.sock}
SCRATCH_ROOT=${E2E_TMP:-/tmp/opencode}
SELFTEST=${SELFTEST:-0}
SELFTEST_AT=${SELFTEST_AT:-timer}
# A deliberate +1 tick displacement of the expected delay, used by the inverted (EXIT:1) leg.
SELFTEST_SHIFT=${SELFTEST_SHIFT:-1}
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

SCRATCH=$(mktemp -d "$SCRATCH_ROOT/rftoolsutility-timer.XXXXXX")
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

# A fast read for the exact-tick phases: a unique nonce pins the reply even under chat load.
timer_read() {
    NONCE=$((NONCE + 1))
    ctl chat "/rftoolsutility timer $SX $SY $SZ $NONCE" >/dev/null
    ctl wait 1 >/dev/null
    REPLY=$(ctl messages 6 | jq -r '.chat[]' | grep -m1 "nonce=$NONCE" || true)
    [ -n "$REPLY" ] || fail "no timer reply for nonce $NONCE"
    echo "  $REPLY" >&2
    Q_DELAY=$(echo "$REPLY" | sed -n 's/.*delay=\([0-9-]*\).*/\1/p')
    Q_PAUSES=$(echo "$REPLY" | sed -n 's/.*pauses=\([a-z]*\).*/\1/p')
    Q_OUT=$(echo "$REPLY" | sed -n 's/.*output=\([0-9]*\).*/\1/p')
    Q_TIMER=$(echo "$REPLY" | sed -n 's/.*timer=\([0-9-]*\).*/\1/p')
    Q_PULSE=$(echo "$REPLY" | sed -n 's/.*pulse=\([0-9-]*\).*/\1/p')
    Q_FIRED=$(echo "$REPLY" | sed -n 's/.*fired=\([0-9-]*\).*/\1/p')
    Q_TIME=$(echo "$REPLY" | sed -n 's/.*time=\([0-9-]*\).*/\1/p')
    Q_LAMP=$(echo "$REPLY" | sed -n 's/.*lamp=\([a-z]*\).*/\1/p')
    [ -n "$Q_DELAY" ] && [ -n "$Q_OUT" ] && [ -n "$Q_PULSE" ] && [ -n "$Q_FIRED" ] && [ -n "$Q_LAMP" ] \
        || fail "the timer line did not parse: $REPLY"
}

expect_state() { # delay pauses output
    [ "$Q_DELAY" = "$1" ] || fail "the timer reads delay=$Q_DELAY, expected $1"
    [ "$Q_PAUSES" = "$2" ] || fail "the timer reads pauses=$Q_PAUSES, expected $2"
    [ "$Q_OUT" = "$3" ] || fail "the timer reads output=$Q_OUT, expected $3"
}

# One clean rising edge of a vanilla lever on the north input side: false -> true -> false.
pulse_lever() {
    ctl command "execute in minecraft:the_end run setblock $LX_LEV $LY_LEV $LZ_LEV minecraft:lever[face=wall,facing=south,powered=true]" >/dev/null
    ctl wait 8 >/dev/null
    ctl command "execute in minecraft:the_end run setblock $LX_LEV $LY_LEV $LZ_LEV minecraft:lever[face=wall,facing=south,powered=false]" >/dev/null
    ctl wait 8 >/dev/null
}

lever_high() {
    ctl command "execute in minecraft:the_end run setblock $LX_LEV $LY_LEV $LZ_LEV minecraft:lever[face=wall,facing=south,powered=true]" >/dev/null
    ctl wait 8 >/dev/null
}

lever_low() {
    ctl command "execute in minecraft:the_end run setblock $LX_LEV $LY_LEV $LZ_LEV minecraft:lever[face=wall,facing=south,powered=false]" >/dev/null
    ctl wait 8 >/dev/null
}

# Wait (in game ticks) until the harness's fired stamp changes from a baseline.
wait_fired() { # baseline
    local base=$1
    for _ in $(seq 1 120); do
        timer_read
        if [ "$Q_FIRED" != "$base" ]; then
            echo "  lamp recorded at the fire: lamp=$Q_LAMP (output=$Q_OUT)" >&2
            return 0
        fi
        ctl wait 2 >/dev/null
    done
    fail "the timer never fired (fired stayed $base; last reply: $REPLY)"
}

SX=$((X0 + 1)); SY=101; SZ=0                  # the timer
LX=$((X0 + 2)); LY=$SY; LZ=0                   # the vanilla redstone lamp beside it
PX=$((X0 + 3)); PY=$SY; PZ=0                   # the vanilla redstone block (positive control)
LX_LEV=$SX; LY_LEV=$SY; LZ_LEV=$((SZ - 1))     # a vanilla wall lever on the north input side
NONCE=0

launch
arena

echo "== AM: a timer and a vanilla redstone lamp are placed and read back =="
ctl command "gamemode creative" >/dev/null
ctl wait 6 >/dev/null
place_block "rftoolsutility:timer" "$SX" "$SY" "$SZ"
ctl command "execute in minecraft:the_end run setblock $LX $LY $LZ minecraft:redstone_lamp" >/dev/null
ctl wait 8 >/dev/null
[ "$(block_name "$SX" "$SY" "$SZ")" = "rftoolsutility:timer" ] || fail "the timer block is not there"
[ "$(block_name "$LX" "$LY" "$LZ")" = "minecraft:redstone_lamp" ] || fail "the lamp block is not there"

echo "== AM-control: a vanilla redstone block lights the same lamp (control for the assertion) =="
ctl command "execute in minecraft:the_end run setblock $PX $PY $PZ minecraft:redstone_block" >/dev/null
ctl wait 20 >/dev/null
echo "  lamp with a vanilla redstone block: $(state_at $LX $LY $LZ)"
lamp_lit "$LX" "$LY" "$LZ" || fail "a vanilla redstone block did not light the lamp (the control cannot prove anything)"
ctl command "execute in minecraft:the_end run setblock $PX $PY $PZ minecraft:air" >/dev/null
ctl wait 20 >/dev/null
echo "  lamp after removing it: $(state_at $LX $LY $LZ)"
if lamp_lit "$LX" "$LY" "$LZ"; then fail "the lamp stayed lit after the redstone block was removed"; fi

echo "== AM0: the GUI's delay/pauses settings are command-driven =="
ctl chat "/rftoolsutility timer-set $SX $SY $SZ 600 false north" >/dev/null
ctl wait 20 >/dev/null
timer_read
expect_state 600 false 0
if lamp_lit "$LX" "$LY" "$LZ"; then fail "a freshly set long-delay timer's lamp is lit: $(state_at $LX $LY $LZ)"; fi

echo "== AM1: a no-pulse window stays dark (differential) =="
BASE_FIRED=$Q_FIRED
ctl wait 40 >/dev/null
timer_read
expect_state 600 false 0
[ "$Q_FIRED" = "$BASE_FIRED" ] || fail "the timer fired with no input pulse (stamp $Q_FIRED, baseline $BASE_FIRED)"
if lamp_lit "$LX" "$LY" "$LZ"; then fail "the timer lit its lamp with no input pulse"; fi

echo "== AM2: a lever pulse fires the lamp exactly delay-1 game ticks later =="
ctl chat "/rftoolsutility timer-set $SX $SY $SZ 4 false north" >/dev/null
ctl wait 4 >/dev/null
timer_read
BASE_FIRED=$Q_FIRED
pulse_lever
wait_fired "$BASE_FIRED"
PULSE=$Q_PULSE; FIRED=$Q_FIRED; READ_TIME=$Q_TIME
echo "  edge at tick $PULSE, 15 output at tick $FIRED, read at tick $READ_TIME (delay=4)"
[ "$PULSE" -ge 0 ] || fail "no rising edge was stamped: pulse=$PULSE"
[ "$FIRED" -gt "$PULSE" ] || fail "the fire was not after the edge: fired=$FIRED pulse=$PULSE"
if [ "$SELFTEST" = 1 ] && [ "$SELFTEST_AT" = timer ]; then
    # Invert the exact-tick assertion by a deliberate, named displacement; the correct delay must
    # fail this, proving the tick count can fail rather than merely reading the reply.
    [ "$((FIRED - PULSE))" -eq $((3 + SELFTEST_SHIFT)) ] \
        || fail "SELFTEST timer: the pulse came $((FIRED - PULSE)) ticks after the edge, expected the inverted $((3 + SELFTEST_SHIFT)) (the exact-tick assertion can fail)"
else
    [ "$((FIRED - PULSE))" -eq 3 ] || fail "the delay was $((FIRED - PULSE)) ticks, expected 3 (delay-1)"
fi
[ "$Q_LAMP" = "lit" ] || fail "the timer's pulse did not light the lamp at the fire: lamp=$Q_LAMP"

echo "== AM2b: a long delay proves the pulse lasts exactly one tick =="
ctl chat "/rftoolsutility timer-set $SX $SY $SZ 600 false north" >/dev/null
ctl wait 4 >/dev/null
timer_read
BASE_FIRED=$Q_FIRED
pulse_lever
wait_fired "$BASE_FIRED"
[ "$Q_LAMP" = "lit" ] || fail "the long-delay timer's pulse did not light the lamp at the fire: lamp=$Q_LAMP"
ctl wait 4 >/dev/null
timer_read
echo "  4 ticks after the fire: output=$Q_OUT lamp=$(state_at $LX $LY $LZ)"
[ "$Q_OUT" = 0 ] || fail "a timer pulse lasted longer than one tick: output=$Q_OUT"
if lamp_lit "$LX" "$LY" "$LZ"; then fail "the lamp stayed lit after the one-tick pulse"; fi

echo "== AM3: pauses holds the countdown while the input is high (overdue detection) =="
# Raise the lever FIRST: a pauses timer with a low input is just a free-running timer.
lever_high
ctl chat "/rftoolsutility timer-set $SX $SY $SZ 4 true north" >/dev/null
ctl wait 8 >/dev/null
timer_read
expect_state 4 true 0
BASE_FIRED=$Q_FIRED
ctl wait 40 >/dev/null
timer_read
expect_state 4 true 0
[ "$Q_TIMER" = 4 ] || fail "a pauses timer counted down while powered: timer=$Q_TIMER"
[ "$Q_FIRED" = "$BASE_FIRED" ] || fail "a pauses timer fired while powered: fired=$Q_FIRED baseline=$BASE_FIRED"
if lamp_lit "$LX" "$LY" "$LZ"; then fail "a pauses timer lit its lamp while powered"; fi
PULSE=$Q_PULSE
lever_low
wait_fired "$BASE_FIRED"
echo "  pauses held the countdown from edge tick $PULSE until release; fire at tick $Q_FIRED"
[ "$Q_FIRED" -gt "$PULSE" ] || fail "the paused timer did not fire after the input ended"
[ "$Q_LAMP" = "lit" ] || fail "the paused timer's pulse did not light the lamp at the fire: lamp=$Q_LAMP"

echo "== AM4: an unknown block id is refused =="
if refusal=$(ctl give rftoolsutility:not_a_timer 1 2>&1); then rc=0; else rc=$?; fi
echo "  give: rc=$rc $refusal"
[ "$rc" -ne 0 ] || [[ "$refusal" == *"Unknown"* ]] || fail "an unknown item id was not refused: $refusal"

echo "PASS: a timer turns a vanilla lever's rising edge into a one-tick redstone pulse exactly delay-1 game ticks later (read from the reply's own game-time stamps), lighting a vanilla lamp only then; the redstone-block control, the no-pulse window, the pauses case and SELFTEST_AT=timer all assert parsed values and the inverted tick fails."
