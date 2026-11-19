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
}
