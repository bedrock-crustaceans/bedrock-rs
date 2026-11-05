//! The Brain prototype's memory goals (`minecraft:memory_behavior.*`) and memory sensors
//! (`minecraft:memory_sensor.*`): goals that act on the entity's declared memories, and sensors
//! that fill them. Vanilla binds them only for entities with a `brain`.
//!
//! No schema gives these members' types: where a member's JSON shape is not known it is kept as
//! an untyped [`Value`].
// The member sets here are the typed Brain group. Where the game's schema does not fix a
// member's type, the crate picks the type that reads the members the game's own packs write.

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, JsonPath};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{StrictFloat, StrictInt, StrictString, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor, Step};
use bedrock_addon_core::shared::{BlockDescriptor, EventTrigger, Filter, FloatRange};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use facet::Facet;

use super::check::{Bound, Check, Goal};
use super::common::{ControlFlag, GameEventType, LegacyConditions, SoundEvent, Vec2, Vec3};
use super::molang::{push_triggers, visit_blocks, visit_conditions};

/// `minecraft:memory_behavior.move_to_position.<name>`: moves the mob to the block position a
/// memory holds, sticking to it until the memory is cleared. The key needs a `.<name>` suffix
/// (`[A-Za-z0-9_]`, 1 to 32 characters); goals with different suffixes are different goals.
// Typed from 1.26.20, on top of the shared memory-goal members (`priority`, `control_flags`,
// `conditional_filters`, `chance_to_start`, `retry_attempt_cooldown`, `duration`).
#[doc(alias = "minecraft:memory_behavior.move_to_position")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct MemoryMoveToPosition {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `conditional_filters`: the goal starts only when these pass.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub conditional_filters: Option<Filter>,
    /// `chance_to_start`: the chance per try that the goal starts.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub chance_to_start: Option<StrictFloat<f32>>,
    /// `retry_attempt_cooldown`: how long to wait before trying again.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub retry_attempt_cooldown: Option<StrictFloat<f32>>,
    /// `duration`: how long the goal runs, in seconds, a number or a range; absent, without
    /// end (a shared memory-goal member).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub duration: Option<FloatRange>,
    /// `speed_modifier`: the movement speed multiplier while moving.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_modifier: Option<StrictFloat<f32>>,
    /// `goal_radius`: how close counts as arrived.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub goal_radius: Option<StrictFloat<f32>>,
    /// `maximum_range`: the largest distance to a position the goal moves to.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub maximum_range: Option<StrictFloat<f32>>,
    /// `interval`: seconds between position checks, a range.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub interval: Option<FloatRange>,
    /// `memory`: the block-position memory to move to.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub memory: Option<StrictString>,
    /// `on_reach`: the events fired when the position is reached.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_reach: Option<Vec<EventTrigger>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check; written back
    /// from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl MemoryMoveToPosition {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }
}

impl Goal for MemoryMoveToPosition {
    const TYPED: FormatVersion = known::V1_26_20;

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.filter("conditional_filters", self.conditional_filters.as_ref());
        c.float(
            "chance_to_start",
            &mut self.chance_to_start,
            Bound::MinMax(0.0, 1.0),
        );
        c.seconds("duration", &mut self.duration);
        c.seconds("interval", &mut self.interval);
        for (key, slot) in [
            ("speed_modifier", &mut self.speed_modifier),
            ("goal_radius", &mut self.goal_radius),
            ("maximum_range", &mut self.maximum_range),
        ] {
            c.float_at_least(key, slot, 0.0);
        }
        c.triggers("on_reach", self.on_reach.as_deref());
    }
}

impl MolangFields for MemoryMoveToPosition {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_triggers(&mut triggers, "on_reach", self.on_reach.as_deref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:memory_behavior.wait`: the mob waits for a while.
// Typed from 1.26.0, built on the shared memory-goal members from 1.26.30. The current
// definition adds `on_complete` and makes the shared `duration` optional.
#[doc(alias = "minecraft:memory_behavior.wait")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct MemoryWait {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `conditional_filters`: the goal starts only when these pass.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub conditional_filters: Option<Filter>,
    /// `chance_to_start`: the chance per try that the goal starts.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub chance_to_start: Option<StrictFloat<f32>>,
    /// `retry_attempt_cooldown`: how long to wait before trying again.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub retry_attempt_cooldown: Option<StrictFloat<f32>>,
    /// `duration`: how long to wait, in seconds, a number or a range; absent, without end (the
    /// shared memory-goal `duration`, optional).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub duration: Option<FloatRange>,
    /// `on_complete`: the events fired when the wait ends (a list of event triggers).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_complete: Option<Vec<EventTrigger>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl MemoryWait {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }
}

impl Goal for MemoryWait {
    const TYPED: FormatVersion = known::V1_26_0;

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.filter("conditional_filters", self.conditional_filters.as_ref());
        c.float(
            "chance_to_start",
            &mut self.chance_to_start,
            Bound::MinMax(0.0, 1.0),
        );
        c.seconds("duration", &mut self.duration);
        c.triggers("on_complete", self.on_complete.as_deref());
    }
}

impl MolangFields for MemoryWait {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:memory_behavior.look_at_entity`: the mob looks at the entity a memory holds.
// Typed from 1.26.10: `look_distance` (required, at least 0), `look_angle` (a `Vec2`, each
// component 0 to 360) and `target_entity` (required). The current definition has no
// `look_time`; it takes the shared memory-goal members instead.
#[doc(alias = "minecraft:memory_behavior.look_at_entity")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct MemoryLookAtEntity {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `conditional_filters`: the goal starts only when these pass (a shared memory-goal member).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub conditional_filters: Option<Filter>,
    /// `chance_to_start`: the chance per try that the goal starts; 0 to 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub chance_to_start: Option<StrictFloat<f32>>,
    /// `retry_attempt_cooldown`: how long to wait before trying again, in seconds.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub retry_attempt_cooldown: Option<StrictFloat<f32>>,
    /// `duration`: how long the goal runs, in seconds, a number or a range; absent, without
    /// end.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub duration: Option<FloatRange>,
    /// `look_distance`: the largest distance at which the mob looks. Required; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub look_distance: Option<StrictFloat<f32>>,
    /// `look_angle`: the angles of view, in degrees, a `Vec2`, each
    /// component 0 to 360.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub look_angle: Option<Vec2>,
    /// `target_entity`: the entity memory to look at. Required.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub target_entity: Option<StrictString>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl MemoryLookAtEntity {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }
}

impl Goal for MemoryLookAtEntity {
    const TYPED: FormatVersion = known::V1_26_10;

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.filter("conditional_filters", self.conditional_filters.as_ref());
        c.float(
            "chance_to_start",
            &mut self.chance_to_start,
            Bound::MinMax(0.0, 1.0),
        );
        c.seconds("duration", &mut self.duration);
        c.required("look_distance", self.look_distance.is_some());
        c.float("look_distance", &mut self.look_distance, Bound::Min(0.0));
        c.vector("look_angle", self.look_angle.as_ref().map(|v| &v.extra));
        if self
            .look_angle
            .as_ref()
            .is_some_and(|v| [v.x, v.y].iter().any(|a| !(0.0..=360.0).contains(a)))
        {
            c.reject("look_angle", None);
            self.look_angle = None;
        }
        c.required("target_entity", self.target_entity.is_some());
    }
}

impl MolangFields for MemoryLookAtEntity {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:memory_behavior.follow_target`: the mob follows the entity a memory holds.
// Typed from 1.26.20: `target_memory` (an `entity_ref` memory) and `speed_modifier`. The
// current definition also has `min_range` and the shared memory-goal members.
#[doc(alias = "minecraft:memory_behavior.follow_target")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct MemoryFollowTarget {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `conditional_filters`: the goal starts only when these pass (a shared memory-goal member).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub conditional_filters: Option<Filter>,
    /// `chance_to_start`: the chance per try that the goal starts; 0 to 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub chance_to_start: Option<StrictFloat<f32>>,
    /// `retry_attempt_cooldown`: how long to wait before trying again, in seconds.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub retry_attempt_cooldown: Option<StrictFloat<f32>>,
    /// `duration`: how long the goal runs, in seconds, a number or a range; absent, without
    /// end.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub duration: Option<FloatRange>,
    /// `target_memory`: the memory, of type `entity_ref`, naming the entity to follow.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub target_memory: Option<StrictString>,
    /// `speed_modifier`: the movement speed multiplier while following; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_modifier: Option<StrictFloat<f32>>,
    /// `min_range`: the distance the mob keeps from the target; at least 0 (a number).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_range: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl MemoryFollowTarget {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }
}

impl Goal for MemoryFollowTarget {
    const TYPED: FormatVersion = known::V1_26_20;

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.filter("conditional_filters", self.conditional_filters.as_ref());
        c.float(
            "chance_to_start",
            &mut self.chance_to_start,
            Bound::MinMax(0.0, 1.0),
        );
        c.seconds("duration", &mut self.duration);
        c.float_at_least("speed_modifier", &mut self.speed_modifier, 0.0);
        c.float_at_least("min_range", &mut self.min_range, 0.0);
    }
}

impl MolangFields for MemoryFollowTarget {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// What a sensor does when its condition changes: fire `event` and write memories.
// A response names the event to fire and the memory setters that write the memories.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct SensorResponse {
    /// `event`: the event to fire on the entity.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub event: Option<StrictString>,
    /// `setters`: the memories to write.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub setters: Option<Vec<MemorySetter>>,
    /// Members the response does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One memory a [`SensorResponse`] writes.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct MemorySetter {
    /// `memory`: the memory's name.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub memory: Option<StrictString>,
    /// `value`: the value to write, of the memory's type.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub value: Option<Value>,
    /// Members the setter does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// Declares a memory sensor: `sensor_entries` (required), a list of entries of `$entry`.
macro_rules! sensor {
    (
        $(#[$meta:meta])*
        $name:ident($entry:ident, $key:literal, $typed:expr)
    ) => {
        $(#[$meta])*
        #[doc(alias = $key)]
        #[derive(Clone, Debug, Default, PartialEq, Facet)]
        pub struct $name {
            /// `sensor_entries`: the sensor's entries, each searched on its own. Required.
            #[facet(default, skip_serializing_if = Option::is_none)]
            pub sensor_entries: Option<Vec<$entry>>,
            /// Members the sensor does not define, kept for write-back.
            #[facet(skip, rename = "\u{0}extra")]
            pub extra: Extra,
        }

        impl Goal for $name {
            const TYPED: FormatVersion = $typed;

            fn extra_mut(&mut self) -> &mut Extra {
                &mut self.extra
            }

            fn check(&mut self, c: &mut Check<'_>) {
                c.required("sensor_entries", self.sensor_entries.is_some());
                c.each("sensor_entries", &mut self.sensor_entries, $entry::check);
            }
        }
    };
}

sensor! {
    /// `minecraft:memory_sensor.find_nearest_block`: finds the nearest block of a set and
    /// stores its position in a memory.
    // Typed from 1.26.30.
    MemorySensorFindNearestBlock(FindNearestBlockEntry, "minecraft:memory_sensor.find_nearest_block", known::V1_26_30)
}

sensor! {
    /// `minecraft:memory_sensor.find_nearest_entity`: finds the nearest entity the filters
    /// pass and stores it in a memory.
    // Typed from 1.26.20; the members follow the game's legacy parser.
    MemorySensorFindNearestEntity(FindNearestEntityEntry, "minecraft:memory_sensor.find_nearest_entity", known::V1_26_20)
}

sensor! {
    /// `minecraft:memory_sensor.find_nearest_poi`: finds the nearest point of interest of a
    /// kind and stores it in a memory.
    // Typed from 1.26.20; the members follow the game's legacy parser.
    MemorySensorFindNearestPoi(FindNearestPoiEntry, "minecraft:memory_sensor.find_nearest_poi", known::V1_26_20)
}

sensor! {
    /// `minecraft:memory_sensor.game_event`: reacts to game events (a goat horn, an instrument)
    /// near the entity.
    // A sensor component whose entries each name a game event type.
    MemorySensorGameEvent(GameEventEntry, "minecraft:memory_sensor.game_event", known::V1_26_20)
}

sensor! {
    /// `minecraft:memory_sensor.in_range_of_entity`: tracks whether the entity a memory holds
    /// is in range and responds when that changes.
    // Typed from 1.26.0 and declared again at 1.26.20.
    MemorySensorInRangeOfEntity(InRangeOfEntityEntry, "minecraft:memory_sensor.in_range_of_entity", known::V1_26_0)
}

impl MolangFields for MemorySensorFindNearestBlock {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let Some(entries) = &mut self.sensor_entries else {
            return;
        };
        visitor.enter(Step::from("sensor_entries"));
        for (i, entry) in entries.iter_mut().enumerate() {
            visitor.enter(Step::from(i));
            visit_blocks(visitor, "blocks", &mut entry.blocks);
            visitor.leave();
        }
        visitor.leave();
    }
}

impl MolangFields for MemorySensorFindNearestEntity {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

impl MolangFields for MemorySensorFindNearestPoi {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

impl MolangFields for MemorySensorGameEvent {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

impl MolangFields for MemorySensorInRangeOfEntity {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// One entry of `memory_sensor.find_nearest_block`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct FindNearestBlockEntry {
    /// `conditional_filters`: the entry searches only when these pass.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub conditional_filters: Option<Filter>,
    /// `cooldown`: seconds between searches.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown: Option<StrictFloat<f32>>,
    /// `search_blockpos_memory`: the block-position memory to search around.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_blockpos_memory: Option<StrictString>,
    /// `found_blockpos_memory`: the block-position memory the result goes to.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub found_blockpos_memory: Option<StrictString>,
    /// `blocks`: the blocks searched for.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub blocks: Option<Vec<BlockDescriptor>>,
    /// `search_range`: the horizontal search distance, as written.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_range: Option<Value>,
    /// `search_height`: the vertical search distance, as written.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_height: Option<Value>,
    /// Members the entry does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl FindNearestBlockEntry {
    fn check(&mut self, c: &mut Check<'_>) {
        c.filter("conditional_filters", self.conditional_filters.as_ref());
        c.float("cooldown", &mut self.cooldown, Bound::Min(0.0));
        c.blocks("blocks", self.blocks.as_deref());
    }
}

/// One entry of `memory_sensor.find_nearest_entity`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct FindNearestEntityEntry {
    /// `name`: the entry's name.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub name: Option<StrictString>,
    /// `found_entity_memory`: the entity memory the result goes to.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub found_entity_memory: Option<StrictString>,
    /// `cooldown`: seconds between searches.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown: Option<StrictFloat<f32>>,
    /// `conditional_filters`: the entry searches only when these pass.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub conditional_filters: Option<Filter>,
    /// `target_filters`: the entities that match.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub target_filters: Option<Filter>,
    /// `search_range`: the search box's half extents `[x, y, z]`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_range: Option<Vec3>,
    /// Members the entry does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl FindNearestEntityEntry {
    fn check(&mut self, c: &mut Check<'_>) {
        c.float("cooldown", &mut self.cooldown, Bound::Min(0.0));
        c.filter("conditional_filters", self.conditional_filters.as_ref());
        c.filter("target_filters", self.target_filters.as_ref());
    }
}

/// One entry of `memory_sensor.find_nearest_poi`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct FindNearestPoiEntry {
    /// `name`: the entry's name.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub name: Option<StrictString>,
    /// `found_poi_memory`: the point-of-interest memory the result goes to.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub found_poi_memory: Option<StrictString>,
    /// `cooldown`: seconds between searches.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown: Option<StrictFloat<f32>>,
    /// `conditional_filters`: the entry searches only when these pass.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub conditional_filters: Option<Filter>,
    /// `poi_name`: the point-of-interest type or tag searched for.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub poi_name: Option<StrictString>,
    /// `search_radius`: the search distance.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_radius: Option<StrictFloat<f32>>,
    /// Members the entry does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl FindNearestPoiEntry {
    fn check(&mut self, c: &mut Check<'_>) {
        c.float("cooldown", &mut self.cooldown, Bound::Min(0.0));
        c.filter("conditional_filters", self.conditional_filters.as_ref());
    }
}

/// One entry of `memory_sensor.game_event`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct GameEventEntry {
    /// `source_entity_filters`: the entities whose events count.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub source_entity_filters: Option<Filter>,
    /// `radius`: how far away an event is heard.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub radius: Option<StrictFloat<f32>>,
    /// `event_type`: which game events count.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub event_type: Option<GameEventType>,
    /// `response`: what the sensor does when it hears one.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub response: Option<SensorResponse>,
    /// Members the entry does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl GameEventEntry {
    fn check(&mut self, c: &mut Check<'_>) {
        c.filter("source_entity_filters", self.source_entity_filters.as_ref());
        c.float("radius", &mut self.radius, Bound::Min(0.0));
    }
}

/// One entry of `memory_sensor.in_range_of_entity`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct InRangeOfEntityEntry {
    /// `conditional_filters`: the entry checks only when these pass.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub conditional_filters: Option<Filter>,
    /// `cooldown`: seconds between checks.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown: Option<StrictFloat<f32>>,
    /// `target_entity_memory`: the entity memory whose distance is checked.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub target_entity_memory: Option<StrictString>,
    /// `search_range`: the range, as written.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_range: Option<Value>,
    /// `entered_range_response`: when the entity comes into range.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub entered_range_response: Option<SensorResponse>,
    /// `exited_range_response`: when the entity leaves the range.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub exited_range_response: Option<SensorResponse>,
    /// `within_range_response`: while the entity is in range.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub within_range_response: Option<SensorResponse>,
    /// `out_of_range_response`: while the entity is out of range.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub out_of_range_response: Option<SensorResponse>,
    /// Members the entry does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl InRangeOfEntityEntry {
    fn check(&mut self, c: &mut Check<'_>) {
        c.filter("conditional_filters", self.conditional_filters.as_ref());
        c.float("cooldown", &mut self.cooldown, Bound::Min(0.0));
    }
}

/// `minecraft:memory_behavior.jump_to_block.<name>`: the mob jumps to the block position a
/// memory holds, with the velocity another holds. Requires a jumpable-block sensor.
// Typed from 1.26.50, on top of the base members every generic goal has.
#[doc(alias = "minecraft:memory_behavior.jump_to_block")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct MemoryJumpToBlock {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `jump_velocity_memory`: the `vec3` memory holding the jump velocity.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub jump_velocity_memory: Option<StrictString>,
    /// `on_complete`: events run after the mob lands.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_complete: Option<Vec<EventTrigger>>,
    /// `scale_factor`: the bounding box scale during the jump. Default 0.7; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub scale_factor: Option<StrictFloat<f32>>,
    /// `target_blockpos_memory`: the `block_pos` memory holding the target.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub target_blockpos_memory: Option<StrictString>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl MemoryJumpToBlock {
    /// `scale_factor`, or its default 0.7.
    pub fn scale_factor(&self) -> f32 {
        self.scale_factor.map_or(0.7, |v| v.0)
    }
}

impl Goal for MemoryJumpToBlock {
    const TYPED: FormatVersion = BRAIN_1_26_50;

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.float("scale_factor", &mut self.scale_factor, Bound::Min(0.0));
        c.triggers("on_complete", self.on_complete.as_deref());
    }
}

impl MolangFields for MemoryJumpToBlock {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_triggers(&mut triggers, "on_complete", self.on_complete.as_deref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:memory_behavior.prepare_ram_attack.<name>`: watches the entity the mob prepares
/// to ram and tells when it leaves the position the preparation was based on.
// Typed from 1.26.50.
#[doc(alias = "minecraft:memory_behavior.prepare_ram_attack")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct MemoryPrepareRamAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `on_entity_moved`: events run once the target has left the held position.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_entity_moved: Option<Vec<EventTrigger>>,
    /// `target_entity_memory`: the `entity_ref` memory holding the target.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub target_entity_memory: Option<StrictString>,
    /// `target_position_memory`: the `block_pos` memory holding the target's position.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub target_position_memory: Option<StrictString>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Goal for MemoryPrepareRamAttack {
    const TYPED: FormatVersion = BRAIN_1_26_50;

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.triggers("on_entity_moved", self.on_entity_moved.as_deref());
    }
}

impl MolangFields for MemoryPrepareRamAttack {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_triggers(
            &mut triggers,
            "on_entity_moved",
            self.on_entity_moved.as_deref(),
        );
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:memory_behavior.ram_attack.<name>`: oversees the mob's charge at a target.
// Typed from 1.26.50. The two sound members default to the sound event `undefined`.
#[doc(alias = "minecraft:memory_behavior.ram_attack")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct MemoryRamAttack {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `baby_knockback_modifier`: the knockback modifier of babies. Default 1/3.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub baby_knockback_modifier: Option<StrictFloat<f32>>,
    /// `knockback_force`: the force of the ram's knockback. Default 5.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub knockback_force: Option<StrictFloat<f32>>,
    /// `knockback_height`: the height of the ram's knockback. Default 0.1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub knockback_height: Option<StrictFloat<f32>>,
    /// `on_collide`: events run once the charge has hit something.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_collide: Option<Vec<EventTrigger>>,
    /// `pre_ram_sound`: the sound when the charge begins. Default `undefined`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub pre_ram_sound: Option<SoundEvent>,
    /// `ram_impact_sound`: the sound when the charge hits. Default `undefined`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub ram_impact_sound: Option<SoundEvent>,
    /// `ram_position_memory`: the `block_pos` memory holding the position charged towards.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub ram_position_memory: Option<StrictString>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl MemoryRamAttack {
    /// `baby_knockback_modifier`, or its default 1/3.
    pub fn baby_knockback_modifier(&self) -> f32 {
        self.baby_knockback_modifier.map_or(1.0 / 3.0, |v| v.0)
    }

    /// `knockback_force`, or its default 5.
    pub fn knockback_force(&self) -> f32 {
        self.knockback_force.map_or(5.0, |v| v.0)
    }

    /// `knockback_height`, or its default 0.1.
    pub fn knockback_height(&self) -> f32 {
        self.knockback_height.map_or(0.1, |v| v.0)
    }

    /// `pre_ram_sound`, or its default `undefined`.
    pub fn pre_ram_sound(&self) -> &str {
        self.pre_ram_sound
            .as_ref()
            .map_or("undefined", SoundEvent::name)
    }

    /// `ram_impact_sound`, or its default `undefined`.
    pub fn ram_impact_sound(&self) -> &str {
        self.ram_impact_sound
            .as_ref()
            .map_or("undefined", SoundEvent::name)
    }
}

impl Goal for MemoryRamAttack {
    const TYPED: FormatVersion = BRAIN_1_26_50;

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.triggers("on_collide", self.on_collide.as_deref());
        c.sound("pre_ram_sound", self.pre_ram_sound.as_ref());
        c.sound("ram_impact_sound", self.ram_impact_sound.as_ref());
    }
}

impl MolangFields for MemoryRamAttack {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_triggers(&mut triggers, "on_collide", self.on_collide.as_deref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// The version that types the four ram / jump Brain keys, which the 1.26.50 format introduces.
const BRAIN_1_26_50: FormatVersion = known::V1_26_50;

sensor! {
    /// `minecraft:memory_sensor.find_ram_charge_position`: picks the position to charge a
    /// target from and stores it, the target's position and the position past it.
    // Typed from 1.26.50.
    MemorySensorFindRamChargePosition(FindRamChargePositionEntry, "minecraft:memory_sensor.find_ram_charge_position", BRAIN_1_26_50)
}

impl MolangFields for MemorySensorFindRamChargePosition {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// One entry of `memory_sensor.find_ram_charge_position`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct FindRamChargePositionEntry {
    /// `charge_position_memory`: the `block_pos` memory the charge position goes to.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub charge_position_memory: Option<StrictString>,
    /// `conditional_filters`: the entry searches only when these pass.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub conditional_filters: Option<Filter>,
    /// `cooldown`: seconds between searches, a number or a range.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown: Option<FloatRange>,
    /// `max_ram_distance`: the largest distance between the target and the charge position. Default 7; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_ram_distance: Option<StrictInt<i32>>,
    /// `min_ram_distance`: the smallest distance between the target and the charge position. Default 4; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_ram_distance: Option<StrictInt<i32>>,
    /// `ram_position_memory`: the `block_pos` memory the position past the target goes to.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub ram_position_memory: Option<StrictString>,
    /// `target_entity_memory`: the `entity_ref` memory holding the target.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub target_entity_memory: Option<StrictString>,
    /// `target_position_memory`: the `block_pos` memory the target's position goes to.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub target_position_memory: Option<StrictString>,
    /// Members the entry does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl FindRamChargePositionEntry {
    /// `min_ram_distance`, or its default 4.
    pub fn min_ram_distance(&self) -> i32 {
        self.min_ram_distance.map_or(4, |v| v.0)
    }

    /// `max_ram_distance`, or its default 7.
    pub fn max_ram_distance(&self) -> i32 {
        self.max_ram_distance.map_or(7, |v| v.0)
    }

    fn check(&mut self, c: &mut Check<'_>) {
        c.filter("conditional_filters", self.conditional_filters.as_ref());
        c.range("cooldown", &mut self.cooldown);
        c.int(
            "min_ram_distance",
            &mut self.min_ram_distance,
            Bound::Min(0),
        );
        c.int(
            "max_ram_distance",
            &mut self.max_ram_distance,
            Bound::Min(0),
        );
    }
}

/// The generic goal key that needs a `.<name>` suffix, without it.
pub(crate) const MOVE_TO_POSITION: &str = "minecraft:memory_behavior.move_to_position";
/// `memory_behavior.jump_to_block.<name>` (a component family, 1.26.50).
pub(crate) const JUMP_TO_BLOCK: &str = "minecraft:memory_behavior.jump_to_block";
/// `memory_behavior.prepare_ram_attack.<name>` (a component family, 1.26.50).
pub(crate) const PREPARE_RAM_ATTACK: &str = "minecraft:memory_behavior.prepare_ram_attack";
/// `memory_behavior.ram_attack.<name>` (a component family, 1.26.50).
pub(crate) const RAM_ATTACK: &str = "minecraft:memory_behavior.ram_attack";

/// Every key bound as a family (a name of 1 to 32 characters from `[A-Za-z0-9_]`): the
/// key needs a `.<name>` suffix, and its row is the bare key. Longest first, so that no family
/// is taken for the prefix of another.
const FAMILIES: [&str; 4] = [
    PREPARE_RAM_ATTACK,
    MOVE_TO_POSITION,
    JUMP_TO_BLOCK,
    RAM_ATTACK,
];

/// What a component key says about the generic memory goal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code, reason = "the component map's key routing uses it")]
pub(crate) enum GenericGoal<'a> {
    /// Not `minecraft:memory_behavior.move_to_position` in any form.
    No,
    /// A family goal with its suffix: the row is the family's bare key.
    Named {
        /// The family's bare key (the row).
        base: &'static str,
        /// The suffix.
        suffix: &'a str,
    },
    /// The bare key: `GenericGoalSuffixMissing` from 1.26.20.
    MissingSuffix,
    /// A suffix that is empty, longer than 32 characters or not `[A-Za-z0-9_]`.
    BadSuffix(&'a str),
}

/// Splits `key` into the generic memory goal and its suffix.
// The suffix is `[A-Za-z0-9_]{1,32}`, the schema's pattern key
// `^minecraft:memory_behavior.move_to_position\.[A-Za-z0-9_]{1,32}$`.
#[allow(dead_code, reason = "the component map's key routing uses it")]
pub(crate) fn generic_goal(key: &str) -> GenericGoal<'_> {
    let Some((base, rest)) = FAMILIES
        .iter()
        .find_map(|base| key.strip_prefix(base).map(|rest| (*base, rest)))
        .filter(|(_, rest)| rest.is_empty() || rest.starts_with('.'))
    else {
        return GenericGoal::No;
    };
    let Some(suffix) = rest.strip_prefix('.') else {
        return GenericGoal::MissingSuffix;
    };
    let valid = (1..=32).contains(&suffix.len())
        && suffix
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_');
    if valid {
        GenericGoal::Named { base, suffix }
    } else {
        GenericGoal::BadSuffix(suffix)
    }
}

impl GenericGoal<'_> {
    /// The finding the key gets at `at`: [`suffix_missing`] for a missing or a bad suffix.
    #[allow(dead_code, reason = "the component map's key routing uses it")]
    pub(crate) fn finding(self, at: JsonPath) -> Option<Diagnostic> {
        matches!(self, Self::MissingSuffix | Self::BadSuffix(_)).then(|| suffix_missing(at))
    }
}

/// The finding for a generic goal key without a valid suffix ([`GenericGoal::MissingSuffix`]
/// or [`GenericGoal::BadSuffix`]), at `at`: an information from 1.26.20; the component is
/// dropped and the entity loads.
///
/// A suffix outside `[A-Za-z0-9_]{1,32}` matches no key of the schema either and gets the same
/// finding. The game writes no content-log line for this, so the crate reports it as an
/// information.
pub(crate) fn suffix_missing(at: JsonPath) -> Diagnostic {
    Diagnostic::info(DiagnosticCode::GenericGoalSuffixMissing).at(at)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::diagnostic::Severity;

    /// `v06_generic_goal_suffix`: a missing suffix and a 33-character suffix are
    /// refused; a 32-character one is a named goal.
    #[test]
    fn v06_the_generic_goal_needs_a_suffix_of_1_to_32_word_characters() {
        assert_eq!(generic_goal(MOVE_TO_POSITION), GenericGoal::MissingSuffix);
        let long = "a".repeat(33);
        let key = format!("{MOVE_TO_POSITION}.{long}");
        assert_eq!(generic_goal(&key), GenericGoal::BadSuffix(&long));
        let ok = "A_9".repeat(10) + "zz";
        let key = format!("{MOVE_TO_POSITION}.{ok}");
        assert_eq!(
            generic_goal(&key),
            GenericGoal::Named {
                base: MOVE_TO_POSITION,
                suffix: &ok
            }
        );
        assert_eq!(
            generic_goal(&format!("{MOVE_TO_POSITION}.")),
            GenericGoal::BadSuffix("")
        );
        assert_eq!(
            generic_goal(&format!("{MOVE_TO_POSITION}.a-b")),
            GenericGoal::BadSuffix("a-b")
        );
        assert_eq!(generic_goal("minecraft:behavior.float"), GenericGoal::No);
        assert_eq!(
            generic_goal("minecraft:memory_behavior.move_to_positions"),
            GenericGoal::No
        );
        let d = suffix_missing(JsonPath::new());
        assert_eq!(d.code, DiagnosticCode::GenericGoalSuffixMissing);
        assert_eq!(d.severity, Severity::Info);
        assert!(!d.kept);
        for key in [
            MOVE_TO_POSITION.to_owned(),
            format!("{MOVE_TO_POSITION}.{long}"),
        ] {
            let finding = generic_goal(&key).finding(JsonPath::new());
            assert_eq!(
                finding.map(|d| (d.code, d.severity, d.kept)),
                Some((
                    DiagnosticCode::GenericGoalSuffixMissing,
                    Severity::Info,
                    false
                )),
                "{key}"
            );
        }
        let named = format!("{MOVE_TO_POSITION}.x");
        assert!(generic_goal(&named).finding(JsonPath::new()).is_none());
    }
}
