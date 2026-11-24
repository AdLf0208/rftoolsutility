#!/usr/bin/env bash
# Focused, fail-capable slice-21 leg: the environmental controller's *second* module.
#
# Artefact (f799314): createRegenerationPlusModule() =
#   PotionEffectModule.create("regeneration", 2, BUFF_REGENERATIONPLUS,
#                             REGENERATIONPLUS_RFPERTICK.get());  // 0.0045
# The first module is the same potion at amplifier 0 / 0.0015. Both are applied to the
# player's own game effect store; the stronger amplifier wins, and each module contributes
# its own (int)(rfPerTick * volume) to the controller's FE/tick (volume 6283 at radius 20,
# y 100..104: 9 + 28 = 37, x2 in MODE_ALL = 74, up from 18 with regeneration alone).
# Every claim is a parsed field of a nonce-pinned ctl reply, never the absence of an error.
export LC_ALL=C
[ -z "${RFU_DEBUG:-}" ] || set -x

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/rftoolsutility
CTL=${CTL:-}
GAME=${GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/rftoolsutility-environmental2.sock}
SCRATCH_ROOT=${E2E_TMP:-/tmp/opencode}
SELFTEST=${SELFTEST:-0}
SELFTEST_AT=${SELFTEST_AT:-environmental2}
CONTROL=${CONTROL:-0}
KEEP=${KEEP:-0}
WORLD=RFToolsUtilityE2E2
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

SCRATCH=$(mktemp -d "$SCRATCH_ROOT/rftoolsutility-environmental2.XXXXXX")
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
    EN_RF=$(echo "$REPLY" | sed -n 's/.*rf=\([0-9-]*\).*/\1/p')
    EN_MODULES=$(echo "$REPLY" | sed -n 's/.*modules=\([0-9-]*\).*/\1/p')
    EN_EFFECT=$(echo "$REPLY" | sed -n 's/.*effect=\([^ ]*\).*/\1/p')
    EN_AMP=$(echo "$REPLY" | sed -n 's/.*amplifier=\([^ ]*\).*/\1/p')
    [ -n "$ENERGY" ] && [ -n "$EN_CAP" ] && [ -n "$EN_ACTIVE" ] && [ -n "$EN_MODE" ] \
        && [ -n "$EN_RADIUS" ] && [ -n "$EN_RF" ] && [ -n "$EN_MODULES" ] \
        && [ -n "$EN_EFFECT" ] && [ -n "$EN_AMP" ] \
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
NONCE=0

echo "artefact f799314: regen+ = PotionEffectModule.create(\"regeneration\", 2, BUFF_REGENERATIONPLUS, 0.0045); MAXTICKS=180, duration=540"
echo "volume radius 20 / y 100..104 = 6283: regen (int)(0.0015*6283)=9, plus (int)(0.0045*6283)=28, sum 37, MODE_ALL x2 = 74"

launch
arena

echo "== EP0: a fresh controller has no module and no effect =="
ctl command "gamemode creative" >/dev/null
ctl wait 6 >/dev/null
place_block "rftoolsutility:environmental_controller" "$PX" "$PY" "$PZ"
en_set "$PX" "$PY" "$PZ" 20 100 104 all true
ctl command "execute in minecraft:the_end run tp @s $((PX + 2)).5 $PY $PZ.5" >/dev/null
ctl wait 6 >/dev/null
environmental_read "$PX" "$PY" "$PZ"
[ "$EN_CAP" = 500000 ] || fail "the controller's capacity is $EN_CAP, expected 500000: $REPLY"
[ "$EN_MODE" = all ] && [ "$EN_RADIUS" = 20 ] \
    || fail "environmental-set did not save the area/mode: $REPLY"
[ "$ENERGY" = 0 ] && [ "$EN_ACTIVE" = false ] && [ "$EN_MODULES" = 0 ] \
    && [ "$EN_RF" = 0 ] && [ "$EN_EFFECT" = none ] \
    || fail "a fresh, unpowered controller is not empty and inactive: $REPLY"

echo "== EP1: regeneration alone applies amplifier 0 at the artefact's own 18 FE/tick =="
ctl chat "/rftoolsutility buffer $PX $PY $PZ 0 rftoolsutility:regeneration_module 1" >/dev/null
en_set "$PX" "$PY" "$PZ" 20 100 104 all true
ctl chat "/rftoolsutility energy $PX $PY $PZ 100000" >/dev/null
ctl wait 60 >/dev/null
environmental_read "$PX" "$PY" "$PZ"
[ "$EN_MODULES" = 1 ] || fail "the first module was not inserted into slot 0: $REPLY"
[ "$EN_RF" = 18 ] || fail "one regeneration module at radius 20 costs rf=$EN_RF, expected 18: $REPLY"
[ "$EN_ACTIVE" = true ] || fail "a fed controller with a module is not active: $REPLY"
[ "$EN_EFFECT" = minecraft:regeneration ] && [ "$EN_AMP" = 0 ] \
    || fail "the powered regeneration module read $EN_EFFECT amplifier=$EN_AMP, expected minecraft:regeneration amplifier=0: $REPLY"
E1=$ENERGY
ctl wait 60 >/dev/null
environmental_read "$PX" "$PY" "$PZ"
D1=$((E1 - ENERGY))
[ "$D1" -gt 0 ] && [ $((D1 % 18)) -eq 0 ] \
    || fail "the one-module buffer fell by $D1 FE, not a positive multiple of 18: $REPLY"
echo "  one module: the buffer fell $D1 FE = $((D1 / 18)) x 18 FE/tick"

echo "== EP2: regenerationplus alongside it gives amplifier 2 and adds its own cost =="
clear_effect
ctl chat "/rftoolsutility buffer $PX $PY $PZ 1 rftoolsutility:regenerationplus_module 1" >/dev/null
en_set "$PX" "$PY" "$PZ" 20 100 104 all true
ctl wait 60 >/dev/null
environmental_read "$PX" "$PY" "$PZ"
[ "$EN_MODULES" = 2 ] || fail "the second module was not inserted into slot 1: $REPLY"
[ "$EN_RF" = 74 ] \
    || fail "two modules cost rf=$EN_RF, expected 74 = (9+28)x2 (the plus module's own 28x2 on top of 18): $REPLY"
[ "$EN_EFFECT" = minecraft:regeneration ] \
    || fail "the player's effect id is $EN_EFFECT, expected minecraft:regeneration: $REPLY"
if [ "$SELFTEST" = 1 ] && [ "$SELFTEST_AT" = environmental2 ]; then
    # Invert the product assertion: the stronger module must NOT raise the amplifier.
    if [ "$EN_AMP" = 2 ]; then
        fail "SELFTEST environmental2: the powered plus module read $EN_EFFECT amplifier=$EN_AMP, expected it to stay at amplifier 0"
    fi
fi
[ "$EN_AMP" = 2 ] \
    || fail "the powered plus module read amplifier=$EN_AMP, expected 2 (regeneration III): $REPLY"
E2=$ENERGY
ctl wait 60 >/dev/null
environmental_read "$PX" "$PY" "$PZ"
D2=$((E2 - ENERGY))
[ "$D2" -gt 0 ] && [ $((D2 % 74)) -eq 0 ] \
    || fail "the two-module buffer fell by $D2 FE, not a positive multiple of 74: $REPLY"
echo "  two modules: the buffer fell $D2 FE = $((D2 / 74)) x 74 FE/tick (56 more per tick than one module)"

echo "== EP3: removing the second module restores the weaker amplifier (differential) =="
clear_effect
ctl chat "/rftoolsutility buffer $PX $PY $PZ 1 minecraft:air 0" >/dev/null
en_set "$PX" "$PY" "$PZ" 20 100 104 all true
ctl wait 60 >/dev/null
environmental_read "$PX" "$PY" "$PZ"
[ "$EN_MODULES" = 1 ] && [ "$EN_RF" = 18 ] \
    || fail "removing the plus module did not restore the one-module readout: $REPLY"
[ "$EN_EFFECT" = minecraft:regeneration ] && [ "$EN_AMP" = 0 ] \
    || fail "after removing the plus module the effect read $EN_EFFECT amplifier=$EN_AMP, expected amplifier 0: $REPLY"

echo "== EP4: an unknown module id is refused =="
if refusal=$(ctl give rftoolsutility:not_a_module 1 2>&1); then rc=0; else rc=$?; fi
echo "  give: rc=$rc $refusal"
[ "$rc" -ne 0 ] || [[ "$refusal" == *"Unknown"* ]] \
    || fail "an unknown module id was not refused: $refusal"

echo "PASS: with regeneration and regenerationplus in the controller's seven slots the player's own game effect store reads minecraft:regeneration amplifier=2 (the stronger of the artefact's amplifiers 0 and 2), the parsed FE/tick rises from 18 to 74 by the plus module's own 28x2 cost and the real buffer drop follows it; removing the module returns amplifier 0; SELFTEST_AT=environmental2 ends EXIT:1."
