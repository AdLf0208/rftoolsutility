// Pure RFTools Utility slice-1 arithmetic (standalone crate lib, injected as
// `crate::game::rftoolsutility`). No game access lives here: the tank's
// capacity, its one-fluid-at-a-time rule, its fill level for the block model,
// and what a bucket click does.
//
// Upstream (`TankTE`, `TankConfiguration`, `TankBakedModel`, read from
// RFToolsUtility 1.21-7.0.13) is a 32,000 mB tank (`maxCapacity`'s default, and
// the mod's own manual: "can (by default) hold up to 32 buckets") that holds one
// fluid at a time, fills and drains from a bucket in the player's hand, and maps
// its contents to a 0..8 render level.

/// `TankConfiguration.MAXCAPACITY`'s default: the tank's size in millibuckets.
pub const CAPACITY: i32 = 32_000;
/// One bucket of fluid: vanilla's bucket holds 1,000 mB.
pub const BUCKET: i32 = 1_000;
/// `TankTE.computeLevel`'s top level, and the number of fill textures the jar
/// ships (`block/tank0.png` … `block/tank8.png`).
pub const MAX_LEVEL: u8 = 8;

/// `TankTE.computeLevel`: the block model's fill level for `amount` mB.
///
/// Upstream: 0 when the tank is empty, else `8 * amount / capacity + 1`, capped
/// at 8. So 1..3,999 mB is level 1 and a full 32,000 mB tank is level 8 (the
/// uncapped arithmetic would say 9).
#[must_use]
pub fn fill_level(amount: i32) -> u8 {
    if amount <= 0 {
        return 0;
    }
    // `8 * amount` in `i64`: the product overflows an `i32` for a tank a mod
    // hands a nonsense amount, and upstream's `int` arithmetic would wrap.
    let steps = 8 * i64::from(amount) / i64::from(CAPACITY) + 1;
    u8::try_from(steps.clamp(1, i64::from(MAX_LEVEL))).unwrap_or(MAX_LEVEL)
}

/// One tank's contents: the fluid it holds (`None` when empty) and how much.
///
/// Upstream keeps this in a NeoForge `FluidTank` (one fluid at a time, fills
/// clamped to the capacity, drains clamped to what it holds).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tank {
    /// `minecraft:water`, `minecraft:lava` or a mod's own fluid; `None` when
    /// the tank is empty.
    pub fluid: Option<String>,
    /// Stored millibuckets, 0..=CAPACITY.
    pub amount: i32,
}

impl Tank {
    /// An empty tank.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The render level the block's state carries (`TankTE.computeLevel`).
    #[must_use]
    pub fn level(&self) -> u8 {
        fill_level(self.amount)
    }

    /// How many millibuckets the tank can still take.
    #[must_use]
    pub fn room(&self) -> i32 {
        (CAPACITY - self.amount).max(0)
    }

    /// Whether the tank would take `fluid`: an empty tank takes any fluid, a
    /// filled one only the fluid it already holds.
    #[must_use]
    pub fn accepts(&self, fluid: &str) -> bool {
        match self.fluid.as_deref() {
            None => true,
            Some(held) => held == fluid,
        }
    }

    /// `FluidTank.fill`: up to `amount` mB of `fluid`, clamped to the room left,
    /// answering what did not fit. A different fluid than the one it holds is
    /// refused whole.
    ///
    /// The engine performs the write (`native_fluid_insert`); this is the model
    /// the tests pin it against, so the game build does not call it.
    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "the pure model of the engine's fill; the glue writes through native_fluid_insert"
        )
    )]
    pub fn insert(&mut self, fluid: &str, amount: i32) -> i32 {
        if amount <= 0 || !self.accepts(fluid) {
            return amount;
        }
        let taken = amount.min(self.room());
        if taken > 0 {
            self.fluid = Some(fluid.to_owned());
            self.amount += taken;
        }
        amount - taken
    }

    /// `FluidTank.drain`: up to `amount` mB, answering what was taken.
    ///
    /// The engine performs the write (`native_fluid_extract`); this is the model
    /// the tests pin it against, so the game build does not call it.
    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "the pure model of the engine's drain; the glue writes through native_fluid_extract"
        )
    )]
    pub fn extract(&mut self, amount: i32) -> i32 {
        if amount <= 0 {
            return 0;
        }
        let taken = amount.min(self.amount.max(0));
        self.amount -= taken;
        if self.amount == 0 {
            self.fluid = None;
        }
        taken
    }
}

/// What a right click with `held` does to the tank
/// (`TankTE.onBlockActivated`: `tryEmptyContainerAndStow`, then
/// `tryFillContainerAndStow`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Click {
    /// The held container pours `fluid` in and becomes `result`.
    Fill {
        /// The fluid the held container holds.
        fluid: &'static str,
        /// What the hand holds afterwards: the empty container.
        result: &'static str,
    },
    /// The held container takes a bucket out of the tank and becomes `result`.
    Drain {
        /// What the hand holds afterwards: the filled container.
        result: &'static str,
    },
    /// The tank owns the click but does nothing: it is full, holds another
    /// fluid, or holds less than a whole bucket.
    Refused,
    /// The held item is no fluid container: the click is not the tank's.
    NotYours,
}

/// `minecraft:water_bucket` → `minecraft:water` (the containers
/// `TankTE.CONTAINER_FACTORY` accepts, which `FluidUtil` can empty).
#[must_use]
pub fn held_fluid(held: &str) -> Option<&'static str> {
    match held {
        "minecraft:water_bucket" => Some("minecraft:water"),
        "minecraft:lava_bucket" => Some("minecraft:lava"),
        _ => None,
    }
}

/// `minecraft:water` → `minecraft:water_bucket`: the container a drained
/// bucket fills.
#[must_use]
pub fn fluid_container(fluid: &str) -> Option<&'static str> {
    match fluid {
        "minecraft:water" => Some("minecraft:water_bucket"),
        "minecraft:lava" => Some("minecraft:lava_bucket"),
        _ => None,
    }
}

/// What `held` does to `tank` when a player right-clicks it.
///
/// Upstream empties the held container first and fills it second, so a filled
/// bucket always pours in and an empty one always draws out; a whole bucket is
/// the unit in both directions (`FluidUtil` moves one container's worth).
#[must_use]
pub fn bucket_click(held: &str, tank: &Tank) -> Click {
    if let Some(fluid) = held_fluid(held) {
        return if tank.accepts(fluid) && tank.room() >= BUCKET {
            Click::Fill {
                fluid,
                result: "minecraft:bucket",
            }
        } else {
            Click::Refused
        };
    }
    if held == "minecraft:bucket" {
        return match tank.fluid.as_deref().and_then(fluid_container) {
            Some(result) if tank.amount >= BUCKET => Click::Drain { result },
            _ => Click::Refused,
        };
    }
    Click::NotYours
}

/// What a broken tank carries on its item: what it holds, or nothing when it is
/// empty (`TankTE.collectImplicitComponents` moves the tank's fluid handler onto
/// the item, and an empty handler carries no fluid).
#[must_use]
pub fn carried(tank: &Tank) -> Option<&Tank> {
    (tank.amount > 0 && tank.fluid.is_some()).then_some(tank)
}

/// The tank an item's carried contents restore: at most the capacity, because
/// `FluidTank.setFluid` clamps a loaded amount, and empty when the item names no
/// fluid (`TankTE.applyImplicitComponents`).
#[must_use]
pub fn clamped(tank: &Tank) -> Tank {
    match tank.fluid.as_deref() {
        Some(fluid) if tank.amount > 0 => Tank {
            fluid: Some(fluid.to_owned()),
            amount: tank.amount.clamp(0, CAPACITY),
        },
        _ => Tank::new(),
    }
}

// ---------------------------------------------------------------------------
// Slice 3: the crafter (`CrafterBaseTE`, `CrafterConfiguration`, `CrafterContainer`)
// ---------------------------------------------------------------------------

/// `CrafterConfiguration.MAXENERGY`'s default (`crafterMaxRF`): the crafter's buffer.
pub const CRAFTER_CAPACITY: i32 = 50_000;
/// `CrafterConfiguration.RECEIVEPERTICK`'s default (`crafterRFPerTick`).
pub const CRAFTER_RECEIVE: i32 = 500;
/// `CrafterConfiguration.rfPerOperation`'s default: what one crafting operation costs.
pub const RF_PER_OPERATION: i32 = 100;
/// `CrafterConfiguration.speedOperations`'s default: operations per tick in fast mode.
pub const FAST_OPERATIONS: i32 = 5;
/// `CrafterContainer.SLOT_CRAFTINPUT`: cell (0,0) of the 3x3 ghost grid.
pub const GHOST_INPUT: usize = 0;
/// `CrafterContainer.SLOT_CRAFTOUTPUT`: the ghost grid's result slot.
pub const GHOST_RESULT: usize = 9;
/// `CrafterContainer.SLOT_BUFFER`: the first of the 26 input buffer slots.
pub const BUFFER_IN: usize = 10;
/// `CrafterContainer.BUFFER_SIZE`: 13 x 2 input buffer slots.
pub const BUFFER_IN_SLOTS: usize = 26;
/// `CrafterContainer.SLOT_BUFFEROUT`: the first of the four output buffer slots.
pub const BUFFER_OUT: usize = 36;
/// `CrafterContainer.BUFFEROUT_SIZE`.
pub const BUFFER_OUT_SLOTS: usize = 4;
/// The crafter's container: 10 ghost slots, 26 buffer slots, 4 output slots and the filter
/// module slot (`CrafterContainer.CONTAINER_FACTORY`'s `10 + 26 + 4 + 1`).
pub const CRAFTER_SLOTS: usize = 41;

/// The crafter's three tiers (`CrafterModule.CRAFTER1..3`): one block entity with 2, 4 or 8
/// recipe slots (`CrafterBaseTE.createTier1/2/3`'s `supportedRecipes`, and the jar's own lang
/// lines "can handle up to 2/4/8 recipes at once").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrafterTier {
    /// `rftoolsutility:crafter1`, two recipes.
    Tier1,
    /// `rftoolsutility:crafter2`, four recipes.
    Tier2,
    /// `rftoolsutility:crafter3`, eight recipes.
    Tier3,
}

impl CrafterTier {
    /// The three tiers, low to high: the order the port registers, drops and lists them in.
    pub const ALL: [CrafterTier; 3] = [Self::Tier1, Self::Tier2, Self::Tier3];

    /// The block and item id of the tier (`CrafterModule.CRAFTER1..3`).
    #[must_use]
    pub fn block(self) -> &'static str {
        match self {
            Self::Tier1 => "rftoolsutility:crafter1",
            Self::Tier2 => "rftoolsutility:crafter2",
            Self::Tier3 => "rftoolsutility:crafter3",
        }
    }

    /// The block entity type name the port declares for the tier; the registry namespaces it
    /// to `<mod>:<kind>`, which is the block id ([`Self::block`]).
    #[must_use]
    pub fn kind(self) -> &'static str {
        match self {
            Self::Tier1 => "crafter1",
            Self::Tier2 => "crafter2",
            Self::Tier3 => "crafter3",
        }
    }

    /// The registry id a `minecraft:block_entity_data` payload on the tier's item carries
    /// (`TypedEntityData.loadInto` only loads data that names the block entity it becomes).
    #[must_use]
    pub fn type_id(self) -> &'static str {
        self.block()
    }

    /// `supportedRecipes`: how many recipes the tier keeps, and the count a slot past it is
    /// refused with.
    #[must_use]
    pub fn recipes(self) -> usize {
        match self {
            Self::Tier1 => 2,
            Self::Tier2 => 4,
            Self::Tier3 => 8,
        }
    }

    /// The tier's slot in [`Self::ALL`], which is also its slot in the port's own tables.
    #[must_use]
    pub fn index(self) -> usize {
        match self {
            Self::Tier1 => 0,
            Self::Tier2 => 1,
            Self::Tier3 => 2,
        }
    }

    /// The tier whose block is `block`, if any: the one lookup a drop, a command or a tick
    /// makes from a block id.
    #[must_use]
    pub fn from_block(block: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|tier| tier.block() == block)
    }
}

/// `SpeedMode`: whether the crafter runs one operation a tick or five.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Speed {
    /// One operation per tick, upstream's default (`CrafterData.createDefault`).
    #[default]
    Slow,
    /// `FAST_OPERATIONS` operations per tick.
    Fast,
}

impl Speed {
    /// The name the machine's own setting uses (`SpeedMode.getSerializedName`).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Speed::Slow => "Slow",
            Speed::Fast => "Fast",
        }
    }

    /// The speed a name from the harness names, case-insensitive; none for anything else.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text.to_ascii_lowercase().as_str() {
            "slow" => Some(Speed::Slow),
            "fast" => Some(Speed::Fast),
            _ => None,
        }
    }
}

/// A stack as the crafter's planning sees it: the item id and how many. Components are the
/// glue's business; a crafting operation only moves whole items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stack {
    /// `minecraft:oak_log`, `minecraft:oak_planks`, …
    pub item: String,
    /// How many items the slot holds.
    pub count: u32,
}

/// One cell of the operation's work grid and the buffer slot that feeds it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Take {
    /// The work grid's cell, row-major, 0..9.
    pub cell: usize,
    /// The buffer slot the item comes out of.
    pub slot: usize,
}

/// `CraftMode`: where a crafted result is put (`CrafterBaseTE.placeResult`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CraftMode {
    /// The four output slots the automation takes from (upstream's `Ext` and `ExtC`).
    #[default]
    Ext,
    /// The input buffer, where the next recipe of the cycle can use it: upstream's chained
    /// crafting ("allows recipes to use the crafting results of previous steps").
    Int,
}

impl CraftMode {
    /// The name the machine's own setting uses (`CraftMode.getSerializedName`).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            CraftMode::Ext => "Ext",
            CraftMode::Int => "Int",
        }
    }

    /// The craft mode a name from the harness names, case-insensitive; none for anything else.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text.to_ascii_lowercase().as_str() {
            "ext" => Some(CraftMode::Ext),
            "int" => Some(CraftMode::Int),
            _ => None,
        }
    }
}

/// The buffer slots a crafted result goes to: [`CraftMode::Int`]'s input buffer, which the next
/// recipe of the cycle reads, or [`CraftMode::Ext`]'s four output slots.
#[must_use]
pub fn result_range(mode: CraftMode) -> std::ops::Range<usize> {
    match mode {
        CraftMode::Int => BUFFER_IN..BUFFER_IN + BUFFER_IN_SLOTS,
        CraftMode::Ext => BUFFER_OUT..BUFFER_OUT + BUFFER_OUT_SLOTS,
    }
}

/// How many operations one tick runs: one in slow mode, [`FAST_OPERATIONS`] in fast, and never
/// more than the energy held pays for (`CrafterBaseTE.tickServer`).
#[must_use]
pub fn steps(speed: Speed, energy: i32) -> i32 {
    let wanted = match speed {
        Speed::Slow => 1,
        Speed::Fast => FAST_OPERATIONS,
    };
    wanted.min(energy.max(0) / RF_PER_OPERATION.max(1))
}

/// The plan for one operation: which buffer slot feeds each cell of the 3x3 work grid, in
/// cell order (`CrafterBaseTE.testAndConsume`). A take's `slot` is an index into `buffer` (the
/// caller's input buffer slice), not a world slot. `cells` holds, for each of the nine cells, the
/// item ids the recipe accepts there (an empty list for a cell the recipe leaves empty).
///
/// Upstream splits one item per cell out of the first slot that still holds more than `keep`
/// *after the cells already fed*, so one slot of four planks feeds a recipe that wants four
/// (this is what makes chained crafting work); `keep_one` (`KeepMode.KEEP`) leaves the last item
/// of a slot in place. None when the buffer cannot supply every filled cell.
#[must_use]
pub fn plan(
    cells: &[Vec<String>; 9],
    buffer: &[Option<Stack>],
    keep_one: bool,
) -> Option<Vec<Take>> {
    let keep = u32::from(keep_one);
    let mut left: Vec<u32> = buffer
        .iter()
        .map(|slot| slot.as_ref().map_or(0, |stack| stack.count))
        .collect();
    let mut takes = Vec::new();
    for (cell, accepted) in cells.iter().enumerate() {
        if accepted.is_empty() {
            continue;
        }
        let found = buffer.iter().enumerate().find(|(slot, stack)| {
            let Some(stack) = stack else {
                return false;
            };
            left[*slot] > keep && accepted.contains(&stack.item)
        });
        let (slot, _) = found?;
        left[slot] -= 1;
        takes.push(Take { cell, slot });
    }
    (!takes.is_empty()).then_some(takes)
}

/// Take one item per planned cell out of `buffer` (`testAndConsume`'s `split(1)`), answering
/// whether every planned take happened.
pub fn consume(takes: &[Take], buffer: &mut [Option<Stack>], keep_one: bool) -> bool {
    if takes.is_empty() {
        return false;
    }
    let keep = u32::from(keep_one);
    for take in takes {
        let Some(Some(stack)) = buffer.get_mut(take.slot) else {
            return false;
        };
        if stack.count <= keep {
            return false;
        }
        stack.count -= 1;
        if stack.count == 0 {
            buffer[take.slot] = None;
        }
    }
    true
}

/// The energy a tick's cycles cost: one cycle runs each taught recipe once and costs
/// [`RF_PER_OPERATION`] (`CrafterBaseTE.tickServer`'s `rf * i`, `i` the cycles run).
#[must_use]
pub fn spend(cycles: i32) -> i32 {
    cycles.max(0) * RF_PER_OPERATION
}

/// Where each of `stacks` lands in the output buffer, in order: the index into `out` per stack
/// (the caller adds its own base slot), or none when the buffer has no room for all of them
/// (`InventoryTools.insertItemRanged` over the four output slots). `max_of` is the item's own
/// stack limit.
#[must_use]
pub fn placements(
    stacks: &[Stack],
    out: &[Option<Stack>],
    max_of: &dyn Fn(&str) -> u32,
) -> Option<Vec<usize>> {
    let mut room: Vec<Option<Stack>> = out.to_vec();
    let mut slots = Vec::with_capacity(stacks.len());
    for stack in stacks {
        let limit = max_of(&stack.item).max(1);
        // `InventoryTools.insertItemRanged` merges into a slot that already holds the item
        // first, then takes an empty one; a stack larger than the limit never fits.
        let merge = room.iter().position(|slot| {
            slot.as_ref().is_some_and(|held| {
                held.item == stack.item && held.count.saturating_add(stack.count) <= limit
            })
        });
        let target = match merge {
            Some(slot) => slot,
            None => {
                if stack.count > limit {
                    return None;
                }
                room.iter().position(Option::is_none)?
            }
        };
        match &mut room[target] {
            Some(held) => held.count += stack.count,
            empty @ None => {
                *empty = Some(stack.clone());
            }
        }
        slots.push(target);
    }
    Some(slots)
}

// ---------------------------------------------------------------------------
// Slice 4: the spawner (`SpawnerTileEntity`, `SpawnerConfiguration`, `SpawnerRecipes`)
// ---------------------------------------------------------------------------

/// `SpawnerConfiguration.SPAWNER_MAXENERGY`: the spawner's energy buffer.
pub const SPAWNER_CAPACITY: i32 = 200_000;
/// `SpawnerConfiguration.SPAWNER_RECEIVEPERTICK`: the most one call may fill.
pub const SPAWNER_RECEIVE: i32 = 2_000;

/// One mob's spawner recipe (`SpawnerRecipes.MobData`): the entity it spawns, the RF one
/// spawn costs (`spawnRf`), and the three matter amounts it wants (`item1..3`).
///
/// Upstream keys its table by entity id (`SpawnerRecipes.mobData`, filled from the
/// `rftoolsutility:spawner` recipe files), so the mob a machine spawns is the recipe's own
/// entity: the setting is which recipe the machine holds.
#[derive(Debug, Clone, PartialEq)]
pub struct MobRecipe {
    /// `minecraft:zombie`, the recipe's `entity`.
    pub entity: String,
    /// The recipe's `power`: FE one spawn costs.
    pub power: i32,
    /// The recipe's `item1..3` amounts: the matter one spawn consumes. The matter system
    /// (its three tanks and the beamers that fill them) is not part of this slice; the
    /// numbers are carried and read so the recipe is the machine's own data.
    pub matter: [f32; 3],
}

/// The recipe that spawns `mob` (`SpawnerRecipes.getMobData`, a lookup by entity id).
#[must_use]
pub fn recipe_for<'a>(recipes: &'a [MobRecipe], mob: &str) -> Option<&'a MobRecipe> {
    recipes.iter().find(|recipe| recipe.entity == mob)
}

/// What one spawn leaves in the buffer (`SpawnerTileEntity.tickServer`: the buffer must
/// cover the recipe's `power`, and the spawn is paid for once).
///
/// None when the machine has no cost to charge — a mob with no recipe, or data a hand-made
/// item wrote — so a machine cannot spawn for free; every carried recipe costs 100 FE or
/// more.
#[must_use]
pub fn spawn_once(energy: i32, cost: i32) -> Option<i32> {
    (cost > 0 && energy >= cost).then(|| energy - cost)
}

/// Where a spawn from `pos` facing `facing` puts the mob: the neighbouring block in that
/// direction, centred on x and z (`SpawnerTileEntity.tickServer`: the `facing` block state's
/// `getNormal()` offset, then `moveTo(sx + 0.5, sy, sz + 0.5)`).
///
/// None for a direction the port's four-way facing does not have.
#[must_use]
pub fn spawn_at(pos: [i32; 3], facing: &str) -> Option<[f64; 3]> {
    let [x, y, z] = pos;
    let (x, z) = match facing {
        "north" => (x, z - 1),
        "south" => (x, z + 1),
        "east" => (x + 1, z),
        "west" => (x - 1, z),
        _ => return None,
    };
    Some([f64::from(x) + 0.5, f64::from(y), f64::from(z) + 0.5])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tank(fluid: &str, amount: i32) -> Tank {
        Tank {
            fluid: Some(fluid.to_owned()),
            amount,
        }
    }

    #[test]
    fn an_empty_tank_is_level_zero_and_a_full_one_level_eight() {
        assert_eq!(fill_level(0), 0);
        assert_eq!(fill_level(-1), 0);
        assert_eq!(fill_level(CAPACITY), 8);
        assert_eq!(fill_level(CAPACITY * 2), 8, "the level is capped at 8");
    }

    #[test]
    fn the_level_rises_one_step_per_four_thousand_millibuckets() {
        assert_eq!(fill_level(1), 1);
        assert_eq!(fill_level(3_999), 1);
        assert_eq!(fill_level(4_000), 2);
        assert_eq!(fill_level(8_000), 3);
        assert_eq!(fill_level(12_000), 4);
        assert_eq!(fill_level(31_999), 8);
    }

    #[test]
    fn a_new_tank_is_empty() {
        let tank = Tank::new();
        assert_eq!(tank.amount, 0);
        assert_eq!(tank.fluid, None);
        assert_eq!(tank.level(), 0);
        assert_eq!(tank.room(), CAPACITY);
    }

    #[test]
    fn a_tank_takes_one_fluid_at_a_time() {
        let mut tank = Tank::new();
        assert!(tank.accepts("minecraft:water"));
        assert_eq!(tank.insert("minecraft:water", BUCKET), 0);
        assert_eq!(tank.fluid.as_deref(), Some("minecraft:water"));
        assert_eq!(tank.amount, BUCKET);
        assert!(tank.accepts("minecraft:water"));
        assert!(!tank.accepts("minecraft:lava"));
        assert_eq!(
            tank.insert("minecraft:lava", BUCKET),
            BUCKET,
            "a different fluid is refused whole"
        );
        assert_eq!(tank.amount, BUCKET);
        assert_eq!(tank.fluid.as_deref(), Some("minecraft:water"));
    }

    #[test]
    fn a_fill_is_clamped_to_the_room_left() {
        let mut tank = tank("minecraft:water", 31_500);
        assert_eq!(tank.room(), 500);
        assert_eq!(tank.insert("minecraft:water", BUCKET), 500);
        assert_eq!(tank.amount, CAPACITY);
        assert_eq!(tank.level(), 8);
        assert_eq!(tank.insert("minecraft:water", BUCKET), BUCKET);
        assert_eq!(tank.amount, CAPACITY);
    }

    #[test]
    fn a_fill_of_nothing_changes_nothing() {
        let mut tank = Tank::new();
        assert_eq!(tank.insert("minecraft:water", 0), 0);
        assert_eq!(tank.amount, 0);
        assert_eq!(tank.fluid, None, "a zero fill does not choose a fluid");
        assert_eq!(tank.insert("minecraft:water", -5), -5);
        assert_eq!(tank.amount, 0);
    }

    #[test]
    fn a_drain_gives_what_was_asked_up_to_what_is_held() {
        let mut tank = tank("minecraft:water", 1_500);
        assert_eq!(tank.extract(400), 400);
        assert_eq!(tank.amount, 1_100);
        assert_eq!(tank.extract(5_000), 1_100);
        assert_eq!(tank.amount, 0);
        assert_eq!(tank.fluid, None, "an emptied tank holds no fluid");
        assert_eq!(tank.extract(BUCKET), 0);
    }

    #[test]
    fn a_water_bucket_fills_an_empty_tank() {
        let tank = Tank::new();
        assert_eq!(
            bucket_click("minecraft:water_bucket", &tank),
            Click::Fill {
                fluid: "minecraft:water",
                result: "minecraft:bucket"
            }
        );
    }

    #[test]
    fn a_lava_bucket_fills_an_empty_tank() {
        let tank = Tank::new();
        assert_eq!(
            bucket_click("minecraft:lava_bucket", &tank),
            Click::Fill {
                fluid: "minecraft:lava",
                result: "minecraft:bucket"
            }
        );
    }

    #[test]
    fn an_empty_bucket_drains_one_whole_bucket() {
        let tank = tank("minecraft:water", BUCKET);
        assert_eq!(
            bucket_click("minecraft:bucket", &tank),
            Click::Drain {
                result: "minecraft:water_bucket"
            }
        );
    }

    #[test]
    fn a_tank_holding_lava_drains_a_lava_bucket() {
        let tank = tank("minecraft:lava", 5_000);
        assert_eq!(
            bucket_click("minecraft:bucket", &tank),
            Click::Drain {
                result: "minecraft:lava_bucket"
            }
        );
    }

    #[test]
    fn an_empty_bucket_on_an_empty_tank_does_nothing() {
        assert_eq!(
            bucket_click("minecraft:bucket", &Tank::new()),
            Click::Refused
        );
    }

    #[test]
    fn less_than_a_whole_bucket_cannot_fill_a_bucket() {
        let tank = tank("minecraft:water", BUCKET - 1);
        assert_eq!(bucket_click("minecraft:bucket", &tank), Click::Refused);
    }

    #[test]
    fn a_bucket_that_does_not_fit_is_refused() {
        let tank = tank("minecraft:water", CAPACITY - 500);
        assert_eq!(
            bucket_click("minecraft:water_bucket", &tank),
            Click::Refused
        );
    }

    #[test]
    fn a_bucket_of_another_fluid_is_refused() {
        let tank = tank("minecraft:water", BUCKET);
        assert_eq!(bucket_click("minecraft:lava_bucket", &tank), Click::Refused);
    }

    #[test]
    fn a_full_water_tank_takes_no_more_water() {
        let tank = tank("minecraft:water", CAPACITY);
        assert_eq!(
            bucket_click("minecraft:water_bucket", &tank),
            Click::Refused
        );
    }

    #[test]
    fn a_non_container_is_not_the_tanks_click() {
        assert_eq!(
            bucket_click("minecraft:stone", &Tank::new()),
            Click::NotYours
        );
        assert_eq!(
            bucket_click("minecraft:milk_bucket", &Tank::new()),
            Click::NotYours,
            "milk is no fluid in this game, so its bucket is no fluid container"
        );
        assert_eq!(
            bucket_click("rftoolsutility:tank", &Tank::new()),
            Click::NotYours
        );
    }

    #[test]
    fn a_fill_then_a_drain_leaves_the_tank_as_it_was() {
        let mut tank = Tank::new();
        let Click::Fill { fluid, .. } = bucket_click("minecraft:water_bucket", &tank) else {
            panic!("a water bucket must fill an empty tank");
        };
        assert_eq!(tank.insert(fluid, BUCKET), 0);
        assert_eq!(tank.level(), 1);
        assert_eq!(
            bucket_click("minecraft:bucket", &tank),
            Click::Drain {
                result: "minecraft:water_bucket"
            }
        );
        assert_eq!(tank.extract(BUCKET), BUCKET);
        assert_eq!(tank, Tank::new());
    }

    #[test]
    fn an_empty_tank_carries_nothing_on_its_item() {
        assert_eq!(carried(&Tank::new()), None);
        assert_eq!(
            carried(&Tank {
                fluid: None,
                amount: 4_000
            }),
            None,
            "an amount without a fluid carries nothing"
        );
    }

    #[test]
    fn a_broken_tank_carries_exactly_what_it_holds() {
        let held = tank("minecraft:water", 5_000);
        assert_eq!(carried(&held), Some(&held));
        let full = tank("minecraft:lava", CAPACITY);
        assert_eq!(carried(&full), Some(&full));
        let one = tank("minecraft:water", 1);
        assert_eq!(carried(&one), Some(&one), "a single millibucket travels");
    }

    #[test]
    fn a_carried_tank_is_restored_at_most_the_capacity() {
        assert_eq!(
            clamped(&tank("minecraft:water", 99_999)),
            tank("minecraft:water", CAPACITY),
            "a doctored item is clamped, as FluidTank.setFluid clamps it"
        );
        assert_eq!(
            clamped(&tank("minecraft:water", CAPACITY * 2)),
            tank("minecraft:water", CAPACITY)
        );
        assert_eq!(clamped(&tank("minecraft:water", -5)), Tank::new());
        assert_eq!(
            clamped(&Tank {
                fluid: None,
                amount: 1_000
            }),
            Tank::new(),
            "an amount without a fluid restores empty"
        );
        let held = tank("minecraft:lava", 5_000);
        assert_eq!(clamped(&held), held, "a carried tank comes back unchanged");
    }

    // -- the crafter (slice 3) -------------------------------------------------

    fn stack(item: &str, count: u32) -> Option<Stack> {
        Some(Stack {
            item: item.to_owned(),
            count,
        })
    }

    fn ids(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    fn grid(cells: [Vec<String>; 9]) -> [Vec<String>; 9] {
        cells
    }

    #[test]
    fn each_crafter_tier_keeps_its_own_number_of_recipes() {
        assert_eq!(
            CrafterTier::Tier1.recipes(),
            2,
            "the jar's \"up to 2 recipes\""
        );
        assert_eq!(
            CrafterTier::Tier2.recipes(),
            4,
            "the jar's \"up to 4 recipes\""
        );
        assert_eq!(
            CrafterTier::Tier3.recipes(),
            8,
            "the jar's \"up to 8 recipes\""
        );
    }

    #[test]
    fn a_crafter_tier_is_found_by_its_own_block() {
        assert_eq!(
            CrafterTier::from_block("rftoolsutility:crafter1"),
            Some(CrafterTier::Tier1)
        );
        assert_eq!(
            CrafterTier::from_block("rftoolsutility:crafter2"),
            Some(CrafterTier::Tier2)
        );
        assert_eq!(
            CrafterTier::from_block("rftoolsutility:crafter3"),
            Some(CrafterTier::Tier3)
        );
        assert_eq!(CrafterTier::from_block("rftoolsutility:crafter4"), None);
        assert_eq!(
            CrafterTier::from_block("rftoolsutility:tank"),
            None,
            "another machine is not a crafter"
        );
        assert_eq!(CrafterTier::from_block(""), None);
    }

    #[test]
    fn every_crafter_tier_has_its_own_identity_and_slot() {
        for (slot, tier) in CrafterTier::ALL.iter().enumerate() {
            assert_eq!(tier.index(), slot, "{tier:?} is not in its own table slot");
            assert_eq!(
                tier.block(),
                tier.type_id(),
                "{tier:?}: the carried block_entity_data names the block entity"
            );
            assert_eq!(tier.block(), format!("rftoolsutility:{}", tier.kind()));
            assert_eq!(CrafterTier::from_block(tier.block()), Some(*tier));
        }
        let kinds: Vec<&str> = CrafterTier::ALL.iter().map(|tier| tier.kind()).collect();
        assert_eq!(kinds, ["crafter1", "crafter2", "crafter3"]);
    }

    #[test]
    fn a_crafter_burns_one_operation_in_slow_mode_and_five_in_fast() {
        assert_eq!(steps(Speed::Slow, CRAFTER_CAPACITY), 1);
        assert_eq!(steps(Speed::Fast, CRAFTER_CAPACITY), FAST_OPERATIONS);
    }

    #[test]
    fn energy_pays_for_no_more_operations_than_it_covers() {
        assert_eq!(
            steps(Speed::Fast, 250),
            2,
            "250 RF pays for two 100 RF operations"
        );
        assert_eq!(
            steps(Speed::Fast, 99),
            0,
            "less than one operation runs none"
        );
        assert_eq!(steps(Speed::Slow, 0), 0);
        assert_eq!(steps(Speed::Fast, -500), 0, "a nonsense amount runs none");
        assert_eq!(spend(3), 3 * RF_PER_OPERATION);
        assert_eq!(spend(0), 0);
    }

    #[test]
    fn one_log_crafts_from_the_buffer() {
        // `minecraft:oak_planks` is shapeless: one log, one filled cell.
        let cells = grid([
            ids(&["minecraft:oak_log"]),
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        ]);
        let mut buffer = vec![None; BUFFER_IN_SLOTS];
        buffer[0] = stack("minecraft:oak_log", 3);
        let takes = plan(&cells, &buffer, false).expect("one log feeds the one cell");
        assert_eq!(takes, vec![Take { cell: 0, slot: 0 }]);
        assert!(consume(&takes, &mut buffer, false));
        assert_eq!(buffer[0], stack("minecraft:oak_log", 2));
    }

    #[test]
    fn a_cell_accepts_any_item_of_its_ingredient() {
        // A `#minecraft:planks` cell: four ids, the buffer holds the third.
        let cells = grid([
            ids(&[
                "minecraft:oak_planks",
                "minecraft:birch_planks",
                "minecraft:spruce_planks",
            ]),
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        ]);
        let mut buffer = vec![None; BUFFER_IN_SLOTS];
        buffer[7] = stack("minecraft:spruce_planks", 1);
        let takes = plan(&cells, &buffer, false).expect("spruce is a plank");
        assert_eq!(takes, vec![Take { cell: 0, slot: 7 }]);
    }

    #[test]
    fn one_slot_can_feed_several_cells() {
        // Upstream splits one item per cell out of the same slot while it still holds more than
        // `keep`; two cells of one ingredient are fed by one slot of two.
        let cells = grid([
            ids(&["minecraft:oak_log"]),
            ids(&["minecraft:oak_log"]),
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        ]);
        let mut two = vec![None; BUFFER_IN_SLOTS];
        two[0] = stack("minecraft:oak_log", 2);
        let takes = plan(&cells, &two, false).expect("one slot of two feeds two cells");
        assert_eq!(
            takes,
            vec![Take { cell: 0, slot: 0 }, Take { cell: 1, slot: 0 }]
        );
        assert!(consume(&takes, &mut two, false));
        assert_eq!(two[0], None, "both logs left the slot");
        // One item cannot feed two cells, and keep-one leaves the last one behind.
        let mut one = vec![None; BUFFER_IN_SLOTS];
        one[0] = stack("minecraft:oak_log", 1);
        assert_eq!(plan(&cells, &one, false), None);
        let mut kept = vec![None; BUFFER_IN_SLOTS];
        kept[0] = stack("minecraft:oak_log", 2);
        assert_eq!(
            plan(&cells, &kept, true),
            None,
            "keep-one needs a spare item"
        );
        kept[0] = stack("minecraft:oak_log", 3);
        assert_eq!(plan(&cells, &kept, true).map(|takes| takes.len()), Some(2));
        // A second slot is not needed either way.
        let mut spread = vec![None; BUFFER_IN_SLOTS];
        spread[0] = stack("minecraft:oak_log", 1);
        spread[3] = stack("minecraft:oak_log", 1);
        let takes = plan(&cells, &spread, false).expect("two slots feed two cells");
        assert_eq!(
            takes,
            vec![Take { cell: 0, slot: 0 }, Take { cell: 1, slot: 3 }]
        );
    }

    #[test]
    fn keeping_one_leaves_the_last_item_behind() {
        let cells = grid([
            ids(&["minecraft:oak_log"]),
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        ]);
        let mut one = vec![None; BUFFER_IN_SLOTS];
        one[3] = stack("minecraft:oak_log", 1);
        assert_eq!(
            plan(&cells, &one, true),
            None,
            "keep-one needs more than one item to take one"
        );
        let mut two = vec![None; BUFFER_IN_SLOTS];
        two[3] = stack("minecraft:oak_log", 2);
        let takes = plan(&cells, &two, true).expect("the second log is spare");
        assert!(consume(&takes, &mut two, true));
        assert_eq!(two[3], stack("minecraft:oak_log", 1), "one log is kept");
    }

    #[test]
    fn an_empty_grid_has_no_operation() {
        let cells = grid([
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        ]);
        let mut buffer = vec![None; BUFFER_IN_SLOTS];
        buffer[0] = stack("minecraft:oak_log", 5);
        assert_eq!(plan(&cells, &buffer, false), None);
        assert!(!consume(&[], &mut buffer, false), "an empty take consumes");
        assert_eq!(buffer[0], stack("minecraft:oak_log", 5));
    }

    #[test]
    fn the_output_merges_up_to_the_stack_limit() {
        let out = vec![None, stack("minecraft:oak_planks", 60), None, None];
        let four = Stack {
            item: "minecraft:oak_planks".to_owned(),
            count: 4,
        };
        let slots =
            placements(std::slice::from_ref(&four), &out, &|_| 64).expect("60 + 4 fits a 64 stack");
        assert_eq!(
            slots,
            vec![1],
            "it merges into the slot that already holds planks"
        );
        let pickaxe = Stack {
            item: "minecraft:wooden_pickaxe".to_owned(),
            count: 1,
        };
        let two = vec![pickaxe.clone(), pickaxe];
        assert_eq!(
            placements(&two, &[None, None, None, None], &|_| 1),
            Some(vec![0, 1]),
            "unstackable results take a slot each"
        );
    }

    #[test]
    fn a_full_output_buffer_refuses_the_operation() {
        let out = vec![
            stack("minecraft:wooden_pickaxe", 1),
            stack("minecraft:wooden_sword", 1),
            stack("minecraft:wooden_shovel", 1),
            stack("minecraft:wooden_axe", 1),
        ];
        let hoe = Stack {
            item: "minecraft:wooden_hoe".to_owned(),
            count: 1,
        };
        assert_eq!(placements(&[hoe], &out, &|_| 1), None);
    }

    #[test]
    fn an_internal_result_lands_in_the_input_buffer_where_the_next_recipe_reads_it() {
        let range = result_range(CraftMode::Int);
        assert_eq!(range, BUFFER_IN..BUFFER_IN + BUFFER_IN_SLOTS);
        assert_eq!(
            CraftMode::default(),
            CraftMode::Ext,
            "upstream's CraftingRecipe defaults to Ext"
        );
        assert_eq!(CraftMode::Int.name(), "Int");
        assert_eq!(CraftMode::parse("ext"), Some(CraftMode::Ext));
        assert_eq!(CraftMode::parse("INT"), Some(CraftMode::Int));
        assert_eq!(CraftMode::parse("chain"), None);
    }

    #[test]
    fn an_external_result_lands_in_the_output_slots() {
        assert_eq!(
            result_range(CraftMode::Ext),
            BUFFER_OUT..BUFFER_OUT + BUFFER_OUT_SLOTS
        );
    }

    // -- the spawner (slice 4) -------------------------------------------------

    fn mob(entity: &str, power: i32) -> MobRecipe {
        MobRecipe {
            entity: entity.to_owned(),
            power,
            matter: [0.1, 0.2, 20.0],
        }
    }

    #[test]
    fn a_recipe_is_found_by_the_mob_it_spawns() {
        let zombie = mob("minecraft:zombie", 800);
        let blaze = MobRecipe {
            entity: "minecraft:blaze".to_owned(),
            power: 1_000,
            matter: [0.1, 0.5, 30.0],
        };
        let recipes = [zombie, blaze];
        assert_eq!(
            recipe_for(&recipes, "minecraft:blaze").map(|recipe| recipe.power),
            Some(1_000),
            "the table is keyed by entity id, not by file name"
        );
        assert_eq!(recipe_for(&recipes, "minecraft:zombie"), Some(&recipes[0]));
        assert_eq!(
            recipe_for(&recipes, "minecraft:not_a_mob"),
            None,
            "a mob no recipe names cannot be spawned"
        );
        assert_eq!(recipe_for(&[], "minecraft:zombie"), None);
    }

    #[test]
    fn a_spawn_costs_the_recipes_own_power_once() {
        assert_eq!(spawn_once(800, 800), Some(0), "exact pay is a spawn");
        assert_eq!(spawn_once(5_000, 800), Some(4_200));
        assert_eq!(
            spawn_once(799, 800),
            None,
            "one FE short is no spawn, and nothing is spent"
        );
        assert_eq!(spawn_once(0, 800), None);
        assert_eq!(
            spawn_once(1_000, 1_000),
            Some(0),
            "a blaze costs its own 1,000"
        );
    }

    #[test]
    fn a_machine_with_no_known_cost_spawns_nothing() {
        assert_eq!(
            spawn_once(200_000, 0),
            None,
            "a machine with no recipe cost must not spawn for free"
        );
        assert_eq!(spawn_once(200_000, -1), None);
        assert_eq!(
            spawn_once(0, 0),
            None,
            "an empty buffer with no cost is still no spawn"
        );
    }

    #[test]
    fn a_spawn_lands_on_the_neighbour_the_facing_names() {
        assert_eq!(
            spawn_at([200, 101, 0], "north"),
            Some([200.5, 101.0, -0.5]),
            "north is -z, and x/z are centred as `moveTo` centres them"
        );
        assert_eq!(spawn_at([200, 101, 0], "south"), Some([200.5, 101.0, 1.5]));
        assert_eq!(spawn_at([200, 101, 0], "east"), Some([201.5, 101.0, 0.5]));
        assert_eq!(spawn_at([200, 101, 0], "west"), Some([199.5, 101.0, 0.5]));
        assert_eq!(
            spawn_at([200, 101, 0], "up"),
            None,
            "the port's four-way facing has no up: the block cannot be turned up"
        );
        assert_eq!(spawn_at([200, 101, 0], ""), None);
    }
}
