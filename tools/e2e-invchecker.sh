#!/usr/bin/env bash
# Focused, fail-capable slice-17 leg: the inventory checker reads a vanilla chest's live item
# slots on its input side and emits the artefact's declared 15/0. A vanilla redstone lamp beside
# the checker is the physical assertion; a vanilla redstone block powers the same lamp first as the
# positive control. The chest is seeded with a known stack through `ctl container`; the amount,
# slot, matcher, damage mode, tag and input side are written by the GUI-equivalent command. The
# readout carries the neighbour slot, the parsed threshold, the check's verdict, the game time of
# the last change and a live read of the lamp, so every claim comes from a parsed `ctl` reply and
# the game's own block state, never wall time or the absence of an error. Reuses the launcher-built
# game/ctl/resource exports (E2E_TMP, never /tmp).
export LC_ALL=C
[ -z "${RFU_DEBUG:-}" ] || set -x

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/rftoolsutility
CTL=${CTL:-}
GAME=${GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/rftoolsutility-invchecker.sock}
SCRATCH_ROOT=${E2E_TMP:-/tmp/opencode}
SELFTEST=${SELFTEST:-0}
SELFTEST_AT=${SELFTEST_AT:-invchecker}
CONTROL=${CONTROL:-0}
KEEP=${KEEP:-0}
WORLD=RFToolsUtilityE2E
X0=200

[ -n "$CTL" ] || { echo "set CTL= to this pin's horizon-ctl (launcher-game.sh env)" >&2; exit 1; }
[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -n "$GAME" ] || { echo "set GAME= to a game built with this package" >&2; exit 1; }
if [ "$CONTROL" = 1 ]; then
    [ -n "${CONTROL_GAME:-}" ] || { echo "set CONTROL_GAME= to the unmodded Nightly" >&2; exit 1; }
    GAME=$CONTROL_GAME
fi
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d "$SCRATCH_ROOT/rftoolsutility-invchecker.XXXXXX")
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
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
inv_read() {
    NONCE=$((NONCE + 1))
    ctl chat "/rftoolsutility invchecker $SX $SY $SZ $NONCE" >/dev/null
    ctl wait 1 >/dev/null
    REPLY=$(ctl messages 6 | jq -r '.chat[]' | grep -m1 "nonce=$NONCE" || true)
    [ -n "$REPLY" ] || fail "no invchecker reply for nonce $NONCE"
    echo "  $REPLY" >&2
    Q_SEEN=$(echo "$REPLY" | sed -n 's/.*seen=\([a-z_:0-9-]*\).*/\1/p')
    Q_COUNT=$(echo "$REPLY" | sed -n 's/.*count=\([0-9]*\).*/\1/p')
    Q_AMOUNT=$(echo "$REPLY" | sed -n 's/.*amount=\([0-9-]*\).*/\1/p')
    Q_SLOT=$(echo "$REPLY" | sed -n 's/.*slot=\([0-9-]*\).*/\1/p')
    Q_MATCHED=$(echo "$REPLY" | sed -n 's/.*matched=\([a-z]*\).*/\1/p')
    Q_OUT=$(echo "$REPLY" | sed -n 's/.*output=\([0-9]*\).*/\1/p')
    Q_CHANGED=$(echo "$REPLY" | sed -n 's/.*changed=\([0-9-]*\).*/\1/p')
    Q_LAMP=$(echo "$REPLY" | sed -n 's/.*lamp=\([a-z]*\).*/\1/p')
    Q_TIME=$(echo "$REPLY" | sed -n 's/.*time=\([0-9]*\).*/\1/p')
    for field in Q_SEEN Q_COUNT Q_AMOUNT Q_SLOT Q_MATCHED Q_OUT Q_CHANGED Q_LAMP Q_TIME; do
        [ -n "${!field}" ] || fail "the invchecker line did not parse ($field): $REPLY"
    done
}

setblock() { ctl command "execute in minecraft:the_end run setblock $1 $2 $3 $4" >/dev/null; }

# Write all four `IncCheckerData` fields, the ghost matcher and the input side with one command.
configure() { # amount slot damage matcher mdamage tag facing
    ctl chat "/rftoolsutility invchecker-set $SX $SY $SZ $1 $2 $3 $4 $5 $6 $7" >/dev/null
    ctl wait 14 >/dev/null
}

# Assert the reply's parsed count/threshold/verdict/strength, the game's own cached block state and
# the lamp agree. The stack-of-1/amount-2 pair is the differential the brief mandates.
expect_output() { # count amount slot matched output lamp
    [ "$Q_COUNT" = "$1" ] || fail "the checker sees count=$Q_COUNT, expected $1"
    [ "$Q_AMOUNT" = "$2" ] || fail "the checker reads amount=$Q_AMOUNT, expected $2"
    [ "$Q_SLOT" = "$3" ] || fail "the checker reads slot=$Q_SLOT, expected $3"
    [ "$Q_MATCHED" = "$4" ] || fail "the checker reads matched=$Q_MATCHED, expected $4"
    if [ "$SELFTEST" = 1 ] && [ "$SELFTEST_AT" = invchecker ] && [ "$5" = 15 ]; then
        [ "$Q_OUT" != 15 ] || fail "SELFTEST invchecker: the satisfied check reads 15, expected it inverted (the count/threshold assertion can fail)"
    fi
    [ "$Q_OUT" = "$5" ] || fail "the checker reads output=$Q_OUT, expected $5"
    local state_signal want_state=0
    state_signal=$(signal_at "$SX" "$SY" "$SZ")
    [ "$5" = 15 ] && want_state=1
    [ "$state_signal" = "$want_state" ] || fail "the block state reads signal=$state_signal, expected $want_state for output $5"
    if [ "$6" = lit ]; then
        [ "$Q_LAMP" = lit ] || fail "the lamp reads $Q_LAMP in the reply, expected lit"
        lamp_lit "$LX" "$LY" "$LZ" || fail "the vanilla lamp is dark: $(state_at "$LX" "$LY" "$LZ")"
    else
        [ "$Q_LAMP" = dark ] || fail "the lamp reads $Q_LAMP in the reply, expected dark"
        if lamp_lit "$LX" "$LY" "$LZ"; then fail "the vanilla lamp is lit: $(state_at "$LX" "$LY" "$LZ")"; fi
    fi
}

SX=$((X0 + 2)); SY=101; SZ=0              # the inventory checker
CX=$SX; CY=$SY; CZ=$((SZ - 1))            # the vanilla chest on the input (north) side
LX=$SX; LY=$SY; LZ=$((SZ + 1))             # the vanilla redstone lamp (south/back side)
PX=$LX; PY=$LY; PZ=$((LZ + 1))             # the vanilla redstone block (positive control, by the lamp)
NONCE=0

launch
arena

echo "== IV: a checker, a vanilla chest and a vanilla redstone lamp are placed and read back =="
ctl command "gamemode creative" >/dev/null
ctl wait 6 >/dev/null
place_block "rftoolsutility:invchecker" "$SX" "$SY" "$SZ"
setblock "$CX" "$CY" "$CZ" minecraft:chest
setblock "$LX" "$LY" "$LZ" minecraft:redstone_lamp
ctl wait 8 >/dev/null
[ "$(block_name "$SX" "$SY" "$SZ")" = "rftoolsutility:invchecker" ] || fail "the invchecker block is not there"
[ "$(block_name "$CX" "$CY" "$CZ")" = "minecraft:chest" ] || fail "the chest block is not there"
[ "$(block_name "$LX" "$LY" "$LZ")" = "minecraft:redstone_lamp" ] || fail "the lamp block is not there"

echo "== IV-control: a vanilla redstone block lights the same lamp (control for the assertion) =="
setblock "$PX" "$PY" "$PZ" minecraft:redstone_block
ctl wait 20 >/dev/null
echo "  lamp with a vanilla redstone block: $(state_at $LX $LY $LZ)"
lamp_lit "$LX" "$LY" "$LZ" || fail "a vanilla redstone block did not light the lamp (the control cannot prove anything)"
setblock "$PX" "$PY" "$PZ" minecraft:air
ctl wait 20 >/dev/null
echo "  lamp after removing it: $(state_at $LX $LY $LZ)"
if lamp_lit "$LX" "$LY" "$LZ"; then fail "the lamp stayed lit after the redstone block was removed"; fi

echo "== IV0: a single stack with the default amount=1 satisfies the checker =="
ctl container "$CX" "$CY" "$CZ" minecraft:stone 1 >/dev/null
ctl wait 4 >/dev/null
echo "  chest slot 0: $(ctl container "$CX" "$CY" "$CZ" | jq -c '.slots[0]')"
configure 1 0 Ignore - 0 - north
inv_read
expect_output 1 1 0 true 15 lit
CHANGE1=$Q_CHANGED
[ "$CHANGE1" -ge 0 ] || fail "no output-change stamp was written: changed=$CHANGE1"
[ "$Q_TIME" -ge "$CHANGE1" ] || fail "the change stamp $CHANGE1 is after the readout's time $Q_TIME"
[ "$Q_SEEN" = minecraft:stone ] || fail "the checker saw '$Q_SEEN' in the chest, expected minecraft:stone"

echo "== IV1: the same stack against amount=2 is the 0 differential =="
configure 2 0 Ignore - 0 - north
inv_read
expect_output 1 2 0 false 0 dark
[ "$Q_CHANGED" -gt "$CHANGE1" ] || fail "the change stamp did not advance: $Q_CHANGED <= $CHANGE1"

echo "== IV2: a matcher for another item refuses the otherwise sufficient stack =="
configure 1 0 Ignore dirt 0 - north
inv_read
expect_output 1 1 0 false 0 dark

echo "== IV3: matching the stack's item restores 15 (the matcher differential) =="
configure 1 0 Ignore stone 0 - north
inv_read
expect_output 1 1 0 true 15 lit

echo "== IV4: an empty configured slot reads nothing and drives 0 =="
configure 1 1 Ignore - 0 - north
inv_read
expect_output 0 1 1 false 0 dark

echo "== IV5: a larger stack satisfies amount=2 (count >= amount, not a fixed 1) =="
# Replace the chest first: `ctl container` inserts into the first free slot, so an add on a
# half-full chest lands in slot 1 rather than stacking onto slot 0.
setblock "$CX" "$CY" "$CZ" minecraft:air
ctl wait 4 >/dev/null
setblock "$CX" "$CY" "$CZ" minecraft:chest
ctl wait 4 >/dev/null
ctl container "$CX" "$CY" "$CZ" minecraft:stone 3 >/dev/null
ctl wait 4 >/dev/null
echo "  chest slots: $(ctl container "$CX" "$CY" "$CZ" | jq -c '.slots')"
configure 2 0 Ignore - 0 - north
inv_read
[ "$Q_COUNT" -ge 2 ] || fail "the chest does not hold at least 2 stone: count=$Q_COUNT"
expect_output "$Q_COUNT" 2 0 true 15 lit

echo "== IV6: an unknown block id is refused =="
if refusal=$(ctl give rftoolsutility:not_an_invchecker 1 2>&1); then rc=0; else rc=$?; fi
echo "  give: rc=$rc $refusal"
[ "$rc" -ne 0 ] || [[ "$refusal" == *"Unknown"* ]] || fail "an unknown item id was not refused: $refusal"

echo "PASS: an inventory checker reads a vanilla chest's live slots through the game's ITEMS capability and emits the artefact's declared 15 when the configured slot holds at least amount matching items and 0 otherwise; the stack-1/amount-2 differential, the matcher and empty-slot differentials, the larger stack, the redstone-block lamp control, the game-time stamps and SELFTEST_AT=invchecker all assert parsed values."
