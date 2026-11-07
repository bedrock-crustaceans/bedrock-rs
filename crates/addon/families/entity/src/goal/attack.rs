//! Attack goals: melee, ranged and special attacks.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::schema::{self, GoalMeta, Schema};
use bedrock_addon_core::shared::{EventTrigger, Filter, FloatRange};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use facet::Facet;
use facet_value::VObject;

use super::check::{
    Bound, Deprecated, Goal, GoalCheck, V1_26_0, V1_26_40, V1_26_50, deprecated_text, float_of,
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct AquaticChargeAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `attack_reach`: horizontal reach grown around the mob's AABB to register a hit. Default 0.05; at least 0.
    #[facet(schema::fallback = 0.05)]
    pub attack_reach: Option<f32>,
    /// `charge_cooldown_time`: range of time in seconds to wait before starting another charge. Default 2 to 6.
    pub charge_cooldown_time: Option<FloatRange>,
    /// `charge_overshoot_distance`: distance beyond the target the mob aims during a charge. Default 1.5; at least 0.
    #[facet(schema::fallback = 1.5)]
    pub charge_overshoot_distance: Option<f32>,
    /// `charge_speed`: absolute speed used during the charge attack. Default 0.06; at least 0.05.
    #[facet(schema::fallback = 0.06)]
    pub charge_speed: Option<f32>,
    /// `knockback_force`: knockback force applied to the target on hit. Default 2; at least 0.
    #[facet(schema::fallback = 2.0)]
    pub knockback_force: Option<f32>,
    /// `max_charge_distance`: maximum distance at which the mob attempts a charge. Default 16; at least 0.
    #[facet(schema::fallback = 16.0)]
    pub max_charge_distance: Option<f32>,
}

impl AquaticChargeAttack {
    /// `charge_cooldown_time`, or its default 2 to 6.
    pub fn charge_cooldown_time(&self) -> (f32, f32) {
        self.charge_cooldown_time
            .map_or((2.0, 6.0), |range| range.resolve(2.0, 6.0))
    }
}

impl Goal for AquaticChargeAttack {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    const DEPRECATED: &'static [Deprecated] = &[(
        "charge_speed_multiplier",
        deprecated_text!("charge_speed_multiplier"),
        known::V1_26_0,
    )];

    fn check(&mut self, c: &mut GoalCheck<'_>) {
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
            self.charge_speed = float_of(value);
        }
    }

    fn upgrade(&mut self, from: FormatVersion, d: &mut Diagnostics) {
        // below 1.26.0 `charge_speed_multiplier` is the live name.
        if from < V1_26_0 {
            convert::charge_speed_typed(&mut self.charge_speed, d.unknown_mut());
        }
    }

    fn lenient_shape(json: &mut VObject, _: &mut Diagnostics) {
        convert::charge_speed(json);
    }
}

/// `minecraft:behavior.charge_attack`: allows this entity to damage a target by using a running attack.
// `behavior.charge_attack`: typed from 1.21.100.
#[doc(alias = "minecraft:behavior.charge_attack")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct ChargeAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `max_distance`: a charge attack cannot start if the entity is farther than this distance to the target. Default 3.
    #[facet(schema::fallback = 3.0)]
    pub max_distance: Option<f32>,
    /// `min_distance`: a charge attack cannot start if the entity is closer than this distance to the target. Default 2.
    #[facet(schema::fallback = 2.0)]
    pub min_distance: Option<f32>,
    /// `sound_event`: default `charge`.
    pub sound_event: Option<SoundEvent>,
    /// `speed_multiplier`: modifies the entity's speed when charging toward the target. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
    /// `success_rate`: percent chance this entity will start a charge attack, if not already attacking (1.0 = 100%). Default 0.1428.
    #[facet(schema::fallback = 0.1428)]
    pub success_rate: Option<f32>,
}

impl ChargeAttack {
    /// `sound_event`, or its default `charge`.
    pub fn sound_event(&self) -> &str {
        self.sound_event.as_ref().map_or("charge", SoundEvent::name)
    }
}

impl Goal for ChargeAttack {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
        c.sound("sound_event", self.sound_event.as_ref());
    }
}

/// `minecraft:behavior.delayed_attack`: allows an entity to attack, while also delaying the damage-dealt until a specific time in the attack animation.
///
/// Deprecated members, reported and dropped: `cooldown_time`.
// `behavior.delayed_attack`: typed from 1.26.0, redeclared 1.26.40.
#[doc(alias = "minecraft:behavior.delayed_attack")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct DelayedAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `attack_duration`: the entity's attack animation will play out over this duration (in seconds). Also controls attack cooldown. Default 0.75; at least 0.
    pub attack_duration: Option<f32>,
    /// `attack_once`: allows the mob to perform this melee attack behavior only once during its lifetime. Default `false`.
    pub attack_once: Option<bool>,
    /// `attack_types`: defines the entity types this entity will attack. Default `""`.
    pub attack_types: Option<String>,
    /// `can_spread_on_fire`: allows the mob, if on fire and empty handed, to ignite its target upon a successful attack. Default `false`.
    pub can_spread_on_fire: Option<bool>,
    /// `hit_delay_pct`: the percentage into the attack animation to apply the damage of the attack (1.0 = 100%). Default 0.5; 0 to 1.
    pub hit_delay_pct: Option<f32>,
    /// `inner_boundary_time_increase`: time, in seconds, added to the attack path recalculation interval when the target is beyond `path_inner_boundary`. Default 0.25; at least 0.
    pub inner_boundary_time_increase: Option<f32>,
    /// `max_path_time`: maximum base time, in seconds, before recalculating a new attack path to the target (before increases are applied). Default 0.55; at least 0.
    pub max_path_time: Option<f32>,
    /// `melee_fov`: field of view, in degrees, used by the hard-coded sensing component to detect a valid attack target. Default 9e+01; at least 0.
    pub melee_fov: Option<f32>,
    /// `min_path_time`: minimum base time, in seconds, before recalculating a new attack path to the target (before increases are applied). Default 0.2; at least 0.
    pub min_path_time: Option<f32>,
    /// `on_attack`: defines the event to trigger when this entity successfully attacks.
    pub on_attack: Option<EventTrigger>,
    /// `on_kill`: defines the event to trigger when this entity kills the target.
    pub on_kill: Option<EventTrigger>,
    /// `outer_boundary_time_increase`: time, in seconds, added to the attack path recalculation interval when the target is beyond `path_outer_boundary`. Default 0.5; at least 0.
    pub outer_boundary_time_increase: Option<f32>,
    /// `path_fail_time_increase`: time, in seconds, added to the attack path recalculation interval when the mob cannot move along the current path. Default 0.75; at least 0.
    pub path_fail_time_increase: Option<f32>,
    /// `path_inner_boundary`: distance at which to increase attack path recalculation by `inner_boundary_time_increase`. Default 16; at least 0.
    pub path_inner_boundary: Option<f32>,
    /// `path_outer_boundary`: distance at which to increase attack path recalculation by `outer_boundary_time_increase`. Default 32; at least 0.
    pub path_outer_boundary: Option<f32>,
    /// `random_stop_interval`: defines a 1-in-N chance for the mob to stop its current attack, where N equals `random_stop_interval`. Default 0; at least 0.
    pub random_stop_interval: Option<i32>,
    /// `reach_multiplier`: used with the base size of the entity to determine minimum target-distance before trying to deal attack damage. Default 1.5; at least 0.
    pub reach_multiplier: Option<f32>,
    /// `require_complete_path`: specifies whether a full navigation path from the mob to the target is required. Default `false`.
    pub require_complete_path: Option<bool>,
    /// `sound_event`: default `attack.strong`.
    pub sound_event: Option<SoundEvent>,
    /// `speed_multiplier`: multiplier applied to the mob's movement speed when moving toward its target. Default 1; at least 0.
    pub speed_multiplier: Option<f32>,
    /// `track_target`: allows the mob to track its target even if it lacks a hard-coded sensing component. Default `true`.
    pub track_target: Option<bool>,
    /// `x_max_rotation`: maximum rotation, in degrees, on the X-axis while the mob is trying to look at its target. Default 3e+01; at least 0.
    pub x_max_rotation: Option<f32>,
    /// `y_max_head_rotation`: maximum rotation, in degrees, on the Y-axis while the mob is trying to look at its target. Default 3e+01; at least 0.
    pub y_max_head_rotation: Option<f32>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl DelayedAttack {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `attack_duration`, or its default 0.75.
    pub fn attack_duration(&self) -> f32 {
        self.attack_duration.unwrap_or(0.75)
    }

    /// `attack_once`, or its default `false`.
    pub fn attack_once(&self) -> bool {
        self.attack_once.is_some_and(|v| v)
    }

    /// `attack_types`, or its default `""`.
    pub fn attack_types(&self) -> &str {
        self.attack_types.as_ref().map_or("", |v| v.as_str())
    }

    /// `can_spread_on_fire`, or its default `false`.
    pub fn can_spread_on_fire(&self) -> bool {
        self.can_spread_on_fire.is_some_and(|v| v)
    }

    /// `hit_delay_pct`, or its default 0.5.
    pub fn hit_delay_pct(&self) -> f32 {
        self.hit_delay_pct.unwrap_or(0.5)
    }

    /// `inner_boundary_time_increase`, or its default 0.25.
    pub fn inner_boundary_time_increase(&self) -> f32 {
        self.inner_boundary_time_increase.unwrap_or(0.25)
    }

    /// `max_path_time`, or its default 0.55.
    pub fn max_path_time(&self) -> f32 {
        self.max_path_time.unwrap_or(0.55)
    }

    /// `melee_fov`, or its default 9e+01.
    pub fn melee_fov(&self) -> f32 {
        self.melee_fov.unwrap_or(9.0e1)
    }

    /// `min_path_time`, or its default 0.2.
    pub fn min_path_time(&self) -> f32 {
        self.min_path_time.unwrap_or(0.2)
    }

    /// `outer_boundary_time_increase`, or its default 0.5.
    pub fn outer_boundary_time_increase(&self) -> f32 {
        self.outer_boundary_time_increase.unwrap_or(0.5)
    }

    /// `path_fail_time_increase`, or its default 0.75.
    pub fn path_fail_time_increase(&self) -> f32 {
        self.path_fail_time_increase.unwrap_or(0.75)
    }

    /// `path_inner_boundary`, or its default 16.
    pub fn path_inner_boundary(&self) -> f32 {
        self.path_inner_boundary.unwrap_or(16.0)
    }

    /// `path_outer_boundary`, or its default 32.
    pub fn path_outer_boundary(&self) -> f32 {
        self.path_outer_boundary.unwrap_or(32.0)
    }

    /// `random_stop_interval`, or its default 0.
    pub fn random_stop_interval(&self) -> i32 {
        self.random_stop_interval.unwrap_or(0)
    }

    /// `reach_multiplier`, or its default 1.5.
    pub fn reach_multiplier(&self) -> f32 {
        self.reach_multiplier.unwrap_or(1.5)
    }

    /// `require_complete_path`, or its default `false`.
    pub fn require_complete_path(&self) -> bool {
        self.require_complete_path.is_some_and(|v| v)
    }

    /// `sound_event`, or its default `attack.strong`.
    pub fn sound_event(&self) -> &str {
        self.sound_event
            .as_ref()
            .map_or("attack.strong", SoundEvent::name)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.unwrap_or(1.0)
    }

    /// `track_target`, or its default `true`.
    pub fn track_target(&self) -> bool {
        self.track_target.is_none_or(|v| v)
    }

    /// `x_max_rotation`, or its default 3e+01.
    pub fn x_max_rotation(&self) -> f32 {
        self.x_max_rotation.unwrap_or(3.0e1)
    }

    /// `y_max_head_rotation`, or its default 3e+01.
    pub fn y_max_head_rotation(&self) -> f32 {
        self.y_max_head_rotation.unwrap_or(3.0e1)
    }
}

impl Goal for DelayedAttack {
    const TYPED: FormatVersion = known::V1_26_0;

    const DEPRECATED: &'static [Deprecated] = &[(
        "cooldown_time",
        deprecated_text!("cooldown_time"),
        known::V1_26_0,
    )];

    fn check(&mut self, c: &mut GoalCheck<'_>) {
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct EatMob {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `eat_animation_time`: sets the time in seconds the eat animation should play for. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub eat_animation_time: Option<f32>,
    /// `eat_mob_sound`: sets the sound that should play when eating a mob. Default `""`.
    pub eat_mob_sound: Option<SoundEvent>,
    /// `loot_table`: the loot table for loot to be dropped when eating a mob. Default `""`.
    pub loot_table: Option<String>,
    /// `pull_in_force`: sets the force which the mob-to-be-eaten is pulled towards the eating mob. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub pull_in_force: Option<f32>,
    /// `reach_mob_distance`: sets the desired distance to be reached before eating the mob. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub reach_mob_distance: Option<f32>,
    /// `run_speed`: sets the entity's speed when running toward the target. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub run_speed: Option<f32>,
}

impl EatMob {
    /// `eat_mob_sound`, or its default `""`.
    pub fn eat_mob_sound(&self) -> &str {
        self.eat_mob_sound.as_ref().map_or("", SoundEvent::name)
    }

    /// `loot_table`, or its default `""`.
    pub fn loot_table(&self) -> &str {
        self.loot_table.as_ref().map_or("", |v| v.as_str())
    }
}

impl Goal for EatMob {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
        c.sound("eat_mob_sound", self.eat_mob_sound.as_ref());
    }
}

/// `minecraft:behavior.fire_at_target`: allows an entity to attack by firing a shot with a delay. Anchor and offset parameters of this component overrides the anchor and offset from projectile component.
// `behavior.fire_at_target`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.fire_at_target")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct FireAtTarget {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `attack_cooldown`: the cooldown time in seconds before this goal can be used again. Default 0.5; at least 0.
    #[facet(schema::fallback = 0.5)]
    pub attack_cooldown: Option<f32>,
    /// `attack_range`: target needs to be within this range for the attack to happen. Default 0 to 0.
    pub attack_range: Option<FloatRange>,
    /// `filters`: conditions that need to be met for the behavior to start.
    pub filters: Option<Filter>,
    /// `max_head_rotation_x`: maximum head rotation (in degrees), on the X-axis, that this entity can apply while trying to look at the target. Default 3e+01.
    #[facet(schema::fallback = 3.0e1)]
    pub max_head_rotation_x: Option<f32>,
    /// `max_head_rotation_y`: maximum head rotation (in degrees), on the Y-axis, that this entity can apply while trying to look at the target. Default 3e+01.
    #[facet(schema::fallback = 3.0e1)]
    pub max_head_rotation_y: Option<f32>,
    /// `owner_anchor`: entity anchor for the projectile spawn location. Default 2.
    #[facet(schema::fallback = 2)]
    pub owner_anchor: Option<i32>,
    /// `owner_offset`: offset vector from the `owner_anchor`. Default `[0, 0, 0]`.
    pub owner_offset: Option<Vec3>,
    /// `post_shoot_delay`: time in seconds between firing the projectile and ending the goal. Default 0.2; at least 0.
    #[facet(schema::fallback = 0.2)]
    pub post_shoot_delay: Option<f32>,
    /// `pre_shoot_delay`: time in seconds before firing the projectile. Default 0.75; at least 0.
    #[facet(schema::fallback = 0.75)]
    pub pre_shoot_delay: Option<f32>,
    /// `projectile_def`: actor definition to use as projectile for the ranged attack. The actor must be a projectile. This field is required for the goal to be usable. Default `""`.
    pub projectile_def: Option<String>,
    /// `ranged_fov`: field of view (in degrees) when using sensing to detect a target for attack. Default 9e+01.
    #[facet(schema::fallback = 9.0e1)]
    pub ranged_fov: Option<f32>,
    /// `target_anchor`: entity anchor for projectile target. Default 2.
    #[facet(schema::fallback = 2)]
    pub target_anchor: Option<i32>,
    /// `target_offset`: offset vector from the `target_anchor`. Default `[0, 0, 0]`.
    pub target_offset: Option<Vec3>,
}

impl FireAtTarget {
    /// `attack_range`, or its default 0 to 0.
    pub fn attack_range(&self) -> (f32, f32) {
        self.attack_range
            .map_or((0.0, 0.0), |range| range.resolve(0.0, 0.0))
    }

    /// `owner_offset`, or its default `[0, 0, 0]`.
    pub fn owner_offset(&self) -> [f32; 3] {
        self.owner_offset
            .as_ref()
            .map_or([0.0, 0.0, 0.0], |v| [v.x, v.y, v.z])
    }

    /// `projectile_def`, or its default `""`.
    pub fn projectile_def(&self) -> &str {
        self.projectile_def.as_ref().map_or("", |v| v.as_str())
    }

    /// `target_offset`, or its default `[0, 0, 0]`.
    pub fn target_offset(&self) -> [f32; 3] {
        self.target_offset
            .as_ref()
            .map_or([0.0, 0.0, 0.0], |v| [v.x, v.y, v.z])
    }
}

impl Goal for FireAtTarget {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
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

/// `minecraft:behavior.guardian_attack`: allows this entity to use a laser beam attack. Can only be used by Guardians and Elder Guardians.
// `behavior.guardian_attack`: typed from 1.26.0.
#[doc(alias = "minecraft:behavior.guardian_attack")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct GuardianAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `elder_extra_magic_damage`: amount of additional damage dealt from an elder guardian's magic attack. Default 2; greater than 0.
    #[facet(schema::fallback = 2, schema::above = 0)]
    pub elder_extra_magic_damage: Option<i32>,
    /// `hard_mode_extra_magic_damage`: in hard difficulty, amount of additional damage dealt from a guardian's magic attack. Default 2; greater than 0.
    #[facet(schema::fallback = 2, schema::above = 0)]
    pub hard_mode_extra_magic_damage: Option<i32>,
    /// `magic_damage`: amount of damage dealt from a guardian's magic attack. Magic attack damage is added to the guardian's base attack damage. Default 1; greater than 0.
    #[facet(schema::fallback = 1, schema::above = 0)]
    pub magic_damage: Option<i32>,
    /// `min_distance`: guardian attack behavior stops if the target is closer than this distance (doesn't apply to elders). Default 3; at least 0.
    #[facet(schema::fallback = 3.0, schema::min = 0.0)]
    pub min_distance: Option<f32>,
    /// `sound_delay_time`: time (in seconds) to wait after starting an attack before playing the guardian attack sound. Default 0.5; greater than 0.
    #[facet(schema::fallback = 0.5, schema::above = 0.0)]
    pub sound_delay_time: Option<f32>,
    /// `x_max_rotation`: maximum rotation (in degrees), on the X-axis, this entity can rotate while trying to look at the target. Default 9e+01; greater than 0.
    #[facet(schema::fallback = 9.0e1, schema::above = 0.0)]
    pub x_max_rotation: Option<f32>,
    /// `y_max_head_rotation`: maximum rotation (in degrees), on the Y-axis, this entity can rotate its head while trying to look at the target. Default 9e+01; greater than 0.
    #[facet(schema::fallback = 9.0e1, schema::above = 0.0)]
    pub y_max_head_rotation: Option<f32>,
}

impl Goal for GuardianAttack {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.jump_around_target`: allows an entity to jump around a target.
// `behavior.jump_around_target`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.jump_around_target")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct JumpAroundTarget {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `check_collision`: enables collision checks when calculating the jump. Setting `check_collision` to true may affect performance and should be used with care. Default `false`.
    #[facet(schema::fallback = false)]
    pub check_collision: Option<bool>,
    /// `entity_bounding_box_scale`: scaling temporarily applied to the entity's AABB bounds when jumping. A smaller bounding box reduces the risk of collisions during the jump. When `check_collision` is true it also increases the chance of being able to jump when close to obstacles. Default 0.7.
    #[facet(schema::fallback = 0.7)]
    pub entity_bounding_box_scale: Option<f32>,
    /// `filters`: conditions that need to be met for the behavior to start.
    pub filters: Option<Filter>,
    /// `jump_angles`: the jump angles in float degrees that are allowed when performing the jump. The order in which the angles are chosen is randomized.
    pub jump_angles: Option<Vec<f32>>,
    /// `jump_cooldown_duration`: the time in seconds to spend in cooldown before this goal can be used again. Default 0.5.
    #[facet(schema::fallback = 0.5)]
    pub jump_cooldown_duration: Option<f32>,
    /// `jump_cooldown_when_hurt_duration`: the time in seconds to spend in cooldown after being hurt before this goal can be used again. Default 0.1.
    #[facet(schema::fallback = 0.1)]
    pub jump_cooldown_when_hurt_duration: Option<f32>,
    /// `landing_distance_from_target`: the range deciding how close to and how far away from the target the landing position can be when jumping. Default 4 to 8.
    pub landing_distance_from_target: Option<FloatRange>,
    /// `landing_position_spread_degrees`: this angle (in degrees) is used for controlling the spread when picking a landing position behind the target. A zero spread angle means the landing position will be straight behind the target with no variance. A 90 degree spread angle means the landing position can be up to 45 degrees to the left and to the right of the position straight behind the target's view direction. Default 90.
    #[facet(schema::fallback = 90)]
    pub landing_position_spread_degrees: Option<i32>,
    /// `last_hurt_duration`: if the entity was hurt within these last seconds, the `jump_cooldown_when_hurt_duration` will be used instead of `jump_cooldown_duration`. Default 2.
    #[facet(schema::fallback = 2.0)]
    pub last_hurt_duration: Option<f32>,
    /// `line_of_sight_obstruction_height_ignore`: if the entity's line of sight towards its target is obstructed by an obstacle with a height below this number, the obstacle will be ignored, and the goal will try to find a valid landing position. Default 4.
    #[facet(schema::fallback = 4)]
    pub line_of_sight_obstruction_height_ignore: Option<i32>,
    /// `max_jump_velocity`: maximum velocity a jump can be performed at. Default 1.4.
    #[facet(schema::fallback = 1.4)]
    pub max_jump_velocity: Option<f32>,
    /// `prepare_jump_duration`: the time in seconds to spend preparing for the jump. Default 0.5.
    #[facet(schema::fallback = 0.5)]
    pub prepare_jump_duration: Option<f32>,
    /// `required_vertical_space`: the number of blocks above the entity's head that has to be air for this goal to be usable. Default 4.
    #[facet(schema::fallback = 4)]
    pub required_vertical_space: Option<i32>,
    /// `snap_to_surface_block_range`: the number of blocks above and below from the jump target position that will be checked to find a surface to land on. Default 10.
    #[facet(schema::fallback = 10)]
    pub snap_to_surface_block_range: Option<i32>,
    /// `valid_distance_to_target`: target needs to be within this range for the jump to happen. Default 4 to 2e+01.
    pub valid_distance_to_target: Option<FloatRange>,
}

impl JumpAroundTarget {
    /// `landing_distance_from_target`, or its default 4 to 8.
    pub fn landing_distance_from_target(&self) -> (f32, f32) {
        self.landing_distance_from_target
            .map_or((4.0, 8.0), |range| range.resolve(4.0, 8.0))
    }

    /// `valid_distance_to_target`, or its default 4 to 2e+01.
    pub fn valid_distance_to_target(&self) -> (f32, f32) {
        self.valid_distance_to_target
            .map_or((4.0, 2.0e1), |range| range.resolve(4.0, 2.0e1))
    }
}

impl Goal for JumpAroundTarget {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
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

/// `minecraft:behavior.knockback_roar`: allows the mob to perform a damaging knockback that affects all nearby entities.
// `behavior.knockback_roar`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.knockback_roar")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct KnockbackRoar {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `attack_time`: the delay after which the knockback occurs (in seconds). Default 0.5.
    pub attack_time: Option<f32>,
    /// `cooldown_time`: time (in seconds) the mob has to wait before using the goal again. Default 0.1.
    pub cooldown_time: Option<f32>,
    /// `damage_filters`: the list of conditions another entity must meet to be a valid target to apply damage to.
    pub damage_filters: Option<Filter>,
    /// `duration`: the max duration of the roar (in seconds). Default 1.
    pub duration: Option<f32>,
    /// `knockback_damage`: the damage dealt by the knockback roar. Default 6.
    pub knockback_damage: Option<i32>,
    /// `knockback_filters`: the list of conditions another entity must meet to be a valid target to apply knockback to.
    pub knockback_filters: Option<Filter>,
    /// `knockback_height_cap`: the maximum height for vertical knockback. Default 0.4.
    pub knockback_height_cap: Option<f32>,
    /// `knockback_horizontal_strength`: the strength of the horizontal knockback. Default 4.
    pub knockback_horizontal_strength: Option<i32>,
    /// `knockback_range`: the radius (in blocks) of the knockback effect. Default 4.
    pub knockback_range: Option<i32>,
    /// `knockback_vertical_strength`: the strength of the vertical knockback. Default 4.
    pub knockback_vertical_strength: Option<i32>,
    /// `on_roar_end`: event that is triggered when the roar ends.
    pub on_roar_end: Option<EventTrigger>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl KnockbackRoar {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `attack_time`, or its default 0.5.
    pub fn attack_time(&self) -> f32 {
        self.attack_time.unwrap_or(0.5)
    }

    /// `cooldown_time`, or its default 0.1.
    pub fn cooldown_time(&self) -> f32 {
        self.cooldown_time.unwrap_or(0.1)
    }

    /// `duration`, or its default 1.
    pub fn duration(&self) -> f32 {
        self.duration.unwrap_or(1.0)
    }

    /// `knockback_damage`, or its default 6.
    pub fn knockback_damage(&self) -> i32 {
        self.knockback_damage.unwrap_or(6)
    }

    /// `knockback_height_cap`, or its default 0.4.
    pub fn knockback_height_cap(&self) -> f32 {
        self.knockback_height_cap.unwrap_or(0.4)
    }

    /// `knockback_horizontal_strength`, or its default 4.
    pub fn knockback_horizontal_strength(&self) -> i32 {
        self.knockback_horizontal_strength.unwrap_or(4)
    }

    /// `knockback_range`, or its default 4.
    pub fn knockback_range(&self) -> i32 {
        self.knockback_range.unwrap_or(4)
    }

    /// `knockback_vertical_strength`, or its default 4.
    pub fn knockback_vertical_strength(&self) -> i32 {
        self.knockback_vertical_strength.unwrap_or(4)
    }
}

impl Goal for KnockbackRoar {
    const TYPED: FormatVersion = known::V1_26_40;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct LeapAtTarget {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `must_be_on_ground`: if true, the mob will only jump at its target if its on the ground. Setting it to false will allow it to jump even if its already in the air. Default `true`.
    pub must_be_on_ground: Option<bool>,
    /// `set_persistent`: allows the actor to be set to persist upon targeting a player. Default `false`.
    pub set_persistent: Option<bool>,
    /// `yd`: the height in blocks the mob jumps when leaping at its target. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub yd: Option<f32>,
}

impl LeapAtTarget {
    /// `must_be_on_ground`, or its default `true`.
    pub fn must_be_on_ground(&self) -> bool {
        self.must_be_on_ground.is_none_or(|v| v)
    }

    /// `set_persistent`, or its default `false`.
    pub fn set_persistent(&self) -> bool {
        self.set_persistent.is_some_and(|v| v)
    }
}

impl Goal for LeapAtTarget {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.melee_attack`: allows an entity to deal damage through a melee attack.
// `behavior.melee_attack`: typed from 1.26.0, redeclared 1.26.40.
#[doc(alias = "minecraft:behavior.melee_attack")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct MeleeAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `attack_once`: allows the mob to perform this melee attack behavior only once during its lifetime. Default `false`.
    pub attack_once: Option<bool>,
    /// `attack_types`: defines the entity types this entity will attack. Default `""`.
    pub attack_types: Option<String>,
    /// `can_spread_on_fire`: allows the mob, if on fire and empty handed, to ignite its target upon a successful attack. Default `false`.
    pub can_spread_on_fire: Option<bool>,
    /// `cooldown_time`: cooldown time, in seconds, between consecutive attacks. Default 1; at least 0.
    pub cooldown_time: Option<f32>,
    /// `inner_boundary_time_increase`: time, in seconds, added to the attack path recalculation interval when the target is beyond `path_inner_boundary`. Default 0.25; at least 0.
    pub inner_boundary_time_increase: Option<f32>,
    /// `max_path_time`: maximum base time, in seconds, before recalculating a new attack path to the target (before increases are applied). Default 0.55; at least 0.
    pub max_path_time: Option<f32>,
    /// `melee_fov`: field of view, in degrees, used by the hard-coded sensing component to detect a valid attack target. Default 9e+01; at least 0.
    pub melee_fov: Option<f32>,
    /// `min_path_time`: minimum base time, in seconds, before recalculating a new attack path to the target (before increases are applied). Default 0.2; at least 0.
    pub min_path_time: Option<f32>,
    /// `on_attack`: defines the event to trigger when this entity successfully attacks.
    pub on_attack: Option<EventTrigger>,
    /// `on_kill`: defines the event to trigger when this entity kills the target.
    pub on_kill: Option<EventTrigger>,
    /// `outer_boundary_time_increase`: time, in seconds, added to the attack path recalculation interval when the target is beyond `path_outer_boundary`. Default 0.5; at least 0.
    pub outer_boundary_time_increase: Option<f32>,
    /// `path_fail_time_increase`: time, in seconds, added to the attack path recalculation interval when the mob cannot move along the current path. Default 0.75; at least 0.
    pub path_fail_time_increase: Option<f32>,
    /// `path_inner_boundary`: distance at which to increase attack path recalculation by `inner_boundary_time_increase`. Default 16; at least 0.
    pub path_inner_boundary: Option<f32>,
    /// `path_outer_boundary`: distance at which to increase attack path recalculation by `outer_boundary_time_increase`. Default 32; at least 0.
    pub path_outer_boundary: Option<f32>,
    /// `random_stop_interval`: defines a 1-in-N chance for the mob to stop its current attack, where N equals `random_stop_interval`. Default 0; at least 0.
    pub random_stop_interval: Option<i32>,
    /// `reach_multiplier`: used with the base size of the entity to determine minimum target-distance before trying to deal attack damage. Default 2; at least 0.
    pub reach_multiplier: Option<f32>,
    /// `require_complete_path`: specifies whether a full navigation path from the mob to the target is required. Default `false`.
    pub require_complete_path: Option<bool>,
    /// `speed_multiplier`: multiplier applied to the mob's movement speed when moving toward its target. Default 1; at least 0.
    pub speed_multiplier: Option<f32>,
    /// `track_target`: allows the mob to track its target even if it lacks a hard-coded sensing component. Default `false`.
    pub track_target: Option<bool>,
    /// `x_max_rotation`: maximum rotation, in degrees, on the X-axis while the mob is trying to look at its target. Default 3e+01; at least 0.
    pub x_max_rotation: Option<f32>,
    /// `y_max_head_rotation`: maximum rotation, in degrees, on the Y-axis while the mob is trying to look at its target. Default 3e+01; at least 0.
    pub y_max_head_rotation: Option<f32>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl MeleeAttack {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `attack_once`, or its default `false`.
    pub fn attack_once(&self) -> bool {
        self.attack_once.is_some_and(|v| v)
    }

    /// `attack_types`, or its default `""`.
    pub fn attack_types(&self) -> &str {
        self.attack_types.as_ref().map_or("", |v| v.as_str())
    }

    /// `can_spread_on_fire`, or its default `false`.
    pub fn can_spread_on_fire(&self) -> bool {
        self.can_spread_on_fire.is_some_and(|v| v)
    }

    /// `cooldown_time`, or its default 1.
    pub fn cooldown_time(&self) -> f32 {
        self.cooldown_time.unwrap_or(1.0)
    }

    /// `inner_boundary_time_increase`, or its default 0.25.
    pub fn inner_boundary_time_increase(&self) -> f32 {
        self.inner_boundary_time_increase.unwrap_or(0.25)
    }

    /// `max_path_time`, or its default 0.55.
    pub fn max_path_time(&self) -> f32 {
        self.max_path_time.unwrap_or(0.55)
    }

    /// `melee_fov`, or its default 9e+01.
    pub fn melee_fov(&self) -> f32 {
        self.melee_fov.unwrap_or(9.0e1)
    }

    /// `min_path_time`, or its default 0.2.
    pub fn min_path_time(&self) -> f32 {
        self.min_path_time.unwrap_or(0.2)
    }

    /// `outer_boundary_time_increase`, or its default 0.5.
    pub fn outer_boundary_time_increase(&self) -> f32 {
        self.outer_boundary_time_increase.unwrap_or(0.5)
    }

    /// `path_fail_time_increase`, or its default 0.75.
    pub fn path_fail_time_increase(&self) -> f32 {
        self.path_fail_time_increase.unwrap_or(0.75)
    }

    /// `path_inner_boundary`, or its default 16.
    pub fn path_inner_boundary(&self) -> f32 {
        self.path_inner_boundary.unwrap_or(16.0)
    }

    /// `path_outer_boundary`, or its default 32.
    pub fn path_outer_boundary(&self) -> f32 {
        self.path_outer_boundary.unwrap_or(32.0)
    }

    /// `random_stop_interval`, or its default 0.
    pub fn random_stop_interval(&self) -> i32 {
        self.random_stop_interval.unwrap_or(0)
    }

    /// `reach_multiplier`, or its default 2.
    pub fn reach_multiplier(&self) -> f32 {
        self.reach_multiplier.unwrap_or(2.0)
    }

    /// `require_complete_path`, or its default `false`.
    pub fn require_complete_path(&self) -> bool {
        self.require_complete_path.is_some_and(|v| v)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.unwrap_or(1.0)
    }

    /// `track_target`, or its default `false`.
    pub fn track_target(&self) -> bool {
        self.track_target.is_some_and(|v| v)
    }

    /// `x_max_rotation`, or its default 3e+01.
    pub fn x_max_rotation(&self) -> f32 {
        self.x_max_rotation.unwrap_or(3.0e1)
    }

    /// `y_max_head_rotation`, or its default 3e+01.
    pub fn y_max_head_rotation(&self) -> f32 {
        self.y_max_head_rotation.unwrap_or(3.0e1)
    }
}

impl Goal for MeleeAttack {
    const TYPED: FormatVersion = known::V1_26_0;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
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
#[facet(schema::unknown = "keep")]
pub struct MeleeBoxAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `attack_once`: allows the mob to perform this melee attack behavior only once during its lifetime. Default `false`.
    pub attack_once: Option<bool>,
    /// `attack_types`: defines the entity types this entity will attack. Default `""`.
    pub attack_types: Option<String>,
    /// `can_spread_on_fire`: allows the mob, if on fire and empty handed, to ignite its target upon a successful attack. Default `false`.
    pub can_spread_on_fire: Option<bool>,
    /// `cooldown_time`: cooldown time, in seconds, between consecutive attacks. Default 1; at least 0.
    pub cooldown_time: Option<f32>,
    /// `horizontal_reach`: the attack reach of the mob will be a box with the size of the mobs bounds increased by this value in all horizontal directions. Default 0.8; at least 0.
    pub horizontal_reach: Option<f32>,
    /// `inner_boundary_time_increase`: time, in seconds, added to the attack path recalculation interval when the target is beyond `path_inner_boundary`. Default 0.25; at least 0.
    pub inner_boundary_time_increase: Option<f32>,
    /// `max_path_time`: maximum base time, in seconds, before recalculating a new attack path to the target (before increases are applied). Default 0.55; at least 0.
    pub max_path_time: Option<f32>,
    /// `melee_fov`: field of view, in degrees, used by the hard-coded sensing component to detect a valid attack target. Default 9e+01; at least 0.
    pub melee_fov: Option<f32>,
    /// `min_path_time`: minimum base time, in seconds, before recalculating a new attack path to the target (before increases are applied). Default 0.2; at least 0.
    pub min_path_time: Option<f32>,
    /// `on_attack`: defines the event to trigger when this entity successfully attacks.
    pub on_attack: Option<EventTrigger>,
    /// `on_kill`: defines the event to trigger when this entity kills the target.
    pub on_kill: Option<EventTrigger>,
    /// `outer_boundary_time_increase`: time, in seconds, added to the attack path recalculation interval when the target is beyond `path_outer_boundary`. Default 0.5; at least 0.
    pub outer_boundary_time_increase: Option<f32>,
    /// `path_fail_time_increase`: time, in seconds, added to the attack path recalculation interval when the mob cannot move along the current path. Default 0.75; at least 0.
    pub path_fail_time_increase: Option<f32>,
    /// `path_inner_boundary`: distance at which to increase attack path recalculation by `inner_boundary_time_increase`. Default 16; at least 0.
    pub path_inner_boundary: Option<f32>,
    /// `path_outer_boundary`: distance at which to increase attack path recalculation by `outer_boundary_time_increase`. Default 32; at least 0.
    pub path_outer_boundary: Option<f32>,
    /// `random_stop_interval`: defines a 1-in-N chance for the mob to stop its current attack, where N equals `random_stop_interval`. Default 0; at least 0.
    pub random_stop_interval: Option<i32>,
    /// `require_complete_path`: specifies whether a full navigation path from the mob to the target is required. Default `false`.
    pub require_complete_path: Option<bool>,
    /// `speed_multiplier`: multiplier applied to the mob's movement speed when moving toward its target. Default 1; at least 0.
    pub speed_multiplier: Option<f32>,
    /// `track_target`: allows the mob to track its target even if it lacks a hard-coded sensing component. Default `false`.
    pub track_target: Option<bool>,
    /// `x_max_rotation`: maximum rotation, in degrees, on the X-axis while the mob is trying to look at its target. Default 3e+01; at least 0.
    pub x_max_rotation: Option<f32>,
    /// `y_max_head_rotation`: maximum rotation, in degrees, on the Y-axis while the mob is trying to look at its target. Default 3e+01; at least 0.
    pub y_max_head_rotation: Option<f32>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl MeleeBoxAttack {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `attack_once`, or its default `false`.
    pub fn attack_once(&self) -> bool {
        self.attack_once.is_some_and(|v| v)
    }

    /// `attack_types`, or its default `""`.
    pub fn attack_types(&self) -> &str {
        self.attack_types.as_ref().map_or("", |v| v.as_str())
    }

    /// `can_spread_on_fire`, or its default `false`.
    pub fn can_spread_on_fire(&self) -> bool {
        self.can_spread_on_fire.is_some_and(|v| v)
    }

    /// `cooldown_time`, or its default 1.
    pub fn cooldown_time(&self) -> f32 {
        self.cooldown_time.unwrap_or(1.0)
    }

    /// `horizontal_reach`, or its default 0.8.
    pub fn horizontal_reach(&self) -> f32 {
        self.horizontal_reach.unwrap_or(0.8)
    }

    /// `inner_boundary_time_increase`, or its default 0.25.
    pub fn inner_boundary_time_increase(&self) -> f32 {
        self.inner_boundary_time_increase.unwrap_or(0.25)
    }

    /// `max_path_time`, or its default 0.55.
    pub fn max_path_time(&self) -> f32 {
        self.max_path_time.unwrap_or(0.55)
    }

    /// `melee_fov`, or its default 9e+01.
    pub fn melee_fov(&self) -> f32 {
        self.melee_fov.unwrap_or(9.0e1)
    }

    /// `min_path_time`, or its default 0.2.
    pub fn min_path_time(&self) -> f32 {
        self.min_path_time.unwrap_or(0.2)
    }

    /// `outer_boundary_time_increase`, or its default 0.5.
    pub fn outer_boundary_time_increase(&self) -> f32 {
        self.outer_boundary_time_increase.unwrap_or(0.5)
    }

    /// `path_fail_time_increase`, or its default 0.75.
    pub fn path_fail_time_increase(&self) -> f32 {
        self.path_fail_time_increase.unwrap_or(0.75)
    }

    /// `path_inner_boundary`, or its default 16.
    pub fn path_inner_boundary(&self) -> f32 {
        self.path_inner_boundary.unwrap_or(16.0)
    }

    /// `path_outer_boundary`, or its default 32.
    pub fn path_outer_boundary(&self) -> f32 {
        self.path_outer_boundary.unwrap_or(32.0)
    }

    /// `random_stop_interval`, or its default 0.
    pub fn random_stop_interval(&self) -> i32 {
        self.random_stop_interval.unwrap_or(0)
    }

    /// `require_complete_path`, or its default `false`.
    pub fn require_complete_path(&self) -> bool {
        self.require_complete_path.is_some_and(|v| v)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.unwrap_or(1.0)
    }

    /// `track_target`, or its default `false`.
    pub fn track_target(&self) -> bool {
        self.track_target.is_some_and(|v| v)
    }

    /// `x_max_rotation`, or its default 3e+01.
    pub fn x_max_rotation(&self) -> f32 {
        self.x_max_rotation.unwrap_or(3.0e1)
    }

    /// `y_max_head_rotation`, or its default 3e+01.
    pub fn y_max_head_rotation(&self) -> f32 {
        self.y_max_head_rotation.unwrap_or(3.0e1)
    }
}

impl Goal for MeleeBoxAttack {
    const TYPED: FormatVersion = known::V1_26_0;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Ocelotattack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `cooldown_time`: time (in seconds) between attacks. Default 1; greater than 0.
    #[facet(schema::fallback = 1.0, schema::above = 0.0)]
    pub cooldown_time: Option<f32>,
    /// `max_distance`: max distance from the target, this entity will use this attack behavior. Default 15; greater than 0.
    #[facet(schema::fallback = 15.0, schema::above = 0.0)]
    pub max_distance: Option<f32>,
    /// `max_sneak_range`: max distance from the target, this entity starts sneaking. Default 15; greater than 0.
    #[facet(schema::fallback = 15.0, schema::above = 0.0)]
    pub max_sneak_range: Option<f32>,
    /// `max_sprint_range`: max distance from the target, this entity starts sprinting (sprinting takes priority over sneaking). Default 4; greater than 0.
    #[facet(schema::fallback = 4.0, schema::above = 0.0)]
    pub max_sprint_range: Option<f32>,
    /// `reach_multiplier`: used with the base size of the entity to determine minimum target-distance before trying to deal attack damage. Default 2; greater than 0.
    #[facet(schema::fallback = 2.0, schema::above = 0.0)]
    pub reach_multiplier: Option<f32>,
    /// `sneak_speed_multiplier`: modifies the attacking entity's movement speed while sneaking. Default 0.6; greater than 0.
    #[facet(schema::fallback = 0.6, schema::above = 0.0)]
    pub sneak_speed_multiplier: Option<f32>,
    /// `sprint_speed_multiplier`: modifies the attacking entity's movement speed while sprinting. Default 1.33; greater than 0.
    #[facet(schema::fallback = 1.33, schema::above = 0.0)]
    pub sprint_speed_multiplier: Option<f32>,
    /// `walk_speed_multiplier`: modifies the attacking entity's movement speed when not sneaking or sprinting, but still within attack range. Default 0.8; greater than 0.
    #[facet(schema::fallback = 0.8, schema::above = 0.0)]
    pub walk_speed_multiplier: Option<f32>,
    /// `x_max_rotation`: maximum rotation (in degrees), on the X-axis, this entity can rotate while trying to look at the target. Default 3e+01; greater than 0.
    #[facet(schema::fallback = 3.0e1, schema::above = 0.0)]
    pub x_max_rotation: Option<f32>,
    /// `y_max_head_rotation`: maximum rotation (in degrees), on the Y-axis, this entity's head can rotate while trying to look at the target. Default 3e+01; greater than 0.
    #[facet(schema::fallback = 3.0e1, schema::above = 0.0)]
    pub y_max_head_rotation: Option<f32>,
}

impl Goal for Ocelotattack {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.ram_attack`: allows this entity to damage a target by using a running attack.
// `behavior.ram_attack`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.ram_attack")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct RamAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `baby_knockback_modifier`: the modifier to knockback that babies have. Default 0.33333334.
    pub baby_knockback_modifier: Option<f32>,
    /// `cooldown_range`: minimum and maximum cooldown time-range (positive, in seconds) between each attempted ram attack. Default 1e+01 to 2e+01.
    pub cooldown_range: Option<FloatRange>,
    /// `knockback_force`: the force of the knockback of the ram attack. Default 5.
    pub knockback_force: Option<f32>,
    /// `knockback_height`: the height of the knockback of the ram attack. Default 0.1.
    pub knockback_height: Option<f32>,
    /// `min_ram_distance`: the minimum distance at which the mob can start a ram attack. Default 4.
    pub min_ram_distance: Option<i32>,
    /// `on_start`: the event to trigger when attacking.
    pub on_start: Option<Vec<EventTrigger>>,
    /// `pre_ram_sound`: the sound to play when an entity is about to perform a ram attack. Default `undefined`.
    pub pre_ram_sound: Option<SoundEvent>,
    /// `ram_distance`: the distance at which the mob start to run with ram speed. Default 7.
    pub ram_distance: Option<i32>,
    /// `ram_impact_sound`: the sound to play when an entity is impacting on a ram attack. Default `undefined`.
    pub ram_impact_sound: Option<SoundEvent>,
    /// `ram_speed`: sets the entity's speed when charging toward the target. Default 2.
    pub ram_speed: Option<f32>,
    /// `run_speed`: sets the entity's speed when running toward the target. Default 1.
    pub run_speed: Option<f32>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl RamAttack {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `baby_knockback_modifier`, or its default 0.33333334.
    pub fn baby_knockback_modifier(&self) -> f32 {
        self.baby_knockback_modifier.unwrap_or(0.333_333_34)
    }

    /// `cooldown_range`, or its default 1e+01 to 2e+01.
    pub fn cooldown_range(&self) -> (f32, f32) {
        self.cooldown_range
            .map_or((1.0e1, 2.0e1), |range| range.resolve(1.0e1, 2.0e1))
    }

    /// `knockback_force`, or its default 5.
    pub fn knockback_force(&self) -> f32 {
        self.knockback_force.unwrap_or(5.0)
    }

    /// `knockback_height`, or its default 0.1.
    pub fn knockback_height(&self) -> f32 {
        self.knockback_height.unwrap_or(0.1)
    }

    /// `min_ram_distance`, or its default 4.
    pub fn min_ram_distance(&self) -> i32 {
        self.min_ram_distance.unwrap_or(4)
    }

    /// `pre_ram_sound`, or its default `undefined`.
    pub fn pre_ram_sound(&self) -> &str {
        self.pre_ram_sound
            .as_ref()
            .map_or("undefined", SoundEvent::name)
    }

    /// `ram_distance`, or its default 7.
    pub fn ram_distance(&self) -> i32 {
        self.ram_distance.unwrap_or(7)
    }

    /// `ram_impact_sound`, or its default `undefined`.
    pub fn ram_impact_sound(&self) -> &str {
        self.ram_impact_sound
            .as_ref()
            .map_or("undefined", SoundEvent::name)
    }

    /// `ram_speed`, or its default 2.
    pub fn ram_speed(&self) -> f32 {
        self.ram_speed.unwrap_or(2.0)
    }

    /// `run_speed`, or its default 1.
    pub fn run_speed(&self) -> f32 {
        self.run_speed.unwrap_or(1.0)
    }
}

impl Goal for RamAttack {
    const TYPED: FormatVersion = known::V1_26_10;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct RangedAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `attack_interval`: reload-time range (in seconds), when not using a charged shot. Reload-time range scales with target-distance. Default -1 to -1.
    pub attack_interval: Option<FloatRange>,
    /// `attack_range`: range of distances from the target within which the entity can perform a ranged attack. Default 0 to 0.
    pub attack_range: Option<FloatRange>,
    /// `burst_interval`: time (in seconds) between each individual shot when firing a burst of shots from a charged up attack. Default 0; greater than 0.
    #[facet(schema::fallback = 0.0)]
    pub burst_interval: Option<f32>,
    /// `burst_shots`: number of shots fired every time the attacking entity uses a charged up attack. Default 1.
    #[facet(schema::fallback = 1)]
    pub burst_shots: Option<i32>,
    /// `charge_charged_trigger`: time (in seconds, then add `charge_shoot_trigger`), before a charged up attack is done charging. Charge-time decays while target is not in sight. Default 0; greater than 0.
    #[facet(schema::fallback = 0.0)]
    pub charge_charged_trigger: Option<f32>,
    /// `charge_shoot_trigger`: amount of time (in seconds, then doubled) a charged shot must be charging before reloading burst shots. Charge-time decays while target is not in sight. Default 0; greater than 0.
    #[facet(schema::fallback = 0.0)]
    pub charge_shoot_trigger: Option<f32>,
    /// `in_range_movement_mode`: controls how the entity moves while its target is within the configured attack range. `hold_position` makes the entity stop and hold its position while facing and attacking the target. `follow_target` makes the entity continue moving toward the target while attacking, stopping just outside the minimum attack range. Default `hold_position`.
    #[facet(schema::fallback = InRangeMovementMode::HoldPosition)]
    pub in_range_movement_mode: Option<InRangeMovementMode>,
    /// `ranged_fov`: field of view (in degrees) when using sensing to detect a target for attack. Default 9e+01; at least 0.
    #[facet(schema::fallback = 9.0e1)]
    pub ranged_fov: Option<f32>,
    /// `set_persistent`: allows the actor to be set to persist upon targeting a player. Default `false`.
    #[facet(schema::fallback = false)]
    pub set_persistent: Option<bool>,
    /// `speed_multiplier`: during attack behavior, this multiplier modifies the entity's speed when moving toward the target. Default 1; greater than 0.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
    /// `swing`: if a swing animation (using `variable.attack_time`) exists, this causes the actor to swing their arm(s) upon firing the ranged attack. Default `false`.
    #[facet(schema::fallback = false)]
    pub swing: Option<bool>,
    /// `target_in_sight_time`: minimum amount of time (in seconds) the attacking entity needs to see the target before moving toward it. Default 1; greater than 0.
    #[facet(schema::fallback = 1.0)]
    pub target_in_sight_time: Option<f32>,
    /// `x_max_rotation`: maximum rotation (in degrees), on the X-axis, this entity can rotate while trying to look at the target. Default 3e+01; at least 0.
    #[facet(schema::fallback = 3.0e1)]
    pub x_max_rotation: Option<f32>,
    /// `y_max_head_rotation`: maximum rotation (in degrees), on the Y-axis, this entity can rotate its head while trying to look at the target. Default 3e+01; at least 0.
    #[facet(schema::fallback = 3.0e1)]
    pub y_max_head_rotation: Option<f32>,
}

impl RangedAttack {
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
}

impl Goal for RangedAttack {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    const REJECTED: &'static [&'static str] = &["attack_interval_min", "attack_interval_max"];

    const GATED: &'static [(&'static str, FormatVersion)] = &[
        ("attack_range", V1_26_50),
        ("in_range_movement_mode", V1_26_50),
    ];

    fn check(&mut self, c: &mut GoalCheck<'_>) {
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

    fn upgrade(&mut self, from: FormatVersion, d: &mut Diagnostics) {
        if from < V1_26_50 {
            convert::ranged_radius_typed(self, d.unknown_mut());
        }
    }

    fn lenient_shape(json: &mut VObject, d: &mut Diagnostics) {
        convert::ranged_interval(json, d);
        convert::ranged_radius(json, d);
    }
}

/// `minecraft:behavior.slime_attack`: causes the entity to grow tired every once in a while, while attacking.
// `behavior.slime_attack`: typed from 1.21.110.
#[doc(alias = "minecraft:behavior.slime_attack")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SlimeAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `grow_tired_cooldown_time`: default 15.
    #[facet(schema::fallback = 15.0)]
    pub grow_tired_cooldown_time: Option<f32>,
    /// `set_persistent`: allows the actor to be set to persist upon targeting a player. Default `false`.
    pub set_persistent: Option<bool>,
    /// `speed_multiplier`: during attack behavior, this multiplier modifies the entity's speed when moving toward the target. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
    /// `x_max_rotation`: maximum rotation (in degrees), on the X-axis, this entity can rotate while trying to look at the target. Default 1e+01.
    #[facet(schema::fallback = 1.0e1)]
    pub x_max_rotation: Option<f32>,
    /// `y_max_rotation`: maximum rotation (in degrees), on the Y-axis, this entity can rotate while trying to look at the target. Default 1e+01.
    #[facet(schema::fallback = 1.0e1)]
    pub y_max_rotation: Option<f32>,
}

impl SlimeAttack {
    /// `set_persistent`, or its default `false`.
    pub fn set_persistent(&self) -> bool {
        self.set_persistent.is_some_and(|v| v)
    }
}

impl Goal for SlimeAttack {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.sonic_boom`: allows this entity to perform a 'sonic boom' ranged attack.
// `behavior.sonic_boom`: typed from 1.21.110.
#[doc(alias = "minecraft:behavior.sonic_boom")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SonicBoom {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `attack_cooldown`: cooldown in seconds required after using this attack until the entity can use sonic boom again. Default 5.
    #[facet(schema::fallback = 5.0)]
    pub attack_cooldown: Option<f32>,
    /// `attack_damage`: attack damage of the sonic boom. Default 3e+01.
    #[facet(schema::fallback = 3.0e1)]
    pub attack_damage: Option<f32>,
    /// `attack_range_horizontal`: horizontal range (in blocks) at which the sonic boom can damage the target. Default 15.
    #[facet(schema::fallback = 15.0)]
    pub attack_range_horizontal: Option<f32>,
    /// `attack_range_vertical`: vertical range (in blocks) at which the sonic boom can damage the target. Default 2e+01.
    #[facet(schema::fallback = 2.0e1)]
    pub attack_range_vertical: Option<f32>,
    /// `attack_sound`: sound event for the attack. Default `""`.
    pub attack_sound: Option<SoundEvent>,
    /// `charge_sound`: sound event for the charge up. Default `""`.
    pub charge_sound: Option<SoundEvent>,
    /// `duration`: goal duration in seconds. Default 0; at least 0.
    #[facet(schema::fallback = 0.0)]
    pub duration: Option<f32>,
    /// `duration_until_attack_sound`: duration in seconds until the attack sound is played. Default 1.7.
    #[facet(schema::fallback = 1.7)]
    pub duration_until_attack_sound: Option<f32>,
    /// `knockback_height_cap`: height cap of the attack knockback's vertical delta. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub knockback_height_cap: Option<f32>,
    /// `knockback_horizontal_strength`: horizontal strength of the attack's knockback applied to the attack target. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub knockback_horizontal_strength: Option<f32>,
    /// `knockback_vertical_strength`: vertical strength of the attack's knockback applied to the attack target. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub knockback_vertical_strength: Option<f32>,
    /// `speed_multiplier`: this multiplier modifies the attacking entity's speed when moving toward the target. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
}

impl SonicBoom {
    /// `attack_sound`, or its default `""`.
    pub fn attack_sound(&self) -> &str {
        self.attack_sound.as_ref().map_or("", SoundEvent::name)
    }

    /// `charge_sound`, or its default `""`.
    pub fn charge_sound(&self) -> &str {
        self.charge_sound.as_ref().map_or("", SoundEvent::name)
    }
}

impl Goal for SonicBoom {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        // Bounds by hand, not schema::min/max: a bound here follows another check, and the
        // derived check runs first, which would reorder the findings.
        c.base(&mut self.priority);
        c.sound("attack_sound", self.attack_sound.as_ref());
        c.sound("charge_sound", self.charge_sound.as_ref());
        c.float("duration", &mut self.duration, Bound::Min(0.0));
    }
}

/// `minecraft:behavior.stalk_and_pounce_on_target`: allows a mob to stalk a target, then once within range pounce onto a target, on success the target will be attacked dealing damage defined by the attack component. On failure, the mob will risk getting stuck.
// `behavior.stalk_and_pounce_on_target`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.stalk_and_pounce_on_target")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct StalkAndPounceOnTarget {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `interest_time`: the amount of time the mob will be interested before pouncing. This happens when the mob is within range of pouncing. Default 2.
    #[facet(schema::fallback = 2.0)]
    pub interest_time: Option<f32>,
    /// `leap_dist`: the distance in blocks the mob jumps in the direction of its target. Default 0.8.
    #[facet(schema::fallback = 0.8)]
    pub leap_dist: Option<f32>,
    /// `leap_height`: the height in blocks the mob jumps when leaping at its target. Default 0.9.
    #[facet(schema::fallback = 0.9)]
    pub leap_height: Option<f32>,
    /// `max_stalk_dist`: the maximum distance away a target can be before the mob gives up on stalking. Default 1e+01.
    #[facet(schema::fallback = 1.0e1)]
    pub max_stalk_dist: Option<f32>,
    /// `pounce_max_dist`: the maximum distance away from the target in blocks to begin pouncing at the target. Default 5.
    #[facet(schema::fallback = 5.0)]
    pub pounce_max_dist: Option<f32>,
    /// `set_persistent`: allows the actor to be set to persist upon targeting a player. Default `false`.
    #[facet(schema::fallback = false)]
    pub set_persistent: Option<bool>,
    /// `stalk_speed`: the movement speed in which you stalk your target. Default 1.2.
    #[facet(schema::fallback = 1.2)]
    pub stalk_speed: Option<f32>,
    /// `strike_dist`: the max distance away from the target when landing from the pounce that will still result in damaging the target. Default 2.
    #[facet(schema::fallback = 2.0)]
    pub strike_dist: Option<f32>,
    /// `stuck_blocks`: blocks that the mob can get stuck in after a pounce.
    pub stuck_blocks: Option<Filter>,
    /// `stuck_time`: the amount of time the mob will be stuck if they fail and land on a block they can be stuck on. Default 2.
    #[facet(schema::fallback = 2.0)]
    pub stuck_time: Option<f32>,
}

impl Goal for StalkAndPounceOnTarget {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.filter("stuck_blocks", self.stuck_blocks.as_ref());
    }
}

/// `minecraft:behavior.stomp_attack`: allows an entity to attack using stomp `AoE` damage behavior.
///
/// Deprecated members, reported and dropped: `melee_fov`.
// `behavior.stomp_attack`: typed from 1.26.0, redeclared 1.26.40.
#[doc(alias = "minecraft:behavior.stomp_attack")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct StompAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `attack_once`: allows the mob to perform this melee attack behavior only once during its lifetime. Default `false`.
    pub attack_once: Option<bool>,
    /// `attack_types`: defines the entity types this entity will attack. Default `""`.
    pub attack_types: Option<String>,
    /// `can_spread_on_fire`: allows the mob, if on fire and empty handed, to ignite its target upon a successful attack. Default `false`.
    pub can_spread_on_fire: Option<bool>,
    /// `cooldown_time`: cooldown time, in seconds, between consecutive attacks. Default 1; at least 0.
    pub cooldown_time: Option<f32>,
    /// `inner_boundary_time_increase`: time, in seconds, added to the attack path recalculation interval when the target is beyond `path_inner_boundary`. Default 0.25; at least 0.
    pub inner_boundary_time_increase: Option<f32>,
    /// `max_path_time`: maximum base time, in seconds, before recalculating a new attack path to the target (before increases are applied). Default 0.55; at least 0.
    pub max_path_time: Option<f32>,
    /// `min_path_time`: minimum base time, in seconds, before recalculating a new attack path to the target (before increases are applied). Default 0.2; at least 0.
    pub min_path_time: Option<f32>,
    /// `no_damage_range_multiplier`: multiplied with the final `AoE` damage range to determine a no damage range. The stomp attack will go on cooldown if target is in this no damage range. Default 2; greater than 0.
    pub no_damage_range_multiplier: Option<f32>,
    /// `on_attack`: defines the event to trigger when this entity successfully attacks.
    pub on_attack: Option<EventTrigger>,
    /// `on_kill`: defines the event to trigger when this entity kills the target.
    pub on_kill: Option<EventTrigger>,
    /// `outer_boundary_time_increase`: time, in seconds, added to the attack path recalculation interval when the target is beyond `path_outer_boundary`. Default 0.5; at least 0.
    pub outer_boundary_time_increase: Option<f32>,
    /// `path_fail_time_increase`: time, in seconds, added to the attack path recalculation interval when the mob cannot move along the current path. Default 0.75; at least 0.
    pub path_fail_time_increase: Option<f32>,
    /// `path_inner_boundary`: distance at which to increase attack path recalculation by `inner_boundary_time_increase`. Default 16; at least 0.
    pub path_inner_boundary: Option<f32>,
    /// `path_outer_boundary`: distance at which to increase attack path recalculation by `outer_boundary_time_increase`. Default 32; at least 0.
    pub path_outer_boundary: Option<f32>,
    /// `random_stop_interval`: defines a 1-in-N chance for the mob to stop its current attack, where N equals `random_stop_interval`. Default 0; at least 0.
    pub random_stop_interval: Option<i32>,
    /// `reach_multiplier`: used with the base size of the entity to determine minimum target-distance before trying to deal attack damage. Default 2; at least 0.
    pub reach_multiplier: Option<f32>,
    /// `require_complete_path`: specifies whether a full navigation path from the mob to the target is required. Default `false`.
    pub require_complete_path: Option<bool>,
    /// `speed_multiplier`: multiplier applied to the mob's movement speed when moving toward its target. Default 1; at least 0.
    pub speed_multiplier: Option<f32>,
    /// `stomp_range_multiplier`: multiplied with the base size of the entity to determine stomp `AoE` damage range. Default 2; greater than 0.
    pub stomp_range_multiplier: Option<f32>,
    /// `track_target`: allows the mob to track its target even if it lacks a hard-coded sensing component. Default `false`.
    pub track_target: Option<bool>,
    /// `x_max_rotation`: maximum rotation, in degrees, on the X-axis while the mob is trying to look at its target. Default 3e+01; at least 0.
    pub x_max_rotation: Option<f32>,
    /// `y_max_head_rotation`: maximum rotation, in degrees, on the Y-axis while the mob is trying to look at its target. Default 3e+01; at least 0.
    pub y_max_head_rotation: Option<f32>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl StompAttack {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `attack_once`, or its default `false`.
    pub fn attack_once(&self) -> bool {
        self.attack_once.is_some_and(|v| v)
    }

    /// `attack_types`, or its default `""`.
    pub fn attack_types(&self) -> &str {
        self.attack_types.as_ref().map_or("", |v| v.as_str())
    }

    /// `can_spread_on_fire`, or its default `false`.
    pub fn can_spread_on_fire(&self) -> bool {
        self.can_spread_on_fire.is_some_and(|v| v)
    }

    /// `cooldown_time`, or its default 1.
    pub fn cooldown_time(&self) -> f32 {
        self.cooldown_time.unwrap_or(1.0)
    }

    /// `inner_boundary_time_increase`, or its default 0.25.
    pub fn inner_boundary_time_increase(&self) -> f32 {
        self.inner_boundary_time_increase.unwrap_or(0.25)
    }

    /// `max_path_time`, or its default 0.55.
    pub fn max_path_time(&self) -> f32 {
        self.max_path_time.unwrap_or(0.55)
    }

    /// `min_path_time`, or its default 0.2.
    pub fn min_path_time(&self) -> f32 {
        self.min_path_time.unwrap_or(0.2)
    }

    /// `no_damage_range_multiplier`, or its default 2.
    pub fn no_damage_range_multiplier(&self) -> f32 {
        self.no_damage_range_multiplier.unwrap_or(2.0)
    }

    /// `outer_boundary_time_increase`, or its default 0.5.
    pub fn outer_boundary_time_increase(&self) -> f32 {
        self.outer_boundary_time_increase.unwrap_or(0.5)
    }

    /// `path_fail_time_increase`, or its default 0.75.
    pub fn path_fail_time_increase(&self) -> f32 {
        self.path_fail_time_increase.unwrap_or(0.75)
    }

    /// `path_inner_boundary`, or its default 16.
    pub fn path_inner_boundary(&self) -> f32 {
        self.path_inner_boundary.unwrap_or(16.0)
    }

    /// `path_outer_boundary`, or its default 32.
    pub fn path_outer_boundary(&self) -> f32 {
        self.path_outer_boundary.unwrap_or(32.0)
    }

    /// `random_stop_interval`, or its default 0.
    pub fn random_stop_interval(&self) -> i32 {
        self.random_stop_interval.unwrap_or(0)
    }

    /// `reach_multiplier`, or its default 2.
    pub fn reach_multiplier(&self) -> f32 {
        self.reach_multiplier.unwrap_or(2.0)
    }

    /// `require_complete_path`, or its default `false`.
    pub fn require_complete_path(&self) -> bool {
        self.require_complete_path.is_some_and(|v| v)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.unwrap_or(1.0)
    }

    /// `stomp_range_multiplier`, or its default 2.
    pub fn stomp_range_multiplier(&self) -> f32 {
        self.stomp_range_multiplier.unwrap_or(2.0)
    }

    /// `track_target`, or its default `false`.
    pub fn track_target(&self) -> bool {
        self.track_target.is_some_and(|v| v)
    }

    /// `x_max_rotation`, or its default 3e+01.
    pub fn x_max_rotation(&self) -> f32 {
        self.x_max_rotation.unwrap_or(3.0e1)
    }

    /// `y_max_head_rotation`, or its default 3e+01.
    pub fn y_max_head_rotation(&self) -> f32 {
        self.y_max_head_rotation.unwrap_or(3.0e1)
    }
}

impl Goal for StompAttack {
    const TYPED: FormatVersion = known::V1_26_0;

    const DEPRECATED: &'static [Deprecated] =
        &[("melee_fov", deprecated_text!("melee_fov"), known::V1_26_0)];

    fn check(&mut self, c: &mut GoalCheck<'_>) {
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SummonEntity {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `summon_choices`: list of spells for the mob to use to summon entities. Each spell has the following parameters.
    pub summon_choices: Option<Vec<SummonChoice>>,
}

impl Goal for SummonEntity {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
        c.each(
            "summon_choices",
            &mut self.summon_choices,
            SummonChoice::check,
        );
    }
}

/// `minecraft:behavior.swell`: allows the creeper to swell up when a player is nearby. It can only be used by Creepers.
// `behavior.swell`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.swell")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Swell {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `start_distance`: this mob starts swelling when a target is at least this many blocks away. Default 1e+01.
    #[facet(schema::fallback = 1.0e1)]
    pub start_distance: Option<f32>,
    /// `stop_distance`: this mob stops swelling when a target has moved away at least this many blocks. Default 2.
    #[facet(schema::fallback = 2.0)]
    pub stop_distance: Option<f32>,
}

impl Goal for Swell {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.swoop_attack`: allows an entity to attack using swoop attack behavior; Ideal for use with flying mobs. The behavior ends if the entity has a horizontal collision or gets hit.
// `behavior.swoop_attack`: typed from 1.21.110.
#[doc(alias = "minecraft:behavior.swoop_attack")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SwoopAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `damage_reach`: added to the base size of the entity, to determine the target's maximum allowable distance, when trying to deal attack damage. Default 0.2.
    #[facet(schema::fallback = 0.2)]
    pub damage_reach: Option<f32>,
    /// `delay_range`: minimum and maximum cooldown time-range (in seconds) between each attempted swoop attack. Default 1e+01 to 2e+01.
    pub delay_range: Option<FloatRange>,
    /// `speed_multiplier`: during swoop attack behavior, this determines the multiplier the entity's speed is modified by when moving toward the target. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
}

impl SwoopAttack {
    /// `delay_range`, or its default 1e+01 to 2e+01.
    pub fn delay_range(&self) -> (f32, f32) {
        self.delay_range
            .map_or((1.0e1, 2.0e1), |range| range.resolve(1.0e1, 2.0e1))
    }
}

impl Goal for SwoopAttack {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.range("delay_range", &mut self.delay_range);
    }
}

/// `minecraft:behavior.use_kinetic_weapon`: enables a mob to use kinetic weaponry by intermittently charging at its target and repositioning afterward.
// `behavior.use_kinetic_weapon`: typed from 1.21.120, redeclared 1.26.40.
#[doc(alias = "minecraft:behavior.use_kinetic_weapon")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct UseKineticWeapon {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `approach_distance`: the distance to the target within which the mob begins using its kinetic weapon. Default 8; at least 0.
    pub approach_distance: Option<f32>,
    /// `attack_once`: allows the mob to perform this melee attack behavior only once during its lifetime. Default `false`.
    pub attack_once: Option<bool>,
    /// `attack_types`: defines the entity types this entity will attack. Default `""`.
    pub attack_types: Option<String>,
    /// `can_spread_on_fire`: allows the mob, if on fire and empty handed, to ignite its target upon a successful attack. Default `false`.
    pub can_spread_on_fire: Option<bool>,
    /// `cooldown_distance`: the distance the mob retreats to after all of the item's `minecraft:kinetic_weapon` component's `max_duration` values have elapsed. After reaching this position and once `cooldown` has elapsed, the mob will approach again. This value is added to the reach midpoint. Default 8 to 8.
    pub cooldown_distance: Option<FloatRange>,
    /// `cooldown_speed_multiplier`: multiplier applied to the mob's movement speed while on cooldown. Default 1; at least 0.
    pub cooldown_speed_multiplier: Option<f32>,
    /// `cooldown_time`: cooldown time, in seconds, between consecutive attacks. Default 1; at least 0.
    pub cooldown_time: Option<f32>,
    /// `hijack_mount_navigation`: allows the mob to override its mount's navigation behavior with the logic defined by this goal. Requires the mount to be running the `minecraft:behavior.mount_pathing` goal, which default behavior will be ignored. Default `false`.
    pub hijack_mount_navigation: Option<bool>,
    /// `inner_boundary_time_increase`: time, in seconds, added to the attack path recalculation interval when the target is beyond `path_inner_boundary`. Default 0.25; at least 0.
    pub inner_boundary_time_increase: Option<f32>,
    /// `max_path_time`: maximum base time, in seconds, before recalculating a new attack path to the target (before increases are applied). Default 0.55; at least 0.
    pub max_path_time: Option<f32>,
    /// `melee_fov`: field of view, in degrees, used by the hard-coded sensing component to detect a valid attack target. Default 9e+01; at least 0.
    pub melee_fov: Option<f32>,
    /// `min_path_time`: minimum base time, in seconds, before recalculating a new attack path to the target (before increases are applied). Default 0.2; at least 0.
    pub min_path_time: Option<f32>,
    /// `on_attack`: defines the event to trigger when this entity successfully attacks.
    pub on_attack: Option<EventTrigger>,
    /// `on_kill`: defines the event to trigger when this entity kills the target.
    pub on_kill: Option<EventTrigger>,
    /// `outer_boundary_time_increase`: time, in seconds, added to the attack path recalculation interval when the target is beyond `path_outer_boundary`. Default 0.5; at least 0.
    pub outer_boundary_time_increase: Option<f32>,
    /// `path_fail_time_increase`: time, in seconds, added to the attack path recalculation interval when the mob cannot move along the current path. Default 0.75; at least 0.
    pub path_fail_time_increase: Option<f32>,
    /// `path_inner_boundary`: distance at which to increase attack path recalculation by `inner_boundary_time_increase`. Default 16; at least 0.
    pub path_inner_boundary: Option<f32>,
    /// `path_outer_boundary`: distance at which to increase attack path recalculation by `outer_boundary_time_increase`. Default 32; at least 0.
    pub path_outer_boundary: Option<f32>,
    /// `random_stop_interval`: defines a 1-in-N chance for the mob to stop its current attack, where N equals `random_stop_interval`. Default 0; at least 0.
    pub random_stop_interval: Option<i32>,
    /// `reposition_distance`: the distance the mob retreats to once the target is closer than the midpoint of the item's `minecraft:kinetic_weapon` component's minimum and maximum `reach`. After reaching this position, the mob will charge again. This value is added to the reach midpoint. Default 2 to 2.
    pub reposition_distance: Option<FloatRange>,
    /// `reposition_speed_multiplier`: multiplier applied to the mob's movement speed while repositioning. Default 1; at least 0.
    pub reposition_speed_multiplier: Option<f32>,
    /// `require_complete_path`: specifies whether a full navigation path from the mob to the target is required. Default `false`.
    pub require_complete_path: Option<bool>,
    /// `speed_multiplier`: multiplier applied to the mob's movement speed when moving toward its target. Default 1; at least 0.
    pub speed_multiplier: Option<f32>,
    /// `track_target`: allows the mob to track its target even if it lacks a hard-coded sensing component. Default `false`.
    pub track_target: Option<bool>,
    /// `weapon_min_speed_multiplier`: multiplier applied to each `min_speed` and `min_relative_speed` condition in the item's `minecraft:kinetic_weapon` component. Default 1; at least 0.
    pub weapon_min_speed_multiplier: Option<f32>,
    /// `weapon_reach_multiplier`: multiplier applied to the item's `minecraft:kinetic_weapon` component's `reach`. Default 1; at least 0.
    pub weapon_reach_multiplier: Option<f32>,
    /// `x_max_rotation`: maximum rotation, in degrees, on the X-axis while the mob is trying to look at its target. Default 3e+01; at least 0.
    pub x_max_rotation: Option<f32>,
    /// `y_max_head_rotation`: maximum rotation, in degrees, on the Y-axis while the mob is trying to look at its target. Default 3e+01; at least 0.
    pub y_max_head_rotation: Option<f32>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl UseKineticWeapon {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `approach_distance`, or its default 8.
    pub fn approach_distance(&self) -> f32 {
        self.approach_distance.unwrap_or(8.0)
    }

    /// `attack_once`, or its default `false`.
    pub fn attack_once(&self) -> bool {
        self.attack_once.is_some_and(|v| v)
    }

    /// `attack_types`, or its default `""`.
    pub fn attack_types(&self) -> &str {
        self.attack_types.as_ref().map_or("", |v| v.as_str())
    }

    /// `can_spread_on_fire`, or its default `false`.
    pub fn can_spread_on_fire(&self) -> bool {
        self.can_spread_on_fire.is_some_and(|v| v)
    }

    /// `cooldown_distance`, or its default 8 to 8.
    pub fn cooldown_distance(&self) -> (f32, f32) {
        self.cooldown_distance
            .map_or((8.0, 8.0), |range| range.resolve(8.0, 8.0))
    }

    /// `cooldown_speed_multiplier`, or its default 1.
    pub fn cooldown_speed_multiplier(&self) -> f32 {
        self.cooldown_speed_multiplier.unwrap_or(1.0)
    }

    /// `cooldown_time`, or its default 1.
    pub fn cooldown_time(&self) -> f32 {
        self.cooldown_time.unwrap_or(1.0)
    }

    /// `hijack_mount_navigation`, or its default `false`.
    pub fn hijack_mount_navigation(&self) -> bool {
        self.hijack_mount_navigation.is_some_and(|v| v)
    }

    /// `inner_boundary_time_increase`, or its default 0.25.
    pub fn inner_boundary_time_increase(&self) -> f32 {
        self.inner_boundary_time_increase.unwrap_or(0.25)
    }

    /// `max_path_time`, or its default 0.55.
    pub fn max_path_time(&self) -> f32 {
        self.max_path_time.unwrap_or(0.55)
    }

    /// `melee_fov`, or its default 9e+01.
    pub fn melee_fov(&self) -> f32 {
        self.melee_fov.unwrap_or(9.0e1)
    }

    /// `min_path_time`, or its default 0.2.
    pub fn min_path_time(&self) -> f32 {
        self.min_path_time.unwrap_or(0.2)
    }

    /// `outer_boundary_time_increase`, or its default 0.5.
    pub fn outer_boundary_time_increase(&self) -> f32 {
        self.outer_boundary_time_increase.unwrap_or(0.5)
    }

    /// `path_fail_time_increase`, or its default 0.75.
    pub fn path_fail_time_increase(&self) -> f32 {
        self.path_fail_time_increase.unwrap_or(0.75)
    }

    /// `path_inner_boundary`, or its default 16.
    pub fn path_inner_boundary(&self) -> f32 {
        self.path_inner_boundary.unwrap_or(16.0)
    }

    /// `path_outer_boundary`, or its default 32.
    pub fn path_outer_boundary(&self) -> f32 {
        self.path_outer_boundary.unwrap_or(32.0)
    }

    /// `random_stop_interval`, or its default 0.
    pub fn random_stop_interval(&self) -> i32 {
        self.random_stop_interval.unwrap_or(0)
    }

    /// `reposition_distance`, or its default 2 to 2.
    pub fn reposition_distance(&self) -> (f32, f32) {
        self.reposition_distance
            .map_or((2.0, 2.0), |range| range.resolve(2.0, 2.0))
    }

    /// `reposition_speed_multiplier`, or its default 1.
    pub fn reposition_speed_multiplier(&self) -> f32 {
        self.reposition_speed_multiplier.unwrap_or(1.0)
    }

    /// `require_complete_path`, or its default `false`.
    pub fn require_complete_path(&self) -> bool {
        self.require_complete_path.is_some_and(|v| v)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.unwrap_or(1.0)
    }

    /// `track_target`, or its default `false`.
    pub fn track_target(&self) -> bool {
        self.track_target.is_some_and(|v| v)
    }

    /// `weapon_min_speed_multiplier`, or its default 1.
    pub fn weapon_min_speed_multiplier(&self) -> f32 {
        self.weapon_min_speed_multiplier.unwrap_or(1.0)
    }

    /// `weapon_reach_multiplier`, or its default 1.
    pub fn weapon_reach_multiplier(&self) -> f32 {
        self.weapon_reach_multiplier.unwrap_or(1.0)
    }

    /// `x_max_rotation`, or its default 3e+01.
    pub fn x_max_rotation(&self) -> f32 {
        self.x_max_rotation.unwrap_or(3.0e1)
    }

    /// `y_max_head_rotation`, or its default 3e+01.
    pub fn y_max_head_rotation(&self) -> f32 {
        self.y_max_head_rotation.unwrap_or(3.0e1)
    }
}

impl Goal for UseKineticWeapon {
    const TYPED: FormatVersion = known::V1_21_120;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SummonStage {
    /// `base_delay`: amount of time in seconds to wait before this step starts. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub base_delay: Option<f32>,
    /// `delay_per_summon`: amount of time in seconds before each entity is summoned in this step. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub delay_per_summon: Option<f32>,
    /// `entity_lifespan`: amount of time in seconds that the spawned entity will be alive for. A value of -1.0 means it will remain alive for as long as it can. Default -1.
    #[facet(schema::fallback = -1.0)]
    pub entity_lifespan: Option<f32>,
    /// `entity_type`: the entity type of the entities we will spawn in this step. Default `""`.
    pub entity_type: Option<String>,
    /// `num_entities_spawned`: number of entities that will be spawned in this step. Default 1.
    #[facet(schema::fallback = 1)]
    pub num_entities_spawned: Option<i32>,
    /// `shape`: the base shape of this step. Valid values are circle and line. Default `line`.
    #[facet(schema::fallback = SummonShape::Line)]
    pub shape: Option<SummonShape>,
    /// `size`: the base size of the entity. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub size: Option<f32>,
    /// `sound_event`: the sound event to play for this step. Default `undefined`.
    pub sound_event: Option<SoundEvent>,
    /// `summon_cap`: maximum number of summoned entities at any given time. Default 0.
    #[facet(schema::fallback = 0)]
    pub summon_cap: Option<i32>,
    /// `summon_cap_radius`: default 0.
    #[facet(schema::fallback = 0.0)]
    pub summon_cap_radius: Option<f32>,
    /// `summon_event`: event to invoke on each summoned entity on spawn. Default `""`.
    pub summon_event: Option<String>,
    /// `target`: the target of the spell. This is where the spell will start (line will start here, circle will be centered here). Default `self`.
    #[facet(schema::fallback = SummonTarget::Self_)]
    pub target: Option<SummonTarget>,
}

impl SummonStage {
    /// `entity_type`, or its default `""`.
    pub fn entity_type(&self) -> &str {
        self.entity_type.as_ref().map_or("", |v| v.as_str())
    }

    /// `sound_event`, or its default `undefined`.
    pub fn sound_event(&self) -> &str {
        self.sound_event
            .as_ref()
            .map_or("undefined", SoundEvent::name)
    }

    /// `summon_event`, or its default `""`.
    pub fn summon_event(&self) -> &str {
        self.summon_event.as_ref().map_or("", |v| v.as_str())
    }
}

impl SummonStage {
    /// Checks the entry's members.
    pub(crate) fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.sound("sound_event", self.sound_event.as_ref());
    }
}

/// One spell `summon_entity` may cast: when it is chosen and the stages it runs.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SummonChoice {
    /// `cast_duration`: time in seconds the spell casting will take. Default -1.
    #[facet(schema::fallback = -1.0)]
    pub cast_duration: Option<f32>,
    /// `cooldown_time`: time in seconds the mob has to wait before using the spell again. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub cooldown_time: Option<f32>,
    /// `do_casting`: if true, the mob will do the casting animations and render spell particles. Default `true`.
    #[facet(schema::fallback = true)]
    pub do_casting: Option<bool>,
    /// `filters`: filters allow data objects to specify test criteria which allows their use. Filters can be defined by a single object of type (Filter Test), an array of tests, collection groups, or a combination of these objects.
    pub filters: Option<Filter>,
    /// `max_activation_range`: upper bound of the activation distance in blocks for this spell, must not be negative. Default 32; at least 0.
    #[facet(schema::fallback = 32.0)]
    pub max_activation_range: Option<f32>,
    /// `min_activation_range`: lower bound of the activation distance in blocks for this spell, must not be negative. Default 1; at least 0.
    #[facet(schema::fallback = 1.0)]
    pub min_activation_range: Option<f32>,
    /// `particle_color`: the color of the particles for this spell.
    pub particle_color: Option<ParticleColor>,
    /// `sequence`: list of steps for the spell. Each step has the following parameters.
    pub sequence: Option<Vec<SummonStage>>,
    /// `start_sound_event`: the sound event to play when using this spell. Default `undefined`.
    pub start_sound_event: Option<SoundEvent>,
    /// `weight`: the weight of this spell. Controls how likely the mob is to choose this spell when casting one. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub weight: Option<f32>,
}

impl SummonChoice {
    /// `start_sound_event`, or its default `undefined`.
    pub fn start_sound_event(&self) -> &str {
        self.start_sound_event
            .as_ref()
            .map_or("undefined", SoundEvent::name)
    }
}

impl SummonChoice {
    /// Checks the entry's members.
    pub(crate) fn check(&mut self, c: &mut GoalCheck<'_>) {
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SendEvent {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `event_choices`: list of spell options to select from. One is randomly chosen each time the goal activates.
    pub event_choices: Option<Vec<SendEventChoice>>,
}

impl Goal for SendEvent {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
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
        d.push(Diagnostic::of(&core_codes::VALUE_REWRITTEN).at(at));
    }
}

/// A warning of the legacy reader at `at`, with its text: the game logs the legacy goal
/// parser's activation-range lines (a negative minimum, a negative maximum, the swap) at the
/// warning level.
fn at_warning(at: JsonPath, text: &'static str) -> Diagnostic {
    Diagnostic::of(core_codes::INVALID_VALUE_WORDED.with([text]))
        .severity(Severity::Warning)
        .at(at)
}

/// An `f32` as the JSON number that reads back to it.
fn f32_value(value: f32) -> Value {
    Value::from(f64::from(value))
}

/// One entry of `send_event.event_choices`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SendEventChoice {
    /// `cast_duration`: time in seconds for the entire event sending process. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub cast_duration: Option<f32>,
    /// `cooldown_time`: time in seconds the mob has to wait before using this spell again. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub cooldown_time: Option<f32>,
    /// `do_casting`: if true, the mob will show a casting animation while sending the event. Default `true`.
    #[facet(schema::fallback = true)]
    pub do_casting: Option<bool>,
    /// `filters`: conditions that the target must meet to be eligible for receiving the event.
    pub filters: Option<Filter>,
    /// `look_at_target`: if true, the mob will face the entity it sends an event to. Default `true`.
    #[facet(schema::fallback = true)]
    pub look_at_target: Option<bool>,
    /// `max_activation_range`: maximum distance in blocks the target can be for this spell to activate. Default 32.
    #[facet(schema::fallback = 32.0)]
    pub max_activation_range: Option<f32>,
    /// `min_activation_range`: minimum distance in blocks the target must be for this spell to activate. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub min_activation_range: Option<f32>,
    /// `particle_color`: the color of the particles spawned while casting this spell. Default white.
    pub particle_color: Option<ParticleColor>,
    /// `sequence`: list of events to send.
    pub sequence: Option<Vec<SendEventStage>>,
    /// `start_sound_event`: the sound event to play when this spell starts. Default `""`.
    pub start_sound_event: Option<SoundEvent>,
    /// `weight`: relative weight of this spell choice when randomly selected from valid choices. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub weight: Option<f32>,
}

impl SendEventChoice {
    /// `start_sound_event`, or its default `""`.
    pub fn start_sound_event(&self) -> &str {
        self.start_sound_event.as_ref().map_or("", SoundEvent::name)
    }

    /// Checks the entry's members.
    pub(crate) fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.filter("filters", self.filters.as_ref());
        c.each("sequence", &mut self.sequence, SendEventStage::check);
        c.sound("start_sound_event", self.start_sound_event.as_ref());
    }
}

/// One step of a `send_event` choice's `sequence`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SendEventStage {
    /// `base_delay`: amount of time in seconds before starting this step. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub base_delay: Option<f32>,
    /// `event`: the event to send to the entity. Default `""`.
    pub event: Option<String>,
    /// `sound_event`: the sound event to play when this step happens. Default `""`.
    pub sound_event: Option<SoundEvent>,
}

impl SendEventStage {
    /// `event`, or its default `""`.
    pub fn event(&self) -> &str {
        self.event.as_ref().map_or("", |v| v.as_str())
    }

    /// `sound_event`, or its default `""`.
    pub fn sound_event(&self) -> &str {
        self.sound_event.as_ref().map_or("", SoundEvent::name)
    }

    /// Checks the entry's members.
    pub(crate) fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.sound("sound_event", self.sound_event.as_ref());
    }
}

crate::trigger_fields!(DelayedAttack {
    on_attack: Option<EventTrigger>,
    on_kill: Option<EventTrigger>,
});
crate::trigger_fields!(KnockbackRoar {
    on_roar_end: Option<EventTrigger>,
});
crate::trigger_fields!(MeleeAttack {
    on_attack: Option<EventTrigger>,
    on_kill: Option<EventTrigger>,
});
crate::trigger_fields!(MeleeBoxAttack {
    on_attack: Option<EventTrigger>,
    on_kill: Option<EventTrigger>,
});
crate::trigger_fields!(RamAttack {
    on_start: Option<Vec<EventTrigger>>,
});
crate::trigger_fields!(StompAttack {
    on_attack: Option<EventTrigger>,
    on_kill: Option<EventTrigger>,
});
crate::trigger_fields!(UseKineticWeapon {
    on_attack: Option<EventTrigger>,
    on_kill: Option<EventTrigger>,
});
