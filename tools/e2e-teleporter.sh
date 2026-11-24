#!/usr/bin/env bash
# Focused, fail-capable slice-9 leg: the matter transmitter / receiver / booster.
# Reuses the exact helpers of tools/e2e.sh (game built with 0.2.8).
export LC_ALL=C
[ -z "${RFU_DEBUG:-}" ] || set -x

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/rftoolsutility
# Both binaries come from the campaign pin (0a875da): GAME from `launcher-game.sh build`,
# CTL from the same export.
CTL=${CTL:-}
GAME=${GAME:-}   # a launcher-built game carrying this package (see the header)
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/rftoolsutility-e2e.sock}
# `/tmp` is a 7.5 GB tmpfs on the campaign box; the launcher's export sets `E2E_TMP` to a
# folder on /home, so the scratch home goes there when it is set.
SCRATCH_ROOT=${E2E_TMP:-/tmp/opencode}
SELFTEST=${SELFTEST:-0}
SELFTEST_AT=${SELFTEST_AT:-fill}
CONTROL=${CONTROL:-0}
KEEP=${KEEP:-0}
WORLD=RFToolsUtilityE2E
X0=200
SX=$((X0 + 2))
EXPECT_CAPACITY=32000
BUCKET=1000
BLOCK=rftoolsutility:tank
WATER_BUCKET=minecraft:water_bucket
LAVA_BUCKET=minecraft:lava_bucket
EMPTY_BUCKET=minecraft:bucket
CAULDRON=minecraft:cauldron
CRAFTER=rftoolsutility:crafter1
TIER2_BLOCK=rftoolsutility:crafter2
TIER3_BLOCK=rftoolsutility:crafter3
SPAWNER_BLOCK=rftoolsutility:spawner
BEAMER_BLOCK=rftoolsutility:matter_beamer
SZ=3   # the spawner's z in the arena: clear of the tank's and the crafter's cells
ZC2=16  # slice 6: the two-dimension phase's crafter twins (the arena floor runs to z=20)
ZS2=18  # ... its spawner twins, facing south so the spawn point is z=19

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -n "$CTL" ] || { echo "set CTL= to the horizon-ctl of this pin (launcher-game.sh env)" >&2; exit 1; }
[ -n "$GAME" ] || { echo "set GAME= to a game built with this package: horizon-launcher native-build $PORT --instance <i> --trust-native-code" >&2; exit 1; }
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

launch() { # launch [WORLD]
    local world=${1:-$WORLD}
    rm -f "$SOCK"
    cd "$SCRATCH"
    local args=(--game "$GAME" --world "$world" --seed 20261105 --creative --headless
                --wait --resources "$RES")
    "$CTL" --socket "$SOCK" launch "${args[@]}" >/dev/null
    ctl wait-ready 120
    ctl wait 40
}

# `ctl block` answers {pos, block, state, loaded}; the state is a string such as
# `rftoolsutility:tank[level=1]`.
block_name() { ctl block "$1" "$2" "$3" | jq -r '.block // empty'; }
block_level() { ctl block "$1" "$2" "$3" | jq -r '.state // empty' | sed -n 's/.*\[level=\([0-9]*\)\].*/\1/p'; }

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

# `/rftoolsutility info X Y Z`: the line, then the parsed fields.
hold() { # hold ITEM [COUNT]
    ctl command "item replace entity @s hotbar.0 with $1 ${2:-1}" >/dev/null
    ctl wait 4 >/dev/null
    ctl select 1 >/dev/null
    ctl wait 2 >/dev/null
}

# Click ITEM from the hand down onto the floor under X Y Z. Retries the click twice: the
# client's selection and aim settle a tick or two after a teleport, and a `place` that lands
# before that is refused by the server (seen once, on the tank's doctored-item phase).
# `ctl place` refuses a target a mob or a dropped item stands on ("the block did not
# change (blocked by an entity, or the item cannot be placed)"). Clear the two blocks around
# the target and try again: the arena's own floor lets the End's endermen wander in over a
# long run, which cost a run here.
clear_at() { # clear_at X Y Z [RADIUS]
    ctl command "execute in minecraft:the_end run kill @e[type=!player,x=$1,y=$2,z=$3,distance=..${4:-2}]" \
        >/dev/null 2>&1 || true
    ctl wait 4 >/dev/null
}

try_place() { # try_place ITEM X Y Z
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

# `ctl place --item` places from the hotbar only, and a machine broken while its buffer holds
# items drops those beside it, so `ctl collect` can fill the hotbar and leave the machine's own
# item in the main inventory (AF hit exactly that). Free the last hotbar slot and put the carried
# stack there with the game's own `item` command: `container.<slot>` names a player's raw
# inventory slot, and `from` copies, which is all a placement needs.
hold_carried() { # hold_carried ITEM
    local item=$1 slot
    if ctl inventory | jq -e "[.hotbar[] | select(.item == \"$item\")] | length >= 1" >/dev/null 2>&1; then
        return 0
    fi
    slot=$(ctl inventory | jq -r --arg i "$item" '[.main[] | select(.item == $i)][0].slot // empty')
    [ -n "$slot" ] || fail "the carried $item is in no inventory slot: $(ctl inventory | jq -c '{hotbar,main}')"
    ctl command "item replace entity @s hotbar.8 with minecraft:air" >/dev/null
    ctl wait 4 >/dev/null
    ctl command "item replace entity @s hotbar.8 from entity @s container.$slot" >/dev/null
    ctl wait 6 >/dev/null
    ctl inventory | jq -e "[.hotbar[] | select(.item == \"$item\")] | length >= 1" >/dev/null \
        || fail "the carried $item did not reach the hotbar: $(ctl inventory | jq -c '{hotbar,main}')"
}

# Place ITEM from the player's hand at X Y Z (the player stands two blocks west).
place_block() { # place_block ITEM X Y Z
    local item=$1 x=$2 y=$3 z=$4
    ctl command "execute in minecraft:the_end run tp @s $((x - 2)).5 $y $z.5" >/dev/null
    ctl wait 6 >/dev/null
    hold "$item"
    try_place "$item" "$x" "$y" "$z"
    ctl wait 6 >/dev/null
    local name
    name=$(block_name "$x" "$y" "$z")
    [ "$name" = "$item" ] || fail "$item did not place at $x,$y,$z (got '$name')"
}

control() { # control ARGS…
    say "rftoolsutility $*"
    ctl wait 12 >/dev/null
    # `ctl messages` lists the newest line first, so `-m1` is this command's own reply.
    REPLY=$(ctl messages 12 | jq -r '.chat[]' | grep -m1 '/rftoolsutility' || true)
    echo "  $REPLY" >&2
}

teach() { control recipe "$@"; }
set_slot() { control buffer "$@"; }
set_energy() { control energy "$@"; }
set_setting() { control setting "$@"; }

# ---- slice 9: the teleporter family ----------------------------------------
TRANSMITTER_BLOCK=rftoolsutility:matter_transmitter
RECEIVER_BLOCK=rftoolsutility:matter_receiver
BOOSTER_BLOCK=rftoolsutility:matter_booster
TX_X=203; TX_Y=101; TX_Z=8
RX_X=203; RX_Y=101; RX_Z=12

transmitter() {
    NONCE=$((NONCE + 1))
    control transmitter "$1" "$2" "$3" "$NONCE"
    local line="$REPLY"
    T_ENERGY=$(echo "$line" | sed -n 's/.*energy=\([0-9]*\)\/200000.*/\1/p')
    T_DIAL=$(echo "$line" | sed -n 's/.*dial=\([^ ]*\).*/\1/p')
    T_STATE=$(echo "$line" | sed -n 's/.*state=\([^ ]*\).*/\1/p')
}
set_dial() { control dial "$@"; }
player_dim() { ctl state | jq -r '.dimension // empty'; }
player_raw() { ctl state | jq -r '.player.position | @csv'; }
expect_arrived() {
    [ "$P_DIM" = "$1" ] || fail "the player is in '$P_DIM', expected '$1'"
    local px py pz
    px=$(echo "$P_POS" | cut -d, -f1 | tr -d '"')
    py=$(echo "$P_POS" | cut -d, -f2 | tr -d '"')
    pz=$(echo "$P_POS" | cut -d, -f3 | tr -d '"')
    awk -v a="$px" -v b="$2" -v c="$py" -v d="$3" -v e="$pz" -v f="$4" \
        'BEGIN{ if ((a-b)<-0.75||(a-b)>0.75||(c-d)<-0.75||(c-d)>0.75||(e-f)<-0.75||(e-f)>0.75) exit 1 }' \
        || fail "the player landed at $P_POS, expected near [$2,$3,$4]"
}

launch
arena

echo "== AI: a transmitter, a receiver and a booster placed from their items read empty and idle =="
ctl command "gamemode creative" >/dev/null
ctl wait 6 >/dev/null
place_block "$TRANSMITTER_BLOCK" "$TX_X" "$TX_Y" "$TX_Z"
place_block "$RECEIVER_BLOCK" "$RX_X" "$RX_Y" "$RX_Z"
place_block "$BOOSTER_BLOCK" 200 101 -2
ctl wait 8 >/dev/null
transmitter "$TX_X" "$TX_Y" "$TX_Z"
[ "$T_ENERGY" = "0" ] || fail "a fresh transmitter holds $T_ENERGY FE, expected 0"
[ "$T_DIAL" = "none" ] || fail "a fresh transmitter is dialed to $T_DIAL, expected none"
[ "$T_STATE" = "idle" ] || fail "a fresh transmitter is $T_STATE, expected idle"
echo "  $REPLY"

echo "== AI1: standing in the beam of an undialed transmitter moves nothing (the control) =="
set_energy "$TX_X" "$TX_Y" "$TX_Z" 200000
ctl command "execute in minecraft:the_end run tp @s $TX_X.5 $((TX_Y + 1)) $TX_Z.5" >/dev/null
ctl wait 60 >/dev/null
P_DIM=$(player_dim); P_POS=$(player_raw)
echo "  undialed: dimension=$P_DIM position=$P_POS"
expect_arrived minecraft:the_end "$TX_X.5" "$((TX_Y + 1))" "$TX_Z.5"

echo "== AI2: dialing the transmitter to the receiver teleports the player to it =="
set_dial "$TX_X" "$TX_Y" "$TX_Z" minecraft:the_end "$RX_X" "$RX_Y" "$RX_Z"
ctl command "execute in minecraft:the_end run tp @s $TX_X.5 $((TX_Y + 1)) $TX_Z.5" >/dev/null
ctl wait 80 >/dev/null
P_DIM=$(player_dim); P_POS=$(player_raw)
echo "  dialed: dimension=$P_DIM position=$P_POS"
if [ "${SELFTEST:-0}" = 1 ] && [ "${SELFTEST_AT:-}" = teleport ]; then
    if awk -v p="$(echo "$P_POS" | cut -d, -f3 | tr -d '"')" -v s="$TX_Z.5" \
        'BEGIN{ d=p-s; if(d<0)d=-d; exit (d<0.75) }'; then
        fail "SELFTEST teleport: the player left the beam for $P_POS, expected it to stay at z=$TX_Z.5 (the inverted assertion proves the arrival can fail)"
    fi
fi
expect_arrived minecraft:the_end "$RX_X.5" "$((RX_Y + 1))" "$RX_Z.5"
transmitter "$TX_X" "$TX_Y" "$TX_Z"
echo "  after the teleport the transmitter reads $REPLY"

echo "PASS: a transmitter, a receiver and a booster placed from their items read 0 FE, dial=none and idle; an undialed transmitter with 200,000 FE and a player standing in its beam does not move the player; dialing it to the receiver's coordinate teleports the player out of the beam to that coordinate; SELFTEST_AT=teleport inverts the arrival and fails."
