//! RFTools Utility — the tank, as one `impl Hooks` at the crate root.
//!
//! The carried content is this package's own `assets/` and `data/` (`rftoolsutility:tank`, its
//! item, the nine fill models and the recipe); the behaviour is here: a [`BlockEntityPart`] on
//! the tank declares its block entity and ticks on the server to mirror the tank's contents into
//! the block's `level` state; `use_item_on_block` takes a bucket click (the port moves the bucket
//! itself, and hands a container it does not model to the engine's [`fluid_util`]); `block_drops`
//! puts the tank's contents on its own item, which `TankItemPart` clamps back when placed; and
//! `/rftoolsutility info|tank` serves the harness.
//!
//! The numbers — the capacity, the fill level, one fluid at a time, what a bucket click does —
//! are in [`runtime`], a plain module with no `mod_api` types, byte-identical to the retired
//! `.hnmod` bundle's `runtime.rs`, so `cargo test` alone proves them.
use mod_api::{
    self, BlockClick, BlockDropsEvent, BlockEntityPart, BlockEntityRef, BlockEntityTickEvent,
    BlockEntityTickReply, BlockEntityTicksEvent, BlockPart, Command, CommandArgumentValue,
    CommandSource, Compound, Direction, Flow, FluidResource, FluidStack, Hand, Hooks, HostSide,
    ITEM_FLUIDS, Identifier, InteractionResult, ItemKey, ItemPart, ItemPlace, ItemStack, LevelHost,
    LevelRef, ModFluidTanks, OnBlockPlacedByEvent, PlayerRef, RegisterCommandsEvent, Registrar,
    RegistrationContext, RunModCommandEvent, ServerHost, ServerTickEvent, Tag, Text,
    UseItemOnBlockEvent, UsePhase, fluid_util,
};
use std::sync::{Mutex, OnceLock};

pub mod runtime;

use runtime::{BUCKET, CAPACITY, Click, Tank, bucket_click, carried, clamped, fill_level};

/// The tank block and its item (`TankModule.TANK`).
const TANK: &str = "rftoolsutility:tank";
/// The block entity type the port declares for it (`EntityBlock.newBlockEntity`).
const TANK_TYPE: &str = "tank";
/// The block entity type's registry id, which `minecraft:block_entity_data` tags its payload with
/// (`TypedEntityData.loadInto` only loads data whose `id` names the block entity it is placed as).
const TANK_TYPE_ID: &str = "rftoolsutility:tank";
/// The block's fill-level property (`TankTE.computeLevel` → `tank<level>`).
const KEY_LEVEL: &str = "level";
/// The tank's index in its fluid table (upstream has one tank).
const TANK_INDEX: usize = 0;
/// The vanilla component that moves a block entity's saved data on its item
/// (`BlockEntity.applyComponentsFromItemStack`).
const BLOCK_ENTITY_DATA: &str = "minecraft:block_entity_data";

/// The tank's item, resolved once in `register`: `block_drops` runs with a read-only level host,
/// which cannot name an item, and a drop is only given the tank's contents when it is the tank.
static TANK_ITEM: OnceLock<ItemKey> = OnceLock::new();

/// The hand swaps a bucket move still owes, one per hand. A player-inventory write made inside the
/// click's own transaction is undone when that transaction ends (measured at this pin), so the
/// swap waits for the next server tick; that is also when the client is sent the slot
/// (`ContainerSetSlot`, vanilla's `broadcastChanges`) — the retired bundle's one-tick resend. An
/// entry is dropped when the hand no longer holds the item the click was made with, so a newer
/// click or a pick-up is never clobbered.
static PENDING_SWAP: Mutex<std::collections::BTreeMap<(u128, u8), PendingSwap>> =
    Mutex::new(std::collections::BTreeMap::new());

/// A container swap the hand still owes, and the item it expects to find there: a swap whose item
/// the player has already changed by hand is dropped rather than clobbering the newer item.
struct PendingSwap {
    player: u128,
    hand: Hand,
    expected: String,
    result: &'static str,
}

fn hand_key(hand: Hand) -> u8 {
    match hand {
        Hand::Main => 0,
        Hand::Off => 1,
    }
}

// ---------------------------------------------------------------------------
// The tank in the block entity's own compound
// ---------------------------------------------------------------------------

/// A compound's key for `name`. [`Compound`]'s key type is UTF-16 and the API does not export
/// it, so the key is taken from a string tag's own payload: the same conversion a mod's NBT
/// string uses, with no name needed.
fn key_tag(name: &str) -> Tag {
    Tag::String(name.into())
}

fn tag<'a>(nbt: &'a Compound, name: &str) -> Option<&'a Tag> {
    let Tag::String(utf16) = key_tag(name) else {
        return None;
    };
    nbt.get(&utf16)
}

fn put(nbt: &mut Compound, name: &str, value: Tag) {
    if let Tag::String(utf16) = key_tag(name) {
        nbt.insert(utf16, value);
    }
}

/// The engine's saved tanks (`read_tanks`: a `fluids` list of `{Fluid, Amount}` compounds,
/// `minecraft:empty` for an empty tank), as the runtime's model of one tank.
fn tank_of(nbt: &Compound) -> Tank {
    let Some(Tag::List(list)) = tag(nbt, "fluids") else {
        return Tank::new();
    };
    let Some(Tag::Compound(entry)) = list.first() else {
        return Tank::new();
    };
    let fluid = match tag(entry, "Fluid") {
        Some(Tag::String(fluid)) if fluid.to_string_lossy() != "minecraft:empty" => {
            Some(fluid.to_string_lossy())
        }
        _ => return Tank::new(),
    };
    let amount = match tag(entry, "Amount") {
        Some(Tag::Int(amount)) => *amount,
        _ => 0,
    };
    Tank { fluid, amount }
}

/// Write the tank back in the engine's `fluids` shape, for `/rftoolsutility tank`.
fn write_tank(nbt: &mut Compound, tank: &Tank) {
    let mut entry = Compound::new();
    put(
        &mut entry,
        "Fluid",
        Tag::String(tank.fluid.as_deref().unwrap_or("minecraft:empty").into()),
    );
    put(&mut entry, "Amount", Tag::Int(tank.amount));
    put(nbt, "fluids", Tag::List(vec![Tag::Compound(entry)]));
}

/// The `minecraft:block_entity_data` payload a broken tank's item carries: the block entity's own
/// saved data, tagged with its type id (`TypedEntityData.loadInto` loads a payload only when its
/// `id` names the block entity it is placed as). This is upstream's
/// `collectImplicitComponents` moving the fluid handler onto the item.
fn carried_data(saved: &Compound) -> Compound {
    let mut payload = saved.clone();
    put(&mut payload, "id", Tag::String(TANK_TYPE_ID.into()));
    payload
}

/// The component patch that puts `payload` on a dropped tank's item.
fn carried_patch(saved: &Compound) -> Compound {
    let mut patch = Compound::new();
    put(
        &mut patch,
        BLOCK_ENTITY_DATA,
        Tag::Compound(carried_data(saved)),
    );
    patch
}

/// What `/rftoolsutility info` prints. The shape is the harness's contract
/// (`tools/parse_info.py`), so the end-to-end scenario can parse it.
fn readout(
    pos: [i32; 3],
    tank: &Tank,
    held: Option<(String, u32)>,
    time: i64,
    nonce: Option<&str>,
) -> String {
    let fluid = tank.fluid.as_deref().unwrap_or("empty");
    let held = match held {
        Some((item, count)) => format!("{item} x{count}"),
        None => "empty".to_owned(),
    };
    let suffix = nonce.map_or(String::new(), |nonce| format!(" nonce={nonce}"));
    format!(
        "Tank at {pos:?}: {fluid} {}/{CAPACITY} mB (level {}), held {held}, t={time}{suffix}",
        tank.amount,
        tank.level()
    )
}

/// What a bucket move leaves behind, for the sound and game event: the fluid that moved and
/// whether it came out of the tank (`FluidUtil.triggerSoundAndGameEvent`).
fn moved_fluid(click: Click, tank: &Tank) -> (&'static str, bool) {
    match click {
        Click::Fill { fluid, .. } => (fluid, false),
        Click::Drain { .. } => (
            match tank.fluid.as_deref() {
                Some("minecraft:lava") => "minecraft:lava",
                _ => "minecraft:water",
            },
            true,
        ),
        Click::Refused | Click::NotYours => ("minecraft:water", false),
    }
}

// ---------------------------------------------------------------------------
// Reading the world
// ---------------------------------------------------------------------------

/// The tank at `pos`, if the block there is ours, read from its own saved compound: a block
/// entity is out of the world while the command or the tick runs, so the engine's fluid table
/// is not the place to read it.
fn tank_at(host: &mut dyn ServerHost, level: LevelRef, pos: [i32; 3]) -> Option<Tank> {
    let block = level.block(&*host, pos)?;
    if block.id(&*host) != TANK {
        return None;
    }
    let entity = host.block_entity(level, pos)?;
    Some(tank_of(&entity.data(&*host)?))
}

/// The tank block's entity at `pos`, when the block there is ours.
fn tank_entity(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
) -> Option<BlockEntityRef> {
    let block = level.block(&*host, pos)?;
    (block.id(&*host) == TANK)
        .then(|| host.block_entity(level, pos))
        .flatten()
}

/// Write the tank's fill level into the block state. `set_block_property` writes only when it
/// differs, so the tick does not dirty a chunk that did not change.
fn mirror_level(host: &mut dyn ServerHost, pos: [i32; 3], tank: &Tank) {
    host.set_block_property(pos, KEY_LEVEL, &fill_level(tank.amount).to_string());
}

/// Replace the tank's contents in its own saved compound (`FluidTank.fill`/`drain`'s write).
fn store_tank(host: &mut dyn ServerHost, level: LevelRef, pos: [i32; 3], tank: &Tank) -> bool {
    let Some(entity) = tank_entity(host, level, pos) else {
        return false;
    };
    let Some(mut data) = entity.data(&*host) else {
        return false;
    };
    write_tank(&mut data, tank);
    if entity.replace_data(host, data).is_err() {
        return false;
    }
    mirror_level(host, pos, tank);
    true
}

/// Put `item` in the player's `hand`, as upstream's `player.setItemInHand` does after a
/// container move. The write is the engine's own inventory write (`/item replace`'s), sent to the
/// client with the tick's inventory sync. It runs from `server_tick`, not from the click: both
/// `ItemPlace::PlayerHand.exchange` and this write are undone when the click's own transaction
/// ends at this pin (measured in game).
fn set_hand(host: &mut dyn ServerHost, player: PlayerRef, hand: Hand, item: &str) -> bool {
    let Some(key) = Identifier::parse(item)
        .ok()
        .and_then(|id| host.item_key(&id))
    else {
        return false;
    };
    let slot = match hand {
        Hand::Main => u32::try_from(player.selected_slot(&*host).unwrap_or(0)).ok(),
        Hand::Off => Some(40),
    };
    let Some(slot) = slot else {
        return false;
    };
    player
        .set_inventory_slot(&mut *host, slot, Some(ItemStack::of(key)))
        .is_ok()
}

/// The bucket's fill or empty sound and game event at the tank's centre (`FluidUtil` plays them
/// for the move it makes; the port's own move plays the same ones).
fn pour(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
    player: PlayerRef,
    fluid: &str,
    pickup: bool,
) {
    let Some(key) = host.fluid_key(fluid) else {
        return;
    };
    let moved = FluidStack::new(key, u32::try_from(BUCKET).unwrap_or(1_000));
    let at = pos.map(|c| f64::from(c) + 0.5);
    fluid_util::trigger_sound_and_game_event(
        host,
        &FluidResource::of(&moved),
        level,
        at,
        Some(player),
        pickup,
    );
}

/// The tank's own bucket move: the tank takes or gives one whole bucket now, and the hand's new
/// container is queued for the next server tick (`PENDING_SWAP`). A refused tank write leaves the
/// hand untouched.
fn bucket_move(
    host: &mut dyn ServerHost,
    click: &BlockClick<'_>,
    decision: Click,
    tank: &Tank,
) -> bool {
    let Some(player) = host.player(click.player) else {
        return false;
    };
    let Some(level) = player.level(&*host) else {
        return false;
    };
    let (next, result) = match decision {
        // `bucket_click` only answers `Fill` when a whole bucket fits, so the fill takes all of it.
        Click::Fill { fluid, result } => {
            let mut next = tank.clone();
            next.insert(fluid, BUCKET);
            (next, result)
        }
        Click::Drain { result } => {
            let mut next = tank.clone();
            next.extract(BUCKET);
            (next, result)
        }
        Click::Refused | Click::NotYours => return false,
    };
    if !store_tank(host, level, click.pos, &next) {
        return false;
    }
    // The container the hand becomes is written on the next tick (see `PENDING_SWAP`).
    let expected = click
        .stack
        .map(|stack| stack.item().id(&*host))
        .unwrap_or_default();
    // One swap per hand: a click delivered twice for one action must not stack two swaps.
    PENDING_SWAP
        .lock()
        .expect("the swap queue is never poisoned")
        .insert(
            (player.uuid(), hand_key(click.hand)),
            PendingSwap {
                player: player.uuid(),
                hand: click.hand,
                expected,
                result,
            },
        );
    let (fluid, pickup) = moved_fluid(decision, tank);
    pour(host, level, click.pos, player, fluid, pickup);
    true
}

/// Whether the player's hand holds a fluid container the engine knows: vanilla's buckets and any
/// a mod registered through the item-fluid capability (`FluidUtil`'s own test).
fn is_fluid_container(host: &mut dyn ServerHost, player: PlayerRef, hand: Hand) -> bool {
    ItemPlace::PlayerHand {
        player: player.uuid(),
        hand,
    }
    .capability(host, &ITEM_FLUIDS)
    .is_some()
}

// ---------------------------------------------------------------------------
// The tank part: its block entity, its tick and its bucket click
// ---------------------------------------------------------------------------

/// The tank: its block entity (ticking, one 32,000 mB tank) and its click.
struct TankPart;

#[mod_api::horizon_part]
impl BlockPart for TankPart {}

#[mod_api::horizon_part]
impl BlockEntityPart for TankPart {
    /// The tank's block entity ticks on the server (`EntityBlock.getTicker`).
    fn block_entity_ticks(&self, _host: &dyn mod_api::Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick: the block's `level` state follows the tank. Upstream did this in the
    /// tank's `onContentsChanged`; the pin's tank is written by pipes and by the hand alike, so
    /// the tick is the one place that sees every write. This part writes nothing into the block
    /// entity's own payload, so it answers `Unchanged` and leaves the chunk alone.
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        *event.reply = BlockEntityTickReply::Unchanged;
        let tank = tank_of(&event.entity.nbt);
        mirror_level(host, event.pos, &tank);
    }
}

#[derive(Default)]
struct Mod;

/// The tank's item: a placed tank takes the contents the item carries
/// (`BlockItemExtension.onBlockPlacedBy`, after vanilla applied the item's
/// `minecraft:block_entity_data`).
struct TankItemPart;

#[mod_api::horizon_part]
impl ItemPart for TankItemPart {
    /// Clamp a placed tank to its capacity, as `FluidTank.setFluid` clamps a loaded amount, and
    /// put the fill level on the block at once. A plain tank item carries nothing and is left as
    /// the game placed it.
    fn on_block_placed_by(&self, host: &mut dyn ServerHost, event: &mut OnBlockPlacedByEvent<'_>) {
        let Some(player) = event.placer else {
            return;
        };
        let Some(level) = player.level(&*host) else {
            return;
        };
        let Some(entity) = tank_entity(host, level, event.pos) else {
            return;
        };
        let Some(data) = entity.data(&*host) else {
            return;
        };
        let tank = tank_of(&data);
        let restored = clamped(&tank);
        if restored != tank {
            store_tank(host, level, event.pos, &restored);
        }
    }
}

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Declare the tank's block entity and its one tank (`EntityBlock.newBlockEntity`,
    /// `Capabilities.Fluid.BLOCK`).
    fn register(&self, r: &mut dyn Registrar, cx: &RegistrationContext) {
        let block = r.block_key(TANK);
        let item = r.item_key(TANK);
        // `block_drops` has no game host, so the one item it may write to is resolved here.
        let _ = TANK_ITEM.set(item);
        r.item_part(item, TankItemPart);
        let kind = r.block_entity_type(TANK_TYPE).blocks(&[TANK]).build();
        // Only our own duplicate declaration can refuse here, and without the part the block
        // would sit inert: refuse the mod's registration rather than load half a tank.
        if let Err(error) = r.block_entity_part(&kind, TankPart) {
            cx.refuse(&format!(
                "rftoolsutility: the tank part was refused: {error}"
            ));
            return;
        }
        let all = vec![TANK_INDEX];
        let mut tanks = ModFluidTanks::default();
        tanks.tanks = vec![CAPACITY];
        tanks.faces = [
            all.clone(),
            all.clone(),
            all.clone(),
            all.clone(),
            all.clone(),
            all,
        ];
        tanks.inputs = vec![TANK_INDEX];
        tanks.outputs = vec![TANK_INDEX];
        r.block_fluids(&block, tanks);
    }

    /// `UseItemOnBlockEvent` at the block phase: the tank's container click, taken before the
    /// block's own use and before the item's `useOn`, so a water bucket does not also empty onto
    /// the ground (the engine runs the item's `useOn` even when a block part answered, measured at
    /// this pin). A click that is not a container is left to vanilla.
    fn use_item_on_block(&self, host: HostSide<'_>, event: &mut UseItemOnBlockEvent<'_>) -> Flow {
        if event.phase != UsePhase::Block {
            return Flow::Continue;
        }
        let host = match host {
            HostSide::Server(host) => host,
            HostSide::Client(client) => {
                // The client predicts nothing for a bucket on the tank: the tank write and the
                // hand swap are the server's, and a prediction here empties the bucket locally
                // (this client's copy of the tank's contents may be stale).
                let Some(stack) = event.click.stack else {
                    return Flow::Continue;
                };
                let held = stack.item().id(client);
                if matches!(
                    bucket_click(&held, &Tank::new()),
                    Click::Fill { .. } | Click::Refused
                ) {
                    *event.cancellation_result = InteractionResult::Success;
                    return Flow::Cancel;
                }
                return Flow::Continue;
            }
        };
        let click = event.click;
        let Some(player) = host.player(click.player) else {
            return Flow::Continue;
        };
        let Some(level) = player.level(&*host) else {
            return Flow::Continue;
        };
        let Some(stack) = click.stack else {
            return Flow::Continue;
        };
        let held = stack.item().id(&*host);
        let Some(tank) = tank_at(host, level, click.pos) else {
            return Flow::Continue;
        };
        let decision = bucket_click(&held, &tank);
        let accepted = match decision {
            Click::NotYours => {
                // A container the engine knows and the runtime does not (a mod's own): the
                // engine's `FluidUtil` moves it, as upstream's `FluidUtil` did.
                if !is_fluid_container(host, player, click.hand) {
                    return Flow::Continue;
                }
                let side = Direction::ALL
                    .into_iter()
                    .find(|direction| direction.normal() == click.hit.face);
                fluid_util::interact_with_fluid_handler(
                    host, player, click.hand, level, click.pos, side,
                )
            }
            // A refused container click is consumed and does nothing: upstream answers `PASS`,
            // which empties the bucket beside the tank (the recorded deviation).
            Click::Refused => true,
            move_it => bucket_move(host, click, move_it, &tank),
        };
        *event.cancellation_result = if accepted {
            InteractionResult::Success
        } else {
            InteractionResult::Fail
        };
        Flow::Cancel
    }

    /// Apply the hand swaps a bucket move queued (`PENDING_SWAP`).
    fn server_tick(&self, host: &mut dyn ServerHost, _event: &mut ServerTickEvent<'_>) {
        let swaps: Vec<PendingSwap> = std::mem::take(
            &mut *PENDING_SWAP
                .lock()
                .expect("the swap queue is never poisoned"),
        )
        .into_values()
        .collect();
        for swap in swaps {
            let Some(player) = host.player(swap.player) else {
                continue;
            };
            let now = host
                .living(player.entity())
                .and_then(|living| living.hand_item(&*host, swap.hand))
                .map(|stack| stack.item().id(&*host));
            if now.as_deref() != Some(swap.expected.as_str()) {
                continue;
            }
            set_hand(host, player, swap.hand, swap.result);
        }
    }

    /// `BlockDropsEvent`: a broken tank's own item carries the tank's contents, so placing the
    /// item puts them back (`TankTE`'s loot table copies `mcjtylib:item_fluids`; the port moves
    /// the same data as the vanilla `minecraft:block_entity_data` component, which vanilla's
    /// placement applies). An empty tank's item carries nothing.
    fn block_drops(&self, level: &dyn LevelHost, event: &mut BlockDropsEvent<'_>) -> Flow {
        if level.state_block(event.state) != Some(TANK) {
            return Flow::Continue;
        }
        let Some(entity) = event.block_entity else {
            return Flow::Continue;
        };
        let Some(saved) = level.block_entity_data(entity) else {
            return Flow::Continue;
        };
        if carried(&tank_of(&saved)).is_none() {
            return Flow::Continue;
        }
        let Some(item) = TANK_ITEM.get() else {
            return Flow::Continue;
        };
        let patch = carried_patch(&saved);
        for drop in event.drops.iter_mut() {
            if drop.stack.item() == *item {
                let _ = drop.stack.apply_components(&patch);
            }
        }
        Flow::Continue
    }

    /// `/rftoolsutility info|tank` (upstream has no command: its controls are the GUI and the
    /// filter slot, and this slice has neither, so the harness needs a readout and a way to set
    /// the tank without 32 buckets — the `/rftoolsbase energy` precedent).
    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        event.commands.push(Command::new(
            "rftoolsutility info <pos:block_pos> [nonce:word]",
        ));
        event.commands.push(Command::op(
            "rftoolsutility tank <pos:block_pos> <fluid:id> <mb:integer>",
            2,
        ));
    }

    /// Run the `/rftoolsutility` commands a player types.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        let invocation = event.command;
        let literals: Vec<&str> = invocation.literals.iter().map(String::as_str).collect();
        if !matches!(literals.as_slice(), ["rftoolsutility", "info" | "tank"]) {
            return Flow::Continue;
        }
        let Some(CommandArgumentValue::BlockPos(pos)) = invocation.argument("pos") else {
            feedback(
                host,
                "Usage: /rftoolsutility info <pos> [nonce] or /rftoolsutility tank <pos> <fluid> <mb>",
            );
            return Flow::Cancel;
        };
        let pos = *pos;
        let Some(level) = host
            .levels()
            .list_levels()
            .into_iter()
            .find(|level| level.dimension() == invocation.dimension)
        else {
            feedback(host, "No tank there.");
            return Flow::Cancel;
        };
        match literals.as_slice() {
            ["rftoolsutility", "info"] => {
                let Some(tank) = tank_at(host, level, pos) else {
                    feedback(host, "No tank there.");
                    return Flow::Cancel;
                };
                // The harness passes a nonce so it can tell its own reply from an older one
                // still in the client's chat.
                let nonce = match invocation.argument("nonce") {
                    Some(CommandArgumentValue::String(word)) => Some(word.as_str()),
                    _ => None,
                };
                let held = held_of(host, event.source);
                let time = level.game_time(&*host).unwrap_or(0);
                feedback(host, &readout(pos, &tank, held, time, nonce));
            }
            ["rftoolsutility", "tank"] => {
                // `<fluid:id>` is a free identifier, which the engine delivers as a `String`
                // (`resource:<registry>` would deliver an `Id`); accept either so the usage and
                // the dispatcher cannot disagree.
                let fluid = match invocation.argument("fluid") {
                    Some(CommandArgumentValue::String(fluid)) => fluid.clone(),
                    Some(CommandArgumentValue::Id(fluid)) => fluid.to_string(),
                    _ => {
                        feedback(host, "Usage: /rftoolsutility tank <pos> <fluid> <mb>");
                        return Flow::Cancel;
                    }
                };
                let Some(CommandArgumentValue::Integer(mb)) = invocation.argument("mb") else {
                    feedback(host, "Usage: /rftoolsutility tank <pos> <fluid> <mb>");
                    return Flow::Cancel;
                };
                let Some(entity) = tank_entity(host, level, pos) else {
                    feedback(host, "No tank there.");
                    return Flow::Cancel;
                };
                let Some(mut data) = entity.data(&*host) else {
                    feedback(host, "No tank there.");
                    return Flow::Cancel;
                };
                // A harness control, not a player action: it replaces the contents, clamped to
                // the capacity, instead of merging with what the tank holds.
                let tank = command_tank(&fluid, *mb);
                write_tank(&mut data, &tank);
                if entity.replace_data(host, data).is_err() {
                    feedback(host, "No tank there.");
                    return Flow::Cancel;
                }
                mirror_level(host, pos, &tank);
                feedback(
                    host,
                    &format!(
                        "Tank at {pos:?}: {} {}/{CAPACITY} mB.",
                        tank.fluid.as_deref().unwrap_or("empty"),
                        tank.amount
                    ),
                );
            }
            _ => return Flow::Continue,
        }
        Flow::Cancel
    }
}

/// `/rftoolsutility tank <pos> <fluid> <mb>`'s write: the tank it leaves, clamped to the capacity
/// and empty for a non-positive amount (the command replaces, it does not merge).
fn command_tank(fluid: &str, mb: i32) -> Tank {
    let amount = mb.clamp(0, CAPACITY);
    if amount == 0 {
        Tank::new()
    } else {
        Tank {
            fluid: Some(fluid.to_owned()),
            amount,
        }
    }
}

/// The hand of the command's player, as `item x count` (`Game.held` in the retired bundle).
fn held_of(host: &mut dyn ServerHost, source: &CommandSource) -> Option<(String, u32)> {
    let CommandSource::Player { player, .. } = *source else {
        return None;
    };
    let stack = host
        .living(player.entity())?
        .hand_item(&*host, Hand::Main)?;
    Some((stack.item().id(&*host), stack.count()))
}

/// The command's success feedback, routed to its originating source (the swapped-in player, the
/// console, a command block).
fn feedback(host: &mut dyn ServerHost, line: &str) {
    let _ = host.commands().feedback(Text::literal(line));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn water(amount: i32) -> Tank {
        Tank {
            fluid: Some("minecraft:water".to_owned()),
            amount,
        }
    }

    #[test]
    fn the_engine_s_saved_tank_reads_back() {
        let nbt = Tag::compound([(
            "fluids",
            Tag::List(vec![Tag::compound([
                ("Fluid", Tag::String("minecraft:water".into())),
                ("Amount", Tag::Int(1_000)),
            ])]),
        )]);
        let Tag::Compound(nbt) = nbt else {
            unreachable!("the fixture is a compound")
        };
        assert_eq!(tank_of(&nbt), water(runtime::BUCKET));
    }

    #[test]
    fn an_empty_or_odd_compound_is_an_empty_tank() {
        assert_eq!(tank_of(&Compound::new()), Tank::new());
        let empty = Tag::compound([(
            "fluids",
            Tag::List(vec![Tag::compound([
                ("Fluid", Tag::String("minecraft:empty".into())),
                ("Amount", Tag::Int(0)),
            ])]),
        )]);
        let Tag::Compound(empty) = empty else {
            unreachable!("the fixture is a compound")
        };
        assert_eq!(tank_of(&empty), Tank::new());
        let odd = Tag::compound([("fluids", Tag::Int(3))]);
        let Tag::Compound(odd) = odd else {
            unreachable!("the fixture is a compound")
        };
        assert_eq!(tank_of(&odd), Tank::new());
    }

    #[test]
    fn a_written_tank_reads_back() {
        for tank in [Tank::new(), water(runtime::BUCKET), water(CAPACITY)] {
            let mut nbt = Compound::new();
            write_tank(&mut nbt, &tank);
            assert_eq!(tank_of(&nbt), tank);
        }
    }

    #[test]
    fn the_readout_is_the_line_the_harness_parses() {
        assert_eq!(
            readout(
                [200, 101, 0],
                &water(runtime::BUCKET),
                Some(("minecraft:bucket".to_owned(), 1)),
                264,
                Some("1"),
            ),
            "Tank at [200, 101, 0]: minecraft:water 1000/32000 mB (level 1), \
             held minecraft:bucket x1, t=264 nonce=1"
        );
        assert_eq!(
            readout([1, 2, 3], &Tank::new(), None, 7, None),
            "Tank at [1, 2, 3]: empty 0/32000 mB (level 0), held empty, t=7"
        );
    }

    #[test]
    fn a_non_container_is_not_the_tank_s_click() {
        assert_eq!(
            bucket_click("minecraft:stone", &Tank::new()),
            Click::NotYours
        );
        assert_eq!(
            bucket_click("minecraft:water_bucket", &water(CAPACITY)),
            Click::Refused
        );
        assert_eq!(
            bucket_click("minecraft:bucket", &Tank::new()),
            Click::Refused
        );
    }

    #[test]
    fn a_bucket_that_moves_names_the_container_the_hand_gets() {
        assert_eq!(
            bucket_click("minecraft:water_bucket", &Tank::new()),
            Click::Fill {
                fluid: "minecraft:water",
                result: "minecraft:bucket"
            }
        );
        assert_eq!(
            bucket_click("minecraft:bucket", &water(runtime::BUCKET)),
            Click::Drain {
                result: "minecraft:water_bucket"
            }
        );
    }

    #[test]
    fn the_carried_payload_is_the_vanilla_block_entity_data() {
        let mut nbt = Compound::new();
        write_tank(&mut nbt, &water(5_000));
        let patch = carried_patch(&nbt);
        let Some(Tag::Compound(payload)) = tag(&patch, BLOCK_ENTITY_DATA) else {
            panic!("the patch has no {BLOCK_ENTITY_DATA}");
        };
        assert_eq!(
            tag(payload, "id"),
            Some(&Tag::String(TANK_TYPE_ID.into())),
            "vanilla's TypedEntityData needs the type id"
        );
        assert_eq!(
            tank_of(payload),
            water(5_000),
            "the tank's own saved shape travels unchanged"
        );
    }

    #[test]
    fn the_tank_command_replaces_and_clamps() {
        assert_eq!(
            command_tank("minecraft:water", 40_000),
            water(CAPACITY),
            "a request above the capacity is clamped, not refused"
        );
        assert_eq!(
            command_tank("minecraft:lava", 1_000),
            Tank {
                fluid: Some("minecraft:lava".to_owned()),
                amount: 1_000,
            }
        );
        assert_eq!(command_tank("minecraft:water", 0), Tank::new());
        assert_eq!(command_tank("minecraft:water", -5), Tank::new());
    }

    #[test]
    fn an_empty_tank_has_nothing_to_carry() {
        let mut nbt = Compound::new();
        write_tank(&mut nbt, &Tank::new());
        assert_eq!(carried(&tank_of(&nbt)), None);
        assert_eq!(carried(&tank_of(&Compound::new())), None);
    }

    #[test]
    fn the_level_a_tick_mirrors_covers_every_value() {
        let levels: Vec<String> = [0, 1, 3_999, 4_000, CAPACITY]
            .iter()
            .map(|amount| fill_level(*amount).to_string())
            .collect();
        assert_eq!(levels, vec!["0", "1", "1", "2", "8"]);
    }
}
