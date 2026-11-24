#!/usr/bin/env bash
# Focused, fail-capable slice-15 leg: the analog's three-input arithmetic and its exact 0..15
# redstone strength. A vanilla redstone lamp beside the analog is the physical assertion; a vanilla
# redstone block powers the same lamp first as the positive control. The input side (I) and the two
# comparison sides (A = left/clockwise, B = right/counterclockwise) are driven by redstone blocks
# placed with `setblock`; the six arithmetic settings are written by the GUI-equivalent command.
# The readout carries the game time of the last output change and a live read of the lamp, so every
# claim is made from parsed `ctl` replies and the game's own block state, never wall time or the
# absence of an error. Reuses the launcher-built game/ctl/resource exports (E2E_TMP, never /tmp).
export LC_ALL=C
[ -z "${RFU_DEBUG:-}" ] || set -x

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/rftoolsutility
CTL=${CTL:-}
GAME=${GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/rftoolsutility-analog.sock}
SCRATCH_ROOT=${E2E_TMP:-/tmp/opencode}
SELFTEST=${SELFTEST:-0}
SELFTEST_AT=${SELFTEST_AT:-analog}
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

SCRATCH=$(mktemp -d "$SCRATCH_ROOT/rftoolsutility-analog.XXXXXX")
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
signal_at() { state_at "$@" | sed -n 's/.*signal=\([0-9][0-9]*\).*/\1/p'; }

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
    ctl command "execute in minecraft:the_end run fill $((X0 - 2)) 100 -20 $((X0 + 6)) 100 20 minecraft:stone" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "execute in minecraft:the_end run fill $((X0 - 2)) 101 -20 $((X0 + 6)) 108 20 minecraft:air" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "execute in minecraft:the_end run kill @e[type=!player]" >/dev/null
    ctl command "execute in minecraft:the_end run tp @s $X0.5 101 0.5" >/dev/null
    ctl wait 20
    [ "$(block_name "$X0" 100 0)" = "minecraft:stone" ] || fail "the arena floor is not stone"
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
analog_read() {
    NONCE=$((NONCE + 1))
    ctl chat "/rftoolsutility analog $SX $SY $SZ $NONCE" >/dev/null
    ctl wait 1 >/dev/null
    REPLY=$(ctl messages 6 | jq -r '.chat[]' | grep -m1 "nonce=$NONCE" || true)
    [ -n "$REPLY" ] || fail "no analog reply for nonce $NONCE"
    echo "  $REPLY" >&2
    Q_INPUT=$(echo "$REPLY" | sed -n 's/.*input=\([0-9-]*\).*/\1/p')
    Q_A=$(echo "$REPLY" | sed -n 's/.* a=\([0-9-]*\).*/\1/p')
    Q_B=$(echo "$REPLY" | sed -n 's/.* b=\([0-9-]*\).*/\1/p')
    Q_BRANCH=$(echo "$REPLY" | sed -n 's/.*branch=\([a-z]*\).*/\1/p')
    Q_OUT=$(echo "$REPLY" | sed -n 's/.*output=\([0-9]*\).*/\1/p')
    Q_CHANGED=$(echo "$REPLY" | sed -n 's/.*changed=\([0-9-]*\).*/\1/p')
    Q_LAMP=$(echo "$REPLY" | sed -n 's/.*lamp=\([a-z]*\).*/\1/p')
    Q_TIME=$(echo "$REPLY" | sed -n 's/.*time=\([0-9]*\).*/\1/p')
    [ -n "$Q_INPUT" ] && [ -n "$Q_A" ] && [ -n "$Q_B" ] && [ -n "$Q_BRANCH" ] \
        && [ -n "$Q_OUT" ] && [ -n "$Q_CHANGED" ] && [ -n "$Q_LAMP" ] && [ -n "$Q_TIME" ] \
        || fail "the analog line did not parse: $REPLY"
}

setblock() { ctl command "execute in minecraft:the_end run setblock $1 $2 $3 $4" >/dev/null; }

# Write all six GuiAnalog fields plus the input side with one harness command.
configure() { # mul_eq mul_less mul_greater add_eq add_less add_greater
    ctl chat "/rftoolsutility analog-set $SX $SY $SZ $1 $2 $3 $4 $5 $6 north" >/dev/null
    ctl wait 12 >/dev/null
}

# Assert the reply's parsed strength, the game's own cached block state and the lamp agree.
expect_output() { # input a b branch output lamp
    [ "$Q_INPUT" = "$1" ] || fail "the analog reads input=$Q_INPUT, expected $1"
    [ "$Q_A" = "$2" ] || fail "the analog reads a=$Q_A, expected $2"
    [ "$Q_B" = "$3" ] || fail "the analog reads b=$Q_B, expected $3"
    [ "$Q_BRANCH" = "$4" ] || fail "the analog chose branch=$Q_BRANCH, expected $4"
    if [ "$SELFTEST" = 1 ] && [ "$SELFTEST_AT" = analog ] && [ "$5" = 7 ]; then
        # Invert the truncation assertion: the correct 0.5 * 15 -> 7 output must not read 7.
        [ "$Q_OUT" != 7 ] || fail "SELFTEST analog: the 0.5 * 15 output reads 7, expected it inverted (the strength assertion can fail)"
    fi
    [ "$Q_OUT" = "$5" ] || fail "the analog reads output=$Q_OUT, expected $5"
    local state_signal
    state_signal=$(signal_at "$SX" "$SY" "$SZ")
    [ "$state_signal" = "$5" ] || fail "the block state reads signal=$state_signal, expected $5"
    if [ "$6" = lit ]; then
        [ "$Q_LAMP" = lit ] || fail "the lamp reads $Q_LAMP in the reply, expected lit"
        lamp_lit "$LX" "$LY" "$LZ" || fail "the vanilla lamp is dark: $(state_at "$LX" "$LY" "$LZ")"
    else
        [ "$Q_LAMP" = dark ] || fail "the lamp reads $Q_LAMP in the reply, expected dark"
        if lamp_lit "$LX" "$LY" "$LZ"; then fail "the vanilla lamp is lit: $(state_at "$LX" "$LY" "$LZ")"; fi
    fi
}

SX=$((X0 + 2)); SY=101; SZ=0              # the analog
LX=$SX; LY=$SY; LZ=$((SZ + 1))             # the vanilla redstone lamp (south/back side)
PX=$SX; PY=$SY; PZ=$((LZ + 1))             # the vanilla redstone block (positive control, adjacent to the lamp)
IX=$SX; IY=$SY; IZ=$((SZ - 1))             # I: the north input side
AX=$((SX + 1)); AY=$SY; AZ=$SZ             # A: the left (east) side
BX=$((SX - 1)); BY=$SY; BZ=$SZ             # B: the right (west) side
NONCE=0

launch
arena

echo "== AO: an analog and a vanilla redstone lamp are placed and read back =="
ctl command "gamemode creative" >/dev/null
ctl wait 6 >/dev/null
place_block "rftoolsutility:analog" "$SX" "$SY" "$SZ"
setblock "$LX" "$LY" "$LZ" minecraft:redstone_lamp
ctl wait 8 >/dev/null
[ "$(block_name "$SX" "$SY" "$SZ")" = "rftoolsutility:analog" ] || fail "the analog block is not there"
[ "$(block_name "$LX" "$LY" "$LZ")" = "minecraft:redstone_lamp" ] || fail "the lamp block is not there"

echo "== AO-control: a vanilla redstone block lights the same lamp (control for the assertion) =="
setblock "$PX" "$PY" "$PZ" minecraft:redstone_block
ctl wait 20 >/dev/null
echo "  lamp with a vanilla redstone block: $(state_at $LX $LY $LZ)"
lamp_lit "$LX" "$LY" "$LZ" || fail "a vanilla redstone block did not light the lamp (the control cannot prove anything)"
setblock "$PX" "$PY" "$PZ" minecraft:air
ctl wait 20 >/dev/null
echo "  lamp after removing it: $(state_at $LX $LY $LZ)"
if lamp_lit "$LX" "$LY" "$LZ"; then fail "the lamp stayed lit after the redstone block was removed"; fi

echo "== AO0: the GUI's six arithmetic fields are command-driven and start at the identity =="
configure 1 1 1 0 0 0
analog_read
expect_output 0 0 0 eq 0 dark
[ "$Q_CHANGED" = -1 ] || fail "a fresh analog already has an output-change stamp: changed=$Q_CHANGED"

echo "== AO1: full input, equal sides -> 15 * 0.5 = 7 (Java (int) truncation) =="
setblock "$IX" "$IY" "$IZ" minecraft:redstone_block
configure 0.5 1 1 0 0 0
ctl wait 20 >/dev/null
analog_read
expect_output 15 0 0 eq 7 lit
CHANGE1=$Q_CHANGED
[ "$CHANGE1" -ge 0 ] || fail "no output-change stamp was written: changed=$CHANGE1"
[ "$Q_TIME" -ge "$CHANGE1" ] || fail "the change stamp $CHANGE1 is after the readout's time $Q_TIME"

echo "== AO2: the same input with the identity multiplier -> 15 (branch arithmetic applies) =="
configure 1 1 1 0 0 0
ctl wait 20 >/dev/null
analog_read
expect_output 15 0 0 eq 15 lit
[ "$Q_CHANGED" -gt "$CHANGE1" ] || fail "the change stamp did not advance: $Q_CHANGED <= $CHANGE1"

echo "== AO3: A > B selects mul_greater; the equal branch is set to zero as the differential =="
configure 0 1 1 0 0 0
setblock "$AX" "$AY" "$AZ" minecraft:redstone_block
ctl wait 20 >/dev/null
analog_read
expect_output 15 15 0 greater 15 lit
setblock "$AX" "$AY" "$AZ" minecraft:air

echo "== AO4: A < B selects mul_less; again the equal branch is zero =="
configure 0 1 0 0 0 0
setblock "$BX" "$BY" "$BZ" minecraft:redstone_block
ctl wait 20 >/dev/null
analog_read
expect_output 15 0 15 less 15 lit
setblock "$BX" "$BY" "$BZ" minecraft:air

echo "== AO5: a zero input is the differential (the lamp must go dark) =="
configure 1 1 1 0 0 0
setblock "$IX" "$IY" "$IZ" minecraft:air
ctl wait 20 >/dev/null
analog_read
expect_output 0 0 0 eq 0 dark

echo "== AO6: an offset below zero is clamped to no signal =="
configure 0 1 1 -5 0 0
setblock "$IX" "$IY" "$IZ" minecraft:redstone_block
ctl wait 20 >/dev/null
analog_read
expect_output 15 0 0 eq 0 dark

echo "== AO7: an unknown block id is refused =="
if refusal=$(ctl give rftoolsutility:not_an_analog 1 2>&1); then rc=0; else rc=$?; fi
echo "  give: rc=$rc $refusal"
[ "$rc" -ne 0 ] || [[ "$refusal" == *"Unknown"* ]] || fail "an unknown item id was not refused: $refusal"

echo "PASS: an analog applies the artefact's branch arithmetic to three redstone inputs and emits the exact clamped 0..15 strength (a vanilla lamp lights only when it is nonzero); the redstone-block control, the zero-input differential, the greater/less branch differentials, the clamping case, the game-time stamps and SELFTEST_AT=analog all assert parsed values."
