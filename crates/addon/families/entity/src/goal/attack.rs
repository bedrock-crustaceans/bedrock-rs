//! Attack goals: melee, ranged and special attacks.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, VanillaText,
};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{StrictBool, StrictFloat, StrictInt, StrictString, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::shared::{EventTrigger, Filter, FloatRange};
use bedrock_addon_core::version::FormatVersion;
use facet::Facet;
use facet_value::VObject;

use super::check::{
    Bound, Check, Deprecated, Goal, V1_26_0, V1_26_40, V1_26_50, deprecated_text, float_of,
};
use super::common::{
    ControlFlag, InRangeMovementMode, LegacyConditions, ParticleColor, SoundEvent, SummonShape,
    SummonTarget, Vec3,
};
use super::convert;
use super::molang::{push_trigger, push_triggers, visit_conditions};

/// `minecraft:behavior.aquatic_charge_attack`: enables an aquatic mob to dash at its target with knockback; includes overshoot and cooldown settings.
///
/// Deprecated members, reported and dropped: `charge_speed_multiplier`.
// `behavior.aquatic_charge_attack`: typed from 1.21.120, redeclared 1.26.0.
#[doc(alias = "minecraft:behavior.aquatic_charge_attack")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct AquaticChargeAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `attack_reach`: horizontal reach grown around the mob's AABB to register a hit. Default 0.05; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_reach: Option<StrictFloat<f32>>,
    /// `charge_cooldown_time`: range of time in seconds to wait before starting another charge. Default 2 to 6.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub charge_cooldown_time: Option<FloatRange>,
    /// `charge_overshoot_distance`: distance beyond the target the mob aims during a charge. Default 1.5; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub charge_overshoot_distance: Option<StrictFloat<f32>>,
    /// `charge_speed`: absolute speed used during the charge attack. Default 0.06; at least 0.05.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub charge_speed: Option<StrictFloat<f32>>,
    /// `knockback_force`: knockback force applied to the target on hit. Default 2; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub knockback_force: Option<StrictFloat<f32>>,
    /// `max_charge_distance`: maximum distance at which the mob attempts a charge. Default 16; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_charge_distance: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl AquaticChargeAttack {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `attack_reach`, or its default 0.05.
    pub fn attack_reach(&self) -> f32 {
        self.attack_reach.map_or(0.05, |v| v.0)
    }

    /// `charge_cooldown_time`, or its default 2 to 6.
    pub fn charge_cooldown_time(&self) -> (f32, f32) {
        self.charge_cooldown_time
            .map_or((2.0, 6.0), |range| range.resolve(2.0, 6.0))
    }

    /// `charge_overshoot_distance`, or its default 1.5.
    pub fn charge_overshoot_distance(&self) -> f32 {
        self.charge_overshoot_distance.map_or(1.5, |v| v.0)
    }

    /// `charge_speed`, or its default 0.06.
    pub fn charge_speed(&self) -> f32 {
        self.charge_speed.map_or(0.06, |v| v.0)
    }

    /// `knockback_force`, or its default 2.
    pub fn knockback_force(&self) -> f32 {
        self.knockback_force.map_or(2.0, |v| v.0)
    }

    /// `max_charge_distance`, or its default 16.
    pub fn max_charge_distance(&self) -> f32 {
        self.max_charge_distance.map_or(16.0, |v| v.0)
    }
}

impl Goal for AquaticChargeAttack {
    const TYPED: FormatVersion = FormatVersion::new(1, 21, 120);

    const DEPRECATED: &'static [Deprecated] = &[(
        "charge_speed_multiplier",
        deprecated_text!("charge_speed_multiplier"),
        FormatVersion::new(1, 26, 0),
    )];

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.float("attack_reach", &mut self.attack_reach, Bound::Min(0.0));
        c.range("charge_cooldown_time", &mut self.charge_cooldown_time);
        c.float(
            "charge_overshoot_distance",
            &mut self.charge_overshoot_distance,
            Bound::Min(0.0),
        );
        c.float("charge_speed", &mut self.charge_speed, Bound::Min(0.05));
        c.float(
            "knockback_force",
            &mut self.knockback_force,
            Bound::Min(0.0),
        );
        c.float(
            "max_charge_distance",
            &mut self.max_charge_distance,
            Bound::Min(0.0),
        );
    }

    fn deprecated(&mut self, name: &str, value: &Value) {
        // copied into `charge_speed` when that is absent.
        if name == "charge_speed_multiplier" && self.charge_speed.is_none() {
            self.charge_speed = float_of(value).map(StrictFloat);
        }
    }

    fn upgrade(&mut self, from: FormatVersion, _: &mut Diagnostics) {
        // below 1.26.0 `charge_speed_multiplier` is the live name.
        if from < V1_26_0 {
            convert::charge_speed_typed(&mut self.charge_speed, &mut self.extra);
        }
    }

    fn lenient_shape(json: &mut VObject, _: &mut Diagnostics) {
        convert::charge_speed(json);
    }
}

impl MolangFields for AquaticChargeAttack {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.charge_attack`: allows this entity to damage a target by using a running attack.
// `behavior.charge_attack`: typed from 1.21.100.
#[doc(alias = "minecraft:behavior.charge_attack")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct ChargeAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `max_distance`: a charge attack cannot start if the entity is farther than this distance to the target. Default 3.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_distance: Option<StrictFloat<f32>>,
    /// `min_distance`: a charge attack cannot start if the entity is closer than this distance to the target. Default 2.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_distance: Option<StrictFloat<f32>>,
    /// `sound_event`: default `charge`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sound_event: Option<SoundEvent>,
    /// `speed_multiplier`: modifies the entity's speed when charging toward the target. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// `success_rate`: percent chance this entity will start a charge attack, if not already attacking (1.0 = 100%). Default 0.1428.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub success_rate: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl ChargeAttack {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `max_distance`, or its default 3.
    pub fn max_distance(&self) -> f32 {
        self.max_distance.map_or(3.0, |v| v.0)
    }

    /// `min_distance`, or its default 2.
    pub fn min_distance(&self) -> f32 {
        self.min_distance.map_or(2.0, |v| v.0)
    }

    /// `sound_event`, or its default `charge`.
    pub fn sound_event(&self) -> &str {
        self.sound_event.as_ref().map_or("charge", SoundEvent::name)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }

    /// `success_rate`, or its default 0.1428.
    pub fn success_rate(&self) -> f32 {
        self.success_rate.map_or(0.1428, |v| v.0)
    }
}

impl Goal for ChargeAttack {
    const TYPED: FormatVersion = FormatVersion::new(1, 21, 100);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.sound("sound_event", self.sound_event.as_ref());
    }
}

impl MolangFields for ChargeAttack {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.delayed_attack`: allows an entity to attack, while also delaying the damage-dealt until a specific time in the attack animation.
///
/// Deprecated members, reported and dropped: `cooldown_time`.
// `behavior.delayed_attack`: typed from 1.26.0, redeclared 1.26.40.
#[doc(alias = "minecraft:behavior.delayed_attack")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct DelayedAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `attack_duration`: the entity's attack animation will play out over this duration (in seconds). Also controls attack cooldown. Default 0.75; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_duration: Option<StrictFloat<f32>>,
    /// `attack_once`: allows the mob to perform this melee attack behavior only once during its lifetime. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_once: Option<StrictBool>,
    /// `attack_types`: defines the entity types this entity will attack. Default `""`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_types: Option<StrictString>,
    /// `can_spread_on_fire`: allows the mob, if on fire and empty handed, to ignite its target upon a successful attack. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub can_spread_on_fire: Option<StrictBool>,
    /// `hit_delay_pct`: the percentage into the attack animation to apply the damage of the attack (1.0 = 100%). Default 0.5; 0 to 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub hit_delay_pct: Option<StrictFloat<f32>>,
    /// `inner_boundary_time_increase`: time, in seconds, added to the attack path recalculation interval when the target is beyond `path_inner_boundary`. Default 0.25; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub inner_boundary_time_increase: Option<StrictFloat<f32>>,
    /// `max_path_time`: maximum base time, in seconds, before recalculating a new attack path to the target (before increases are applied). Default 0.55; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_path_time: Option<StrictFloat<f32>>,
    /// `melee_fov`: field of view, in degrees, used by the hard-coded sensing component to detect a valid attack target. Default 9e+01; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub melee_fov: Option<StrictFloat<f32>>,
    /// `min_path_time`: minimum base time, in seconds, before recalculating a new attack path to the target (before increases are applied). Default 0.2; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_path_time: Option<StrictFloat<f32>>,
    /// `on_attack`: defines the event to trigger when this entity successfully attacks.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_attack: Option<EventTrigger>,
    /// `on_kill`: defines the event to trigger when this entity kills the target.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_kill: Option<EventTrigger>,
    /// `outer_boundary_time_increase`: time, in seconds, added to the attack path recalculation interval when the target is beyond `path_outer_boundary`. Default 0.5; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub outer_boundary_time_increase: Option<StrictFloat<f32>>,
    /// `path_fail_time_increase`: time, in seconds, added to the attack path recalculation interval when the mob cannot move along the current path. Default 0.75; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub path_fail_time_increase: Option<StrictFloat<f32>>,
    /// `path_inner_boundary`: distance at which to increase attack path recalculation by `inner_boundary_time_increase`. Default 16; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub path_inner_boundary: Option<StrictFloat<f32>>,
    /// `path_outer_boundary`: distance at which to increase attack path recalculation by `outer_boundary_time_increase`. Default 32; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub path_outer_boundary: Option<StrictFloat<f32>>,
    /// `random_stop_interval`: defines a 1-in-N chance for the mob to stop its current attack, where N equals `random_stop_interval`. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub random_stop_interval: Option<StrictInt<i32>>,
    /// `reach_multiplier`: used with the base size of the entity to determine minimum target-distance before trying to deal attack damage. Default 1.5; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub reach_multiplier: Option<StrictFloat<f32>>,
    /// `require_complete_path`: specifies whether a full navigation path from the mob to the target is required. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub require_complete_path: Option<StrictBool>,
    /// `sound_event`: default `attack.strong`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sound_event: Option<SoundEvent>,
    /// `speed_multiplier`: multiplier applied to the mob's movement speed when moving toward its target. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// `track_target`: allows the mob to track its target even if it lacks a hard-coded sensing component. Default `true`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub track_target: Option<StrictBool>,
    /// `x_max_rotation`: maximum rotation, in degrees, on the X-axis while the mob is trying to look at its target. Default 3e+01; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub x_max_rotation: Option<StrictFloat<f32>>,
    /// `y_max_head_rotation`: maximum rotation, in degrees, on the Y-axis while the mob is trying to look at its target. Default 3e+01; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub y_max_head_rotation: Option<StrictFloat<f32>>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl DelayedAttack {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `attack_duration`, or its default 0.75.
    pub fn attack_duration(&self) -> f32 {
        self.attack_duration.map_or(0.75, |v| v.0)
    }

    /// `attack_once`, or its default `false`.
    pub fn attack_once(&self) -> bool {
        self.attack_once.is_some_and(|v| v.0)
    }

    /// `attack_types`, or its default `""`.
    pub fn attack_types(&self) -> &str {
        self.attack_types.as_ref().map_or("", |v| v.0.as_str())
    }

    /// `can_spread_on_fire`, or its default `false`.
    pub fn can_spread_on_fire(&self) -> bool {
        self.can_spread_on_fire.is_some_and(|v| v.0)
    }

    /// `hit_delay_pct`, or its default 0.5.
    pub fn hit_delay_pct(&self) -> f32 {
        self.hit_delay_pct.map_or(0.5, |v| v.0)
    }

    /// `inner_boundary_time_increase`, or its default 0.25.
    pub fn inner_boundary_time_increase(&self) -> f32 {
        self.inner_boundary_time_increase.map_or(0.25, |v| v.0)
    }

    /// `max_path_time`, or its default 0.55.
    pub fn max_path_time(&self) -> f32 {
        self.max_path_time.map_or(0.55, |v| v.0)
    }

    /// `melee_fov`, or its default 9e+01.
    pub fn melee_fov(&self) -> f32 {
        self.melee_fov.map_or(9.0e1, |v| v.0)
    }

    /// `min_path_time`, or its default 0.2.
    pub fn min_path_time(&self) -> f32 {
        self.min_path_time.map_or(0.2, |v| v.0)
    }

    /// `outer_boundary_time_increase`, or its default 0.5.
    pub fn outer_boundary_time_increase(&self) -> f32 {
        self.outer_boundary_time_increase.map_or(0.5, |v| v.0)
    }

    /// `path_fail_time_increase`, or its default 0.75.
    pub fn path_fail_time_increase(&self) -> f32 {
        self.path_fail_time_increase.map_or(0.75, |v| v.0)
    }

    /// `path_inner_boundary`, or its default 16.
    pub fn path_inner_boundary(&self) -> f32 {
        self.path_inner_boundary.map_or(16.0, |v| v.0)
    }

    /// `path_outer_boundary`, or its default 32.
    pub fn path_outer_boundary(&self) -> f32 {
        self.path_outer_boundary.map_or(32.0, |v| v.0)
    }

    /// `random_stop_interval`, or its default 0.
    pub fn random_stop_interval(&self) -> i32 {
        self.random_stop_interval.map_or(0, |v| v.0)
    }

    /// `reach_multiplier`, or its default 1.5.
    pub fn reach_multiplier(&self) -> f32 {
        self.reach_multiplier.map_or(1.5, |v| v.0)
    }

    /// `require_complete_path`, or its default `false`.
    pub fn require_complete_path(&self) -> bool {
        self.require_complete_path.is_some_and(|v| v.0)
    }

    /// `sound_event`, or its default `attack.strong`.
    pub fn sound_event(&self) -> &str {
        self.sound_event
            .as_ref()
            .map_or("attack.strong", SoundEvent::name)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }

    /// `track_target`, or its default `true`.
    pub fn track_target(&self) -> bool {
        self.track_target.is_none_or(|v| v.0)
    }

    /// `x_max_rotation`, or its default 3e+01.
    pub fn x_max_rotation(&self) -> f32 {
        self.x_max_rotation.map_or(3.0e1, |v| v.0)
    }

    /// `y_max_head_rotation`, or its default 3e+01.
    pub fn y_max_head_rotation(&self) -> f32 {
        self.y_max_head_rotation.map_or(3.0e1, |v| v.0)
    }
}

impl Goal for DelayedAttack {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 0);

    const DEPRECATED: &'static [Deprecated] = &[(
        "cooldown_time",
        deprecated_text!("cooldown_time"),
        FormatVersion::new(1, 26, 0),
    )];

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.float(
            "attack_duration",
            &mut self.attack_duration,
            Bound::Min(0.0),
        );
        c.float(
            "hit_delay_pct",
            &mut self.hit_delay_pct,
            Bound::MinMax(0.0, 1.0),
        );
        c.float(
            "inner_boundary_time_increase",
            &mut self.inner_boundary_time_increase,
            Bound::Min(0.0),
        );
        c.float("max_path_time", &mut self.max_path_time, Bound::Min(0.0));
        c.float("melee_fov", &mut self.melee_fov, Bound::Min(0.0));
        c.float("min_path_time", &mut self.min_path_time, Bound::Min(0.0));
        c.trigger("on_attack", self.on_attack.as_ref());
        c.trigger("on_kill", self.on_kill.as_ref());
        c.float(
            "outer_boundary_time_increase",
            &mut self.outer_boundary_time_increase,
            Bound::Min(0.0),
        );
        c.float(
            "path_fail_time_increase",
            &mut self.path_fail_time_increase,
            Bound::Min(0.0),
        );
        c.float(
            "path_inner_boundary",
            &mut self.path_inner_boundary,
            Bound::Min(0.0),
        );
        c.float(
            "path_outer_boundary",
            &mut self.path_outer_boundary,
            Bound::Min(0.0),
        );
        c.int(
            "random_stop_interval",
            &mut self.random_stop_interval,
            Bound::Min(0),
        );
        c.float(
            "reach_multiplier",
            &mut self.reach_multiplier,
            Bound::Min(0.0),
        );
        c.sound("sound_event", self.sound_event.as_ref());
        c.float(
            "speed_multiplier",
            &mut self.speed_multiplier,
            Bound::Min(0.0),
        );
        c.float("x_max_rotation", &mut self.x_max_rotation, Bound::Min(0.0));
        c.float(
            "y_max_head_rotation",
            &mut self.y_max_head_rotation,
            Bound::Min(0.0),
        );
    }

    fn upgrade(&mut self, from: FormatVersion, _: &mut Diagnostics) {
        if from < V1_26_40 {
            convert::fold_on_kill_fields(&mut self.on_attack, &mut self.on_kill);
        }
    }

    fn lenient_shape(json: &mut VObject, d: &mut Diagnostics) {
        convert::fold_on_kill(json, d);
    }
}

impl MolangFields for DelayedAttack {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(&mut triggers, "on_attack", self.on_attack.as_ref());
        push_trigger(&mut triggers, "on_kill", self.on_kill.as_ref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.eat_mob`: allows the entity to eat a specified Mob.
// `behavior.eat_mob`: typed from 1.21.100.
#[doc(alias = "minecraft:behavior.eat_mob")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct EatMob {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `eat_animation_time`: sets the time in seconds the eat animation should play for. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub eat_animation_time: Option<StrictFloat<f32>>,
    /// `eat_mob_sound`: sets the sound that should play when eating a mob. Default `""`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub eat_mob_sound: Option<SoundEvent>,
    /// `loot_table`: the loot table for loot to be dropped when eating a mob. Default `""`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub loot_table: Option<StrictString>,
    /// `pull_in_force`: sets the force which the mob-to-be-eaten is pulled towards the eating mob. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub pull_in_force: Option<StrictFloat<f32>>,
    /// `reach_mob_distance`: sets the desired distance to be reached before eating the mob. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub reach_mob_distance: Option<StrictFloat<f32>>,
    /// `run_speed`: sets the entity's speed when running toward the target. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub run_speed: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl EatMob {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `eat_animation_time`, or its default 1.
    pub fn eat_animation_time(&self) -> f32 {
        self.eat_animation_time.map_or(1.0, |v| v.0)
    }

    /// `eat_mob_sound`, or its default `""`.
    pub fn eat_mob_sound(&self) -> &str {
        self.eat_mob_sound.as_ref().map_or("", SoundEvent::name)
    }

    /// `loot_table`, or its default `""`.
    pub fn loot_table(&self) -> &str {
        self.loot_table.as_ref().map_or("", |v| v.0.as_str())
    }

    /// `pull_in_force`, or its default 1.
    pub fn pull_in_force(&self) -> f32 {
        self.pull_in_force.map_or(1.0, |v| v.0)
    }

    /// `reach_mob_distance`, or its default 1.
    pub fn reach_mob_distance(&self) -> f32 {
        self.reach_mob_distance.map_or(1.0, |v| v.0)
    }

    /// `run_speed`, or its default 1.
    pub fn run_speed(&self) -> f32 {
        self.run_speed.map_or(1.0, |v| v.0)
    }
}

impl Goal for EatMob {
    const TYPED: FormatVersion = FormatVersion::new(1, 21, 100);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.sound("eat_mob_sound", self.eat_mob_sound.as_ref());
    }
}

impl MolangFields for EatMob {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.fire_at_target`: allows an entity to attack by firing a shot with a delay. Anchor and offset parameters of this component overrides the anchor and offset from projectile component.
// `behavior.fire_at_target`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.fire_at_target")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct FireAtTarget {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `attack_cooldown`: the cooldown time in seconds before this goal can be used again. Default 0.5; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_cooldown: Option<StrictFloat<f32>>,
    /// `attack_range`: target needs to be within this range for the attack to happen. Default 0 to 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_range: Option<FloatRange>,
    /// `filters`: conditions that need to be met for the behavior to start.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub filters: Option<Filter>,
    /// `max_head_rotation_x`: maximum head rotation (in degrees), on the X-axis, that this entity can apply while trying to look at the target. Default 3e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_head_rotation_x: Option<StrictFloat<f32>>,
    /// `max_head_rotation_y`: maximum head rotation (in degrees), on the Y-axis, that this entity can apply while trying to look at the target. Default 3e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_head_rotation_y: Option<StrictFloat<f32>>,
    /// `owner_anchor`: entity anchor for the projectile spawn location. Default 2.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub owner_anchor: Option<StrictInt<i32>>,
    /// `owner_offset`: offset vector from the `owner_anchor`. Default `[0, 0, 0]`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub owner_offset: Option<Vec3>,
    /// `post_shoot_delay`: time in seconds between firing the projectile and ending the goal. Default 0.2; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub post_shoot_delay: Option<StrictFloat<f32>>,
    /// `pre_shoot_delay`: time in seconds before firing the projectile. Default 0.75; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub pre_shoot_delay: Option<StrictFloat<f32>>,
    /// `projectile_def`: actor definition to use as projectile for the ranged attack. The actor must be a projectile. This field is required for the goal to be usable. Default `""`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub projectile_def: Option<StrictString>,
    /// `ranged_fov`: field of view (in degrees) when using sensing to detect a target for attack. Default 9e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub ranged_fov: Option<StrictFloat<f32>>,
    /// `target_anchor`: entity anchor for projectile target. Default 2.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub target_anchor: Option<StrictInt<i32>>,
    /// `target_offset`: offset vector from the `target_anchor`. Default `[0, 0, 0]`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub target_offset: Option<Vec3>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl FireAtTarget {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `attack_cooldown`, or its default 0.5.
    pub fn attack_cooldown(&self) -> f32 {
        self.attack_cooldown.map_or(0.5, |v| v.0)
    }

    /// `attack_range`, or its default 0 to 0.
    pub fn attack_range(&self) -> (f32, f32) {
        self.attack_range
            .map_or((0.0, 0.0), |range| range.resolve(0.0, 0.0))
    }

    /// `max_head_rotation_x`, or its default 3e+01.
    pub fn max_head_rotation_x(&self) -> f32 {
        self.max_head_rotation_x.map_or(3.0e1, |v| v.0)
    }

    /// `max_head_rotation_y`, or its default 3e+01.
    pub fn max_head_rotation_y(&self) -> f32 {
        self.max_head_rotation_y.map_or(3.0e1, |v| v.0)
    }

    /// `owner_anchor`, or its default 2.
    pub fn owner_anchor(&self) -> i32 {
        self.owner_anchor.map_or(2, |v| v.0)
    }

    /// `owner_offset`, or its default `[0, 0, 0]`.
    pub fn owner_offset(&self) -> [f32; 3] {
        self.owner_offset
            .as_ref()
            .map_or([0.0, 0.0, 0.0], |v| [v.x, v.y, v.z])
    }

    /// `post_shoot_delay`, or its default 0.2.
    pub fn post_shoot_delay(&self) -> f32 {
        self.post_shoot_delay.map_or(0.2, |v| v.0)
    }

    /// `pre_shoot_delay`, or its default 0.75.
    pub fn pre_shoot_delay(&self) -> f32 {
        self.pre_shoot_delay.map_or(0.75, |v| v.0)
    }

    /// `projectile_def`, or its default `""`.
    pub fn projectile_def(&self) -> &str {
        self.projectile_def.as_ref().map_or("", |v| v.0.as_str())
    }

    /// `ranged_fov`, or its default 9e+01.
    pub fn ranged_fov(&self) -> f32 {
        self.ranged_fov.map_or(9.0e1, |v| v.0)
    }

    /// `target_anchor`, or its default 2.
    pub fn target_anchor(&self) -> i32 {
        self.target_anchor.map_or(2, |v| v.0)
    }

    /// `target_offset`, or its default `[0, 0, 0]`.
    pub fn target_offset(&self) -> [f32; 3] {
        self.target_offset
            .as_ref()
            .map_or([0.0, 0.0, 0.0], |v| [v.x, v.y, v.z])
    }
}

impl Goal for FireAtTarget {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 30);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.float(
            "attack_cooldown",
            &mut self.attack_cooldown,
            Bound::Min(0.0),
        );
        c.range("attack_range", &mut self.attack_range);
        c.filter("filters", self.filters.as_ref());
        c.vector("owner_offset", self.owner_offset.as_ref().map(|v| &v.extra));
        c.float(
            "post_shoot_delay",
            &mut self.post_shoot_delay,
            Bound::Min(0.0),
        );
        c.float(
            "pre_shoot_delay",
            &mut self.pre_shoot_delay,
            Bound::Min(0.0),
        );
        c.vector(
            "target_offset",
            self.target_offset.as_ref().map(|v| &v.extra),
        );
    }
}

impl MolangFields for FireAtTarget {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.guardian_attack`: allows this entity to use a laser beam attack. Can only be used by Guardians and Elder Guardians.
// `behavior.guardian_attack`: typed from 1.26.0.
#[doc(alias = "minecraft:behavior.guardian_attack")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct GuardianAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `elder_extra_magic_damage`: amount of additional damage dealt from an elder guardian's magic attack. Default 2; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub elder_extra_magic_damage: Option<StrictInt<i32>>,
    /// `hard_mode_extra_magic_damage`: in hard difficulty, amount of additional damage dealt from a guardian's magic attack. Default 2; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub hard_mode_extra_magic_damage: Option<StrictInt<i32>>,
    /// `magic_damage`: amount of damage dealt from a guardian's magic attack. Magic attack damage is added to the guardian's base attack damage. Default 1; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub magic_damage: Option<StrictInt<i32>>,
    /// `min_distance`: guardian attack behavior stops if the target is closer than this distance (doesn't apply to elders). Default 3; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_distance: Option<StrictFloat<f32>>,
    /// `sound_delay_time`: time (in seconds) to wait after starting an attack before playing the guardian attack sound. Default 0.5; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sound_delay_time: Option<StrictFloat<f32>>,
    /// `x_max_rotation`: maximum rotation (in degrees), on the X-axis, this entity can rotate while trying to look at the target. Default 9e+01; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub x_max_rotation: Option<StrictFloat<f32>>,
    /// `y_max_head_rotation`: maximum rotation (in degrees), on the Y-axis, this entity can rotate its head while trying to look at the target. Default 9e+01; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub y_max_head_rotation: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl GuardianAttack {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `elder_extra_magic_damage`, or its default 2.
    pub fn elder_extra_magic_damage(&self) -> i32 {
        self.elder_extra_magic_damage.map_or(2, |v| v.0)
    }

    /// `hard_mode_extra_magic_damage`, or its default 2.
    pub fn hard_mode_extra_magic_damage(&self) -> i32 {
        self.hard_mode_extra_magic_damage.map_or(2, |v| v.0)
    }

    /// `magic_damage`, or its default 1.
    pub fn magic_damage(&self) -> i32 {
        self.magic_damage.map_or(1, |v| v.0)
    }

    /// `min_distance`, or its default 3.
    pub fn min_distance(&self) -> f32 {
        self.min_distance.map_or(3.0, |v| v.0)
    }

    /// `sound_delay_time`, or its default 0.5.
    pub fn sound_delay_time(&self) -> f32 {
        self.sound_delay_time.map_or(0.5, |v| v.0)
    }

    /// `x_max_rotation`, or its default 9e+01.
    pub fn x_max_rotation(&self) -> f32 {
        self.x_max_rotation.map_or(9.0e1, |v| v.0)
    }

    /// `y_max_head_rotation`, or its default 9e+01.
    pub fn y_max_head_rotation(&self) -> f32 {
        self.y_max_head_rotation.map_or(9.0e1, |v| v.0)
    }
}

impl Goal for GuardianAttack {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 0);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.int(
            "elder_extra_magic_damage",
            &mut self.elder_extra_magic_damage,
            Bound::Above(0),
        );
        c.int(
            "hard_mode_extra_magic_damage",
            &mut self.hard_mode_extra_magic_damage,
            Bound::Above(0),
        );
        c.int("magic_damage", &mut self.magic_damage, Bound::Above(0));
        c.float("min_distance", &mut self.min_distance, Bound::Min(0.0));
        c.float(
            "sound_delay_time",
            &mut self.sound_delay_time,
            Bound::Above(0.0),
        );
        c.float(
            "x_max_rotation",
            &mut self.x_max_rotation,
            Bound::Above(0.0),
        );
        c.float(
            "y_max_head_rotation",
            &mut self.y_max_head_rotation,
            Bound::Above(0.0),
        );
    }
}

impl MolangFields for GuardianAttack {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.jump_around_target`: allows an entity to jump around a target.
// `behavior.jump_around_target`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.jump_around_target")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct JumpAroundTarget {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `check_collision`: enables collision checks when calculating the jump. Setting `check_collision` to true may affect performance and should be used with care. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub check_collision: Option<StrictBool>,
    /// `entity_bounding_box_scale`: scaling temporarily applied to the entity's AABB bounds when jumping. A smaller bounding box reduces the risk of collisions during the jump. When `check_collision` is true it also increases the chance of being able to jump when close to obstacles. Default 0.7.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub entity_bounding_box_scale: Option<StrictFloat<f32>>,
    /// `filters`: conditions that need to be met for the behavior to start.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub filters: Option<Filter>,
    /// `jump_angles`: the jump angles in float degrees that are allowed when performing the jump. The order in which the angles are chosen is randomized.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub jump_angles: Option<Vec<StrictFloat<f32>>>,
    /// `jump_cooldown_duration`: the time in seconds to spend in cooldown before this goal can be used again. Default 0.5.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub jump_cooldown_duration: Option<StrictFloat<f32>>,
    /// `jump_cooldown_when_hurt_duration`: the time in seconds to spend in cooldown after being hurt before this goal can be used again. Default 0.1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub jump_cooldown_when_hurt_duration: Option<StrictFloat<f32>>,
    /// `landing_distance_from_target`: the range deciding how close to and how far away from the target the landing position can be when jumping. Default 4 to 8.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub landing_distance_from_target: Option<FloatRange>,
    /// `landing_position_spread_degrees`: this angle (in degrees) is used for controlling the spread when picking a landing position behind the target. A zero spread angle means the landing position will be straight behind the target with no variance. A 90 degree spread angle means the landing position can be up to 45 degrees to the left and to the right of the position straight behind the target's view direction. Default 90.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub landing_position_spread_degrees: Option<StrictInt<i32>>,
    /// `last_hurt_duration`: if the entity was hurt within these last seconds, the `jump_cooldown_when_hurt_duration` will be used instead of `jump_cooldown_duration`. Default 2.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub last_hurt_duration: Option<StrictFloat<f32>>,
    /// `line_of_sight_obstruction_height_ignore`: if the entity's line of sight towards its target is obstructed by an obstacle with a height below this number, the obstacle will be ignored, and the goal will try to find a valid landing position. Default 4.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub line_of_sight_obstruction_height_ignore: Option<StrictInt<i32>>,
    /// `max_jump_velocity`: maximum velocity a jump can be performed at. Default 1.4.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_jump_velocity: Option<StrictFloat<f32>>,
    /// `prepare_jump_duration`: the time in seconds to spend preparing for the jump. Default 0.5.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub prepare_jump_duration: Option<StrictFloat<f32>>,
    /// `required_vertical_space`: the number of blocks above the entity's head that has to be air for this goal to be usable. Default 4.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub required_vertical_space: Option<StrictInt<i32>>,
    /// `snap_to_surface_block_range`: the number of blocks above and below from the jump target position that will be checked to find a surface to land on. Default 10.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub snap_to_surface_block_range: Option<StrictInt<i32>>,
    /// `valid_distance_to_target`: target needs to be within this range for the jump to happen. Default 4 to 2e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub valid_distance_to_target: Option<FloatRange>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl JumpAroundTarget {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `check_collision`, or its default `false`.
    pub fn check_collision(&self) -> bool {
        self.check_collision.is_some_and(|v| v.0)
    }

    /// `entity_bounding_box_scale`, or its default 0.7.
    pub fn entity_bounding_box_scale(&self) -> f32 {
        self.entity_bounding_box_scale.map_or(0.7, |v| v.0)
    }

    /// `jump_cooldown_duration`, or its default 0.5.
    pub fn jump_cooldown_duration(&self) -> f32 {
        self.jump_cooldown_duration.map_or(0.5, |v| v.0)
    }

    /// `jump_cooldown_when_hurt_duration`, or its default 0.1.
    pub fn jump_cooldown_when_hurt_duration(&self) -> f32 {
        self.jump_cooldown_when_hurt_duration.map_or(0.1, |v| v.0)
    }

    /// `landing_distance_from_target`, or its default 4 to 8.
    pub fn landing_distance_from_target(&self) -> (f32, f32) {
        self.landing_distance_from_target
            .map_or((4.0, 8.0), |range| range.resolve(4.0, 8.0))
    }

    /// `landing_position_spread_degrees`, or its default 90.
    pub fn landing_position_spread_degrees(&self) -> i32 {
        self.landing_position_spread_degrees.map_or(90, |v| v.0)
    }

    /// `last_hurt_duration`, or its default 2.
    pub fn last_hurt_duration(&self) -> f32 {
        self.last_hurt_duration.map_or(2.0, |v| v.0)
    }

    /// `line_of_sight_obstruction_height_ignore`, or its default 4.
    pub fn line_of_sight_obstruction_height_ignore(&self) -> i32 {
        self.line_of_sight_obstruction_height_ignore
            .map_or(4, |v| v.0)
    }

    /// `max_jump_velocity`, or its default 1.4.
    pub fn max_jump_velocity(&self) -> f32 {
        self.max_jump_velocity.map_or(1.4, |v| v.0)
    }

    /// `prepare_jump_duration`, or its default 0.5.
    pub fn prepare_jump_duration(&self) -> f32 {
        self.prepare_jump_duration.map_or(0.5, |v| v.0)
    }

    /// `required_vertical_space`, or its default 4.
    pub fn required_vertical_space(&self) -> i32 {
        self.required_vertical_space.map_or(4, |v| v.0)
    }

    /// `snap_to_surface_block_range`, or its default 10.
    pub fn snap_to_surface_block_range(&self) -> i32 {
        self.snap_to_surface_block_range.map_or(10, |v| v.0)
    }

    /// `valid_distance_to_target`, or its default 4 to 2e+01.
    pub fn valid_distance_to_target(&self) -> (f32, f32) {
        self.valid_distance_to_target
            .map_or((4.0, 2.0e1), |range| range.resolve(4.0, 2.0e1))
    }
}

impl Goal for JumpAroundTarget {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 20);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.filter("filters", self.filters.as_ref());
        c.range(
            "landing_distance_from_target",
            &mut self.landing_distance_from_target,
        );
        c.range(
            "valid_distance_to_target",
            &mut self.valid_distance_to_target,
        );
    }
}

impl MolangFields for JumpAroundTarget {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.knockback_roar`: allows the mob to perform a damaging knockback that affects all nearby entities.
// `behavior.knockback_roar`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.knockback_roar")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct KnockbackRoar {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `attack_time`: the delay after which the knockback occurs (in seconds). Default 0.5.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_time: Option<StrictFloat<f32>>,
    /// `cooldown_time`: time (in seconds) the mob has to wait before using the goal again. Default 0.1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_time: Option<StrictFloat<f32>>,
    /// `damage_filters`: the list of conditions another entity must meet to be a valid target to apply damage to.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub damage_filters: Option<Filter>,
    /// `duration`: the max duration of the roar (in seconds). Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub duration: Option<StrictFloat<f32>>,
    /// `knockback_damage`: the damage dealt by the knockback roar. Default 6.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub knockback_damage: Option<StrictInt<i32>>,
    /// `knockback_filters`: the list of conditions another entity must meet to be a valid target to apply knockback to.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub knockback_filters: Option<Filter>,
    /// `knockback_height_cap`: the maximum height for vertical knockback. Default 0.4.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub knockback_height_cap: Option<StrictFloat<f32>>,
    /// `knockback_horizontal_strength`: the strength of the horizontal knockback. Default 4.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub knockback_horizontal_strength: Option<StrictInt<i32>>,
    /// `knockback_range`: the radius (in blocks) of the knockback effect. Default 4.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub knockback_range: Option<StrictInt<i32>>,
    /// `knockback_vertical_strength`: the strength of the vertical knockback. Default 4.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub knockback_vertical_strength: Option<StrictInt<i32>>,
    /// `on_roar_end`: event that is triggered when the roar ends.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_roar_end: Option<EventTrigger>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl KnockbackRoar {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `attack_time`, or its default 0.5.
    pub fn attack_time(&self) -> f32 {
        self.attack_time.map_or(0.5, |v| v.0)
    }

    /// `cooldown_time`, or its default 0.1.
    pub fn cooldown_time(&self) -> f32 {
        self.cooldown_time.map_or(0.1, |v| v.0)
    }

    /// `duration`, or its default 1.
    pub fn duration(&self) -> f32 {
        self.duration.map_or(1.0, |v| v.0)
    }

    /// `knockback_damage`, or its default 6.
    pub fn knockback_damage(&self) -> i32 {
        self.knockback_damage.map_or(6, |v| v.0)
    }

    /// `knockback_height_cap`, or its default 0.4.
    pub fn knockback_height_cap(&self) -> f32 {
        self.knockback_height_cap.map_or(0.4, |v| v.0)
    }

    /// `knockback_horizontal_strength`, or its default 4.
    pub fn knockback_horizontal_strength(&self) -> i32 {
        self.knockback_horizontal_strength.map_or(4, |v| v.0)
    }

    /// `knockback_range`, or its default 4.
    pub fn knockback_range(&self) -> i32 {
        self.knockback_range.map_or(4, |v| v.0)
    }

    /// `knockback_vertical_strength`, or its default 4.
    pub fn knockback_vertical_strength(&self) -> i32 {
        self.knockback_vertical_strength.map_or(4, |v| v.0)
    }
}

impl Goal for KnockbackRoar {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 40);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.filter("damage_filters", self.damage_filters.as_ref());
        c.filter("knockback_filters", self.knockback_filters.as_ref());
        c.trigger("on_roar_end", self.on_roar_end.as_ref());
    }
}

impl MolangFields for KnockbackRoar {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(&mut triggers, "on_roar_end", self.on_roar_end.as_ref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.leap_at_target`: allows monsters to jump at and attack their target. Can only be used by hostile mobs.
// `behavior.leap_at_target`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.leap_at_target")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct LeapAtTarget {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `must_be_on_ground`: if true, the mob will only jump at its target if its on the ground. Setting it to false will allow it to jump even if its already in the air. Default `true`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub must_be_on_ground: Option<StrictBool>,
    /// `set_persistent`: allows the actor to be set to persist upon targeting a player. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub set_persistent: Option<StrictBool>,
    /// `yd`: the height in blocks the mob jumps when leaping at its target. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub yd: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl LeapAtTarget {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `must_be_on_ground`, or its default `true`.
    pub fn must_be_on_ground(&self) -> bool {
        self.must_be_on_ground.is_none_or(|v| v.0)
    }

    /// `set_persistent`, or its default `false`.
    pub fn set_persistent(&self) -> bool {
        self.set_persistent.is_some_and(|v| v.0)
    }

    /// `yd`, or its default 0.
    pub fn yd(&self) -> f32 {
        self.yd.map_or(0.0, |v| v.0)
    }
}

impl Goal for LeapAtTarget {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 40);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for LeapAtTarget {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.melee_attack`: allows an entity to deal damage through a melee attack.
// `behavior.melee_attack`: typed from 1.26.0, redeclared 1.26.40.
#[doc(alias = "minecraft:behavior.melee_attack")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct MeleeAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `attack_once`: allows the mob to perform this melee attack behavior only once during its lifetime. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_once: Option<StrictBool>,
    /// `attack_types`: defines the entity types this entity will attack. Default `""`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_types: Option<StrictString>,
    /// `can_spread_on_fire`: allows the mob, if on fire and empty handed, to ignite its target upon a successful attack. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub can_spread_on_fire: Option<StrictBool>,
    /// `cooldown_time`: cooldown time, in seconds, between consecutive attacks. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_time: Option<StrictFloat<f32>>,
    /// `inner_boundary_time_increase`: time, in seconds, added to the attack path recalculation interval when the target is beyond `path_inner_boundary`. Default 0.25; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub inner_boundary_time_increase: Option<StrictFloat<f32>>,
    /// `max_path_time`: maximum base time, in seconds, before recalculating a new attack path to the target (before increases are applied). Default 0.55; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_path_time: Option<StrictFloat<f32>>,
    /// `melee_fov`: field of view, in degrees, used by the hard-coded sensing component to detect a valid attack target. Default 9e+01; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub melee_fov: Option<StrictFloat<f32>>,
    /// `min_path_time`: minimum base time, in seconds, before recalculating a new attack path to the target (before increases are applied). Default 0.2; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_path_time: Option<StrictFloat<f32>>,
    /// `on_attack`: defines the event to trigger when this entity successfully attacks.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_attack: Option<EventTrigger>,
    /// `on_kill`: defines the event to trigger when this entity kills the target.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_kill: Option<EventTrigger>,
    /// `outer_boundary_time_increase`: time, in seconds, added to the attack path recalculation interval when the target is beyond `path_outer_boundary`. Default 0.5; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub outer_boundary_time_increase: Option<StrictFloat<f32>>,
    /// `path_fail_time_increase`: time, in seconds, added to the attack path recalculation interval when the mob cannot move along the current path. Default 0.75; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub path_fail_time_increase: Option<StrictFloat<f32>>,
    /// `path_inner_boundary`: distance at which to increase attack path recalculation by `inner_boundary_time_increase`. Default 16; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub path_inner_boundary: Option<StrictFloat<f32>>,
    /// `path_outer_boundary`: distance at which to increase attack path recalculation by `outer_boundary_time_increase`. Default 32; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub path_outer_boundary: Option<StrictFloat<f32>>,
    /// `random_stop_interval`: defines a 1-in-N chance for the mob to stop its current attack, where N equals `random_stop_interval`. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub random_stop_interval: Option<StrictInt<i32>>,
    /// `reach_multiplier`: used with the base size of the entity to determine minimum target-distance before trying to deal attack damage. Default 2; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub reach_multiplier: Option<StrictFloat<f32>>,
    /// `require_complete_path`: specifies whether a full navigation path from the mob to the target is required. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub require_complete_path: Option<StrictBool>,
    /// `speed_multiplier`: multiplier applied to the mob's movement speed when moving toward its target. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// `track_target`: allows the mob to track its target even if it lacks a hard-coded sensing component. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub track_target: Option<StrictBool>,
    /// `x_max_rotation`: maximum rotation, in degrees, on the X-axis while the mob is trying to look at its target. Default 3e+01; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub x_max_rotation: Option<StrictFloat<f32>>,
    /// `y_max_head_rotation`: maximum rotation, in degrees, on the Y-axis while the mob is trying to look at its target. Default 3e+01; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub y_max_head_rotation: Option<StrictFloat<f32>>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl MeleeAttack {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `attack_once`, or its default `false`.
    pub fn attack_once(&self) -> bool {
        self.attack_once.is_some_and(|v| v.0)
    }

    /// `attack_types`, or its default `""`.
    pub fn attack_types(&self) -> &str {
        self.attack_types.as_ref().map_or("", |v| v.0.as_str())
    }

    /// `can_spread_on_fire`, or its default `false`.
    pub fn can_spread_on_fire(&self) -> bool {
        self.can_spread_on_fire.is_some_and(|v| v.0)
    }

    /// `cooldown_time`, or its default 1.
    pub fn cooldown_time(&self) -> f32 {
        self.cooldown_time.map_or(1.0, |v| v.0)
    }

    /// `inner_boundary_time_increase`, or its default 0.25.
    pub fn inner_boundary_time_increase(&self) -> f32 {
        self.inner_boundary_time_increase.map_or(0.25, |v| v.0)
    }

    /// `max_path_time`, or its default 0.55.
    pub fn max_path_time(&self) -> f32 {
        self.max_path_time.map_or(0.55, |v| v.0)
    }

    /// `melee_fov`, or its default 9e+01.
    pub fn melee_fov(&self) -> f32 {
        self.melee_fov.map_or(9.0e1, |v| v.0)
    }

    /// `min_path_time`, or its default 0.2.
    pub fn min_path_time(&self) -> f32 {
        self.min_path_time.map_or(0.2, |v| v.0)
    }

    /// `outer_boundary_time_increase`, or its default 0.5.
    pub fn outer_boundary_time_increase(&self) -> f32 {
        self.outer_boundary_time_increase.map_or(0.5, |v| v.0)
    }

    /// `path_fail_time_increase`, or its default 0.75.
    pub fn path_fail_time_increase(&self) -> f32 {
        self.path_fail_time_increase.map_or(0.75, |v| v.0)
    }

    /// `path_inner_boundary`, or its default 16.
    pub fn path_inner_boundary(&self) -> f32 {
        self.path_inner_boundary.map_or(16.0, |v| v.0)
    }

    /// `path_outer_boundary`, or its default 32.
    pub fn path_outer_boundary(&self) -> f32 {
        self.path_outer_boundary.map_or(32.0, |v| v.0)
    }

    /// `random_stop_interval`, or its default 0.
    pub fn random_stop_interval(&self) -> i32 {
        self.random_stop_interval.map_or(0, |v| v.0)
    }

    /// `reach_multiplier`, or its default 2.
    pub fn reach_multiplier(&self) -> f32 {
        self.reach_multiplier.map_or(2.0, |v| v.0)
    }

    /// `require_complete_path`, or its default `false`.
    pub fn require_complete_path(&self) -> bool {
        self.require_complete_path.is_some_and(|v| v.0)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }

    /// `track_target`, or its default `false`.
    pub fn track_target(&self) -> bool {
        self.track_target.is_some_and(|v| v.0)
    }

    /// `x_max_rotation`, or its default 3e+01.
    pub fn x_max_rotation(&self) -> f32 {
        self.x_max_rotation.map_or(3.0e1, |v| v.0)
    }

    /// `y_max_head_rotation`, or its default 3e+01.
    pub fn y_max_head_rotation(&self) -> f32 {
        self.y_max_head_rotation.map_or(3.0e1, |v| v.0)
    }
}

impl Goal for MeleeAttack {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 0);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.float("cooldown_time", &mut self.cooldown_time, Bound::Min(0.0));
        c.float(
            "inner_boundary_time_increase",
            &mut self.inner_boundary_time_increase,
            Bound::Min(0.0),
        );
        c.float("max_path_time", &mut self.max_path_time, Bound::Min(0.0));
        c.float("melee_fov", &mut self.melee_fov, Bound::Min(0.0));
        c.float("min_path_time", &mut self.min_path_time, Bound::Min(0.0));
        c.trigger("on_attack", self.on_attack.as_ref());
        c.trigger("on_kill", self.on_kill.as_ref());
        c.float(
            "outer_boundary_time_increase",
            &mut self.outer_boundary_time_increase,
            Bound::Min(0.0),
        );
        c.float(
            "path_fail_time_increase",
            &mut self.path_fail_time_increase,
            Bound::Min(0.0),
        );
        c.float(
            "path_inner_boundary",
            &mut self.path_inner_boundary,
            Bound::Min(0.0),
        );
        c.float(
            "path_outer_boundary",
            &mut self.path_outer_boundary,
            Bound::Min(0.0),
        );
        c.int(
            "random_stop_interval",
            &mut self.random_stop_interval,
            Bound::Min(0),
        );
        c.float(
            "reach_multiplier",
            &mut self.reach_multiplier,
            Bound::Min(0.0),
        );
        c.float(
            "speed_multiplier",
            &mut self.speed_multiplier,
            Bound::Min(0.0),
        );
        c.float("x_max_rotation", &mut self.x_max_rotation, Bound::Min(0.0));
        c.float(
            "y_max_head_rotation",
            &mut self.y_max_head_rotation,
            Bound::Min(0.0),
        );
    }

    fn upgrade(&mut self, from: FormatVersion, _: &mut Diagnostics) {
        if from < V1_26_40 {
            convert::fold_on_kill_fields(&mut self.on_attack, &mut self.on_kill);
        }
    }

    fn lenient_shape(json: &mut VObject, d: &mut Diagnostics) {
        convert::fold_on_kill(json, d);
    }
}

impl MolangFields for MeleeAttack {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(&mut triggers, "on_attack", self.on_attack.as_ref());
        push_trigger(&mut triggers, "on_kill", self.on_kill.as_ref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.melee_box_attack`: allows an entity to deal damage through a melee attack with reach calculations based on bounding boxes.
// `behavior.melee_box_attack`: typed from 1.26.0, redeclared 1.26.40.
#[doc(alias = "minecraft:behavior.melee_box_attack")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct MeleeBoxAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `attack_once`: allows the mob to perform this melee attack behavior only once during its lifetime. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_once: Option<StrictBool>,
    /// `attack_types`: defines the entity types this entity will attack. Default `""`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_types: Option<StrictString>,
    /// `can_spread_on_fire`: allows the mob, if on fire and empty handed, to ignite its target upon a successful attack. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub can_spread_on_fire: Option<StrictBool>,
    /// `cooldown_time`: cooldown time, in seconds, between consecutive attacks. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_time: Option<StrictFloat<f32>>,
    /// `horizontal_reach`: the attack reach of the mob will be a box with the size of the mobs bounds increased by this value in all horizontal directions. Default 0.8; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub horizontal_reach: Option<StrictFloat<f32>>,
    /// `inner_boundary_time_increase`: time, in seconds, added to the attack path recalculation interval when the target is beyond `path_inner_boundary`. Default 0.25; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub inner_boundary_time_increase: Option<StrictFloat<f32>>,
    /// `max_path_time`: maximum base time, in seconds, before recalculating a new attack path to the target (before increases are applied). Default 0.55; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_path_time: Option<StrictFloat<f32>>,
    /// `melee_fov`: field of view, in degrees, used by the hard-coded sensing component to detect a valid attack target. Default 9e+01; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub melee_fov: Option<StrictFloat<f32>>,
    /// `min_path_time`: minimum base time, in seconds, before recalculating a new attack path to the target (before increases are applied). Default 0.2; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_path_time: Option<StrictFloat<f32>>,
    /// `on_attack`: defines the event to trigger when this entity successfully attacks.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_attack: Option<EventTrigger>,
    /// `on_kill`: defines the event to trigger when this entity kills the target.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_kill: Option<EventTrigger>,
    /// `outer_boundary_time_increase`: time, in seconds, added to the attack path recalculation interval when the target is beyond `path_outer_boundary`. Default 0.5; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub outer_boundary_time_increase: Option<StrictFloat<f32>>,
    /// `path_fail_time_increase`: time, in seconds, added to the attack path recalculation interval when the mob cannot move along the current path. Default 0.75; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub path_fail_time_increase: Option<StrictFloat<f32>>,
    /// `path_inner_boundary`: distance at which to increase attack path recalculation by `inner_boundary_time_increase`. Default 16; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub path_inner_boundary: Option<StrictFloat<f32>>,
    /// `path_outer_boundary`: distance at which to increase attack path recalculation by `outer_boundary_time_increase`. Default 32; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub path_outer_boundary: Option<StrictFloat<f32>>,
    /// `random_stop_interval`: defines a 1-in-N chance for the mob to stop its current attack, where N equals `random_stop_interval`. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub random_stop_interval: Option<StrictInt<i32>>,
    /// `require_complete_path`: specifies whether a full navigation path from the mob to the target is required. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub require_complete_path: Option<StrictBool>,
    /// `speed_multiplier`: multiplier applied to the mob's movement speed when moving toward its target. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// `track_target`: allows the mob to track its target even if it lacks a hard-coded sensing component. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub track_target: Option<StrictBool>,
    /// `x_max_rotation`: maximum rotation, in degrees, on the X-axis while the mob is trying to look at its target. Default 3e+01; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub x_max_rotation: Option<StrictFloat<f32>>,
    /// `y_max_head_rotation`: maximum rotation, in degrees, on the Y-axis while the mob is trying to look at its target. Default 3e+01; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub y_max_head_rotation: Option<StrictFloat<f32>>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl MeleeBoxAttack {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `attack_once`, or its default `false`.
    pub fn attack_once(&self) -> bool {
        self.attack_once.is_some_and(|v| v.0)
    }

    /// `attack_types`, or its default `""`.
    pub fn attack_types(&self) -> &str {
        self.attack_types.as_ref().map_or("", |v| v.0.as_str())
    }

    /// `can_spread_on_fire`, or its default `false`.
    pub fn can_spread_on_fire(&self) -> bool {
        self.can_spread_on_fire.is_some_and(|v| v.0)
    }

    /// `cooldown_time`, or its default 1.
    pub fn cooldown_time(&self) -> f32 {
        self.cooldown_time.map_or(1.0, |v| v.0)
    }

    /// `horizontal_reach`, or its default 0.8.
    pub fn horizontal_reach(&self) -> f32 {
        self.horizontal_reach.map_or(0.8, |v| v.0)
    }

    /// `inner_boundary_time_increase`, or its default 0.25.
    pub fn inner_boundary_time_increase(&self) -> f32 {
        self.inner_boundary_time_increase.map_or(0.25, |v| v.0)
    }

    /// `max_path_time`, or its default 0.55.
    pub fn max_path_time(&self) -> f32 {
        self.max_path_time.map_or(0.55, |v| v.0)
    }

    /// `melee_fov`, or its default 9e+01.
    pub fn melee_fov(&self) -> f32 {
        self.melee_fov.map_or(9.0e1, |v| v.0)
    }

    /// `min_path_time`, or its default 0.2.
    pub fn min_path_time(&self) -> f32 {
        self.min_path_time.map_or(0.2, |v| v.0)
    }

    /// `outer_boundary_time_increase`, or its default 0.5.
    pub fn outer_boundary_time_increase(&self) -> f32 {
        self.outer_boundary_time_increase.map_or(0.5, |v| v.0)
    }

    /// `path_fail_time_increase`, or its default 0.75.
    pub fn path_fail_time_increase(&self) -> f32 {
        self.path_fail_time_increase.map_or(0.75, |v| v.0)
    }

    /// `path_inner_boundary`, or its default 16.
    pub fn path_inner_boundary(&self) -> f32 {
        self.path_inner_boundary.map_or(16.0, |v| v.0)
    }

    /// `path_outer_boundary`, or its default 32.
    pub fn path_outer_boundary(&self) -> f32 {
        self.path_outer_boundary.map_or(32.0, |v| v.0)
    }

    /// `random_stop_interval`, or its default 0.
    pub fn random_stop_interval(&self) -> i32 {
        self.random_stop_interval.map_or(0, |v| v.0)
    }

    /// `require_complete_path`, or its default `false`.
    pub fn require_complete_path(&self) -> bool {
        self.require_complete_path.is_some_and(|v| v.0)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }

    /// `track_target`, or its default `false`.
    pub fn track_target(&self) -> bool {
        self.track_target.is_some_and(|v| v.0)
    }

    /// `x_max_rotation`, or its default 3e+01.
    pub fn x_max_rotation(&self) -> f32 {
        self.x_max_rotation.map_or(3.0e1, |v| v.0)
    }

    /// `y_max_head_rotation`, or its default 3e+01.
    pub fn y_max_head_rotation(&self) -> f32 {
        self.y_max_head_rotation.map_or(3.0e1, |v| v.0)
    }
}

impl Goal for MeleeBoxAttack {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 0);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.float("cooldown_time", &mut self.cooldown_time, Bound::Min(0.0));
        c.float(
            "horizontal_reach",
            &mut self.horizontal_reach,
            Bound::Min(0.0),
        );
        c.float(
            "inner_boundary_time_increase",
            &mut self.inner_boundary_time_increase,
            Bound::Min(0.0),
        );
        c.float("max_path_time", &mut self.max_path_time, Bound::Min(0.0));
        c.float("melee_fov", &mut self.melee_fov, Bound::Min(0.0));
        c.float("min_path_time", &mut self.min_path_time, Bound::Min(0.0));
        c.trigger("on_attack", self.on_attack.as_ref());
        c.trigger("on_kill", self.on_kill.as_ref());
        c.float(
            "outer_boundary_time_increase",
            &mut self.outer_boundary_time_increase,
            Bound::Min(0.0),
        );
        c.float(
            "path_fail_time_increase",
            &mut self.path_fail_time_increase,
            Bound::Min(0.0),
        );
        c.float(
            "path_inner_boundary",
            &mut self.path_inner_boundary,
            Bound::Min(0.0),
        );
        c.float(
            "path_outer_boundary",
            &mut self.path_outer_boundary,
            Bound::Min(0.0),
        );
        c.int(
            "random_stop_interval",
            &mut self.random_stop_interval,
            Bound::Min(0),
        );
        c.float(
            "speed_multiplier",
            &mut self.speed_multiplier,
            Bound::Min(0.0),
        );
        c.float("x_max_rotation", &mut self.x_max_rotation, Bound::Min(0.0));
        c.float(
            "y_max_head_rotation",
            &mut self.y_max_head_rotation,
            Bound::Min(0.0),
        );
    }

    fn upgrade(&mut self, from: FormatVersion, _: &mut Diagnostics) {
        if from < V1_26_40 {
            convert::fold_on_kill_fields(&mut self.on_attack, &mut self.on_kill);
        }
    }

    fn lenient_shape(json: &mut VObject, d: &mut Diagnostics) {
        convert::fold_on_kill(json, d);
    }
}

impl MolangFields for MeleeBoxAttack {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(&mut triggers, "on_attack", self.on_attack.as_ref());
        push_trigger(&mut triggers, "on_kill", self.on_kill.as_ref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.ocelotattack`: controls specific attack behavior for Ocelots.
// `behavior.ocelotattack`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.ocelotattack")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Ocelotattack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `cooldown_time`: time (in seconds) between attacks. Default 1; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_time: Option<StrictFloat<f32>>,
    /// `max_distance`: max distance from the target, this entity will use this attack behavior. Default 15; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_distance: Option<StrictFloat<f32>>,
    /// `max_sneak_range`: max distance from the target, this entity starts sneaking. Default 15; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_sneak_range: Option<StrictFloat<f32>>,
    /// `max_sprint_range`: max distance from the target, this entity starts sprinting (sprinting takes priority over sneaking). Default 4; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_sprint_range: Option<StrictFloat<f32>>,
    /// `reach_multiplier`: used with the base size of the entity to determine minimum target-distance before trying to deal attack damage. Default 2; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub reach_multiplier: Option<StrictFloat<f32>>,
    /// `sneak_speed_multiplier`: modifies the attacking entity's movement speed while sneaking. Default 0.6; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sneak_speed_multiplier: Option<StrictFloat<f32>>,
    /// `sprint_speed_multiplier`: modifies the attacking entity's movement speed while sprinting. Default 1.33; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sprint_speed_multiplier: Option<StrictFloat<f32>>,
    /// `walk_speed_multiplier`: modifies the attacking entity's movement speed when not sneaking or sprinting, but still within attack range. Default 0.8; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub walk_speed_multiplier: Option<StrictFloat<f32>>,
    /// `x_max_rotation`: maximum rotation (in degrees), on the X-axis, this entity can rotate while trying to look at the target. Default 3e+01; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub x_max_rotation: Option<StrictFloat<f32>>,
    /// `y_max_head_rotation`: maximum rotation (in degrees), on the Y-axis, this entity's head can rotate while trying to look at the target. Default 3e+01; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub y_max_head_rotation: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Ocelotattack {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `cooldown_time`, or its default 1.
    pub fn cooldown_time(&self) -> f32 {
        self.cooldown_time.map_or(1.0, |v| v.0)
    }

    /// `max_distance`, or its default 15.
    pub fn max_distance(&self) -> f32 {
        self.max_distance.map_or(15.0, |v| v.0)
    }

    /// `max_sneak_range`, or its default 15.
    pub fn max_sneak_range(&self) -> f32 {
        self.max_sneak_range.map_or(15.0, |v| v.0)
    }

    /// `max_sprint_range`, or its default 4.
    pub fn max_sprint_range(&self) -> f32 {
        self.max_sprint_range.map_or(4.0, |v| v.0)
    }

    /// `reach_multiplier`, or its default 2.
    pub fn reach_multiplier(&self) -> f32 {
        self.reach_multiplier.map_or(2.0, |v| v.0)
    }

    /// `sneak_speed_multiplier`, or its default 0.6.
    pub fn sneak_speed_multiplier(&self) -> f32 {
        self.sneak_speed_multiplier.map_or(0.6, |v| v.0)
    }

    /// `sprint_speed_multiplier`, or its default 1.33.
    pub fn sprint_speed_multiplier(&self) -> f32 {
        self.sprint_speed_multiplier.map_or(1.33, |v| v.0)
    }

    /// `walk_speed_multiplier`, or its default 0.8.
    pub fn walk_speed_multiplier(&self) -> f32 {
        self.walk_speed_multiplier.map_or(0.8, |v| v.0)
    }

    /// `x_max_rotation`, or its default 3e+01.
    pub fn x_max_rotation(&self) -> f32 {
        self.x_max_rotation.map_or(3.0e1, |v| v.0)
    }

    /// `y_max_head_rotation`, or its default 3e+01.
    pub fn y_max_head_rotation(&self) -> f32 {
        self.y_max_head_rotation.map_or(3.0e1, |v| v.0)
    }
}

impl Goal for Ocelotattack {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 10);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.float("cooldown_time", &mut self.cooldown_time, Bound::Above(0.0));
        c.float("max_distance", &mut self.max_distance, Bound::Above(0.0));
        c.float(
            "max_sneak_range",
            &mut self.max_sneak_range,
            Bound::Above(0.0),
        );
        c.float(
            "max_sprint_range",
            &mut self.max_sprint_range,
            Bound::Above(0.0),
        );
        c.float(
            "reach_multiplier",
            &mut self.reach_multiplier,
            Bound::Above(0.0),
        );
        c.float(
            "sneak_speed_multiplier",
            &mut self.sneak_speed_multiplier,
            Bound::Above(0.0),
        );
        c.float(
            "sprint_speed_multiplier",
            &mut self.sprint_speed_multiplier,
            Bound::Above(0.0),
        );
        c.float(
            "walk_speed_multiplier",
            &mut self.walk_speed_multiplier,
            Bound::Above(0.0),
        );
        c.float(
            "x_max_rotation",
            &mut self.x_max_rotation,
            Bound::Above(0.0),
        );
        c.float(
            "y_max_head_rotation",
            &mut self.y_max_head_rotation,
            Bound::Above(0.0),
        );
    }
}

impl MolangFields for Ocelotattack {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.ram_attack`: allows this entity to damage a target by using a running attack.
// `behavior.ram_attack`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.ram_attack")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct RamAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `baby_knockback_modifier`: the modifier to knockback that babies have. Default 0.33333334.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub baby_knockback_modifier: Option<StrictFloat<f32>>,
    /// `cooldown_range`: minimum and maximum cooldown time-range (positive, in seconds) between each attempted ram attack. Default 1e+01 to 2e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_range: Option<FloatRange>,
    /// `knockback_force`: the force of the knockback of the ram attack. Default 5.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub knockback_force: Option<StrictFloat<f32>>,
    /// `knockback_height`: the height of the knockback of the ram attack. Default 0.1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub knockback_height: Option<StrictFloat<f32>>,
    /// `min_ram_distance`: the minimum distance at which the mob can start a ram attack. Default 4.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_ram_distance: Option<StrictInt<i32>>,
    /// `on_start`: the event to trigger when attacking.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_start: Option<Vec<EventTrigger>>,
    /// `pre_ram_sound`: the sound to play when an entity is about to perform a ram attack. Default `undefined`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub pre_ram_sound: Option<SoundEvent>,
    /// `ram_distance`: the distance at which the mob start to run with ram speed. Default 7.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub ram_distance: Option<StrictInt<i32>>,
    /// `ram_impact_sound`: the sound to play when an entity is impacting on a ram attack. Default `undefined`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub ram_impact_sound: Option<SoundEvent>,
    /// `ram_speed`: sets the entity's speed when charging toward the target. Default 2.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub ram_speed: Option<StrictFloat<f32>>,
    /// `run_speed`: sets the entity's speed when running toward the target. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub run_speed: Option<StrictFloat<f32>>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl RamAttack {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `baby_knockback_modifier`, or its default 0.33333334.
    pub fn baby_knockback_modifier(&self) -> f32 {
        self.baby_knockback_modifier.map_or(0.333_333_34, |v| v.0)
    }

    /// `cooldown_range`, or its default 1e+01 to 2e+01.
    pub fn cooldown_range(&self) -> (f32, f32) {
        self.cooldown_range
            .map_or((1.0e1, 2.0e1), |range| range.resolve(1.0e1, 2.0e1))
    }

    /// `knockback_force`, or its default 5.
    pub fn knockback_force(&self) -> f32 {
        self.knockback_force.map_or(5.0, |v| v.0)
    }

    /// `knockback_height`, or its default 0.1.
    pub fn knockback_height(&self) -> f32 {
        self.knockback_height.map_or(0.1, |v| v.0)
    }

    /// `min_ram_distance`, or its default 4.
    pub fn min_ram_distance(&self) -> i32 {
        self.min_ram_distance.map_or(4, |v| v.0)
    }

    /// `pre_ram_sound`, or its default `undefined`.
    pub fn pre_ram_sound(&self) -> &str {
        self.pre_ram_sound
            .as_ref()
            .map_or("undefined", SoundEvent::name)
    }

    /// `ram_distance`, or its default 7.
    pub fn ram_distance(&self) -> i32 {
        self.ram_distance.map_or(7, |v| v.0)
    }

    /// `ram_impact_sound`, or its default `undefined`.
    pub fn ram_impact_sound(&self) -> &str {
        self.ram_impact_sound
            .as_ref()
            .map_or("undefined", SoundEvent::name)
    }

    /// `ram_speed`, or its default 2.
    pub fn ram_speed(&self) -> f32 {
        self.ram_speed.map_or(2.0, |v| v.0)
    }

    /// `run_speed`, or its default 1.
    pub fn run_speed(&self) -> f32 {
        self.run_speed.map_or(1.0, |v| v.0)
    }
}

impl Goal for RamAttack {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 10);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.range("cooldown_range", &mut self.cooldown_range);
        c.triggers("on_start", self.on_start.as_deref());
        c.sound("pre_ram_sound", self.pre_ram_sound.as_ref());
        c.sound("ram_impact_sound", self.ram_impact_sound.as_ref());
    }
}

impl MolangFields for RamAttack {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_triggers(&mut triggers, "on_start", self.on_start.as_deref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.ranged_attack`: allows an entity to attack by using ranged shots. `charge_shoot_trigger` must be greater than 0 to enable charged up burst-shot attacks. Requires `minecraft:shooter` to define projectile behaviour.
// `behavior.ranged_attack`: typed from 1.26.40, redeclared 1.26.50.
#[doc(alias = "minecraft:behavior.ranged_attack")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct RangedAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `attack_interval`: reload-time range (in seconds), when not using a charged shot. Reload-time range scales with target-distance. Default -1 to -1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_interval: Option<FloatRange>,
    /// `attack_range`: range of distances from the target within which the entity can perform a ranged attack. Default 0 to 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_range: Option<FloatRange>,
    /// `burst_interval`: time (in seconds) between each individual shot when firing a burst of shots from a charged up attack. Default 0; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub burst_interval: Option<StrictFloat<f32>>,
    /// `burst_shots`: number of shots fired every time the attacking entity uses a charged up attack. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub burst_shots: Option<StrictInt<i32>>,
    /// `charge_charged_trigger`: time (in seconds, then add `charge_shoot_trigger`), before a charged up attack is done charging. Charge-time decays while target is not in sight. Default 0; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub charge_charged_trigger: Option<StrictFloat<f32>>,
    /// `charge_shoot_trigger`: amount of time (in seconds, then doubled) a charged shot must be charging before reloading burst shots. Charge-time decays while target is not in sight. Default 0; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub charge_shoot_trigger: Option<StrictFloat<f32>>,
    /// `in_range_movement_mode`: controls how the entity moves while its target is within the configured attack range. `hold_position` makes the entity stop and hold its position while facing and attacking the target. `follow_target` makes the entity continue moving toward the target while attacking, stopping just outside the minimum attack range. Default `hold_position`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub in_range_movement_mode: Option<InRangeMovementMode>,
    /// `ranged_fov`: field of view (in degrees) when using sensing to detect a target for attack. Default 9e+01; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub ranged_fov: Option<StrictFloat<f32>>,
    /// `set_persistent`: allows the actor to be set to persist upon targeting a player. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub set_persistent: Option<StrictBool>,
    /// `speed_multiplier`: during attack behavior, this multiplier modifies the entity's speed when moving toward the target. Default 1; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// `swing`: if a swing animation (using `variable.attack_time`) exists, this causes the actor to swing their arm(s) upon firing the ranged attack. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub swing: Option<StrictBool>,
    /// `target_in_sight_time`: minimum amount of time (in seconds) the attacking entity needs to see the target before moving toward it. Default 1; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub target_in_sight_time: Option<StrictFloat<f32>>,
    /// `x_max_rotation`: maximum rotation (in degrees), on the X-axis, this entity can rotate while trying to look at the target. Default 3e+01; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub x_max_rotation: Option<StrictFloat<f32>>,
    /// `y_max_head_rotation`: maximum rotation (in degrees), on the Y-axis, this entity can rotate its head while trying to look at the target. Default 3e+01; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub y_max_head_rotation: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl RangedAttack {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `attack_interval`, or its default -1 to -1.
    pub fn attack_interval(&self) -> (f32, f32) {
        self.attack_interval
            .map_or((-1.0, -1.0), |range| range.resolve(-1.0, -1.0))
    }

    /// `attack_range`, or its default 0 to 0.
    pub fn attack_range(&self) -> (f32, f32) {
        // In the order written: the cereal range is not reordered.
        self.attack_range.map_or((0.0, 0.0), |range| {
            (
                if range.missing_min { 0.0 } else { range.min },
                if range.missing_max { 0.0 } else { range.max },
            )
        })
    }

    /// `burst_interval`, or its default 0.
    pub fn burst_interval(&self) -> f32 {
        self.burst_interval.map_or(0.0, |v| v.0)
    }

    /// `burst_shots`, or its default 1.
    pub fn burst_shots(&self) -> i32 {
        self.burst_shots.map_or(1, |v| v.0)
    }

    /// `charge_charged_trigger`, or its default 0.
    pub fn charge_charged_trigger(&self) -> f32 {
        self.charge_charged_trigger.map_or(0.0, |v| v.0)
    }

    /// `charge_shoot_trigger`, or its default 0.
    pub fn charge_shoot_trigger(&self) -> f32 {
        self.charge_shoot_trigger.map_or(0.0, |v| v.0)
    }

    /// `in_range_movement_mode`, or its default `hold_position`.
    pub fn in_range_movement_mode(&self) -> InRangeMovementMode {
        self.in_range_movement_mode
            .unwrap_or(InRangeMovementMode::HoldPosition)
    }

    /// `ranged_fov`, or its default 9e+01.
    pub fn ranged_fov(&self) -> f32 {
        self.ranged_fov.map_or(9.0e1, |v| v.0)
    }

    /// `set_persistent`, or its default `false`.
    pub fn set_persistent(&self) -> bool {
        self.set_persistent.is_some_and(|v| v.0)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }

    /// `swing`, or its default `false`.
    pub fn swing(&self) -> bool {
        self.swing.is_some_and(|v| v.0)
    }

    /// `target_in_sight_time`, or its default 1.
    pub fn target_in_sight_time(&self) -> f32 {
        self.target_in_sight_time.map_or(1.0, |v| v.0)
    }

    /// `x_max_rotation`, or its default 3e+01.
    pub fn x_max_rotation(&self) -> f32 {
        self.x_max_rotation.map_or(3.0e1, |v| v.0)
    }

    /// `y_max_head_rotation`, or its default 3e+01.
    pub fn y_max_head_rotation(&self) -> f32 {
        self.y_max_head_rotation.map_or(3.0e1, |v| v.0)
    }
}

impl Goal for RangedAttack {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 40);

    const REJECTED: &'static [&'static str] = &["attack_interval_min", "attack_interval_max"];

    const GATED: &'static [(&'static str, FormatVersion)] = &[
        ("attack_range", V1_26_50),
        ("in_range_movement_mode", V1_26_50),
    ];

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.range("attack_interval", &mut self.attack_interval);
        c.float(
            "burst_interval",
            &mut self.burst_interval,
            Bound::Above(0.0),
        );
        c.float(
            "charge_charged_trigger",
            &mut self.charge_charged_trigger,
            Bound::Above(0.0),
        );
        c.float(
            "charge_shoot_trigger",
            &mut self.charge_shoot_trigger,
            Bound::Above(0.0),
        );
        c.float("ranged_fov", &mut self.ranged_fov, Bound::Min(0.0));
        c.float(
            "speed_multiplier",
            &mut self.speed_multiplier,
            Bound::Above(0.0),
        );
        c.float(
            "target_in_sight_time",
            &mut self.target_in_sight_time,
            Bound::Above(0.0),
        );
        c.float("x_max_rotation", &mut self.x_max_rotation, Bound::Min(0.0));
        c.float(
            "y_max_head_rotation",
            &mut self.y_max_head_rotation,
            Bound::Min(0.0),
        );
        // The cereal `FloatRange` binds `min` / `max` with no order constraint and no swap:
        // a range written out of order is held as written, so `attack_range.min ≤ max` is
        // not enforced.
        if let Some(range) = self.attack_range.as_mut().filter(|range| range.swapped) {
            std::mem::swap(&mut range.min, &mut range.max);
            range.swapped = false;
        }
    }

    fn forget(&mut self, name: &str) {
        match name {
            "attack_range" => self.attack_range = None,
            "in_range_movement_mode" => self.in_range_movement_mode = None,
            _ => {}
        }
    }

    fn upgrade(&mut self, from: FormatVersion, _d: &mut Diagnostics) {
        if from < V1_26_50 {
            convert::ranged_radius_typed(self);
        }
    }

    fn lenient_shape(json: &mut VObject, d: &mut Diagnostics) {
        convert::ranged_interval(json, d);
        convert::ranged_radius(json, d);
    }
}

impl MolangFields for RangedAttack {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.slime_attack`: causes the entity to grow tired every once in a while, while attacking.
// `behavior.slime_attack`: typed from 1.21.110.
#[doc(alias = "minecraft:behavior.slime_attack")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct SlimeAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `grow_tired_cooldown_time`: default 15.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub grow_tired_cooldown_time: Option<StrictFloat<f32>>,
    /// `set_persistent`: allows the actor to be set to persist upon targeting a player. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub set_persistent: Option<StrictBool>,
    /// `speed_multiplier`: during attack behavior, this multiplier modifies the entity's speed when moving toward the target. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// `x_max_rotation`: maximum rotation (in degrees), on the X-axis, this entity can rotate while trying to look at the target. Default 1e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub x_max_rotation: Option<StrictFloat<f32>>,
    /// `y_max_rotation`: maximum rotation (in degrees), on the Y-axis, this entity can rotate while trying to look at the target. Default 1e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub y_max_rotation: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl SlimeAttack {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `grow_tired_cooldown_time`, or its default 15.
    pub fn grow_tired_cooldown_time(&self) -> f32 {
        self.grow_tired_cooldown_time.map_or(15.0, |v| v.0)
    }

    /// `set_persistent`, or its default `false`.
    pub fn set_persistent(&self) -> bool {
        self.set_persistent.is_some_and(|v| v.0)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }

    /// `x_max_rotation`, or its default 1e+01.
    pub fn x_max_rotation(&self) -> f32 {
        self.x_max_rotation.map_or(1.0e1, |v| v.0)
    }

    /// `y_max_rotation`, or its default 1e+01.
    pub fn y_max_rotation(&self) -> f32 {
        self.y_max_rotation.map_or(1.0e1, |v| v.0)
    }
}

impl Goal for SlimeAttack {
    const TYPED: FormatVersion = FormatVersion::new(1, 21, 110);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for SlimeAttack {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.sonic_boom`: allows this entity to perform a 'sonic boom' ranged attack.
// `behavior.sonic_boom`: typed from 1.21.110.
#[doc(alias = "minecraft:behavior.sonic_boom")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct SonicBoom {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `attack_cooldown`: cooldown in seconds required after using this attack until the entity can use sonic boom again. Default 5.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_cooldown: Option<StrictFloat<f32>>,
    /// `attack_damage`: attack damage of the sonic boom. Default 3e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_damage: Option<StrictFloat<f32>>,
    /// `attack_range_horizontal`: horizontal range (in blocks) at which the sonic boom can damage the target. Default 15.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_range_horizontal: Option<StrictFloat<f32>>,
    /// `attack_range_vertical`: vertical range (in blocks) at which the sonic boom can damage the target. Default 2e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_range_vertical: Option<StrictFloat<f32>>,
    /// `attack_sound`: sound event for the attack. Default `""`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_sound: Option<SoundEvent>,
    /// `charge_sound`: sound event for the charge up. Default `""`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub charge_sound: Option<SoundEvent>,
    /// `duration`: goal duration in seconds. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub duration: Option<StrictFloat<f32>>,
    /// `duration_until_attack_sound`: duration in seconds until the attack sound is played. Default 1.7.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub duration_until_attack_sound: Option<StrictFloat<f32>>,
    /// `knockback_height_cap`: height cap of the attack knockback's vertical delta. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub knockback_height_cap: Option<StrictFloat<f32>>,
    /// `knockback_horizontal_strength`: horizontal strength of the attack's knockback applied to the attack target. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub knockback_horizontal_strength: Option<StrictFloat<f32>>,
    /// `knockback_vertical_strength`: vertical strength of the attack's knockback applied to the attack target. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub knockback_vertical_strength: Option<StrictFloat<f32>>,
    /// `speed_multiplier`: this multiplier modifies the attacking entity's speed when moving toward the target. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl SonicBoom {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `attack_cooldown`, or its default 5.
    pub fn attack_cooldown(&self) -> f32 {
        self.attack_cooldown.map_or(5.0, |v| v.0)
    }

    /// `attack_damage`, or its default 3e+01.
    pub fn attack_damage(&self) -> f32 {
        self.attack_damage.map_or(3.0e1, |v| v.0)
    }

    /// `attack_range_horizontal`, or its default 15.
    pub fn attack_range_horizontal(&self) -> f32 {
        self.attack_range_horizontal.map_or(15.0, |v| v.0)
    }

    /// `attack_range_vertical`, or its default 2e+01.
    pub fn attack_range_vertical(&self) -> f32 {
        self.attack_range_vertical.map_or(2.0e1, |v| v.0)
    }

    /// `attack_sound`, or its default `""`.
    pub fn attack_sound(&self) -> &str {
        self.attack_sound.as_ref().map_or("", SoundEvent::name)
    }

    /// `charge_sound`, or its default `""`.
    pub fn charge_sound(&self) -> &str {
        self.charge_sound.as_ref().map_or("", SoundEvent::name)
    }

    /// `duration`, or its default 0.
    pub fn duration(&self) -> f32 {
        self.duration.map_or(0.0, |v| v.0)
    }

    /// `duration_until_attack_sound`, or its default 1.7.
    pub fn duration_until_attack_sound(&self) -> f32 {
        self.duration_until_attack_sound.map_or(1.7, |v| v.0)
    }

    /// `knockback_height_cap`, or its default 0.
    pub fn knockback_height_cap(&self) -> f32 {
        self.knockback_height_cap.map_or(0.0, |v| v.0)
    }

    /// `knockback_horizontal_strength`, or its default 0.
    pub fn knockback_horizontal_strength(&self) -> f32 {
        self.knockback_horizontal_strength.map_or(0.0, |v| v.0)
    }

    /// `knockback_vertical_strength`, or its default 0.
    pub fn knockback_vertical_strength(&self) -> f32 {
        self.knockback_vertical_strength.map_or(0.0, |v| v.0)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }
}

impl Goal for SonicBoom {
    const TYPED: FormatVersion = FormatVersion::new(1, 21, 110);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.sound("attack_sound", self.attack_sound.as_ref());
        c.sound("charge_sound", self.charge_sound.as_ref());
        c.float("duration", &mut self.duration, Bound::Min(0.0));
    }
}

impl MolangFields for SonicBoom {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.stalk_and_pounce_on_target`: allows a mob to stalk a target, then once within range pounce onto a target, on success the target will be attacked dealing damage defined by the attack component. On failure, the mob will risk getting stuck.
// `behavior.stalk_and_pounce_on_target`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.stalk_and_pounce_on_target")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct StalkAndPounceOnTarget {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `interest_time`: the amount of time the mob will be interested before pouncing. This happens when the mob is within range of pouncing. Default 2.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub interest_time: Option<StrictFloat<f32>>,
    /// `leap_dist`: the distance in blocks the mob jumps in the direction of its target. Default 0.8.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub leap_dist: Option<StrictFloat<f32>>,
    /// `leap_height`: the height in blocks the mob jumps when leaping at its target. Default 0.9.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub leap_height: Option<StrictFloat<f32>>,
    /// `max_stalk_dist`: the maximum distance away a target can be before the mob gives up on stalking. Default 1e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_stalk_dist: Option<StrictFloat<f32>>,
    /// `pounce_max_dist`: the maximum distance away from the target in blocks to begin pouncing at the target. Default 5.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub pounce_max_dist: Option<StrictFloat<f32>>,
    /// `set_persistent`: allows the actor to be set to persist upon targeting a player. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub set_persistent: Option<StrictBool>,
    /// `stalk_speed`: the movement speed in which you stalk your target. Default 1.2.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub stalk_speed: Option<StrictFloat<f32>>,
    /// `strike_dist`: the max distance away from the target when landing from the pounce that will still result in damaging the target. Default 2.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub strike_dist: Option<StrictFloat<f32>>,
    /// `stuck_blocks`: blocks that the mob can get stuck in after a pounce.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub stuck_blocks: Option<Filter>,
    /// `stuck_time`: the amount of time the mob will be stuck if they fail and land on a block they can be stuck on. Default 2.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub stuck_time: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl StalkAndPounceOnTarget {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `interest_time`, or its default 2.
    pub fn interest_time(&self) -> f32 {
        self.interest_time.map_or(2.0, |v| v.0)
    }

    /// `leap_dist`, or its default 0.8.
    pub fn leap_dist(&self) -> f32 {
        self.leap_dist.map_or(0.8, |v| v.0)
    }

    /// `leap_height`, or its default 0.9.
    pub fn leap_height(&self) -> f32 {
        self.leap_height.map_or(0.9, |v| v.0)
    }

    /// `max_stalk_dist`, or its default 1e+01.
    pub fn max_stalk_dist(&self) -> f32 {
        self.max_stalk_dist.map_or(1.0e1, |v| v.0)
    }

    /// `pounce_max_dist`, or its default 5.
    pub fn pounce_max_dist(&self) -> f32 {
        self.pounce_max_dist.map_or(5.0, |v| v.0)
    }

    /// `set_persistent`, or its default `false`.
    pub fn set_persistent(&self) -> bool {
        self.set_persistent.is_some_and(|v| v.0)
    }

    /// `stalk_speed`, or its default 1.2.
    pub fn stalk_speed(&self) -> f32 {
        self.stalk_speed.map_or(1.2, |v| v.0)
    }

    /// `strike_dist`, or its default 2.
    pub fn strike_dist(&self) -> f32 {
        self.strike_dist.map_or(2.0, |v| v.0)
    }

    /// `stuck_time`, or its default 2.
    pub fn stuck_time(&self) -> f32 {
        self.stuck_time.map_or(2.0, |v| v.0)
    }
}

impl Goal for StalkAndPounceOnTarget {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 40);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.filter("stuck_blocks", self.stuck_blocks.as_ref());
    }
}

impl MolangFields for StalkAndPounceOnTarget {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.stomp_attack`: allows an entity to attack using stomp `AoE` damage behavior.
///
/// Deprecated members, reported and dropped: `melee_fov`.
// `behavior.stomp_attack`: typed from 1.26.0, redeclared 1.26.40.
#[doc(alias = "minecraft:behavior.stomp_attack")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct StompAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `attack_once`: allows the mob to perform this melee attack behavior only once during its lifetime. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_once: Option<StrictBool>,
    /// `attack_types`: defines the entity types this entity will attack. Default `""`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_types: Option<StrictString>,
    /// `can_spread_on_fire`: allows the mob, if on fire and empty handed, to ignite its target upon a successful attack. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub can_spread_on_fire: Option<StrictBool>,
    /// `cooldown_time`: cooldown time, in seconds, between consecutive attacks. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_time: Option<StrictFloat<f32>>,
    /// `inner_boundary_time_increase`: time, in seconds, added to the attack path recalculation interval when the target is beyond `path_inner_boundary`. Default 0.25; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub inner_boundary_time_increase: Option<StrictFloat<f32>>,
    /// `max_path_time`: maximum base time, in seconds, before recalculating a new attack path to the target (before increases are applied). Default 0.55; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_path_time: Option<StrictFloat<f32>>,
    /// `min_path_time`: minimum base time, in seconds, before recalculating a new attack path to the target (before increases are applied). Default 0.2; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_path_time: Option<StrictFloat<f32>>,
    /// `no_damage_range_multiplier`: multiplied with the final `AoE` damage range to determine a no damage range. The stomp attack will go on cooldown if target is in this no damage range. Default 2; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub no_damage_range_multiplier: Option<StrictFloat<f32>>,
    /// `on_attack`: defines the event to trigger when this entity successfully attacks.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_attack: Option<EventTrigger>,
    /// `on_kill`: defines the event to trigger when this entity kills the target.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_kill: Option<EventTrigger>,
    /// `outer_boundary_time_increase`: time, in seconds, added to the attack path recalculation interval when the target is beyond `path_outer_boundary`. Default 0.5; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub outer_boundary_time_increase: Option<StrictFloat<f32>>,
    /// `path_fail_time_increase`: time, in seconds, added to the attack path recalculation interval when the mob cannot move along the current path. Default 0.75; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub path_fail_time_increase: Option<StrictFloat<f32>>,
    /// `path_inner_boundary`: distance at which to increase attack path recalculation by `inner_boundary_time_increase`. Default 16; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub path_inner_boundary: Option<StrictFloat<f32>>,
    /// `path_outer_boundary`: distance at which to increase attack path recalculation by `outer_boundary_time_increase`. Default 32; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub path_outer_boundary: Option<StrictFloat<f32>>,
    /// `random_stop_interval`: defines a 1-in-N chance for the mob to stop its current attack, where N equals `random_stop_interval`. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub random_stop_interval: Option<StrictInt<i32>>,
    /// `reach_multiplier`: used with the base size of the entity to determine minimum target-distance before trying to deal attack damage. Default 2; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub reach_multiplier: Option<StrictFloat<f32>>,
    /// `require_complete_path`: specifies whether a full navigation path from the mob to the target is required. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub require_complete_path: Option<StrictBool>,
    /// `speed_multiplier`: multiplier applied to the mob's movement speed when moving toward its target. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// `stomp_range_multiplier`: multiplied with the base size of the entity to determine stomp `AoE` damage range. Default 2; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub stomp_range_multiplier: Option<StrictFloat<f32>>,
    /// `track_target`: allows the mob to track its target even if it lacks a hard-coded sensing component. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub track_target: Option<StrictBool>,
    /// `x_max_rotation`: maximum rotation, in degrees, on the X-axis while the mob is trying to look at its target. Default 3e+01; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub x_max_rotation: Option<StrictFloat<f32>>,
    /// `y_max_head_rotation`: maximum rotation, in degrees, on the Y-axis while the mob is trying to look at its target. Default 3e+01; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub y_max_head_rotation: Option<StrictFloat<f32>>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl StompAttack {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `attack_once`, or its default `false`.
    pub fn attack_once(&self) -> bool {
        self.attack_once.is_some_and(|v| v.0)
    }

    /// `attack_types`, or its default `""`.
    pub fn attack_types(&self) -> &str {
        self.attack_types.as_ref().map_or("", |v| v.0.as_str())
    }

    /// `can_spread_on_fire`, or its default `false`.
    pub fn can_spread_on_fire(&self) -> bool {
        self.can_spread_on_fire.is_some_and(|v| v.0)
    }

    /// `cooldown_time`, or its default 1.
    pub fn cooldown_time(&self) -> f32 {
        self.cooldown_time.map_or(1.0, |v| v.0)
    }

    /// `inner_boundary_time_increase`, or its default 0.25.
    pub fn inner_boundary_time_increase(&self) -> f32 {
        self.inner_boundary_time_increase.map_or(0.25, |v| v.0)
    }

    /// `max_path_time`, or its default 0.55.
    pub fn max_path_time(&self) -> f32 {
        self.max_path_time.map_or(0.55, |v| v.0)
    }

    /// `min_path_time`, or its default 0.2.
    pub fn min_path_time(&self) -> f32 {
        self.min_path_time.map_or(0.2, |v| v.0)
    }

    /// `no_damage_range_multiplier`, or its default 2.
    pub fn no_damage_range_multiplier(&self) -> f32 {
        self.no_damage_range_multiplier.map_or(2.0, |v| v.0)
    }

    /// `outer_boundary_time_increase`, or its default 0.5.
    pub fn outer_boundary_time_increase(&self) -> f32 {
        self.outer_boundary_time_increase.map_or(0.5, |v| v.0)
    }

    /// `path_fail_time_increase`, or its default 0.75.
    pub fn path_fail_time_increase(&self) -> f32 {
        self.path_fail_time_increase.map_or(0.75, |v| v.0)
    }

    /// `path_inner_boundary`, or its default 16.
    pub fn path_inner_boundary(&self) -> f32 {
        self.path_inner_boundary.map_or(16.0, |v| v.0)
    }

    /// `path_outer_boundary`, or its default 32.
    pub fn path_outer_boundary(&self) -> f32 {
        self.path_outer_boundary.map_or(32.0, |v| v.0)
    }

    /// `random_stop_interval`, or its default 0.
    pub fn random_stop_interval(&self) -> i32 {
        self.random_stop_interval.map_or(0, |v| v.0)
    }

    /// `reach_multiplier`, or its default 2.
    pub fn reach_multiplier(&self) -> f32 {
        self.reach_multiplier.map_or(2.0, |v| v.0)
    }

    /// `require_complete_path`, or its default `false`.
    pub fn require_complete_path(&self) -> bool {
        self.require_complete_path.is_some_and(|v| v.0)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }

    /// `stomp_range_multiplier`, or its default 2.
    pub fn stomp_range_multiplier(&self) -> f32 {
        self.stomp_range_multiplier.map_or(2.0, |v| v.0)
    }

    /// `track_target`, or its default `false`.
    pub fn track_target(&self) -> bool {
        self.track_target.is_some_and(|v| v.0)
    }

    /// `x_max_rotation`, or its default 3e+01.
    pub fn x_max_rotation(&self) -> f32 {
        self.x_max_rotation.map_or(3.0e1, |v| v.0)
    }

    /// `y_max_head_rotation`, or its default 3e+01.
    pub fn y_max_head_rotation(&self) -> f32 {
        self.y_max_head_rotation.map_or(3.0e1, |v| v.0)
    }
}

impl Goal for StompAttack {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 0);

    const DEPRECATED: &'static [Deprecated] = &[(
        "melee_fov",
        deprecated_text!("melee_fov"),
        FormatVersion::new(1, 26, 0),
    )];

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.float("cooldown_time", &mut self.cooldown_time, Bound::Min(0.0));
        c.float(
            "inner_boundary_time_increase",
            &mut self.inner_boundary_time_increase,
            Bound::Min(0.0),
        );
        c.float("max_path_time", &mut self.max_path_time, Bound::Min(0.0));
        c.float("min_path_time", &mut self.min_path_time, Bound::Min(0.0));
        c.float(
            "no_damage_range_multiplier",
            &mut self.no_damage_range_multiplier,
            Bound::Above(0.0),
        );
        c.trigger("on_attack", self.on_attack.as_ref());
        c.trigger("on_kill", self.on_kill.as_ref());
        c.float(
            "outer_boundary_time_increase",
            &mut self.outer_boundary_time_increase,
            Bound::Min(0.0),
        );
        c.float(
            "path_fail_time_increase",
            &mut self.path_fail_time_increase,
            Bound::Min(0.0),
        );
        c.float(
            "path_inner_boundary",
            &mut self.path_inner_boundary,
            Bound::Min(0.0),
        );
        c.float(
            "path_outer_boundary",
            &mut self.path_outer_boundary,
            Bound::Min(0.0),
        );
        c.int(
            "random_stop_interval",
            &mut self.random_stop_interval,
            Bound::Min(0),
        );
        c.float(
            "reach_multiplier",
            &mut self.reach_multiplier,
            Bound::Min(0.0),
        );
        c.float(
            "speed_multiplier",
            &mut self.speed_multiplier,
            Bound::Min(0.0),
        );
        c.float(
            "stomp_range_multiplier",
            &mut self.stomp_range_multiplier,
            Bound::Above(0.0),
        );
        c.float("x_max_rotation", &mut self.x_max_rotation, Bound::Min(0.0));
        c.float(
            "y_max_head_rotation",
            &mut self.y_max_head_rotation,
            Bound::Min(0.0),
        );
    }

    fn upgrade(&mut self, from: FormatVersion, _: &mut Diagnostics) {
        if from < V1_26_40 {
            convert::fold_on_kill_fields(&mut self.on_attack, &mut self.on_kill);
        }
    }

    fn lenient_shape(json: &mut VObject, d: &mut Diagnostics) {
        convert::fold_on_kill(json, d);
    }
}

impl MolangFields for StompAttack {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(&mut triggers, "on_attack", self.on_attack.as_ref());
        push_trigger(&mut triggers, "on_kill", self.on_kill.as_ref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.summon_entity`: allows the mob to attack the player by summoning other entities.
// `behavior.summon_entity`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.summon_entity")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct SummonEntity {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `summon_choices`: list of spells for the mob to use to summon entities. Each spell has the following parameters.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub summon_choices: Option<Vec<SummonChoice>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl SummonEntity {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }
}

impl Goal for SummonEntity {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 40);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.each(
            "summon_choices",
            &mut self.summon_choices,
            SummonChoice::check,
        );
    }
}

impl MolangFields for SummonEntity {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.swell`: allows the creeper to swell up when a player is nearby. It can only be used by Creepers.
// `behavior.swell`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.swell")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Swell {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `start_distance`: this mob starts swelling when a target is at least this many blocks away. Default 1e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub start_distance: Option<StrictFloat<f32>>,
    /// `stop_distance`: this mob stops swelling when a target has moved away at least this many blocks. Default 2.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub stop_distance: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Swell {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `start_distance`, or its default 1e+01.
    pub fn start_distance(&self) -> f32 {
        self.start_distance.map_or(1.0e1, |v| v.0)
    }

    /// `stop_distance`, or its default 2.
    pub fn stop_distance(&self) -> f32 {
        self.stop_distance.map_or(2.0, |v| v.0)
    }
}

impl Goal for Swell {
    const TYPED: FormatVersion = FormatVersion::new(1, 26, 30);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for Swell {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.swoop_attack`: allows an entity to attack using swoop attack behavior; Ideal for use with flying mobs. The behavior ends if the entity has a horizontal collision or gets hit.
// `behavior.swoop_attack`: typed from 1.21.110.
#[doc(alias = "minecraft:behavior.swoop_attack")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct SwoopAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `damage_reach`: added to the base size of the entity, to determine the target's maximum allowable distance, when trying to deal attack damage. Default 0.2.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub damage_reach: Option<StrictFloat<f32>>,
    /// `delay_range`: minimum and maximum cooldown time-range (in seconds) between each attempted swoop attack. Default 1e+01 to 2e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub delay_range: Option<FloatRange>,
    /// `speed_multiplier`: during swoop attack behavior, this determines the multiplier the entity's speed is modified by when moving toward the target. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl SwoopAttack {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `damage_reach`, or its default 0.2.
    pub fn damage_reach(&self) -> f32 {
        self.damage_reach.map_or(0.2, |v| v.0)
    }

    /// `delay_range`, or its default 1e+01 to 2e+01.
    pub fn delay_range(&self) -> (f32, f32) {
        self.delay_range
            .map_or((1.0e1, 2.0e1), |range| range.resolve(1.0e1, 2.0e1))
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }
}

impl Goal for SwoopAttack {
    const TYPED: FormatVersion = FormatVersion::new(1, 21, 110);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.range("delay_range", &mut self.delay_range);
    }
}

impl MolangFields for SwoopAttack {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.use_kinetic_weapon`: enables a mob to use kinetic weaponry by intermittently charging at its target and repositioning afterward.
// `behavior.use_kinetic_weapon`: typed from 1.21.120, redeclared 1.26.40.
#[doc(alias = "minecraft:behavior.use_kinetic_weapon")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct UseKineticWeapon {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `approach_distance`: the distance to the target within which the mob begins using its kinetic weapon. Default 8; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub approach_distance: Option<StrictFloat<f32>>,
    /// `attack_once`: allows the mob to perform this melee attack behavior only once during its lifetime. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_once: Option<StrictBool>,
    /// `attack_types`: defines the entity types this entity will attack. Default `""`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub attack_types: Option<StrictString>,
    /// `can_spread_on_fire`: allows the mob, if on fire and empty handed, to ignite its target upon a successful attack. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub can_spread_on_fire: Option<StrictBool>,
    /// `cooldown_distance`: the distance the mob retreats to after all of the item's `minecraft:kinetic_weapon` component's `max_duration` values have elapsed. After reaching this position and once `cooldown` has elapsed, the mob will approach again. This value is added to the reach midpoint. Default 8 to 8.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_distance: Option<FloatRange>,
    /// `cooldown_speed_multiplier`: multiplier applied to the mob's movement speed while on cooldown. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_speed_multiplier: Option<StrictFloat<f32>>,
    /// `cooldown_time`: cooldown time, in seconds, between consecutive attacks. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_time: Option<StrictFloat<f32>>,
    /// `hijack_mount_navigation`: allows the mob to override its mount's navigation behavior with the logic defined by this goal. Requires the mount to be running the `minecraft:behavior.mount_pathing` goal, which default behavior will be ignored. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub hijack_mount_navigation: Option<StrictBool>,
    /// `inner_boundary_time_increase`: time, in seconds, added to the attack path recalculation interval when the target is beyond `path_inner_boundary`. Default 0.25; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub inner_boundary_time_increase: Option<StrictFloat<f32>>,
    /// `max_path_time`: maximum base time, in seconds, before recalculating a new attack path to the target (before increases are applied). Default 0.55; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_path_time: Option<StrictFloat<f32>>,
    /// `melee_fov`: field of view, in degrees, used by the hard-coded sensing component to detect a valid attack target. Default 9e+01; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub melee_fov: Option<StrictFloat<f32>>,
    /// `min_path_time`: minimum base time, in seconds, before recalculating a new attack path to the target (before increases are applied). Default 0.2; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_path_time: Option<StrictFloat<f32>>,
    /// `on_attack`: defines the event to trigger when this entity successfully attacks.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_attack: Option<EventTrigger>,
    /// `on_kill`: defines the event to trigger when this entity kills the target.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_kill: Option<EventTrigger>,
    /// `outer_boundary_time_increase`: time, in seconds, added to the attack path recalculation interval when the target is beyond `path_outer_boundary`. Default 0.5; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub outer_boundary_time_increase: Option<StrictFloat<f32>>,
    /// `path_fail_time_increase`: time, in seconds, added to the attack path recalculation interval when the mob cannot move along the current path. Default 0.75; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub path_fail_time_increase: Option<StrictFloat<f32>>,
    /// `path_inner_boundary`: distance at which to increase attack path recalculation by `inner_boundary_time_increase`. Default 16; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub path_inner_boundary: Option<StrictFloat<f32>>,
    /// `path_outer_boundary`: distance at which to increase attack path recalculation by `outer_boundary_time_increase`. Default 32; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub path_outer_boundary: Option<StrictFloat<f32>>,
    /// `random_stop_interval`: defines a 1-in-N chance for the mob to stop its current attack, where N equals `random_stop_interval`. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub random_stop_interval: Option<StrictInt<i32>>,
    /// `reposition_distance`: the distance the mob retreats to once the target is closer than the midpoint of the item's `minecraft:kinetic_weapon` component's minimum and maximum `reach`. After reaching this position, the mob will charge again. This value is added to the reach midpoint. Default 2 to 2.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub reposition_distance: Option<FloatRange>,
    /// `reposition_speed_multiplier`: multiplier applied to the mob's movement speed while repositioning. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub reposition_speed_multiplier: Option<StrictFloat<f32>>,
    /// `require_complete_path`: specifies whether a full navigation path from the mob to the target is required. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub require_complete_path: Option<StrictBool>,
    /// `speed_multiplier`: multiplier applied to the mob's movement speed when moving toward its target. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// `track_target`: allows the mob to track its target even if it lacks a hard-coded sensing component. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub track_target: Option<StrictBool>,
    /// `weapon_min_speed_multiplier`: multiplier applied to each `min_speed` and `min_relative_speed` condition in the item's `minecraft:kinetic_weapon` component. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub weapon_min_speed_multiplier: Option<StrictFloat<f32>>,
    /// `weapon_reach_multiplier`: multiplier applied to the item's `minecraft:kinetic_weapon` component's `reach`. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub weapon_reach_multiplier: Option<StrictFloat<f32>>,
    /// `x_max_rotation`: maximum rotation, in degrees, on the X-axis while the mob is trying to look at its target. Default 3e+01; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub x_max_rotation: Option<StrictFloat<f32>>,
    /// `y_max_head_rotation`: maximum rotation, in degrees, on the Y-axis while the mob is trying to look at its target. Default 3e+01; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub y_max_head_rotation: Option<StrictFloat<f32>>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl UseKineticWeapon {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `approach_distance`, or its default 8.
    pub fn approach_distance(&self) -> f32 {
        self.approach_distance.map_or(8.0, |v| v.0)
    }

    /// `attack_once`, or its default `false`.
    pub fn attack_once(&self) -> bool {
        self.attack_once.is_some_and(|v| v.0)
    }

    /// `attack_types`, or its default `""`.
    pub fn attack_types(&self) -> &str {
        self.attack_types.as_ref().map_or("", |v| v.0.as_str())
    }

    /// `can_spread_on_fire`, or its default `false`.
    pub fn can_spread_on_fire(&self) -> bool {
        self.can_spread_on_fire.is_some_and(|v| v.0)
    }

    /// `cooldown_distance`, or its default 8 to 8.
    pub fn cooldown_distance(&self) -> (f32, f32) {
        self.cooldown_distance
            .map_or((8.0, 8.0), |range| range.resolve(8.0, 8.0))
    }

    /// `cooldown_speed_multiplier`, or its default 1.
    pub fn cooldown_speed_multiplier(&self) -> f32 {
        self.cooldown_speed_multiplier.map_or(1.0, |v| v.0)
    }

    /// `cooldown_time`, or its default 1.
    pub fn cooldown_time(&self) -> f32 {
        self.cooldown_time.map_or(1.0, |v| v.0)
    }

    /// `hijack_mount_navigation`, or its default `false`.
    pub fn hijack_mount_navigation(&self) -> bool {
        self.hijack_mount_navigation.is_some_and(|v| v.0)
    }

    /// `inner_boundary_time_increase`, or its default 0.25.
    pub fn inner_boundary_time_increase(&self) -> f32 {
        self.inner_boundary_time_increase.map_or(0.25, |v| v.0)
    }

    /// `max_path_time`, or its default 0.55.
    pub fn max_path_time(&self) -> f32 {
        self.max_path_time.map_or(0.55, |v| v.0)
    }

    /// `melee_fov`, or its default 9e+01.
    pub fn melee_fov(&self) -> f32 {
        self.melee_fov.map_or(9.0e1, |v| v.0)
    }

    /// `min_path_time`, or its default 0.2.
    pub fn min_path_time(&self) -> f32 {
        self.min_path_time.map_or(0.2, |v| v.0)
    }

    /// `outer_boundary_time_increase`, or its default 0.5.
    pub fn outer_boundary_time_increase(&self) -> f32 {
        self.outer_boundary_time_increase.map_or(0.5, |v| v.0)
    }

    /// `path_fail_time_increase`, or its default 0.75.
    pub fn path_fail_time_increase(&self) -> f32 {
        self.path_fail_time_increase.map_or(0.75, |v| v.0)
    }

    /// `path_inner_boundary`, or its default 16.
    pub fn path_inner_boundary(&self) -> f32 {
        self.path_inner_boundary.map_or(16.0, |v| v.0)
    }

    /// `path_outer_boundary`, or its default 32.
    pub fn path_outer_boundary(&self) -> f32 {
        self.path_outer_boundary.map_or(32.0, |v| v.0)
    }

    /// `random_stop_interval`, or its default 0.
    pub fn random_stop_interval(&self) -> i32 {
        self.random_stop_interval.map_or(0, |v| v.0)
    }

    /// `reposition_distance`, or its default 2 to 2.
    pub fn reposition_distance(&self) -> (f32, f32) {
        self.reposition_distance
            .map_or((2.0, 2.0), |range| range.resolve(2.0, 2.0))
    }

    /// `reposition_speed_multiplier`, or its default 1.
    pub fn reposition_speed_multiplier(&self) -> f32 {
        self.reposition_speed_multiplier.map_or(1.0, |v| v.0)
    }

    /// `require_complete_path`, or its default `false`.
    pub fn require_complete_path(&self) -> bool {
        self.require_complete_path.is_some_and(|v| v.0)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }

    /// `track_target`, or its default `false`.
    pub fn track_target(&self) -> bool {
        self.track_target.is_some_and(|v| v.0)
    }

    /// `weapon_min_speed_multiplier`, or its default 1.
    pub fn weapon_min_speed_multiplier(&self) -> f32 {
        self.weapon_min_speed_multiplier.map_or(1.0, |v| v.0)
    }

    /// `weapon_reach_multiplier`, or its default 1.
    pub fn weapon_reach_multiplier(&self) -> f32 {
        self.weapon_reach_multiplier.map_or(1.0, |v| v.0)
    }

    /// `x_max_rotation`, or its default 3e+01.
    pub fn x_max_rotation(&self) -> f32 {
        self.x_max_rotation.map_or(3.0e1, |v| v.0)
    }

    /// `y_max_head_rotation`, or its default 3e+01.
    pub fn y_max_head_rotation(&self) -> f32 {
        self.y_max_head_rotation.map_or(3.0e1, |v| v.0)
    }
}

impl Goal for UseKineticWeapon {
    const TYPED: FormatVersion = FormatVersion::new(1, 21, 120);

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.float(
            "approach_distance",
            &mut self.approach_distance,
            Bound::Min(0.0),
        );
        c.range("cooldown_distance", &mut self.cooldown_distance);
        c.float(
            "cooldown_speed_multiplier",
            &mut self.cooldown_speed_multiplier,
            Bound::Min(0.0),
        );
        c.float("cooldown_time", &mut self.cooldown_time, Bound::Min(0.0));
        c.float(
            "inner_boundary_time_increase",
            &mut self.inner_boundary_time_increase,
            Bound::Min(0.0),
        );
        c.float("max_path_time", &mut self.max_path_time, Bound::Min(0.0));
        c.float("melee_fov", &mut self.melee_fov, Bound::Min(0.0));
        c.float("min_path_time", &mut self.min_path_time, Bound::Min(0.0));
        c.trigger("on_attack", self.on_attack.as_ref());
        c.trigger("on_kill", self.on_kill.as_ref());
        c.float(
            "outer_boundary_time_increase",
            &mut self.outer_boundary_time_increase,
            Bound::Min(0.0),
        );
        c.float(
            "path_fail_time_increase",
            &mut self.path_fail_time_increase,
            Bound::Min(0.0),
        );
        c.float(
            "path_inner_boundary",
            &mut self.path_inner_boundary,
            Bound::Min(0.0),
        );
        c.float(
            "path_outer_boundary",
            &mut self.path_outer_boundary,
            Bound::Min(0.0),
        );
        c.int(
            "random_stop_interval",
            &mut self.random_stop_interval,
            Bound::Min(0),
        );
        c.range("reposition_distance", &mut self.reposition_distance);
        c.float(
            "reposition_speed_multiplier",
            &mut self.reposition_speed_multiplier,
            Bound::Min(0.0),
        );
        c.float(
            "speed_multiplier",
            &mut self.speed_multiplier,
            Bound::Min(0.0),
        );
        c.float(
            "weapon_min_speed_multiplier",
            &mut self.weapon_min_speed_multiplier,
            Bound::Min(0.0),
        );
        c.float(
            "weapon_reach_multiplier",
            &mut self.weapon_reach_multiplier,
            Bound::Min(0.0),
        );
        c.float("x_max_rotation", &mut self.x_max_rotation, Bound::Min(0.0));
        c.float(
            "y_max_head_rotation",
            &mut self.y_max_head_rotation,
            Bound::Min(0.0),
        );
    }

    fn upgrade(&mut self, from: FormatVersion, _: &mut Diagnostics) {
        if from < V1_26_40 {
            convert::fold_on_kill_fields(&mut self.on_attack, &mut self.on_kill);
        }
    }

    fn lenient_shape(json: &mut VObject, d: &mut Diagnostics) {
        convert::fold_on_kill(json, d);
    }
}

impl MolangFields for UseKineticWeapon {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(&mut triggers, "on_attack", self.on_attack.as_ref());
        push_trigger(&mut triggers, "on_kill", self.on_kill.as_ref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// One stage of a [`SummonChoice`]: what it summons, how many and where.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct SummonStage {
    /// `base_delay`: amount of time in seconds to wait before this step starts. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub base_delay: Option<StrictFloat<f32>>,
    /// `delay_per_summon`: amount of time in seconds before each entity is summoned in this step. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub delay_per_summon: Option<StrictFloat<f32>>,
    /// `entity_lifespan`: amount of time in seconds that the spawned entity will be alive for. A value of -1.0 means it will remain alive for as long as it can. Default -1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub entity_lifespan: Option<StrictFloat<f32>>,
    /// `entity_type`: the entity type of the entities we will spawn in this step. Default `""`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub entity_type: Option<StrictString>,
    /// `num_entities_spawned`: number of entities that will be spawned in this step. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub num_entities_spawned: Option<StrictInt<i32>>,
    /// `shape`: the base shape of this step. Valid values are circle and line. Default `line`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub shape: Option<SummonShape>,
    /// `size`: the base size of the entity. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub size: Option<StrictFloat<f32>>,
    /// `sound_event`: the sound event to play for this step. Default `undefined`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sound_event: Option<SoundEvent>,
    /// `summon_cap`: maximum number of summoned entities at any given time. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub summon_cap: Option<StrictInt<i32>>,
    /// `summon_cap_radius`: default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub summon_cap_radius: Option<StrictFloat<f32>>,
    /// `summon_event`: event to invoke on each summoned entity on spawn. Default `""`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub summon_event: Option<StrictString>,
    /// `target`: the target of the spell. This is where the spell will start (line will start here, circle will be centered here). Default `self`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub target: Option<SummonTarget>,
    /// Members the entry does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl SummonStage {
    /// `base_delay`, or its default 0.
    pub fn base_delay(&self) -> f32 {
        self.base_delay.map_or(0.0, |v| v.0)
    }

    /// `delay_per_summon`, or its default 0.
    pub fn delay_per_summon(&self) -> f32 {
        self.delay_per_summon.map_or(0.0, |v| v.0)
    }

    /// `entity_lifespan`, or its default -1.
    pub fn entity_lifespan(&self) -> f32 {
        self.entity_lifespan.map_or(-1.0, |v| v.0)
    }

    /// `entity_type`, or its default `""`.
    pub fn entity_type(&self) -> &str {
        self.entity_type.as_ref().map_or("", |v| v.0.as_str())
    }

    /// `num_entities_spawned`, or its default 1.
    pub fn num_entities_spawned(&self) -> i32 {
        self.num_entities_spawned.map_or(1, |v| v.0)
    }

    /// `shape`, or its default `line`.
    pub fn shape(&self) -> SummonShape {
        self.shape.unwrap_or(SummonShape::Line)
    }

    /// `size`, or its default 1.
    pub fn size(&self) -> f32 {
        self.size.map_or(1.0, |v| v.0)
    }

    /// `sound_event`, or its default `undefined`.
    pub fn sound_event(&self) -> &str {
        self.sound_event
            .as_ref()
            .map_or("undefined", SoundEvent::name)
    }

    /// `summon_cap`, or its default 0.
    pub fn summon_cap(&self) -> i32 {
        self.summon_cap.map_or(0, |v| v.0)
    }

    /// `summon_cap_radius`, or its default 0.
    pub fn summon_cap_radius(&self) -> f32 {
        self.summon_cap_radius.map_or(0.0, |v| v.0)
    }

    /// `summon_event`, or its default `""`.
    pub fn summon_event(&self) -> &str {
        self.summon_event.as_ref().map_or("", |v| v.0.as_str())
    }

    /// `target`, or its default `self`.
    pub fn target(&self) -> SummonTarget {
        self.target.unwrap_or(SummonTarget::Self_)
    }
}

impl SummonStage {
    /// Checks the entry's members.
    pub(crate) fn check(&mut self, c: &mut Check<'_>) {
        c.sound("sound_event", self.sound_event.as_ref());
    }
}

/// One spell `summon_entity` may cast: when it is chosen and the stages it runs.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct SummonChoice {
    /// `cast_duration`: time in seconds the spell casting will take. Default -1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cast_duration: Option<StrictFloat<f32>>,
    /// `cooldown_time`: time in seconds the mob has to wait before using the spell again. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_time: Option<StrictFloat<f32>>,
    /// `do_casting`: if true, the mob will do the casting animations and render spell particles. Default `true`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub do_casting: Option<StrictBool>,
    /// `filters`: filters allow data objects to specify test criteria which allows their use. Filters can be defined by a single object of type (Filter Test), an array of tests, collection groups, or a combination of these objects.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub filters: Option<Filter>,
    /// `max_activation_range`: upper bound of the activation distance in blocks for this spell, must not be negative. Default 32; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_activation_range: Option<StrictFloat<f32>>,
    /// `min_activation_range`: lower bound of the activation distance in blocks for this spell, must not be negative. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_activation_range: Option<StrictFloat<f32>>,
    /// `particle_color`: the color of the particles for this spell.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub particle_color: Option<ParticleColor>,
    /// `sequence`: list of steps for the spell. Each step has the following parameters.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sequence: Option<Vec<SummonStage>>,
    /// `start_sound_event`: the sound event to play when using this spell. Default `undefined`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub start_sound_event: Option<SoundEvent>,
    /// `weight`: the weight of this spell. Controls how likely the mob is to choose this spell when casting one. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub weight: Option<StrictFloat<f32>>,
    /// Members the entry does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl SummonChoice {
    /// `cast_duration`, or its default -1.
    pub fn cast_duration(&self) -> f32 {
        self.cast_duration.map_or(-1.0, |v| v.0)
    }

    /// `cooldown_time`, or its default 0.
    pub fn cooldown_time(&self) -> f32 {
        self.cooldown_time.map_or(0.0, |v| v.0)
    }

    /// `do_casting`, or its default `true`.
    pub fn do_casting(&self) -> bool {
        self.do_casting.is_none_or(|v| v.0)
    }

    /// `max_activation_range`, or its default 32.
    pub fn max_activation_range(&self) -> f32 {
        self.max_activation_range.map_or(32.0, |v| v.0)
    }

    /// `min_activation_range`, or its default 1.
    pub fn min_activation_range(&self) -> f32 {
        self.min_activation_range.map_or(1.0, |v| v.0)
    }

    /// `start_sound_event`, or its default `undefined`.
    pub fn start_sound_event(&self) -> &str {
        self.start_sound_event
            .as_ref()
            .map_or("undefined", SoundEvent::name)
    }

    /// `weight`, or its default 0.
    pub fn weight(&self) -> f32 {
        self.weight.map_or(0.0, |v| v.0)
    }
}

impl SummonChoice {
    /// Checks the entry's members.
    pub(crate) fn check(&mut self, c: &mut Check<'_>) {
        c.filter("filters", self.filters.as_ref());
        c.float(
            "max_activation_range",
            &mut self.max_activation_range,
            Bound::Min(0.0),
        );
        c.float(
            "min_activation_range",
            &mut self.min_activation_range,
            Bound::Min(0.0),
        );
        c.each("sequence", &mut self.sequence, SummonStage::check);
        c.sound("start_sound_event", self.start_sound_event.as_ref());
    }
}

/// Vanilla's message for a negative `min_activation_range` in an older document's
/// `send_event` choice (logged as a warning): the value becomes 1.
pub(crate) const MIN_ACTIVATION_NEGATIVE: &str =
    "'min_activation_range' must not be negative, defaulting to 1.0";
/// Vanilla's message for a negative `max_activation_range` (logged as a warning): the value
/// becomes 512.
pub(crate) const MAX_ACTIVATION_NEGATIVE: &str =
    "'max_activation_range' should not be negative, setting to 512.";
/// Vanilla's message for a `min_activation_range` above `max_activation_range` (logged as a
/// warning): the two are swapped.
pub(crate) const ACTIVATION_RANGE_SWAPPED: &str =
    "'min_activation_range' must be less than or equal to 'max_activation_range', swapping values.";

/// `minecraft:behavior.send_event`: allows the mob to send an event to another mob.
// `minecraft:behavior.send_event`: typed from 1.26.60; older documents are read by the
// legacy goal parser, which `lenient_shape` mirrors.
#[doc(alias = "minecraft:behavior.send_event")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct SendEvent {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `event_choices`: list of spell options to select from. One is randomly chosen each time the goal activates.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub event_choices: Option<Vec<SendEventChoice>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl SendEvent {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }
}

impl Goal for SendEvent {
    const TYPED: FormatVersion = super::movement::V1_26_60;

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.each(
            "event_choices",
            &mut self.event_choices,
            SendEventChoice::check,
        );
    }

    /// The legacy reader's conversions: each choice's activation range as the legacy
    /// reader settles it, with its texts; the times it stores in whole ticks (seconds × 20,
    /// truncated) read back as seconds, a value that changes so told where it is.
    fn lenient_shape(json: &mut VObject, d: &mut Diagnostics) {
        let Some(choices) = json.get_mut("event_choices").and_then(Value::as_array_mut) else {
            return;
        };
        for (i, choice) in choices.iter_mut().enumerate() {
            let Some(choice) = choice.as_object_mut() else {
                continue;
            };
            let at = |key: &str| JsonPath::new().child("event_choices").index(i).child(key);
            settle_activation_range(choice, &at, d);
            for key in ["cooldown_time", "cast_duration"] {
                whole_ticks(choice, key, at(key), d);
            }
            if let Some(stages) = choice.get_mut("sequence").and_then(Value::as_array_mut) {
                for (j, stage) in stages.iter_mut().enumerate() {
                    if let Some(stage) = stage.as_object_mut() {
                        let mut stage_at = at("sequence");
                        stage_at.push_index(j);
                        stage_at.push("base_delay");
                        whole_ticks(stage, "base_delay", stage_at, d);
                    }
                }
            }
        }
    }
}

impl MolangFields for SendEvent {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// The legacy reader's activation range of one choice: a negative
/// minimum becomes 1, a negative maximum 512, and a minimum above the maximum is swapped,
/// each with its warning. A member that is no number is left to the typed read.
fn settle_activation_range(
    choice: &mut VObject,
    at: &dyn Fn(&str) -> JsonPath,
    d: &mut Diagnostics,
) {
    const MIN: &str = "min_activation_range";
    const MAX: &str = "max_activation_range";
    let number = |choice: &VObject, key: &str, default: f32| match choice.get(key) {
        None => Some(default),
        Some(value) => float_of(value),
    };
    let (Some(mut min), Some(mut max)) = (number(choice, MIN, 1.0), number(choice, MAX, 32.0))
    else {
        return;
    };
    if min < 0.0 {
        min = 1.0;
        choice.insert(MIN, f32_value(min));
        d.push(at_warning(at(MIN), MIN_ACTIVATION_NEGATIVE));
    }
    if max < 0.0 {
        max = 512.0;
        choice.insert(MAX, f32_value(max));
        d.push(at_warning(at(MAX), MAX_ACTIVATION_NEGATIVE));
    }
    if min > max {
        choice.insert(MIN, f32_value(max));
        choice.insert(MAX, f32_value(min));
        d.push(at_warning(at(MIN), ACTIVATION_RANGE_SWAPPED));
    }
}

/// A time member the legacy reader stores as whole ticks: `trunc(seconds × 20) / 20` in
/// single precision, as the reader (a truncation toward zero) and the transform (`/ 20.0`) compute it. A value
/// that changes is rewritten and told (`ValueRewritten`, INFO: the game reads it so without a
/// word). A member that is no number is left to the typed read.
fn whole_ticks(object: &mut VObject, key: &str, at: JsonPath, d: &mut Diagnostics) {
    let Some(seconds) = object.get(key).and_then(float_of) else {
        return;
    };
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        reason = "the legacy reader's own conversion: seconds × 20 truncated to an int, and back"
    )]
    let ticked = ((seconds * 20.0) as i32) as f32 / 20.0;
    if ticked.to_bits() != seconds.to_bits() {
        object.insert(key, f32_value(ticked));
        d.push(Diagnostic::info(DiagnosticCode::ValueRewritten { text: None }).at(at));
    }
}

/// A warning of the legacy reader at `at`, with its text: the game logs the legacy goal
/// parser's activation-range lines (a negative minimum, a negative maximum, the swap) at the
/// warning level.
fn at_warning(at: JsonPath, text: &'static str) -> Diagnostic {
    Diagnostic::warn(DiagnosticCode::InvalidValue {
        text: Some(VanillaText::Fixed(text)),
    })
    .at(at)
}

/// An `f32` as the JSON number that reads back to it.
fn f32_value(value: f32) -> Value {
    Value::from(f64::from(value))
}

/// One entry of `send_event.event_choices`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct SendEventChoice {
    /// `cast_duration`: time in seconds for the entire event sending process. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cast_duration: Option<StrictFloat<f32>>,
    /// `cooldown_time`: time in seconds the mob has to wait before using this spell again. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_time: Option<StrictFloat<f32>>,
    /// `do_casting`: if true, the mob will show a casting animation while sending the event. Default `true`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub do_casting: Option<StrictBool>,
    /// `filters`: conditions that the target must meet to be eligible for receiving the event.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub filters: Option<Filter>,
    /// `look_at_target`: if true, the mob will face the entity it sends an event to. Default `true`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub look_at_target: Option<StrictBool>,
    /// `max_activation_range`: maximum distance in blocks the target can be for this spell to activate. Default 32.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_activation_range: Option<StrictFloat<f32>>,
    /// `min_activation_range`: minimum distance in blocks the target must be for this spell to activate. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_activation_range: Option<StrictFloat<f32>>,
    /// `particle_color`: the color of the particles spawned while casting this spell. Default white.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub particle_color: Option<ParticleColor>,
    /// `sequence`: list of events to send.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sequence: Option<Vec<SendEventStage>>,
    /// `start_sound_event`: the sound event to play when this spell starts. Default `""`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub start_sound_event: Option<SoundEvent>,
    /// `weight`: relative weight of this spell choice when randomly selected from valid choices. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub weight: Option<StrictFloat<f32>>,
    /// Members the entry does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl SendEventChoice {
    /// `cast_duration`, or its default 0.
    pub fn cast_duration(&self) -> f32 {
        self.cast_duration.map_or(0.0, |v| v.0)
    }

    /// `cooldown_time`, or its default 0.
    pub fn cooldown_time(&self) -> f32 {
        self.cooldown_time.map_or(0.0, |v| v.0)
    }

    /// `do_casting`, or its default `true`.
    pub fn do_casting(&self) -> bool {
        self.do_casting.is_none_or(|v| v.0)
    }

    /// `look_at_target`, or its default `true`.
    pub fn look_at_target(&self) -> bool {
        self.look_at_target.is_none_or(|v| v.0)
    }

    /// `max_activation_range`, or its default 32.
    pub fn max_activation_range(&self) -> f32 {
        self.max_activation_range.map_or(32.0, |v| v.0)
    }

    /// `min_activation_range`, or its default 1.
    pub fn min_activation_range(&self) -> f32 {
        self.min_activation_range.map_or(1.0, |v| v.0)
    }

    /// `start_sound_event`, or its default `""`.
    pub fn start_sound_event(&self) -> &str {
        self.start_sound_event.as_ref().map_or("", SoundEvent::name)
    }

    /// `weight`, or its default 0.
    pub fn weight(&self) -> f32 {
        self.weight.map_or(0.0, |v| v.0)
    }

    /// Checks the entry's members.
    pub(crate) fn check(&mut self, c: &mut Check<'_>) {
        c.filter("filters", self.filters.as_ref());
        c.each("sequence", &mut self.sequence, SendEventStage::check);
        c.sound("start_sound_event", self.start_sound_event.as_ref());
    }
}

/// One step of a `send_event` choice's `sequence`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct SendEventStage {
    /// `base_delay`: amount of time in seconds before starting this step. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub base_delay: Option<StrictFloat<f32>>,
    /// `event`: the event to send to the entity. Default `""`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub event: Option<StrictString>,
    /// `sound_event`: the sound event to play when this step happens. Default `""`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sound_event: Option<SoundEvent>,
    /// Members the entry does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl SendEventStage {
    /// `base_delay`, or its default 0.
    pub fn base_delay(&self) -> f32 {
        self.base_delay.map_or(0.0, |v| v.0)
    }

    /// `event`, or its default `""`.
    pub fn event(&self) -> &str {
        self.event.as_ref().map_or("", |v| v.0.as_str())
    }

    /// `sound_event`, or its default `""`.
    pub fn sound_event(&self) -> &str {
        self.sound_event.as_ref().map_or("", SoundEvent::name)
    }

    /// Checks the entry's members.
    pub(crate) fn check(&mut self, c: &mut Check<'_>) {
        c.sound("sound_event", self.sound_event.as_ref());
    }
}
