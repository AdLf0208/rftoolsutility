#!/usr/bin/env bash
# Focused, fail-capable slice-19 leg: the wire's exact directional pass-through. A run of
# `rftoolsutility:wire` blocks carries the input-side neighbour's strength to the output side;
# a vanilla redstone lamp at the far end is the physical assertion. A vanilla redstone block next
# to the same lamp is the positive control; an air gap and a reversed wire are the broken-link and
# directional differentials. Every claim is made from parsed `ctl` replies, the game's own block
# state and game-time stamps, never wall time or the absence of an error. Reuses the launcher-built
# game/ctl/resource exports (E2E_TMP, never /tmp).
export LC_ALL=C
[ -z "${RFU_DEBUG:-}" ] || set -x

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/rftoolsutility
CTL=${CTL:-}
GAME=${GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/rftoolsutility-wire.sock}
SCRATCH_ROOT=${E2E_TMP:-/tmp/opencode}
SELFTEST=${SELFTEST:-0}
SELFTEST_AT=${SELFTEST_AT:-wire}
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

SCRATCH=$(mktemp -d "$SCRATCH_ROOT/rftoolsutility-wire.XXXXXX")
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

setblock() { ctl command "execute in minecraft:the_end run setblock $1 $2 $3 $4" >/dev/null; }

# A unique nonce pins the reply under chat load; parse its own fields.
wire_read() { # x y z
    NONCE=$((NONCE + 1))
    ctl chat "/rftoolsutility wire $1 $2 $3 $NONCE" >/dev/null
    ctl wait 1 >/dev/null
    REPLY=$(ctl messages 6 | jq -r '.chat[]' | grep -m1 "nonce=$NONCE" || true)
    [ -n "$REPLY" ] || fail "no wire reply for nonce $NONCE at $1,$2,$3"
    echo "  $REPLY" >&2
    Q_INPUT=$(echo "$REPLY" | sed -n 's/.*input=\([0-9-]*\).*/\1/p')
    Q_POWER=$(echo "$REPLY" | sed -n 's/.*power=\([0-9-]*\).*/\1/p')
    Q_OUTPUT=$(echo "$REPLY" | sed -n 's/.*output=\([0-9-]*\).*/\1/p')
    Q_CHANGED=$(echo "$REPLY" | sed -n 's/.*changed=\([0-9-]*\).*/\1/p')
    Q_FORWARD=$(echo "$REPLY" | sed -n 's/.*forward=\(\[[^]]*\]\).*/\1/p')
    Q_LAMP=$(echo "$REPLY" | sed -n 's/.*lamp=\([a-z]*\).*/\1/p')
    Q_FACING=$(echo "$REPLY" | sed -n 's/.*facing=\([a-z]*\).*/\1/p')
    Q_TIME=$(echo "$REPLY" | sed -n 's/.*time=\([0-9]*\).*/\1/p')
    [ -n "$Q_INPUT" ] && [ -n "$Q_POWER" ] && [ -n "$Q_OUTPUT" ] && [ -n "$Q_CHANGED" ] \
        && [ -n "$Q_FORWARD" ] && [ -n "$Q_LAMP" ] && [ -n "$Q_FACING" ] && [ -n "$Q_TIME" ] \
        || fail "the wire line did not parse: $REPLY"
    [ "$Q_POWER" = "$Q_OUTPUT" ] || fail "the wire's power and output disagree in the reply: $REPLY"
}

face_north() { # x y z
    ctl chat "/rftoolsutility wire-set $1 $2 $3 north" >/dev/null
    ctl wait 2 >/dev/null
}
face_south() { # x y z
    ctl chat "/rftoolsutility wire-set $1 $2 $3 south" >/dev/null
    ctl wait 2 >/dev/null
}

# The run: six wires from z=-3 to z=2, each reading its north neighbour and feeding south.
SX=$((X0 + 2)); SY=101
FIRST=-3; LAST=2
LX=$SX; LY=$SY; LZ=$((LAST + 1))    # the vanilla redstone lamp, south of the last wire
PX=$((SX + 1)); PY=$SY; PZ=$LZ      # the positive-control redstone block, east of the lamp
NONCE=0

launch
arena

echo "== W: a wire run and a vanilla redstone lamp are placed and read back =="
ctl command "gamemode creative" >/dev/null
ctl wait 6 >/dev/null
for z in $(seq "$FIRST" "$LAST"); do
    place_block "rftoolsutility:wire" "$SX" "$SY" "$z"
    face_north "$SX" "$SY" "$z"
done
setblock "$LX" "$LY" "$LZ" minecraft:redstone_lamp
ctl wait 20 >/dev/null
for z in $(seq "$FIRST" "$LAST"); do
    [ "$(block_name "$SX" "$SY" "$z")" = "rftoolsutility:wire" ] || fail "wire at z=$z is not there"
done
[ "$(block_name "$LX" "$LY" "$LZ")" = "minecraft:redstone_lamp" ] || fail "the lamp block is not there"
SZ=$FIRST
wire_read "$SX" "$SY" "$SZ"
[ "$Q_INPUT" = 0 ] && [ "$Q_POWER" = 0 ] && [ "$Q_CHANGED" = -1 ] \
    || fail "a fresh, unfed wire run already has a signal: $REPLY"
[ "$Q_FACING" = north ] || fail "wire-set did not save the north input side: $REPLY"
[ "$Q_FORWARD" = "[$SX, $SY, $((SZ + 1))]" ] \
    || fail "the wire does not forward to the side opposite its input: $REPLY"
if lamp_lit "$LX" "$LY" "$LZ"; then fail "the far lamp is lit with no source"; fi

echo "== W-control: a vanilla redstone block lights the same lamp (control for the assertion) =="
setblock "$PX" "$PY" "$PZ" minecraft:redstone_block
ctl wait 20 >/dev/null
echo "  lamp with a vanilla redstone block: $(state_at $LX $LY $LZ)"
lamp_lit "$LX" "$LY" "$LZ" || fail "a vanilla redstone block did not light the lamp (the control cannot prove anything)"
setblock "$PX" "$PY" "$PZ" minecraft:air
ctl wait 20 >/dev/null
echo "  lamp after removing it: $(state_at $LX $LY $LZ)"
if lamp_lit "$LX" "$LY" "$LZ"; then fail "the lamp stayed lit after the redstone block was removed"; fi

echo "== W1: a redstone block at the north end lights the lamp at the far end =="
setblock "$SX" "$SY" "$((FIRST - 1))" minecraft:redstone_block
ctl wait 30 >/dev/null
for z in $(seq "$FIRST" "$LAST"); do
    SZ=$z
    wire_read "$SX" "$SY" "$SZ"
    [ "$Q_INPUT" = 15 ] || fail "the wire at z=$z reads input=$Q_INPUT, expected 15: $REPLY"
    [ "$Q_POWER" = 15 ] || fail "the wire at z=$z passes output=$Q_POWER, expected 15: $REPLY"
    [ "$Q_CHANGED" -ge 0 ] || fail "no output-change stamp was written at z=$z: $REPLY"
    [ "$Q_TIME" -ge "$Q_CHANGED" ] || fail "the change stamp is after the readout's time: $REPLY"
    SIGNAL=$(signal_at "$SX" "$SY" "$SZ")
    [ "$SIGNAL" = 15 ] || fail "the block state reads signal=$SIGNAL at z=$z, expected 15"
done
if [ "$SELFTEST" = 1 ] && [ "$SELFTEST_AT" = wire ]; then
    # Invert the far-end lamp assertion: the fed run must not light it.
    if lamp_lit "$LX" "$LY" "$LZ"; then
        fail "SELFTEST wire: the fed run lit the far lamp, expected it to stay dark"
    fi
fi
lamp_lit "$LX" "$LY" "$LZ" || fail "the far lamp is dark though the whole run reads 15: $(state_at $LX $LY $LZ)"
SZ=$LAST
wire_read "$SX" "$SY" "$SZ"
[ "$Q_LAMP" = lit ] || fail "the far wire's live lamp read is $Q_LAMP, expected lit"
CHANGE_W1=$Q_CHANGED

echo "== W2: a broken link keeps the far lamp dark =="
setblock "$SX" "$SY" 0 minecraft:air
ctl wait 30 >/dev/null
SZ=1
wire_read "$SX" "$SY" "$SZ"
[ "$Q_INPUT" = 0 ] || fail "the wire after the gap reads input=$Q_INPUT, expected 0: $REPLY"
[ "$Q_POWER" = 0 ] || fail "the wire after the gap passes output=$Q_POWER, expected 0: $REPLY"
if lamp_lit "$LX" "$LY" "$LZ"; then fail "the far lamp is lit through a broken link"; fi
place_block "rftoolsutility:wire" "$SX" "$SY" 0
face_north "$SX" "$SY" 0
ctl wait 30 >/dev/null
lamp_lit "$LX" "$LY" "$LZ" || fail "the far lamp did not relight when the link was restored"

echo "== W3: a single wire's input side decides which side the lamp sees =="
# A lone wire avoids the iron-block template's redstone conduction: `LevelRef::signal` adds
# conducted power after the redstone hook, so a chained run's reads are not a directional oracle.
# The lamp reads the wire's own weak signal directly, which is the artefact's getRedstoneOutput.
DX=$((SX + 3)); DY=$SY; DZ=8
DLX=$DX; DLY=$DY; DLZ=$((DZ + 1))
place_block "rftoolsutility:wire" "$DX" "$DY" "$DZ"
face_north "$DX" "$DY" "$DZ"
setblock "$DLX" "$DLY" "$DLZ" minecraft:redstone_lamp
setblock "$DX" "$DY" "$((DZ - 1))" minecraft:redstone_block
ctl wait 30 >/dev/null
wire_read "$DX" "$DY" "$DZ"
[ "$Q_POWER" = 15 ] || fail "a north-facing wire fed from the north passes $Q_POWER, expected 15: $REPLY"
[ "$Q_FACING" = north ] || fail "wire-set did not save the north input side: $REPLY"
[ "$Q_FORWARD" = "[$DX, $DY, $((DZ + 1))]" ] \
    || fail "the north-facing wire forwards to the wrong side: $REPLY"
lamp_lit "$DLX" "$DLY" "$DLZ" || fail "a north-facing wire did not light the lamp on its output side"

face_south "$DX" "$DY" "$DZ"
ctl wait 30 >/dev/null
wire_read "$DX" "$DY" "$DZ"
[ "$Q_POWER" = 0 ] || fail "a south-facing wire with its source to the north passes $Q_POWER, expected 0: $REPLY"
[ "$Q_FACING" = south ] || fail "wire-set did not save the south input side: $REPLY"
[ "$Q_FORWARD" = "[$DX, $DY, $((DZ - 1))]" ] \
    || fail "the south-facing wire forwards to the wrong side: $REPLY"
[ "$(signal_at "$DX" "$DY" "$DZ")" = 0 ] || fail "the reversed wire's cached signal state is not 0"
if lamp_lit "$DLX" "$DLY" "$DLZ"; then fail "a south-facing wire lit the lamp on the wrong side"; fi

echo "== W4: an unknown block id is refused =="
if refusal=$(ctl give rftoolsutility:not_a_wire 1 2>&1); then rc=0; else rc=$?; fi
echo "  give: rc=$rc $refusal"
[ "$rc" -ne 0 ] || [[ "$refusal" == *"Unknown"* ]] || fail "an unknown item id was not refused: $refusal"

echo "PASS: a run of rftoolsutility wires passes the input-side neighbour's exact 15 to the output side only; the far lamp lights at the declared strength, an air gap and a reversed wire each darken it, the redstone-block control proves the lamp and its parser work, every reply and game-time stamp is parsed, and SELFTEST_AT=wire ends EXIT:1."
