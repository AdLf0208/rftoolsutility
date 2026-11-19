//! RFTools Utility — the tank, as one `impl Hooks` at the crate root.
//!
//! The carried content is this package's own `assets/` and `data/` (`rftoolsutility:tank`, its
//! item, the nine fill models and the recipe); the behaviour is here: a [`BlockEntityPart`] on
//! the tank declares its block entity, ticks on the server to mirror the tank's contents into
//! the block's `level` state, answers a right click with a fluid container through
//! [`fluid_util`], and offers `/rftoolsutility info|tank` for the harness.
//!
//! The numbers — the capacity, the fill level, one fluid at a time, what a bucket click does —
//! are in [`runtime`], a plain module with no `mod_api` types, byte-identical to the retired
//! `.hnmod` bundle's `runtime.rs`, so `cargo test` alone proves them.
use mod_api::{
    self, BlockEntityPart, BlockEntityRef, BlockEntityTickEvent, BlockEntityTickReply,
    BlockEntityTicksEvent, BlockPart, Command, CommandArgumentValue, CommandSource, Compound,
    Direction, Flow, Hand, Hooks, ITEM_FLUIDS, InteractionResult, ItemPlace, LevelRef,
    ModFluidTanks, PlayerRef, RegisterCommandsEvent, Registrar, RegistrationContext,
    RunModCommandEvent, ServerHost, Tag, Text, UseItemOnEvent, fluid_util,
};

pub mod runtime;

use runtime::{CAPACITY, Click, Tank, bucket_click, fill_level};

/// The tank block and its item (`TankModule.TANK`).
const TANK: &str = "rftoolsutility:tank";
/// The block entity type the port declares for it (`EntityBlock.newBlockEntity`).
const TANK_TYPE: &str = "tank";
/// The block's fill-level property (`TankTE.computeLevel` → `tank<level>`).
const KEY_LEVEL: &str = "level";
/// The tank's index in its fluid table (upstream has one tank).
const TANK_INDEX: usize = 0;

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

/// What a bucket click answers (`TankTE.onBlockActivated`, measured against the engine).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Answer {
    /// Not a fluid container: the click is not the tank's, so vanilla's own runs.
    Pass,
    /// A fluid container the tank refuses: the click ends here, so nothing pours beside it.
    Refused,
    /// A container to move fluid with; [`fluid_util`] performs it.
    Move,
}

fn answer(click: Click) -> Answer {
    match click {
        Click::NotYours => Answer::Pass,
        Click::Refused => Answer::Refused,
        Click::Fill { .. } | Click::Drain { .. } => Answer::Move,
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
impl BlockPart for TankPart {
    /// `TankTE.onBlockActivated`: a fluid container in the hand fills the tank, and a drained
    /// tank fills the container. Upstream's `FluidUtil` accepts any container, so a container
    /// the runtime does not model is still a move (the engine decides); the tank owns every
    /// fluid-container click, so a tank that is full, holds another fluid or holds less than a
    /// whole bucket ends the click instead of pouring the bucket on the ground beside it (a
    /// recorded deviation: upstream answers `PASS`, which lets vanilla empty it there).
    fn use_item_on(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseItemOnEvent<'_>,
    ) -> Option<InteractionResult> {
        let click = event.click;
        let stack = click.stack?;
        let held = stack.item().id(&*host);
        let player = host.player(click.player)?;
        let level = player.level(&*host)?;
        let tank = tank_at(host, level, click.pos)?;
        let decision = match answer(bucket_click(&held, &tank)) {
            // Not one of the runtime's two vanilla buckets: the engine still knows a mod's own
            // fluid container, and upstream accepted those too.
            Answer::Pass if is_fluid_container(host, player, click.hand) => Answer::Move,
            other => other,
        };
        match decision {
            Answer::Pass => None,
            Answer::Refused => Some(InteractionResult::Fail),
            Answer::Move => {
                // `FluidUtil.interactWithFluidHandler`: the engine moves the container's own
                // fluid, swaps the hand and plays the sound and game event.
                let side = Direction::ALL
                    .into_iter()
                    .find(|direction| direction.normal() == click.hit.face);
                let moved = fluid_util::interact_with_fluid_handler(
                    host, player, click.hand, level, click.pos, side,
                );
                Some(if moved {
                    InteractionResult::Success
                } else {
                    InteractionResult::Fail
                })
            }
        }
    }
}

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

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Declare the tank's block entity and its one tank (`EntityBlock.newBlockEntity`,
    /// `Capabilities.Fluid.BLOCK`).
    fn register(&self, r: &mut dyn Registrar, cx: &RegistrationContext) {
        let block = r.block_key(TANK);
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
                let (
                    Some(CommandArgumentValue::Id(fluid)),
                    Some(CommandArgumentValue::Integer(mb)),
                ) = (invocation.argument("fluid"), invocation.argument("mb"))
                else {
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
                let amount = (*mb).clamp(0, CAPACITY);
                let tank = if amount == 0 {
                    Tank::new()
                } else {
                    Tank {
                        fluid: Some(fluid.to_string()),
                        amount,
                    }
                };
                write_tank(&mut data, &tank);
                if entity.replace_data(host, data).is_err() {
                    feedback(host, "No tank there.");
                    return Flow::Cancel;
                }
                mirror_level(host, pos, &tank);
                feedback(
                    host,
                    &format!(
                        "Tank at {pos:?}: {} {amount}/{CAPACITY} mB.",
                        tank.fluid.as_deref().unwrap_or("empty")
                    ),
                );
            }
            _ => return Flow::Continue,
        }
        Flow::Cancel
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
            answer(bucket_click("minecraft:stone", &Tank::new())),
            Answer::Pass
        );
        assert_eq!(
            answer(bucket_click("minecraft:water_bucket", &water(CAPACITY))),
            Answer::Refused
        );
        assert_eq!(
            answer(bucket_click("minecraft:bucket", &Tank::new())),
            Answer::Refused
        );
    }

    #[test]
    fn a_bucket_that_moves_is_a_move() {
        assert_eq!(
            answer(bucket_click("minecraft:water_bucket", &Tank::new())),
            Answer::Move
        );
        assert_eq!(
            answer(bucket_click("minecraft:bucket", &water(runtime::BUCKET))),
            Answer::Move
        );
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
