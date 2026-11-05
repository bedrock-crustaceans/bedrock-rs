//! The ender dragon's goals.

use bedrock_addon_core::schema::{self, GoalMeta, Schema};
use bedrock_addon_core::shared::FloatRange;
use bedrock_addon_core::version::FormatVersion;
use facet::Facet;

use super::check::{Bound, Goal, GoalCheck};
use super::common::ControlFlag;

/// `minecraft:behavior.dragonchargeplayer`: allows this entity to attack a player by charging at them. The player is chosen by the `minecraft:behavior.dragonscanning`. Can only be used by the Ender Dragon.
// `behavior.dragonchargeplayer`: typed from 1.26.0.
#[doc(alias = "minecraft:behavior.dragonchargeplayer")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DragonChargePlayer {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `active_speed`: the speed this entity moves when this behavior has started or while it's active. Default 1; at least 0.
    #[facet(schema::fallback = 1.0)]
    pub active_speed: Option<f32>,
    /// `continue_charge_threshold_time`: if the dragon is outside the `target_zone` for longer than `continue_charge_threshold_time` seconds, the charge is canceled. Default 0.5; greater than 0.
    #[facet(schema::fallback = 0.5)]
    pub continue_charge_threshold_time: Option<f32>,
    /// `flight_speed`: the speed this entity moves while this behavior is not active. Default 0.6; at least 0.
    #[facet(schema::fallback = 0.6)]
    pub flight_speed: Option<f32>,
    /// `target_zone`: minimum and maximum distance, from the target, this entity can use this behavior. Default 1e+01 to 1.5e+02.
    pub target_zone: Option<FloatRange>,
    /// `turn_speed`: the speed at which this entity turns while using this behavior. Default 0.7; at least 0.
    #[facet(schema::fallback = 0.7)]
    pub turn_speed: Option<f32>,
}

impl DragonChargePlayer {
    /// `target_zone`, or its default 1e+01 to 1.5e+02.
    pub fn target_zone(&self) -> (f32, f32) {
        self.target_zone
            .map_or((1.0e1, 1.5e2), |range| range.resolve(1.0e1, 1.5e2))
    }
}

impl Goal for DragonChargePlayer {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.float("active_speed", &mut self.active_speed, Bound::Min(0.0));
        c.float(
            "continue_charge_threshold_time",
            &mut self.continue_charge_threshold_time,
            Bound::Above(0.0),
        );
        c.float("flight_speed", &mut self.flight_speed, Bound::Min(0.0));
        c.range("target_zone", &mut self.target_zone);
        c.float("turn_speed", &mut self.turn_speed, Bound::Min(0.0));
    }
}

/// `minecraft:behavior.dragondeath`: allows the dragon to go out with glory. This controls the Ender Dragon's death animation and can't be used by other mobs.
// `behavior.dragondeath`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.dragondeath")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DragonDeath {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
}

impl Goal for DragonDeath {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.dragonflaming`: allows this entity to use a flame-breath attack. Can only be used by the Ender Dragon.
// `behavior.dragonflaming`: typed from 1.21.100.
#[doc(alias = "minecraft:behavior.dragonflaming")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DragonFlaming {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `cooldown_time`: time (in seconds) between each start of the cycle to roar, then breath flame. Default 1e+01.
    #[facet(schema::fallback = 1.0e1)]
    pub cooldown_time: Option<f32>,
    /// `flame_time`: time (in seconds), after roar, to breath flame. Default 0.5.
    #[facet(schema::fallback = 0.5)]
    pub flame_time: Option<f32>,
    /// `ground_flame_count`: number of ground flame-breath attacks to use before flight-takeoff. Default 4.
    #[facet(schema::fallback = 4)]
    pub ground_flame_count: Option<i32>,
    /// `roar_time`: time (in seconds) to roar, before breathing flame. Default 2.
    #[facet(schema::fallback = 2.0)]
    pub roar_time: Option<f32>,
    /// `smoke_color`: default `#DC00EF`.
    pub smoke_color: Option<String>,
    /// `smoke_radius`: default 5.
    #[facet(schema::fallback = 5.0)]
    pub smoke_radius: Option<f32>,
    /// `smoke_time`: default 1e+01.
    #[facet(schema::fallback = 1.0e1)]
    pub smoke_time: Option<f32>,
}

impl DragonFlaming {
    /// `smoke_color`, or its default `#DC00EF`.
    pub fn smoke_color(&self) -> &str {
        self.smoke_color.as_ref().map_or("#DC00EF", |v| v.as_str())
    }
}

impl Goal for DragonFlaming {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.dragonholdingpattern`: allows the Dragon to fly around in a circle around the center podium. Can only be used by the Ender Dragon.
// `behavior.dragonholdingpattern`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.dragonholdingpattern")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DragonHoldingPattern {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
}

impl Goal for DragonHoldingPattern {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.dragonlanding`: allows the Dragon to stop flying and transition into perching mode. Can only be used by the Ender Dragon.
// `behavior.dragonlanding`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.dragonlanding")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DragonLanding {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
}

impl Goal for DragonLanding {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.dragonscanning`: allows the dragon to look around for a player to attack while in perch mode. Can only be used by the Ender Dragon.
// `behavior.dragonscanning`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.dragonscanning")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DragonScanning {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
}

impl Goal for DragonScanning {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.dragonstrafeplayer`: allows this entity to fly around looking for a player to shoot fireballs at. Can only be used by the Ender Dragon.
// `behavior.dragonstrafeplayer`: typed from 1.26.0.
#[doc(alias = "minecraft:behavior.dragonstrafeplayer")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DragonStrafePlayer {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `active_speed`: the speed this entity moves when this behavior has started or while it's active. Default 1; at least 0.
    #[facet(schema::fallback = 1.0)]
    pub active_speed: Option<f32>,
    /// `fireball_range`: maximum distance of this entity's fireball attack while strafing. Default 64; greater than 0.
    #[facet(schema::fallback = 64.0)]
    pub fireball_range: Option<f32>,
    /// `flight_speed`: the speed this entity moves while this behavior is not active. Default 0.6; at least 0.
    #[facet(schema::fallback = 0.6)]
    pub flight_speed: Option<f32>,
    /// `switch_direction_probability`: percent chance to to switch this entity's strafe direction between clockwise and counterclockwise. Switch direction chance occurs each time a new target is chosen (1.0 = 100%). Default 0.125; 0 to 1.
    #[facet(schema::fallback = 0.125)]
    pub switch_direction_probability: Option<f32>,
    /// `target_in_range_and_in_view_time`: time (in seconds) the target must be in fireball range, and in view \[ie, no solid terrain in-between the target and this entity\], before a fireball can be shot. Default 0.25; greater than 0.
    #[facet(schema::fallback = 0.25)]
    pub target_in_range_and_in_view_time: Option<f32>,
    /// `target_zone`: minimum and maximum distance, from the target, this entity can use this behavior. Default 1e+01 to 1.5e+02.
    pub target_zone: Option<FloatRange>,
    /// `turn_speed`: the speed at which this entity turns while using this behavior. Default 0.7; at least 0.
    #[facet(schema::fallback = 0.7)]
    pub turn_speed: Option<f32>,
    /// `view_angle`: the target must be within `view_angle` degrees of the dragon's current rotation before a fireball can be shot. Default 1e+01; greater than 0.
    #[facet(schema::fallback = 1.0e1)]
    pub view_angle: Option<f32>,
}

impl DragonStrafePlayer {
    /// `target_zone`, or its default 1e+01 to 1.5e+02.
    pub fn target_zone(&self) -> (f32, f32) {
        self.target_zone
            .map_or((1.0e1, 1.5e2), |range| range.resolve(1.0e1, 1.5e2))
    }
}

impl Goal for DragonStrafePlayer {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.float("active_speed", &mut self.active_speed, Bound::Min(0.0));
        c.float(
            "fireball_range",
            &mut self.fireball_range,
            Bound::Above(0.0),
        );
        c.float("flight_speed", &mut self.flight_speed, Bound::Min(0.0));
        c.float(
            "switch_direction_probability",
            &mut self.switch_direction_probability,
            Bound::MinMax(0.0, 1.0),
        );
        c.float(
            "target_in_range_and_in_view_time",
            &mut self.target_in_range_and_in_view_time,
            Bound::Above(0.0),
        );
        c.range("target_zone", &mut self.target_zone);
        c.float("turn_speed", &mut self.turn_speed, Bound::Min(0.0));
        c.float("view_angle", &mut self.view_angle, Bound::Above(0.0));
    }
}

/// `minecraft:behavior.dragontakeoff`: allows the dragon to leave perch mode and go back to flying around. Can only be used by the Ender Dragon.
// `behavior.dragontakeoff`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.dragontakeoff")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DragonTakeoff {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
}

impl Goal for DragonTakeoff {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}
