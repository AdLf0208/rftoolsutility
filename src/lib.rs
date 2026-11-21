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
//! need. `/rftoolsutility info|tank|crafter|recipe|buffer|energy|setting|spawner|mob|recipes`
//! serves the harness, because upstream's controls are its GUIs.
//!
//! The numbers — the capacity, the fill level, what a bucket click does, the craft loop, the
//! spawn cost — are in [`runtime`], a plain module with no `mod_api` types, so `cargo test`
//! alone proves them.
use mod_api::{
    self, BlockClick, BlockDropsEvent, BlockEntityPart, BlockEntityRef, BlockEntityTickEvent,
    BlockEntityTickReply, BlockEntityTicksEvent, BlockPart, Command, CommandArgumentValue,
    CommandSource, Compound, CraftingGrid, CraftingRecipeCodec, CraftingRecipeKind, Direction,
    Flow, FluidResource, FluidStack, Hand, Hooks, Host, HostBlockStates, HostSide, HostTransaction,
    ITEM_FLUIDS, Identifier, InteractionResult, ItemKey, ItemPart, ItemPlace, ItemStack, LevelHost,
    LevelRef, ModContainer, ModEnergyBuffer, ModFluidTanks, OnBlockPlacedByEvent, PlayerRef,
    RecipeInput, RegisterCommandsEvent, Registrar, RegistrationContext, RunModCommandEvent,
    ServerHost, ServerTickEvent, SpawnReason, Tag, Text, UseItemOnBlockEvent, UsePhase, fluid_util,
};
use std::sync::{Mutex, OnceLock};

pub mod runtime;

use runtime::{
    BUCKET, BUFFER_IN, BUFFER_IN_SLOTS, BUFFER_OUT, BUFFER_OUT_SLOTS, CAPACITY, CRAFTER_CAPACITY,
    CRAFTER_RECEIVE, CRAFTER_SLOTS, Click, CraftMode, CrafterTier, GHOST_INPUT, GHOST_RESULT,
    MobRecipe, SPAWNER_CAPACITY, SPAWNER_RECEIVE, Speed, Tank, bucket_click, carried, clamped,
    fill_level, placements, plan, recipe_for, result_range, spawn_at, spawn_once, spend, steps,
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
const KEY_SPAWNED: &str = "spawned";
/// The block's own placement state, which picks the side a spawn appears on
/// (`OrientationTools.getOrientation`).
const KEY_FACING: &str = "facing";

/// A spawner's item, resolved once in `register` for the same reason as the tank's.
static SPAWNER_ITEM: OnceLock<ItemKey> = OnceLock::new();

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
}

impl SpawnerValue {
    /// The machine's own copy of the recipe's rules.
    fn recipe(&self) -> MobRecipe {
        MobRecipe {
            entity: self.entity.clone(),
            power: self.power,
            matter: self.matter,
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
    })
}

/// Writes the value back as the recipe file it came from, for the client's copy of the table.
fn write_spawner(value: &SpawnerValue) -> String {
    mod_api::serde_json::json!({
        "type": SPAWNER_RECIPE_TYPE,
        "entity": value.entity.as_str(),
        "power": value.power,
        "item1": { "amount": value.matter[0] },
        "item2": { "amount": value.matter[1] },
        "item3": { "amount": value.matter[2] },
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

/// One server tick of the spawner (`SpawnerTileEntity.tickServer`): with a mob set and its
/// recipe's `power` in the buffer, pay the cost and spawn one mob on the block's facing side.
///
/// Upstream returns early until all three matter amounts are stored; the matter system (its
/// three tanks and the matter beamers that fill them) is not part of this slice, so this tick
/// spends energy only. The recipe's own amounts are still read and kept on the machine
/// (`KEY_NEEDS`) and printed by the readout.
fn spawner_tick(host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
    let pos = event.pos;
    let Some(mob) = read_text(&event.entity.nbt, KEY_MOB) else {
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    };
    let cost = read_int(&event.entity.nbt, KEY_COST);
    let energy = read_int(&event.entity.nbt, KEY_ENERGY).clamp(0, SPAWNER_CAPACITY);
    let Some(left) = spawn_once(energy, cost) else {
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
    // adds it to the level. A refusal spawns nothing, and nothing is paid.
    if level
        .spawn_mob(host, &mob, at, SpawnReason::Spawner)
        .is_none()
    {
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    }
    write_int(&mut event.entity.nbt, KEY_ENERGY, left);
    let spawned = read_int(&event.entity.nbt, KEY_SPAWNED).saturating_add(1);
    write_int(&mut event.entity.nbt, KEY_SPAWNED, spawned);
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
    let needs = read_matter(nbt, KEY_NEEDS).map_or_else(
        || "-".to_owned(),
        |needs| format!("{},{},{}", needs[0], needs[1], needs[2]),
    );
    let spawned = read_int(nbt, KEY_SPAWNED).max(0);
    let suffix = nonce.map_or(String::new(), |nonce| format!(" nonce={nonce}"));
    format!(
        "Spawner at {pos:?}: energy={energy}/{SPAWNER_CAPACITY} mob={mob} cost={cost} \
         needs={needs} spawned={spawned}{suffix}"
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
                "Usage: /rftoolsutility info|tank|crafter|recipe|buffer|energy|setting|spawner|mob <pos> …",
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
/// it). The harness's stand-in for a hopper; a slot outside the buffer range is refused.
fn set_slot(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
    slot: i32,
    item: &str,
    count: i32,
) -> String {
    let Some((entity, _tier)) = crafter_at(host, level, pos) else {
        return "No crafter there.".to_owned();
    };
    let slot = usize::try_from(slot).unwrap_or(usize::MAX);
    let buffers = BUFFER_IN..BUFFER_OUT + BUFFER_OUT_SLOTS;
    if !buffers.contains(&slot) {
        return format!(
            "/rftoolsutility: slot {slot} is not one of the crafter's buffer slots \
             ({BUFFER_IN}..{}).",
            BUFFER_IN + BUFFER_IN_SLOTS + BUFFER_OUT_SLOTS
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
            "/rftoolsutility: the crafter at {pos:?} slot {slot} holds {}.",
            match count {
                0 => "nothing".to_owned(),
                count => format!("{item} x{count}"),
            }
        ),
        _ => "No crafter there.".to_owned(),
    }
}

/// `/rftoolsutility energy`: set a machine's buffer, clamped to its own capacity. The crafter
/// and the spawner both keep their FE in the engine's `energy` int, so one command serves both.
fn set_energy(host: &mut dyn ServerHost, level: LevelRef, pos: [i32; 3], amount: i32) -> String {
    let (entity, capacity, name) = if let Some((entity, _tier)) = crafter_at(&*host, level, pos) {
        (entity, CRAFTER_CAPACITY, "crafter")
    } else if let Some(entity) = machine_at(&*host, level, pos, SPAWNER) {
        (entity, SPAWNER_CAPACITY, "spawner")
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
/// per table load).
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
    write_text(&mut data, KEY_MOB, &recipe.entity);
    write_int(&mut data, KEY_COST, recipe.power);
    write_matter(&mut data, KEY_NEEDS, recipe.matter);
    if entity.replace_data(host, data).is_err() {
        return "No spawner there.".to_owned();
    }
    format!(
        "/rftoolsutility: the spawner at {pos:?} spawns {} for {} FE.",
        recipe.entity, recipe.power
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
        write_int(&mut nbt, KEY_ENERGY, 800);
        write_int(&mut nbt, KEY_SPAWNED, 1);
        assert_eq!(
            spawner_readout([202, 101, 3], &nbt, Some("7")),
            "Spawner at [202, 101, 3]: energy=800/200000 mob=minecraft:zombie cost=800 \
             needs=0.1,0.2,20 spawned=1 nonce=7"
        );
        assert_eq!(
            spawner_readout([1, 2, 3], &Compound::new(), None),
            "Spawner at [1, 2, 3]: energy=0/200000 mob=none cost=- needs=- spawned=0",
            "a fresh spawner knows no mob and no cost"
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
}
