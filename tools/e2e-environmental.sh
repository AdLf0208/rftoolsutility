#!/usr/bin/env bash
# Focused, fail-capable slice-20 leg: the environmental controller's first module.
#
# A controller with a regeneration module, enough FE and MODE_ALL reads the player's own
# minecraft:regeneration straight out of the game's effect store with LivingRef::effect, and its
# buffer falls by the artefact's volume-scaled FE/tick. The differentials each change one input:
# a vanilla /effect give proves the readout can see an effect at all (positive control), a vanilla
# clear with no module / no power / the controller disabled / the player outside the radius each
# reads no effect, and SELFEST_AT=environmental inverts the effect assertion. Every claim is a
# parsed field of a nonce-pinned ctl reply, never the absence of an error.
export LC_ALL=C
[ -z "${RFU_DEBUG:-}" ] || set -x

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/rftoolsutility
CTL=${CTL:-}
GAME=${GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/rftoolsutility-environmental.sock}
SCRATCH_ROOT=${E2E_TMP:-/tmp/opencode}
SELFTEST=${SELFTEST:-0}
SELFTEST_AT=${SELFTEST_AT:-environmental}
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

SCRATCH=$(mktemp -d "$SCRATCH_ROOT/rftoolsutility-environmental.XXXXXX")
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
    ctl command "execute in minecraft:the_end run fill $((X0 - 4)) 100 -14 $((X0 + 24)) 100 14 minecraft:stone" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "execute in minecraft:the_end run fill $((X0 - 4)) 101 -14 $((X0 + 24)) 108 14 minecraft:air" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "execute in minecraft:the_end run kill @e[type=!player]" >/dev/null
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

# Parse the controller's own readout. The nonce pins this reply under chat load.
environmental_read() { # x y z
    NONCE=$((NONCE + 1))
    ctl chat "/rftoolsutility environmental $1 $2 $3 $NONCE" >/dev/null
    ctl wait 1 >/dev/null
    REPLY=$(ctl messages 8 | jq -r '.chat[]' | grep -m1 "nonce=$NONCE" || true)
    [ -n "$REPLY" ] || fail "no environmental reply for nonce $NONCE at $1,$2,$3"
    echo "  $REPLY" >&2
    ENERGY=$(echo "$REPLY" | sed -n 's/.*energy=\([0-9-]*\).*/\1/p')
    EN_CAP=$(echo "$REPLY" | sed -n 's/.*energy=[0-9-]*\/\([0-9-]*\).*/\1/p')
    EN_ACTIVE=$(echo "$REPLY" | sed -n 's/.*active=\([a-z]*\).*/\1/p')
    EN_MODE=$(echo "$REPLY" | sed -n 's/.*mode=\([a-z]*\).*/\1/p')
    EN_RADIUS=$(echo "$REPLY" | sed -n 's/.*radius=\([0-9-]*\).*/\1/p')
    EN_MINY=$(echo "$REPLY" | sed -n 's/.*miny=\([0-9-]*\).*/\1/p')
    EN_MAXY=$(echo "$REPLY" | sed -n 's/.*maxy=\([0-9-]*\).*/\1/p')
    EN_RF=$(echo "$REPLY" | sed -n 's/.*rf=\([0-9-]*\).*/\1/p')
    EN_MODULES=$(echo "$REPLY" | sed -n 's/.*modules=\([0-9-]*\).*/\1/p')
    EN_EFFECT=$(echo "$REPLY" | sed -n 's/.*effect=\([^ ]*\).*/\1/p')
    EN_AMP=$(echo "$REPLY" | sed -n 's/.*amplifier=\([^ ]*\).*/\1/p')
    [ -n "$ENERGY" ] && [ -n "$EN_CAP" ] && [ -n "$EN_ACTIVE" ] && [ -n "$EN_MODE" ] \
        && [ -n "$EN_RADIUS" ] && [ -n "$EN_MINY" ] && [ -n "$EN_MAXY" ] && [ -n "$EN_RF" ] \
        && [ -n "$EN_MODULES" ] && [ -n "$EN_EFFECT" ] && [ -n "$EN_AMP" ] \
        || fail "the environmental line did not parse: $REPLY"
}

en_set() { # x y z radius miny maxy mode enabled
    ctl chat "/rftoolsutility environmental-set $1 $2 $3 $4 $5 $6 $7 $8" >/dev/null
    ctl wait 2 >/dev/null
}

clear_effect() {
    ctl chat "/effect clear @s minecraft:regeneration" >/dev/null 2>&1 || true
    ctl wait 2 >/dev/null
}

PX=$X0; PY=101; PZ=0
FAR_X=$((X0 + 20))
NONCE=0

launch
arena

echo "== EN: the controller is placed and read back =="
ctl command "gamemode creative" >/dev/null
ctl wait 6 >/dev/null
place_block "rftoolsutility:environmental_controller" "$PX" "$PY" "$PZ"
en_set "$PX" "$PY" "$PZ" 8 100 102 all true
ctl command "execute in minecraft:the_end run tp @s $((PX + 2)).5 $PY $PZ.5" >/dev/null
ctl wait 6 >/dev/null
environmental_read "$PX" "$PY" "$PZ"
[ "$EN_CAP" = 500000 ] || fail "the controller's capacity is $EN_CAP, expected 500000: $REPLY"
[ "$EN_MODE" = all ] && [ "$EN_RADIUS" = 8 ] && [ "$EN_MINY" = 100 ] && [ "$EN_MAXY" = 102 ] \
    || fail "environmental-set did not save the area/mode: $REPLY"
[ "$ENERGY" = 0 ] && [ "$EN_ACTIVE" = false ] && [ "$EN_MODULES" = 0 ] \
    || fail "a fresh, unpowered controller is not empty and inactive: $REPLY"

echo "== EN-control: a vanilla effect proves the readout can see an effect =="
ctl chat "/effect give @s minecraft:regeneration 100 0" >/dev/null
ctl wait 4 >/dev/null
environmental_read "$PX" "$PY" "$PZ"
[ "$EN_EFFECT" = minecraft:regeneration ] && [ "$EN_AMP" = 0 ] \
    || fail "the readout could not see a vanilla regeneration effect: $REPLY"
clear_effect
environmental_read "$PX" "$PY" "$PZ"
[ "$EN_EFFECT" = none ] || fail "the effect did not clear: $REPLY"

echo "== EN1: a module with no FE stays inactive (the power differential) =="
ctl chat "/rftoolsutility buffer $PX $PY $PZ 0 rftoolsutility:regeneration_module 1" >/dev/null
ctl wait 4 >/dev/null
ctl chat "/rftoolsutility energy $PX $PY $PZ 0" >/dev/null
ctl wait 200 >/dev/null
environmental_read "$PX" "$PY" "$PZ"
[ "$EN_MODULES" = 1 ] || fail "the module was not inserted into slot 0: $REPLY"
[ "$EN_RF" = 5 ] || fail "one module at radius 8 costs rf=$EN_RF, expected the 5 FE floor: $REPLY"
[ "$EN_ACTIVE" = false ] && [ "$EN_EFFECT" = none ] \
    || fail "an unpowered controller with a module applied an effect: $REPLY"

echo "== EN2: with >= rfNeeded the module applies regeneration and spends the buffer =="
ctl chat "/rftoolsutility energy $PX $PY $PZ 5000" >/dev/null
ctl wait 200 >/dev/null
environmental_read "$PX" "$PY" "$PZ"
[ "$EN_ACTIVE" = true ] || fail "a fed controller with a module is not active: $REPLY"
[ "$EN_RF" -gt 0 ] || fail "rfNeeded is not positive: $REPLY"
DROP=$((5000 - ENERGY))
[ "$DROP" -gt 0 ] || fail "the buffer did not fall from 5000 FE (now $ENERGY): $REPLY"
[ $((DROP % EN_RF)) -eq 0 ] \
    || fail "the buffer fell by $DROP FE, not a multiple of rfNeeded=$EN_RF: $REPLY"
if [ "$SELFTEST" = 1 ] && [ "$SELFTEST_AT" = environmental ]; then
    # Invert the product assertion: the powered module must NOT show the effect.
    if [ "$EN_EFFECT" = minecraft:regeneration ] && [ "$EN_AMP" = 0 ]; then
        fail "SELFTEST environmental: the powered module read $EN_EFFECT amplifier=$EN_AMP, expected none"
    fi
fi
[ "$EN_EFFECT" = minecraft:regeneration ] && [ "$EN_AMP" = 0 ] \
    || fail "the powered module did not give minecraft:regeneration amplifier=0: $REPLY"

echo "== EN3: a disabled controller applies no effect =="
clear_effect
en_set "$PX" "$PY" "$PZ" 8 100 102 all false
ctl wait 200 >/dev/null
environmental_read "$PX" "$PY" "$PZ"
[ "$EN_ACTIVE" = false ] && [ "$EN_EFFECT" = none ] \
    || fail "a disabled controller still applied an effect: $REPLY"

echo "== EN4: a player outside the radius reads no effect =="
en_set "$PX" "$PY" "$PZ" 8 100 102 all true
clear_effect
ctl command "execute in minecraft:the_end run tp @s $FAR_X.5 $PY $PZ.5" >/dev/null
ctl wait 200 >/dev/null
environmental_read "$PX" "$PY" "$PZ"
[ "$EN_ACTIVE" = true ] || fail "the controller stopped being active while powered: $REPLY"
[ "$EN_EFFECT" = none ] || fail "a player outside radius 8 got the effect: $REPLY"

echo "== EN5: removing the last module deactivates the controller =="
ctl command "execute in minecraft:the_end run tp @s $((PX + 2)).5 $PY $PZ.5" >/dev/null
clear_effect
ctl chat "/rftoolsutility buffer $PX $PY $PZ 0 minecraft:air 0" >/dev/null
ctl wait 200 >/dev/null
environmental_read "$PX" "$PY" "$PZ"
[ "$EN_MODULES" = 0 ] && [ "$EN_RF" = 0 ] && [ "$EN_ACTIVE" = false ] && [ "$EN_EFFECT" = none ] \
    || fail "a controller with no module is still active: $REPLY"

echo "== EN6: an unknown item id is refused =="
if refusal=$(ctl give rftoolsutility:not_a_controller 1 2>&1); then rc=0; else rc=$?; fi
echo "  give: rc=$rc $refusal"
[ "$rc" -ne 0 ] || [[ "$refusal" == *"Unknown"* ]] \
    || fail "an unknown item id was not refused: $refusal"

echo "PASS: the environmental controller with one regeneration module, MODE_ALL, radius 8 and enough FE reads minecraft:regeneration amplifier=0 from the player's own game effect store, spends the artefact's 5 FE/tick, and reads no effect when unpowered, disabled, out of radius or without a module; the vanilla-effect control proves the readout, and SELFTEST_AT=environmental ends EXIT:1."
