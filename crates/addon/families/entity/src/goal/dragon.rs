//! The ender dragon's goals.

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{StrictFloat, StrictInt, StrictString};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::shared::FloatRange;
use bedrock_addon_core::version::FormatVersion;
use facet::Facet;

use super::check::{Bound, Check, Goal};
use super::common::ControlFlag;

/// `minecraft:behavior.dragonchargeplayer`: allows this entity to attack a player by charging at them. The player is chosen by the `minecraft:behavior.dragonscanning`. Can only be used by the Ender Dragon.
// `behavior.dragonchargeplayer`: typed from 1.26.0.
#[doc(alias = "minecraft:behavior.dragonchargeplayer")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct DragonChargePlayer {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `active_speed`: the speed this entity moves when this behavior has started or while it's active. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub active_speed: Option<StrictFloat<f32>>,
    /// `continue_charge_threshold_time`: if the dragon is outside the `target_zone` for longer than `continue_charge_threshold_time` seconds, the charge is canceled. Default 0.5; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub continue_charge_threshold_time: Option<StrictFloat<f32>>,
    /// `flight_speed`: the speed this entity moves while this behavior is not active. Default 0.6; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub flight_speed: Option<StrictFloat<f32>>,
    /// `target_zone`: minimum and maximum distance, from the target, this entity can use this behavior. Default 1e+01 to 1.5e+02.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub target_zone: Option<FloatRange>,
    /// `turn_speed`: the speed at which this entity turns while using this behavior. Default 0.7; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub turn_speed: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl DragonChargePlayer {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `active_speed`, or its default 1.
    pub fn active_speed(&self) -> f32 {
        self.active_speed.map_or(1.0, |v| v.0)
    }

    /// `continue_charge_threshold_time`, or its default 0.5.
    pub fn continue_charge_threshold_time(&self) -> f32 {
        self.continue_charge_threshold_time.map_or(0.5, |v| v.0)
    }

    /// `flight_speed`, or its default 0.6.
    pub fn flight_speed(&self) -> f32 {
        self.flight_speed.map_or(0.6, |v| v.0)
    }

    /// `target_zone`, or its default 1e+01 to 1.5e+02.
    pub fn target_zone(&self) -> (f32, f32) {
        self.target_zone
            .map_or((1.0e1, 1.5e2), |range| range.resolve(1.0e1, 1.5e2))
    }

    /// `turn_speed`, or its default 0.7.
    pub fn turn_speed(&self) -> f32 {
        self.turn_speed.map_or(0.7, |v| v.0)
    }
}

impl Goal for DragonChargePlayer {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 0);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
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

impl MolangFields for DragonChargePlayer {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.dragondeath`: allows the dragon to go out with glory. This controls the Ender Dragon's death animation and can't be used by other mobs.
// `behavior.dragondeath`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.dragondeath")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct DragonDeath {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl DragonDeath {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }
}

impl Goal for DragonDeath {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 30);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for DragonDeath {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.dragonflaming`: allows this entity to use a flame-breath attack. Can only be used by the Ender Dragon.
// `behavior.dragonflaming`: typed from 1.21.100.
#[doc(alias = "minecraft:behavior.dragonflaming")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct DragonFlaming {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `cooldown_time`: time (in seconds) between each start of the cycle to roar, then breath flame. Default 1e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_time: Option<StrictFloat<f32>>,
    /// `flame_time`: time (in seconds), after roar, to breath flame. Default 0.5.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub flame_time: Option<StrictFloat<f32>>,
    /// `ground_flame_count`: number of ground flame-breath attacks to use before flight-takeoff. Default 4.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub ground_flame_count: Option<StrictInt<i32>>,
    /// `roar_time`: time (in seconds) to roar, before breathing flame. Default 2.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub roar_time: Option<StrictFloat<f32>>,
    /// `smoke_color`: default `#DC00EF`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub smoke_color: Option<StrictString>,
    /// `smoke_radius`: default 5.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub smoke_radius: Option<StrictFloat<f32>>,
    /// `smoke_time`: default 1e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub smoke_time: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl DragonFlaming {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `cooldown_time`, or its default 1e+01.
    pub fn cooldown_time(&self) -> f32 {
        self.cooldown_time.map_or(1.0e1, |v| v.0)
    }

    /// `flame_time`, or its default 0.5.
    pub fn flame_time(&self) -> f32 {
        self.flame_time.map_or(0.5, |v| v.0)
    }

    /// `ground_flame_count`, or its default 4.
    pub fn ground_flame_count(&self) -> i32 {
        self.ground_flame_count.map_or(4, |v| v.0)
    }

    /// `roar_time`, or its default 2.
    pub fn roar_time(&self) -> f32 {
        self.roar_time.map_or(2.0, |v| v.0)
    }

    /// `smoke_color`, or its default `#DC00EF`.
    pub fn smoke_color(&self) -> &str {
        self.smoke_color
            .as_ref()
            .map_or("#DC00EF", |v| v.0.as_str())
    }

    /// `smoke_radius`, or its default 5.
    pub fn smoke_radius(&self) -> f32 {
        self.smoke_radius.map_or(5.0, |v| v.0)
    }

    /// `smoke_time`, or its default 1e+01.
    pub fn smoke_time(&self) -> f32 {
        self.smoke_time.map_or(1.0e1, |v| v.0)
    }
}

impl Goal for DragonFlaming {
    const TYPED: FormatVersion = FormatVersion::new(1, 21, 100);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for DragonFlaming {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.dragonholdingpattern`: allows the Dragon to fly around in a circle around the center podium. Can only be used by the Ender Dragon.
// `behavior.dragonholdingpattern`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.dragonholdingpattern")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct DragonHoldingPattern {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl DragonHoldingPattern {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }
}

impl Goal for DragonHoldingPattern {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 30);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for DragonHoldingPattern {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.dragonlanding`: allows the Dragon to stop flying and transition into perching mode. Can only be used by the Ender Dragon.
// `behavior.dragonlanding`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.dragonlanding")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct DragonLanding {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl DragonLanding {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }
}

impl Goal for DragonLanding {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 30);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for DragonLanding {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.dragonscanning`: allows the dragon to look around for a player to attack while in perch mode. Can only be used by the Ender Dragon.
// `behavior.dragonscanning`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.dragonscanning")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct DragonScanning {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl DragonScanning {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }
}

impl Goal for DragonScanning {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 30);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for DragonScanning {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.dragonstrafeplayer`: allows this entity to fly around looking for a player to shoot fireballs at. Can only be used by the Ender Dragon.
// `behavior.dragonstrafeplayer`: typed from 1.26.0.
#[doc(alias = "minecraft:behavior.dragonstrafeplayer")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct DragonStrafePlayer {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `active_speed`: the speed this entity moves when this behavior has started or while it's active. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub active_speed: Option<StrictFloat<f32>>,
    /// `fireball_range`: maximum distance of this entity's fireball attack while strafing. Default 64; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub fireball_range: Option<StrictFloat<f32>>,
    /// `flight_speed`: the speed this entity moves while this behavior is not active. Default 0.6; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub flight_speed: Option<StrictFloat<f32>>,
    /// `switch_direction_probability`: percent chance to to switch this entity's strafe direction between clockwise and counterclockwise. Switch direction chance occurs each time a new target is chosen (1.0 = 100%). Default 0.125; 0 to 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub switch_direction_probability: Option<StrictFloat<f32>>,
    /// `target_in_range_and_in_view_time`: time (in seconds) the target must be in fireball range, and in view \[ie, no solid terrain in-between the target and this entity\], before a fireball can be shot. Default 0.25; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub target_in_range_and_in_view_time: Option<StrictFloat<f32>>,
    /// `target_zone`: minimum and maximum distance, from the target, this entity can use this behavior. Default 1e+01 to 1.5e+02.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub target_zone: Option<FloatRange>,
    /// `turn_speed`: the speed at which this entity turns while using this behavior. Default 0.7; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub turn_speed: Option<StrictFloat<f32>>,
    /// `view_angle`: the target must be within `view_angle` degrees of the dragon's current rotation before a fireball can be shot. Default 1e+01; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub view_angle: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl DragonStrafePlayer {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `active_speed`, or its default 1.
    pub fn active_speed(&self) -> f32 {
        self.active_speed.map_or(1.0, |v| v.0)
    }

    /// `fireball_range`, or its default 64.
    pub fn fireball_range(&self) -> f32 {
        self.fireball_range.map_or(64.0, |v| v.0)
    }

    /// `flight_speed`, or its default 0.6.
    pub fn flight_speed(&self) -> f32 {
        self.flight_speed.map_or(0.6, |v| v.0)
    }

    /// `switch_direction_probability`, or its default 0.125.
    pub fn switch_direction_probability(&self) -> f32 {
        self.switch_direction_probability.map_or(0.125, |v| v.0)
    }

    /// `target_in_range_and_in_view_time`, or its default 0.25.
    pub fn target_in_range_and_in_view_time(&self) -> f32 {
        self.target_in_range_and_in_view_time.map_or(0.25, |v| v.0)
    }

    /// `target_zone`, or its default 1e+01 to 1.5e+02.
    pub fn target_zone(&self) -> (f32, f32) {
        self.target_zone
            .map_or((1.0e1, 1.5e2), |range| range.resolve(1.0e1, 1.5e2))
    }

    /// `turn_speed`, or its default 0.7.
    pub fn turn_speed(&self) -> f32 {
        self.turn_speed.map_or(0.7, |v| v.0)
    }

    /// `view_angle`, or its default 1e+01.
    pub fn view_angle(&self) -> f32 {
        self.view_angle.map_or(1.0e1, |v| v.0)
    }
}

impl Goal for DragonStrafePlayer {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 0);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
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

impl MolangFields for DragonStrafePlayer {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.dragontakeoff`: allows the dragon to leave perch mode and go back to flying around. Can only be used by the Ender Dragon.
// `behavior.dragontakeoff`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.dragontakeoff")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct DragonTakeoff {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl DragonTakeoff {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }
}

impl Goal for DragonTakeoff {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 30);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for DragonTakeoff {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}
