//! Everyday goals: work, rest, breeding, play and idle behaviour.

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{StrictBool, StrictFloat, StrictInt, StrictString};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::shared::{BlockDescriptor, EventTrigger, Filter, FloatRange, IntRange};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use facet::Facet;

use super::check::{Bound, Check, Deprecated, Goal, deprecated_text};
use super::common::{ControlFlag, DamageCause, EntityTypes, LegacyConditions, SoundEvent, Vec3};
use super::molang::{push_trigger, visit_blocks, visit_conditions};

/// `minecraft:behavior.breed`: allows this mob to breed with other mobs.
// `behavior.breed`: typed from 1.21.90.
#[doc(alias = "minecraft:behavior.breed")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Breed {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `speed_multiplier`: movement speed multiplier applied to the mob when using this AI Goal. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Breed {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }
}

impl Goal for Breed {
    const TYPED: FormatVersion = known::V1_21_90;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for Breed {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.celebrate`: allows this entity to celebrate surviving a raid by making celebration sounds and jumping.
// `behavior.celebrate`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.celebrate")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Celebrate {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `celebration_sound`: the sound event to trigger during the celebration. Default `celebrate`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub celebration_sound: Option<SoundEvent>,
    /// `duration`: the duration in seconds that the celebration lasts for. Default 3e+01; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub duration: Option<StrictFloat<f32>>,
    /// `jump_interval`: minimum and maximum time between jumping (positive, in seconds). Default 1 to 3.5.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub jump_interval: Option<FloatRange>,
    /// `on_celebration_end_event`: the event to trigger when the goal's duration expires.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_celebration_end_event: Option<EventTrigger>,
    /// `sound_interval`: minimum and maximum time between sound events (positive, in seconds). Default 2 to 7.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sound_interval: Option<FloatRange>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Celebrate {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `celebration_sound`, or its default `celebrate`.
    pub fn celebration_sound(&self) -> &str {
        self.celebration_sound
            .as_ref()
            .map_or("celebrate", SoundEvent::name)
    }

    /// `duration`, or its default 3e+01.
    pub fn duration(&self) -> f32 {
        self.duration.map_or(3.0e1, |v| v.0)
    }

    /// `jump_interval`, or its default 1 to 3.5.
    pub fn jump_interval(&self) -> (f32, f32) {
        self.jump_interval
            .map_or((1.0, 3.5), |range| range.resolve(1.0, 3.5))
    }

    /// `sound_interval`, or its default 2 to 7.
    pub fn sound_interval(&self) -> (f32, f32) {
        self.sound_interval
            .map_or((2.0, 7.0), |range| range.resolve(2.0, 7.0))
    }
}

impl Goal for Celebrate {
    const TYPED: FormatVersion = known::V1_26_10;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.sound("celebration_sound", self.celebration_sound.as_ref());
        c.float("duration", &mut self.duration, Bound::Min(0.0));
        c.range("jump_interval", &mut self.jump_interval);
        c.trigger(
            "on_celebration_end_event",
            self.on_celebration_end_event.as_ref(),
        );
        c.range("sound_interval", &mut self.sound_interval);
    }
}

impl MolangFields for Celebrate {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(
            &mut triggers,
            "on_celebration_end_event",
            self.on_celebration_end_event.as_ref(),
        );
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.celebrate_survive`: allows the player to trade with this mob. When the goal starts, it will stop the mob's navigation.
// `behavior.celebrate_survive`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.celebrate_survive")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct CelebrateSurvive {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `duration`: the duration in seconds that the celebration lasts for. Default 3e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub duration: Option<StrictFloat<f32>>,
    /// `fireworks_interval`: minimum and maximum time between firework (positive, in seconds). Default 1e+01 to 2e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub fireworks_interval: Option<FloatRange>,
    /// `on_celebration_end_event`: the event to trigger when the goal's duration expires.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_celebration_end_event: Option<EventTrigger>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl CelebrateSurvive {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `duration`, or its default 3e+01.
    pub fn duration(&self) -> f32 {
        self.duration.map_or(3.0e1, |v| v.0)
    }

    /// `fireworks_interval`, or its default 1e+01 to 2e+01.
    pub fn fireworks_interval(&self) -> (f32, f32) {
        self.fireworks_interval
            .map_or((1.0e1, 2.0e1), |range| range.resolve(1.0e1, 2.0e1))
    }
}

impl Goal for CelebrateSurvive {
    const TYPED: FormatVersion = known::V1_26_10;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.range("fireworks_interval", &mut self.fireworks_interval);
        c.trigger(
            "on_celebration_end_event",
            self.on_celebration_end_event.as_ref(),
        );
    }
}

impl MolangFields for CelebrateSurvive {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(
            &mut triggers,
            "on_celebration_end_event",
            self.on_celebration_end_event.as_ref(),
        );
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.croak`: allows the entity to croak at a random time interval with configurable conditions.
// `behavior.croak`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.croak")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Croak {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `duration`: random range in seconds after which the croaking stops. Can also be a constant. Default 4.5 to 4.5.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub duration: Option<FloatRange>,
    /// `filters`: conditions for the behavior to start and keep running. The interval between runs only starts after passing the filters.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub filters: Option<Filter>,
    /// `interval`: random range in seconds between runs of this behavior. Can also be a constant. Default 1e+01 to 2e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub interval: Option<FloatRange>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Croak {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `duration`, or its default 4.5 to 4.5.
    pub fn duration(&self) -> (f32, f32) {
        self.duration
            .map_or((4.5, 4.5), |range| range.resolve(4.5, 4.5))
    }

    /// `interval`, or its default 1e+01 to 2e+01.
    pub fn interval(&self) -> (f32, f32) {
        self.interval
            .map_or((1.0e1, 2.0e1), |range| range.resolve(1.0e1, 2.0e1))
    }
}

impl Goal for Croak {
    const TYPED: FormatVersion = known::V1_26_30;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.range("duration", &mut self.duration);
        c.filter("filters", self.filters.as_ref());
        c.range("interval", &mut self.interval);
    }
}

impl MolangFields for Croak {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.dig`: allows this entity to dig into the ground before despawning.
// `behavior.dig`: typed from 1.21.120.
#[doc(alias = "minecraft:behavior.dig")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Dig {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `allow_dig_when_named`: if true, this behavior can run when this entity is named. Otherwise not. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub allow_dig_when_named: Option<StrictBool>,
    /// `digs_in_daylight`: indicates that the actor should start digging when it sees daylight. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub digs_in_daylight: Option<StrictBool>,
    /// `duration`: goal duration in seconds. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub duration: Option<StrictFloat<f32>>,
    /// `idle_time`: the minimum idle time in seconds between the last detected disturbance to the start of digging. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub idle_time: Option<StrictFloat<f32>>,
    /// `on_start`: event(s) to run when the goal starts.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_start: Option<EventTrigger>,
    /// `suspicion_is_disturbance`: if true, finding new suspicious locations count as disturbances that may delay the start of this goal. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub suspicion_is_disturbance: Option<StrictBool>,
    /// `vibration_is_disturbance`: if true, vibrations count as disturbances that may delay the start of this goal. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub vibration_is_disturbance: Option<StrictBool>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Dig {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `allow_dig_when_named`, or its default `false`.
    pub fn allow_dig_when_named(&self) -> bool {
        self.allow_dig_when_named.is_some_and(|v| v.0)
    }

    /// `digs_in_daylight`, or its default `false`.
    pub fn digs_in_daylight(&self) -> bool {
        self.digs_in_daylight.is_some_and(|v| v.0)
    }

    /// `duration`, or its default 0.
    pub fn duration(&self) -> f32 {
        self.duration.map_or(0.0, |v| v.0)
    }

    /// `idle_time`, or its default 0.
    pub fn idle_time(&self) -> f32 {
        self.idle_time.map_or(0.0, |v| v.0)
    }

    /// `suspicion_is_disturbance`, or its default `false`.
    pub fn suspicion_is_disturbance(&self) -> bool {
        self.suspicion_is_disturbance.is_some_and(|v| v.0)
    }

    /// `vibration_is_disturbance`, or its default `false`.
    pub fn vibration_is_disturbance(&self) -> bool {
        self.vibration_is_disturbance.is_some_and(|v| v.0)
    }
}

impl Goal for Dig {
    const TYPED: FormatVersion = known::V1_21_120;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.trigger("on_start", self.on_start.as_ref());
    }
}

impl MolangFields for Dig {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(&mut triggers, "on_start", self.on_start.as_ref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.emerge`: allows this entity to emerge from the ground.
// `behavior.emerge`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.emerge")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Emerge {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `cooldown_time`: time in seconds the mob has to wait before using the goal again. Default 0.5; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_time: Option<StrictFloat<f32>>,
    /// `duration`: goal duration in seconds. Default 5; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub duration: Option<StrictFloat<f32>>,
    /// `on_done`: trigger to be executed when the goal execution is about to end.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_done: Option<EventTrigger>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Emerge {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `cooldown_time`, or its default 0.5.
    pub fn cooldown_time(&self) -> f32 {
        self.cooldown_time.map_or(0.5, |v| v.0)
    }

    /// `duration`, or its default 5.
    pub fn duration(&self) -> f32 {
        self.duration.map_or(5.0, |v| v.0)
    }
}

impl Goal for Emerge {
    const TYPED: FormatVersion = known::V1_26_30;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.float("cooldown_time", &mut self.cooldown_time, Bound::Min(0.0));
        c.float("duration", &mut self.duration, Bound::Min(0.0));
        c.trigger("on_done", self.on_done.as_ref());
    }
}

impl MolangFields for Emerge {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(&mut triggers, "on_done", self.on_done.as_ref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.fertilize_farm_block`: allows the mob to search within an area for a growable crop block. If found, the mob will use any available fertilizer in their inventory on the crop. This goal will not execute if the mob does not have a fertilizer item in its inventory.
// `behavior.fertilize_farm_block`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.fertilize_farm_block")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct FertilizeFarmBlock {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `goal_radius`: distance in blocks within the mob considers it has reached it's target position. Default 1.5; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub goal_radius: Option<StrictFloat<f32>>,
    /// `max_fertilizer_usage`: the maximum number of times the mob will use fertilzer on the target block. Default 1; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_fertilizer_usage: Option<StrictInt<i32>>,
    /// `search_cooldown_max_seconds`: the maximum amount of time in seconds that the goal can take before searching again. The time is chosen between 0 and this number. Default 8; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_cooldown_max_seconds: Option<StrictFloat<f32>>,
    /// `search_count`: the number of randomly selected blocks each tick that the mob will check within its search range and height for a valid block to move to. A value of 0 will have the mob check every block within range in one tick. Default 9; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_count: Option<StrictInt<i32>>,
    /// `search_height`: the height in blocks the mob will search within to find a valid target position. Default 1; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_height: Option<StrictInt<i32>>,
    /// `search_range`: the distance in blocks the mob will search within to find a valid target position. Default 1; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_range: Option<StrictInt<i32>>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this goal. Default 0.5; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl FertilizeFarmBlock {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `goal_radius`, or its default 1.5.
    pub fn goal_radius(&self) -> f32 {
        self.goal_radius.map_or(1.5, |v| v.0)
    }

    /// `max_fertilizer_usage`, or its default 1.
    pub fn max_fertilizer_usage(&self) -> i32 {
        self.max_fertilizer_usage.map_or(1, |v| v.0)
    }

    /// `search_cooldown_max_seconds`, or its default 8.
    pub fn search_cooldown_max_seconds(&self) -> f32 {
        self.search_cooldown_max_seconds.map_or(8.0, |v| v.0)
    }

    /// `search_count`, or its default 9.
    // The goal's own default (the game's schema for this goal), not the base shape's.
    pub fn search_count(&self) -> i32 {
        self.search_count.map_or(9, |v| v.0)
    }

    /// `search_height`, or its default 1.
    pub fn search_height(&self) -> i32 {
        self.search_height.map_or(1, |v| v.0)
    }

    /// `search_range`, or its default 1.
    pub fn search_range(&self) -> i32 {
        self.search_range.map_or(1, |v| v.0)
    }

    /// `speed_multiplier`, or its default 0.5.
    // The goal's own default (the game's schema for this goal), not the base shape's.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(0.5, |v| v.0)
    }
}

impl Goal for FertilizeFarmBlock {
    const TYPED: FormatVersion = known::V1_26_10;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.float("goal_radius", &mut self.goal_radius, Bound::Above(0.0));
        c.int(
            "max_fertilizer_usage",
            &mut self.max_fertilizer_usage,
            Bound::Above(0),
        );
        c.float(
            "search_cooldown_max_seconds",
            &mut self.search_cooldown_max_seconds,
            Bound::Above(0.0),
        );
        c.int("search_count", &mut self.search_count, Bound::Min(0));
        c.int("search_height", &mut self.search_height, Bound::Above(0));
        c.int("search_range", &mut self.search_range, Bound::Above(0));
        c.float(
            "speed_multiplier",
            &mut self.speed_multiplier,
            Bound::Min(0.0),
        );
    }
}

impl MolangFields for FertilizeFarmBlock {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.harvest_farm_block`: allows the entity to search within an area for farmland with air above it. If found, the entity will replace the air block by planting a seed item from its inventory on the farmland block. This goal requires `minecraft:inventory` and `minecraft:navigation` to execute. This goal will not execute if the entity does not have an item in its inventory.
// `behavior.harvest_farm_block`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.harvest_farm_block")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct HarvestFarmBlock {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `goal_radius`: distance in blocks within the entity considers it has reached it's target position. Default 1.5; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub goal_radius: Option<StrictFloat<f32>>,
    /// `max_seconds_before_search`: the maximum amount of time in seconds that the goal can take before searching for the first harvest block. The time is chosen between 0 and this number. Default 1; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_seconds_before_search: Option<StrictFloat<f32>>,
    /// `search_cooldown_max_seconds`: the maximum amount of time in seconds that the goal can take before searching again, after failing to find a a harvest block already. The time is chosen between 0 and this number. Default 8; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_cooldown_max_seconds: Option<StrictFloat<f32>>,
    /// `search_count`: the number of randomly selected blocks each tick that the mob will check within its search range and height for a valid block to move to. A value of 0 will have the mob check every block within range in one tick. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_count: Option<StrictInt<i32>>,
    /// `search_height`: the height in blocks the entity will search within to find a valid target position. Default 1; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_height: Option<StrictInt<i32>>,
    /// `search_range`: the distance in blocks the entity will search within to find a valid target position. Default 16; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_range: Option<StrictInt<i32>>,
    /// `seconds_until_new_task`: the amount of time in seconds that the goal will cooldown after a successful reap/sow, before it can start again. Default 0.5; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub seconds_until_new_task: Option<StrictFloat<f32>>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this goal. Default 0.5; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl HarvestFarmBlock {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `goal_radius`, or its default 1.5.
    pub fn goal_radius(&self) -> f32 {
        self.goal_radius.map_or(1.5, |v| v.0)
    }

    /// `max_seconds_before_search`, or its default 1.
    pub fn max_seconds_before_search(&self) -> f32 {
        self.max_seconds_before_search.map_or(1.0, |v| v.0)
    }

    /// `search_cooldown_max_seconds`, or its default 8.
    pub fn search_cooldown_max_seconds(&self) -> f32 {
        self.search_cooldown_max_seconds.map_or(8.0, |v| v.0)
    }

    /// `search_count`, or its default 0.
    // The goal's own default (the game's schema for this goal), not the base shape's.
    pub fn search_count(&self) -> i32 {
        self.search_count.map_or(0, |v| v.0)
    }

    /// `search_height`, or its default 1.
    pub fn search_height(&self) -> i32 {
        self.search_height.map_or(1, |v| v.0)
    }

    /// `search_range`, or its default 16.
    pub fn search_range(&self) -> i32 {
        self.search_range.map_or(16, |v| v.0)
    }

    /// `seconds_until_new_task`, or its default 0.5.
    pub fn seconds_until_new_task(&self) -> f32 {
        self.seconds_until_new_task.map_or(0.5, |v| v.0)
    }

    /// `speed_multiplier`, or its default 0.5.
    // The goal's own default (the game's schema for this goal), not the base shape's.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(0.5, |v| v.0)
    }
}

impl Goal for HarvestFarmBlock {
    const TYPED: FormatVersion = known::V1_26_10;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.float("goal_radius", &mut self.goal_radius, Bound::Above(0.0));
        c.float(
            "max_seconds_before_search",
            &mut self.max_seconds_before_search,
            Bound::Above(0.0),
        );
        c.float(
            "search_cooldown_max_seconds",
            &mut self.search_cooldown_max_seconds,
            Bound::Above(0.0),
        );
        c.int("search_count", &mut self.search_count, Bound::Min(0));
        c.int("search_height", &mut self.search_height, Bound::Above(0));
        c.int("search_range", &mut self.search_range, Bound::Above(0));
        c.float(
            "seconds_until_new_task",
            &mut self.seconds_until_new_task,
            Bound::Above(0.0),
        );
        c.float(
            "speed_multiplier",
            &mut self.speed_multiplier,
            Bound::Min(0.0),
        );
    }
}

impl MolangFields for HarvestFarmBlock {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.hold_ground`: the mob freezes and looks at the mob they are targeting.
// `behavior.hold_ground`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.hold_ground")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct HoldGround {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `broadcast`: whether to broadcast out the mob's target to other mobs of the same type. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub broadcast: Option<StrictBool>,
    /// `broadcast_range`: range in blocks for how far to broadcast. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub broadcast_range: Option<StrictFloat<f32>>,
    /// `min_radius`: minimum distance the target must be for the mob to run this goal. Default 1e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_radius: Option<StrictFloat<f32>>,
    /// `within_radius_event`: event to run when target is within the radius. This event is broadcasted if broadcast is true.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub within_radius_event: Option<EventTrigger>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl HoldGround {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `broadcast`, or its default `false`.
    pub fn broadcast(&self) -> bool {
        self.broadcast.is_some_and(|v| v.0)
    }

    /// `broadcast_range`, or its default 0.
    pub fn broadcast_range(&self) -> f32 {
        self.broadcast_range.map_or(0.0, |v| v.0)
    }

    /// `min_radius`, or its default 1e+01.
    pub fn min_radius(&self) -> f32 {
        self.min_radius.map_or(1.0e1, |v| v.0)
    }
}

impl Goal for HoldGround {
    const TYPED: FormatVersion = known::V1_26_20;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.trigger("within_radius_event", self.within_radius_event.as_ref());
    }
}

impl MolangFields for HoldGround {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(
            &mut triggers,
            "within_radius_event",
            self.within_radius_event.as_ref(),
        );
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.inspect_bookshelf`: allows the mob to inspect bookshelves.
// `behavior.inspect_bookshelf`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.inspect_bookshelf")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct InspectBookshelf {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `goal_radius`: distance in blocks within the mob considers it has reached the goal. This is the "wiggle room" to stop the AI from bouncing back and forth trying to reach a specific spot. Default 0.5; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub goal_radius: Option<StrictFloat<f32>>,
    /// `search_count`: the number of randomly selected blocks each tick that the mob will check within its search range and height for a valid block to move to. A value of 0 will have the mob check every block within range in one tick. Default 10; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_count: Option<StrictInt<i32>>,
    /// `search_height`: the height that the mob will search for bookshelves. Default 1; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_height: Option<StrictInt<i32>>,
    /// `search_range`: the distance in blocks the mob will look for books to inspect. Default 0; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_range: Option<StrictInt<i32>>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this goal. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl InspectBookshelf {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `goal_radius`, or its default 0.5.
    pub fn goal_radius(&self) -> f32 {
        self.goal_radius.map_or(0.5, |v| v.0)
    }

    /// `search_count`, or its default 10.
    pub fn search_count(&self) -> i32 {
        self.search_count.map_or(10, |v| v.0)
    }

    /// `search_height`, or its default 1.
    pub fn search_height(&self) -> i32 {
        self.search_height.map_or(1, |v| v.0)
    }

    /// `search_range`, or its default 0.
    pub fn search_range(&self) -> i32 {
        self.search_range.map_or(0, |v| v.0)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }
}

impl Goal for InspectBookshelf {
    const TYPED: FormatVersion = known::V1_26_10;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.float("goal_radius", &mut self.goal_radius, Bound::Above(0.0));
        c.int("search_count", &mut self.search_count, Bound::Min(0));
        c.int("search_height", &mut self.search_height, Bound::Above(0));
        c.int("search_range", &mut self.search_range, Bound::Above(0));
        c.float(
            "speed_multiplier",
            &mut self.speed_multiplier,
            Bound::Min(0.0),
        );
    }
}

impl MolangFields for InspectBookshelf {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.investigate_suspicious_location`: allows this entity to move towards a `suspicious` position based on data gathered in `minecraft:suspect_tracking`.
// `behavior.investigate_suspicious_location`: typed from 1.21.130.
#[doc(alias = "minecraft:behavior.investigate_suspicious_location")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct InvestigateSuspiciousLocation {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `goal_radius`: distance in blocks within the entity considers it has reached it's target position. Default 1.5.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub goal_radius: Option<StrictFloat<f32>>,
    /// `speed_multiplier`: movement speed multiplier. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl InvestigateSuspiciousLocation {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `goal_radius`, or its default 1.5.
    pub fn goal_radius(&self) -> f32 {
        self.goal_radius.map_or(1.5, |v| v.0)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }
}

impl Goal for InvestigateSuspiciousLocation {
    const TYPED: FormatVersion = known::V1_21_130;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for InvestigateSuspiciousLocation {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.lay_down`: allows mobs to lay down at times.
// `behavior.lay_down`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.lay_down")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct LayDown {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `interval`: a random value to determine at what intervals something can occur. This has a 1/interval chance to choose this goal. Default 120.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub interval: Option<StrictInt<i32>>,
    /// `random_stop_interval`: a random value in which the goal can use to pull out of the behavior. This is a 1/interval chance to stop laying down. Default 120.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub random_stop_interval: Option<StrictInt<i32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl LayDown {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `interval`, or its default 120.
    pub fn interval(&self) -> i32 {
        self.interval.map_or(120, |v| v.0)
    }

    /// `random_stop_interval`, or its default 120.
    pub fn random_stop_interval(&self) -> i32 {
        self.random_stop_interval.map_or(120, |v| v.0)
    }
}

impl Goal for LayDown {
    const TYPED: FormatVersion = known::V1_26_20;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for LayDown {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.lay_egg`: allows the mob to lay an egg block on certain types of blocks if the mob is pregnant.
///
/// Deprecated members, reported and dropped: `search_count`.
// `behavior.lay_egg`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.lay_egg")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct LayEgg {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `allow_laying_from_below`: allows the mob to lay its eggs from below the target if it can't get there. This is useful if the target block is water with air above, since mobs may not be able to get to the air block above water. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub allow_laying_from_below: Option<StrictBool>,
    /// `egg_type`: block type for the egg to lay. If this is a turtle egg, the number of eggs in the block is randomly set. Default `minecraft:turtle_egg`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub egg_type: Option<StrictString>,
    /// `goal_radius`: distance in blocks within the mob considers it has reached the goal. This is the "wiggle room" to stop the AI from bouncing back and forth trying to reach a specific spot. Default 0.5; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub goal_radius: Option<StrictFloat<f32>>,
    /// `lay_egg_sound`: name of the sound event played when laying the egg. Defaults to `lay_egg`, which is used for Turtles. Default `lay_egg`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub lay_egg_sound: Option<SoundEvent>,
    /// `lay_seconds`: duration of the laying egg process in seconds. Default 1e+01; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub lay_seconds: Option<StrictFloat<f32>>,
    /// `on_lay`: event to run when this mob lays the egg.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_lay: Option<EventTrigger>,
    /// `search_height`: the height in blocks the mob will look for the block to move towards. Default 1; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_height: Option<StrictInt<i32>>,
    /// `search_range`: the distance in blocks it will look for the block to move towards. Default 0; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_range: Option<StrictInt<i32>>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this goal. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// `target_blocks`: blocks that the mob can lay its eggs on top of.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub target_blocks: Option<Vec<StrictString>>,
    /// `target_materials_above_block`: types of materials that can exist above the target block. Valid types are Air, Water, and Lava.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub target_materials_above_block: Option<Vec<StrictString>>,
    /// `use_default_animation`: specifies if the default lay-egg animation should be played when the egg is placed or not. Default `true`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub use_default_animation: Option<StrictBool>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl LayEgg {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `allow_laying_from_below`, or its default `false`.
    pub fn allow_laying_from_below(&self) -> bool {
        self.allow_laying_from_below.is_some_and(|v| v.0)
    }

    /// `egg_type`, or its default `minecraft:turtle_egg`.
    pub fn egg_type(&self) -> &str {
        self.egg_type
            .as_ref()
            .map_or("minecraft:turtle_egg", |v| v.0.as_str())
    }

    /// `goal_radius`, or its default 0.5.
    pub fn goal_radius(&self) -> f32 {
        self.goal_radius.map_or(0.5, |v| v.0)
    }

    /// `lay_egg_sound`, or its default `lay_egg`.
    pub fn lay_egg_sound(&self) -> &str {
        self.lay_egg_sound
            .as_ref()
            .map_or("lay_egg", SoundEvent::name)
    }

    /// `lay_seconds`, or its default 1e+01.
    pub fn lay_seconds(&self) -> f32 {
        self.lay_seconds.map_or(1.0e1, |v| v.0)
    }

    /// `search_height`, or its default 1.
    pub fn search_height(&self) -> i32 {
        self.search_height.map_or(1, |v| v.0)
    }

    /// `search_range`, or its default 0.
    pub fn search_range(&self) -> i32 {
        self.search_range.map_or(0, |v| v.0)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }

    /// `use_default_animation`, or its default `true`.
    pub fn use_default_animation(&self) -> bool {
        self.use_default_animation.is_none_or(|v| v.0)
    }
}

impl Goal for LayEgg {
    const TYPED: FormatVersion = known::V1_26_10;

    const DEPRECATED: &'static [Deprecated] = &[(
        "search_count",
        deprecated_text!("search_count"),
        known::V1_26_10,
    )];

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        if c.lenient() {
            // The game's legacy goal reader looks each name up in its material table (`Air`,
            // `Water`, `Lava`, `Any`, case-sensitive); a miss logs this at the warning level and
            // reads as Air. The name is kept as written.
            for (i, name) in self
                .target_materials_above_block
                .iter()
                .flatten()
                .enumerate()
            {
                if !matches!(name.0.as_str(), "Air" | "Water" | "Lava" | "Any") {
                    let mut at = c.path("target_materials_above_block");
                    at.push_index(i);
                    c.legacy_warning_at(at, "Could not find specified material. Using Air.");
                }
            }
        }
        c.float("goal_radius", &mut self.goal_radius, Bound::Above(0.0));
        c.sound("lay_egg_sound", self.lay_egg_sound.as_ref());
        c.float("lay_seconds", &mut self.lay_seconds, Bound::Min(0.0));
        c.trigger("on_lay", self.on_lay.as_ref());
        c.int("search_height", &mut self.search_height, Bound::Above(0));
        c.int("search_range", &mut self.search_range, Bound::Above(0));
        c.float(
            "speed_multiplier",
            &mut self.speed_multiplier,
            Bound::Min(0.0),
        );
    }
}

impl MolangFields for LayEgg {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(&mut triggers, "on_lay", self.on_lay.as_ref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.make_love`: allows the villager to look for a mate to spawn other villagers with. Can only be used by Villagers.
// `behavior.make_love`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.make_love")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct MakeLove {
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

impl MakeLove {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }
}

impl Goal for MakeLove {
    const TYPED: FormatVersion = known::V1_26_30;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for MakeLove {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.mingle`: allows an entity to go to the village bell and mingle with other entities.
///
/// Deprecated members, reported and dropped: `poi_type`.
// `behavior.mingle`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.mingle")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Mingle {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `cooldown_time`: time in seconds the mob has to wait before using the goal again. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_time: Option<StrictFloat<f32>>,
    /// `duration`: amount of time in seconds that the entity will chat with another entity. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub duration: Option<StrictFloat<f32>>,
    /// `mingle_distance`: the distance from its partner that this entity will mingle. If the entity type is not the same as the entity, this value needs to be identical on both entities. Default 2; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub mingle_distance: Option<StrictFloat<f32>>,
    /// `mingle_partner_type`: the entity type that this entity is allowed to mingle with. Default `""`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub mingle_partner_type: Option<StrictString>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this goal. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Mingle {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `cooldown_time`, or its default 0.
    pub fn cooldown_time(&self) -> f32 {
        self.cooldown_time.map_or(0.0, |v| v.0)
    }

    /// `duration`, or its default 1.
    pub fn duration(&self) -> f32 {
        self.duration.map_or(1.0, |v| v.0)
    }

    /// `mingle_distance`, or its default 2.
    pub fn mingle_distance(&self) -> f32 {
        self.mingle_distance.map_or(2.0, |v| v.0)
    }

    /// `mingle_partner_type`, or its default `""`.
    pub fn mingle_partner_type(&self) -> &str {
        self.mingle_partner_type
            .as_ref()
            .map_or("", |v| v.0.as_str())
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }
}

impl Goal for Mingle {
    const TYPED: FormatVersion = known::V1_26_20;

    const DEPRECATED: &'static [Deprecated] =
        &[("poi_type", deprecated_text!("poi_type"), known::V1_26_20)];

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.float("cooldown_time", &mut self.cooldown_time, Bound::Min(0.0));
        c.float("duration", &mut self.duration, Bound::Min(0.0));
        c.float(
            "mingle_distance",
            &mut self.mingle_distance,
            Bound::Min(0.0),
        );
        c.float(
            "speed_multiplier",
            &mut self.speed_multiplier,
            Bound::Min(0.0),
        );
    }
}

impl MolangFields for Mingle {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.nap`: allows mobs to occassionally stop and take a nap under certain conditions.
// `behavior.nap`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.nap")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Nap {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `can_nap_filters`: conditions that need to be met for the entity to nap.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub can_nap_filters: Option<Filter>,
    /// `cooldown_max`: maximum time in seconds the mob has to wait before using the goal again. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_max: Option<StrictFloat<f32>>,
    /// `cooldown_min`: minimum time in seconds the mob has to wait before using the goal again. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_min: Option<StrictFloat<f32>>,
    /// `mob_detect_dist`: the block distance in x and z that will be checked for mobs that this mob detects. Default 6.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub mob_detect_dist: Option<StrictFloat<f32>>,
    /// `mob_detect_height`: the block distance in y that will be checked for mobs that this mob detects. Default 6.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub mob_detect_height: Option<StrictFloat<f32>>,
    /// `wake_mob_exceptions`: filters for mobs that will not wake this entity from napping.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub wake_mob_exceptions: Option<Filter>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Nap {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `cooldown_max`, or its default 0.
    pub fn cooldown_max(&self) -> f32 {
        self.cooldown_max.map_or(0.0, |v| v.0)
    }

    /// `cooldown_min`, or its default 0.
    pub fn cooldown_min(&self) -> f32 {
        self.cooldown_min.map_or(0.0, |v| v.0)
    }

    /// `mob_detect_dist`, or its default 6.
    pub fn mob_detect_dist(&self) -> f32 {
        self.mob_detect_dist.map_or(6.0, |v| v.0)
    }

    /// `mob_detect_height`, or its default 6.
    pub fn mob_detect_height(&self) -> f32 {
        self.mob_detect_height.map_or(6.0, |v| v.0)
    }
}

impl Goal for Nap {
    const TYPED: FormatVersion = known::V1_26_40;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.filter("can_nap_filters", self.can_nap_filters.as_ref());
        c.filter("wake_mob_exceptions", self.wake_mob_exceptions.as_ref());
    }
}

impl MolangFields for Nap {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.pet_sleep_with_owner`: allows the pet mob to move onto a bed with its owner while sleeping.
// `behavior.pet_sleep_with_owner`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.pet_sleep_with_owner")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct PetSleepWithOwner {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `goal_radius`: distance in blocks within the mob considers it has reached the goal. This is the "wiggle room" to stop the AI from bouncing back and forth trying to reach a specific spot. Default 0.5.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub goal_radius: Option<StrictFloat<f32>>,
    /// `search_height`: height in blocks from the owner the pet can be to sleep with owner. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_height: Option<StrictInt<i32>>,
    /// `search_range`: the distance in blocks from the owner the pet can be to sleep with owner. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_range: Option<StrictInt<i32>>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl PetSleepWithOwner {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `goal_radius`, or its default 0.5.
    pub fn goal_radius(&self) -> f32 {
        self.goal_radius.map_or(0.5, |v| v.0)
    }

    /// `search_height`, or its default 1.
    pub fn search_height(&self) -> i32 {
        self.search_height.map_or(1, |v| v.0)
    }

    /// `search_range`, or its default 0.
    pub fn search_range(&self) -> i32 {
        self.search_range.map_or(0, |v| v.0)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }
}

impl Goal for PetSleepWithOwner {
    const TYPED: FormatVersion = known::V1_26_20;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for PetSleepWithOwner {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.play`: allows the mob to offer a flower to another mob with the `minecraft:take_flower` behavior.
// `behavior.play`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.play")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Play {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `chance_to_start`: percent chance that the mob will start this goal, from 0 to 1. Default 0.0025; 0 to 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub chance_to_start: Option<StrictFloat<f32>>,
    /// `follow_distance`: the distance (in blocks) that the mob tries to be in range of the friend it's following. Default 2; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub follow_distance: Option<StrictInt<i32>>,
    /// `friend_search_area`: the dimensions of the AABB used to search for a potential friend to play with. Default `[6, 3, 6]`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub friend_search_area: Option<Vec3>,
    /// `friend_types`: the entity type(s) to consider when searching for a potential friend to play with.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub friend_types: Option<EntityTypes>,
    /// `max_play_duration_seconds`: the max amount of seconds that the mob will play for before exiting the Goal. Default 5e+01; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_play_duration_seconds: Option<StrictFloat<f32>>,
    /// `random_pos_search_height`: the height (in blocks) that the mob will search within to find a random position position to move to. Must be at least 1. Default 3; at least 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub random_pos_search_height: Option<StrictInt<i32>>,
    /// `random_pos_search_range`: the distance (in blocks) on ground that the mob will search within to find a random position to move to. Must be at least 1. Default 16; at least 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub random_pos_search_range: Option<StrictInt<i32>>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Play {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `chance_to_start`, or its default 0.0025.
    pub fn chance_to_start(&self) -> f32 {
        self.chance_to_start.map_or(0.0025, |v| v.0)
    }

    /// `follow_distance`, or its default 2.
    pub fn follow_distance(&self) -> i32 {
        self.follow_distance.map_or(2, |v| v.0)
    }

    /// `friend_search_area`, or its default `[6, 3, 6]`.
    pub fn friend_search_area(&self) -> [f32; 3] {
        self.friend_search_area
            .as_ref()
            .map_or([6.0, 3.0, 6.0], |v| [v.x, v.y, v.z])
    }

    /// `max_play_duration_seconds`, or its default 5e+01.
    pub fn max_play_duration_seconds(&self) -> f32 {
        self.max_play_duration_seconds.map_or(5.0e1, |v| v.0)
    }

    /// `random_pos_search_height`, or its default 3.
    pub fn random_pos_search_height(&self) -> i32 {
        self.random_pos_search_height.map_or(3, |v| v.0)
    }

    /// `random_pos_search_range`, or its default 16.
    pub fn random_pos_search_range(&self) -> i32 {
        self.random_pos_search_range.map_or(16, |v| v.0)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }
}

impl Goal for Play {
    const TYPED: FormatVersion = known::V1_26_10;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.float(
            "chance_to_start",
            &mut self.chance_to_start,
            Bound::MinMax(0.0, 1.0),
        );
        c.int("follow_distance", &mut self.follow_distance, Bound::Min(0));
        c.vector(
            "friend_search_area",
            self.friend_search_area.as_ref().map(|v| &v.extra),
        );
        c.entity_types("friend_types", self.friend_types.as_ref());
        c.float(
            "max_play_duration_seconds",
            &mut self.max_play_duration_seconds,
            Bound::Min(0.0),
        );
        c.int(
            "random_pos_search_height",
            &mut self.random_pos_search_height,
            Bound::Min(1),
        );
        c.int(
            "random_pos_search_range",
            &mut self.random_pos_search_range,
            Bound::Min(1),
        );
        c.float(
            "speed_multiplier",
            &mut self.speed_multiplier,
            Bound::Min(0.0),
        );
    }
}

impl MolangFields for Play {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.play_dead`: allows this entity to pretend to be dead to avoid being targeted by attackers.
// `behavior.play_dead`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.play_dead")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct PlayDead {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `apply_regeneration`: whether the mob will receive the regeneration effect while playing dead. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub apply_regeneration: Option<StrictBool>,
    /// `damage_sources`: the list of Entity Damage Sources that will cause this mob to play dead.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub damage_sources: Option<Vec<DamageCause>>,
    /// `duration`: the amount of time the mob will remain playing dead (in seconds). Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub duration: Option<StrictFloat<f32>>,
    /// `filters`: the list of other triggers that are required for the mob to activate play dead.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub filters: Option<Filter>,
    /// `force_below_health`: the amount of health at which damage will cause the mob to play dead. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub force_below_health: Option<StrictInt<i32>>,
    /// `random_damage_range`: the range of damage that may cause the goal to start depending on randomness. Damage taken below the min will never cause the goal to start. Damage taken above the max will always cause the goal to start. Default 0 to 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub random_damage_range: Option<IntRange>,
    /// `random_start_chance`: the likelihood of this goal starting upon taking damage. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub random_start_chance: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl PlayDead {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `apply_regeneration`, or its default `false`.
    pub fn apply_regeneration(&self) -> bool {
        self.apply_regeneration.is_some_and(|v| v.0)
    }

    /// `duration`, or its default 1.
    pub fn duration(&self) -> f32 {
        self.duration.map_or(1.0, |v| v.0)
    }

    /// `force_below_health`, or its default 0.
    pub fn force_below_health(&self) -> i32 {
        self.force_below_health.map_or(0, |v| v.0)
    }

    /// `random_damage_range`, or its default 0 to 0.
    pub fn random_damage_range(&self) -> (i32, i32) {
        self.random_damage_range
            .map_or((0, 0), |range| range.resolve(0, 0))
    }

    /// `random_start_chance`, or its default 1.
    pub fn random_start_chance(&self) -> f32 {
        self.random_start_chance.map_or(1.0, |v| v.0)
    }
}

impl Goal for PlayDead {
    const TYPED: FormatVersion = known::V1_26_10;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.filter("filters", self.filters.as_ref());
        c.range("random_damage_range", &mut self.random_damage_range);
    }
}

impl MolangFields for PlayDead {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.raid_garden`: allows the mob to eat/raid crops out of farms until they are full.
///
/// Deprecated members, reported and dropped: `search_count`.
// `behavior.raid_garden`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.raid_garden")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct RaidGarden {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `blocks`: blocks that the mob is looking for to eat/raid.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub blocks: Option<Vec<StrictString>>,
    /// `eat_delay`: time in seconds between each time it eats/raids. Default 2; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub eat_delay: Option<StrictInt<i32>>,
    /// `full_delay`: amount of time in seconds before this mob wants to eat/raid again after eating its maximum. Default 100; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub full_delay: Option<StrictInt<i32>>,
    /// `goal_radius`: distance in blocks within the mob considers it has reached the goal. This is the "wiggle room" to stop the AI from bouncing back and forth trying to reach a specific spot. Default 0.5; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub goal_radius: Option<StrictFloat<f32>>,
    /// `initial_eat_delay`: time in seconds before starting to eat/raid once it arrives at it. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub initial_eat_delay: Option<StrictInt<i32>>,
    /// `max_to_eat`: maximum number of crops this entity wants to eat/raid. If set to zero or less then it doesn't have a maximum. Default 6.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_to_eat: Option<StrictInt<i32>>,
    /// `search_height`: height in blocks the mob will look for crops to eat. Default 1; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_height: Option<StrictInt<i32>>,
    /// `search_range`: distance in blocks the mob will look for crops to eat. Default 0; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_range: Option<StrictInt<i32>>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this goal. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl RaidGarden {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `eat_delay`, or its default 2.
    pub fn eat_delay(&self) -> i32 {
        self.eat_delay.map_or(2, |v| v.0)
    }

    /// `full_delay`, or its default 100.
    pub fn full_delay(&self) -> i32 {
        self.full_delay.map_or(100, |v| v.0)
    }

    /// `goal_radius`, or its default 0.5.
    pub fn goal_radius(&self) -> f32 {
        self.goal_radius.map_or(0.5, |v| v.0)
    }

    /// `initial_eat_delay`, or its default 0.
    pub fn initial_eat_delay(&self) -> i32 {
        self.initial_eat_delay.map_or(0, |v| v.0)
    }

    /// `max_to_eat`, or its default 6.
    pub fn max_to_eat(&self) -> i32 {
        self.max_to_eat.map_or(6, |v| v.0)
    }

    /// `search_height`, or its default 1.
    pub fn search_height(&self) -> i32 {
        self.search_height.map_or(1, |v| v.0)
    }

    /// `search_range`, or its default 0.
    pub fn search_range(&self) -> i32 {
        self.search_range.map_or(0, |v| v.0)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }
}

impl Goal for RaidGarden {
    const TYPED: FormatVersion = known::V1_26_10;

    const DEPRECATED: &'static [Deprecated] = &[(
        "search_count",
        deprecated_text!("search_count"),
        known::V1_26_10,
    )];

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.int("eat_delay", &mut self.eat_delay, Bound::Min(0));
        c.int("full_delay", &mut self.full_delay, Bound::Min(0));
        c.float("goal_radius", &mut self.goal_radius, Bound::Above(0.0));
        c.int(
            "initial_eat_delay",
            &mut self.initial_eat_delay,
            Bound::Min(0),
        );
        c.int("search_height", &mut self.search_height, Bound::Above(0));
        c.int("search_range", &mut self.search_range, Bound::Above(0));
        c.float(
            "speed_multiplier",
            &mut self.speed_multiplier,
            Bound::Min(0.0),
        );
    }
}

impl MolangFields for RaidGarden {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.random_search_and_dig`: allows this entity to locate a random target block that it can path find to. Once found, the entity will move towards it and dig up an item. \[Default target block types: Dirt, Grass, Podzol, `DirtWithRoots`, `MossBlock`, Mud, `MuddyMangroveRoots`\].
// `behavior.random_search_and_dig`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.random_search_and_dig")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct RandomSearchAndDig {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `cooldown_range`: goal cooldown range in seconds. Default 0 to 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_range: Option<FloatRange>,
    /// `digging_duration_range`: digging duration in seconds. Default 8 to 1e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub digging_duration_range: Option<FloatRange>,
    /// `find_valid_position_retries`: amount of retries to find a valid target position within search range. Default 5.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub find_valid_position_retries: Option<StrictInt<i32>>,
    /// `goal_radius`: distance in blocks within the entity to considers it has reached it's target position. Default 1.5.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub goal_radius: Option<StrictFloat<f32>>,
    /// `item_table`: file path relative to the resource pack root for items to spawn list (loot table format). Default `""`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub item_table: Option<StrictString>,
    /// `on_digging_start`: event to run when the goal ends searching has begins digging.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_digging_start: Option<EventTrigger>,
    /// `on_fail_during_digging`: event to run when the goal failed while in digging state.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_fail_during_digging: Option<EventTrigger>,
    /// `on_fail_during_searching`: event to run when the goal failed while in searching state.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_fail_during_searching: Option<EventTrigger>,
    /// `on_item_found`: event to run when the goal find a item.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_item_found: Option<EventTrigger>,
    /// `on_searching_start`: event to run when the goal starts searching.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_searching_start: Option<EventTrigger>,
    /// `on_success`: event to run when searching and digging has ended.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_success: Option<EventTrigger>,
    /// `search_range_xz`: width and length of the volume around the entity used to find a valid target position. Default 10.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_range_xz: Option<StrictInt<i32>>,
    /// `search_range_y`: height of the volume around the entity used to find a valid target position. Default 7.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_range_y: Option<StrictInt<i32>>,
    /// `spawn_item_after_seconds`: digging duration before spawning item in seconds. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub spawn_item_after_seconds: Option<StrictFloat<f32>>,
    /// `spawn_item_pos_offset`: distance to offset the item's spawn location in the direction the mob is facing. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub spawn_item_pos_offset: Option<StrictFloat<f32>>,
    /// `speed_multiplier`: searching movement speed multiplier. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// `target_blocks`: list of target block types the goal will look to dig on. Overrides the default list.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub target_blocks: Option<Vec<BlockDescriptor>>,
    /// `target_dig_position_offset`: dig target position offset from the feet position of the mob in their facing direction. Default 2.25.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub target_dig_position_offset: Option<StrictFloat<f32>>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl RandomSearchAndDig {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `cooldown_range`, or its default 0 to 0.
    pub fn cooldown_range(&self) -> (f32, f32) {
        self.cooldown_range
            .map_or((0.0, 0.0), |range| range.resolve(0.0, 0.0))
    }

    /// `digging_duration_range`, or its default 8 to 1e+01.
    pub fn digging_duration_range(&self) -> (f32, f32) {
        self.digging_duration_range
            .map_or((8.0, 1.0e1), |range| range.resolve(8.0, 1.0e1))
    }

    /// `find_valid_position_retries`, or its default 5.
    pub fn find_valid_position_retries(&self) -> i32 {
        self.find_valid_position_retries.map_or(5, |v| v.0)
    }

    /// `goal_radius`, or its default 1.5.
    pub fn goal_radius(&self) -> f32 {
        self.goal_radius.map_or(1.5, |v| v.0)
    }

    /// `item_table`, or its default `""`.
    pub fn item_table(&self) -> &str {
        self.item_table.as_ref().map_or("", |v| v.0.as_str())
    }

    /// `search_range_xz`, or its default 10.
    pub fn search_range_xz(&self) -> i32 {
        self.search_range_xz.map_or(10, |v| v.0)
    }

    /// `search_range_y`, or its default 7.
    pub fn search_range_y(&self) -> i32 {
        self.search_range_y.map_or(7, |v| v.0)
    }

    /// `spawn_item_after_seconds`, or its default 0.
    pub fn spawn_item_after_seconds(&self) -> f32 {
        self.spawn_item_after_seconds.map_or(0.0, |v| v.0)
    }

    /// `spawn_item_pos_offset`, or its default 0.
    pub fn spawn_item_pos_offset(&self) -> f32 {
        self.spawn_item_pos_offset.map_or(0.0, |v| v.0)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }

    /// `target_dig_position_offset`, or its default 2.25.
    pub fn target_dig_position_offset(&self) -> f32 {
        self.target_dig_position_offset.map_or(2.25, |v| v.0)
    }
}

impl Goal for RandomSearchAndDig {
    const TYPED: FormatVersion = known::V1_26_10;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.range("cooldown_range", &mut self.cooldown_range);
        c.range("digging_duration_range", &mut self.digging_duration_range);
        c.trigger("on_digging_start", self.on_digging_start.as_ref());
        c.trigger(
            "on_fail_during_digging",
            self.on_fail_during_digging.as_ref(),
        );
        c.trigger(
            "on_fail_during_searching",
            self.on_fail_during_searching.as_ref(),
        );
        c.trigger("on_item_found", self.on_item_found.as_ref());
        c.trigger("on_searching_start", self.on_searching_start.as_ref());
        c.trigger("on_success", self.on_success.as_ref());
        c.float(
            "spawn_item_after_seconds",
            &mut self.spawn_item_after_seconds,
            Bound::Min(0.0),
        );
        c.float(
            "spawn_item_pos_offset",
            &mut self.spawn_item_pos_offset,
            Bound::Min(0.0),
        );
        c.float(
            "speed_multiplier",
            &mut self.speed_multiplier,
            Bound::Min(0.0),
        );
        c.blocks("target_blocks", self.target_blocks.as_deref());
    }
}

impl MolangFields for RandomSearchAndDig {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visit_blocks(visitor, "target_blocks", &mut self.target_blocks);
        let mut triggers = Vec::new();
        push_trigger(
            &mut triggers,
            "on_digging_start",
            self.on_digging_start.as_ref(),
        );
        push_trigger(
            &mut triggers,
            "on_fail_during_digging",
            self.on_fail_during_digging.as_ref(),
        );
        push_trigger(
            &mut triggers,
            "on_fail_during_searching",
            self.on_fail_during_searching.as_ref(),
        );
        push_trigger(&mut triggers, "on_item_found", self.on_item_found.as_ref());
        push_trigger(
            &mut triggers,
            "on_searching_start",
            self.on_searching_start.as_ref(),
        );
        push_trigger(&mut triggers, "on_success", self.on_success.as_ref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.random_sitting`: allows the mob to randomly sit for a duration.
// `behavior.random_sitting`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.random_sitting")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct RandomSitting {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `cooldown`: time in seconds the mob has to wait before using the goal again. Default 1e+02.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown: Option<StrictFloat<f32>>,
    /// `min_sit_time`: the minimum amount of time in seconds before the mob will stand back up. Default 1e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_sit_time: Option<StrictFloat<f32>>,
    /// `start_chance`: this is the chance that the mob will start this goal, from 0 to 1. Default 0.1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub start_chance: Option<StrictFloat<f32>>,
    /// `stop_chance`: this is the chance that the mob will stop this goal, from 0 to 1. Default 0.3.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub stop_chance: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl RandomSitting {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `cooldown`, or its default 1e+02.
    pub fn cooldown(&self) -> f32 {
        self.cooldown.map_or(1.0e2, |v| v.0)
    }

    /// `min_sit_time`, or its default 1e+01.
    pub fn min_sit_time(&self) -> f32 {
        self.min_sit_time.map_or(1.0e1, |v| v.0)
    }

    /// `start_chance`, or its default 0.1.
    pub fn start_chance(&self) -> f32 {
        self.start_chance.map_or(0.1, |v| v.0)
    }

    /// `stop_chance`, or its default 0.3.
    pub fn stop_chance(&self) -> f32 {
        self.stop_chance.map_or(0.3, |v| v.0)
    }
}

impl Goal for RandomSitting {
    const TYPED: FormatVersion = known::V1_26_20;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for RandomSitting {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.receive_love`: allows the villager to stop so another villager can breed with it. Can only be used by a Villager.
// `behavior.receive_love`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.receive_love")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct ReceiveLove {
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

impl ReceiveLove {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }
}

impl Goal for ReceiveLove {
    const TYPED: FormatVersion = known::V1_26_30;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for ReceiveLove {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.roar`: allows this entity to roar at another entity based on data in `minecraft:anger_level.` Once the anger threshold specified in `minecraft:anger_level` has been reached, this entity will roar for the specified amount of time, look at the other entity, apply anger boost towards it, and finally target it.
// `behavior.roar`: typed from 1.21.110.
#[doc(alias = "minecraft:behavior.roar")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Roar {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `duration`: the amount of time to roar for. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub duration: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Roar {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `duration`, or its default 0.
    pub fn duration(&self) -> f32 {
        self.duration.map_or(0.0, |v| v.0)
    }
}

impl Goal for Roar {
    const TYPED: FormatVersion = known::V1_21_110;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.float("duration", &mut self.duration, Bound::Min(0.0));
    }
}

impl MolangFields for Roar {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.roll`: this allows the mob to roll forward.
// `behavior.roll`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.roll")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Roll {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `probability`: the probability that the mob will use the goal. Default 0.02.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub probability: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Roll {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `probability`, or its default 0.02.
    pub fn probability(&self) -> f32 {
        self.probability.map_or(0.02, |v| v.0)
    }
}

impl Goal for Roll {
    const TYPED: FormatVersion = known::V1_26_30;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for Roll {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.run_around_like_crazy`: allows the mob to run around aimlessly.
// `behavior.run_around_like_crazy`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.run_around_like_crazy")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct RunAroundLikeCrazy {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl RunAroundLikeCrazy {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }
}

impl Goal for RunAroundLikeCrazy {
    const TYPED: FormatVersion = known::V1_26_30;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for RunAroundLikeCrazy {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.scared`: allows the mob to become scared when the weather outside is thundering.
// `behavior.scared`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.scared")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Scared {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `sound_delay`: the interval in which a sound will play when active in a 1/delay chance to kick off. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sound_delay: Option<StrictInt<i32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Scared {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `sound_delay`, or its default 0.
    pub fn sound_delay(&self) -> i32 {
        self.sound_delay.map_or(0, |v| v.0)
    }
}

impl Goal for Scared {
    const TYPED: FormatVersion = known::V1_26_30;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for Scared {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.silverfish_merge_with_stone`: allows the mob to go into stone blocks like Silverfish do. Currently it can only be used by Silverfish.
// `behavior.silverfish_merge_with_stone`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.silverfish_merge_with_stone")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct SilverfishMergeWithStone {
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

impl SilverfishMergeWithStone {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }
}

impl Goal for SilverfishMergeWithStone {
    const TYPED: FormatVersion = known::V1_26_40;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for SilverfishMergeWithStone {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.silverfish_wake_up_friends`: allows the mob to alert mobs in nearby blocks to come out. Currently it can only be used by Silverfish.
// `behavior.silverfish_wake_up_friends`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.silverfish_wake_up_friends")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct SilverfishWakeUpFriends {
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

impl SilverfishWakeUpFriends {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }
}

impl Goal for SilverfishWakeUpFriends {
    const TYPED: FormatVersion = known::V1_26_40;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for SilverfishWakeUpFriends {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.skeleton_horse_trap`: allows Equine mobs to be Horse Traps and be triggered like them, spawning a lightning bolt and a bunch of horses when a player is nearby. Can only be used by Horses, Mules, Donkeys and Skeleton Horses.
// `behavior.skeleton_horse_trap`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.skeleton_horse_trap")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct SkeletonHorseTrap {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `duration`: amount of time in seconds the trap exists. After this amount of time is elapsed, the trap is removed from the world if it hasn't been activated. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub duration: Option<StrictFloat<f32>>,
    /// `within_radius`: distance in blocks that the player has to be within to trigger the horse trap. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub within_radius: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl SkeletonHorseTrap {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `duration`, or its default 1.
    pub fn duration(&self) -> f32 {
        self.duration.map_or(1.0, |v| v.0)
    }

    /// `within_radius`, or its default 0.
    pub fn within_radius(&self) -> f32 {
        self.within_radius.map_or(0.0, |v| v.0)
    }
}

impl Goal for SkeletonHorseTrap {
    const TYPED: FormatVersion = known::V1_26_30;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for SkeletonHorseTrap {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.sleep`: allows mobs that own a bed to in a village to move to and sleep in it.
///
/// Deprecated members, reported and dropped: `poi_type`.
// `behavior.sleep`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.sleep")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Sleep {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `can_sleep_while_riding`: if true, the mob will be able to use the sleep goal if riding something. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub can_sleep_while_riding: Option<StrictBool>,
    /// `cooldown_time`: time in seconds the mob has to wait before using the goal again. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_time: Option<StrictFloat<f32>>,
    /// `sleep_collider_height`: the height of the mob's collider while sleeping. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sleep_collider_height: Option<StrictFloat<f32>>,
    /// `sleep_collider_width`: the width of the mob's collider while sleeping. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sleep_collider_width: Option<StrictFloat<f32>>,
    /// `sleep_y_offset`: the y offset of the mob's collider while sleeping. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sleep_y_offset: Option<StrictFloat<f32>>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this goal. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// `timeout_cooldown`: the cooldown time in seconds before the goal can be reused after a internal failure or timeout condition. Default 8; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub timeout_cooldown: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Sleep {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `can_sleep_while_riding`, or its default `false`.
    pub fn can_sleep_while_riding(&self) -> bool {
        self.can_sleep_while_riding.is_some_and(|v| v.0)
    }

    /// `cooldown_time`, or its default 0.
    pub fn cooldown_time(&self) -> f32 {
        self.cooldown_time.map_or(0.0, |v| v.0)
    }

    /// `sleep_collider_height`, or its default 1.
    pub fn sleep_collider_height(&self) -> f32 {
        self.sleep_collider_height.map_or(1.0, |v| v.0)
    }

    /// `sleep_collider_width`, or its default 1.
    pub fn sleep_collider_width(&self) -> f32 {
        self.sleep_collider_width.map_or(1.0, |v| v.0)
    }

    /// `sleep_y_offset`, or its default 1.
    pub fn sleep_y_offset(&self) -> f32 {
        self.sleep_y_offset.map_or(1.0, |v| v.0)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }

    /// `timeout_cooldown`, or its default 8.
    pub fn timeout_cooldown(&self) -> f32 {
        self.timeout_cooldown.map_or(8.0, |v| v.0)
    }
}

impl Goal for Sleep {
    const TYPED: FormatVersion = known::V1_26_20;

    const DEPRECATED: &'static [Deprecated] =
        &[("poi_type", deprecated_text!("poi_type"), known::V1_26_20)];

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.float("cooldown_time", &mut self.cooldown_time, Bound::Min(0.0));
        c.float(
            "sleep_collider_height",
            &mut self.sleep_collider_height,
            Bound::Min(0.0),
        );
        c.float(
            "sleep_collider_width",
            &mut self.sleep_collider_width,
            Bound::Min(0.0),
        );
        c.float("sleep_y_offset", &mut self.sleep_y_offset, Bound::Min(0.0));
        c.float(
            "speed_multiplier",
            &mut self.speed_multiplier,
            Bound::Min(0.0),
        );
        c.float(
            "timeout_cooldown",
            &mut self.timeout_cooldown,
            Bound::Min(0.0),
        );
    }
}

impl MolangFields for Sleep {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.sneeze`: allows the mob to sneeze, causing it to drop items and affect nearby mobs.
// `behavior.sneeze`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.sneeze")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Sneeze {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `cooldown_time`: time in seconds the mob has to wait before using the goal again. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_time: Option<StrictFloat<f32>>,
    /// `drop_item_chance`: the probability that the mob will drop an item when it sneezes. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub drop_item_chance: Option<StrictFloat<f32>>,
    /// `entity_types`: list of entity types this mob will startle (cause to jump) when it sneezes.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub entity_types: Option<EntityTypes>,
    /// `loot_table`: loot table to select dropped items from. Default `""`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub loot_table: Option<StrictString>,
    /// `prepare_sound`: sound to play when the sneeze is about to happen. Default `""`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub prepare_sound: Option<SoundEvent>,
    /// `prepare_time`: the time in seconds that the mob takes to prepare to sneeze (while the `prepare_sound` is playing). Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub prepare_time: Option<StrictFloat<f32>>,
    /// `probability`: the probability of sneezing. A value of 1.00 is 100%. Default 0.02.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub probability: Option<StrictFloat<f32>>,
    /// `sound`: sound to play when the sneeze occurs. Default `""`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sound: Option<SoundEvent>,
    /// `within_radius`: distance in blocks that mobs will be startled. Default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub within_radius: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Sneeze {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `cooldown_time`, or its default 0.
    pub fn cooldown_time(&self) -> f32 {
        self.cooldown_time.map_or(0.0, |v| v.0)
    }

    /// `drop_item_chance`, or its default 1.
    pub fn drop_item_chance(&self) -> f32 {
        self.drop_item_chance.map_or(1.0, |v| v.0)
    }

    /// `loot_table`, or its default `""`.
    pub fn loot_table(&self) -> &str {
        self.loot_table.as_ref().map_or("", |v| v.0.as_str())
    }

    /// `prepare_sound`, or its default `""`.
    pub fn prepare_sound(&self) -> &str {
        self.prepare_sound.as_ref().map_or("", SoundEvent::name)
    }

    /// `prepare_time`, or its default 1.
    pub fn prepare_time(&self) -> f32 {
        self.prepare_time.map_or(1.0, |v| v.0)
    }

    /// `probability`, or its default 0.02.
    pub fn probability(&self) -> f32 {
        self.probability.map_or(0.02, |v| v.0)
    }

    /// `sound`, or its default `""`.
    pub fn sound(&self) -> &str {
        self.sound.as_ref().map_or("", SoundEvent::name)
    }

    /// `within_radius`, or its default 0.
    pub fn within_radius(&self) -> f32 {
        self.within_radius.map_or(0.0, |v| v.0)
    }
}

impl Goal for Sneeze {
    const TYPED: FormatVersion = known::V1_26_30;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.entity_types("entity_types", self.entity_types.as_ref());
        c.sound("prepare_sound", self.prepare_sound.as_ref());
        c.sound("sound", self.sound.as_ref());
    }
}

impl MolangFields for Sneeze {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.sniff`: allows this entity to detect the nearest player within `sniffing_radius` and update its `minecraft:suspect_tracking` component state.
// `behavior.sniff`: typed from 1.21.110.
#[doc(alias = "minecraft:behavior.sniff")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Sniff {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `cooldown_range`: cooldown range between sniffs in seconds. Default 3 to 1e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_range: Option<FloatRange>,
    /// `duration`: sniffing duration in seconds. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub duration: Option<StrictFloat<f32>>,
    /// `sniffing_radius`: mob detection radius. Default 5.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sniffing_radius: Option<StrictFloat<f32>>,
    /// `suspicion_radius_horizontal`: mob suspicion horizontal radius. When a player is within this radius horizontally, the anger level towards that player is increased. Default 3.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub suspicion_radius_horizontal: Option<StrictFloat<f32>>,
    /// `suspicion_radius_vertical`: mob suspicion vertical radius. When a player is within this radius vertically, the anger level towards that player is increased. Default 3; at least 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub suspicion_radius_vertical: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Sniff {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `cooldown_range`, or its default 3 to 1e+01.
    pub fn cooldown_range(&self) -> (f32, f32) {
        self.cooldown_range
            .map_or((3.0, 1.0e1), |range| range.resolve(3.0, 1.0e1))
    }

    /// `duration`, or its default 1.
    pub fn duration(&self) -> f32 {
        self.duration.map_or(1.0, |v| v.0)
    }

    /// `sniffing_radius`, or its default 5.
    pub fn sniffing_radius(&self) -> f32 {
        self.sniffing_radius.map_or(5.0, |v| v.0)
    }

    /// `suspicion_radius_horizontal`, or its default 3.
    pub fn suspicion_radius_horizontal(&self) -> f32 {
        self.suspicion_radius_horizontal.map_or(3.0, |v| v.0)
    }

    /// `suspicion_radius_vertical`, or its default 3.
    pub fn suspicion_radius_vertical(&self) -> f32 {
        self.suspicion_radius_vertical.map_or(3.0, |v| v.0)
    }
}

impl Goal for Sniff {
    const TYPED: FormatVersion = known::V1_21_110;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.range("cooldown_range", &mut self.cooldown_range);
        c.float(
            "suspicion_radius_vertical",
            &mut self.suspicion_radius_vertical,
            Bound::Min(1.0),
        );
    }
}

impl MolangFields for Sniff {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.stay_near_noteblock`: the entity will attempt to toss the items from its inventory to a nearby recently played noteblock.
// `behavior.stay_near_noteblock`: typed from 1.21.110.
#[doc(alias = "minecraft:behavior.stay_near_noteblock")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct StayNearNoteblock {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `listen_time`: sets the time an entity should stay near a noteblock after hearing it. Default 30.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub listen_time: Option<StrictInt<i32>>,
    /// `speed`: sets the entity's speed when moving toward the block. Default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed: Option<StrictFloat<f32>>,
    /// `start_distance`: sets the distance the entity needs to be away from the block to attempt to start the goal. Default 1e+01.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub start_distance: Option<StrictFloat<f32>>,
    /// `stop_distance`: sets the distance from the block the entity will attempt to reach. Default 2.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub stop_distance: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl StayNearNoteblock {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `listen_time`, or its default 30.
    pub fn listen_time(&self) -> i32 {
        self.listen_time.map_or(30, |v| v.0)
    }

    /// `speed`, or its default 1.
    pub fn speed(&self) -> f32 {
        self.speed.map_or(1.0, |v| v.0)
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

impl Goal for StayNearNoteblock {
    const TYPED: FormatVersion = known::V1_21_110;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for StayNearNoteblock {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.stay_while_sitting`: allows the mob to stay put while it is in a sitting state instead of doing something else.
// `behavior.stay_while_sitting`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.stay_while_sitting")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct StayWhileSitting {
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

impl StayWhileSitting {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }
}

impl Goal for StayWhileSitting {
    const TYPED: FormatVersion = known::V1_26_30;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
    }
}

impl MolangFields for StayWhileSitting {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.stomp_turtle_egg`: allows this mob to stomp turtle eggs.
// `behavior.stomp_turtle_egg`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.stomp_turtle_egg")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct StompTurtleEgg {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `goal_radius`: distance in blocks within the mob considers it has reached the goal. This is the "wiggle room" to stop the AI from bouncing back and forth trying to reach a specific spot. Default 0.5; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub goal_radius: Option<StrictFloat<f32>>,
    /// `interval`: a random value to determine when to randomly move somewhere. This has a 1/interval chance to choose this goal. Default 120; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub interval: Option<StrictInt<i32>>,
    /// `search_count`: the number of randomly selected blocks each tick that the mob will check within its search range and height for a valid block to move to. A value of 0 will have the mob check every block within range in one tick. Default 10; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_count: Option<StrictInt<i32>>,
    /// `search_height`: height in blocks the mob will look for turtle eggs to move towards. Default 1; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_height: Option<StrictInt<i32>>,
    /// `search_range`: the distance in blocks it will look for turtle eggs to move towards. Default 0; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub search_range: Option<StrictInt<i32>>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this goal. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl StompTurtleEgg {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `goal_radius`, or its default 0.5.
    pub fn goal_radius(&self) -> f32 {
        self.goal_radius.map_or(0.5, |v| v.0)
    }

    /// `interval`, or its default 120.
    pub fn interval(&self) -> i32 {
        self.interval.map_or(120, |v| v.0)
    }

    /// `search_count`, or its default 10.
    pub fn search_count(&self) -> i32 {
        self.search_count.map_or(10, |v| v.0)
    }

    /// `search_height`, or its default 1.
    pub fn search_height(&self) -> i32 {
        self.search_height.map_or(1, |v| v.0)
    }

    /// `search_range`, or its default 0.
    pub fn search_range(&self) -> i32 {
        self.search_range.map_or(0, |v| v.0)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(1.0, |v| v.0)
    }
}

impl Goal for StompTurtleEgg {
    const TYPED: FormatVersion = known::V1_26_10;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.float("goal_radius", &mut self.goal_radius, Bound::Above(0.0));
        c.int("interval", &mut self.interval, Bound::Above(0));
        c.int("search_count", &mut self.search_count, Bound::Min(0));
        c.int("search_height", &mut self.search_height, Bound::Above(0));
        c.int("search_range", &mut self.search_range, Bound::Above(0));
        c.float(
            "speed_multiplier",
            &mut self.speed_multiplier,
            Bound::Min(0.0),
        );
    }
}

impl MolangFields for StompTurtleEgg {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

/// `minecraft:behavior.timer_flag_1`: fires an event when this behavior starts, then waits for a duration before stopping. When stopping due to that timeout or due to being interrupted by another behavior, fires another event. `query.timer_flag_1` will return 1.0 on both the client and server when this behavior is running, and 0.0 otherwise.
// `behavior.timer_flag_1`: typed from 1.26.0.
#[doc(alias = "minecraft:behavior.timer_flag_1")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct TimerFlag1 {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `cooldown_range`: goal cooldown range in seconds. If specified, the cooldown will have to elapse even before the goal can be selected for the first time. Default 0 to 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_range: Option<FloatRange>,
    /// `duration_range`: goal duration range in seconds. Default 0 to 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub duration_range: Option<FloatRange>,
    /// `on_end`: event(s) to run when the goal ends.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_end: Option<EventTrigger>,
    /// `on_start`: event(s) to run when the goal starts.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_start: Option<EventTrigger>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl TimerFlag1 {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `cooldown_range`, or its default 0 to 0.
    pub fn cooldown_range(&self) -> (f32, f32) {
        self.cooldown_range
            .map_or((0.0, 0.0), |range| range.resolve(0.0, 0.0))
    }

    /// `duration_range`, or its default 0 to 0.
    pub fn duration_range(&self) -> (f32, f32) {
        self.duration_range
            .map_or((0.0, 0.0), |range| range.resolve(0.0, 0.0))
    }
}

impl Goal for TimerFlag1 {
    const TYPED: FormatVersion = known::V1_26_0;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.range("cooldown_range", &mut self.cooldown_range);
        c.range("duration_range", &mut self.duration_range);
        c.trigger("on_end", self.on_end.as_ref());
        c.trigger("on_start", self.on_start.as_ref());
    }
}

impl MolangFields for TimerFlag1 {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(&mut triggers, "on_end", self.on_end.as_ref());
        push_trigger(&mut triggers, "on_start", self.on_start.as_ref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.timer_flag_2`: fires an event when this behavior starts, then waits for a duration before stopping. When stopping due to that timeout or due to being interrupted by another behavior, fires another event. `query.timer_flag_2` will return 1.0 on both the client and server when this behavior is running, and 0.0 otherwise.
// `behavior.timer_flag_2`: typed from 1.26.0.
#[doc(alias = "minecraft:behavior.timer_flag_2")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct TimerFlag2 {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `cooldown_range`: goal cooldown range in seconds. If specified, the cooldown will have to elapse even before the goal can be selected for the first time. Default 0 to 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_range: Option<FloatRange>,
    /// `duration_range`: goal duration range in seconds. Default 0 to 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub duration_range: Option<FloatRange>,
    /// `on_end`: event(s) to run when the goal ends.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_end: Option<EventTrigger>,
    /// `on_start`: event(s) to run when the goal starts.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_start: Option<EventTrigger>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl TimerFlag2 {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `cooldown_range`, or its default 0 to 0.
    pub fn cooldown_range(&self) -> (f32, f32) {
        self.cooldown_range
            .map_or((0.0, 0.0), |range| range.resolve(0.0, 0.0))
    }

    /// `duration_range`, or its default 0 to 0.
    pub fn duration_range(&self) -> (f32, f32) {
        self.duration_range
            .map_or((0.0, 0.0), |range| range.resolve(0.0, 0.0))
    }
}

impl Goal for TimerFlag2 {
    const TYPED: FormatVersion = known::V1_26_0;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.range("cooldown_range", &mut self.cooldown_range);
        c.range("duration_range", &mut self.duration_range);
        c.trigger("on_end", self.on_end.as_ref());
        c.trigger("on_start", self.on_start.as_ref());
    }
}

impl MolangFields for TimerFlag2 {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(&mut triggers, "on_end", self.on_end.as_ref());
        push_trigger(&mut triggers, "on_start", self.on_start.as_ref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.timer_flag_3`: fires an event when this behavior starts, then waits for a duration before stopping. When stopping due to that timeout or due to being interrupted by another behavior, fires another event. `query.timer_flag_3` will return 1.0 on both the client and server when this behavior is running, and 0.0 otherwise.
// `behavior.timer_flag_3`: typed from 1.26.0.
#[doc(alias = "minecraft:behavior.timer_flag_3")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct TimerFlag3 {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `cooldown_range`: goal cooldown range in seconds. If specified, the cooldown will have to elapse even before the goal can be selected for the first time. Default 0 to 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown_range: Option<FloatRange>,
    /// `duration_range`: goal duration range in seconds. Default 0 to 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub duration_range: Option<FloatRange>,
    /// `on_end`: event(s) to run when the goal ends.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_end: Option<EventTrigger>,
    /// `on_start`: event(s) to run when the goal starts.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_start: Option<EventTrigger>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl TimerFlag3 {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `cooldown_range`, or its default 0 to 0.
    pub fn cooldown_range(&self) -> (f32, f32) {
        self.cooldown_range
            .map_or((0.0, 0.0), |range| range.resolve(0.0, 0.0))
    }

    /// `duration_range`, or its default 0 to 0.
    pub fn duration_range(&self) -> (f32, f32) {
        self.duration_range
            .map_or((0.0, 0.0), |range| range.resolve(0.0, 0.0))
    }
}

impl Goal for TimerFlag3 {
    const TYPED: FormatVersion = known::V1_26_0;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.range("cooldown_range", &mut self.cooldown_range);
        c.range("duration_range", &mut self.duration_range);
        c.trigger("on_end", self.on_end.as_ref());
        c.trigger("on_start", self.on_start.as_ref());
    }
}

impl MolangFields for TimerFlag3 {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(&mut triggers, "on_end", self.on_end.as_ref());
        push_trigger(&mut triggers, "on_start", self.on_start.as_ref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.work`: allows the NPC to use the POI.
// `behavior.work`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.work")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Work {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `active_time`: the amount of ticks the NPC will stay in their the work location. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub active_time: Option<StrictInt<i32>>,
    /// `can_work_in_rain`: if true, this entity can work when their jobsite POI is being rained on. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub can_work_in_rain: Option<StrictBool>,
    /// `goal_cooldown`: the amount of ticks the goal will be on cooldown before it can be used again. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub goal_cooldown: Option<StrictInt<i32>>,
    /// `on_arrival`: event to run when the mob reaches their jobsite.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_arrival: Option<EventTrigger>,
    /// `sound_delay_max`: the max interval in which a sound will play. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sound_delay_max: Option<StrictInt<i32>>,
    /// `sound_delay_min`: the min interval in which a sound will play. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sound_delay_min: Option<StrictInt<i32>>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 0.5; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// `work_in_rain_tolerance`: if `can_work_in_rain` is false, this is the maximum number of ticks left in the goal where rain will not interrupt the goal. Default -1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub work_in_rain_tolerance: Option<StrictInt<i32>>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Work {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `active_time`, or its default 0.
    pub fn active_time(&self) -> i32 {
        self.active_time.map_or(0, |v| v.0)
    }

    /// `can_work_in_rain`, or its default `false`.
    pub fn can_work_in_rain(&self) -> bool {
        self.can_work_in_rain.is_some_and(|v| v.0)
    }

    /// `goal_cooldown`, or its default 0.
    pub fn goal_cooldown(&self) -> i32 {
        self.goal_cooldown.map_or(0, |v| v.0)
    }

    /// `sound_delay_max`, or its default 0.
    pub fn sound_delay_max(&self) -> i32 {
        self.sound_delay_max.map_or(0, |v| v.0)
    }

    /// `sound_delay_min`, or its default 0.
    pub fn sound_delay_min(&self) -> i32 {
        self.sound_delay_min.map_or(0, |v| v.0)
    }

    /// `speed_multiplier`, or its default 0.5.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(0.5, |v| v.0)
    }

    /// `work_in_rain_tolerance`, or its default -1.
    pub fn work_in_rain_tolerance(&self) -> i32 {
        self.work_in_rain_tolerance.map_or(-1, |v| v.0)
    }
}

impl Goal for Work {
    const TYPED: FormatVersion = known::V1_26_10;

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        // A negative legacy `active_time` is caught by the schema reader's numeric check: the
        // game's text is a kept error and the member takes its default. The game's legacy goal
        // reader has no `behavior.work` row, so there is no separate legacy rule.
        c.int("active_time", &mut self.active_time, Bound::Min(0));
        c.int("goal_cooldown", &mut self.goal_cooldown, Bound::Min(0));
        c.trigger("on_arrival", self.on_arrival.as_ref());
        c.int("sound_delay_max", &mut self.sound_delay_max, Bound::Min(0));
        c.int("sound_delay_min", &mut self.sound_delay_min, Bound::Min(0));
        c.float(
            "speed_multiplier",
            &mut self.speed_multiplier,
            Bound::Above(0.0),
        );
        c.int(
            "work_in_rain_tolerance",
            &mut self.work_in_rain_tolerance,
            Bound::Min(0),
        );
    }
}

impl MolangFields for Work {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(&mut triggers, "on_arrival", self.on_arrival.as_ref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.work_composter`: allows the NPC to use the composter POI to convert excess seeds into bone meal.
///
/// Deprecated members, reported and dropped: `sound_delay_min`, `sound_delay_max`.
// `behavior.work_composter`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.work_composter")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct WorkComposter {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `active_time`: the amount of ticks the NPC will stay in their the work location. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub active_time: Option<StrictInt<i32>>,
    /// `block_interaction_max`: the maximum number of times the mob will interact with the composter. Default 1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub block_interaction_max: Option<StrictInt<i32>>,
    /// `can_empty_composter`: determines whether the mob can empty a full composter. Default `true`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub can_empty_composter: Option<StrictBool>,
    /// `can_fill_composter`: determines whether the mob can add items to a composter given that it is not full. Default `true`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub can_fill_composter: Option<StrictBool>,
    /// `can_work_in_rain`: if true, this entity can work when their jobsite POI is being rained on. Default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub can_work_in_rain: Option<StrictBool>,
    /// `goal_cooldown`: the amount of ticks the goal will be on cooldown before it can be used again. Default 0; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub goal_cooldown: Option<StrictInt<i32>>,
    /// `items_per_use_max`: the maximum number of items which can be added to the composter per block interaction. Default 20; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub items_per_use_max: Option<StrictInt<i32>>,
    /// `min_item_count`: limits the amount of each compostable item the mob can use. Any amount held over this number will be composted if possible. Default 10.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_item_count: Option<StrictInt<i32>>,
    /// `on_arrival`: event to run when the mob reaches their jobsite.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub on_arrival: Option<EventTrigger>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 0.5; greater than 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub speed_multiplier: Option<StrictFloat<f32>>,
    /// `use_block_max`: the maximum interval in which the mob will interact with the composter. Default 200; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub use_block_max: Option<StrictInt<i32>>,
    /// `use_block_min`: the minimum interval in which the mob will interact with the composter. Default 100; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub use_block_min: Option<StrictInt<i32>>,
    /// `work_in_rain_tolerance`: if `can_work_in_rain` is false, this is the maximum number of ticks left in the goal where rain will not interrupt the goal. Default -1; at least 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub work_in_rain_tolerance: Option<StrictInt<i32>>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
    /// Members the goal does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl WorkComposter {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.map_or(0, |v| v.0)
    }

    /// `active_time`, or its default 0.
    pub fn active_time(&self) -> i32 {
        self.active_time.map_or(0, |v| v.0)
    }

    /// `block_interaction_max`, or its default 1.
    pub fn block_interaction_max(&self) -> i32 {
        self.block_interaction_max.map_or(1, |v| v.0)
    }

    /// `can_empty_composter`, or its default `true`.
    pub fn can_empty_composter(&self) -> bool {
        self.can_empty_composter.is_none_or(|v| v.0)
    }

    /// `can_fill_composter`, or its default `true`.
    pub fn can_fill_composter(&self) -> bool {
        self.can_fill_composter.is_none_or(|v| v.0)
    }

    /// `can_work_in_rain`, or its default `false`.
    pub fn can_work_in_rain(&self) -> bool {
        self.can_work_in_rain.is_some_and(|v| v.0)
    }

    /// `goal_cooldown`, or its default 0.
    pub fn goal_cooldown(&self) -> i32 {
        self.goal_cooldown.map_or(0, |v| v.0)
    }

    /// `items_per_use_max`, or its default 20.
    pub fn items_per_use_max(&self) -> i32 {
        self.items_per_use_max.map_or(20, |v| v.0)
    }

    /// `min_item_count`, or its default 10.
    pub fn min_item_count(&self) -> i32 {
        self.min_item_count.map_or(10, |v| v.0)
    }

    /// `speed_multiplier`, or its default 0.5.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.map_or(0.5, |v| v.0)
    }

    /// `use_block_max`, or its default 200.
    pub fn use_block_max(&self) -> i32 {
        self.use_block_max.map_or(200, |v| v.0)
    }

    /// `use_block_min`, or its default 100.
    pub fn use_block_min(&self) -> i32 {
        self.use_block_min.map_or(100, |v| v.0)
    }

    /// `work_in_rain_tolerance`, or its default -1.
    pub fn work_in_rain_tolerance(&self) -> i32 {
        self.work_in_rain_tolerance.map_or(-1, |v| v.0)
    }
}

impl Goal for WorkComposter {
    const TYPED: FormatVersion = known::V1_26_10;

    const DEPRECATED: &'static [Deprecated] = &[
        (
            "sound_delay_min",
            deprecated_text!("sound_delay_min"),
            known::V1_26_10,
        ),
        (
            "sound_delay_max",
            deprecated_text!("sound_delay_max"),
            known::V1_26_10,
        ),
    ];

    fn check(&mut self, c: &mut Check<'_>) {
        c.base(&mut self.priority);
        c.int("active_time", &mut self.active_time, Bound::Min(0));
        c.int(
            "block_interaction_max",
            &mut self.block_interaction_max,
            Bound::Min(0),
        );
        c.int("goal_cooldown", &mut self.goal_cooldown, Bound::Min(0));
        c.int(
            "items_per_use_max",
            &mut self.items_per_use_max,
            Bound::Min(0),
        );
        c.trigger("on_arrival", self.on_arrival.as_ref());
        c.float(
            "speed_multiplier",
            &mut self.speed_multiplier,
            Bound::Above(0.0),
        );
        c.int("use_block_max", &mut self.use_block_max, Bound::Min(0));
        c.int("use_block_min", &mut self.use_block_min, Bound::Min(0));
        c.int(
            "work_in_rain_tolerance",
            &mut self.work_in_rain_tolerance,
            Bound::Min(0),
        );
    }
}

impl MolangFields for WorkComposter {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(&mut triggers, "on_arrival", self.on_arrival.as_ref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}
