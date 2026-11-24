//! RFTools Utility — the tank, the crafter and the spawner, as one `impl Hooks` at the crate
//! root.
//!
//! The carried content is this package's own `assets/` and `data/`; the behaviour is here.
//! [`TankPart`] and [`TankItemPart`] are the tank: a block entity whose tick mirrors its one
//! 32,000 mB tank into the block's `level` state, a bucket click the port performs itself, and
//! the contents travelling on the item. [`CrafterPart`] is the crafter: 41 slots, 50,000 FE and
//! a tick that runs taught 3x3 recipes from its buffer. [`SpawnerPart`] is the spawner:
//! 200,000 FE and a tick that spawns the mob its own recipe names, for that recipe's RF cost;
//! [`SpawnerRecipeKind`] is the `rftoolsutility:spawner` recipe kind its 70 carried mob recipes
//! need. [`SensorPart`], [`SequencerPart`], [`TimerPart`] and [`CounterPart`] are the four logic
//! blocks: a ten-tick block check, a rising-edge-driven 64-step pattern, a rising-edge-reloaded
//! delay that emits a one-tick 15 pulse every `delay` ticks, and a rising-edge pulse counter that
//! latches 15 when its count reaches the limit, all emitting a real 15/0 redstone signal.
//! [`AnalogPart`] is the last redstone-output sibling: it compares two side inputs and emits the
//! artefact's own multiply-and-offset arithmetic clamped to 0..15. (Its display-only sibling
//! `digit` is not ported: upstream's whole behaviour is a client renderer, with no redstone
//! output for a fail-capable lamp assertion.) The `/rftoolsutility …` commands serve the harness,
//! because upstream's controls are its GUIs.
//!
//! The numbers — the capacity, the fill level, what a bucket click does, the craft loop, the
//! spawn cost — are in [`runtime`], a plain module with no `mod_api` types, so `cargo test`
//! alone proves them.
use mod_api::{
    self, BlockClick, BlockDropsEvent, BlockEntityPart, BlockEntityRef, BlockEntityTickEvent,
    BlockEntityTickReply, BlockEntityTicksEvent, BlockPart, BlockSignalEvent, Command,
    CommandArgumentValue, CommandSource, Compound, CraftingGrid, CraftingRecipeCodec,
    CraftingRecipeKind, Direction, Flow, FluidResource, FluidStack, Hand, Hooks, Host,
    HostBlockStates, HostSide, HostTransaction, ITEM_FLUIDS, Identifier, InteractionResult,
    IsSignalSourceEvent, ItemKey, ItemPart, ItemPlace, ItemStack, LevelHost, LevelRef,
    ModContainer, ModEnergyBuffer, ModFluidTanks, NeighbourUpdate, OnBlockPlacedByEvent, PlayerRef,
    RecipeInput, RegisterCommandsEvent, Registrar, RegistrationContext, RunModCommandEvent,
    ServerHost, ServerTickEvent, SpawnReason, Tag, Text, UseItemOnBlockEvent, UsePhase, fluid_util,
};
use std::sync::{Mutex, OnceLock};

pub mod runtime;

use runtime::{
    AVERAGE_YIELD, AnalogData, BEAMER_CAPACITY, BEAMER_RECEIVE, BOOST_RF, BUCKET, BUFFER_IN,
    BUFFER_IN_SLOTS, BUFFER_OUT, BUFFER_OUT_SLOTS, CAPACITY, CRAFTER_CAPACITY, CRAFTER_RECEIVE,
    CRAFTER_SLOTS, Click, CounterData, CounterState, CraftMode, CrafterTier, GHOST_INPUT,
    GHOST_RESULT, HIGH_YIELD, LOW_YIELD, MATTER_STORAGE, MAX_BEAM_DISTANCE, MatterSource,
    MobRecipe, RECEIVER_CAPACITY, RECEIVER_RECEIVE, RF_PER_TICK, SENSOR_OFF, SENSOR_ON,
    SPAWNER_CAPACITY, SPAWNER_RECEIVE, SensorKind, SequencerData, SequencerMode, SequencerState,
    Speed, TRANSMITTER_CAPACITY, TRANSMITTER_RECEIVE, Tank, TimerData, TimerState, add_matter,
    analog_branch, analog_neighbours, analog_output, analyzer_due, analyzer_status, beam_due,
    beam_reach, beam_send, bucket_click, carried, clamped, counter_signal,
    counter_tick as run_counter_tick, fill_level, in_beam, placements, plan, receiver_ready,
    recipe_for, result_range, rising_edge, sensor_due, sensor_front, sensor_sees, sensor_signal,
    sequencer_tick as run_sequencer_tick, spawn_at, spawn_cycle, spend, start_cost, start_time,
    steps, timer_signal, timer_tick as run_timer_tick,
};

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

// ---------------------------------------------------------------------------
// The crafter (slice 3)
// ---------------------------------------------------------------------------

/// The crafter tiers (`CrafterModule.CRAFTER1..3`), one block entity with 2, 4 or 8 recipe
/// slots; the tier's own block id, type id and count live in [`runtime::CrafterTier`].
const CRAFTER_TIERS: [CrafterTier; 3] = CrafterTier::ALL;
/// The one recipe type the crafter matches (`RecipeType.CRAFTING`).
const RECIPE_TYPE: &str = "minecraft:crafting";
/// The crafter's saved keys beside the engine's own (`CrafterContainer`'s item slots are the
/// engine's `Items`, and the energy amount the engine's `energy` int).
const KEY_ENERGY: &str = "energy";
const KEY_RECIPES: &str = "recipes";
const KEY_SPEED: &str = "speed";
const KEY_MODE: &str = "mode";
const KEY_KEEP: &str = "keep";

/// The crafters' items, resolved once in `register` for the same reason as the tank's, one per
/// tier in [`CRAFTER_TIERS`]' order.
static CRAFTER_ITEMS: OnceLock<[ItemKey; 3]> = OnceLock::new();

// ---------------------------------------------------------------------------
// The spawner (slice 4)
// ---------------------------------------------------------------------------

/// The spawner block and its item (`SpawnerModule.SPAWNER`).
const SPAWNER: &str = "rftoolsutility:spawner";
/// The block entity type the port declares for it (`EntityBlock.newBlockEntity`).
const SPAWNER_TYPE: &str = "spawner";
/// The type id a `minecraft:block_entity_data` payload on a spawner's item carries
/// (`TypedEntityData.loadInto`).
const SPAWNER_TYPE_ID: &str = "rftoolsutility:spawner";
/// The recipe type the jar's 70 mob recipes name (`SpawnerModule.SPAWNER_RECIPE_TYPE_ID`).
const SPAWNER_RECIPE_TYPE: &str = "rftoolsutility:spawner";
/// The serializer name this port registers for it; `register_crafting_recipe_serializer` files
/// it under the mod's own namespace (`rftoolsutility:spawner`, the type the files name) and
/// under the `minecraft:crafting` recipe type, which is the only kind the API has.
const SPAWNER_KIND: &str = "spawner";
/// The spawner's saved keys beside the engine's own: the mob it spawns (`SpawnerData.mob`) and
/// the recipe numbers read for it (its `spawnRf` and its three matter amounts), so a tick needs
/// no recipe-table walk. `energy` is the engine's int, as the crafter's is.
const KEY_MOB: &str = "mob";
const KEY_COST: &str = "cost";
const KEY_NEEDS: &str = "needs";
/// The three matter stores (`SpawnerData.matter0..2`), one per ingredient the recipe asks for.
/// Upstream keeps them as three floats in the machine's own data and a matter beamer fills them;
/// this port keeps the same three floats in its saved compound (the shape [`KEY_NEEDS`] already
/// uses) and `/rftoolsutility matter` stands in for a beamer's send.
const KEY_MATTER: &str = "matter";
const KEY_SPAWNED: &str = "spawned";
/// The block's own placement state, which picks the side a spawn appears on
/// (`OrientationTools.getOrientation`).
const KEY_FACING: &str = "facing";

/// A spawner's item, resolved once in `register` for the same reason as the tank's.
static SPAWNER_ITEM: OnceLock<ItemKey> = OnceLock::new();

// ---------------------------------------------------------------------------
// The matter beamer (slice 8)
// ---------------------------------------------------------------------------

/// The beamer block and its item (`SpawnerModule.MATTER_BEAMER`).
const BEAMER: &str = "rftoolsutility:matter_beamer";
/// The block entity type the port declares for it (`EntityBlock.newBlockEntity`).
const BEAMER_TYPE: &str = "matter_beamer";
/// The type id a `minecraft:block_entity_data` payload on a beamer's item carries.
const BEAMER_TYPE_ID: &str = "rftoolsutility:matter_beamer";
/// The beamer's one material slot (`SLOT_MATERIAL`).
const BEAMER_SLOT: usize = 0;
/// The linked spawner's block position (`MatterBeamerTileEntity`'s `destination`).
const KEY_DEST: &str = "dest";
/// The sends the machine has made. Upstream has no such counter; the spawner's `spawned` is the
/// same harness readout, and it is what lets the end-to-end leg say `blocks` without guessing.
const KEY_SENT: &str = "sent";
/// The block's own glow state (`BlockStateProperties.LIT`, which the jar's blockstate keys its
/// two models on).
const KEY_LIT: &str = "lit";
/// The jar's own `living/*` tags: what a recipe's empty `object` yields
/// (`SpawnerRecipes.MobSpawnAmount.match`).
const HIGH_YIELD_TAG: &str = "rftoolsutility:living/highyield";
const AVERAGE_YIELD_TAG: &str = "rftoolsutility:living/averageyield";
const LOW_YIELD_TAG: &str = "rftoolsutility:living/lowyield";

/// A beamer's item, resolved once in `register` for the same reason as the tank's.
static BEAMER_ITEM: OnceLock<ItemKey> = OnceLock::new();

// ---------------------------------------------------------------------------
// The teleporter family (slice 9): matter transmitter / receiver / booster
// ---------------------------------------------------------------------------

/// `TeleporterModule.MATTER_TRANSMITTER`: the player teleporter's block and item.
const TRANSMITTER: &str = "rftoolsutility:matter_transmitter";
const TRANSMITTER_TYPE: &str = "matter_transmitter";
const TRANSMITTER_TYPE_ID: &str = "rftoolsutility:matter_transmitter";
/// `TeleporterModule.MATTER_RECEIVER`: the destination marker's block and item.
const RECEIVER: &str = "rftoolsutility:matter_receiver";
const RECEIVER_TYPE: &str = "matter_receiver";
const RECEIVER_TYPE_ID: &str = "rftoolsutility:matter_receiver";
/// `TeleporterModule.MATTER_BOOSTER`: the adjacent safety marker (a plain block).
const BOOSTER: &str = "rftoolsutility:matter_booster";
/// The dialed destination's dimension id and block coordinate (`MatterTransmitterData.destinationId`
/// is replaced by the direct `TeleportDestination`, upstream's own fallback when the coordinate
/// has no registry id — this port has no dialing-device GUI or global destination registry).
const KEY_DIAL_DIM: &str = "dial_dim";
const KEY_DIAL_POS: &str = "dial_pos";
/// The transmitter's in-progress teleport (`MatterTransmitterTileEntity`): the player's UUID, the
/// ticks left, the cooldown, the good/bad/total tick counts and the per-tick RF the start cost was
/// amortized into.
const KEY_TP_PLAYER: &str = "tpPlayer";
const KEY_TP_TIMER: &str = "tpTimer";
const KEY_COOLDOWN: &str = "cooldownTimer";
const KEY_TOTAL: &str = "totalTicks";
const KEY_GOOD: &str = "goodTicks";
const KEY_BAD: &str = "badTicks";
const KEY_RFPT: &str = "rfPerTick";

/// The transmitter's and receiver's items, resolved in `register` for `block_drops` (which has no
/// game host) as the other machines' items are.
static TRANSMITTER_ITEM: OnceLock<ItemKey> = OnceLock::new();
static RECEIVER_ITEM: OnceLock<ItemKey> = OnceLock::new();

// ---------------------------------------------------------------------------
// The destination analyzer (slice 10)
// ---------------------------------------------------------------------------

/// `TeleporterModule.DESTINATION_ANALYZER`: a **passive** block (`DestinationAnalyzerBlock` extends
/// `Block`, with only a `FACING` property — no block entity and no redstone output). Its whole
/// effect is the adjacency check in `DialingDeviceTileEntity.isDestinationAnalyzerAvailable`,
/// which lets a dialed transmitter check its receiver's status (`checkReceiverStatus`).
const ANALYZER: &str = "rftoolsutility:destination_analyzer";
/// The status a dialed transmitter keeps (`MatterTransmitterTileEntity.status`, the green/red/
/// yellow beam state the jar's lang line describes). The harness reads this through
/// `/rftoolsutility analyzer` as a signal strength; the block itself emits no redstone.
const KEY_STATUS: &str = "status";

// ---------------------------------------------------------------------------
// The sensor (slice 11) — `SensorTileEntity`
// ---------------------------------------------------------------------------

/// `LogicBlockModule.SENSOR`: a block entity whose 10-tick check emits 15 or 0 through the
/// redstone hook. The jar's 24 `logic_facing` slab rotations cannot be expressed by the content
/// schema, so the input side is a saved setting (the analyzer/booster facing deviation).
const SENSOR: &str = "rftoolsutility:sensor";
const SENSOR_TYPE: &str = "sensor";
const SENSOR_TYPE_ID: &str = "rftoolsutility:sensor";
const KEY_SENSOR_KIND: &str = "sensorKind";
const KEY_SENSOR_TARGET: &str = "sensorTarget";
const KEY_SENSOR_FACING: &str = "sensorFacing";
/// The state property that caches the signal for `BlockPart::block_signal` (vanilla asks it very
/// often, from inside the level's update, and the hook must not read the world).
const KEY_SIGNAL: &str = "signal";

/// A sensor's item, resolved once in `register` for `block_drops`.
static SENSOR_ITEM: OnceLock<ItemKey> = OnceLock::new();

// ---------------------------------------------------------------------------
// The sequencer (slice 12) — `SequencerTileEntity`
// ---------------------------------------------------------------------------

/// `LogicBlockModule.SEQUENCER`: the sibling of the sensor, a block entity with a 64-bit pattern,
/// seven modes and a real 15/0 redstone output. The input is a **rising edge**
/// (`pulse = (powerLevel > 0) && !prevIn`), which a vanilla lever on any side expresses through
/// `LevelRef::best_neighbor_signal`.
const SEQUENCER: &str = "rftoolsutility:sequencer";
const SEQUENCER_TYPE: &str = "sequencer";
const SEQUENCER_TYPE_ID: &str = "rftoolsutility:sequencer";
/// `SequencerData`'s five record fields, and the four live fields `SequencerTileEntity` saves
/// beside them (`rs`, `step`, `prevIn`, `timer`).
const KEY_SEQ_BITS: &str = "bits";
const KEY_SEQ_MODE: &str = "sequencerMode";
const KEY_SEQ_DELAY: &str = "delay";
const KEY_SEQ_COUNT: &str = "stepcount";
const KEY_SEQ_ENDSTATE: &str = "endstate";
const KEY_SEQ_STEP: &str = "step";
const KEY_SEQ_PREV: &str = "prevIn";
const KEY_SEQ_TIMER: &str = "timer";
const KEY_SEQ_RS: &str = "rs";
/// The cosmetic input-side setting (the jar's 24 `logic_facing` slab rotations cannot be a
/// content block's states); the engine reads the strongest adjacent signal, so the side is a
/// saved advisory, as the analyzer's and the booster's facing are.
const KEY_SEQ_FACING: &str = "seqFacing";
/// A sequencer's item, resolved once in `register` for `block_drops`.
static SEQUENCER_ITEM: OnceLock<ItemKey> = OnceLock::new();

// ---------------------------------------------------------------------------
// The timer (slice 13) — `TimerTileEntity`
// ---------------------------------------------------------------------------

/// `LogicBlockModule.TIMER`: the sensor's other sibling. Its `delay` and `pauses` are written by
/// the jar's GUI, so this port writes them with `/rftoolsutility timer-set`; the tick rule is the
/// artefact's own. The input side is a saved advisory for the same reason the sensor's is.
const TIMER: &str = "rftoolsutility:timer";
const TIMER_TYPE: &str = "timer";
const TIMER_TYPE_ID: &str = "rftoolsutility:timer";
/// `TimerData`'s two record fields and `TimerTileEntity`'s three live saved fields.
const KEY_TIMER_DELAY: &str = "delay";
const KEY_TIMER_PAUSES: &str = "pauses";
const KEY_TIMER_PREV: &str = "prevIn";
const KEY_TIMER_TIMER: &str = "timer";
const KEY_TIMER_RS: &str = "rs";
const KEY_TIMER_FACING: &str = "timerFacing";
/// Harness-only tick stamps: the game time of the last rising edge and of the last 15 output.
/// Upstream saves no such fields; they let the in-game leg measure the delay in the game's own
/// ticks instead of wall time (the beamer's `sent` counter is the same pattern).
const KEY_TIMER_PULSE_AT: &str = "timerPulseAt";
const KEY_TIMER_FIRED_AT: &str = "timerFiredAt";
/// A latch so the `fired` stamp is the **first** pulse after the last edge, not the latest of the
/// timer's repeating pulses (the leg needs the first one to measure the delay).
const KEY_TIMER_FIRED_LOCK: &str = "timerFiredLock";
/// The vanilla lamp's own `lit` state at the fire tick, read from the game right after the
/// neighbour update. A one-tick pulse is too brief for a polling `ctl block`; this is the lamp's
/// real state at the instant the timer emits 15, and the leg asserts it.
const KEY_TIMER_LAMP: &str = "timerLampLit";
/// A timer's item, resolved once in `register` for `block_drops`.
static TIMER_ITEM: OnceLock<ItemKey> = OnceLock::new();

// ---------------------------------------------------------------------------
// The counter (slice 14) — `CounterTileEntity`, `CounterData`
// ---------------------------------------------------------------------------

/// `LogicBlockModule.COUNTER`: the timer's sibling. Its `counter` (limit) and `current` are written
/// only by the jar's GUI (`GuiCounter`'s `integerfield`s), so this port writes the limit with
/// `/rftoolsutility counter-set`; the tick rule is the artefact's own. The input is a rising edge on
/// the input-side neighbour, read directionally as the sensor/sequencer/timer read theirs.
const COUNTER: &str = "rftoolsutility:counter";
const COUNTER_TYPE: &str = "counter";
const COUNTER_TYPE_ID: &str = "rftoolsutility:counter";
/// `CounterData`'s two fields and the two live fields `CounterTileEntity` saves (`rs`, `prevIn`).
const KEY_COUNTER_LIMIT: &str = "counter";
const KEY_COUNTER_CURRENT: &str = "current";
const KEY_COUNTER_PREV: &str = "prevIn";
const KEY_COUNTER_RS: &str = "rs";
const KEY_COUNTER_FACING: &str = "counterFacing";
/// Harness-only stamps: the game time of the last rising edge and of the fire that reached the
/// limit, plus a latch so `fire` is the *first* fire after a `counter-set`. The lamp stamp is the
/// adjacent vanilla lamp's own `lit` state at the fire instant — a game-time observation the leg
/// asserts instead of wall time or an absence of errors.
const KEY_COUNTER_PULSE_AT: &str = "counterPulseAt";
const KEY_COUNTER_FIRE_AT: &str = "counterFireAt";
const KEY_COUNTER_FIRED_LOCK: &str = "counterFiredLock";
const KEY_COUNTER_LAMP: &str = "counterLampLit";
/// The number of rising edges since the last `counter-set`, so the leg can assert "only on the
/// limit-th pulse" from the reply itself.
const KEY_COUNTER_PULSES: &str = "counterPulses";
/// A counter's item, resolved once in `register` for `block_drops`.
static COUNTER_ITEM: OnceLock<ItemKey> = OnceLock::new();

// ---------------------------------------------------------------------------
// The analog (slice 15) — `AnalogTileEntity`, `AnalogData`
// ---------------------------------------------------------------------------

/// `LogicBlockModule.ANALOG`: the last redstone-output logic sibling. Its six arithmetic settings
/// are `@GuiValue`s upstream only writes from `GuiAnalog`, so this port writes them with
/// `/rftoolsutility analog-set` (the same stand-in the other logic blocks use for their GUIs).
const ANALOG: &str = "rftoolsutility:analog";
const ANALOG_TYPE: &str = "analog";
const ANALOG_TYPE_ID: &str = "rftoolsutility:analog";
/// The six `AnalogData` fields (`mul_eq`, `mul_less`, `mul_greater` are f32; the three `add_*` are
/// ints), stored under their artefact codec names so the saved compound reads like the record.
const KEY_ANALOG_MUL_EQ: &str = "mul_eq";
const KEY_ANALOG_MUL_LESS: &str = "mul_less";
const KEY_ANALOG_MUL_GREATER: &str = "mul_greater";
const KEY_ANALOG_ADD_EQ: &str = "add_eq";
const KEY_ANALOG_ADD_LESS: &str = "add_less";
const KEY_ANALOG_ADD_GREATER: &str = "add_greater";
/// The saved input side (the jar's 24 `logic_facing` slab rotations cannot be content states).
const KEY_ANALOG_FACING: &str = "analogFacing";
/// Harness-only observations saved in the compound: the game time of the last output change, and
/// the three input strengths that produced it. They let the leg assert the artefact's arithmetic
/// from the reply's own numbers, not from a wall-clock wait or an absence of errors.
const KEY_ANALOG_CHANGED_AT: &str = "analogChangedAt";
const KEY_ANALOG_INPUT: &str = "analogInput";
const KEY_ANALOG_A: &str = "analogA";
const KEY_ANALOG_B: &str = "analogB";
/// An analog's item, resolved once in `register` for `block_drops`.
static ANALOG_ITEM: OnceLock<ItemKey> = OnceLock::new();

/// One recipe the crafter was taught (`CrafterData.recipes[i]`): the engine recipe it names
/// and, per grid cell, the item ids that cell accepts. Upstream keeps the ghost grid and asks
/// the engine's recipe manager every operation; the port resolves the ingredient sets once,
/// when the harness teaches the recipe, so a tick needs no recipe-table walk.
struct Taught {
    /// `minecraft:oak_planks`, the recipe the grid matches.
    id: String,
    /// Each of the nine cells: the item ids the cell accepts (empty for a cell the pattern
    /// leaves empty).
    cells: [Vec<String>; 9],
}

/// What a `/rftoolsutility setting` writes.
#[derive(Clone, Copy)]
enum Setting {
    Speed(Speed),
    Mode(CraftMode),
    Keep(bool),
}

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

/// Drop a saved key (the unlink of a beamer's destination).
fn remove(nbt: &mut Compound, name: &str) {
    if let Tag::String(utf16) = key_tag(name) {
        nbt.remove(&utf16);
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

/// The `minecraft:block_entity_data` payload a broken block's item carries: the block entity's
/// own saved data, tagged with its type id (`TypedEntityData.loadInto` loads a payload only when
/// its `id` names the block entity it is placed as). This is upstream's
/// `collectImplicitComponents` moving a machine's contents onto its item.
fn carried_data(saved: &Compound, type_id: &str) -> Compound {
    let mut payload = saved.clone();
    put(&mut payload, "id", Tag::String(type_id.into()));
    payload
}

/// The component patch that puts `saved` on a dropped machine's item, as `type_id`'s block
/// entity.
fn entity_patch(saved: &Compound, type_id: &str) -> Compound {
    let mut patch = Compound::new();
    put(
        &mut patch,
        BLOCK_ENTITY_DATA,
        Tag::Compound(carried_data(saved, type_id)),
    );
    patch
}

/// Put the broken machine's own saved data on its drop (`collectImplicitComponents`): vanilla's
/// `minecraft:block_entity_data` is what `BlockItem.updateCustomBlockEntityTag` applies on
/// placement, so the head of the machine (a crafter's recipes, a spawner's mob) comes back.
fn carry_machine(
    level: &dyn LevelHost,
    event: &mut BlockDropsEvent<'_>,
    item: ItemKey,
    type_id: &str,
) {
    let Some(entity) = event.block_entity else {
        return;
    };
    let Some(saved) = level.block_entity_data(entity) else {
        return;
    };
    let patch = entity_patch(&saved, type_id);
    for drop in event.drops.iter_mut() {
        if drop.stack.item() == item {
            let _ = drop.stack.apply_components(&patch);
        }
    }
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
// The crafter's saved state
// ---------------------------------------------------------------------------

/// Save an int, replacing any value of another kind under the same name.
fn write_int(nbt: &mut Compound, name: &str, value: i32) {
    put(nbt, name, Tag::Int(value));
}

/// The int under `name`, or 0 when absent or of another kind.
fn read_int(nbt: &Compound, name: &str) -> i32 {
    match tag(nbt, name) {
        Some(Tag::Int(value)) => *value,
        _ => 0,
    }
}

/// Save a string, replacing any value of another kind under the same name.
fn write_text(nbt: &mut Compound, name: &str, value: &str) {
    put(nbt, name, Tag::String(value.into()));
}

/// The text under `name`, or none when absent or of another kind.
fn read_text(nbt: &Compound, name: &str) -> Option<String> {
    match tag(nbt, name) {
        Some(Tag::String(value)) => Some(value.to_string_lossy()),
        _ => None,
    }
}

/// The three float amounts under `name` (`SpawnerData.matter0..2`). NBT carries a float as its
/// bits (`Tag::Float(u32)`), so the round trip is explicit.
fn read_matter(nbt: &Compound, name: &str) -> Option<[f32; 3]> {
    let Some(Tag::List(entries)) = tag(nbt, name) else {
        return None;
    };
    let mut matter = [0.0f32; 3];
    for (slot, entry) in matter.iter_mut().zip(entries) {
        let Tag::Float(bits) = entry else {
            return None;
        };
        *slot = f32::from_bits(*bits);
    }
    Some(matter)
}

/// Save three float amounts, replacing any value of another kind under the same name.
fn write_matter(nbt: &mut Compound, name: &str, matter: [f32; 3]) {
    put(
        nbt,
        name,
        Tag::List(
            matter
                .iter()
                .map(|amount| Tag::Float(amount.to_bits()))
                .collect(),
        ),
    );
}

/// The crafter's taught recipes (`CrafterData.recipes`), in the order they are crafted.
fn read_taught(nbt: &Compound) -> Vec<Taught> {
    let Some(Tag::List(entries)) = tag(nbt, KEY_RECIPES) else {
        return Vec::new();
    };
    let mut taught = Vec::with_capacity(entries.len());
    for entry in entries {
        let Tag::Compound(entry) = entry else {
            continue;
        };
        let Some(id) = read_text(entry, "id") else {
            continue;
        };
        let mut cells: [Vec<String>; 9] = std::array::from_fn(|_| Vec::new());
        if let Some(Tag::List(list)) = tag(entry, "cells") {
            for (cell, entry) in cells.iter_mut().zip(list) {
                if let Tag::List(ids) = entry {
                    cell.extend(ids.iter().filter_map(|id| match id {
                        Tag::String(id) => Some(id.to_string_lossy()),
                        _ => None,
                    }));
                }
            }
        }
        taught.push(Taught { id, cells });
    }
    taught
}

/// Write the crafter's taught recipes back.
fn write_taught(nbt: &mut Compound, taught: &[Taught]) {
    let entries: Vec<Tag> = taught
        .iter()
        .map(|recipe| {
            let cells: Vec<Tag> = recipe
                .cells
                .iter()
                .map(|accepted| {
                    Tag::List(
                        accepted
                            .iter()
                            .map(|id| Tag::String(id.as_str().into()))
                            .collect(),
                    )
                })
                .collect();
            Tag::compound([
                ("id", Tag::String(recipe.id.as_str().into())),
                ("cells", Tag::List(cells)),
            ])
        })
        .collect();
    put(nbt, KEY_RECIPES, Tag::List(entries));
}

// ---------------------------------------------------------------------------
// The crafter: its craft loop, its saved state and its readout
// ---------------------------------------------------------------------------

/// A slot's contents as the runtime's planning sees them.
fn planned(host: &dyn Host, stack: &ItemStack) -> runtime::Stack {
    runtime::Stack {
        item: stack.item().id(host),
        count: stack.count(),
    }
}

/// The buffer slots `range`, as the runtime's planning sees them.
fn planned_range(
    host: &dyn Host,
    items: &[Option<ItemStack>],
    range: std::ops::Range<usize>,
) -> Vec<Option<runtime::Stack>> {
    range
        .map(|slot| {
            items
                .get(slot)
                .and_then(Option::as_ref)
                .map(|s| planned(host, s))
        })
        .collect()
}

/// An item's stack limit (`Item.getMaxStackSize`), 64 for an id the game does not know.
fn max_stack(host: &dyn Host, item: &str) -> u32 {
    Identifier::parse(item)
        .ok()
        .and_then(|id| host.item_key(&id))
        .and_then(|key| host.max_stack_size(key))
        .unwrap_or(64)
}

/// The level a block-entity tick runs in: the level the server ticks now (`LevelRef::in_place`).
/// A tick is told a position, not its level, and a block entity is out of the world while it
/// ticks. Resolving the level by looking for a level that holds *a block like this* at that
/// position picks the first dimension in vanilla order, so with the same machine at the same
/// coordinates in two dimensions the tick acted in the other one: the spawner paid from this
/// block and spawned nothing (a level that is not in place refuses a spawn), and the crafter
/// asked the wrong level's recipe table. The level in place *is* the tick's own dimension,
/// whatever else is loaded and whatever stands at the same coordinates there.
fn ticking_level(host: &dyn ServerHost) -> Option<LevelRef> {
    host.levels()
        .list_levels()
        .into_iter()
        .find(|level| level.in_place(host))
}

/// One crafting operation (`CrafterBaseTE.craftOneItem`): take one item per cell of the recipe's
/// grid out of the input buffer, let the engine assemble it, and put the result and the
/// recipe's remainders where the craft mode says. Answers whether the operation happened.
fn craft_one(
    host: &mut dyn ServerHost,
    level: LevelRef,
    items: &mut [Option<ItemStack>],
    taught: &Taught,
    mode: CraftMode,
    keep: bool,
) -> bool {
    let inputs = BUFFER_IN..BUFFER_IN + BUFFER_IN_SLOTS;
    let buffer = planned_range(&*host, items, inputs.clone());
    let Some(takes) = plan(&taught.cells, &buffer, keep) else {
        return false;
    };
    // The work grid holds the item each planned cell takes; the engine matches it against its
    // own recipe table, which is what makes the recipe's tags and remainders work.
    let mut grid: Vec<Option<ItemStack>> = vec![None; 9];
    for take in &takes {
        // `plan` numbers the slots of the buffer slice it was given, not the world.
        let slot = inputs.start + take.slot;
        let Some(stack) = items.get(slot).and_then(Option::as_ref) else {
            return false;
        };
        let mut one = stack.clone();
        if one.set_count(1).is_err() {
            return false;
        }
        grid[take.cell] = Some(one);
    }
    let Ok(kind) = Identifier::parse(RECIPE_TYPE) else {
        return false;
    };
    let input = RecipeInput::Crafting {
        width: 3,
        height: 3,
        slots: grid,
    };
    let Ok(Some(matched)) = level.recipe_for(&*host, &kind, &input) else {
        return false;
    };
    if matched.recipe.to_string() != taught.id {
        return false;
    }
    // What the operation produces, in order: the result, then the recipe's remainders.
    let produced: Vec<ItemStack> = std::iter::once(matched.output.clone())
        .chain(matched.remainders.iter().flatten().cloned())
        .filter(|stack| stack.count() > 0)
        .collect();
    if produced.is_empty() {
        return false;
    }
    let stacks: Vec<runtime::Stack> = produced.iter().map(|s| planned(&*host, s)).collect();
    let out = result_range(mode);
    let room = planned_range(&*host, items, out.clone());
    let limit = |item: &str| max_stack(&*host, item);
    let Some(targets) = placements(&stacks, &room, &limit) else {
        return false;
    };
    // `placements` numbers the slots of `room`; the write is to the buffer's own slots.
    let targets: Vec<usize> = targets.into_iter().map(|slot| out.start + slot).collect();
    // Everything is known to fit: consume, then write.
    let mut after = planned_range(&*host, items, inputs.clone());
    if !runtime::consume(&takes, &mut after, keep) {
        return false;
    }
    for (slot, planned) in inputs.zip(after) {
        match planned {
            None => items[slot] = None,
            Some(planned) => {
                let Some(stack) = items.get_mut(slot).and_then(Option::as_mut) else {
                    return false;
                };
                if stack.set_count(planned.count).is_err() {
                    return false;
                }
            }
        }
    }
    for (stack, slot) in produced.into_iter().zip(targets) {
        match items.get_mut(slot).and_then(Option::as_mut) {
            None => items[slot] = Some(stack),
            Some(held) => {
                if !held.is_stackable_with(&stack) {
                    return false;
                }
                if held.grow(stack.count()).is_err() {
                    return false;
                }
            }
        }
    }
    true
}

/// One server tick of the crafter (`CrafterBaseTE.tickServer`): while the energy pays for an
/// operation, run the taught recipes in order — a result `Int` mode puts in the input buffer is
/// what the next recipe of the same cycle can use (upstream's chained crafting).
fn crafter_tick(host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
    let mut changed = false;
    if event.entity.items.len() < CRAFTER_SLOTS {
        event.entity.items.resize(CRAFTER_SLOTS, None);
        changed = true;
    }
    let taught = read_taught(&event.entity.nbt);
    let energy = read_int(&event.entity.nbt, KEY_ENERGY).clamp(0, CRAFTER_CAPACITY);
    let speed = read_text(&event.entity.nbt, KEY_SPEED)
        .as_deref()
        .and_then(Speed::parse)
        .unwrap_or_default();
    let mode = read_text(&event.entity.nbt, KEY_MODE)
        .as_deref()
        .and_then(CraftMode::parse)
        .unwrap_or_default();
    let keep = read_text(&event.entity.nbt, KEY_KEEP).as_deref() == Some("Keep");
    let mut operations = steps(speed, energy);
    if operations == 0 || taught.is_empty() {
        if !changed {
            *event.reply = BlockEntityTickReply::Unchanged;
        }
        return;
    }
    let Some(level) = ticking_level(&*host) else {
        if !changed {
            *event.reply = BlockEntityTickReply::Unchanged;
        }
        return;
    };
    let mut done = 0;
    'cycles: for _ in 0..operations {
        let mut crafted = false;
        for recipe in &taught {
            if craft_one(host, level, &mut event.entity.items, recipe, mode, keep) {
                crafted = true;
            }
        }
        if !crafted {
            break 'cycles;
        }
        done += 1;
    }
    operations = done;
    if operations > 0 {
        write_int(
            &mut event.entity.nbt,
            KEY_ENERGY,
            energy.saturating_sub(spend(operations)),
        );
        changed = true;
    }
    if !changed {
        *event.reply = BlockEntityTickReply::Unchanged;
    }
}

/// What `/rftoolsutility crafter` prints. The shape is the harness's contract
/// (`tools/parse_crafter.py`), so the end-to-end scenario can parse it.
fn crafter_readout(
    host: &dyn Host,
    pos: [i32; 3],
    items: &[Option<ItemStack>],
    nbt: &Compound,
    nonce: Option<&str>,
) -> String {
    let energy = read_int(nbt, KEY_ENERGY).clamp(0, CRAFTER_CAPACITY);
    let speed = read_text(nbt, KEY_SPEED).unwrap_or_else(|| Speed::default().name().to_owned());
    let mode = read_text(nbt, KEY_MODE).unwrap_or_else(|| CraftMode::default().name().to_owned());
    let keep = read_text(nbt, KEY_KEEP).unwrap_or_else(|| "All".to_owned());
    let taught = read_taught(nbt);
    let recipes = if taught.is_empty() {
        "-".to_owned()
    } else {
        taught
            .iter()
            .map(|recipe| recipe.id.as_str())
            .collect::<Vec<_>>()
            .join("+")
    };
    let slots = |range: std::ops::Range<usize>| {
        let listed: Vec<String> = range
            .filter_map(|slot| {
                let stack = items.get(slot)?.as_ref()?;
                Some(format!(
                    "{slot}={}x{}",
                    stack.item().id(host),
                    stack.count()
                ))
            })
            .collect();
        if listed.is_empty() {
            "-".to_owned()
        } else {
            listed.join(";")
        }
    };
    let suffix = nonce.map_or(String::new(), |nonce| format!(" nonce={nonce}"));
    format!(
        "Crafter at {pos:?}: energy={energy}/{CRAFTER_CAPACITY} speed={speed} mode={mode} \
         keep={keep} recipes={recipes} in={} out={}{suffix}",
        slots(BUFFER_IN..BUFFER_IN + BUFFER_IN_SLOTS),
        slots(BUFFER_OUT..BUFFER_OUT + BUFFER_OUT_SLOTS)
    )
}

/// The crafter's container: 41 slots, the ten ghost slots and the filter module slot outside
/// the automation's reach, the 26 input slots hoppers may fill and the four output slots they
/// may take from (`CrafterContainer.CONTAINER_FACTORY` and its `in()`/`out()` slots).
fn crafter_container() -> ModContainer {
    let inputs: Vec<usize> = (BUFFER_IN..BUFFER_IN + BUFFER_IN_SLOTS).collect();
    let outputs: Vec<usize> = (BUFFER_OUT..BUFFER_OUT + BUFFER_OUT_SLOTS).collect();
    let reachable: Vec<usize> = inputs.iter().chain(&outputs).copied().collect();
    ModContainer {
        slots: CRAFTER_SLOTS,
        faces: std::array::from_fn(|_| reachable.clone()),
        inputs,
        outputs,
        menu: None,
    }
}

/// The crafter's energy domain (`GenericEnergyStorage(this, true, 50,000, 500)`): a fill only,
/// on every face. The amount is the `energy` int in the saved compound.
fn crafter_energy() -> ModEnergyBuffer {
    ModEnergyBuffer {
        capacity: CRAFTER_CAPACITY,
        max_insert: CRAFTER_RECEIVE,
        max_extract: 0,
        insert_faces: [true; 6],
        extract_faces: [false; 6],
    }
}

/// The crafter's Part: its block entity, its tick, and (as `ItemPart`) nothing — a crafter's
/// item carries its saved data as vanilla's `minecraft:block_entity_data` and needs no clamp.
struct CrafterPart;

#[mod_api::horizon_part]
impl BlockPart for CrafterPart {}

#[mod_api::horizon_part]
impl BlockEntityPart for CrafterPart {
    /// The crafter ticks on the server (`EntityBlock.getTicker`).
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick of the craft loop.
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        crafter_tick(host, event);
    }
}

// ---------------------------------------------------------------------------
// The spawner: its recipes, its saved state, its tick and its readout
// ---------------------------------------------------------------------------

/// One `rftoolsutility:spawner` recipe as it crosses the API boundary: plain data, so its
/// encoding is a `Wire` value and not a game type.
#[derive(Clone, Debug, PartialEq, mod_api::Wire)]
struct SpawnerValue {
    /// The mob it spawns (`SpawnerRecipe.entity`).
    entity: String,
    /// The recipe's `power`: FE one spawn costs.
    power: i32,
    /// The recipe's `item1..3` amounts.
    matter: [f32; 3],
    /// The recipe's `item1..3.object` sources, which a matter beamer matches a material
    /// against (`SpawnerRecipes.MobSpawnAmount.match`).
    objects: [Vec<WireSource>; 3],
}

impl SpawnerValue {
    /// The machine's own copy of the recipe's rules.
    fn recipe(&self) -> MobRecipe {
        MobRecipe {
            entity: self.entity.clone(),
            power: self.power,
            matter: self.matter,
            objects: std::array::from_fn(|index| {
                self.objects[index].iter().map(WireSource::source).collect()
            }),
        }
    }
}

/// Reads one `rftoolsutility:spawner` recipe file (`SpawnerRecipeSerializer`): the entity it
/// spawns, its `power`, and the three `item1..3` amounts. The engine hands this the object with
/// its `type` removed, so an absent type is accepted and a wrong one refused.
///
/// Only the `amount` of each `item1..3` is read. Their `object` is upstream's matter
/// *ingredient* — an item, a tag, or a list of either — which this port never parses as an
/// ingredient: `machine_recipe_ingredient`'s 1.21.1 forms are the only shape upstream's own
/// codec knows, and turning them into 26.3 ingredients would change what a recipe means.
fn spawner_recipe(json: &mod_api::serde_json::Value) -> Result<SpawnerValue, String> {
    use mod_api::serde_json::Value;
    if let Some(kind) = json.get("type").and_then(Value::as_str)
        && kind != SPAWNER_RECIPE_TYPE
    {
        return Err(format!("{kind} is not a {SPAWNER_RECIPE_TYPE} recipe"));
    }
    let entity = json
        .get("entity")
        .and_then(Value::as_str)
        .filter(|entity| !entity.is_empty())
        .ok_or("a spawner recipe needs an entity")?;
    let power = json
        .get("power")
        .and_then(|power| {
            power
                .as_i64()
                .or_else(|| power.as_f64().map(|power| power as i64))
        })
        .ok_or("a spawner recipe needs a power")?;
    let power =
        i32::try_from(power).map_err(|_| format!("a spawner power of {power} does not fit"))?;
    // A recipe's amount is a float (`SpawnerData` keeps them as `Codec.FLOAT`), so the
    // machine's copy is f32; an amount a pack leaves out is zero, as upstream's codec requires
    // the field and this one is lenient.
    let amount = |index: usize| {
        json.get(format!("item{}", index + 1))
            .and_then(|item| item.get("amount"))
            .and_then(Value::as_f64)
            .map_or(0.0, |amount| amount as f32)
    };
    Ok(SpawnerValue {
        entity: entity.to_owned(),
        power,
        matter: [amount(0), amount(1), amount(2)],
        objects: objects_of(json),
    })
}

/// Writes the value back as the recipe file it came from, for the client's copy of the table.
fn write_spawner(value: &SpawnerValue) -> String {
    mod_api::serde_json::json!({
        "type": SPAWNER_RECIPE_TYPE,
        "entity": value.entity.as_str(),
        "power": value.power,
        "item1": { "amount": value.matter[0], "object": object_json(&value.objects[0]) },
        "item2": { "amount": value.matter[1], "object": object_json(&value.objects[1]) },
        "item3": { "amount": value.matter[2], "object": object_json(&value.objects[2]) },
    })
    .to_string()
}

/// The registered `rftoolsutility:spawner` kind: a data table, not a grid recipe. Upstream's
/// `SpawnerRecipe` answers false to `matches` and `ItemStack.EMPTY` to `assemble`, and the
/// spawner block is what reads the values.
struct SpawnerRecipeKind;

impl CraftingRecipeKind for SpawnerRecipeKind {
    type Value = SpawnerValue;

    fn codec(&self) -> Option<CraftingRecipeCodec<Self::Value>> {
        Some(CraftingRecipeCodec::wire())
    }

    fn decode(&self, json: &str) -> Result<Self::Value, String> {
        let value: mod_api::serde_json::Value =
            mod_api::serde_json::from_str(json).map_err(|error| error.to_string())?;
        spawner_recipe(&value)
    }

    fn write(&self, value: &Self::Value) -> Result<String, String> {
        Ok(write_spawner(value))
    }

    /// A spawner recipe is not craftable in a grid (`SpawnerRecipe.canCraftInDimensions` is
    /// false and `matches` always refuses).
    fn matches(&self, _: &Self::Value, _: CraftingGrid) -> bool {
        false
    }

    /// …and makes nothing (`SpawnerRecipe.assemble` returns `ItemStack.EMPTY`).
    fn assemble(&self, _: &Self::Value, _: CraftingGrid) -> Option<ItemStack> {
        None
    }
}

/// The mob recipes in force (`SpawnerRecipes.loadRecipes`), in table order.
///
/// The files' `type` names this port's kind, which the pin files under `minecraft:crafting`, so
/// they are read from that table; the type's own table is read as well because the API says a
/// file's type field "can differ from the reported recipe kind". A recipe is counted once, by
/// id.
fn spawner_recipes(host: &dyn Host) -> Vec<MobRecipe> {
    let mut found: Vec<(String, MobRecipe)> = Vec::new();
    for kind in [RECIPE_TYPE, SPAWNER_RECIPE_TYPE] {
        for recipe in host.recipes_of_type(kind).iter() {
            if recipe.json.get("type").and_then(|kind| kind.as_str()) != Some(SPAWNER_RECIPE_TYPE) {
                continue;
            }
            if found.iter().any(|(id, _)| *id == recipe.id) {
                continue;
            }
            let Ok(value) = spawner_recipe(&recipe.json) else {
                continue;
            };
            found.push((recipe.id.clone(), value.recipe()));
        }
    }
    found.into_iter().map(|(_, recipe)| recipe).collect()
}

/// The side a spawn appears on: the block's own `facing` state.
fn facing_of(host: &dyn Host, level: LevelRef, pos: [i32; 3]) -> Option<String> {
    let state = level.block_state(host, pos)?;
    let states = host.block_states();
    state
        .property(&states, KEY_FACING)
        .ok()
        .flatten()
        .map(str::to_owned)
}

/// One server tick of the spawner (`SpawnerTileEntity.tickServer`): with a mob set, its recipe's
/// `power` in the buffer **and every one of its three matter amounts in store**, pay both and
/// spawn one mob on the block's facing side.
///
/// Upstream returns early until all three matter amounts are stored (`for (i…)
/// if (data.getMatter(i) < mobData.getItem(i).getAmount()) return;`) and clears them once the
/// mob exists, so the matter is what gates the spawn; the port's stores are filled by
/// `/rftoolsutility matter` (a matter beamer's send, whose block is a later slice).
fn spawner_tick(host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
    let pos = event.pos;
    let Some(mob) = read_text(&event.entity.nbt, KEY_MOB) else {
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    };
    let cost = read_int(&event.entity.nbt, KEY_COST);
    let energy = read_int(&event.entity.nbt, KEY_ENERGY).clamp(0, SPAWNER_CAPACITY);
    let needs = read_matter(&event.entity.nbt, KEY_NEEDS).unwrap_or([0.0; 3]);
    let matter = read_matter(&event.entity.nbt, KEY_MATTER).unwrap_or([0.0; 3]);
    let Some((left, left_matter)) = spawn_cycle(energy, cost, needs, matter) else {
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    };
    let Some(level) = ticking_level(&*host) else {
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    };
    let Some(facing) = facing_of(&*host, level, pos) else {
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    };
    let Some(at) = spawn_at(pos, &facing) else {
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    };
    // `EntityType.spawn` with the `SPAWNER` reason: the type makes the mob, finalizes it and
    // adds it to the level. A refusal spawns nothing, and neither the power nor the matter is
    // paid (the slice-4 defect fix: upstream pays before it knows the mob exists).
    if level
        .spawn_mob(host, &mob, at, SpawnReason::Spawner)
        .is_none()
    {
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    }
    write_int(&mut event.entity.nbt, KEY_ENERGY, left);
    write_matter(&mut event.entity.nbt, KEY_MATTER, left_matter);
    let spawned = read_int(&event.entity.nbt, KEY_SPAWNED).saturating_add(1);
    write_int(&mut event.entity.nbt, KEY_SPAWNED, spawned);
}

/// Three float amounts as the readout prints them: `a,b,c`, or `-` for an amount the machine
/// does not hold at all.
fn triple(amounts: Option<[f32; 3]>) -> String {
    amounts.map_or_else(
        || "-".to_owned(),
        |amounts| format!("{},{},{}", amounts[0], amounts[1], amounts[2]),
    )
}

/// What `/rftoolsutility spawner` prints. The shape is the harness's contract
/// (`tools/parse_spawner.py`), so the end-to-end scenario can parse it.
fn spawner_readout(pos: [i32; 3], nbt: &Compound, nonce: Option<&str>) -> String {
    let energy = read_int(nbt, KEY_ENERGY).clamp(0, SPAWNER_CAPACITY);
    let mob = read_text(nbt, KEY_MOB).unwrap_or_else(|| "none".to_owned());
    let cost = read_int(nbt, KEY_COST);
    let cost = if cost > 0 {
        cost.to_string()
    } else {
        "-".to_owned()
    };
    let needs = triple(read_matter(nbt, KEY_NEEDS));
    // A store always exists, empty, so it prints its own zeros rather than a dash.
    let matter = triple(Some(read_matter(nbt, KEY_MATTER).unwrap_or([0.0; 3])));
    let spawned = read_int(nbt, KEY_SPAWNED).max(0);
    let suffix = nonce.map_or(String::new(), |nonce| format!(" nonce={nonce}"));
    format!(
        "Spawner at {pos:?}: energy={energy}/{SPAWNER_CAPACITY} mob={mob} cost={cost} \
         needs={needs} matter={matter} spawned={spawned}{suffix}"
    )
}

/// The spawner's energy domain (`GenericEnergyStorage(this, true, 200,000, 2,000)`): a fill
/// only, on every face. The amount is the `energy` int in the saved compound.
fn spawner_energy() -> ModEnergyBuffer {
    ModEnergyBuffer {
        capacity: SPAWNER_CAPACITY,
        max_insert: SPAWNER_RECEIVE,
        max_extract: 0,
        insert_faces: [true; 6],
        extract_faces: [false; 6],
    }
}

/// The spawner's Part: its block entity and its spawn tick. No container and no menu: the mob
/// setting is the syringe slot upstream fills, and this slice has no syringe, so the harness
/// writes the setting instead.
struct SpawnerPart;

#[mod_api::horizon_part]
impl BlockPart for SpawnerPart {}

#[mod_api::horizon_part]
impl BlockEntityPart for SpawnerPart {
    /// The spawner ticks on the server (`EntityBlock.getTicker`).
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick: spawn one mob if the buffer pays for it.
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        spawner_tick(host, event);
    }
}

// ---------------------------------------------------------------------------
// The matter beamer: the material match, the destination, the tick and the readout
// ---------------------------------------------------------------------------

/// One `item1..3.object` source as it crosses the API boundary: the recipe's own spelling, a
/// `Wire` value so the recipe can ride the kind's codec, converted to [`MatterSource`] for the
/// match.
#[derive(Clone, Debug, PartialEq, Eq, mod_api::Wire)]
enum WireSource {
    /// `{"item": id}`.
    Material(String),
    /// `{"tag": id}`.
    Tagged(String),
}

impl WireSource {
    /// The runtime's view of this source.
    fn source(&self) -> MatterSource {
        match self {
            WireSource::Material(item) => MatterSource::Material(item.clone()),
            WireSource::Tagged(tag) => MatterSource::Tagged(tag.clone()),
        }
    }
}

/// `item1..3.object` as the beamer reads it (`SpawnerRecipes.MobSpawnAmount.match`): each type
/// is a list of the item and tag ids it accepts, and an empty list is upstream's "living?" form.
fn objects_of(json: &mod_api::serde_json::Value) -> [Vec<WireSource>; 3] {
    std::array::from_fn(|index| {
        let Some(object) = json
            .get(format!("item{}", index + 1))
            .and_then(|item| item.get("object"))
        else {
            return Vec::new();
        };
        let entries: Vec<&mod_api::serde_json::Value> = match object {
            mod_api::serde_json::Value::Array(entries) => entries.iter().collect(),
            entry => vec![entry],
        };
        entries
            .iter()
            .filter_map(|entry| source_of(entry))
            .collect()
    })
}

/// One `{"item": id}` or `{"tag": id}` entry of an `object`, if it is one; a form this port does
/// not know is left out (it matches nothing).
fn source_of(entry: &mod_api::serde_json::Value) -> Option<WireSource> {
    if let Some(item) = entry.get("item").and_then(|item| item.as_str()) {
        return Some(WireSource::Material(item.to_owned()));
    }
    entry
        .get("tag")
        .and_then(|tag| tag.as_str())
        .map(|tag| WireSource::Tagged(tag.trim_start_matches('#').to_owned()))
}

/// One source in the recipe's own 1.21.1 spelling (`{"item": id}` / `{"tag": id}`).
fn source_json(source: &WireSource) -> mod_api::serde_json::Value {
    match source {
        WireSource::Material(item) => mod_api::serde_json::json!({ "item": item }),
        WireSource::Tagged(tag) => mod_api::serde_json::json!({ "tag": tag }),
    }
}

/// One matter type's `object` as the recipe file writes it: a single object, a list of them, or
/// `[]` for the living form (which is how a `write_spawner` round trip keeps the file's shape).
fn object_json(sources: &[WireSource]) -> mod_api::serde_json::Value {
    let mut entries: Vec<mod_api::serde_json::Value> = sources.iter().map(source_json).collect();
    if entries.len() == 1 {
        entries.pop().unwrap_or(mod_api::serde_json::Value::Null)
    } else {
        mod_api::serde_json::Value::Array(entries)
    }
}

/// A saved block position under `name`: the three ints a link writes, so a broken beamer keeps
/// the spawner its item points at.
fn write_pos(nbt: &mut Compound, name: &str, pos: [i32; 3]) {
    put(
        nbt,
        name,
        Tag::List(pos.iter().map(|value| Tag::Int(*value)).collect()),
    );
}

/// The destination saved under `name`, or none for an unlinked beamer or data that is not three
/// ints.
fn read_pos(nbt: &Compound, name: &str) -> Option<[i32; 3]> {
    let Tag::List(values) = tag(nbt, name)? else {
        return None;
    };
    let mut ints = values.iter().filter_map(|value| match value {
        Tag::Int(value) => Some(*value),
        _ => None,
    });
    let pos = [ints.next()?, ints.next()?, ints.next()?];
    ints.next().is_none().then_some(pos)
}

/// The block's `lit` state (`BlockStateProperties.LIT`), which a send turns on and an empty
/// slot, a missing signal or a lost destination turns off.
fn lit_of(host: &dyn Host, level: LevelRef, pos: [i32; 3]) -> bool {
    let Some(state) = level.block_state(host, pos) else {
        return false;
    };
    let states = host.block_states();
    state.property(&states, KEY_LIT).ok().flatten() == Some("true")
}

/// Write the `lit` state (`enableBlockGlow`/`disableBlockGlow`); `set_block_property` writes only
/// when it differs, so a tick that changes nothing touches no chunk.
fn glow(host: &mut dyn ServerHost, pos: [i32; 3], lit: bool) {
    host.set_block_property(pos, KEY_LIT, if lit { "true" } else { "false" });
}

/// Whether the item id `item` is in the item tag `tag` (`TagTools.hasTag`); an unknown tag or
/// item answers false.
fn item_in_tag(host: &dyn Host, item: &str, tag: &str) -> bool {
    let (Ok(item), Ok(tag)) = (Identifier::parse(item), Identifier::parse(tag)) else {
        return false;
    };
    let (Some(item), Some(tag)) = (host.item_key(&item), host.tags().key(&tag)) else {
        return false;
    };
    host.tags().is_in_tag(item, tag).unwrap_or(false)
}

/// The yield a living item carries (`MobSpawnAmount.match`'s empty-object branch), from the
/// jar's own `living/*` tags.
fn living_yield(host: &dyn Host, item: &str) -> f32 {
    if item_in_tag(host, item, HIGH_YIELD_TAG) {
        HIGH_YIELD
    } else if item_in_tag(host, item, AVERAGE_YIELD_TAG) {
        AVERAGE_YIELD
    } else if item_in_tag(host, item, LOW_YIELD_TAG) {
        LOW_YIELD
    } else {
        0.0
    }
}

/// What `/rftoolsutility beamer` prints. The shape is the harness's contract
/// (`tools/parse_beamer.py`), so the end-to-end scenario can parse it.
fn beamer_readout(
    pos: [i32; 3],
    nbt: &Compound,
    slot: Option<(&str, i32)>,
    lit: bool,
    nonce: Option<&str>,
) -> String {
    let energy = read_int(nbt, KEY_ENERGY).clamp(0, BEAMER_CAPACITY);
    let dest =
        read_pos(nbt, KEY_DEST).map_or_else(|| "none".to_owned(), |dest| format!("{dest:?}"));
    let slot = slot.map_or_else(|| "-".to_owned(), |(item, count)| format!("{item}x{count}"));
    let sent = read_int(nbt, KEY_SENT).max(0);
    let suffix = nonce.map_or(String::new(), |nonce| format!(" nonce={nonce}"));
    format!(
        "Beamer at {pos:?}: energy={energy}/{BEAMER_CAPACITY} dest={dest} slot={slot} \
         sent={sent} lit={lit}{suffix}"
    )
}

/// `/rftoolsutility link <pos> <dest>`: what the wrench does upstream — a wrench on the beamer
/// then on the spawner links them (`SpawnerTileEntity.useWrench`), a second wrench on the beamer
/// clears it, a destination farther than `maxBeamDistance` is refused, and so is a block that is
/// not a spawner. The harness's route, because this port carries no wrench item (the parent
/// port's).
fn link_beamer(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
    dest: [i32; 3],
) -> String {
    let Some(entity) = machine_at(&*host, level, pos, BEAMER) else {
        return "No matter beamer there.".to_owned();
    };
    let Some(mut data) = entity.data(&*host) else {
        return "No matter beamer there.".to_owned();
    };
    if dest == pos {
        remove(&mut data, KEY_DEST);
        if entity.replace_data(host, data).is_err() {
            return "No matter beamer there.".to_owned();
        }
        glow(host, pos, false);
        return format!("/rftoolsutility: the matter beamer at {pos:?} is unlinked.");
    }
    if !beam_reach(pos, dest) {
        return format!(
            "/rftoolsutility: destination distance is too far! (maxBeamDistance is {} blocks)",
            MAX_BEAM_DISTANCE as i32
        );
    }
    if machine_at(&*host, level, dest, SPAWNER).is_none() {
        return format!("/rftoolsutility: the destination {dest:?} is not a spawner.");
    }
    write_pos(&mut data, KEY_DEST, dest);
    if entity.replace_data(host, data).is_err() {
        return "No matter beamer there.".to_owned();
    }
    glow(host, pos, false);
    format!("/rftoolsutility: the matter beamer at {pos:?} sends to the spawner at {dest:?}.")
}

/// One server tick of the beamer (`MatterBeamerTileEntity.tickServer`): with a redstone signal
/// and a send tick, move the material its linked spawner's recipe takes into the store that
/// material feeds, and pay the FE — one block at a time, and only when the send happened.
fn beamer_tick(host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
    let pos = event.pos;
    if event.entity.items.get(BEAMER_SLOT).is_none() {
        event.entity.items.resize(BEAMER_SLOT + 1, None);
    }
    let Some(level) = ticking_level(&*host) else {
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    };
    // Every tick, before the send clock: a beamer without a signal is dark.
    if !level.has_neighbor_signal(&*host, pos) {
        glow(host, pos, false);
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    }
    if !level.game_time(&*host).is_some_and(beam_due) {
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    }
    let Some(dest) = read_pos(&event.entity.nbt, KEY_DEST) else {
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    };
    let Some(spawner) = machine_at(&*host, level, dest, SPAWNER) else {
        // The destination is gone or is not a spawner: upstream's `setDestination(null)`.
        remove(&mut event.entity.nbt, KEY_DEST);
        glow(host, pos, false);
        return;
    };
    let Some(spawner_data) = spawner.data(&*host) else {
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    };
    let Some(item) = event.entity.items.get(BEAMER_SLOT).and_then(Option::as_ref) else {
        glow(host, pos, false);
        return;
    };
    let item_id = item.item().id(&*host);
    let count = i32::try_from(item.count()).unwrap_or(i32::MAX);
    let energy = read_int(&event.entity.nbt, KEY_ENERGY).clamp(0, BEAMER_CAPACITY);
    // The mob's recipe is the table's own data (`SpawnerRecipes.getMobData`), read when the
    // send is due rather than cached, so a reloaded recipe table is seen.
    let recipe = read_text(&spawner_data, KEY_MOB)
        .and_then(|mob| recipe_for(&spawner_recipes(&*host), &mob).cloned());
    let send = recipe.and_then(|recipe| {
        beam_send(
            &recipe.objects,
            &item_id,
            count,
            energy,
            |item, tag| item_in_tag(&*host, item, tag),
            |item| living_yield(&*host, item),
        )
    });
    let Some(send) = send else {
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    };
    // The store goes on the spawner first: only an answered send pays (upstream consumes the
    // FE before `addMatter` and burns it when the material is refused).
    let mut stores = read_matter(&spawner_data, KEY_MATTER).unwrap_or([0.0; 3]);
    stores[send.index] = add_matter(stores[send.index], send.matter);
    let mut next = spawner_data.clone();
    write_matter(&mut next, KEY_MATTER, stores);
    if spawner.replace_data(host, next).is_err() {
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    }
    write_int(&mut event.entity.nbt, KEY_ENERGY, energy - send.rf);
    let left = count - send.blocks;
    if left <= 0 {
        event.entity.items[BEAMER_SLOT] = None;
    } else if let Some(stack) = event.entity.items[BEAMER_SLOT].as_mut() {
        let _ = stack.set_count(left as u32);
    }
    let sent = read_int(&event.entity.nbt, KEY_SENT).saturating_add(1);
    write_int(&mut event.entity.nbt, KEY_SENT, sent);
    glow(host, pos, true);
}

/// The beamer's energy domain (`GenericEnergyStorage(this, true, 200,000, 1,000)`): a fill only,
/// on every face.
fn beamer_energy() -> ModEnergyBuffer {
    ModEnergyBuffer {
        capacity: BEAMER_CAPACITY,
        max_insert: BEAMER_RECEIVE,
        max_extract: 0,
        insert_faces: [true; 6],
        extract_faces: [false; 6],
    }
}

/// The beamer's one material slot (`CONTAINER_FACTORY`'s `generic().in()`): automation inserts
/// into it and takes nothing out.
fn beamer_container() -> ModContainer {
    ModContainer {
        slots: BEAMER_SLOT + 1,
        faces: std::array::from_fn(|_| vec![BEAMER_SLOT]),
        inputs: vec![BEAMER_SLOT],
        outputs: Vec::new(),
        menu: None,
    }
}

/// The beamer's Part: its block entity and its send tick.
struct BeamerPart;

#[mod_api::horizon_part]
impl BlockPart for BeamerPart {}

#[mod_api::horizon_part]
impl BlockEntityPart for BeamerPart {
    /// The beamer ticks on the server (`EntityBlock.getTicker`).
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick: at most one send, every 20 ticks.
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        beamer_tick(host, event);
    }
}

// ---------------------------------------------------------------------------
// The teleporter family (slice 9)
// ---------------------------------------------------------------------------

/// A matter transmitter at `pos`, when the block there is ours.
fn transmitter_at(host: &dyn Host, level: LevelRef, pos: [i32; 3]) -> Option<BlockEntityRef> {
    machine_at(host, level, pos, TRANSMITTER)
}

/// Clear an in-progress teleport and set the cooldown (`MatterTransmitterTileEntity.clearTeleport`).
fn clear_teleport(nbt: &mut Compound, cooldown: i32) {
    remove(nbt, KEY_TP_PLAYER);
    write_int(nbt, KEY_TP_TIMER, 0);
    write_int(nbt, KEY_COOLDOWN, cooldown.max(0));
    write_int(nbt, KEY_GOOD, 0);
    write_int(nbt, KEY_BAD, 0);
    write_int(nbt, KEY_TOTAL, 0);
}

/// Whether a matter booster stands next to `pos` (`DialingDeviceTileEntity.isMatterBoosterAvailable`).
fn booster_adjacent(host: &dyn Host, level: LevelRef, pos: [i32; 3]) -> bool {
    const AROUND: [[i32; 3]; 6] = [
        [1, 0, 0],
        [-1, 0, 0],
        [0, 1, 0],
        [0, -1, 0],
        [0, 0, 1],
        [0, 0, -1],
    ];
    AROUND.iter().any(|off| {
        let at = [pos[0] + off[0], pos[1] + off[1], pos[2] + off[2]];
        level.block(host, at).is_some_and(|b| b.id(host) == BOOSTER)
    })
}

/// The level whose dimension id is `dim` (the destination's level may not be the tick's own).
fn level_of(host: &dyn ServerHost, dim: &str) -> Option<LevelRef> {
    host.levels()
        .list_levels()
        .into_iter()
        .find(|level| level.dimension().id(&*host).as_deref() == Some(dim))
}

/// A receiver is ready to receive at `pos`: two clear blocks above and enough stored power
/// (`MatterReceiverTileEntity.checkStatus`, via the pure [`runtime::receiver_ready`]).
fn receiver_status(host: &dyn Host, level: LevelRef, pos: [i32; 3]) -> bool {
    let clear = [1, 2].iter().all(|up| {
        level
            .block(host, [pos[0], pos[1] + up, pos[2]])
            .is_some_and(|b| {
                matches!(
                    b.id(host).as_str(),
                    "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
                )
            })
    });
    let energy = host
        .block_entity(level, pos)
        .and_then(|entity| entity.data(host))
        .map_or(0, |data| read_int(&data, KEY_ENERGY));
    receiver_ready(clear, energy)
}

/// Whether a destination analyzer stands next to `pos`
/// (`DialingDeviceTileEntity.isDestinationAnalyzerAvailable`: a six-direction neighbour scan).
fn analyzer_adjacent(host: &dyn Host, level: LevelRef, pos: [i32; 3]) -> bool {
    const AROUND: [[i32; 3]; 6] = [
        [1, 0, 0],
        [-1, 0, 0],
        [0, 1, 0],
        [0, -1, 0],
        [0, 0, 1],
        [0, 0, -1],
    ];
    AROUND.iter().any(|off| {
        let at = [pos[0] + off[0], pos[1] + off[1], pos[2] + off[2]];
        level
            .block(host, at)
            .is_some_and(|b| b.id(host) == ANALYZER)
    })
}

/// The dialed receiver's status (`MatterTransmitterTileEntity.checkReceiverStatus`): the
/// destination's level is resolved by dimension id, its block entity is read there (the cross-level
/// lookup slices 6 and 9 established), and the readiness rule is the receiver's own two clear
/// blocks above plus [`RECEIVER_MIN_RF`](runtime::RECEIVER_MIN_RF). An unloaded destination level
/// is the yellow `unknown` state; a missing receiver or a failed readiness check is `warn`.
fn check_receiver_status(
    host: &dyn ServerHost,
    dim: &str,
    dest: [i32; 3],
) -> runtime::AnalyzerStatus {
    let Some(dest_level) = level_of(host, dim) else {
        return analyzer_status(false, None);
    };
    let Some(data) = host
        .block_entity(dest_level, dest)
        .and_then(|entity| entity.data(host))
    else {
        return analyzer_status(true, None);
    };
    let energy = read_int(&data, KEY_ENERGY);
    let clear = [1, 2].iter().all(|up| {
        dest_level
            .block(host, [dest[0], dest[1] + up, dest[2]])
            .is_some_and(|b| {
                matches!(
                    b.id(host).as_str(),
                    "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
                )
            })
    });
    analyzer_status(true, Some(receiver_ready(clear, energy)))
}

/// The receiver's readout: its position, dimension, readiness and buffer.
fn receiver_readout(pos: [i32; 3], data: &Compound, ready: bool, nonce: Option<&str>) -> String {
    let energy = read_int(data, KEY_ENERGY).clamp(0, RECEIVER_CAPACITY);
    let suffix = nonce.map_or(String::new(), |nonce| format!(" nonce={nonce}"));
    format!(
        "/rftoolsutility: receiver at {pos:?}: energy={energy}/{RECEIVER_CAPACITY} ready={ready}{suffix}"
    )
}

/// The transmitter's readout: its dial, buffer and teleport state.
fn transmitter_readout(pos: [i32; 3], data: &Compound, nonce: Option<&str>) -> String {
    let energy = read_int(data, KEY_ENERGY).clamp(0, TRANSMITTER_CAPACITY);
    let dial = match (read_text(data, KEY_DIAL_DIM), read_pos(data, KEY_DIAL_POS)) {
        (Some(dim), Some(pos)) => format!("{dim}@{pos:?}"),
        _ => "none".to_owned(),
    };
    let state = if read_text(data, KEY_TP_PLAYER).is_some() {
        format!("teleporting timer={}", read_int(data, KEY_TP_TIMER).max(0))
    } else if read_int(data, KEY_COOLDOWN) > 0 {
        "cooldown".to_owned()
    } else {
        "idle".to_owned()
    };
    let status = read_text(data, KEY_STATUS).unwrap_or_else(|| "none".to_owned());
    let suffix = nonce.map_or(String::new(), |nonce| format!(" nonce={nonce}"));
    format!(
        "/rftoolsutility: transmitter at {pos:?}: energy={energy}/{TRANSMITTER_CAPACITY} dial={dial} state={state} status={status}{suffix}"
    )
}

/// The analyzer's harness line, from the transmitter's stored status and dial.
fn analyzer_line(pos: [i32; 3], status: &str, dial: &str, nonce: Option<&str>) -> String {
    let strength = match status {
        "ok" => 15,
        "unknown" => 7,
        // `warn`, and `none` before the first 20-tick check has run: conservative red.
        _ => 0,
    };
    let suffix = nonce.map_or(String::new(), |nonce| format!(" nonce={nonce}"));
    format!(
        "/rftoolsutility: analyzer at {pos:?}: signal={strength} status={status} dial={dial}{suffix}"
    )
}

/// The sensor's harness line (`SensorTileEntity.getRedstoneOutput` has no GUI readout upstream, so
/// the port prints the strength it hands the redstone hook beside the block id and its setting).
fn sensor_line(
    pos: [i32; 3],
    signal: i32,
    kind: &str,
    target: &str,
    facing: &str,
    nonce: Option<&str>,
) -> String {
    let suffix = nonce.map_or(String::new(), |nonce| format!(" nonce={nonce}"));
    format!(
        "/rftoolsutility: sensor at {pos:?}: signal={signal} type={kind} target={target} area=1 group=one facing={facing}{suffix}"
    )
}

/// One property out of a serialized block state (`"rftoolsutility:sensor[signal=1]"`).
fn sensor_state_property<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let inner = text.split_once('[')?.1.rsplit_once(']')?.0;
    inner.split(',').find_map(|part| {
        let (name, value) = part.split_once('=')?;
        (name == key).then_some(value)
    })
}

/// The sensor's saved setting: the type, the matcher (none = the empty ghost slot) and the input
/// side. The GUI that writes these upstream is replaced by `/rftoolsutility sensor-set`.
#[derive(Clone, Debug, PartialEq, Eq)]
struct SensorConfig {
    kind: SensorKind,
    target: Option<String>,
    facing: String,
}

impl SensorConfig {
    fn from_data(data: &Compound) -> Self {
        let kind = read_text(data, KEY_SENSOR_KIND)
            .and_then(|name| SensorKind::from_name(&name))
            .unwrap_or(SensorKind::Blocks);
        let target = read_text(data, KEY_SENSOR_TARGET).filter(|id| id != "none");
        let facing = read_text(data, KEY_SENSOR_FACING)
            .filter(|name| sensor_front([0, 0, 0], name).is_some())
            .unwrap_or_else(|| "north".to_owned());
        Self {
            kind,
            target,
            facing,
        }
    }

    fn write(&self, nbt: &mut Compound) {
        write_text(nbt, KEY_SENSOR_KIND, self.kind.as_str());
        write_text(
            nbt,
            KEY_SENSOR_TARGET,
            self.target.as_deref().unwrap_or("none"),
        );
        write_text(nbt, KEY_SENSOR_FACING, &self.facing);
    }
}

/// The cached signal strength on the block state, which the redstone hook reads without touching
/// the world (`LevelHost::state_string`, the `laserbridges` receiver pattern).
fn cached_signal(host: &dyn Host, level: LevelRef, pos: [i32; 3]) -> i32 {
    let Some(state) = level.block_state(host, pos) else {
        return SENSOR_OFF;
    };
    let Some(name) = host.block_state_string(state) else {
        return SENSOR_OFF;
    };
    if sensor_state_property(&name, KEY_SIGNAL) == Some("1") {
        SENSOR_ON
    } else {
        SENSOR_OFF
    }
}

/// `SensorTileEntity.tickServer`: every ten ticks check the block on the input side
/// (`checkSensor`), write the 15/0 strength into the block state and wake the neighbours so a
/// vanilla lamp or wire beside the sensor sees it. The block entity's own payload is untouched.
fn sensor_tick(host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
    let pos = event.pos;
    let Some(level) = ticking_level(host) else {
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    };
    let time = level.game_time(&*host).unwrap_or(0);
    if sensor_due(time) {
        let config = SensorConfig::from_data(&event.entity.nbt);
        let sees = sensor_front(pos, &config.facing).is_some_and(|front| {
            let actual = level.block(&*host, front).map(|block| block.id(&*host));
            // `canOcclude()` is not exposed on every build this package must fit; the stable
            // equivalent is the engine's `replaceable` (air and thin plants are replaceable,
            // stone is not).
            let occludes = level
                .block_state(&*host, front)
                .and_then(|state| {
                    host.block_states()
                        .replaceable(state)
                        .map(|replaceable| !replaceable)
                })
                .unwrap_or(false);
            sensor_sees(
                config.kind,
                config.target.as_deref(),
                actual.as_deref(),
                occludes,
            )
        });
        let wanted = sensor_signal(sees);
        if wanted != cached_signal(&*host, level, pos) {
            host.set_block_property(pos, KEY_SIGNAL, if sees { "1" } else { "0" });
            if let Some(state) = level.block_state(&*host, pos) {
                let _ = level.update_neighbors(host, pos, state, NeighbourUpdate::Neighbours);
            }
        }
    }
    *event.reply = BlockEntityTickReply::Unchanged;
}

/// The sensor's one Part: a block entity with a ten-tick check, and the two redstone methods that
/// make the cached strength a real vanilla signal (`getSignal`, `getDirectSignal` and
/// `isSignalSource`). One struct may implement both parts (the `securitycraft` keypad pattern).
struct SensorPart;

#[mod_api::horizon_part]
impl BlockPart for SensorPart {
    fn is_signal_source(&self, _level: &dyn LevelHost, event: &mut IsSignalSourceEvent) {
        event.source = true;
    }

    fn block_signal(&self, level: &dyn LevelHost, event: &mut BlockSignalEvent) {
        let Some(name) = level.state_string(event.state) else {
            return;
        };
        if sensor_state_property(&name, KEY_SIGNAL) == Some("1") {
            event.signal = SENSOR_ON;
        }
    }
}

#[mod_api::horizon_part]
impl BlockEntityPart for SensorPart {
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        sensor_tick(host, event);
    }
}

// ---------------------------------------------------------------------------
// The sequencer's glue (slice 12)
// ---------------------------------------------------------------------------

/// A boolean out of the block entity's compound (upstream writes NBT booleans as bytes).
fn read_flag(nbt: &Compound, name: &str) -> bool {
    matches!(tag(nbt, name), Some(Tag::Byte(value)) if *value != 0)
}

fn write_flag(nbt: &mut Compound, name: &str, value: bool) {
    put(nbt, name, Tag::Byte(i8::from(value)));
}

/// An int with a fallback, for a declared block entity whose compound starts empty: the artefact's
/// `SequencerData.createDefault()` supplies the values a GUI would otherwise have written.
fn read_int_or(nbt: &Compound, name: &str, default: i32) -> i32 {
    match tag(nbt, name) {
        Some(Tag::Int(value)) => *value,
        _ => default,
    }
}

/// `SequencerData` out of the saved compound. A 64-bit pattern is an NBT Long (`Tag::Long`).
fn sequencer_data(nbt: &Compound) -> SequencerData {
    let defaults = SequencerData::default();
    SequencerData {
        bits: match tag(nbt, KEY_SEQ_BITS) {
            Some(Tag::Long(value)) => *value as u64,
            _ => defaults.bits,
        },
        mode: read_text(nbt, KEY_SEQ_MODE)
            .and_then(|name| SequencerMode::from_name(&name))
            .unwrap_or(defaults.mode),
        delay: read_int_or(nbt, KEY_SEQ_DELAY, defaults.delay),
        stepcount: read_int_or(nbt, KEY_SEQ_COUNT, defaults.stepcount),
        endstate: read_flag(nbt, KEY_SEQ_ENDSTATE),
    }
}

/// The live tick state out of the saved compound, using the mode's own starting step and delay
/// when a freshly placed sequencer has no saved fields yet.
fn sequencer_state(nbt: &Compound, data: &SequencerData) -> SequencerState {
    SequencerState {
        step: read_int_or(nbt, KEY_SEQ_STEP, data.mode.initial_step()),
        prev_in: read_flag(nbt, KEY_SEQ_PREV),
        timer: read_int_or(nbt, KEY_SEQ_TIMER, data.delay),
        output: read_flag(nbt, KEY_SEQ_RS),
    }
}

/// Write both halves back. The engine persists a block entity's compound when its tick answers
/// `Changed` (the default) — the same path the crafter's and the spawner's ticks use.
fn write_sequencer(nbt: &mut Compound, data: &SequencerData, state: &SequencerState) {
    put(nbt, KEY_SEQ_BITS, Tag::Long(data.bits as i64));
    write_text(nbt, KEY_SEQ_MODE, data.mode.as_str());
    write_int(nbt, KEY_SEQ_DELAY, data.delay);
    write_int(nbt, KEY_SEQ_COUNT, data.stepcount);
    write_flag(nbt, KEY_SEQ_ENDSTATE, data.endstate);
    write_int(nbt, KEY_SEQ_STEP, state.step);
    write_flag(nbt, KEY_SEQ_PREV, state.prev_in);
    write_int(nbt, KEY_SEQ_TIMER, state.timer);
    write_flag(nbt, KEY_SEQ_RS, state.output);
}

/// `/rftoolsutility sequencer <pos>`: the harness reads the **parsed** step, pattern and 15/0
/// output, not the absence of an error. `bits` is decimal so a 64-bit pattern round-trips exactly.
fn sequencer_line(
    pos: [i32; 3],
    data: &SequencerData,
    state: &SequencerState,
    facing: &str,
    nonce: Option<&str>,
) -> String {
    let suffix = nonce.map_or(String::new(), |nonce| format!(" nonce={nonce}"));
    format!(
        "/rftoolsutility: sequencer at {pos:?}: step={} bits={} mode={} output={} delay={} count={} endstate={} facing={facing}{suffix}",
        state.step,
        data.bits,
        data.mode.as_str(),
        if state.output { SENSOR_ON } else { SENSOR_OFF },
        data.delay,
        data.stepcount,
        data.endstate,
    )
}

/// The four saved input-side spellings (the jar's 24 `logic_facing` slab rotations are not
/// expressible by a content block's states, so the side is a saved advisory).
fn sequencer_front(facing: &str) -> bool {
    sensor_front([0, 0, 0], facing).is_some()
}

/// The harness's `/rftoolsutility sequencer-set` argument: a 64-bit pattern spelled in decimal or
/// as `0x` hex (the SDK's `word` argument keeps both spellings).
fn parse_bits(text: &str) -> Option<u64> {
    match text.strip_prefix("0x") {
        Some(hex) => u64::from_str_radix(hex, 16).ok(),
        None => text.parse::<u64>().ok(),
    }
}

/// `SequencerTileEntity.tickOnServer`: latch a rising edge of the adjacent redstone power, run the
/// mode's state machine, persist the live fields and mirror the cached 15/0 into the block's
/// `signal` property so a vanilla lamp or wire beside it sees the result. The block entity's own
/// payload carries the state; only the cached strength goes into the block state, as the sensor's
/// tick does.
fn sequencer_block_tick(host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
    let pos = event.pos;
    let data = sequencer_data(&event.entity.nbt);
    let mut state = sequencer_state(&event.entity.nbt, &data);
    let Some(level) = ticking_level(&*host) else {
        return;
    };
    // Read the *input side* neighbour's own emitted signal (`SignalGetter.getSignal(pos, dir)`),
    // not `getBestNeighborSignal`: the latter includes this block's own 15/0 output (it is a
    // signal source), so a high output would latch a phantom rising edge on the next tick and
    // advance the pattern by itself. A vanilla lever on the configured input side gives a clean
    // 0 → 15 → 0 edge here, so no GUI or chat-driven substitute is invented.
    let facing = read_text(&event.entity.nbt, KEY_SEQ_FACING).unwrap_or_else(|| "north".to_owned());
    let (front, into) = match facing.as_str() {
        "north" => ([pos[0], pos[1], pos[2] - 1], Direction::South),
        "south" => ([pos[0], pos[1], pos[2] + 1], Direction::North),
        "west" => ([pos[0] - 1, pos[1], pos[2]], Direction::East),
        "east" => ([pos[0] + 1, pos[1], pos[2]], Direction::West),
        _ => ([pos[0], pos[1], pos[2] - 1], Direction::South),
    };
    let powered = level.signal(&*host, front, into).unwrap_or(0) > 0;
    run_sequencer_tick(&data, &mut state, powered);
    write_sequencer(&mut event.entity.nbt, &data, &state);
    let wanted = if state.output { SENSOR_ON } else { SENSOR_OFF };
    if wanted != cached_signal(&*host, level, pos) {
        host.set_block_property(pos, KEY_SIGNAL, if state.output { "1" } else { "0" });
        if let Some(block_state) = level.block_state(&*host, pos) {
            let _ = level.update_neighbors(host, pos, block_state, NeighbourUpdate::Neighbours);
        }
    }
}

/// The sequencer's one Part: a block entity with a tick, and the two redstone methods that make
/// its cached 15/0 a real vanilla signal (`getRedstoneOutput`/`isSignalSource` in `LogicSupport`).
struct SequencerPart;

#[mod_api::horizon_part]
impl BlockPart for SequencerPart {
    fn is_signal_source(&self, _level: &dyn LevelHost, event: &mut IsSignalSourceEvent) {
        event.source = true;
    }

    fn block_signal(&self, level: &dyn LevelHost, event: &mut BlockSignalEvent) {
        let Some(name) = level.state_string(event.state) else {
            return;
        };
        if sensor_state_property(&name, KEY_SIGNAL) == Some("1") {
            event.signal = SENSOR_ON;
        }
    }
}

#[mod_api::horizon_part]
impl BlockEntityPart for SequencerPart {
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        sequencer_block_tick(host, event);
    }
}

// ---------------------------------------------------------------------------
// The timer's glue (slice 13)
// ---------------------------------------------------------------------------

/// `TimerData` out of the saved compound; a freshly placed block entity's compound is empty, so
/// the artefact's `createDefault()` supplies the two values a GUI would have written.
fn timer_data(nbt: &Compound) -> TimerData {
    let defaults = TimerData::default();
    TimerData {
        delay: read_int_or(nbt, KEY_TIMER_DELAY, defaults.delay),
        redstone_pauses: read_flag(nbt, KEY_TIMER_PAUSES),
    }
}

/// The live fields `TimerTileEntity` saves beside its data (`prevIn`, `timer`, `rs`).
fn timer_state(nbt: &Compound) -> TimerState {
    TimerState {
        prev_in: read_flag(nbt, KEY_TIMER_PREV),
        timer: read_int_or(nbt, KEY_TIMER_TIMER, 0),
        output: read_flag(nbt, KEY_TIMER_RS),
    }
}

/// Write the data and the live fields back. The engine persists the compound when the tick answers
/// `Changed` (the default), as the sequencer's tick does.
fn write_timer(nbt: &mut Compound, data: &TimerData, state: &TimerState) {
    write_int(nbt, KEY_TIMER_DELAY, data.delay);
    write_flag(nbt, KEY_TIMER_PAUSES, data.redstone_pauses);
    write_flag(nbt, KEY_TIMER_PREV, state.prev_in);
    write_int(nbt, KEY_TIMER_TIMER, state.timer);
    write_flag(nbt, KEY_TIMER_RS, state.output);
}

/// The four saved input-side spellings (the jar's 24 `logic_facing` slab rotations cannot be a
/// content block's states, so the side is a saved advisory).
fn timer_front(facing: &str) -> bool {
    sensor_front([0, 0, 0], facing).is_some()
}

/// `/rftoolsutility timer <pos> [nonce]`: the harness reads the **parsed** delay, countdown and
/// 15/0 output. `pulse`/`fired` are game-time stamps and `time` is the readout's own tick, so the
/// leg can say "exactly `delay` ticks" from the replies alone.
#[allow(clippy::too_many_arguments)]
fn timer_line(
    pos: [i32; 3],
    data: &TimerData,
    state: &TimerState,
    pulse: i32,
    fired: i32,
    facing: &str,
    time: i32,
    lamp: &str,
    nonce: Option<&str>,
) -> String {
    let suffix = nonce.map_or(String::new(), |nonce| format!(" nonce={nonce}"));
    format!(
        "/rftoolsutility: timer at {pos:?}: delay={} pauses={} output={} timer={} pulse={pulse} fired={fired} facing={facing} time={time} lamp={lamp}{suffix}",
        data.delay,
        data.redstone_pauses,
        timer_signal(state.output),
        state.timer,
    )
}

/// `TimerTileEntity.tickOnServer`: latch a rising edge of the input-side neighbour's own signal,
/// run the artefact's countdown, persist the live fields and mirror the cached 15/0 into the
/// block's `signal` property so a vanilla lamp beside it sees the pulse. The neighbour is read
/// **directionally** (`SignalGetter.getSignal(neighbour, into)`), never with
/// `getBestNeighborSignal`: the latter includes this block's own 15 and would latch a phantom
/// edge on the next tick (sequencer, slice 12).
fn timer_block_tick(host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
    let pos = event.pos;
    let data = timer_data(&event.entity.nbt);
    let mut state = timer_state(&event.entity.nbt);
    let Some(level) = ticking_level(&*host) else {
        return;
    };
    let facing =
        read_text(&event.entity.nbt, KEY_TIMER_FACING).unwrap_or_else(|| "north".to_owned());
    let (front, into) = match facing.as_str() {
        "north" => ([pos[0], pos[1], pos[2] - 1], Direction::South),
        "south" => ([pos[0], pos[1], pos[2] + 1], Direction::North),
        "west" => ([pos[0] - 1, pos[1], pos[2]], Direction::East),
        "east" => ([pos[0] + 1, pos[1], pos[2]], Direction::West),
        _ => ([pos[0], pos[1], pos[2] - 1], Direction::South),
    };
    let powered = level.signal(&*host, front, into).unwrap_or(0) > 0;
    let time = level.game_time(&*host).unwrap_or(0) as i32;
    let was_output = state.output;
    let was_prev = state.prev_in;
    run_timer_tick(&data, &mut state, powered);
    write_timer(&mut event.entity.nbt, &data, &state);
    if rising_edge(powered, was_prev) {
        write_int(&mut event.entity.nbt, KEY_TIMER_PULSE_AT, time);
        write_flag(&mut event.entity.nbt, KEY_TIMER_FIRED_LOCK, false);
    }
    let wanted = timer_signal(state.output);
    if wanted != cached_signal(&*host, level, pos) {
        host.set_block_property(pos, KEY_SIGNAL, if state.output { "1" } else { "0" });
        if let Some(block_state) = level.block_state(&*host, pos) {
            let _ = level.update_neighbors(host, pos, block_state, NeighbourUpdate::Neighbours);
        }
    }
    // Stamp the *first* fire after the edge once, with the lamp's own state at that instant. The
    // one-tick pulse is far too brief for a polling `ctl block`, so this captured stamp is the
    // fail-capable evidence of the lamp flip; it stays until the next edge or a `timer-set`.
    if !was_output && state.output && !read_flag(&event.entity.nbt, KEY_TIMER_FIRED_LOCK) {
        write_int(&mut event.entity.nbt, KEY_TIMER_FIRED_AT, time);
        write_flag(&mut event.entity.nbt, KEY_TIMER_FIRED_LOCK, true);
        let lit = [
            [pos[0], pos[1], pos[2] - 1],
            [pos[0], pos[1], pos[2] + 1],
            [pos[0] - 1, pos[1], pos[2]],
            [pos[0] + 1, pos[1], pos[2]],
        ]
        .iter()
        .any(|&around| {
            level
                .block_state(&*host, around)
                .and_then(|s| host.block_state_string(s))
                .is_some_and(|name| {
                    name.contains("redstone_lamp")
                        && sensor_state_property(&name, "lit") == Some("true")
                })
        });
        write_flag(&mut event.entity.nbt, KEY_TIMER_LAMP, lit);
    }
}

/// The timer's one Part: a block entity with a tick, and the two redstone methods that make its
/// cached 15/0 a real vanilla signal (`LogicSupport.getRedstoneOutput`/`isSignalSource`).
struct TimerPart;

#[mod_api::horizon_part]
impl BlockPart for TimerPart {
    fn is_signal_source(&self, _level: &dyn LevelHost, event: &mut IsSignalSourceEvent) {
        event.source = true;
    }

    fn block_signal(&self, level: &dyn LevelHost, event: &mut BlockSignalEvent) {
        let Some(name) = level.state_string(event.state) else {
            return;
        };
        if sensor_state_property(&name, KEY_SIGNAL) == Some("1") {
            event.signal = SENSOR_ON;
        }
    }
}

#[mod_api::horizon_part]
impl BlockEntityPart for TimerPart {
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        timer_block_tick(host, event);
    }
}

// ---------------------------------------------------------------------------
// The counter's glue (slice 14)
// ---------------------------------------------------------------------------

/// `CounterData` out of the saved compound; a freshly placed block entity's compound is empty, so
/// the artefact's `createDefault()` (limit 1, current 0) supplies what a GUI would have written.
fn counter_data(nbt: &Compound) -> CounterData {
    let defaults = CounterData::default();
    CounterData {
        counter: read_int_or(nbt, KEY_COUNTER_LIMIT, defaults.counter),
        current: read_int_or(nbt, KEY_COUNTER_CURRENT, defaults.current),
    }
}

/// The two live fields `CounterTileEntity` saves beside its data (`prevIn`, `rs`).
fn counter_state(nbt: &Compound) -> CounterState {
    CounterState {
        prev_in: read_flag(nbt, KEY_COUNTER_PREV),
        output: read_flag(nbt, KEY_COUNTER_RS),
    }
}

/// Write the data and the live fields back. The engine persists the compound when the tick answers
/// `Changed` (the default), as the sequencer's and the timer's ticks do.
fn write_counter(nbt: &mut Compound, data: &CounterData, state: &CounterState) {
    write_int(nbt, KEY_COUNTER_LIMIT, data.counter);
    write_int(nbt, KEY_COUNTER_CURRENT, data.current);
    write_flag(nbt, KEY_COUNTER_PREV, state.prev_in);
    write_flag(nbt, KEY_COUNTER_RS, state.output);
}

/// The four saved input-side spellings (the jar's 24 `logic_facing` slab rotations cannot be a
/// content block's states, so the side is a saved advisory).
fn counter_front(facing: &str) -> bool {
    sensor_front([0, 0, 0], facing).is_some()
}

/// `/rftoolsutility counter <pos> [nonce]`: the harness reads the **parsed** limit, running count,
/// latched 15/0 output, the number of pulses and the game-time stamp of the fire, so it can say
/// "the lamp lit only on the limit-th pulse" from the reply alone.
#[expect(
    clippy::too_many_arguments,
    reason = "the readout carries every parsed field plus game-time stamps for the harness"
)]
fn counter_line(
    pos: [i32; 3],
    data: &CounterData,
    state: &CounterState,
    pulses: i32,
    pulse: i32,
    fire: i32,
    facing: &str,
    time: i32,
    lamp: &str,
    nonce: Option<&str>,
) -> String {
    let suffix = nonce.map_or(String::new(), |nonce| format!(" nonce={nonce}"));
    format!(
        "/rftoolsutility: counter at {pos:?}: limit={} current={} output={} pulses={pulses} pulse={pulse} fire={fire} lamp={lamp} facing={facing} time={time}{suffix}",
        data.counter,
        data.current,
        counter_signal(state.output),
    )
}

/// `CounterTileEntity.update()` (from `checkRedstone`): latch a rising edge of the input-side
/// neighbour's own signal, bump the count, fire a level 15 when it reaches the limit (wrapping the
/// count to 0), and mirror the cached 15/0 into the block's `signal` property so a vanilla lamp
/// beside it sees the result. The neighbour is read **directionally** (`SignalGetter.getSignal`),
/// never with `getBestNeighborSignal`, which would see this block's own 15 as a phantom edge
/// (sequencer, slice 12). The fire/lamp/pulse stamps are harness-only evidence, saved in the
/// compound and asserted from the readout.
fn counter_block_tick(host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
    let pos = event.pos;
    let mut data = counter_data(&event.entity.nbt);
    let mut state = counter_state(&event.entity.nbt);
    let Some(level) = ticking_level(&*host) else {
        return;
    };
    let facing =
        read_text(&event.entity.nbt, KEY_COUNTER_FACING).unwrap_or_else(|| "north".to_owned());
    let (front, into) = match facing.as_str() {
        "north" => ([pos[0], pos[1], pos[2] - 1], Direction::South),
        "south" => ([pos[0], pos[1], pos[2] + 1], Direction::North),
        "west" => ([pos[0] - 1, pos[1], pos[2]], Direction::East),
        "east" => ([pos[0] + 1, pos[1], pos[2]], Direction::West),
        _ => ([pos[0], pos[1], pos[2] - 1], Direction::South),
    };
    let powered = level.signal(&*host, front, into).unwrap_or(0) > 0;
    let was_prev = state.prev_in;
    let time = level.game_time(&*host).unwrap_or(0) as i32;
    let fired = run_counter_tick(&mut data, &mut state, powered);
    write_counter(&mut event.entity.nbt, &data, &state);
    if rising_edge(powered, was_prev) {
        let pulses = read_int_or(&event.entity.nbt, KEY_COUNTER_PULSES, 0) + 1;
        write_int(&mut event.entity.nbt, KEY_COUNTER_PULSE_AT, time);
        write_int(&mut event.entity.nbt, KEY_COUNTER_PULSES, pulses);
    }
    let wanted = counter_signal(state.output);
    if wanted != cached_signal(&*host, level, pos) {
        host.set_block_property(pos, KEY_SIGNAL, if state.output { "1" } else { "0" });
        if let Some(block_state) = level.block_state(&*host, pos) {
            let _ = level.update_neighbors(host, pos, block_state, NeighbourUpdate::Neighbours);
        }
    }
    // Stamp the *first* fire after a counter-set once, with the adjacent lamp's own `lit` state at
    // that instant. Unlike the timer's one-tick pulse, the counter's output is a level latch, but a
    // game-time stamp is still the fail-capable observation the leg asserts (no polling race).
    if fired && !read_flag(&event.entity.nbt, KEY_COUNTER_FIRED_LOCK) {
        write_int(&mut event.entity.nbt, KEY_COUNTER_FIRE_AT, time);
        write_flag(&mut event.entity.nbt, KEY_COUNTER_FIRED_LOCK, true);
        let lit = [
            [pos[0], pos[1], pos[2] - 1],
            [pos[0], pos[1], pos[2] + 1],
            [pos[0] - 1, pos[1], pos[2]],
            [pos[0] + 1, pos[1], pos[2]],
        ]
        .iter()
        .any(|&around| {
            level
                .block_state(&*host, around)
                .and_then(|s| host.block_state_string(s))
                .is_some_and(|name| {
                    name.contains("redstone_lamp")
                        && sensor_state_property(&name, "lit") == Some("true")
                })
        });
        write_flag(&mut event.entity.nbt, KEY_COUNTER_LAMP, lit);
    }
}

/// The counter's one Part: a block entity with a tick, and the two redstone methods that make its
/// cached 15/0 a real vanilla signal (`LogicSupport.getRedstoneOutput`/`isSignalSource`).
struct CounterPart;

#[mod_api::horizon_part]
impl BlockPart for CounterPart {
    fn is_signal_source(&self, _level: &dyn LevelHost, event: &mut IsSignalSourceEvent) {
        event.source = true;
    }

    fn block_signal(&self, level: &dyn LevelHost, event: &mut BlockSignalEvent) {
        let Some(name) = level.state_string(event.state) else {
            return;
        };
        if sensor_state_property(&name, KEY_SIGNAL) == Some("1") {
            event.signal = SENSOR_ON;
        }
    }
}

#[mod_api::horizon_part]
impl BlockEntityPart for CounterPart {
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        counter_block_tick(host, event);
    }
}

// ---------------------------------------------------------------------------
// The analog's glue (slice 15) — `AnalogData`, `AnalogTileEntity.checkRedstone`
// ---------------------------------------------------------------------------

/// One f32 under `name` (`Tag::Float` carries the bits), or the artefact's default when absent.
fn read_float_or(nbt: &Compound, name: &str, default: f32) -> f32 {
    match tag(nbt, name) {
        Some(Tag::Float(bits)) => f32::from_bits(*bits),
        _ => default,
    }
}

/// Save an f32 under `name`, replacing any value of another kind.
fn write_float(nbt: &mut Compound, name: &str, value: f32) {
    put(nbt, name, Tag::Float(value.to_bits()));
}

/// `AnalogData` out of the saved compound. A freshly placed block entity has an empty compound,
/// so the artefact's identity `createDefault()` supplies the six GUI fields.
fn analog_data(nbt: &Compound) -> AnalogData {
    let defaults = AnalogData::default();
    AnalogData {
        mul_equal: read_float_or(nbt, KEY_ANALOG_MUL_EQ, defaults.mul_equal),
        mul_less: read_float_or(nbt, KEY_ANALOG_MUL_LESS, defaults.mul_less),
        mul_greater: read_float_or(nbt, KEY_ANALOG_MUL_GREATER, defaults.mul_greater),
        add_equal: read_int_or(nbt, KEY_ANALOG_ADD_EQ, defaults.add_equal),
        add_less: read_int_or(nbt, KEY_ANALOG_ADD_LESS, defaults.add_less),
        add_greater: read_int_or(nbt, KEY_ANALOG_ADD_GREATER, defaults.add_greater),
    }
}

/// Write the six `AnalogData` fields back under their artefact codec names.
fn write_analog(nbt: &mut Compound, data: &AnalogData) {
    write_float(nbt, KEY_ANALOG_MUL_EQ, data.mul_equal);
    write_float(nbt, KEY_ANALOG_MUL_LESS, data.mul_less);
    write_float(nbt, KEY_ANALOG_MUL_GREATER, data.mul_greater);
    write_int(nbt, KEY_ANALOG_ADD_EQ, data.add_equal);
    write_int(nbt, KEY_ANALOG_ADD_LESS, data.add_less);
    write_int(nbt, KEY_ANALOG_ADD_GREATER, data.add_greater);
}

/// Parse the cached `signal` property out of a serialized block state. Unlike the other logic
/// siblings this is a full 0..15 strength, which `BlockPart::block_signal` reports verbatim.
fn analog_signal_from_state(name: &str) -> i32 {
    sensor_state_property(name, KEY_SIGNAL)
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(0)
}

/// The strength cached in the game's block state (what the redstone hook reports).
fn analog_cached_signal(host: &dyn Host, level: LevelRef, pos: [i32; 3]) -> i32 {
    level
        .block_state(host, pos)
        .and_then(|state| host.block_state_string(state))
        .map_or(0, |name| analog_signal_from_state(&name))
}

/// The direction from each of the three neighbours back into the analog, in the same `[I, A, B]`
/// order as [`analog_neighbours`]. Index 0…3 is north/east/south/west; the opposite side is +2.
fn analog_into_directions(facing: &str) -> [Direction; 3] {
    let index = match facing {
        "east" => 1,
        "south" => 2,
        "west" => 3,
        _ => 0,
    };
    let into = |side: usize| match side % 4 {
        0 => Direction::South,
        1 => Direction::West,
        2 => Direction::North,
        _ => Direction::East,
    };
    [into(index), into(index + 1), into(index + 3)]
}

/// `/rftoolsutility analog <pos> [nonce]`: the harness reads the three input strengths, the chosen
/// branch and the exact 0..15 strength cached in the block state. `changed` is the game time of the
/// last output change and `lamp` is a live read of the adjacent vanilla lamp, so the leg can say
/// "this input produced this strength at this tick" from the reply alone.
#[expect(
    clippy::too_many_arguments,
    reason = "the readout carries the three inputs, the six settings and game-time evidence"
)]
fn analog_line(
    pos: [i32; 3],
    data: &AnalogData,
    input: i32,
    a: i32,
    b: i32,
    output: i32,
    changed: i32,
    facing: &str,
    time: i32,
    lamp: &str,
    nonce: Option<&str>,
) -> String {
    let suffix = nonce.map_or(String::new(), |nonce| format!(" nonce={nonce}"));
    format!(
        "/rftoolsutility: analog at {pos:?}: input={input} a={a} b={b} branch={} mul={},{},{} add={},{},{} output={output} changed={changed} lamp={lamp} facing={facing} time={time}{suffix}",
        analog_branch(a, b).as_str(),
        data.mul_equal,
        data.mul_less,
        data.mul_greater,
        data.add_equal,
        data.add_less,
        data.add_greater,
    )
}

/// `AnalogTileEntity.checkRedstone`: read the three input-side neighbours' own emitted signals,
/// apply the artefact's branch arithmetic, clamp it to 0..15, mirror the exact strength into the
/// block state and wake the neighbours only when it changes. The neighbours are read
/// **directionally** (`SignalGetter.getSignal`), never with `getBestNeighborSignal`, which would
/// see this block's own output as a phantom input (sequencer, slice 12).
fn analog_block_tick(host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
    let pos = event.pos;
    let data = analog_data(&event.entity.nbt);
    let Some(level) = ticking_level(&*host) else {
        return;
    };
    let facing =
        read_text(&event.entity.nbt, KEY_ANALOG_FACING).unwrap_or_else(|| "north".to_owned());
    let Some(neighbours) = analog_neighbours(pos, &facing) else {
        return;
    };
    let intos = analog_into_directions(&facing);
    let mut strengths = [0i32; 3];
    for (slot, neighbour) in neighbours.iter().enumerate() {
        strengths[slot] = level.signal(&*host, *neighbour, intos[slot]).unwrap_or(0);
    }
    let [input, a, b] = strengths;
    let output = analog_output(input, a, b, &data);
    write_analog(&mut event.entity.nbt, &data);
    write_text(&mut event.entity.nbt, KEY_ANALOG_FACING, &facing);
    write_int(&mut event.entity.nbt, KEY_ANALOG_INPUT, input);
    write_int(&mut event.entity.nbt, KEY_ANALOG_A, a);
    write_int(&mut event.entity.nbt, KEY_ANALOG_B, b);
    if output != analog_cached_signal(&*host, level, pos) {
        let time = level.game_time(&*host).unwrap_or(0) as i32;
        write_int(&mut event.entity.nbt, KEY_ANALOG_CHANGED_AT, time);
        host.set_block_property(pos, KEY_SIGNAL, &output.to_string());
        if let Some(state) = level.block_state(&*host, pos) {
            let _ = level.update_neighbors(host, pos, state, NeighbourUpdate::Neighbours);
        }
    }
}

/// The analog's one Part: a block entity with a tick, and the two redstone methods that make its
/// cached 0..15 strength a real vanilla signal (`LogicSupport.getRedstoneOutput`/`isSignalSource`).
struct AnalogPart;

#[mod_api::horizon_part]
impl BlockPart for AnalogPart {
    fn is_signal_source(&self, _level: &dyn LevelHost, event: &mut IsSignalSourceEvent) {
        event.source = true;
    }

    fn block_signal(&self, level: &dyn LevelHost, event: &mut BlockSignalEvent) {
        let Some(name) = level.state_string(event.state) else {
            return;
        };
        event.signal = analog_signal_from_state(&name);
    }
}

#[mod_api::horizon_part]
impl BlockEntityPart for AnalogPart {
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        analog_block_tick(host, event);
    }
}

/// `/rftoolsutility analyzer <pos> [nonce]`: the harness's readout of the passive analyzer block.
/// It finds the adjacent dialed transmitter and prints the receiver-check status the transmitter
/// keeps (`green=ok` → 15, `yellow=unknown` → 7, `red=warn` → 0). The artefact has no redstone
/// emitter here, so this is the harness's own signal, not a vanilla `ctl block` power level.
fn analyzer_readout(
    host: &dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
    nonce: Option<&str>,
) -> String {
    if level
        .block(host, pos)
        .is_none_or(|b| b.id(host) != ANALYZER)
    {
        return format!("/rftoolsutility: no destination analyzer at {pos:?}.");
    }
    let around = [
        [1, 0, 0],
        [-1, 0, 0],
        [0, 1, 0],
        [0, -1, 0],
        [0, 0, 1],
        [0, 0, -1],
    ];
    let tx = around.iter().find_map(|off| {
        let at = [pos[0] + off[0], pos[1] + off[1], pos[2] + off[2]];
        transmitter_at(host, level, at)
    });
    let Some(entity) = tx else {
        return format!("/rftoolsutility: no transmitter adjacent to the analyzer at {pos:?}.");
    };
    let Some(data) = entity.data(host) else {
        return format!("/rftoolsutility: no transmitter adjacent to the analyzer at {pos:?}.");
    };
    let status = read_text(&data, KEY_STATUS).unwrap_or_else(|| "none".to_owned());
    let dial = match (
        read_text(&data, KEY_DIAL_DIM),
        read_pos(&data, KEY_DIAL_POS),
    ) {
        (Some(dim), Some(dest)) => format!("{dim}@{dest:?}"),
        _ => "none".to_owned(),
    };
    analyzer_line(pos, &status, &dial, nonce)
}

/// `/rftoolsutility dial <pos> <dim> <x> <y> <z>`: what the dialing device writes into the
/// transmitter's `MatterTransmitterData.destination` (the GUI's job upstream; this port has no
/// GUI). A dimension of `none` clears the dial.
fn dial_transmitter(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
    dim: &str,
    dest: [i32; 3],
) -> String {
    let Some(entity) = transmitter_at(&*host, level, pos) else {
        return "/rftoolsutility: no matter transmitter there.".to_owned();
    };
    let Some(mut data) = entity.data(&*host) else {
        return "/rftoolsutility: no matter transmitter there.".to_owned();
    };
    if dim == "none" {
        remove(&mut data, KEY_DIAL_DIM);
        remove(&mut data, KEY_DIAL_POS);
        clear_teleport(&mut data, 0);
    } else {
        // Upstream resolves a GlobalPos: the destination dimension must be a loaded level.
        if level_of(&*host, dim).is_none() {
            return format!("/rftoolsutility: unknown destination dimension {dim}.");
        }
        write_text(&mut data, KEY_DIAL_DIM, dim);
        write_pos(&mut data, KEY_DIAL_POS, dest);
        clear_teleport(&mut data, 0);
    }
    if entity.replace_data(host, data).is_err() {
        return "/rftoolsutility: no matter transmitter there.".to_owned();
    }
    format!("/rftoolsutility: the transmitter at {pos:?} is dialed to {dim}@{dest:?}.")
}

/// One server tick of the matter transmitter (`MatterTransmitterTileEntity.tickServer`): while
/// dialed, a player standing in the beam is found, charged per tick, and finally moved to the
/// dialed dimension and coordinate. A lost player, a moved player or a power failure interrupts
/// the teleport with a cooldown; everything is paid only when the tick succeeds.
fn transmitter_tick(host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
    let pos = event.pos;
    let Some(level) = ticking_level(&*host) else {
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    };
    let nbt = &mut event.entity.nbt;
    let (Some(dim), Some(dest)) = (read_text(nbt, KEY_DIAL_DIM), read_pos(nbt, KEY_DIAL_POS))
    else {
        remove(nbt, KEY_STATUS);
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    };
    // Slice 10: an adjacent destination analyzer lets a dialed transmitter check its receiver's
    // status every 20 ticks (upstream's `checkReceiverStatusCounter`). Without one upstream forces
    // STATUS_OK — no red beam — so a removed analyzer clears the checked state.
    let analyzer = analyzer_adjacent(&*host, level, pos);
    if analyzer && level.game_time(&*host).is_some_and(analyzer_due) {
        write_text(
            nbt,
            KEY_STATUS,
            check_receiver_status(&*host, &dim, dest).as_str(),
        );
    } else if !analyzer && read_text(nbt, KEY_STATUS).is_some() {
        write_text(nbt, KEY_STATUS, "ok");
    }
    let cooldown = read_int(nbt, KEY_COOLDOWN);
    if cooldown > 0 {
        write_int(nbt, KEY_COOLDOWN, cooldown - 1);
        return;
    }

    if let Some(uuid_text) = read_text(nbt, KEY_TP_PLAYER) {
        let Ok(uuid) = uuid_text.parse::<u128>() else {
            clear_teleport(nbt, 80);
            return;
        };
        let Some(player) = host.player(uuid) else {
            clear_teleport(nbt, 80);
            return;
        };
        let in_beam_now = player
            .entity()
            .position(&*host)
            .is_some_and(|feet| in_beam(pos, feet));
        if !in_beam_now {
            // `isPlayerOutsideBeam`: leaving the beam interrupts the teleport.
            clear_teleport(nbt, 80);
            return;
        }
        let energy = read_int(nbt, KEY_ENERGY).clamp(0, TRANSMITTER_CAPACITY);
        let fee = read_int(nbt, KEY_RFPT).clamp(RF_PER_TICK, i32::MAX);
        let Some(left) = (energy >= fee).then_some(energy - fee) else {
            // `handleEnergyShortage`: a bad tick; too many interrupt.
            let bad = read_int(nbt, KEY_BAD).saturating_add(1);
            let total = read_int(nbt, KEY_TOTAL).max(1);
            write_int(nbt, KEY_BAD, bad);
            if bad * 2 > total {
                clear_teleport(nbt, 200);
            }
            return;
        };
        write_int(nbt, KEY_ENERGY, left);
        write_int(nbt, KEY_GOOD, read_int(nbt, KEY_GOOD).saturating_add(1));
        let timer = read_int(nbt, KEY_TP_TIMER) - 1;
        if timer > 0 {
            write_int(nbt, KEY_TP_TIMER, timer);
            return;
        }
        // `performTeleport`: move the player to the dialed coordinate. A cross-dimension move
        // uses `Entity.teleport` (the SDK's `teleport_to_level`), the same-level one `teleportTo`.
        let player_entity = player.entity();
        let target = [
            f64::from(dest[0]) + 0.5,
            f64::from(dest[1]) + 1.0,
            f64::from(dest[2]) + 0.5,
        ];
        let moved = player_entity.teleport_to_level(host, &dim, target, None);
        // A booster plus 20,000 FE mitigates an unpowered receiver (upstream's `boosted` flag).
        if booster_adjacent(&*host, level, pos) && read_int(nbt, KEY_ENERGY) >= BOOST_RF {
            write_int(nbt, KEY_ENERGY, read_int(nbt, KEY_ENERGY) - BOOST_RF);
        }
        let _ = moved;
        clear_teleport(nbt, 80);
        return;
    }

    // No player in transit: look for one standing in the beam above this transmitter.
    let Some(players) = level.online_players(&*host) else {
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    };
    for player in players {
        let Some(feet) = player.entity().position(&*host) else {
            continue;
        };
        if !in_beam(pos, feet) {
            continue;
        }
        let same_dim = player
            .dimension(&*host)
            .is_some_and(|found| found.to_string() == dim);
        let cost = start_cost(pos, dest, same_dim);
        let time = start_time(pos, dest, same_dim).max(1);
        let energy = read_int(nbt, KEY_ENERGY).clamp(0, TRANSMITTER_CAPACITY);
        if energy < cost {
            // `startTeleportation`: not enough power to begin.
            write_int(nbt, KEY_COOLDOWN, 80);
            return;
        }
        // Upstream amortizes the start cost into the per-tick fee (`rfPerTick`).
        let fee = (cost + RF_PER_TICK * (time + 1)) / (time + 1);
        write_text(nbt, KEY_TP_PLAYER, &player.uuid().to_string());
        write_int(nbt, KEY_TP_TIMER, time);
        write_int(nbt, KEY_TOTAL, time);
        write_int(nbt, KEY_GOOD, 0);
        write_int(nbt, KEY_BAD, 0);
        write_int(nbt, KEY_RFPT, fee);
        return;
    }
}

/// The transmitter's energy domain (`GenericEnergyStorage(this, true, 200,000, 1,000)`): a fill
/// only, on every face.
fn transmitter_energy() -> ModEnergyBuffer {
    ModEnergyBuffer {
        capacity: TRANSMITTER_CAPACITY,
        max_insert: TRANSMITTER_RECEIVE,
        max_extract: 0,
        insert_faces: [true; 6],
        extract_faces: [false; 6],
    }
}

/// The receiver's energy domain (`GenericEnergyStorage(this, true, 100,000, 500)`): a fill only.
fn receiver_energy() -> ModEnergyBuffer {
    ModEnergyBuffer {
        capacity: RECEIVER_CAPACITY,
        max_insert: RECEIVER_RECEIVE,
        max_extract: 0,
        insert_faces: [true; 6],
        extract_faces: [false; 6],
    }
}

/// The transmitter's block entity and its server tick.
struct TransmitterPart;

#[mod_api::horizon_part]
impl BlockPart for TransmitterPart {}

#[mod_api::horizon_part]
impl BlockEntityPart for TransmitterPart {
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        transmitter_tick(host, event);
    }
}

/// The receiver's block entity: it registers itself as a destination on its first tick
/// (`MatterReceiverTileEntity.tickServer`), which for this port is its readiness the harness reads.
struct ReceiverPart;

#[mod_api::horizon_part]
impl BlockPart for ReceiverPart {}

#[mod_api::horizon_part]
impl BlockEntityPart for ReceiverPart {
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    fn block_entity_tick(&self, _host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        // The receiver's state is its saved compound (read by the readout and the transmitter's
        // destination check); no per-tick payload is written, so it never dirties the chunk.
        *event.reply = BlockEntityTickReply::Unchanged;
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
    /// `Capabilities.Fluid.BLOCK`), and the crafter's block entity, its container and its
    /// energy buffer (`Capabilities.ItemHandler.BLOCK`, `Capabilities.EnergyStorage.BLOCK`).
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

        // The three crafter tiers share one block entity, one container and one energy buffer;
        // each has its own block, item, block entity type and `supportedRecipes` count
        // (`CrafterBaseTE.createTier1/2/3`).
        let mut crafter_items: [Option<ItemKey>; 3] = [None; 3];
        for (slot, tier) in crafter_items.iter_mut().zip(CRAFTER_TIERS) {
            let crafter = r.block_key(tier.block());
            let crafter_item = r.item_key(tier.block());
            *slot = Some(crafter_item);
            let crafter_kind = r
                .block_entity_type(tier.kind())
                .blocks(&[tier.block()])
                .build();
            if let Err(error) = r.block_entity_part(&crafter_kind, CrafterPart) {
                cx.refuse(&format!(
                    "rftoolsutility: the {} part was refused: {error}",
                    tier.kind()
                ));
                return;
            }
            r.block_container(&crafter, crafter_container());
            r.block_energy(&crafter, crafter_energy());
        }
        if let [Some(tier1), Some(tier2), Some(tier3)] = crafter_items {
            let _ = CRAFTER_ITEMS.set([tier1, tier2, tier3]);
        }

        let spawner = r.block_key(SPAWNER);
        let spawner_item = r.item_key(SPAWNER);
        let _ = SPAWNER_ITEM.set(spawner_item);
        let spawner_kind = r.block_entity_type(SPAWNER_TYPE).blocks(&[SPAWNER]).build();
        if let Err(error) = r.block_entity_part(&spawner_kind, SpawnerPart) {
            cx.refuse(&format!(
                "rftoolsutility: the spawner part was refused: {error}"
            ));
            return;
        }
        r.block_energy(&spawner, spawner_energy());
        // The 70 `rftoolsutility:spawner` mob recipes are this kind's data
        // (`SpawnerModule.SPAWNER_SERIALIZER`); without it the files are not in force and the
        // spawner would have no mob to spawn.
        r.register_crafting_recipe_serializer(SPAWNER_KIND, SpawnerRecipeKind);

        let beamer = r.block_key(BEAMER);
        let beamer_item = r.item_key(BEAMER);
        let _ = BEAMER_ITEM.set(beamer_item);
        let beamer_kind = r.block_entity_type(BEAMER_TYPE).blocks(&[BEAMER]).build();
        if let Err(error) = r.block_entity_part(&beamer_kind, BeamerPart) {
            cx.refuse(&format!(
                "rftoolsutility: the matter beamer part was refused: {error}"
            ));
            return;
        }
        r.block_container(&beamer, beamer_container());
        r.block_energy(&beamer, beamer_energy());

        // Slice 9: the teleporter family. The transmitter and receiver each carry a block entity
        // and their own energy domain; the booster is a plain adjacent marker (no code).
        let transmitter = r.block_key(TRANSMITTER);
        let transmitter_kind = r
            .block_entity_type(TRANSMITTER_TYPE)
            .blocks(&[TRANSMITTER])
            .build();
        if let Err(error) = r.block_entity_part(&transmitter_kind, TransmitterPart) {
            cx.refuse(&format!(
                "rftoolsutility: the matter transmitter part was refused: {error}"
            ));
            return;
        }
        r.block_energy(&transmitter, transmitter_energy());

        let receiver = r.block_key(RECEIVER);
        let receiver_kind = r
            .block_entity_type(RECEIVER_TYPE)
            .blocks(&[RECEIVER])
            .build();
        if let Err(error) = r.block_entity_part(&receiver_kind, ReceiverPart) {
            cx.refuse(&format!(
                "rftoolsutility: the matter receiver part was refused: {error}"
            ));
            return;
        }
        r.block_energy(&receiver, receiver_energy());
        let _ = TRANSMITTER_ITEM.set(r.item_key(TRANSMITTER));
        let _ = RECEIVER_ITEM.set(r.item_key(RECEIVER));
        r.block_key(BOOSTER);

        // Slice 11: the sensor's block entity and its redstone Part. The 10-tick check reads the
        // block on the input side and writes the `signal` state the two redstone methods report.
        let _ = SENSOR_ITEM.set(r.item_key(SENSOR));
        let sensor_kind = r.block_entity_type(SENSOR_TYPE).blocks(&[SENSOR]).build();
        if let Err(error) = r.block_entity_part(&sensor_kind, SensorPart) {
            cx.refuse(&format!(
                "rftoolsutility: the sensor part was refused: {error}"
            ));
        }

        // Slice 12: the sequencer's block entity, pattern state machine and redstone Part, on the
        // same iron-block template and `signal` state as its sibling the sensor.
        let _ = SEQUENCER_ITEM.set(r.item_key(SEQUENCER));
        let sequencer_kind = r
            .block_entity_type(SEQUENCER_TYPE)
            .blocks(&[SEQUENCER])
            .build();
        if let Err(error) = r.block_entity_part(&sequencer_kind, SequencerPart) {
            cx.refuse(&format!(
                "rftoolsutility: the sequencer part was refused: {error}"
            ));
        }

        // Slice 13: the timer's block entity, countdown state machine and redstone Part, on the
        // same iron-block template and `signal` state as the sensor and the sequencer.
        let _ = TIMER_ITEM.set(r.item_key(TIMER));
        let timer_kind = r.block_entity_type(TIMER_TYPE).blocks(&[TIMER]).build();
        if let Err(error) = r.block_entity_part(&timer_kind, TimerPart) {
            cx.refuse(&format!(
                "rftoolsutility: the timer part was refused: {error}"
            ));
        }

        // Slice 14: the counter's block entity, rising-edge count and redstone Part, on the same
        // iron-block template and `signal` state as its three logic siblings.
        let _ = COUNTER_ITEM.set(r.item_key(COUNTER));
        let counter_kind = r.block_entity_type(COUNTER_TYPE).blocks(&[COUNTER]).build();
        if let Err(error) = r.block_entity_part(&counter_kind, CounterPart) {
            cx.refuse(&format!(
                "rftoolsutility: the counter part was refused: {error}"
            ));
        }

        // Slice 15: the analog's block entity and redstone Part, on the same iron-block template
        // as its logic siblings, but with a 16-value `signal` state for its exact 0..15 output.
        let _ = ANALOG_ITEM.set(r.item_key(ANALOG));
        let analog_kind = r.block_entity_type(ANALOG_TYPE).blocks(&[ANALOG]).build();
        if let Err(error) = r.block_entity_part(&analog_kind, AnalogPart) {
            cx.refuse(&format!(
                "rftoolsutility: the analog part was refused: {error}"
            ));
        }
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
    /// placement applies). An empty tank's item carries nothing. A broken crafter's item carries
    /// its own saved state the same way (`CrafterBaseTE.collectImplicitComponents` moves the
    /// inventory, the energy and the taught recipes onto the item).
    fn block_drops(&self, level: &dyn LevelHost, event: &mut BlockDropsEvent<'_>) -> Flow {
        let block = level.state_block(event.state);
        if let Some(tier) = block.and_then(CrafterTier::from_block) {
            if let Some(item) = CRAFTER_ITEMS
                .get()
                .and_then(|items| items.get(tier.index()))
            {
                carry_machine(level, event, *item, tier.type_id());
            }
            return Flow::Continue;
        }
        if block == Some(SPAWNER) {
            if let Some(item) = SPAWNER_ITEM.get() {
                carry_machine(level, event, *item, SPAWNER_TYPE_ID);
            }
            return Flow::Continue;
        }
        if block == Some(BEAMER) {
            if let Some(item) = BEAMER_ITEM.get() {
                carry_machine(level, event, *item, BEAMER_TYPE_ID);
            }
            return Flow::Continue;
        }
        if block == Some(TRANSMITTER) {
            if let Some(item) = TRANSMITTER_ITEM.get() {
                carry_machine(level, event, *item, TRANSMITTER_TYPE_ID);
            }
            return Flow::Continue;
        }
        if block == Some(RECEIVER) {
            if let Some(item) = RECEIVER_ITEM.get() {
                carry_machine(level, event, *item, RECEIVER_TYPE_ID);
            }
            return Flow::Continue;
        }
        if block == Some(SENSOR) {
            if let Some(item) = SENSOR_ITEM.get() {
                carry_machine(level, event, *item, SENSOR_TYPE_ID);
            }
            return Flow::Continue;
        }
        if block == Some(SEQUENCER) {
            if let Some(item) = SEQUENCER_ITEM.get() {
                carry_machine(level, event, *item, SEQUENCER_TYPE_ID);
            }
            return Flow::Continue;
        }
        if block == Some(TIMER) {
            if let Some(item) = TIMER_ITEM.get() {
                carry_machine(level, event, *item, TIMER_TYPE_ID);
            }
            return Flow::Continue;
        }
        if block == Some(COUNTER) {
            if let Some(item) = COUNTER_ITEM.get() {
                carry_machine(level, event, *item, COUNTER_TYPE_ID);
            }
            return Flow::Continue;
        }
        if block == Some(ANALOG) {
            if let Some(item) = ANALOG_ITEM.get() {
                carry_machine(level, event, *item, ANALOG_TYPE_ID);
            }
            return Flow::Continue;
        }
        if block != Some(TANK) {
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
        let patch = entity_patch(&saved, TANK_TYPE_ID);
        for drop in event.drops.iter_mut() {
            if drop.stack.item() == *item {
                let _ = drop.stack.apply_components(&patch);
            }
        }
        Flow::Continue
    }

    /// `/rftoolsutility info|tank` and the crafter's controls (upstream has no commands: its
    /// controls are the GUI and the filter slot, and this slice has no GUI, so the harness needs
    /// a readout and a way to teach a recipe and stock a machine — the `/rftoolsbase energy`
    /// precedent).
    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        event.commands.push(Command::new(
            "rftoolsutility info <pos:block_pos> [nonce:word]",
        ));
        event.commands.push(Command::op(
            "rftoolsutility tank <pos:block_pos> <fluid:id> <mb:integer>",
            2,
        ));
        event.commands.push(Command::new(
            "rftoolsutility crafter <pos:block_pos> [nonce:word]",
        ));
        event.commands.push(Command::op(
            "rftoolsutility recipe <pos:block_pos> <index:integer> <recipe:id>",
            2,
        ));
        event.commands.push(Command::op(
            "rftoolsutility buffer <pos:block_pos> <slot:integer> <item:id> <count:integer>",
            2,
        ));
        event.commands.push(Command::op(
            "rftoolsutility energy <pos:block_pos> <amount:integer>",
            2,
        ));
        event.commands.push(Command::op(
            "rftoolsutility setting <pos:block_pos> <key:word> <value:word>",
            2,
        ));
        event.commands.push(Command::new(
            "rftoolsutility spawner <pos:block_pos> [nonce:word]",
        ));
        event.commands.push(Command::op(
            "rftoolsutility mob <pos:block_pos> <mob:id>",
            2,
        ));
        event.commands.push(Command::op(
            "rftoolsutility matter <pos:block_pos> <m0:number> <m1:number> <m2:number>",
            2,
        ));
        event.commands.push(Command::new(
            "rftoolsutility beamer <pos:block_pos> [nonce:word]",
        ));
        event.commands.push(Command::op(
            "rftoolsutility link <pos:block_pos> <dest:block_pos>",
            2,
        ));
        // Slice 9: the teleporter family's harness controls (upstream's dialing device and GUIs).
        event.commands.push(Command::op(
            "rftoolsutility dial <pos:block_pos> <dim:id> <x:integer> <y:integer> <z:integer>",
            2,
        ));
        event.commands.push(Command::new(
            "rftoolsutility transmitter <pos:block_pos> [nonce:word]",
        ));
        event.commands.push(Command::new(
            "rftoolsutility receiver <pos:block_pos> [nonce:word]",
        ));
        // Slice 10: the passive destination analyzer's readout (upstream's checks are GUI/beam).
        event.commands.push(Command::new(
            "rftoolsutility analyzer <pos:block_pos> [nonce:word]",
        ));
        // Slice 11: the sensor's GUI is replaced by two harness controls — a readout of the
        // strength it hands the redstone hook, and the setting the GUI would write.
        event.commands.push(Command::new(
            "rftoolsutility sensor <pos:block_pos> [nonce:word]",
        ));
        event.commands.push(Command::op(
            "rftoolsutility sensor-set <pos:block_pos> <kind:word> <target:id> <area:integer> <group:word> <facing:word>",
            2,
        ));
        // Slice 12: the sequencer's GUI-equivalent controls — a readout of its parsed state and a
        // way to write the 64-bit pattern and mode the GUI would teach.
        event.commands.push(Command::new(
            "rftoolsutility sequencer <pos:block_pos> [nonce:word]",
        ));
        event.commands.push(Command::op(
            "rftoolsutility sequencer-set <pos:block_pos> <mode:word> <bits:word> <delay:integer> <count:integer> <endstate:word> <facing:word>",
            2,
        ));
        // Slice 13: the timer's GUI equivalent — a readout of its parsed state and a way to write
        // the delay/pauses settings the jar's integer field and toggle button write.
        event.commands.push(Command::new(
            "rftoolsutility timer <pos:block_pos> [nonce:word]",
        ));
        event.commands.push(Command::op(
            "rftoolsutility timer-set <pos:block_pos> <delay:integer> <pauses:word> <facing:word>",
            2,
        ));
        // Slice 14: the counter's GUI equivalent — a readout of its parsed count/limit/latched
        // output and a way to write the limit (and reset the count) the GUI would set.
        event.commands.push(Command::new(
            "rftoolsutility counter <pos:block_pos> [nonce:word]",
        ));
        event.commands.push(Command::op(
            "rftoolsutility counter-set <pos:block_pos> <limit:integer> <facing:word>",
            2,
        ));
        // Slice 15: the analog's GUI-equivalent controls — a readout of its three inputs and exact
        // output, and a way to write the six arithmetic fields `GuiAnalog` would set.
        event.commands.push(Command::new(
            "rftoolsutility analog <pos:block_pos> [nonce:word]",
        ));
        event.commands.push(Command::op(
            "rftoolsutility analog-set <pos:block_pos> <mul_eq:number> <mul_less:number> <mul_greater:number> <add_eq:integer> <add_less:integer> <add_greater:integer> <facing:word>",
            2,
        ));
        event
            .commands
            .push(Command::new("rftoolsutility recipes [nonce:word]"));
    }

    /// Run the `/rftoolsutility` commands a player types.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        let invocation = event.command;
        let literals: Vec<&str> = invocation.literals.iter().map(String::as_str).collect();
        let known = matches!(
            literals.as_slice(),
            [
                "rftoolsutility",
                "info"
                    | "tank"
                    | "crafter"
                    | "recipe"
                    | "buffer"
                    | "energy"
                    | "setting"
                    | "spawner"
                    | "mob"
                    | "matter"
                    | "beamer"
                    | "link"
                    | "dial"
                    | "transmitter"
                    | "receiver"
                    | "analyzer"
                    | "sensor"
                    | "sensor-set"
                    | "sequencer"
                    | "sequencer-set"
                    | "timer"
                    | "timer-set"
                    | "counter"
                    | "counter-set"
                    | "analog"
                    | "analog-set"
                    | "recipes"
            ]
        );
        if !known {
            return Flow::Continue;
        }
        // `recipes` names no machine: it counts the mob recipes the game loaded, so the
        // harness can prove the carried family is in force (upstream has no such command; its
        // GUI lists them).
        if literals.as_slice() == ["rftoolsutility", "recipes"] {
            let nonce = match invocation.argument("nonce") {
                Some(CommandArgumentValue::String(word)) => Some(word.as_str()),
                _ => None,
            };
            let recipes = spawner_recipes(&*host);
            let cost = |mob: &str| {
                recipe_for(&recipes, mob)
                    .map_or_else(|| "none".to_owned(), |recipe| recipe.power.to_string())
            };
            let suffix = nonce.map_or(String::new(), |nonce| format!(" nonce={nonce}"));
            feedback(
                host,
                &format!(
                    "/rftoolsutility: {} spawner recipes, zombie={} blaze={}{suffix}",
                    recipes.len(),
                    cost("minecraft:zombie"),
                    cost("minecraft:blaze")
                ),
            );
            return Flow::Cancel;
        }
        let Some(CommandArgumentValue::BlockPos(pos)) = invocation.argument("pos") else {
            feedback(
                host,
                "Usage: /rftoolsutility info|tank|crafter|recipe|buffer|energy|setting|spawner|mob|matter <pos> …",
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
            feedback(host, "No machine there.");
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
            ["rftoolsutility", "crafter"] => {
                let Some((entity, _tier)) = crafter_at(host, level, pos) else {
                    feedback(host, "No crafter there.");
                    return Flow::Cancel;
                };
                let Some(data) = entity.data(&*host) else {
                    feedback(host, "No crafter there.");
                    return Flow::Cancel;
                };
                let nonce = match invocation.argument("nonce") {
                    Some(CommandArgumentValue::String(word)) => Some(word.as_str()),
                    _ => None,
                };
                let items = crafter_slots(host, entity);
                let line = crafter_readout(&*host, pos, &items, &data, nonce);
                feedback(host, &line);
            }
            ["rftoolsutility", "recipe"] => {
                let Some(CommandArgumentValue::Integer(index)) = invocation.argument("index")
                else {
                    feedback(host, "Usage: /rftoolsutility recipe <pos> <index> <recipe>");
                    return Flow::Cancel;
                };
                let id = match invocation.argument("recipe") {
                    Some(CommandArgumentValue::String(id)) => id.clone(),
                    Some(CommandArgumentValue::Id(id)) => id.to_string(),
                    _ => {
                        feedback(host, "Usage: /rftoolsutility recipe <pos> <index> <recipe>");
                        return Flow::Cancel;
                    }
                };
                let line = teach(host, level, pos, *index, &id);
                feedback(host, &line);
            }
            ["rftoolsutility", "buffer"] => {
                let (
                    Some(CommandArgumentValue::Integer(slot)),
                    Some(CommandArgumentValue::Integer(count)),
                ) = (invocation.argument("slot"), invocation.argument("count"))
                else {
                    feedback(
                        host,
                        "Usage: /rftoolsutility buffer <pos> <slot> <item> <count>",
                    );
                    return Flow::Cancel;
                };
                let item = match invocation.argument("item") {
                    Some(CommandArgumentValue::String(item)) => item.clone(),
                    Some(CommandArgumentValue::Id(item)) => item.to_string(),
                    _ => {
                        feedback(
                            host,
                            "Usage: /rftoolsutility buffer <pos> <slot> <item> <count>",
                        );
                        return Flow::Cancel;
                    }
                };
                let line = set_slot(host, level, pos, *slot, &item, *count);
                feedback(host, &line);
            }
            ["rftoolsutility", "energy"] => {
                let Some(CommandArgumentValue::Integer(amount)) = invocation.argument("amount")
                else {
                    feedback(host, "Usage: /rftoolsutility energy <pos> <amount>");
                    return Flow::Cancel;
                };
                let line = set_energy(host, level, pos, *amount);
                feedback(host, &line);
            }
            ["rftoolsutility", "setting"] => {
                let (
                    Some(CommandArgumentValue::String(key)),
                    Some(CommandArgumentValue::String(value)),
                ) = (invocation.argument("key"), invocation.argument("value"))
                else {
                    feedback(
                        host,
                        "Usage: /rftoolsutility setting <pos> speed|mode|keep <value>",
                    );
                    return Flow::Cancel;
                };
                let line = set_setting(host, level, pos, key, value);
                feedback(host, &line);
            }
            ["rftoolsutility", "spawner"] => {
                let Some(entity) = machine_at(&*host, level, pos, SPAWNER) else {
                    feedback(host, "No spawner there.");
                    return Flow::Cancel;
                };
                let Some(data) = entity.data(&*host) else {
                    feedback(host, "No spawner there.");
                    return Flow::Cancel;
                };
                let nonce = match invocation.argument("nonce") {
                    Some(CommandArgumentValue::String(word)) => Some(word.as_str()),
                    _ => None,
                };
                let line = spawner_readout(pos, &data, nonce);
                feedback(host, &line);
            }
            ["rftoolsutility", "mob"] => {
                // `<mob:id>` is a free identifier, which the engine delivers as a `String`
                // (the same finding the tank command recorded); accept either form.
                let mob = match invocation.argument("mob") {
                    Some(CommandArgumentValue::String(mob)) => mob.clone(),
                    Some(CommandArgumentValue::Id(mob)) => mob.to_string(),
                    _ => {
                        feedback(host, "Usage: /rftoolsutility mob <pos> <mob>");
                        return Flow::Cancel;
                    }
                };
                let line = set_mob(host, level, pos, &mob);
                feedback(host, &line);
            }
            ["rftoolsutility", "matter"] => {
                // `<m0:number>` arrives as a double (the SDK's `Number`); the machine keeps
                // floats, the same width the recipe's own amounts use.
                let (
                    Some(CommandArgumentValue::Number(m0)),
                    Some(CommandArgumentValue::Number(m1)),
                    Some(CommandArgumentValue::Number(m2)),
                ) = (
                    invocation.argument("m0"),
                    invocation.argument("m1"),
                    invocation.argument("m2"),
                )
                else {
                    feedback(host, "Usage: /rftoolsutility matter <pos> <m0> <m1> <m2>");
                    return Flow::Cancel;
                };
                let line = feed_matter(host, level, pos, [*m0 as f32, *m1 as f32, *m2 as f32]);
                feedback(host, &line);
            }
            ["rftoolsutility", "beamer"] => {
                let Some(entity) = machine_at(&*host, level, pos, BEAMER) else {
                    feedback(host, "No matter beamer there.");
                    return Flow::Cancel;
                };
                let Some(data) = entity.data(&*host) else {
                    feedback(host, "No matter beamer there.");
                    return Flow::Cancel;
                };
                let nonce = match invocation.argument("nonce") {
                    Some(CommandArgumentValue::String(word)) => Some(word.as_str()),
                    _ => None,
                };
                let slot = entity
                    .item_slot(&*host, BEAMER_SLOT as u32)
                    .ok()
                    .flatten()
                    .map(|stack| (stack.item().id(&*host), stack.count() as i32));
                let line = beamer_readout(
                    pos,
                    &data,
                    slot.as_ref().map(|(item, count)| (item.as_str(), *count)),
                    lit_of(&*host, level, pos),
                    nonce,
                );
                feedback(host, &line);
            }
            ["rftoolsutility", "link"] => {
                let Some(CommandArgumentValue::BlockPos(dest)) = invocation.argument("dest") else {
                    feedback(host, "Usage: /rftoolsutility link <pos> <dest>");
                    return Flow::Cancel;
                };
                let line = link_beamer(host, level, pos, *dest);
                feedback(host, &line);
            }
            ["rftoolsutility", "dial"] => {
                let dim = match invocation.argument("dim") {
                    Some(CommandArgumentValue::String(dim)) => dim.clone(),
                    Some(CommandArgumentValue::Id(dim)) => dim.to_string(),
                    _ => {
                        feedback(host, "Usage: /rftoolsutility dial <pos> <dim> <x> <y> <z>");
                        return Flow::Cancel;
                    }
                };
                let (
                    Some(CommandArgumentValue::Integer(x)),
                    Some(CommandArgumentValue::Integer(y)),
                    Some(CommandArgumentValue::Integer(z)),
                ) = (
                    invocation.argument("x"),
                    invocation.argument("y"),
                    invocation.argument("z"),
                )
                else {
                    feedback(host, "Usage: /rftoolsutility dial <pos> <dim> <x> <y> <z>");
                    return Flow::Cancel;
                };
                let line = dial_transmitter(host, level, pos, &dim, [*x, *y, *z]);
                feedback(host, &line);
            }
            ["rftoolsutility", "transmitter"] => {
                let Some(entity) = transmitter_at(&*host, level, pos) else {
                    feedback(host, "No matter transmitter there.");
                    return Flow::Cancel;
                };
                let Some(data) = entity.data(&*host) else {
                    feedback(host, "No matter transmitter there.");
                    return Flow::Cancel;
                };
                let nonce = match invocation.argument("nonce") {
                    Some(CommandArgumentValue::String(word)) => Some(word.as_str()),
                    _ => None,
                };
                feedback(host, &transmitter_readout(pos, &data, nonce));
            }
            ["rftoolsutility", "receiver"] => {
                let Some(entity) = machine_at(&*host, level, pos, RECEIVER) else {
                    feedback(host, "No matter receiver there.");
                    return Flow::Cancel;
                };
                let Some(data) = entity.data(&*host) else {
                    feedback(host, "No matter receiver there.");
                    return Flow::Cancel;
                };
                let nonce = match invocation.argument("nonce") {
                    Some(CommandArgumentValue::String(word)) => Some(word.as_str()),
                    _ => None,
                };
                let ready = receiver_status(&*host, level, pos);
                feedback(host, &receiver_readout(pos, &data, ready, nonce));
            }
            ["rftoolsutility", "analyzer"] => {
                let nonce = match invocation.argument("nonce") {
                    Some(CommandArgumentValue::String(word)) => Some(word.as_str()),
                    _ => None,
                };
                let line = analyzer_readout(&*host, level, pos, nonce);
                feedback(host, &line);
            }
            ["rftoolsutility", "sensor"] => {
                let Some(entity) = machine_at(&*host, level, pos, SENSOR) else {
                    feedback(host, "No sensor there.");
                    return Flow::Cancel;
                };
                let Some(data) = entity.data(&*host) else {
                    feedback(host, "No sensor there.");
                    return Flow::Cancel;
                };
                let config = SensorConfig::from_data(&data);
                let nonce = match invocation.argument("nonce") {
                    Some(CommandArgumentValue::String(word)) => Some(word.as_str()),
                    _ => None,
                };
                let signal = cached_signal(&*host, level, pos);
                feedback(
                    host,
                    &sensor_line(
                        pos,
                        signal,
                        config.kind.as_str(),
                        config.target.as_deref().unwrap_or("none"),
                        &config.facing,
                        nonce,
                    ),
                );
            }
            ["rftoolsutility", "sensor-set"] => {
                let Some(entity) = machine_at(&*host, level, pos, SENSOR) else {
                    feedback(host, "No sensor there.");
                    return Flow::Cancel;
                };
                let kind = match invocation.argument("kind") {
                    Some(CommandArgumentValue::String(name)) => SensorKind::from_name(name),
                    _ => None,
                };
                let Some(kind) = kind else {
                    feedback(host, "unknown sensor type (blocks|any).");
                    return Flow::Cancel;
                };
                let target = match invocation.argument("target") {
                    Some(CommandArgumentValue::Id(id)) => id.to_string(),
                    Some(CommandArgumentValue::String(id)) => id.clone(),
                    _ => {
                        feedback(
                            host,
                            "Usage: /rftoolsutility sensor-set <pos> <kind> <target> <area> <group> <facing>",
                        );
                        return Flow::Cancel;
                    }
                };
                let target = (target != "none").then_some(target);
                if !matches!(
                    invocation.argument("area"),
                    Some(CommandArgumentValue::Integer(1))
                ) {
                    feedback(host, "this sensor has area 1.");
                    return Flow::Cancel;
                }
                if !matches!(
                    invocation.argument("group"),
                    Some(CommandArgumentValue::String(group)) if group == "one"
                ) {
                    feedback(host, "this sensor has group one.");
                    return Flow::Cancel;
                }
                let Some(CommandArgumentValue::String(facing)) = invocation.argument("facing")
                else {
                    feedback(
                        host,
                        "Usage: /rftoolsutility sensor-set <pos> <kind> <target> <area> <group> <facing>",
                    );
                    return Flow::Cancel;
                };
                if sensor_front(pos, facing).is_none() {
                    feedback(host, "unknown sensor facing (north|south|east|west).");
                    return Flow::Cancel;
                }
                let config = SensorConfig {
                    kind,
                    target,
                    facing: facing.clone(),
                };
                let Some(mut data) = entity.data(&*host) else {
                    feedback(host, "No sensor there.");
                    return Flow::Cancel;
                };
                config.write(&mut data);
                if entity.replace_data(host, data).is_err() {
                    feedback(host, "No sensor there.");
                    return Flow::Cancel;
                }
                feedback(
                    host,
                    &format!(
                        "Sensor at {pos:?} checks {} to the {facing} (area 1, group one).",
                        config.target.as_deref().unwrap_or("any occluding block")
                    ),
                );
            }
            ["rftoolsutility", "sequencer"] => {
                let Some(entity) = machine_at(&*host, level, pos, SEQUENCER) else {
                    feedback(host, "No sequencer there.");
                    return Flow::Cancel;
                };
                let Some(nbt) = entity.data(&*host) else {
                    feedback(host, "No sequencer there.");
                    return Flow::Cancel;
                };
                let data = sequencer_data(&nbt);
                let state = sequencer_state(&nbt, &data);
                let nonce = match invocation.argument("nonce") {
                    Some(CommandArgumentValue::String(word)) => Some(word.as_str()),
                    _ => None,
                };
                let facing = read_text(&nbt, KEY_SEQ_FACING).unwrap_or_else(|| "north".to_owned());
                feedback(host, &sequencer_line(pos, &data, &state, &facing, nonce));
            }
            ["rftoolsutility", "sequencer-set"] => {
                let Some(entity) = machine_at(&*host, level, pos, SEQUENCER) else {
                    feedback(host, "No sequencer there.");
                    return Flow::Cancel;
                };
                let usage = "Usage: /rftoolsutility sequencer-set <pos> <mode> <bits> <delay> <count> <endstate> <facing>";
                let Some(CommandArgumentValue::String(mode)) = invocation.argument("mode") else {
                    feedback(host, usage);
                    return Flow::Cancel;
                };
                let Some(mode) = SequencerMode::from_name(mode) else {
                    feedback(
                        host,
                        "unknown sequencer mode (Once1|Once2|Loop1|Loop2|Loop3|Step|Loop4).",
                    );
                    return Flow::Cancel;
                };
                let Some(CommandArgumentValue::String(bits)) = invocation.argument("bits") else {
                    feedback(host, usage);
                    return Flow::Cancel;
                };
                let Some(bits) = parse_bits(bits) else {
                    feedback(
                        host,
                        "the pattern must be a 64-bit decimal or 0x hex number.",
                    );
                    return Flow::Cancel;
                };
                let iarg = |name: &str| match invocation.argument(name) {
                    Some(CommandArgumentValue::Integer(value)) => Some(*value),
                    _ => None,
                };
                let (Some(delay), Some(count)) = (iarg("delay"), iarg("count")) else {
                    feedback(host, usage);
                    return Flow::Cancel;
                };
                if !(1..=64).contains(&delay) || !(1..=64).contains(&count) {
                    feedback(host, "delay and count must be between 1 and 64.");
                    return Flow::Cancel;
                }
                let Some(CommandArgumentValue::String(endstate)) = invocation.argument("endstate")
                else {
                    feedback(host, usage);
                    return Flow::Cancel;
                };
                let endstate = match endstate.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => {
                        feedback(host, "endstate must be true or false.");
                        return Flow::Cancel;
                    }
                };
                let Some(CommandArgumentValue::String(facing)) = invocation.argument("facing")
                else {
                    feedback(host, usage);
                    return Flow::Cancel;
                };
                if !sequencer_front(facing) {
                    feedback(host, "unknown sequencer facing (north|south|east|west).");
                    return Flow::Cancel;
                }
                let data = SequencerData {
                    bits,
                    mode,
                    delay,
                    stepcount: count,
                    endstate,
                };
                let state = SequencerState::for_mode(&data);
                let Some(mut nbt) = entity.data(&*host) else {
                    feedback(host, "No sequencer there.");
                    return Flow::Cancel;
                };
                write_sequencer(&mut nbt, &data, &state);
                write_text(&mut nbt, KEY_SEQ_FACING, facing);
                if entity.replace_data(host, nbt).is_err() {
                    feedback(host, "No sequencer there.");
                    return Flow::Cancel;
                }
                feedback(
                    host,
                    &format!(
                        "Sequencer at {pos:?}: mode {} pattern {bits} over {count} steps, output {}.",
                        mode.as_str(),
                        state.output as i32 * SENSOR_ON
                    ),
                );
            }
            ["rftoolsutility", "timer"] => {
                let Some(entity) = machine_at(&*host, level, pos, TIMER) else {
                    feedback(host, "No timer there.");
                    return Flow::Cancel;
                };
                let Some(nbt) = entity.data(&*host) else {
                    feedback(host, "No timer there.");
                    return Flow::Cancel;
                };
                let data = timer_data(&nbt);
                let state = timer_state(&nbt);
                let pulse = read_int_or(&nbt, KEY_TIMER_PULSE_AT, -1);
                let fired = read_int_or(&nbt, KEY_TIMER_FIRED_AT, -1);
                let facing =
                    read_text(&nbt, KEY_TIMER_FACING).unwrap_or_else(|| "north".to_owned());
                let lamp = if read_flag(&nbt, KEY_TIMER_LAMP) {
                    "lit"
                } else {
                    "dark"
                };
                let time = level.game_time(&*host).unwrap_or(0) as i32;
                let nonce = match invocation.argument("nonce") {
                    Some(CommandArgumentValue::String(word)) => Some(word.as_str()),
                    _ => None,
                };
                feedback(
                    host,
                    &timer_line(pos, &data, &state, pulse, fired, &facing, time, lamp, nonce),
                );
            }
            ["rftoolsutility", "timer-set"] => {
                let Some(entity) = machine_at(&*host, level, pos, TIMER) else {
                    feedback(host, "No timer there.");
                    return Flow::Cancel;
                };
                let usage = "Usage: /rftoolsutility timer-set <pos> <delay> <pauses> <facing>";
                let delay = match invocation.argument("delay") {
                    Some(CommandArgumentValue::Integer(value)) => *value,
                    _ => {
                        feedback(host, usage);
                        return Flow::Cancel;
                    }
                };
                let pauses = match invocation.argument("pauses") {
                    Some(CommandArgumentValue::String(word)) if word == "true" => true,
                    Some(CommandArgumentValue::String(word)) if word == "false" => false,
                    _ => {
                        feedback(host, "pauses must be true or false.");
                        return Flow::Cancel;
                    }
                };
                let Some(CommandArgumentValue::String(facing)) = invocation.argument("facing")
                else {
                    feedback(host, usage);
                    return Flow::Cancel;
                };
                if !timer_front(facing) {
                    feedback(host, "unknown timer facing (north|south|east|west).");
                    return Flow::Cancel;
                }
                let data = TimerData {
                    delay,
                    redstone_pauses: pauses,
                };
                // The jar writes these through a GUI that also waits out the current countdown;
                // with no menu the harness starts a fresh one, so every phase begins from a known
                // state (recorded as a deviation in notes.md).
                let state = TimerState {
                    prev_in: false,
                    timer: data.delay,
                    output: false,
                };
                let Some(mut nbt) = entity.data(&*host) else {
                    feedback(host, "No timer there.");
                    return Flow::Cancel;
                };
                write_timer(&mut nbt, &data, &state);
                write_int(&mut nbt, KEY_TIMER_PULSE_AT, -1);
                write_int(&mut nbt, KEY_TIMER_FIRED_AT, -1);
                write_flag(&mut nbt, KEY_TIMER_FIRED_LOCK, false);
                write_flag(&mut nbt, KEY_TIMER_LAMP, false);
                write_text(&mut nbt, KEY_TIMER_FACING, facing);
                if entity.replace_data(host, nbt).is_err() {
                    feedback(host, "No timer there.");
                    return Flow::Cancel;
                }
                feedback(
                    host,
                    &format!(
                        "Timer at {pos:?}: delay {delay} ticks, pauses={pauses}, input to the {facing}."
                    ),
                );
            }
            ["rftoolsutility", "counter"] => {
                let Some(entity) = machine_at(&*host, level, pos, COUNTER) else {
                    feedback(host, "No counter there.");
                    return Flow::Cancel;
                };
                let Some(nbt) = entity.data(&*host) else {
                    feedback(host, "No counter there.");
                    return Flow::Cancel;
                };
                let data = counter_data(&nbt);
                let state = counter_state(&nbt);
                let pulses = read_int_or(&nbt, KEY_COUNTER_PULSES, 0);
                let pulse = read_int_or(&nbt, KEY_COUNTER_PULSE_AT, -1);
                let fire = read_int_or(&nbt, KEY_COUNTER_FIRE_AT, -1);
                let facing =
                    read_text(&nbt, KEY_COUNTER_FACING).unwrap_or_else(|| "north".to_owned());
                // A live read of the adjacent vanilla lamp, not the fire-instant stamp: the
                // counter's output is a level latch, so the lamp also goes dark on the *next*
                // pulse that clears it. Reading it here keeps `lamp=` honest in every phase.
                let lamp = if [
                    [pos[0], pos[1], pos[2] - 1],
                    [pos[0], pos[1], pos[2] + 1],
                    [pos[0] - 1, pos[1], pos[2]],
                    [pos[0] + 1, pos[1], pos[2]],
                ]
                .iter()
                .any(|&around| {
                    level
                        .block_state(&*host, around)
                        .and_then(|s| host.block_state_string(s))
                        .is_some_and(|name| {
                            name.contains("redstone_lamp")
                                && sensor_state_property(&name, "lit") == Some("true")
                        })
                }) {
                    "lit"
                } else {
                    "dark"
                };
                let time = level.game_time(&*host).unwrap_or(0) as i32;
                let nonce = match invocation.argument("nonce") {
                    Some(CommandArgumentValue::String(word)) => Some(word.as_str()),
                    _ => None,
                };
                feedback(
                    host,
                    &counter_line(
                        pos, &data, &state, pulses, pulse, fire, &facing, time, lamp, nonce,
                    ),
                );
            }
            ["rftoolsutility", "counter-set"] => {
                let Some(entity) = machine_at(&*host, level, pos, COUNTER) else {
                    feedback(host, "No counter there.");
                    return Flow::Cancel;
                };
                let usage = "Usage: /rftoolsutility counter-set <pos> <limit> <facing>";
                let limit = match invocation.argument("limit") {
                    Some(CommandArgumentValue::Integer(value)) if *value >= 1 => *value,
                    _ => {
                        feedback(host, "the limit must be a positive integer.");
                        return Flow::Cancel;
                    }
                };
                let Some(CommandArgumentValue::String(facing)) = invocation.argument("facing")
                else {
                    feedback(host, usage);
                    return Flow::Cancel;
                };
                if !counter_front(facing) {
                    feedback(host, "unknown counter facing (north|south|east|west).");
                    return Flow::Cancel;
                }
                // `setCounter` with a changed value resets the running count; with no menu the
                // harness also clears the latched output and its stamps so each phase starts clean
                // (recorded as a deviation in notes.md).
                let data = CounterData {
                    counter: limit,
                    current: 0,
                };
                let state = CounterState::default();
                let Some(mut nbt) = entity.data(&*host) else {
                    feedback(host, "No counter there.");
                    return Flow::Cancel;
                };
                write_counter(&mut nbt, &data, &state);
                write_int(&mut nbt, KEY_COUNTER_PULSE_AT, -1);
                write_int(&mut nbt, KEY_COUNTER_FIRE_AT, -1);
                write_flag(&mut nbt, KEY_COUNTER_FIRED_LOCK, false);
                write_flag(&mut nbt, KEY_COUNTER_LAMP, false);
                write_int(&mut nbt, KEY_COUNTER_PULSES, 0);
                write_text(&mut nbt, KEY_COUNTER_FACING, facing);
                if entity.replace_data(host, nbt).is_err() {
                    feedback(host, "No counter there.");
                    return Flow::Cancel;
                }
                feedback(
                    host,
                    &format!(
                        "Counter at {pos:?}: limit {limit}, input to the {facing}, count reset to 0."
                    ),
                );
            }
            ["rftoolsutility", "analog"] => {
                let Some(entity) = machine_at(&*host, level, pos, ANALOG) else {
                    feedback(host, "No analog there.");
                    return Flow::Cancel;
                };
                let Some(nbt) = entity.data(&*host) else {
                    feedback(host, "No analog there.");
                    return Flow::Cancel;
                };
                let data = analog_data(&nbt);
                let nonce = match invocation.argument("nonce") {
                    Some(CommandArgumentValue::String(word)) => Some(word.as_str()),
                    _ => None,
                };
                let input = read_int_or(&nbt, KEY_ANALOG_INPUT, 0);
                let a = read_int_or(&nbt, KEY_ANALOG_A, 0);
                let b = read_int_or(&nbt, KEY_ANALOG_B, 0);
                let output = analog_cached_signal(&*host, level, pos);
                let changed = read_int_or(&nbt, KEY_ANALOG_CHANGED_AT, -1);
                let facing =
                    read_text(&nbt, KEY_ANALOG_FACING).unwrap_or_else(|| "north".to_owned());
                let time = level.game_time(&*host).unwrap_or(0) as i32;
                // A live read of the adjacent lamp, as the counter's readout does: the fire stamp
                // alone could go stale, and a lamp's own `lit` state is the game-time observation.
                let lamp = [
                    [pos[0], pos[1], pos[2] - 1],
                    [pos[0], pos[1], pos[2] + 1],
                    [pos[0] - 1, pos[1], pos[2]],
                    [pos[0] + 1, pos[1], pos[2]],
                ]
                .iter()
                .any(|&around| {
                    level
                        .block_state(&*host, around)
                        .and_then(|state| host.block_state_string(state))
                        .is_some_and(|name| {
                            name.contains("redstone_lamp")
                                && sensor_state_property(&name, "lit") == Some("true")
                        })
                });
                feedback(
                    host,
                    &analog_line(
                        pos,
                        &data,
                        input,
                        a,
                        b,
                        output,
                        changed,
                        &facing,
                        time,
                        if lamp { "lit" } else { "dark" },
                        nonce,
                    ),
                );
            }
            ["rftoolsutility", "analog-set"] => {
                let Some(entity) = machine_at(&*host, level, pos, ANALOG) else {
                    feedback(host, "No analog there.");
                    return Flow::Cancel;
                };
                let usage = "Usage: /rftoolsutility analog-set <pos> <mul_eq> <mul_less> <mul_greater> <add_eq> <add_less> <add_greater> <facing>";
                let number = |name: &str| match invocation.argument(name) {
                    Some(CommandArgumentValue::Number(value)) => Some(*value as f32),
                    _ => None,
                };
                let (Some(mul_equal), Some(mul_less), Some(mul_greater)) =
                    (number("mul_eq"), number("mul_less"), number("mul_greater"))
                else {
                    feedback(host, usage);
                    return Flow::Cancel;
                };
                let integer = |name: &str| match invocation.argument(name) {
                    Some(CommandArgumentValue::Integer(value)) => Some(*value),
                    _ => None,
                };
                let (Some(add_equal), Some(add_less), Some(add_greater)) = (
                    integer("add_eq"),
                    integer("add_less"),
                    integer("add_greater"),
                ) else {
                    feedback(host, usage);
                    return Flow::Cancel;
                };
                let Some(CommandArgumentValue::String(facing)) = invocation.argument("facing")
                else {
                    feedback(host, usage);
                    return Flow::Cancel;
                };
                if analog_neighbours(pos, facing).is_none() {
                    feedback(host, "unknown analog facing (north|south|east|west).");
                    return Flow::Cancel;
                }
                let data = AnalogData {
                    mul_equal,
                    mul_less,
                    mul_greater,
                    add_equal,
                    add_less,
                    add_greater,
                };
                let Some(mut nbt) = entity.data(&*host) else {
                    feedback(host, "No analog there.");
                    return Flow::Cancel;
                };
                write_analog(&mut nbt, &data);
                write_text(&mut nbt, KEY_ANALOG_FACING, facing);
                if entity.replace_data(host, nbt).is_err() {
                    feedback(host, "No analog there.");
                    return Flow::Cancel;
                }
                feedback(
                    host,
                    &format!(
                        "Analog at {pos:?}: mul={mul_equal},{mul_less},{mul_greater} add={add_equal},{add_less},{add_greater}, input to the {facing}."
                    ),
                );
            }
            _ => return Flow::Continue,
        }
        Flow::Cancel
    }
}

// ---------------------------------------------------------------------------
// The crafter's harness controls
// ---------------------------------------------------------------------------

/// The machine of `kind` at `pos`, when the block there is that machine.
fn machine_at(
    host: &dyn Host,
    level: LevelRef,
    pos: [i32; 3],
    kind: &str,
) -> Option<BlockEntityRef> {
    let block = level.block(host, pos)?;
    (block.id(host) == kind)
        .then(|| host.block_entity(level, pos))
        .flatten()
}

/// The crafter at `pos` and its tier, when the block there is one of the three crafter tiers.
fn crafter_at(
    host: &dyn Host,
    level: LevelRef,
    pos: [i32; 3],
) -> Option<(BlockEntityRef, CrafterTier)> {
    let block = level.block(host, pos)?;
    let tier = CrafterTier::from_block(&block.id(host))?;
    host.block_entity(level, pos).map(|entity| (entity, tier))
}

/// The crafter's 41 slots, read one by one (`block_entity_item_slot`). A slot the entity cannot
/// answer reads as empty.
fn crafter_slots(host: &dyn ServerHost, entity: BlockEntityRef) -> Vec<Option<ItemStack>> {
    (0..CRAFTER_SLOTS as u32)
        .map(|slot| entity.item_slot(host, slot).ok().flatten())
        .collect()
}

/// The item ids a recipe's cell accepts, sorted so a saved grid loads back the same way; none
/// for an ingredient the engine cannot enumerate (a component predicate).
fn accepted(ingredient: &mod_api::Ingredient) -> Option<Vec<String>> {
    let mut ids: Vec<String> = if ingredient.items.is_empty() {
        ingredient.order.clone()
    } else {
        ingredient.items.iter().cloned().collect()
    };
    ids.sort_unstable();
    ids.dedup();
    (!ids.is_empty()).then_some(ids)
}

/// The nine cells of a crafting recipe's grid. A shaped recipe's pattern is mapped cell by cell
/// (its ingredient list is the pattern's cells row by row, empty cells left out); a shapeless
/// recipe's ingredients go in order. None for a special crafting recipe (no grid) or an
/// ingredient that cannot be enumerated.
fn recipe_cells(recipe: &mod_api::Recipe) -> Option<[Vec<String>; 9]> {
    let mut cells: [Vec<String>; 9] = std::array::from_fn(|_| Vec::new());
    let pattern = recipe.json.get("pattern").and_then(|p| p.as_array());
    match pattern {
        Some(rows) => {
            let mut ingredient = recipe.ingredients.iter();
            for (row, line) in rows.iter().enumerate().take(3) {
                let line = line.as_str()?;
                for (column, cell) in line.chars().enumerate().take(3) {
                    if cell == ' ' {
                        continue;
                    }
                    cells[row * 3 + column] = accepted(ingredient.next()?)?;
                }
            }
        }
        None => {
            for (cell, ingredient) in cells.iter_mut().zip(&recipe.ingredients) {
                *cell = accepted(ingredient)?;
            }
        }
    }
    cells.iter().any(|cell| !cell.is_empty()).then_some(cells)
}

/// The ghost grid a player's GUI would show for a taught recipe: the first item id of each
/// filled cell, none for a cell the pattern leaves empty (`CrafterBaseTE.setSelected` writes the
/// same 3x3 grid into the ghost slots).
fn ghost_grid(cells: &[Vec<String>; 9]) -> [Option<&str>; 9] {
    std::array::from_fn(|cell| cells[cell].first().map(String::as_str))
}

/// Whether the crafter `tier` has a recipe slot for `index`: a slot past its count is refused
/// with that count (`CrafterBaseTE.setRecipe`'s `sel >= supportedRecipes` bound), which is the
/// reply the harness asserts for a crafter2/3.
fn recipe_slot(tier: CrafterTier, index: usize) -> Result<(), String> {
    if index < tier.recipes() {
        return Ok(());
    }
    Err(format!(
        "/rftoolsutility: this crafter has {} recipe slots.",
        tier.recipes()
    ))
}

/// `/rftoolsutility recipe`: teach the crafter slot `index` the crafting recipe `id`, resolving
/// its grid from the engine's recipe table (`CrafterBaseTE`'s GUI writes the same ghost grid).
/// The ghost slots then show the grid, as a player's would.
fn teach(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
    index: i32,
    id: &str,
) -> String {
    let Some((entity, tier)) = crafter_at(host, level, pos) else {
        return "No crafter there.".to_owned();
    };
    let index = usize::try_from(index).unwrap_or(usize::MAX);
    if let Err(refusal) = recipe_slot(tier, index) {
        return refusal;
    }
    let recipe = host
        .recipes_of_type(RECIPE_TYPE)
        .iter()
        .find(|recipe| recipe.id == id)
        .cloned();
    let Some(recipe) = recipe else {
        return format!("/rftoolsutility: no crafting recipe {id}.");
    };
    let Some(cells) = recipe_cells(&recipe) else {
        return format!("/rftoolsutility: {id} has no 3x3 grid this crafter can hold.");
    };
    // What a player's ghost grid would hold: one of each cell's items, and the recipe's result.
    let mut grid: Vec<Option<ItemStack>> = vec![None; 9];
    for (cell, id) in ghost_grid(&cells).into_iter().enumerate() {
        let Some(id) = id else {
            continue;
        };
        let Some(key) = Identifier::parse(id).ok().and_then(|id| host.item_key(&id)) else {
            return format!("/rftoolsutility: the game does not know {id}.");
        };
        grid[cell] = Some(ItemStack::of(key));
    }
    // The grid must be exactly the recipe it names: a pattern mapped wrongly is refused here
    // rather than silently crafting something else.
    let Ok(kind) = Identifier::parse(RECIPE_TYPE) else {
        return "/rftoolsutility: the crafting recipe type is unparsable.".to_owned();
    };
    let input = RecipeInput::Crafting {
        width: 3,
        height: 3,
        slots: grid.clone(),
    };
    let matched = level.recipe_for(&*host, &kind, &input);
    match matched {
        Ok(Some(matched)) if matched.recipe.to_string() == id => {}
        _ => return format!("/rftoolsutility: {id}'s grid does not match in this world."),
    }
    let Some(mut data) = entity.data(&*host) else {
        return "No crafter there.".to_owned();
    };
    let mut taught = read_taught(&data);
    while taught.len() <= index {
        taught.push(Taught {
            id: String::new(),
            cells: std::array::from_fn(|_| Vec::new()),
        });
    }
    taught[index] = Taught {
        id: id.to_owned(),
        cells,
    };
    write_taught(&mut data, &taught);
    if entity.replace_data(host, data).is_err() {
        return "No crafter there.".to_owned();
    }
    // The ghost grid follows the first recipe, as the GUI's selected recipe does.
    if index == 0 {
        let result = recipe.results.first().cloned();
        let written = host.transaction(|host, tx| -> Result<bool, ()> {
            for (cell, stack) in grid.into_iter().enumerate() {
                if entity
                    .set_item_slot(host, tx, (GHOST_INPUT + cell) as u32, stack)
                    .is_err()
                {
                    return Ok(false);
                }
            }
            entity
                .set_item_slot(host, tx, GHOST_RESULT as u32, result)
                .map_err(|_| ())
        });
        if !matches!(written, Ok(true)) {
            return "No crafter there.".to_owned();
        }
    }
    format!("/rftoolsutility: slot {index} of the crafter at {pos:?} crafts {id}.")
}

/// `/rftoolsutility buffer`: put `count` of `item` in the crafter's buffer slot `slot` (0 clears
/// it), or in the matter beamer's one material slot. The harness's stand-in for a hopper; a slot
/// outside the machine's own range is refused.
fn set_slot(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
    slot: i32,
    item: &str,
    count: i32,
) -> String {
    let (entity, buffers, machine) = if let Some((entity, _tier)) = crafter_at(host, level, pos) {
        (entity, BUFFER_IN..BUFFER_OUT + BUFFER_OUT_SLOTS, "crafter")
    } else if let Some(entity) = machine_at(&*host, level, pos, BEAMER) {
        (entity, BEAMER_SLOT..BEAMER_SLOT + 1, "matter beamer")
    } else {
        return "No machine there.".to_owned();
    };
    let slot = usize::try_from(slot).unwrap_or(usize::MAX);
    if !buffers.contains(&slot) {
        return format!(
            "/rftoolsutility: slot {slot} is not one of the {machine}'s slots ({}..{}).",
            buffers.start, buffers.end
        );
    }
    let stack = if count <= 0 {
        None
    } else {
        let Some(key) = Identifier::parse(item)
            .ok()
            .and_then(|id| host.item_key(&id))
        else {
            return format!("/rftoolsutility: the game does not know {item}.");
        };
        let limit = host.max_stack_size(key).unwrap_or(64);
        let count = u32::try_from(count).unwrap_or(u32::MAX).min(limit);
        ItemStack::new(key, count)
    };
    let written = host.transaction(|host, tx| -> Result<bool, ()> {
        entity
            .set_item_slot(host, tx, slot as u32, stack)
            .map_err(|_| ())
    });
    match written {
        Ok(true) => format!(
            "/rftoolsutility: the {machine} at {pos:?} slot {slot} holds {}.",
            match count {
                0 => "nothing".to_owned(),
                count => format!("{item} x{count}"),
            }
        ),
        _ => "No machine there.".to_owned(),
    }
}

/// `/rftoolsutility energy`: set a machine's buffer, clamped to its own capacity. The crafter,
/// the spawner and the matter beamer all keep their FE in the engine's `energy` int, so one
/// command serves them.
fn set_energy(host: &mut dyn ServerHost, level: LevelRef, pos: [i32; 3], amount: i32) -> String {
    let (entity, capacity, name) = if let Some((entity, _tier)) = crafter_at(&*host, level, pos) {
        (entity, CRAFTER_CAPACITY, "crafter")
    } else if let Some(entity) = machine_at(&*host, level, pos, SPAWNER) {
        (entity, SPAWNER_CAPACITY, "spawner")
    } else if let Some(entity) = machine_at(&*host, level, pos, BEAMER) {
        (entity, BEAMER_CAPACITY, "matter beamer")
    } else if let Some(entity) = transmitter_at(&*host, level, pos) {
        (entity, TRANSMITTER_CAPACITY, "matter transmitter")
    } else if let Some(entity) = machine_at(&*host, level, pos, RECEIVER) {
        (entity, RECEIVER_CAPACITY, "matter receiver")
    } else {
        return "No machine there.".to_owned();
    };
    let Some(mut data) = entity.data(&*host) else {
        return "No machine there.".to_owned();
    };
    let amount = amount.clamp(0, capacity);
    write_int(&mut data, KEY_ENERGY, amount);
    if entity.replace_data(host, data).is_err() {
        return "No machine there.".to_owned();
    }
    format!("/rftoolsutility: the {name} at {pos:?} holds {amount}/{capacity} FE.")
}

/// `/rftoolsutility mob`: set the mob the spawner spawns (`SpawnerData.mob`), from the recipe
/// table. The recipe is the machine's own data, so the id it does not name is refused, and the
/// recipe's `power` and matter amounts are kept beside the mob (upstream's `MobData`, read once
/// per table load). A **new** mob clears the matter stores, as upstream's `testSyringe` does
/// ("if the previously seen mob differs, `clearMatter()`"), so a zombie's matter cannot pay for
/// a blaze.
fn set_mob(host: &mut dyn ServerHost, level: LevelRef, pos: [i32; 3], mob: &str) -> String {
    let Some(entity) = machine_at(&*host, level, pos, SPAWNER) else {
        return "No spawner there.".to_owned();
    };
    let recipes = spawner_recipes(&*host);
    let Some(recipe) = recipe_for(&recipes, mob) else {
        return format!(
            "/rftoolsutility: no spawner recipe for {mob} ({} recipes are loaded).",
            recipes.len()
        );
    };
    let Some(mut data) = entity.data(&*host) else {
        return "No spawner there.".to_owned();
    };
    let held = read_matter(&data, KEY_MATTER).unwrap_or([0.0; 3]);
    let matter =
        matter_after_mob_change(read_text(&data, KEY_MOB).as_deref(), &recipe.entity, held);
    write_text(&mut data, KEY_MOB, &recipe.entity);
    write_int(&mut data, KEY_COST, recipe.power);
    write_matter(&mut data, KEY_NEEDS, recipe.matter);
    write_matter(&mut data, KEY_MATTER, matter);
    if entity.replace_data(host, data).is_err() {
        return "No spawner there.".to_owned();
    }
    format!(
        "/rftoolsutility: the spawner at {pos:?} spawns {} for {} FE.",
        recipe.entity, recipe.power
    )
}

/// The matter a spawner keeps when its mob is set to `mob` (`SpawnerTileEntity.testSyringe`:
/// the stores survive a re-set of the same mob and are cleared by a different one).
fn matter_after_mob_change(previous: Option<&str>, mob: &str, held: [f32; 3]) -> [f32; 3] {
    if previous == Some(mob) {
        held
    } else {
        [0.0; 3]
    }
}

/// The stores after a beamer's send, one [`add_matter`] per type: the pure half of
/// `/rftoolsutility matter`.
fn matter_after_send(held: [f32; 3], amounts: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|index| add_matter(held[index], amounts[index]))
}

/// `/rftoolsutility matter`: what a matter beamer's successful send does
/// (`SpawnerTileEntity.addMatter`) — add each amount to its own store, capped at
/// `maxMatterStorage`. A negative amount drains a store, so the harness can empty one between
/// phases; a player never types this (the beamer is the mod's own path, and its block is a later
/// slice).
fn feed_matter(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
    amounts: [f32; 3],
) -> String {
    let Some(entity) = machine_at(&*host, level, pos, SPAWNER) else {
        return "No spawner there.".to_owned();
    };
    let Some(mut data) = entity.data(&*host) else {
        return "No spawner there.".to_owned();
    };
    let held = read_matter(&data, KEY_MATTER).unwrap_or([0.0; 3]);
    let matter = matter_after_send(held, amounts);
    write_matter(&mut data, KEY_MATTER, matter);
    if entity.replace_data(host, data).is_err() {
        return "No spawner there.".to_owned();
    }
    format!(
        "/rftoolsutility: the spawner at {pos:?} holds matter {}, {}, {} (of {MATTER_STORAGE} each).",
        matter[0], matter[1], matter[2]
    )
}

/// `/rftoolsutility setting`: the crafter's speed, craft mode or keep mode.
fn set_setting(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
    key: &str,
    value: &str,
) -> String {
    let Some((entity, _tier)) = crafter_at(host, level, pos) else {
        return "No crafter there.".to_owned();
    };
    let setting = match key.to_ascii_lowercase().as_str() {
        "speed" => Speed::parse(value).map(Setting::Speed),
        "mode" => CraftMode::parse(value).map(Setting::Mode),
        "keep" => match value.to_ascii_lowercase().as_str() {
            "all" => Some(Setting::Keep(false)),
            "keep" => Some(Setting::Keep(true)),
            _ => None,
        },
        _ => None,
    };
    let Some(setting) = setting else {
        return "/rftoolsutility: settings are speed slow|fast, mode ext|int, keep all|keep."
            .to_owned();
    };
    let Some(mut data) = entity.data(&*host) else {
        return "No crafter there.".to_owned();
    };
    match setting {
        Setting::Speed(speed) => write_text(&mut data, KEY_SPEED, speed.name()),
        Setting::Mode(mode) => write_text(&mut data, KEY_MODE, mode.name()),
        Setting::Keep(keep) => write_text(&mut data, KEY_KEEP, if keep { "Keep" } else { "All" }),
    }
    if entity.replace_data(host, data).is_err() {
        return "No crafter there.".to_owned();
    }
    format!("/rftoolsutility: the crafter at {pos:?} sets {key} to {value}.")
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
        let patch = entity_patch(&nbt, TANK_TYPE_ID);
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

    // -- the crafter's glue (slice 3) ------------------------------------------

    #[test]
    fn the_taught_recipes_round_trip() {
        let taught = vec![
            Taught {
                id: "minecraft:oak_planks".to_owned(),
                cells: std::array::from_fn(|cell| {
                    if cell == 0 {
                        vec!["minecraft:oak_log".to_owned()]
                    } else {
                        Vec::new()
                    }
                }),
            },
            Taught {
                id: "minecraft:crafting_table".to_owned(),
                cells: std::array::from_fn(|cell| {
                    if cell < 4 {
                        vec![
                            "minecraft:oak_planks".to_owned(),
                            "minecraft:spruce_planks".to_owned(),
                        ]
                    } else {
                        Vec::new()
                    }
                }),
            },
        ];
        let mut nbt = Compound::new();
        write_taught(&mut nbt, &taught);
        let back = read_taught(&nbt);
        assert_eq!(back.len(), 2);
        assert_eq!(back[0].id, "minecraft:oak_planks");
        assert_eq!(back[0].cells[0], vec!["minecraft:oak_log"]);
        assert!(back[0].cells[1].is_empty());
        assert_eq!(back[1].cells[3].len(), 2, "a tag's ids survive the trip");
        assert_eq!(read_taught(&Compound::new()).len(), 0);
    }

    fn crafting_recipe(pattern: Option<&[&str]>, ingredients: &[&[&str]]) -> mod_api::Recipe {
        let json = match pattern {
            Some(rows) => mod_api::serde_json::json!({ "pattern": rows }),
            None => mod_api::serde_json::json!({}),
        };
        let mut recipe = mod_api::Recipe::new("test:recipe", "minecraft:crafting", json.into());
        recipe.ingredients = ingredients
            .iter()
            .map(|items| mod_api::Ingredient::of(items.iter().map(|id| (*id).to_owned()).collect()))
            .collect();
        recipe
    }

    #[test]
    fn a_shaped_recipe_maps_its_pattern_onto_the_grid() {
        let planks: &[&str] = &["minecraft:oak_planks"];
        let recipe = crafting_recipe(
            Some(&["X X", " X ", "X X"]),
            &[planks, planks, planks, planks, planks],
        );
        let cells = recipe_cells(&recipe).expect("a pattern maps cell by cell");
        assert_eq!(cells[0], vec!["minecraft:oak_planks"]);
        assert_eq!(cells[1], Vec::<String>::new(), "a space is an empty cell");
        assert_eq!(cells[2], vec!["minecraft:oak_planks"]);
        assert_eq!(cells[4], vec!["minecraft:oak_planks"]);
        assert_eq!(cells[6], vec!["minecraft:oak_planks"]);
        assert_eq!(cells[8], vec!["minecraft:oak_planks"]);
        assert_eq!(cells.iter().filter(|cell| cell.is_empty()).count(), 4);
    }

    #[test]
    fn a_shapeless_recipe_takes_its_ingredients_in_order() {
        let recipe = crafting_recipe(None, &[&["minecraft:oak_log"], &["minecraft:oak_log"]]);
        let cells = recipe_cells(&recipe).expect("a shapeless recipe fills its cells");
        assert_eq!(cells[0], vec!["minecraft:oak_log"]);
        assert_eq!(cells[1], vec!["minecraft:oak_log"]);
        assert!(cells[2..].iter().all(Vec::is_empty));
    }

    #[test]
    fn the_ghost_grid_shows_one_item_per_filled_cell() {
        let mut cells: [Vec<String>; 9] = std::array::from_fn(|_| Vec::new());
        cells[0] = vec![
            "minecraft:oak_planks".to_owned(),
            "minecraft:birch_planks".to_owned(),
        ];
        cells[4] = vec!["minecraft:stick".to_owned()];
        let grid = ghost_grid(&cells);
        assert_eq!(
            grid[0],
            Some("minecraft:oak_planks"),
            "the first candidate is shown"
        );
        assert_eq!(grid[4], Some("minecraft:stick"));
        assert_eq!(grid[1], None, "an empty cell stays empty");
        assert_eq!(
            grid.iter().filter(|cell| cell.is_some()).count(),
            2,
            "only the pattern's own cells are written"
        );
    }

    #[test]
    fn an_ingredient_with_no_named_items_cannot_be_taught() {
        let empty = mod_api::Ingredient::of(Vec::new());
        assert_eq!(accepted(&empty), None);
        let mut sorted = mod_api::Ingredient::of(vec![
            "minecraft:spruce_planks".to_owned(),
            "minecraft:birch_planks".to_owned(),
        ]);
        sorted.order = vec![
            "minecraft:spruce_planks".to_owned(),
            "minecraft:birch_planks".to_owned(),
        ];
        assert_eq!(
            accepted(&sorted),
            Some(vec![
                "minecraft:birch_planks".to_owned(),
                "minecraft:spruce_planks".to_owned()
            ]),
            "the ids are sorted so a saved grid reads back the same"
        );
    }

    #[test]
    fn the_crafter_container_keeps_the_ghost_slots_out_of_automation() {
        let container = crafter_container();
        assert_eq!(container.slots, CRAFTER_SLOTS);
        assert!(container.menu.is_none(), "this slice opens no menu");
        assert_eq!(container.inputs.first(), Some(&BUFFER_IN));
        assert_eq!(container.inputs.len(), BUFFER_IN_SLOTS);
        assert_eq!(container.outputs.first(), Some(&BUFFER_OUT));
        assert_eq!(container.outputs.len(), BUFFER_OUT_SLOTS);
        let reachable: Vec<usize> = container
            .inputs
            .iter()
            .chain(&container.outputs)
            .copied()
            .collect();
        assert!(
            container.faces.iter().all(|face| *face == reachable),
            "every face reaches the buffer slots and nothing else"
        );
        assert!(
            !reachable.contains(&GHOST_RESULT) && !reachable.contains(&GHOST_INPUT),
            "hoppers never reach the ghost grid"
        );
        assert_eq!(
            container.slots - 1,
            BUFFER_OUT + BUFFER_OUT_SLOTS,
            "the last slot is the filter module slot"
        );
    }

    #[test]
    fn a_slot_past_the_tiers_count_is_refused_with_that_count() {
        assert_eq!(recipe_slot(CrafterTier::Tier1, 1), Ok(()));
        assert_eq!(
            recipe_slot(CrafterTier::Tier1, 2).unwrap_err(),
            "/rftoolsutility: this crafter has 2 recipe slots."
        );
        assert_eq!(recipe_slot(CrafterTier::Tier2, 3), Ok(()));
        assert_eq!(
            recipe_slot(CrafterTier::Tier2, 4).unwrap_err(),
            "/rftoolsutility: this crafter has 4 recipe slots."
        );
        assert_eq!(recipe_slot(CrafterTier::Tier3, 7), Ok(()));
        assert_eq!(
            recipe_slot(CrafterTier::Tier3, 8).unwrap_err(),
            "/rftoolsutility: this crafter has 8 recipe slots."
        );
        assert_eq!(
            recipe_slot(CrafterTier::Tier3, usize::MAX).unwrap_err(),
            "/rftoolsutility: this crafter has 8 recipe slots.",
            "a negative slot parses to usize::MAX and is refused, not a panic"
        );
    }

    // -- the spawner's glue (slice 4) ------------------------------------------

    /// The jar's `minecraft_zombie.json`, as the engine hands it to the decoder (type removed
    /// by the caller, item1's object in 1.21.1's single form and item2's in its list form).
    fn zombie_recipe() -> mod_api::serde_json::Value {
        mod_api::serde_json::json!({
            "entity": "minecraft:zombie",
            "id": "rftoolsutility:minecraft_zombie",
            "item1": { "amount": 0.1, "object": { "item": "minecraft:rotten_flesh" } },
            "item2": { "amount": 0.2, "object": [
                { "item": "minecraft:dirt" },
                { "item": "minecraft:gravel" },
                { "item": "minecraft:sand" }
            ] },
            "item3": { "amount": 20.0, "object": [] },
            "power": 800
        })
    }

    #[test]
    fn a_spawner_file_reads_its_entity_power_and_matter() {
        let value = spawner_recipe(&zombie_recipe()).expect("the jar's own recipe reads");
        assert_eq!(value.entity, "minecraft:zombie");
        assert_eq!(value.power, 800);
        assert_eq!(value.matter, [0.1, 0.2, 20.0]);
        assert_eq!(
            value.recipe(),
            MobRecipe {
                entity: "minecraft:zombie".to_owned(),
                power: 800,
                matter: [0.1, 0.2, 20.0],
                objects: [
                    vec![MatterSource::Material("minecraft:rotten_flesh".to_owned())],
                    vec![
                        MatterSource::Material("minecraft:dirt".to_owned()),
                        MatterSource::Material("minecraft:gravel".to_owned()),
                        MatterSource::Material("minecraft:sand".to_owned()),
                    ],
                    Vec::new(),
                ],
            }
        );
        // A file the engine has already stripped of its type reads the same as one naming the
        // kind, which is how it arrives (the fixture above is the stripped form).
        let mut typed = zombie_recipe();
        typed["type"] = mod_api::serde_json::json!("rftoolsutility:spawner");
        assert_eq!(spawner_recipe(&typed), Ok(value));
    }

    #[test]
    fn a_spawner_file_without_an_entity_or_power_is_refused() {
        let mut no_entity = zombie_recipe();
        no_entity["entity"] = mod_api::serde_json::json!(null);
        assert_eq!(
            spawner_recipe(&no_entity),
            Err("a spawner recipe needs an entity".to_owned())
        );
        let mut empty = zombie_recipe();
        empty["entity"] = mod_api::serde_json::json!("");
        assert!(spawner_recipe(&empty).is_err());
        let mut no_power = zombie_recipe();
        no_power.as_object_mut().expect("an object").remove("power");
        assert_eq!(
            spawner_recipe(&no_power),
            Err("a spawner recipe needs a power".to_owned())
        );
        let mut other = zombie_recipe();
        other["type"] = mod_api::serde_json::json!("minecraft:crafting_shaped");
        assert!(
            spawner_recipe(&other).is_err(),
            "a file of another type is not this kind's recipe"
        );
    }

    #[test]
    fn a_spawner_value_writes_back_as_the_recipe_it_came_from() {
        let value = spawner_recipe(&zombie_recipe()).expect("the jar's own recipe reads");
        let written = write_spawner(&value);
        let back: mod_api::serde_json::Value =
            mod_api::serde_json::from_str(&written).expect("the write is JSON");
        assert_eq!(
            spawner_recipe(&back),
            Ok(value),
            "the client's copy decodes to the same rules"
        );
    }

    #[test]
    fn the_matter_amounts_round_trip() {
        let mut nbt = Compound::new();
        write_matter(&mut nbt, KEY_NEEDS, [0.1, 0.5, 30.0]);
        assert_eq!(read_matter(&nbt, KEY_NEEDS), Some([0.1, 0.5, 30.0]));
        assert_eq!(read_matter(&nbt, "nothing"), None);
        let mut short = Compound::new();
        put(
            &mut short,
            KEY_NEEDS,
            Tag::List(vec![Tag::Int(1), Tag::Float(2.0f32.to_bits())]),
        );
        assert_eq!(
            read_matter(&short, KEY_NEEDS),
            None,
            "an amount that is not a float is not a recipe's matter"
        );
    }

    #[test]
    fn the_mob_readout_is_the_line_the_harness_parses() {
        let mut nbt = Compound::new();
        write_text(&mut nbt, KEY_MOB, "minecraft:zombie");
        write_int(&mut nbt, KEY_COST, 800);
        write_matter(&mut nbt, KEY_NEEDS, [0.1, 0.2, 20.0]);
        write_matter(&mut nbt, KEY_MATTER, [0.5, 0.5, 0.5]);
        write_int(&mut nbt, KEY_ENERGY, 800);
        write_int(&mut nbt, KEY_SPAWNED, 1);
        assert_eq!(
            spawner_readout([202, 101, 3], &nbt, Some("7")),
            "Spawner at [202, 101, 3]: energy=800/200000 mob=minecraft:zombie cost=800 \
             needs=0.1,0.2,20 matter=0.5,0.5,0.5 spawned=1 nonce=7"
        );
        assert_eq!(
            spawner_readout([1, 2, 3], &Compound::new(), None),
            "Spawner at [1, 2, 3]: energy=0/200000 mob=none cost=- needs=- matter=0,0,0 spawned=0",
            "a fresh spawner knows no mob and no cost, and its stores are empty"
        );
    }

    #[test]
    fn a_new_mob_clears_the_matter_the_previous_one_was_fed() {
        assert_eq!(
            matter_after_mob_change(
                Some("minecraft:zombie"),
                "minecraft:zombie",
                [0.1, 0.2, 20.0]
            ),
            [0.1, 0.2, 20.0],
            "setting the mob it already holds keeps its stores"
        );
        assert_eq!(
            matter_after_mob_change(
                Some("minecraft:zombie"),
                "minecraft:blaze",
                [0.1, 0.2, 20.0]
            ),
            [0.0, 0.0, 0.0],
            "a different mob must not inherit the previous mob's matter"
        );
        assert_eq!(
            matter_after_mob_change(None, "minecraft:zombie", [9.0, 9.0, 9.0]),
            [0.0, 0.0, 0.0],
            "a machine that knew no mob starts empty"
        );
    }

    #[test]
    fn a_beamer_send_adds_to_each_store_and_caps_it() {
        assert_eq!(
            matter_after_send([0.0, 0.0, 0.0], [0.1, 0.2, 20.0]),
            [0.1, 0.2, 20.0],
            "the recipe's own amounts land one per store"
        );
        assert_eq!(
            matter_after_send([0.1, 0.2, 20.0], [0.5, 0.3, 1.0]),
            [0.6, 0.5, 21.0],
            "each store takes its own amount, not the first one"
        );
        assert_eq!(
            matter_after_send([6_300.0, 6_300.0, 6_300.0], [200.0, 200.0, 200.0]),
            [6_400.0, 6_400.0, 6_400.0],
            "maxMatterStorage caps every store"
        );
        assert_eq!(
            matter_after_send([1.0, 1.0, 1.0], [-1.0, -1.0, -1.0]),
            [0.0, 0.0, 0.0],
            "a negative send drains a store, and no further"
        );
    }

    #[test]
    fn a_spawner_takes_energy_on_every_face_and_gives_none_out() {
        let buffer = spawner_energy();
        assert_eq!(buffer.capacity, SPAWNER_CAPACITY);
        assert_eq!(buffer.max_insert, SPAWNER_RECEIVE);
        assert_eq!(buffer.max_extract, 0, "upstream's spawner only consumes");
        assert!(buffer.insert_faces.iter().all(|face| *face));
        assert!(buffer.extract_faces.iter().all(|face| !*face));
    }

    // -- the matter beamer's glue (slice 8) --------------------------------------

    #[test]
    fn a_spawner_file_reads_the_material_sources_its_objects_name() {
        assert_eq!(
            objects_of(&zombie_recipe()),
            [
                vec![WireSource::Material("minecraft:rotten_flesh".to_owned())],
                vec![
                    WireSource::Material("minecraft:dirt".to_owned()),
                    WireSource::Material("minecraft:gravel".to_owned()),
                    WireSource::Material("minecraft:sand".to_owned()),
                ],
                Vec::new(),
            ],
            "item1 is one object, item2 a list of them, item3 the empty (living) form"
        );
        // The jar's blaze recipe: item1 is a tag, item2 an item.
        let blaze = mod_api::serde_json::json!({
            "entity": "minecraft:blaze",
            "item1": { "amount": 0.1, "object": { "tag": "c:rods/blaze" } },
            "item2": { "amount": 0.5, "object": { "item": "minecraft:netherrack" } },
            "item3": { "amount": 30.0, "object": [] },
            "power": 1000
        });
        let objects = objects_of(&blaze);
        assert_eq!(
            objects[0],
            vec![WireSource::Tagged("c:rods/blaze".to_owned())],
            "a tag object keeps its id, without the JSON's `#`"
        );
        assert_eq!(
            objects[1],
            vec![WireSource::Material("minecraft:netherrack".to_owned())]
        );
        assert!(objects[2].is_empty());
    }

    #[test]
    fn a_destination_round_trips_through_the_saved_compound() {
        let mut nbt = Compound::new();
        assert_eq!(read_pos(&nbt, KEY_DEST), None, "a fresh beamer is unlinked");
        write_pos(&mut nbt, KEY_DEST, [200, 101, -16]);
        assert_eq!(read_pos(&nbt, KEY_DEST), Some([200, 101, -16]));
        remove(&mut nbt, KEY_DEST);
        assert_eq!(read_pos(&nbt, KEY_DEST), None, "an unlink clears the key");
        let mut short = Compound::new();
        put(
            &mut short,
            KEY_DEST,
            Tag::List(vec![Tag::Int(1), Tag::Int(2)]),
        );
        assert_eq!(
            read_pos(&short, KEY_DEST),
            None,
            "a destination that is not three ints is not a destination"
        );
    }

    #[test]
    fn the_beamer_readout_is_the_line_the_harness_parses() {
        let mut nbt = Compound::new();
        write_int(&mut nbt, KEY_ENERGY, 4_000);
        write_int(&mut nbt, KEY_SENT, 2);
        write_pos(&mut nbt, KEY_DEST, [200, 101, -16]);
        assert_eq!(
            beamer_readout(
                [200, 101, -12],
                &nbt,
                Some(("minecraft:rotten_flesh", 6)),
                true,
                Some("9")
            ),
            "Beamer at [200, 101, -12]: energy=4000/200000 dest=[200, 101, -16] \
             slot=minecraft:rotten_fleshx6 sent=2 lit=true nonce=9"
        );
        assert_eq!(
            beamer_readout([1, 2, 3], &Compound::new(), None, false, None),
            "Beamer at [1, 2, 3]: energy=0/200000 dest=none slot=- sent=0 lit=false",
            "a fresh beamer is unlinked, empty and dark"
        );
    }

    // -- the destination analyzer's harness readout (slice 10) -------------------

    #[test]
    fn the_analyzer_readout_is_the_line_the_harness_parses() {
        assert_eq!(
            analyzer_line(
                [200, 101, 8],
                "warn",
                "minecraft:the_end@[203, 101, 12]",
                Some("31")
            ),
            "/rftoolsutility: analyzer at [200, 101, 8]: signal=0 status=warn dial=minecraft:the_end@[203, 101, 12] nonce=31",
            "an underpowered receiver reads a low signal"
        );
        assert_eq!(
            analyzer_line(
                [200, 101, 8],
                "ok",
                "minecraft:the_end@[203, 101, 12]",
                None
            ),
            "/rftoolsutility: analyzer at [200, 101, 8]: signal=15 status=ok dial=minecraft:the_end@[203, 101, 12]",
            "a ready receiver reads the full signal"
        );
        assert!(
            analyzer_line([200, 101, 8], "unknown", "none", None)
                .contains("signal=7 status=unknown")
        );
    }

    // -- the sensor's harness readout (slice 11) ---------------------------------

    #[test]
    fn the_sensor_readout_is_the_line_the_harness_parses() {
        assert_eq!(
            sensor_line(
                [200, 101, 8],
                15,
                "blocks",
                "minecraft:stone",
                "north",
                Some("7")
            ),
            "/rftoolsutility: sensor at [200, 101, 8]: signal=15 type=blocks target=minecraft:stone area=1 group=one facing=north nonce=7"
        );
        assert_eq!(
            sensor_line([200, 101, 8], 0, "any", "none", "south", None),
            "/rftoolsutility: sensor at [200, 101, 8]: signal=0 type=any target=none area=1 group=one facing=south"
        );
    }

    // -- the sequencer's harness readout and pattern parsing (slice 12) ----------

    #[test]
    fn the_sequencer_readout_is_the_line_the_harness_parses() {
        let data = SequencerData {
            bits: 5,
            mode: SequencerMode::Step,
            delay: 1,
            stepcount: 4,
            endstate: false,
        };
        let state = SequencerState {
            step: 2,
            prev_in: false,
            timer: 1,
            output: true,
        };
        assert_eq!(
            sequencer_line([201, 101, 0], &data, &state, "north", Some("9")),
            "/rftoolsutility: sequencer at [201, 101, 0]: step=2 bits=5 mode=Step output=15 delay=1 count=4 endstate=false facing=north nonce=9"
        );
    }

    #[test]
    fn the_sequencer_pattern_accepts_decimal_and_hex() {
        assert_eq!(parse_bits("5"), Some(5));
        assert_eq!(parse_bits("0x5"), Some(5));
        assert_eq!(parse_bits("0xffffffffffffffff"), Some(u64::MAX));
        assert_eq!(parse_bits("nonsense"), None);
        assert!(sequencer_front("north"));
        assert!(!sequencer_front("up"));
    }

    // -- the timer's harness readout (slice 13) -------------------------------

    #[test]
    fn the_timer_readout_is_the_line_the_harness_parses() {
        let data = TimerData {
            delay: 6,
            redstone_pauses: false,
        };
        let state = TimerState {
            prev_in: false,
            timer: 3,
            output: false,
        };
        assert_eq!(
            timer_line(
                [201, 101, 0],
                &data,
                &state,
                12,
                -1,
                "north",
                18,
                "lit",
                Some("1")
            ),
            "/rftoolsutility: timer at [201, 101, 0]: delay=6 pauses=false output=0 timer=3 pulse=12 fired=-1 facing=north time=18 lamp=lit nonce=1"
        );
    }

    // -- the counter's harness readout (slice 14) -----------------------------

    #[test]
    fn the_counter_readout_is_the_line_the_harness_parses() {
        let data = CounterData {
            counter: 4,
            current: 0,
        };
        let state = CounterState {
            prev_in: false,
            output: true,
        };
        assert_eq!(
            counter_line(
                [201, 101, 0],
                &data,
                &state,
                4,
                312,
                312,
                "north",
                320,
                "lit",
                Some("1")
            ),
            "/rftoolsutility: counter at [201, 101, 0]: limit=4 current=0 output=15 pulses=4 pulse=312 fire=312 lamp=lit facing=north time=320 nonce=1"
        );
        assert!(counter_front("north"));
        assert!(!counter_front("up"));
    }

    // ---- Slice 15: the analog's glue ----

    #[test]
    fn the_analog_readout_is_the_line_the_harness_parses() {
        let data = AnalogData {
            mul_equal: 2.0,
            mul_less: 0.5,
            mul_greater: 3.0,
            add_equal: 1,
            add_less: 2,
            add_greater: 3,
        };
        assert_eq!(
            analog_line(
                [201, 101, 0],
                &data,
                15,
                4,
                4,
                11,
                312,
                "north",
                320,
                "lit",
                Some("1")
            ),
            "/rftoolsutility: analog at [201, 101, 0]: input=15 a=4 b=4 branch=eq mul=2,0.5,3 add=1,2,3 output=11 changed=312 lamp=lit facing=north time=320 nonce=1"
        );
    }

    #[test]
    fn an_analogs_six_settings_round_trip_through_the_saved_compound() {
        let mut nbt = Compound::new();
        let data = AnalogData {
            mul_equal: 0.5,
            mul_less: 1.25,
            mul_greater: 2.0,
            add_equal: -10,
            add_less: 0,
            add_greater: 7,
        };
        write_analog(&mut nbt, &data);
        assert_eq!(analog_data(&nbt), data);
        assert_eq!(analog_data(&Compound::new()), AnalogData::default());
        assert_eq!(
            analog_signal_from_state("rftoolsutility:analog[signal=11]"),
            11
        );
        assert_eq!(
            analog_signal_from_state("rftoolsutility:analog[signal=0]"),
            0
        );
    }
}
