#!/usr/bin/env bash
# Focused, fail-capable slice-18 leg: the redstone transmitter writes its input-side strength to a
# world-persistent numbered channel and a receiver emits the artefact's declared strength (15 in
# digital mode, the exact 0..15 in analog mode). A vanilla lamp beside each receiver is the physical
# assertion; a vanilla redstone block powers the same lamp first as the positive control. A receiver
# on another channel stays dark, removing the source darkens the first receiver, a partial-strength
# wire proves analog preservation, and a relaunch proves the channel table was persisted in the
# save rather than held in memory. Every claim comes from a parsed `ctl` reply and the game's own
# block state, never wall time or the absence of an error. Reuses the launcher-built game/ctl and
# the path exports (E2E_TMP, never /tmp).
export LC_ALL=C
[ -z "${RFU_DEBUG:-}" ] || set -x

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/rftoolsutility
CTL=${CTL:-}
GAME=${GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/rftoolsutility-redstone.sock}
SCRATCH_ROOT=${E2E_TMP:-/tmp/opencode}
SELFTEST=${SELFTEST:-0}
SELFTEST_AT=${SELFTEST_AT:-redstone}
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

SCRATCH=$(mktemp -d "$SCRATCH_ROOT/rftoolsutility-redstone.XXXXXX")
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

# A unique nonce pins the reply under chat load; parse the reply's own fields.
red_read() { # posx posy posz
    NONCE=$((NONCE + 1))
    ctl chat "/rftoolsutility redstone $1 $2 $3 $NONCE" >/dev/null
    ctl wait 2 >/dev/null
    REPLY=$(ctl messages 6 | jq -r '.chat[]' | grep -m1 "nonce=$NONCE" || true)
    [ -n "$REPLY" ] || fail "no redstone reply for nonce $NONCE"
    echo "  $REPLY" >&2
    Q_ROLE=$(echo "$REPLY" | sed -n 's/.*redstone \([a-z]*\) at.*/\1/p')
    Q_CHANNEL=$(echo "$REPLY" | sed -n 's/.*channel=\([0-9-]*\).*/\1/p')
    Q_POWER=$(echo "$REPLY" | sed -n 's/.*power=\([0-9-]*\).*/\1/p')
    Q_PREV=$(echo "$REPLY" | sed -n 's/.*prev=\([0-9-]*\).*/\1/p')
    Q_VALUE=$(echo "$REPLY" | sed -n 's/.*value=\([0-9-]*\).*/\1/p')
    Q_OUTPUT=$(echo "$REPLY" | sed -n 's/.*output=\([0-9-]*\).*/\1/p')
    Q_CHANGED=$(echo "$REPLY" | sed -n 's/.*changed=\([0-9-]*\).*/\1/p')
    Q_LAMP=$(echo "$REPLY" | sed -n 's/.*lamp=\([a-z]*\).*/\1/p')
    Q_TIME=$(echo "$REPLY" | sed -n 's/.*time=\([0-9]*\).*/\1/p')
    Q_ANALOG=$(echo "$REPLY" | sed -n 's/.*analog=\([a-z]*\).*/\1/p')
    for field in Q_ROLE Q_CHANNEL Q_LAMP Q_TIME; do
        [ -n "${!field}" ] || fail "the redstone line did not parse ($field): $REPLY"
    done
}

configure() { # posx posy posz channel analog facing
    ctl chat "/rftoolsutility redstone-set $1 $2 $3 $4 $5 $6" >/dev/null
    ctl wait 16 >/dev/null
}

TX=$((X0 + 2)); TY=101; TZ=0           # the transmitter
RX=$((X0 + 2)); RY=101; RZ=4           # the receiver on the fed channel
RX2=$((X0 + 2)); RY2=101; RZ2=8        # the receiver on another channel
LX=$((RX + 1)); LY=$RY; LZ=$RZ         # the fed receiver's vanilla lamp
L2X=$((RX2 + 1)); L2Y=$RY2; L2Z=$RZ2   # the other receiver's vanilla lamp
IN=$((TX - 0)); INY=$TY; INZ=$((TZ - 1))  # the transmitter's north input neighbour
PX=$((LX + 1)); PY=$LY; PZ=$LZ         # the lamp's positive control
NONCE=0

launch
arena

echo "== R: a transmitter, two receivers and two vanilla lamps are placed and read back =="
ctl command "gamemode creative" >/dev/null
ctl wait 6 >/dev/null
place_block "rftoolsutility:redstone_transmitter" "$TX" "$TY" "$TZ"
place_block "rftoolsutility:redstone_receiver" "$RX" "$RY" "$RZ"
place_block "rftoolsutility:redstone_receiver" "$RX2" "$RY2" "$RZ2"
setblock "$LX" "$LY" "$LZ" minecraft:redstone_lamp
setblock "$L2X" "$L2Y" "$L2Z" minecraft:redstone_lamp
ctl wait 8 >/dev/null
for at in "$TX $TY $TZ rftoolsutility:redstone_transmitter" "$RX $RY $RZ rftoolsutility:redstone_receiver" "$RX2 $RY2 $RZ2 rftoolsutility:redstone_receiver" "$LX $LY $LZ minecraft:redstone_lamp" "$L2X $L2Y $L2Z minecraft:redstone_lamp"; do
    read -r x y z want <<<"$at"
    [ "$(block_name "$x" "$y" "$z")" = "$want" ] || fail "$want did not place at $x,$y,$z"
done

echo "== R-control: a vanilla redstone block lights each lamp (control for the assertion) =="
setblock "$PX" "$PY" "$PZ" minecraft:redstone_block
ctl wait 20 >/dev/null
echo "  lamp with a vanilla redstone block: $(state_at $LX $LY $LZ)"
lamp_lit "$LX" "$LY" "$LZ" || fail "a vanilla redstone block did not light the lamp (the control cannot prove anything)"
setblock "$PX" "$PY" "$PZ" minecraft:air
ctl wait 20 >/dev/null
echo "  lamp after removing it: $(state_at $LX $LY $LZ)"
if lamp_lit "$LX" "$LY" "$LZ"; then fail "the lamp stayed lit after the redstone block was removed"; fi

echo "== R0: a fresh channel is allocated and both receivers are bound; no source means dark =="
configure "$TX" "$TY" "$TZ" -1 false north
red_read "$TX" "$TY" "$TZ"
[ "$Q_ROLE" = transmitter ] || fail "the transmitter readout role is '$Q_ROLE'"
CHANNEL=$Q_CHANNEL
[ "$CHANNEL" -ge 1 ] || fail "channel -1 did not allocate a fresh channel: channel=$CHANNEL"
OTHER=$((CHANNEL + 1))
configure "$RX" "$RY" "$RZ" "$CHANNEL" false north
configure "$RX2" "$RY2" "$RZ2" "$OTHER" false north
ctl wait 20 >/dev/null
red_read "$RX" "$RY" "$RZ"
[ "$Q_CHANNEL" = "$CHANNEL" ] || fail "the receiver is on channel $Q_CHANNEL, expected $CHANNEL"
[ "$Q_OUTPUT" = 0 ] || fail "an unfed receiver reads output=$Q_OUTPUT, expected 0"
[ "$Q_LAMP" = dark ] || fail "an unfed receiver's lamp reads $Q_LAMP, expected dark"
if lamp_lit "$LX" "$LY" "$LZ"; then fail "an unfed receiver's vanilla lamp is lit"; fi

echo "== R1: a redstone block feeds the transmitter; channel N emits 15 and channel N+1 stays dark =="
setblock "$IN" "$INY" "$INZ" minecraft:redstone_block
ctl wait 30 >/dev/null
red_read "$TX" "$TY" "$TZ"
[ "$Q_ROLE" = transmitter ] || fail "the transmitter readout role is '$Q_ROLE'"
[ "$Q_POWER" = 15 ] || fail "the transmitter reads power=$Q_POWER, expected 15"
[ "$Q_VALUE" = 15 ] || fail "the transmitter wrote value=$Q_VALUE, expected 15"
red_read "$RX" "$RY" "$RZ"
[ "$Q_ROLE" = receiver ] || fail "the receiver readout role is '$Q_ROLE'"
[ "$Q_VALUE" = 15 ] || fail "the fed channel reads value=$Q_VALUE, expected 15"
[ "$Q_OUTPUT" = 15 ] || fail "a digital receiver reads output=$Q_OUTPUT, expected the artefact's 15"
[ "$Q_CHANGED" -ge 0 ] || fail "no output-change stamp was written: changed=$Q_CHANGED"
[ "$Q_TIME" -ge "$Q_CHANGED" ] || fail "the change stamp is after the readout's time"
if [ "$SELFTEST" = 1 ] && [ "$SELFTEST_AT" = redstone ]; then
    [ "$Q_OUTPUT" != 15 ] || fail "SELFTEST redstone: the fed receiver reads 15, expected it inverted (the lamp assertion can fail)"
fi
[ "$Q_LAMP" = lit ] || fail "the fed receiver's lamp reads $Q_LAMP in the reply, expected lit"
lamp_lit "$LX" "$LY" "$LZ" || fail "the fed receiver's vanilla lamp is dark: $(state_at $LX $LY $LZ)"
[ "$(signal_at "$RX" "$RY" "$RZ")" = 15 ] || fail "the receiver's block state does not cache signal=15"
red_read "$RX2" "$RY2" "$RZ2"
[ "$Q_CHANNEL" = "$OTHER" ] || fail "the other receiver is on channel $Q_CHANNEL, expected $OTHER"
[ "$Q_VALUE" = 0 ] || fail "the other channel reads value=$Q_VALUE, expected 0"
[ "$Q_OUTPUT" = 0 ] || fail "the other receiver reads output=$Q_OUTPUT, expected 0"
[ "$Q_LAMP" = dark ] || fail "the other receiver's lamp reads $Q_LAMP, expected dark"
if lamp_lit "$L2X" "$L2Y" "$L2Z"; then fail "the other receiver's vanilla lamp is lit"; fi

echo "== R2: removing the source darkens the fed receiver (one-input differential) =="
setblock "$IN" "$INY" "$INZ" minecraft:air
ctl wait 30 >/dev/null
red_read "$RX" "$RY" "$RZ"
[ "$Q_VALUE" = 0 ] || fail "the drained channel reads value=$Q_VALUE, expected 0"
[ "$Q_OUTPUT" = 0 ] || fail "the drained receiver reads output=$Q_OUTPUT, expected 0"
[ "$Q_LAMP" = dark ] || fail "the drained receiver's lamp reads $Q_LAMP, expected dark"
if lamp_lit "$LX" "$LY" "$LZ"; then fail "the drained receiver's vanilla lamp is lit"; fi

echo "== R3: a partial-strength wire proves analog mode preserves the exact 0..15 =="
setblock "$IN" "$INY" "$((INZ - 5))" minecraft:redstone_block
for z in $(seq "$((INZ - 4))" "$INZ"); do setblock "$IN" "$INY" "$z" minecraft:redstone_wire; done
ctl wait 30 >/dev/null
red_read "$TX" "$TY" "$TZ"
PARTIAL=$Q_POWER
[ "$PARTIAL" -ge 1 ] && [ "$PARTIAL" -lt 15 ] || fail "the wire input is $PARTIAL, expected a partial 1..14 strength"
[ "$Q_VALUE" = "$PARTIAL" ] || fail "the transmitter wrote value=$Q_VALUE, expected the input power $PARTIAL"
configure "$RX" "$RY" "$RZ" "$CHANNEL" true north
ctl wait 20 >/dev/null
red_read "$RX" "$RY" "$RZ"
[ "$Q_ANALOG" = true ] || fail "the receiver's analog mode reads '$Q_ANALOG', expected true"
[ "$Q_VALUE" = "$PARTIAL" ] || fail "the analog receiver's channel value is $Q_VALUE, expected $PARTIAL"
[ "$Q_OUTPUT" = "$PARTIAL" ] || fail "analog mode reads output=$Q_OUTPUT, expected the preserved strength $PARTIAL"
[ "$Q_LAMP" = lit ] || fail "the analog receiver's lamp reads $Q_LAMP, expected lit"
lamp_lit "$LX" "$LY" "$LZ" || fail "the analog receiver's vanilla lamp is dark"

echo "== R4: the channel table survives a relaunch of the same world =="
# The source is still present, so on reload `prevIn` matches and the transmitter does not
# rewrite the channel: a lit lamp now can only come from the table the save carried.
ctl quit >/dev/null 2>&1 || true
sleep 2
launch "$WORLD"
ctl command "execute in minecraft:the_end run tp @s $X0.5 101 0.5" >/dev/null
ctl wait 30 >/dev/null
red_read "$RX" "$RY" "$RZ"
[ "$Q_CHANNEL" = "$CHANNEL" ] || fail "after relaunch the receiver is on channel $Q_CHANNEL, expected $CHANNEL"
[ "$Q_OUTPUT" = "$PARTIAL" ] || fail "after relaunch the receiver reads output=$Q_OUTPUT, expected the persisted $PARTIAL"
[ "$Q_LAMP" = lit ] || fail "after relaunch the receiver's lamp reads $Q_LAMP, expected lit"
lamp_lit "$LX" "$LY" "$LZ" || fail "after relaunch the vanilla lamp is dark: the channel table was not persisted"

echo "== R5: an unknown block id is refused =="
if refusal=$(ctl give rftoolsutility:not_a_redstone 1 2>&1); then rc=0; else rc=$?; fi
echo "  give: rc=$rc $refusal"
[ "$rc" -ne 0 ] || [[ "$refusal" == *"Unknown"* ]] || fail "an unknown item id was not refused: $refusal"

echo "PASS: a redstone-block-fed transmitter writes its input strength to a world-saved numbered channel, a receiver on that channel lights a vanilla lamp at the artefact's declared 15 (the exact 0..15 in analog mode) while a receiver on another channel stays dark; removing the source, the partial-strength analog case, the relaunch, the redstone-block lamp control, the game-time stamps and SELFTEST_AT=redstone all assert parsed values."
