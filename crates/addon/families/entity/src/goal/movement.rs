//! Movement goals: wandering, following, fleeing and moving to places.

use crate::component::descriptor_tags;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::schema::{self, GoalMeta, Schema};
use bedrock_addon_core::shared::{EventTrigger, Filter, FloatRange, IntRange};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_shared_shapes::ItemDescriptor;
use facet::Facet;
use facet_value::VObject;

use super::check::{Bound, Deprecated, Goal, GoalCheck, deprecated_text};
use super::common::{
    ControlFlag, DamageCause, EntityTypes, LegacyConditions, LiquidMaterial, MaterialType, PoiType,
    SoundEvent, TargetSelectionMethod, Vec3,
};
use super::molang::{push_trigger, push_triggers, visit_conditions, visit_items};

/// `minecraft:behavior.break_door`: allows this mob to break doors.
// `behavior.break_door`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.break_door")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct BreakDoor {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
}

impl Goal for BreakDoor {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.circle_around_anchor`: causes an entity to circle around an anchor point placed near a point or target.
// `behavior.circle_around_anchor`: typed from 1.21.100.
#[doc(alias = "minecraft:behavior.circle_around_anchor")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct CircleAroundAnchor {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `angle_change`: number of degrees to change this entity's facing by, when the entity selects its next anchor point. Default 15.
    #[facet(schema::fallback = 15.0)]
    pub angle_change: Option<f32>,
    /// `goal_radius`: maximum distance from the anchor-point in which this entity considers itself to have reached the anchor point. This is to prevent the entity from bouncing back and forth trying to reach a specific spot. Default 0.5.
    #[facet(schema::fallback = 0.5)]
    pub goal_radius: Option<f32>,
    /// `height_above_target_range`: the number of blocks above the target that the next anchor point can be set. This value is used only when the entity is tracking a target. Default 0 to 0.
    pub height_above_target_range: Option<FloatRange>,
    /// `height_adjustment_chance`: percent chance to determine how often to increase or decrease the current height around the anchor point. 1 = 100%. `height_change_chance` is deprecated and has been replaced with `height_adjustment_chance`. Default 0.002857.
    #[facet(schema::fallback = 0.002_857)]
    pub height_adjustment_chance: Option<f32>,
    /// `height_offset_range`: vertical distance from the anchor point this entity must stay within, upon a successful height adjustment. Default 0 to 0.
    pub height_offset_range: Option<FloatRange>,
    /// `radius_adjustment_chance`: percent chance to determine how often to increase the size of the current movement radius around the anchor point. 1 = 100%. `radius_change_chance` is deprecated and has been replaced with `radius_adjustment_chance`. Default 0.004.
    #[facet(schema::fallback = 0.004)]
    pub radius_adjustment_chance: Option<f32>,
    /// `radius_change`: the number of blocks to increase the current movement radius by, upon successful `radius_adjustment_chance`. If the current radius increases over the range maximum, the current radius will be set back to the range minimum and the entity will change between clockwise and counter-clockwise movement. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub radius_change: Option<f32>,
    /// `radius_range`: horizontal distance from the anchor point this entity must stay within upon a successful radius adjustment. Default 5 to 15.
    pub radius_range: Option<FloatRange>,
    /// `speed_multiplier`: multiplies the speed at which this entity travels to its next desired position. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
}

impl CircleAroundAnchor {
    /// `height_above_target_range`, or its default 0 to 0.
    pub fn height_above_target_range(&self) -> (f32, f32) {
        self.height_above_target_range
            .map_or((0.0, 0.0), |range| range.resolve(0.0, 0.0))
    }

    /// `height_offset_range`, or its default 0 to 0.
    pub fn height_offset_range(&self) -> (f32, f32) {
        self.height_offset_range
            .map_or((0.0, 0.0), |range| range.resolve(0.0, 0.0))
    }

    /// `radius_range`, or its default 5 to 15.
    pub fn radius_range(&self) -> (f32, f32) {
        self.radius_range
            .map_or((5.0, 15.0), |range| range.resolve(5.0, 15.0))
    }
}

impl Goal for CircleAroundAnchor {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.range(
            "height_above_target_range",
            &mut self.height_above_target_range,
        );
        c.range("height_offset_range", &mut self.height_offset_range);
        c.range("radius_range", &mut self.radius_range);
    }
}

/// `minecraft:behavior.controlled_by_player`: allows the entity to be controlled by a player which is holding the item specified in `minecraft:item_controllable` component's `control_items` field (required). Also requires the `minecraft:movement` component, and the `minecraft:rideable` component. On every tick, the entity will attempt to rotate towards where the player is facing with the control item whilst simultaneously moving forward.
// `behavior.controlled_by_player`: typed from 1.21.100.
#[doc(alias = "minecraft:behavior.controlled_by_player")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct ControlledByPlayer {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `fractional_rotation`: the entity will attempt to rotate to face where the player is facing each tick. The entity will target this percentage of their difference in their current facing angles each tick (from 0.0 to 1.0 where 1.0 = 100%). This is limited by `fractional_rotation_limit`. A value of 0.0 will result in the entity no longer turning to where the player is facing. Default 0.5; 0.5 to 1.
    #[facet(schema::fallback = 0.5, schema::min = 0.5, schema::max = 1.0)]
    pub fractional_rotation: Option<f32>,
    /// `fractional_rotation_limit`: limits the total degrees the entity can rotate to face where the player is facing on each tick. Default 5.
    #[facet(schema::fallback = 5.0)]
    pub fractional_rotation_limit: Option<f32>,
    /// `mount_speed_multiplier`: speed multiplier of mount when controlled by player. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub mount_speed_multiplier: Option<f32>,
}

impl Goal for ControlledByPlayer {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.door_interact`: allows the mob to open and close doors.
// `behavior.door_interact`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.door_interact")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DoorInteract {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
}

impl Goal for DoorInteract {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.explore_outskirts`: allows the entity to first travel to a random point on the outskirts of the village, and then explore random points within a small distance. This goal requires `minecraft:dweller` and `minecraft:navigation` to execute.
// `behavior.explore_outskirts`: typed from 1.21.130.
#[doc(alias = "minecraft:behavior.explore_outskirts")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct ExploreOutskirts {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `dist_from_boundary`: the distance from the boundary the villager must be within in to explore the outskirts. Default `[5, 0, 5]`.
    pub dist_from_boundary: Option<Vec3>,
    /// `explore_dist`: total distance in blocks the the entity will explore beyond the village bounds when choosing its travel point. Default 5.
    #[facet(schema::fallback = 5.0)]
    pub explore_dist: Option<f32>,
    /// `max_travel_time`: this is the maximum amount of time an entity will attempt to reach it's travel point on the outskirts of the village before the goal exits. Default 6e+01.
    #[facet(schema::fallback = 6.0e1)]
    pub max_travel_time: Option<f32>,
    /// `max_wait_time`: the wait time in seconds between choosing new explore points will be chosen on a random interval between this value and the minimum wait time. This value is also the total amount of time the entity will explore random points before the goal stops. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub max_wait_time: Option<f32>,
    /// `min_dist_from_target`: the entity must be within this distance for it to consider it has successfully reached its target. Default 2.25.
    #[facet(schema::fallback = 2.25)]
    pub min_dist_from_target: Option<f32>,
    /// `min_perimeter`: the minimum perimeter of the village required to run this goal. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub min_perimeter: Option<f32>,
    /// `min_wait_time`: the wait time in seconds between choosing new explore points will be chosen on a random interval between this value and the maximum wait time. Default 3.
    #[facet(schema::fallback = 3.0)]
    pub min_wait_time: Option<f32>,
    /// `next_xz`: a new explore point will randomly be chosen within this XZ distance of the current target position when navigation has finished and the wait timer has elapsed. Default 5.
    #[facet(schema::fallback = 5)]
    pub next_xz: Option<i32>,
    /// `next_y`: a new explore point will randomly be chosen within this Y distance of the current target position when navigation has finished and the wait timer has elapsed. Default 3.
    #[facet(schema::fallback = 3)]
    pub next_y: Option<i32>,
    /// `speed_multiplier`: the multiplier for speed while using this goal. 1.0 maintains the speed. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
    /// `timer_ratio`: each new explore point will be chosen on a random interval between the minimum and the maximum wait time, divided by this value. This does not apply to the first explore point chosen when the goal runs. Default 2.
    #[facet(schema::fallback = 2.0)]
    pub timer_ratio: Option<f32>,
}

impl ExploreOutskirts {
    /// `dist_from_boundary`, or its default `[5, 0, 5]`.
    pub fn dist_from_boundary(&self) -> [f32; 3] {
        self.dist_from_boundary
            .as_ref()
            .map_or([5.0, 0.0, 5.0], |v| [v.x, v.y, v.z])
    }
}

impl Goal for ExploreOutskirts {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
        c.vector(
            "dist_from_boundary",
            self.dist_from_boundary.as_ref().map(|v| &v.extra),
        );
    }
}

/// `minecraft:behavior.find_cover`: allows the mob to seek shade.
// `behavior.find_cover`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.find_cover")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct FindCover {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `cooldown_time`: time in seconds the mob has to wait before using the goal again. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub cooldown_time: Option<f32>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
}

impl Goal for FindCover {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.find_mount`: allows the mob to look around for another mob to ride atop it.
// `behavior.find_mount`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.find_mount")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct FindMount {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `avoid_water`: if true, the mob will not go into water blocks when going towards a mount. Default `false`.
    pub avoid_water: Option<bool>,
    /// `max_failed_attempts`: number of attempts to find mount. Default 20.
    #[facet(schema::fallback = 20)]
    pub max_failed_attempts: Option<i32>,
    /// `mount_distance`: this is the distance the mob needs to be, in blocks, from the desired mount to mount it. If the value is below 0, the mob will use its default attack distance. Default -1.
    #[facet(schema::fallback = -1.0)]
    pub mount_distance: Option<f32>,
    /// `start_delay`: time the mob will wait before starting to move towards the mount. Default 0.
    #[facet(schema::fallback = 0)]
    pub start_delay: Option<i32>,
    /// `target_needed`: if true, the mob will only look for a mount if it has a target. Default `false`.
    pub target_needed: Option<bool>,
    /// `within_radius`: distance in blocks within which the mob will look for a mount. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub within_radius: Option<f32>,
}

impl FindMount {
    /// `avoid_water`, or its default `false`.
    pub fn avoid_water(&self) -> bool {
        self.avoid_water.is_some_and(|v| v)
    }

    /// `target_needed`, or its default `false`.
    pub fn target_needed(&self) -> bool {
        self.target_needed.is_some_and(|v| v)
    }
}

impl Goal for FindMount {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.find_underwater_treasure`: allows the mob to move towards the nearest underwater ruin or shipwreck.
// `behavior.find_underwater_treasure`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.find_underwater_treasure")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct FindUnderwaterTreasure {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `search_range`: the range that the mob will search for a treasure chest within a ruin or shipwreck to move towards. Default 0.
    #[facet(schema::fallback = 0)]
    pub search_range: Option<i32>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
    /// `stop_distance`: the distance the mob will move before stopping. Default 2.
    #[facet(schema::fallback = 2.0)]
    pub stop_distance: Option<f32>,
}

impl Goal for FindUnderwaterTreasure {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.flee_sun`: allows the mob to run away from direct sunlight and seek shade.
///
/// Deprecated members, reported and dropped: `cooldown_time`.
// `behavior.flee_sun`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.flee_sun")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct FleeSun {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
}

impl Goal for FleeSun {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    const DEPRECATED: &'static [Deprecated] = &[(
        "cooldown_time",
        deprecated_text!("cooldown_time"),
        known::V1_26_20,
    )];

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.float`: allows the mob to stay afloat while swimming. Passengers will be kicked out the moment the mob's head goes underwater, which may not happen for tall mobs.
// `behavior.float`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.float")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Float {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `chance_per_tick_to_float`: the chance per tick to cause an upward impulse. Default 0.8.
    #[facet(schema::fallback = 0.8)]
    pub chance_per_tick_to_float: Option<f32>,
    /// `sink_with_passengers`: if true, the mob will keep sinking as long as it has passengers. Default `false`.
    pub sink_with_passengers: Option<bool>,
    /// `time_under_water_to_dismount_passengers`: time in seconds that a floating vehicles head can be underwater before it causes its passengers to dismount. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub time_under_water_to_dismount_passengers: Option<f32>,
}

impl Float {
    /// `sink_with_passengers`, or its default `false`.
    pub fn sink_with_passengers(&self) -> bool {
        self.sink_with_passengers.is_some_and(|v| v)
    }
}

impl Goal for Float {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
        if c.lenient() {
            // The game's legacy goal reader logs a negative chance at the error level and reads
            // it as 0; a chance above 1 is clamped to 1 without a word; a negative dismount time
            // logs at the error level and reads as 0. The document is kept either way.
            if let Some(chance) = &mut self.chance_per_tick_to_float {
                if *chance < 0.0 {
                    c.legacy_error(
                        "chance_per_tick_to_float",
                        "behavior.float - chance_per_tick_to_float value should be `0.0 <= value <= 1.0`",
                    );
                    *chance = 0.0;
                } else if *chance > 1.0 {
                    *chance = 1.0;
                }
            }
            if let Some(time) = &mut self.time_under_water_to_dismount_passengers
                && *time < 0.0
            {
                c.legacy_error(
                    "time_under_water_to_dismount_passengers",
                    "behavior.float - time_under_water_to_dismount_passengers should be positive",
                );
                *time = 0.0;
            }
        }
    }
}

/// `minecraft:behavior.float_wander`: allows the mob to float around like the Ghast.
// `behavior.float_wander`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.float_wander")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct FloatWander {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `additional_collision_buffer`: if true, the mob will have an additional buffer zone around it to avoid collisions with blocks when picking a position to wander to. Default `false`.
    #[facet(schema::fallback = false)]
    pub additional_collision_buffer: Option<bool>,
    /// `allow_navigating_through_liquids`: if true allows the mob to navigate through liquids on its way to the target position. Default `false`.
    #[facet(schema::fallback = false)]
    pub allow_navigating_through_liquids: Option<bool>,
    /// `float_duration`: range of time in seconds the mob will float around before landing and choosing to do something else. Default 0 to 0.
    pub float_duration: Option<FloatRange>,
    /// `float_wander_has_move_control`: if true, the `MoveControl` flag will be added to the behavior which means that it can no longer be active at the same time as other behaviors with `MoveControl`. Default `true`.
    #[facet(schema::fallback = true)]
    pub float_wander_has_move_control: Option<bool>,
    /// `must_reach`: if true, the point has to be reachable to be a valid target. Default `false`.
    #[facet(schema::fallback = false)]
    pub must_reach: Option<bool>,
    /// `navigate_around_surface`: if true, will prioritize finding random positions in the vicinity of surfaces, `i.e`. blocks that are not Air or Liquid. Default `false`.
    #[facet(schema::fallback = false)]
    pub navigate_around_surface: Option<bool>,
    /// `random_reselect`: if true, the mob will randomly pick a new point while moving to the previously selected one. Default `false`.
    #[facet(schema::fallback = false)]
    pub random_reselect: Option<bool>,
    /// `surface_xz_dist`: the horizontal distance in blocks that the goal will check for a surface from a candidate position. Only valid when `navigate_around_surface` is true. Default 8.
    #[facet(schema::fallback = 8)]
    pub surface_xz_dist: Option<i32>,
    /// `surface_y_dist`: the vertical distance in blocks that the goal will check for a surface from a candidate position. Only valid when `navigate_around_surface` is true. Default 8.
    #[facet(schema::fallback = 8)]
    pub surface_y_dist: Option<i32>,
    /// `use_home_position_restriction`: if true, the mob will respect home position restrictions when choosing new target positions. If false, it will choose target position without considering home restrictions. Default `true`.
    #[facet(schema::fallback = true)]
    pub use_home_position_restriction: Option<bool>,
    /// `xz_dist`: distance in blocks on ground that the mob will look for a new spot to move to. Must be at least 1. Default 10.
    #[facet(schema::fallback = 10)]
    pub xz_dist: Option<i32>,
    /// `y_dist`: distance in blocks that the mob will look up or down for a new spot to move to. Must be at least 1. Default 7.
    #[facet(schema::fallback = 7)]
    pub y_dist: Option<i32>,
    /// `y_offset`: height in blocks to add to the selected target position. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub y_offset: Option<f32>,
}

impl FloatWander {
    /// `float_duration`, or its default 0 to 0.
    pub fn float_duration(&self) -> (f32, f32) {
        self.float_duration
            .map_or((0.0, 0.0), |range| range.resolve(0.0, 0.0))
    }
}

impl Goal for FloatWander {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.object_range("float_duration", &mut self.float_duration);
    }
}

/// `minecraft:behavior.follow_caravan`: allows the mob to follow mobs that are in a caravan.
// `behavior.follow_caravan`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.follow_caravan")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct FollowCaravan {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `entity_count`: number of entities that can be in the caravan. Default 1.
    #[facet(schema::fallback = 1)]
    pub entity_count: Option<i32>,
    /// `entity_types`: list of entity types that this mob can follow in a caravan.
    pub entity_types: Option<EntityTypes>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
}

impl Goal for FollowCaravan {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
        c.entity_types("entity_types", self.entity_types.as_ref());
    }
}

/// `minecraft:behavior.follow_mob`: allows the mob to follow other mobs.
// `behavior.follow_mob`: typed from 1.26.50.
#[doc(alias = "minecraft:behavior.follow_mob")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct FollowMob {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `abort_navigation_on_stop`: whether the mob stops its navigation when the goal stops. Default `true`; the 1.26.50 fixer writes `false` into older documents. The newest struct deprecates the name: it is read from an older document and never written.
    #[facet(schema::fallback = true, skip_serializing_if = never_written)]
    pub abort_navigation_on_stop: Option<bool>,
    /// `filters`: if non-empty, provides criteria for filtering which nearby Mobs can be followed. If empty default criteria will be used, which will exclude Players, Squid variants, Fish variants, Tadpoles, Dolphins, and mobs of the same type as the owner of the Goal.
    pub filters: Option<Filter>,
    /// `preferred_actor_type`: the type of actor to prefer following. If left unspecified, a random actor among those in range will be chosen. Default `""`.
    pub preferred_actor_type: Option<String>,
    /// `search_range`: the distance in blocks it will look for a mob to follow. Default 0.
    #[facet(schema::fallback = 0)]
    pub search_range: Option<i32>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
    /// `stop_distance`: the distance in blocks this mob stops from the mob it is following.
    pub stop_distance: Option<f32>,
    /// `use_home_position_restriction`: if true, the mob will respect the '`minecraft:home`' component's '`restriction_radius`' field when choosing a target to follow. If false, it will choose target position without considering home restrictions. Default `false`.
    #[facet(schema::fallback = false)]
    pub use_home_position_restriction: Option<bool>,
}

impl FollowMob {
    /// `preferred_actor_type`, or its default `""`.
    pub fn preferred_actor_type(&self) -> &str {
        self.preferred_actor_type
            .as_ref()
            .map_or("", |v| v.as_str())
    }

    /// `stop_distance`, when written.
    pub fn stop_distance(&self) -> Option<f32> {
        self.stop_distance
    }
}

impl Goal for FollowMob {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.filter("filters", self.filters.as_ref());
        if self.abort_navigation_on_stop.is_some() {
            c.deprecated_member(
                "abort_navigation_on_stop",
                deprecated_text!("abort_navigation_on_stop"),
            );
        }
    }
}

/// `skip_serializing_if` of a member no document of the newest version may hold.
#[allow(
    clippy::trivially_copy_pass_by_ref,
    clippy::ref_option,
    reason = "facet's `skip_serializing_if` signature"
)]
fn never_written(_: &Option<bool>) -> bool {
    true
}

/// `minecraft:behavior.follow_owner`: allows a mob to follow the player that owns it.
// `behavior.follow_owner`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.follow_owner")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct FollowOwner {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `can_teleport`: defines if the mob will teleport to its owner when too far away. Default `true`.
    pub can_teleport: Option<bool>,
    /// `ignore_vibration`: defines if the mob should disregard following its owner after detecting a recent vibration. Default `false`.
    pub ignore_vibration: Option<bool>,
    /// `max_distance`: the maximum distance the mob can be from its owner to start following it. Applicable only when `can_teleport` is set to false. Default 6e+01.
    #[facet(schema::fallback = 6.0e1)]
    pub max_distance: Option<f32>,
    /// `post_teleport_distance`: defines how far (in blocks) the entity will be from its owner after teleporting. If not specified, it defaults to `stop_distance` + 1, allowing the entity to seamlessly resume navigation. Default 3.
    #[facet(schema::fallback = 3)]
    pub post_teleport_distance: Option<i32>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
    /// `start_distance`: the minimum distance the mob must be from its owner to start following it. Default 1e+01.
    #[facet(schema::fallback = 1.0e1)]
    pub start_distance: Option<f32>,
    /// `stop_distance`: the distance at which the mob will stop following its owner. Default 2.
    #[facet(schema::fallback = 2.0)]
    pub stop_distance: Option<f32>,
}

impl FollowOwner {
    /// `can_teleport`, or its default `true`.
    pub fn can_teleport(&self) -> bool {
        self.can_teleport.is_none_or(|v| v)
    }

    /// `ignore_vibration`, or its default `false`.
    pub fn ignore_vibration(&self) -> bool {
        self.ignore_vibration.is_some_and(|v| v)
    }
}

impl Goal for FollowOwner {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.follow_parent`: allows the mob to follow their parent around.
// `behavior.follow_parent`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.follow_parent")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct FollowParent {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
}

impl Goal for FollowParent {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.follow_target_leader`: allows mob to move towards its target leader.
// `behavior.follow_target_leader`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.follow_target_leader")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct FollowTargetLeader {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `always_look_for_leader`: if true, the mob will always search for a valid leader to follow if none is found or current leader is not valid anymore. Default `false`.
    #[facet(schema::fallback = false)]
    pub always_look_for_leader: Option<bool>,
    /// `follow_distance`: defines the distance in blocks the mob will stay from its target while following. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub follow_distance: Option<f32>,
    /// `leader_filters`: the filters to use when determining if a mob is a suitable leader or not.
    pub leader_filters: Option<Filter>,
    /// `search_cooldown`: number of ticks that must pass before a new search is initiated after a target is not found, lost, or becomes invalid. Default 20.
    #[facet(schema::fallback = 20)]
    pub search_cooldown: Option<i32>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
    /// `within_radius`: defines the maximum distance in blocks a mob can get from its target leader before giving up trying to follow it. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub within_radius: Option<f32>,
}

impl Goal for FollowTargetLeader {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    // The game's legacy goal reader logs this when `leader_filters` is present but its filter
    // group fails to parse, at the warning level, and keeps the goal.
    const LENIENT_TEXTS: &'static [(&'static str, &'static str, Severity)] = &[(
        "leader_filters",
        "'minecraft:follow_target_leader' | 'leader_filters' must specify filters to identify leader to follow.",
        Severity::Warning,
    )];

    fn legacy_member_failed(&mut self, name: &str) -> Vec<&'static str> {
        // A failed `leader_filters` ends the game's legacy reader for this goal: of its members
        // only `search_cooldown` is read after it, so that member keeps its default, and a
        // written one is told as dropped.
        if name == "leader_filters" && self.search_cooldown.take().is_some() {
            return vec!["search_cooldown"];
        }
        Vec::new()
    }

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.filter("leader_filters", self.leader_filters.as_ref());
    }
}

/// `minecraft:behavior.go_home`: allows the mob to move back to the position they were spawned.
// `behavior.go_home`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.go_home")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct GoHome {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `calculate_new_path_radius`: distance in blocks that the mob is considered close enough to the end of the current path. A new path will then be calculated to continue toward home. Default 2.
    pub calculate_new_path_radius: Option<f32>,
    /// `goal_radius`: distance in blocks within the mob considers it has reached the goal. This is the "wiggle room" to stop the AI from bouncing back and forth trying to reach a specific spot. Default 0.5.
    pub goal_radius: Option<f32>,
    /// `interval`: a random value to determine when to randomly move somewhere. This has a 1/interval chance to choose this goal. Default 120.
    pub interval: Option<i32>,
    /// `on_failed`: event(s) to run when this goal fails.
    pub on_failed: Option<Vec<EventTrigger>>,
    /// `on_home`: event(s) to run when this mob gets home.
    pub on_home: Option<Vec<EventTrigger>>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    pub speed_multiplier: Option<f32>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl GoHome {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `calculate_new_path_radius`, or its default 2.
    pub fn calculate_new_path_radius(&self) -> f32 {
        self.calculate_new_path_radius.unwrap_or(2.0)
    }

    /// `goal_radius`, or its default 0.5.
    pub fn goal_radius(&self) -> f32 {
        self.goal_radius.unwrap_or(0.5)
    }

    /// `interval`, or its default 120.
    pub fn interval(&self) -> i32 {
        self.interval.unwrap_or(120)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.unwrap_or(1.0)
    }
}

impl Goal for GoHome {
    const TYPED: FormatVersion = known::V1_26_20;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.triggers("on_failed", self.on_failed.as_deref());
        c.triggers("on_home", self.on_home.as_deref());
    }
}

impl MolangFields for GoHome {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_triggers(&mut triggers, "on_failed", self.on_failed.as_deref());
        push_triggers(&mut triggers, "on_home", self.on_home.as_deref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.hide`: allows a mob with the hide component to attempt to move to - and hide at - an owned or nearby POI.
// `behavior.hide`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.hide")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Hide {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `duration`: amount of time in seconds that the mob reacts. Default 1; at least 0.
    #[facet(schema::fallback = 1.0, schema::min = 0.0)]
    pub duration: Option<f32>,
    /// `poi_type`: tells the goal what POI type it should be looking for.
    pub poi_type: Option<PoiType>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this goal. Default 1; at least 0.
    #[facet(schema::fallback = 1.0, schema::min = 0.0)]
    pub speed_multiplier: Option<f32>,
    /// `timeout_cooldown`: the cooldown time in seconds before the goal can be reused after a internal failure or timeout condition. Default 8; at least 0.
    #[facet(schema::fallback = 8.0, schema::min = 0.0)]
    pub timeout_cooldown: Option<f32>,
}

impl Hide {
    /// `poi_type`, when written.
    pub fn poi_type(&self) -> Option<PoiType> {
        self.poi_type
    }
}

impl Goal for Hide {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.hover`: allows the mob to hover in place.
// `behavior.hover`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.hover")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Hover {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
}

impl Goal for Hover {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.jump_to_block`: allows an entity to jump to another random block.
// `behavior.jump_to_block`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.jump_to_block")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct JumpToBlock {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `cooldown_range`: minimum and maximum cooldown time-range (positive, in seconds) between each attempted jump. Default 1e+01 to 2e+01.
    pub cooldown_range: Option<FloatRange>,
    /// `forbidden_blocks`: blocks that the mob can't jump to.
    #[facet(schema::molang_with = descriptor_tags)]
    pub forbidden_blocks: Option<Vec<ItemDescriptor>>,
    /// `max_velocity`: the maximum velocity with which the mob can jump. Default 1.5.
    #[facet(schema::fallback = 1.5)]
    pub max_velocity: Option<f32>,
    /// `minimum_distance`: the minimum distance (in blocks) from the mob to a block, in order to consider jumping to it. Default 2.
    #[facet(schema::fallback = 2)]
    pub minimum_distance: Option<i32>,
    /// `minimum_path_length`: the minimum length (in blocks) of the mobs path to a block, in order to consider jumping to it. Default 5.
    #[facet(schema::fallback = 5)]
    pub minimum_path_length: Option<i32>,
    /// `preferred_blocks`: blocks that the mob prefers jumping to.
    #[facet(schema::molang_with = descriptor_tags)]
    pub preferred_blocks: Option<Vec<ItemDescriptor>>,
    /// `preferred_blocks_chance`: chance (between 0.0 and 1.0) that the mob will jump to a preferred block, if in range. Only matters if preferred blocks are defined. Default 1; 0 to 1.
    #[facet(schema::fallback = 1.0)]
    pub preferred_blocks_chance: Option<f32>,
    /// `scale_factor`: the scalefactor of the bounding box of the mob while it is jumping. Default 0.7.
    #[facet(schema::fallback = 0.7)]
    pub scale_factor: Option<f32>,
    /// `search_height`: the height (in blocks, in range \[1, 15\]) of the search box, centered around the mob. Default 10; 1 to 15.
    #[facet(schema::fallback = 10)]
    pub search_height: Option<i32>,
    /// `search_width`: the width (in blocks, in range \[1, 15\]) of the search box, centered around the mob. Default 8; 1 to 15.
    #[facet(schema::fallback = 8)]
    pub search_width: Option<i32>,
}

impl JumpToBlock {
    /// `cooldown_range`, or its default 1e+01 to 2e+01.
    pub fn cooldown_range(&self) -> (f32, f32) {
        self.cooldown_range
            .map_or((1.0e1, 2.0e1), |range| range.resolve(1.0e1, 2.0e1))
    }
}

impl Goal for JumpToBlock {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.range("cooldown_range", &mut self.cooldown_range);
        c.items("forbidden_blocks", self.forbidden_blocks.as_deref());
        c.items("preferred_blocks", self.preferred_blocks.as_deref());
        c.float(
            "preferred_blocks_chance",
            &mut self.preferred_blocks_chance,
            Bound::MinMax(0.0, 1.0),
        );
        c.int(
            "search_height",
            &mut self.search_height,
            Bound::MinMax(1, 15),
        );
        c.int("search_width", &mut self.search_width, Bound::MinMax(1, 15));
    }
}

/// `minecraft:behavior.mount_pathing`: allows the mob to move around on its own while mounted seeking a target to attack.
// `behavior.mount_pathing`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.mount_pathing")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MountPathing {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
    /// `target_dist`: the distance at which this mob wants to be away from its target. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub target_dist: Option<f32>,
    /// `track_target`: if true, this mob will chase after the target as long as it's a valid target. Default `false`.
    pub track_target: Option<bool>,
}

impl MountPathing {
    /// `track_target`, or its default `false`.
    pub fn track_target(&self) -> bool {
        self.track_target.is_some_and(|v| v)
    }
}

impl Goal for MountPathing {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.move_around_target`: allows an entity to move around a `target.If` the entity is too close(`i.e.closer` than destination range min and height difference limit) it will try to move away from its `target.If` the entity is too far away from its target it will try to move closer to a random position within the destination `range.A` randomized amount of those positions will be behind the target, and the spread can be tweaked with '`destination_pos_spread_degrees`'.
// `behavior.move_around_target`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.move_around_target")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MoveAroundTarget {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `destination_pos_spread_degrees`: this angle (in degrees) is used for controlling the spread when picking a destination position behind the target. A zero spread angle means the destination position will be straight behind the target with no variance. A 90 degree spread angle means the destination position can be up to 45 degrees to the left and to the right of the position straight behind the target's view direction. Default 9e+01.
    #[facet(schema::fallback = 9.0e1)]
    pub destination_pos_spread_degrees: Option<f32>,
    /// `destination_position_range`: the range of distances from the target entity within which the goal should look for a position to move the owner entity to. Default 4 to 8.
    pub destination_position_range: Option<FloatRange>,
    /// `filters`: conditions that need to be met for the behavior to start.
    pub filters: Option<Filter>,
    /// `height_difference_limit`: distance in height (in blocks) between the owner entity and the target has to be less than this value when owner checks if it is too close and should move away from the target. This value needs to be bigger than zero for the move away logic to trigger. Default 1e+01; at least 0.
    #[facet(schema::fallback = 1.0e1)]
    pub height_difference_limit: Option<f32>,
    /// `horizontal_search_distance`: horizontal search distance (in blocks) when searching for a position to move away from target. Default 5; at least 0.
    #[facet(schema::fallback = 5)]
    pub horizontal_search_distance: Option<i32>,
    /// `movement_speed`: the speed with which the entity should move to its target position. Default 0.6.
    #[facet(schema::fallback = 0.6)]
    pub movement_speed: Option<f32>,
    /// `vertical_search_distance`: vertical search distance (in blocks) when searching for a position to move away from target. Default 5; at least 0.
    #[facet(schema::fallback = 5)]
    pub vertical_search_distance: Option<i32>,
}

impl MoveAroundTarget {
    /// `destination_position_range`, or its default 4 to 8.
    pub fn destination_position_range(&self) -> (f32, f32) {
        self.destination_position_range
            .map_or((4.0, 8.0), |range| range.resolve(4.0, 8.0))
    }
}

impl Goal for MoveAroundTarget {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.range(
            "destination_position_range",
            &mut self.destination_position_range,
        );
        c.filter("filters", self.filters.as_ref());
        c.float(
            "height_difference_limit",
            &mut self.height_difference_limit,
            Bound::Min(0.0),
        );
        c.int(
            "horizontal_search_distance",
            &mut self.horizontal_search_distance,
            Bound::Min(0),
        );
        c.int(
            "vertical_search_distance",
            &mut self.vertical_search_distance,
            Bound::Min(0),
        );
    }
}

/// `minecraft:behavior.move_indoors`: allows this entity to move indoors.
// `behavior.move_indoors`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.move_indoors")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MoveIndoors {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `speed_multiplier`: the movement speed modifier to apply to the entity while it is moving indoors. Default 0.8.
    #[facet(schema::fallback = 0.8)]
    pub speed_multiplier: Option<f32>,
    /// `timeout_cooldown`: the cooldown time in seconds before the goal can be reused after pathfinding fails. Default 8.
    #[facet(schema::fallback = 8.0)]
    pub timeout_cooldown: Option<f32>,
}

impl Goal for MoveIndoors {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.move_outdoors`: allows this entity to move outdoors.
// `behavior.move_outdoors`: typed from 1.26.20.
// `speed_multiplier` / `search_count` are deprecated on the base and re-bound by this goal.
#[doc(alias = "minecraft:behavior.move_outdoors")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MoveOutdoors {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `goal_radius`: the radius away from the target block to count as reaching the goal. Default 2.
    #[facet(schema::fallback = 2.0)]
    pub goal_radius: Option<f32>,
    /// `search_count`: the amount of times to try finding a random outdoors position before failing. Default 10.
    #[facet(schema::fallback = 10)]
    pub search_count: Option<i32>,
    /// `search_height`: the y range to search for an outdoors position for. Default 5.
    #[facet(schema::fallback = 5)]
    pub search_height: Option<i32>,
    /// `search_range`: the x and z range to search for an outdoors position for. Default 15.
    #[facet(schema::fallback = 15)]
    pub search_range: Option<i32>,
    /// `speed_multiplier`: the movement speed modifier to apply to the entity while it is moving outdoors. Default 0.5.
    #[facet(schema::fallback = 0.5)]
    pub speed_multiplier: Option<f32>,
    /// `timeout_cooldown`: the cooldown time in seconds before the goal can be reused after pathfinding fails. Default 8.
    #[facet(schema::fallback = 8.0)]
    pub timeout_cooldown: Option<f32>,
}

impl Goal for MoveOutdoors {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        // `speed_multiplier` and `search_count` are deprecated on the base and re-bound by the
        // goal, with its own defaults (`DEFAULT_SPEED_MULTIPLIER` 0.5 and `DEFAULT_SEARCH_COUNT`
        // 10) and no bound (the game's 1.26.20 schema gives none, where the goals bound by the
        // base give `minimum: 0`).
        c.base(&mut self.priority);
    }
}

/// `minecraft:behavior.move_through_village`: can only be used by Villagers. Allows the villagers to create paths around the village.
// `behavior.move_through_village`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.move_through_village")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MoveThroughVillage {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `only_at_night`: if true, the mob will only move through the village during night time. Default `false`.
    pub only_at_night: Option<bool>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
}

impl MoveThroughVillage {
    /// `only_at_night`, or its default `false`.
    pub fn only_at_night(&self) -> bool {
        self.only_at_night.is_some_and(|v| v)
    }
}

impl Goal for MoveThroughVillage {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.move_to_block`: allows mob to move towards a block.
// `behavior.move_to_block`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.move_to_block")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct MoveToBlock {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `goal_radius`: distance in blocks within the mob considers it has reached the goal. This is the "wiggle room" to stop the AI from bouncing back and forth trying to reach a specific spot. Default 0.5.
    pub goal_radius: Option<f32>,
    /// `on_reach`: event to run on block reached.
    pub on_reach: Option<Vec<EventTrigger>>,
    /// `on_stay_completed`: event to run on completing a stay of `stay_duration` at the block.
    pub on_stay_completed: Option<Vec<EventTrigger>>,
    /// `search_height`: the height in blocks that the mob will look for the block. Default 1.
    pub search_height: Option<i32>,
    /// `search_range`: the distance in blocks that the mob will look for the block. Default 0.
    pub search_range: Option<i32>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    pub speed_multiplier: Option<f32>,
    /// `start_chance`: chance to start the behavior (applied after each random `tick_interval`). Default 1.
    pub start_chance: Option<f32>,
    /// `stay_duration`: number of ticks needed to complete a stay at the block. Default 0.
    pub stay_duration: Option<f32>,
    /// `target_block_filters`: conditions that need to be met for a block to be a valid target.
    pub target_block_filters: Option<Filter>,
    /// `target_blocks`: block types to move to.
    pub target_blocks: Option<Vec<ItemDescriptor>>,
    /// `target_offset`: offset to add to the selected target position. Default `[0, 0, 0]`.
    pub target_offset: Option<Vec3>,
    /// `target_selection_method`: kind of block to find fitting the specification. Valid values are `random` and `nearest`. Default `nearest`.
    pub target_selection_method: Option<TargetSelectionMethod>,
    /// `tick_interval`: average interval in ticks to try to run this behavior. Default 20.
    pub tick_interval: Option<i32>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl MoveToBlock {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `goal_radius`, or its default 0.5.
    pub fn goal_radius(&self) -> f32 {
        self.goal_radius.unwrap_or(0.5)
    }

    /// `search_height`, or its default 1.
    pub fn search_height(&self) -> i32 {
        self.search_height.unwrap_or(1)
    }

    /// `search_range`, or its default 0.
    pub fn search_range(&self) -> i32 {
        self.search_range.unwrap_or(0)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.unwrap_or(1.0)
    }

    /// `start_chance`, or its default 1.
    pub fn start_chance(&self) -> f32 {
        self.start_chance.unwrap_or(1.0)
    }

    /// `stay_duration`, or its default 0.
    pub fn stay_duration(&self) -> f32 {
        self.stay_duration.unwrap_or(0.0)
    }

    /// `target_offset`, or its default `[0, 0, 0]`.
    pub fn target_offset(&self) -> [f32; 3] {
        self.target_offset
            .as_ref()
            .map_or([0.0, 0.0, 0.0], |v| [v.x, v.y, v.z])
    }

    /// `target_selection_method`, or its default `nearest`.
    pub fn target_selection_method(&self) -> TargetSelectionMethod {
        self.target_selection_method
            .unwrap_or(TargetSelectionMethod::Nearest)
    }

    /// `tick_interval`, or its default 20.
    pub fn tick_interval(&self) -> i32 {
        self.tick_interval.unwrap_or(20)
    }
}

impl Goal for MoveToBlock {
    const TYPED: FormatVersion = known::V1_26_40;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.triggers("on_reach", self.on_reach.as_deref());
        c.triggers("on_stay_completed", self.on_stay_completed.as_deref());
        c.filter("target_block_filters", self.target_block_filters.as_ref());
        c.items("target_blocks", self.target_blocks.as_deref());
        c.vector(
            "target_offset",
            self.target_offset.as_ref().map(|v| &v.extra),
        );
    }
}

impl MolangFields for MoveToBlock {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visit_items(visitor, "target_blocks", &mut self.target_blocks);
        let mut triggers = Vec::new();
        push_triggers(&mut triggers, "on_reach", self.on_reach.as_deref());
        push_triggers(
            &mut triggers,
            "on_stay_completed",
            self.on_stay_completed.as_deref(),
        );
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.move_to_land`: allows the mob to move back onto land when in water.
// `behavior.move_to_land`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.move_to_land")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MoveToLand {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `goal_radius`: distance in blocks within the mob considers it has reached the goal. This is the "wiggle room" to stop the AI from bouncing back and forth trying to reach a specific spot. Default 0.5; greater than 0.
    #[facet(schema::fallback = 0.5, schema::above = 0.0)]
    pub goal_radius: Option<f32>,
    /// `search_count`: the number of randomly selected blocks each tick that the mob will check within its search range and height for a valid block to move to. A value of 0 will have the mob check every block within range in one tick. Default 10; at least 0.
    #[facet(schema::fallback = 10, schema::min = 0)]
    pub search_count: Option<i32>,
    /// `search_height`: height in blocks the mob will look for land to move towards. Default 1; greater than 0.
    #[facet(schema::fallback = 1, schema::above = 0)]
    pub search_height: Option<i32>,
    /// `search_range`: the distance in blocks it will look for land to move towards. Default 0; greater than 0.
    #[facet(schema::fallback = 0, schema::above = 0)]
    pub search_range: Option<i32>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this goal. Default 1; at least 0.
    #[facet(schema::fallback = 1.0, schema::min = 0.0)]
    pub speed_multiplier: Option<f32>,
}

impl Goal for MoveToLand {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.move_to_lava`: allows the mob to move back into lava when on land.
///
/// Deprecated members, reported and dropped: `material_type`.
// `behavior.move_to_lava`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.move_to_lava")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MoveToLava {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `goal_radius`: distance in blocks within the mob considers it has reached the goal. This is the "wiggle room" to stop the AI from bouncing back and forth trying to reach a specific spot. Default 0.5; greater than 0.
    #[facet(schema::fallback = 0.5, schema::above = 0.0)]
    pub goal_radius: Option<f32>,
    /// `search_count`: the number of randomly selected blocks each tick that the mob will check within its search range and height for a valid block to move to. A value of 0 will have the mob check every block within range in one tick. Default 10; at least 0.
    #[facet(schema::fallback = 10, schema::min = 0)]
    pub search_count: Option<i32>,
    /// `search_height`: height in blocks the mob will look for lava to move towards. Default 1; greater than 0.
    #[facet(schema::fallback = 1, schema::above = 0)]
    pub search_height: Option<i32>,
    /// `search_range`: the distance in blocks it will look for lava to move towards. Default 0; greater than 0.
    #[facet(schema::fallback = 0, schema::above = 0)]
    pub search_range: Option<i32>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this goal. Default 1; at least 0.
    #[facet(schema::fallback = 1.0, schema::min = 0.0)]
    pub speed_multiplier: Option<f32>,
}

impl Goal for MoveToLava {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    const DEPRECATED: &'static [Deprecated] = &[(
        "material_type",
        deprecated_text!("material_type"),
        known::V1_26_10,
    )];

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.move_to_liquid`: allows the mob to move into a liquid when on land.
// `behavior.move_to_liquid`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.move_to_liquid")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MoveToLiquid {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `goal_radius`: distance in blocks within the mob considers it has reached the goal. This is the "wiggle room" to stop the AI from bouncing back and forth trying to reach a specific spot. Default 0.5; greater than 0.
    #[facet(schema::fallback = 0.5, schema::above = 0.0)]
    pub goal_radius: Option<f32>,
    /// `material_type`: the material type of the liquid block to find. Valid values are `Any`, `Water`, and `Lava`. Default `any`.
    pub material_type: Option<MaterialType>,
    /// `search_count`: the number of randomly selected blocks each tick that the mob will check within its search range and height for a valid block to move to. A value of 0 will have the mob check every block within range in one tick. Default 10; at least 0.
    #[facet(schema::fallback = 10, schema::min = 0)]
    pub search_count: Option<i32>,
    /// `search_height`: height in blocks the mob will look for the liquid block to move towards. Default 1; greater than 0.
    #[facet(schema::fallback = 1, schema::above = 0)]
    pub search_height: Option<i32>,
    /// `search_range`: the distance in blocks it will look for the liquid block to move towards. Default 0; greater than 0.
    #[facet(schema::fallback = 0, schema::above = 0)]
    pub search_range: Option<i32>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this goal. Default 1; at least 0.
    #[facet(schema::fallback = 1.0, schema::min = 0.0)]
    pub speed_multiplier: Option<f32>,
}

impl MoveToLiquid {
    /// `material_type`, or its default `any`.
    pub fn material_type(&self) -> LiquidMaterial {
        self.material_type
            .as_ref()
            .map_or(LiquidMaterial::Any, |m| m.material)
    }
}

impl Goal for MoveToLiquid {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.move_to_poi`: allows the mob to move to a POI if able to.
// `behavior.move_to_poi`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.move_to_poi")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MoveToPoi {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `poi_type`: tells the goal what POI type it should be looking for.
    pub poi_type: Option<PoiType>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this goal. Default 1; at least 0.
    #[facet(schema::fallback = 1.0, schema::min = 0.0)]
    pub speed_multiplier: Option<f32>,
}

impl MoveToPoi {
    /// `poi_type`, when written.
    pub fn poi_type(&self) -> Option<PoiType> {
        self.poi_type
    }
}

impl Goal for MoveToPoi {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.move_to_random_block`: allows mob to move towards a random block.
// `behavior.move_to_random_block`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.move_to_random_block")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MoveToRandomBlock {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `block_distance`: defines the distance from the mob, in blocks, that the block to move to will be chosen. Default 16.
    #[facet(schema::fallback = 16.0)]
    pub block_distance: Option<f32>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
    /// `within_radius`: defines the distance in blocks the mob has to be from the block for the movement to be finished. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub within_radius: Option<f32>,
}

impl Goal for MoveToRandomBlock {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.move_to_village`: allows the mob to move into a random location within a village.
// `behavior.move_to_village`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.move_to_village")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MoveToVillage {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `cooldown_time`: time in seconds the mob has to wait before using the goal again. Default 8; at least 0.
    #[facet(schema::fallback = 8.0, schema::min = 0.0)]
    pub cooldown_time: Option<f32>,
    /// `goal_radius`: distance in blocks within the mob considers it has reached the goal. This is the "wiggle room" to stop the AI from bouncing back and forth trying to reach a specific spot. Default 1.5; at least 0.
    #[facet(schema::fallback = 1.5, schema::min = 0.0)]
    pub goal_radius: Option<f32>,
    /// `search_range`: the distance in blocks to search for villages. If \<= 0, find the closest village regardless of distance. Default 0.
    #[facet(schema::fallback = 0)]
    pub search_range: Option<i32>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this goal. Default 1; at least 0.
    #[facet(schema::fallback = 1.0, schema::min = 0.0)]
    pub speed_multiplier: Option<f32>,
}

impl Goal for MoveToVillage {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.move_to_water`: allows the mob to move back into water when on land.
///
/// Deprecated members, reported and dropped: `material_type`.
// `behavior.move_to_water`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.move_to_water")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MoveToWater {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `goal_radius`: distance in blocks within the mob considers it has reached the goal. This is the "wiggle room" to stop the AI from bouncing back and forth trying to reach a specific spot. Default 0.5; greater than 0.
    #[facet(schema::fallback = 0.5, schema::above = 0.0)]
    pub goal_radius: Option<f32>,
    /// `search_count`: the number of randomly selected blocks each tick that the mob will check within its search range and height for a valid block to move to. A value of 0 will have the mob check every block within range in one tick. Default 10; at least 0.
    #[facet(schema::fallback = 10, schema::min = 0)]
    pub search_count: Option<i32>,
    /// `search_height`: height in blocks the mob will look for water to move towards. Default 1; greater than 0.
    #[facet(schema::fallback = 1, schema::above = 0)]
    pub search_height: Option<i32>,
    /// `search_range`: the distance in blocks it will look for water to move towards. Default 0; greater than 0.
    #[facet(schema::fallback = 0, schema::above = 0)]
    pub search_range: Option<i32>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this goal. Default 1; at least 0.
    #[facet(schema::fallback = 1.0, schema::min = 0.0)]
    pub speed_multiplier: Option<f32>,
}

impl Goal for MoveToWater {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    const DEPRECATED: &'static [Deprecated] = &[(
        "material_type",
        deprecated_text!("material_type"),
        known::V1_26_10,
    )];

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.move_towards_dwelling_restriction`: allows entities with the `minecraft:dweller` component to move toward their village area that the entity should be restricted to.
// `behavior.move_towards_dwelling_restriction`: typed from 1.26.0.
#[doc(alias = "minecraft:behavior.move_towards_dwelling_restriction")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MoveTowardsDwellingRestriction {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `speed_multiplier`: this multiplier modifies the entity's speed when moving towards its restriction. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
}

impl Goal for MoveTowardsDwellingRestriction {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.move_towards_home_restriction`: allows entities with a `minecraft:home` component to move towards their home position. If `restriction_radius` is set, entities will be able to run this behavior only if outside of it.
// `behavior.move_towards_home_restriction`: typed from 1.26.0.
#[doc(alias = "minecraft:behavior.move_towards_home_restriction")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MoveTowardsHomeRestriction {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `speed_multiplier`: this multiplier modifies the entity's speed when moving towards its restriction. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
}

impl Goal for MoveTowardsHomeRestriction {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.move_towards_target`: allows mob to move towards its current target.
// `behavior.move_towards_target`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.move_towards_target")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MoveTowardsTarget {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
    /// `within_radius`: defines the radius in blocks that the mob tries to be from the target. A value of 0 means it tries to occupy the same block as the target. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub within_radius: Option<f32>,
}

impl Goal for MoveTowardsTarget {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.ocelot_sit_on_block`: allows to mob to be able to sit in place like the ocelot.
///
/// Deprecated members, reported and dropped: `search_count`.
// `behavior.ocelot_sit_on_block`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.ocelot_sit_on_block")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct OcelotSitOnBlock {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this goal. Default 1; at least 0.
    #[facet(schema::fallback = 1.0, schema::min = 0.0)]
    pub speed_multiplier: Option<f32>,
}

impl Goal for OcelotSitOnBlock {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    const DEPRECATED: &'static [Deprecated] = &[(
        "search_count",
        deprecated_text!("search_count"),
        known::V1_26_10,
    )];

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.open_door`: allows the mob to open doors. Requires the mob to be able to path through doors, otherwise the mob won't even want to try opening them.
// `behavior.open_door`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.open_door")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct OpenDoor {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `close_door_after`: if true, the mob will close the door after opening it and going through it. Default `true`.
    pub close_door_after: Option<bool>,
}

impl OpenDoor {
    /// `close_door_after`, or its default `true`.
    pub fn close_door_after(&self) -> bool {
        self.close_door_after.is_none_or(|v| v)
    }
}

impl Goal for OpenDoor {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.player_ride_tamed`: allows the mob to be ridden by the player after being tamed.
// `behavior.player_ride_tamed`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.player_ride_tamed")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct PlayerRideTamed {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
}

impl Goal for PlayerRideTamed {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.random_breach`: allows the mob to randomly break surface of the water.
// `behavior.random_breach`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.random_breach")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct RandomBreach {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `cooldown_time`: time in seconds the mob has to wait before using the goal again. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub cooldown_time: Option<f32>,
    /// `interval`: a random value to determine when to randomly move somewhere. This has a 1/interval chance to choose this goal. Default 120.
    #[facet(schema::fallback = 120)]
    pub interval: Option<i32>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
    /// `xz_dist`: distance in blocks on ground that the mob will look for a new spot to move to. Must be at least 1. Default 10.
    #[facet(schema::fallback = 10)]
    pub xz_dist: Option<i32>,
    /// `y_dist`: distance in blocks that the mob will look up or down for a new spot to move to. Must be at least 1. Default 7.
    #[facet(schema::fallback = 7)]
    pub y_dist: Option<i32>,
}

impl Goal for RandomBreach {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.random_fly`: allows a mob to randomly fly around.
///
/// Deprecated members, reported and dropped: `interval`.
// `behavior.random_fly`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.random_fly")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct RandomFly {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `can_land_on_trees`: if true, the mob can stop flying and land on a tree instead of the ground. Default `true`.
    pub can_land_on_trees: Option<bool>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
    /// `xz_dist`: distance in blocks on ground that the mob will look for a new spot to move to. Must be at least 1. Default 10.
    #[facet(schema::fallback = 10)]
    pub xz_dist: Option<i32>,
    /// `y_dist`: distance in blocks that the mob will look up or down for a new spot to move to. Must be at least 1. Default 7.
    #[facet(schema::fallback = 7)]
    pub y_dist: Option<i32>,
}

impl RandomFly {
    /// `can_land_on_trees`, or its default `true`.
    pub fn can_land_on_trees(&self) -> bool {
        self.can_land_on_trees.is_none_or(|v| v)
    }
}

impl Goal for RandomFly {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    const DEPRECATED: &'static [Deprecated] =
        &[("interval", deprecated_text!("interval"), known::V1_26_30)];

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.random_hover`: allows the mob to hover around randomly, close to the surface.
// `behavior.random_hover`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.random_hover")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct RandomHover {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `hover_height`: the height above the surface which the mob will try to maintain. Default 0 to 0.
    pub hover_height: Option<IntRange>,
    /// `interval`: a random value to determine when to randomly move somewhere. This has a 1/interval chance to choose this goal. Default 120.
    #[facet(schema::fallback = 120)]
    pub interval: Option<i32>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
    /// `xz_dist`: distance in blocks on ground that the mob will look for a new spot to move to. Must be at least 1. Default 10.
    #[facet(schema::fallback = 10)]
    pub xz_dist: Option<i32>,
    /// `y_dist`: distance in blocks that the mob will look up or down for a new spot to move to. Must be at least 1. Default 7.
    #[facet(schema::fallback = 7)]
    pub y_dist: Option<i32>,
    /// `y_offset`: height in blocks to add to the selected target position. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub y_offset: Option<f32>,
}

impl RandomHover {
    /// `hover_height`, or its default 0 to 0.
    pub fn hover_height(&self) -> (i32, i32) {
        self.hover_height
            .map_or((0, 0), |range| range.resolve(0, 0))
    }
}

impl Goal for RandomHover {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.range("hover_height", &mut self.hover_height);
    }
}

/// `minecraft:behavior.random_stroll`: allows a mob to randomly stroll around.
// `behavior.random_stroll`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.random_stroll")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct RandomStroll {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `interval`: a random value to determine when to randomly move somewhere. This has a 1/interval chance to choose this goal. Default 120.
    #[facet(schema::fallback = 120)]
    pub interval: Option<i32>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
    /// `xz_dist`: distance in blocks on ground that the mob will look for a new spot to move to. Must be at least 1. Default 10.
    #[facet(schema::fallback = 10)]
    pub xz_dist: Option<i32>,
    /// `y_dist`: distance in blocks that the mob will look up or down for a new spot to move to. Must be at least 1. Default 7.
    #[facet(schema::fallback = 7)]
    pub y_dist: Option<i32>,
}

impl Goal for RandomStroll {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.random_swim`: allows an entity to randomly move through water.
// `behavior.random_swim`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.random_swim")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct RandomSwim {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `avoid_surface`: if true, the mob will avoid surface water blocks by swimming below them. Default `true`.
    pub avoid_surface: Option<bool>,
    /// `interval`: a random value to determine when to randomly move somewhere. This has a 1/interval chance to choose this goal. Default 120.
    #[facet(schema::fallback = 120)]
    pub interval: Option<i32>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
    /// `xz_dist`: distance in blocks on ground that the mob will look for a new spot to move to. Must be at least 1. Default 10.
    #[facet(schema::fallback = 10)]
    pub xz_dist: Option<i32>,
    /// `y_dist`: distance in blocks that the mob will look up or down for a new spot to move to. Must be at least 1. Default 7.
    #[facet(schema::fallback = 7)]
    pub y_dist: Option<i32>,
}

impl RandomSwim {
    /// `avoid_surface`, or its default `true`.
    pub fn avoid_surface(&self) -> bool {
        self.avoid_surface.is_none_or(|v| v)
    }
}

impl Goal for RandomSwim {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.restrict_open_door`: allows the mob to stay indoors during night time.
// `behavior.restrict_open_door`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.restrict_open_door")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct RestrictOpenDoor {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
}

impl Goal for RestrictOpenDoor {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.restrict_sun`: allows the mob to automatically start avoiding the sun when its a clear day out.
// `behavior.restrict_sun`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.restrict_sun")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct RestrictSun {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
}

impl Goal for RestrictSun {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.rise_to_liquid_level`: allows the mob to stay at a certain level when in liquid.
// `behavior.rise_to_liquid_level`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.rise_to_liquid_level")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct RiseToLiquidLevel {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `liquid_y_offset`: target distance down from the liquid surface. `i.e`. Positive values move the target Y down. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub liquid_y_offset: Option<f32>,
    /// `rise_delta`: movement up in Y per tick when below the liquid surface. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub rise_delta: Option<f32>,
    /// `sink_delta`: movement down in Y per tick when above the liquid surface. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub sink_delta: Option<f32>,
}

impl Goal for RiseToLiquidLevel {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.slime_float`: allow slimes to float in water / lava. Can only be used by Slime and Magma Cubes.
// `behavior.slime_float`: typed from 1.21.110.
#[doc(alias = "minecraft:behavior.slime_float")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SlimeFloat {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `jump_chance_percentage`: percent chance a slime or magma cube has to jump while in water / lava. Default 0.8.
    #[facet(schema::fallback = 0.8)]
    pub jump_chance_percentage: Option<f32>,
    /// `speed_multiplier`: determines the multiplier the entity's speed is modified by when moving through water / lava. Default 1.2.
    #[facet(schema::fallback = 1.2)]
    pub speed_multiplier: Option<f32>,
}

impl Goal for SlimeFloat {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.slime_keep_on_jumping`: allows the entity to continuously jump around like a slime.
// `behavior.slime_keep_on_jumping`: typed from 1.21.110.
#[doc(alias = "minecraft:behavior.slime_keep_on_jumping")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SlimeKeepOnJumping {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `speed_multiplier`: determines the multiplier this entity's speed is modified by when jumping around. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
}

impl Goal for SlimeKeepOnJumping {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.slime_random_direction`: allows the entity to move in random directions like a slime.
// `behavior.slime_random_direction`: typed from 1.21.110.
#[doc(alias = "minecraft:behavior.slime_random_direction")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SlimeRandomDirection {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `add_random_time_range`: additional time (in whole seconds), chosen randomly in the range of \[0, `add_random_time_range`\], to add to `min_change_direction_time`. Default 3.
    #[facet(schema::fallback = 3)]
    pub add_random_time_range: Option<i32>,
    /// `min_change_direction_time`: constant minimum time (in seconds) to wait before choosing a new direction. Default 2.
    #[facet(schema::fallback = 2.0)]
    pub min_change_direction_time: Option<f32>,
    /// `turn_range`: maximum rotation angle range (in degrees) when randomly choosing a new direction. Default 360.
    #[facet(schema::fallback = 360)]
    pub turn_range: Option<i32>,
}

impl Goal for SlimeRandomDirection {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.squid_dive`: allows the squid to dive down in water. Can only be used by the Squid.
// `behavior.squid_dive`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.squid_dive")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SquidDive {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
}

impl Goal for SquidDive {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.squid_flee`: allows the squid to swim away. Can only be used by the Squid.
// `behavior.squid_flee`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.squid_flee")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SquidFlee {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
}

impl Goal for SquidFlee {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.squid_idle`: allows the squid to swim in place idly. Can only be used by the Squid.
// `behavior.squid_idle`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.squid_idle")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SquidIdle {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
}

impl Goal for SquidIdle {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.squid_move_away_from_ground`: allows the squid to move away from ground blocks and back to water. Can only be used by the Squid.
// `behavior.squid_move_away_from_ground`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.squid_move_away_from_ground")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SquidMoveAwayFromGround {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
}

impl Goal for SquidMoveAwayFromGround {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.squid_out_of_water`: allows the squid to stick to the ground when outside water. Can only be used by the Squid.
// `behavior.squid_out_of_water`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.squid_out_of_water")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SquidOutOfWater {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
}

impl Goal for SquidOutOfWater {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.stroll_towards_village`: allows the mob to move into a random location within a village within the search range.
// `behavior.stroll_towards_village`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.stroll_towards_village")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct StrollTowardsVillage {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `cooldown_time`: time in seconds the mob has to wait before using the goal again. Default 8; at least 0.
    #[facet(schema::fallback = 8.0, schema::min = 0.0)]
    pub cooldown_time: Option<f32>,
    /// `goal_radius`: distance in blocks within the mob considers it has reached the goal. This is the "wiggle room" to stop the AI from bouncing back and forth trying to reach a specific spot. Default 1.5; at least 0.
    #[facet(schema::fallback = 1.5, schema::min = 0.0)]
    pub goal_radius: Option<f32>,
    /// `search_range`: the distance in blocks to search for villages. If \<= 0, find the closest village regardless of distance. Default 0.
    #[facet(schema::fallback = 0)]
    pub search_range: Option<i32>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this goal. Default 1; at least 0.
    #[facet(schema::fallback = 1.0, schema::min = 0.0)]
    pub speed_multiplier: Option<f32>,
    /// `start_chance`: chance that the mob will start this goal, from 0 to 1. Default 1; at least 0.
    #[facet(schema::fallback = 1.0, schema::min = 0.0)]
    pub start_chance: Option<f32>,
}

impl Goal for StrollTowardsVillage {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.swim_idle`: allows the entity go idle, if swimming. Entity must be in water.
// `behavior.swim_idle`: typed from 1.21.110.
#[doc(alias = "minecraft:behavior.swim_idle")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SwimIdle {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `idle_time`: amount of time (in seconds) to stay idle. Default 5.
    #[facet(schema::fallback = 5.0)]
    pub idle_time: Option<f32>,
    /// `success_rate`: percent chance this entity will go idle, 1.0 = 100%. Default 0.1.
    #[facet(schema::fallback = 0.1)]
    pub success_rate: Option<f32>,
}

impl Goal for SwimIdle {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.swim_up_for_breath`: allows the mob to try to move to air once it is close to running out of its total breathable supply. Requires `minecraft:breathable`.
// `behavior.swim_up_for_breath`: typed from 1.21.110.
#[doc(alias = "minecraft:behavior.swim_up_for_breath")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SwimUpForBreath {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `material_type`: the material the mob is traveling in. An air block will only be considered valid to move to with a block of this material below it. Options are: `water`, `lava`, or `any`, in any case (written back as read). Default `water`.
    // The liquid materials are the ones `move_to_liquid` accepts.
    pub material_type: Option<MaterialType>,
    /// `search_height`: the height (in blocks) above the mob's current position that it will search for a valid air block to move to. If a valid block cannot be found, the mob will move to the position this many blocks above it. Default 16.
    #[facet(schema::fallback = 16)]
    pub search_height: Option<i32>,
    /// `search_radius`: the radius (in blocks) around the mob's current position that it will search for a valid air block to move to. Default 4.
    #[facet(schema::fallback = 4)]
    pub search_radius: Option<i32>,
    /// `speed_mod`: movement speed multiplier of the mob when using this Goal. Default 1.4.
    #[facet(schema::fallback = 1.4)]
    pub speed_mod: Option<f32>,
}

impl SwimUpForBreath {
    /// `material_type`, or its default `water`.
    pub fn material_type(&self) -> LiquidMaterial {
        self.material_type
            .as_ref()
            .map_or(LiquidMaterial::Water, |m| m.material)
    }
}

impl Goal for SwimUpForBreath {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.swim_wander`: allows the entity to wander around while swimming, when not path-finding.
// `behavior.swim_wander`: typed from 1.21.110.
#[doc(alias = "minecraft:behavior.swim_wander")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SwimWander {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `interval`: percent chance to start wandering, when not path-finding. 1 = 100%. Default 0.00833.
    #[facet(schema::fallback = 0.008_33)]
    pub interval: Option<f32>,
    /// `look_ahead`: distance to look ahead for obstacle avoidance, while wandering. Default 5.
    #[facet(schema::fallback = 5.0)]
    pub look_ahead: Option<f32>,
    /// `speed_multiplier`: this multiplier modifies the entity's speed when wandering. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
    /// `wander_time`: amount of time (in seconds) to wander after wandering behavior was successfully started. Default 5.
    #[facet(schema::fallback = 5.0)]
    pub wander_time: Option<f32>,
}

impl Goal for SwimWander {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.swim_with_entity`: allows the entity to follow another entity of the matching types while both swim, matching its direction and catching up when it falls behind.
// `behavior.swim_with_entity`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.swim_with_entity")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SwimWithEntity {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `catch_up_multiplier`: the multiplier this entity's speed is modified by when matching another entity's direction. Default 2.5; greater than 0.
    #[facet(schema::fallback = 2.5)]
    pub catch_up_multiplier: Option<f32>,
    /// `catch_up_threshold`: distance, from the entity being followed, at which this entity will speed up to reach that entity. Default 12; greater than 0.
    #[facet(schema::fallback = 12.0)]
    pub catch_up_threshold: Option<f32>,
    /// `chance_to_stop`: percent chance to stop following the current entity, if they're riding another entity or they're not swimming. 1.0 = 100%. Default 0.0333; greater than 0.
    #[facet(schema::fallback = 0.0333)]
    pub chance_to_stop: Option<f32>,
    /// `entity_types`: filters which types of entities are valid to follow.
    pub entity_types: Option<EntityTypes>,
    /// `match_direction_threshold`: distance, from the entity being followed, at which this entity will try to match that entity's direction. Default 2; greater than 0.
    #[facet(schema::fallback = 2.0)]
    pub match_direction_threshold: Option<f32>,
    /// `search_range`: radius around this entity to search for another entity to follow. Default 2e+01; greater than 0.
    #[facet(schema::fallback = 2.0e1)]
    pub search_range: Option<f32>,
    /// `speed_multiplier`: the multiplier this entity's speed is modified by when trying to catch up to the entity being followed. Default 1.5; greater than 0.
    #[facet(schema::fallback = 1.5)]
    pub speed_multiplier: Option<f32>,
    /// `state_check_interval`: time (in seconds) between checks to determine if this entity should catch up to the entity being followed or match the direction of the entity being followed. Default 0.5; at least 0.
    #[facet(schema::fallback = 0.5)]
    pub state_check_interval: Option<f32>,
    /// `stop_distance`: distance, from the entity being followed, at which this entity will stop following that entity. Default 5; greater than 0.
    #[facet(schema::fallback = 5.0)]
    pub stop_distance: Option<f32>,
    /// `success_rate`: percent chance to start following another entity, if not already doing so. 1.0 = 100%. Default 0.1; greater than 0.
    #[facet(schema::fallback = 0.1)]
    pub success_rate: Option<f32>,
}

impl Goal for SwimWithEntity {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        // Bounds by hand, not schema::min/max: a bound here follows another check, and the
        // derived check runs first, which would reorder the findings.
        c.base(&mut self.priority);
        c.float(
            "catch_up_multiplier",
            &mut self.catch_up_multiplier,
            Bound::Above(0.0),
        );
        c.float(
            "catch_up_threshold",
            &mut self.catch_up_threshold,
            Bound::Above(0.0),
        );
        c.float(
            "chance_to_stop",
            &mut self.chance_to_stop,
            Bound::Above(0.0),
        );
        c.entity_types("entity_types", self.entity_types.as_ref());
        c.float(
            "match_direction_threshold",
            &mut self.match_direction_threshold,
            Bound::Above(0.0),
        );
        c.float("search_range", &mut self.search_range, Bound::Above(0.0));
        c.float(
            "speed_multiplier",
            &mut self.speed_multiplier,
            Bound::Above(0.0),
        );
        c.float(
            "state_check_interval",
            &mut self.state_check_interval,
            Bound::Min(0.0),
        );
        c.float("stop_distance", &mut self.stop_distance, Bound::Above(0.0));
        c.float("success_rate", &mut self.success_rate, Bound::Above(0.0));
    }
}

/// `minecraft:behavior.teleport_to_owner`: allows an entity to teleport to its owner.
// `behavior.teleport_to_owner`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.teleport_to_owner")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct TeleportToOwner {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `cooldown`: the time in seconds that must pass for the entity to be able to try to teleport again. Default 1; at least 0.
    #[facet(schema::fallback = 1.0)]
    pub cooldown: Option<f32>,
    /// `filters`: conditions to be satisfied for the entity to teleport to its owner.
    pub filters: Option<Filter>,
}

impl Goal for TeleportToOwner {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.float("cooldown", &mut self.cooldown, Bound::Min(0.0));
        c.filter("filters", self.filters.as_ref());
    }
}

/// `minecraft:behavior.wither_random_attack_pos_goal`: allows the wither to launch random attacks. Can only be used by the Wither Boss.
///
/// Deprecated members, reported and dropped: `speed_multiplier`, `xz_dist`, `y_dist`, `interval`.
// `behavior.wither_random_attack_pos_goal`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.wither_random_attack_pos_goal")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct WitherRandomAttackPosGoal {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
}

impl Goal for WitherRandomAttackPosGoal {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    const DEPRECATED: &'static [Deprecated] = &[
        (
            "speed_multiplier",
            deprecated_text!("speed_multiplier"),
            known::V1_26_30,
        ),
        ("xz_dist", deprecated_text!("xz_dist"), known::V1_26_30),
        ("y_dist", deprecated_text!("y_dist"), known::V1_26_30),
        ("interval", deprecated_text!("interval"), known::V1_26_30),
    ];

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

// ---- the four goals typed at 1.26.60: `panic`, `tempt`, `float_tempt`, `send_event` ----

pub(crate) use crate::versions::GOALS_TYPED_1_26_60 as V1_26_60;

/// The typed default of a level sound event member of the 1.26.60 goals: the event id 615,
/// read as `undefined` (the event that a name the sound map does not know and the legacy `""`
/// both resolve to).
pub(crate) const UNDEFINED_SOUND: &str = "undefined";

/// Vanilla's message for an empty `panic.damage_sources` in an older document (logged at the
/// error level; the document still loads).
pub(crate) const NO_DAMAGE_SOURCE: &str = "No damage source provided in \"damage_sources\".";
/// Vanilla's message for `all` beside other causes in an older document's
/// `panic.damage_sources` (logged at the warning level).
pub(crate) const ALL_IS_NOT_THE_ONLY_SOURCE: &str = "\"all\" is not the only damage source in the list. The other damage sources can be removed because \"all\" covers every damage source.";

/// `minecraft:behavior.panic`: allows a mob to panic when receiving damage, making it run around and away from the damage source that made it enter this state.
#[doc(alias = "minecraft:behavior.panic")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Panic {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `damage_sources`: the list of damage sources that will cause this mob to panic. Default empty (the game then picks its list by the document's version).
    pub damage_sources: Option<Vec<DamageCause>>,
    /// `force`: if true, the mob will stop panicking only when it can no longer find positions to run away to, or when the goal is removed. Default `false`.
    #[facet(schema::fallback = false)]
    pub force: Option<bool>,
    /// `ignore_mob_damage`: if true, the mob will not panic in response to damage from other mobs. This overrides the damage types in `damage_sources`. Default `false`.
    #[facet(schema::fallback = false)]
    pub ignore_mob_damage: Option<bool>,
    /// `panic_sound`: sound to play while the mob is panicking. Default `undefined`.
    pub panic_sound: Option<SoundEvent>,
    /// `prefer_water`: if true, the mob will prefer water over land. Default `false`.
    #[facet(schema::fallback = false)]
    pub prefer_water: Option<bool>,
    /// `sound_interval`: range of random ticks to wait between playing the panic sound. Default 0 to 0.
    pub sound_interval: Option<FloatRange>,
    /// `speed_multiplier`: movement speed multiplier when using this goal. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
}

impl Panic {
    /// `damage_sources`, or its default (empty).
    pub fn damage_sources(&self) -> &[DamageCause] {
        self.damage_sources.as_deref().unwrap_or_default()
    }

    /// `panic_sound`, or its default `undefined`.
    pub fn panic_sound(&self) -> &str {
        self.panic_sound
            .as_ref()
            .map_or(UNDEFINED_SOUND, SoundEvent::name)
    }

    /// `sound_interval`, or its default 0 to 0.
    pub fn sound_interval(&self) -> (f32, f32) {
        self.sound_interval
            .map_or((0.0, 0.0), |range| range.resolve(0.0, 0.0))
    }
}

impl Goal for Panic {
    const TYPED: FormatVersion = V1_26_60;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.sound("panic_sound", self.panic_sound.as_ref());
        c.range("sound_interval", &mut self.sound_interval);
    }

    /// An older document's present but empty list, and `all` beside other causes, are told
    /// with the legacy reader's texts; the list is kept as written. The empty list is logged
    /// by the game at the error level and the goal is kept: a kept error. `all` beside
    /// others is logged at the warning level: a warning.
    fn lenient_shape(json: &mut VObject, d: &mut Diagnostics) {
        let Some(list) = json.get("damage_sources").and_then(Value::as_array) else {
            return;
        };
        let text = if list.is_empty() {
            Some(NO_DAMAGE_SOURCE)
        } else if list.len() > 1
            && list
                .iter()
                .any(|cause| cause.as_string().is_some_and(|c| c.as_str() == "all"))
        {
            Some(ALL_IS_NOT_THE_ONLY_SOURCE)
        } else {
            None
        };
        if let Some(text) = text {
            let finding = legacy_warning("damage_sources", text);
            if text == NO_DAMAGE_SOURCE {
                d.push_kept_error(finding);
            } else {
                d.push(finding);
            }
        }
    }
}

/// A warning of the legacy goal parser at the goal's member `key`, with its text (it logs and
/// loads the goal): the game's warning level for `all` beside others; the caller makes
/// the error-level empty list a kept error.
pub(crate) fn legacy_warning(key: &str, text: &'static str) -> Diagnostic {
    let at = JsonPath::new().child(key);
    Diagnostic::warn(DiagnosticCode::InvalidValue {
        text: Some(VanillaText::Fixed(text)),
    })
    .at(at)
}

/// The members of the game's shared tempt base, common to `tempt` and `float_tempt`: one
/// struct per goal, each with every member of the base and the same defaults, named for its
/// goal.
macro_rules! tempt_base {
    ($(#[$meta:meta])* $name:ident, $key:literal) => {
        $(#[$meta])*
        #[doc(alias = $key)]
        #[derive(Clone, Debug, Default, PartialEq, Facet)]
        #[facet(schema::unknown = "keep")]
        pub struct $name {
            /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
            pub priority: Option<i32>,
            /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
            pub control_flags: Option<Vec<ControlFlag>>,
            /// `can_get_scared`: if true, the mob can stop being tempted if the player moves too fast while close to this mob. Default `false`.
            pub can_get_scared: Option<bool>,
            /// `can_tempt_vertically`: if true, vertical distance to the player will be considered when tempting. Default `false`.
            pub can_tempt_vertically: Option<bool>,
            /// `can_tempt_while_ridden`: if true, the mob can be tempted even if it has a passenger (i.e. if being ridden). Default `false`.
            pub can_tempt_while_ridden: Option<bool>,
            /// `items`: list of items that can tempt the mob.
            pub items: Option<Vec<ItemDescriptor>>,
            /// `on_tempt_end`: event to trigger when this mob stops being tempted.
            pub on_tempt_end: Option<EventTrigger>,
            /// `on_tempt_start`: event to trigger when this mob starts being tempted.
            pub on_tempt_start: Option<EventTrigger>,
            /// `sound_interval`: range of random ticks to wait between tempt sounds. Default 0 to 0.
            pub sound_interval: Option<FloatRange>,
            /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
            pub speed_multiplier: Option<f32>,
            /// `stop_distance`: the distance at which the mob will stop following the player.
            pub stop_distance: Option<f32>,
            /// `tempt_sound`: sound to play while the mob is being tempted. Default `undefined`.
            pub tempt_sound: Option<SoundEvent>,
            /// `within_radius`: distance in blocks this mob can get tempted by a player holding an item they like. Default 0.
            pub within_radius: Option<f32>,
            /// The legacy `condition`s of the goal's triggers, read for the Molang check;
            /// written back from each trigger's `extra`.
            #[doc(hidden)]
            #[facet(skip, rename = "\u{0}legacy_conditions")]
            pub legacy_conditions: LegacyConditions,
        }

        impl $name {
            /// `priority`, or its default 0.
            pub fn priority(&self) -> i32 {
                self.priority.map_or(0, |v| v)
            }

            /// `can_get_scared`, or its default `false`.
            pub fn can_get_scared(&self) -> bool {
                self.can_get_scared.is_some_and(|v| v)
            }

            /// `can_tempt_vertically`, or its default `false`.
            pub fn can_tempt_vertically(&self) -> bool {
                self.can_tempt_vertically.is_some_and(|v| v)
            }

            /// `can_tempt_while_ridden`, or its default `false`.
            pub fn can_tempt_while_ridden(&self) -> bool {
                self.can_tempt_while_ridden.is_some_and(|v| v)
            }

            /// `sound_interval`, or its default 0 to 0.
            pub fn sound_interval(&self) -> (f32, f32) {
                self.sound_interval
                    .map_or((0.0, 0.0), |range| range.resolve(0.0, 0.0))
            }

            /// `speed_multiplier`, or its default 1.
            pub fn speed_multiplier(&self) -> f32 {
                self.speed_multiplier.map_or(1.0, |v| v)
            }

            /// `stop_distance`, when written.
            pub fn stop_distance(&self) -> Option<f32> {
                self.stop_distance.map(|v| v)
            }

            /// `tempt_sound`, or its default `undefined`.
            pub fn tempt_sound(&self) -> &str {
                self.tempt_sound
                    .as_ref()
                    .map_or(UNDEFINED_SOUND, SoundEvent::name)
            }

            /// `within_radius`, or its default 0.
            pub fn within_radius(&self) -> f32 {
                self.within_radius.map_or(0.0, |v| v)
            }
        }

        // Its two triggers; its Molang walk (the triggers' legacy conditions) crosses fields,
        // which the derive's per-field walk cannot, so it derives no `Schema`.
        crate::trigger_fields!($name {
            on_tempt_end: Option<EventTrigger>,
            on_tempt_start: Option<EventTrigger>,
        });

        impl Goal for $name {
            const TYPED: FormatVersion = V1_26_60;


            fn check(&mut self, c: &mut GoalCheck<'_>) {
                c.base(&mut self.priority);
                c.items("items", self.items.as_deref());
                c.trigger("on_tempt_end", self.on_tempt_end.as_ref());
                c.trigger("on_tempt_start", self.on_tempt_start.as_ref());
                c.range("sound_interval", &mut self.sound_interval);
                c.sound("tempt_sound", self.tempt_sound.as_ref());
            }
        }

        impl MolangFields for $name {
            fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
                visit_items(visitor, "items", &mut self.items);
                let mut triggers = Vec::new();
                push_trigger(&mut triggers, "on_tempt_end", self.on_tempt_end.as_ref());
                push_trigger(&mut triggers, "on_tempt_start", self.on_tempt_start.as_ref());
                visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
            }
        }
    };
}

tempt_base!(
    /// `minecraft:behavior.tempt`: allows a mob to be tempted by a player holding a specific item. Uses pathfinding for movement.
    Tempt,
    "minecraft:behavior.tempt"
);

tempt_base!(
    /// `minecraft:behavior.float_tempt`: allows a mob to be tempted by a player holding a specific item. Uses point-to-point movement. Designed for mobs that are floating (e.g. use the "minecraft:navigation.float" component).
    FloatTempt,
    "minecraft:behavior.float_tempt"
);

crate::trigger_fields!(GoHome {
    on_failed: Option<Vec<EventTrigger>>,
    on_home: Option<Vec<EventTrigger>>,
});
crate::trigger_fields!(MoveToBlock {
    on_reach: Option<Vec<EventTrigger>>,
    on_stay_completed: Option<Vec<EventTrigger>>,
});
