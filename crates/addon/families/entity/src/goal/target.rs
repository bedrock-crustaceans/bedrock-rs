//! Targeting goals: choosing whom to attack, defend or avoid.

use bedrock_addon_core::molang::MolangVisitor;
use bedrock_addon_core::schema::{self, GoalMeta, Schema};
use bedrock_addon_core::shared::{EventTrigger, FloatRange};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_shared_shapes::ItemDescriptor;
use facet::Facet;

use super::check::{Deprecated, Goal, GoalCheck, deprecated_text};
use super::common::{
    ControlFlag, EntityTypes, LegacyConditions, SoundEvent, TargetSelectionMethod,
};
use super::molang::{push_trigger, push_triggers, visit_conditions, visit_items};

/// `minecraft:behavior.avoid_block`: allows this entity to avoid certain blocks.
// `behavior.avoid_block`: typed from 1.21.120.
#[doc(alias = "minecraft:behavior.avoid_block")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang_with = Self::legacy_molang)]
pub struct AvoidBlock {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `avoid_block_sound`: the sound event to play when the mob is avoiding a block. Default `undefined`.
    #[facet(schema::fallback = "undefined")]
    pub avoid_block_sound: Option<SoundEvent>,
    /// `on_escape`: escape triggers.
    pub on_escape: Option<Vec<EventTrigger>>,
    /// `search_height`: maximum distance to look for a block in y. Default 0.
    #[facet(schema::fallback = 0)]
    pub search_height: Option<i32>,
    /// `search_range`: maximum distance to look for a block in xz. Default 0.
    #[facet(schema::fallback = 0)]
    pub search_range: Option<i32>,
    /// `sound_interval`: the range of time in seconds to randomly wait before playing the sound again. Default 3 to 8.
    #[facet(schema::fallback = (3.0, 8.0))]
    pub sound_interval: Option<FloatRange>,
    /// `sprint_speed_modifier`: modifier for sprint speed. 1.0 means keep the regular speed, while higher numbers make the sprint speed faster. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub sprint_speed_modifier: Option<f32>,
    /// `target_blocks`: list of block types this mob avoids.
    pub target_blocks: Option<Vec<ItemDescriptor>>,
    /// `target_selection_method`: block search method. Default `nearest`.
    pub target_selection_method: Option<TargetSelectionMethod>,
    /// `tick_interval`: should start tick interval. Default 1.
    #[facet(schema::fallback = 1)]
    pub tick_interval: Option<i32>,
    /// `walk_speed_modifier`: modifier for walking speed. 1.0 means keep the regular speed, while higher numbers make the walking speed faster. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub walk_speed_modifier: Option<f32>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl AvoidBlock {
    /// `target_selection_method`, or its default `nearest`.
    pub fn target_selection_method(&self) -> TargetSelectionMethod {
        self.target_selection_method
            .unwrap_or(TargetSelectionMethod::Nearest)
    }
}

impl Goal for AvoidBlock {
    const TYPED: FormatVersion = known::V1_21_120;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
        c.sound("avoid_block_sound", self.avoid_block_sound.as_ref());
        c.triggers("on_escape", self.on_escape.as_deref());
        c.range("sound_interval", &mut self.sound_interval);
        c.items("target_blocks", self.target_blocks.as_deref());
    }
}

impl AvoidBlock {
    /// The goal's Molang walk (`schema::molang_with`): its triggers' legacy conditions
    /// read once into its cache, and its descriptors' tags.
    fn legacy_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visit_items(visitor, "target_blocks", &mut self.target_blocks);
        let mut triggers = Vec::new();
        push_triggers(&mut triggers, "on_escape", self.on_escape.as_deref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.avoid_mob_type`: allows the entity to run away from other entities that meet the criteria specified.
// `behavior.avoid_mob_type`: typed from 1.21.120.
#[doc(alias = "minecraft:behavior.avoid_mob_type")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang_with = Self::legacy_molang)]
pub struct AvoidMobType {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `avoid_mob_sound`: the sound event to play when the mob is avoiding another mob. Default `undefined`.
    #[facet(schema::fallback = "undefined")]
    pub avoid_mob_sound: Option<SoundEvent>,
    /// `avoid_target_xz`: the next target position the entity chooses to avoid another entity will be chosen within this XZ Distance. Default 16.
    #[facet(schema::fallback = 16)]
    pub avoid_target_xz: Option<i32>,
    /// `avoid_target_y`: the next target position the entity chooses to avoid another entity will be chosen within this Y Distance. Default 7.
    #[facet(schema::fallback = 7)]
    pub avoid_target_y: Option<i32>,
    /// `entity_types`: the list of conditions another entity must meet to be a valid target to avoid.
    pub entity_types: Option<EntityTypes>,
    /// `ignore_visibility`: whether or not to ignore direct line of sight while this entity is running away from other specified entities. Default `false`.
    #[facet(schema::fallback = false)]
    pub ignore_visibility: Option<bool>,
    /// `max_dist`: maximum distance to look for an avoid target for the entity. Default 3.
    #[facet(schema::fallback = 3.0)]
    pub max_dist: Option<f32>,
    /// `max_flee`: how many blocks away from its avoid target the entity must be for it to stop fleeing from the avoid target. Default 1e+01.
    #[facet(schema::fallback = 1.0e1)]
    pub max_flee: Option<f32>,
    /// `on_escape_event`: event that is triggered when escaping from a mob.
    pub on_escape_event: Option<EventTrigger>,
    /// `probability_per_strength`: percent chance this entity will stop avoiding another entity based on that entity's strength, where 1.0 = 100%. Default 1; at most 1.
    #[facet(schema::fallback = 1.0, schema::max = 1.0)]
    pub probability_per_strength: Option<f32>,
    /// `remove_target`: determine if we should remove target when fleeing or not. Default `false`.
    #[facet(schema::fallback = false)]
    pub remove_target: Option<bool>,
    /// `river_speed_multiplier`: multiplier for speed when following river. 1.0 means keep the regular speed, while higher numbers make the speed faster. Default 0.75.
    #[facet(schema::fallback = 0.75)]
    pub river_speed_multiplier: Option<f32>,
    /// `sound_interval`: the range of time in seconds to randomly wait before playing the sound again. Default 3 to 8.
    #[facet(schema::fallback = (3.0, 8.0))]
    pub sound_interval: Option<FloatRange>,
    /// `sprint_distance`: how many blocks within range of its avoid target the entity must be for it to begin sprinting away from the avoid target. Default 7.
    #[facet(schema::fallback = 7.0)]
    pub sprint_distance: Option<f32>,
    /// `sprint_speed_multiplier`: multiplier for sprint speed. 1.0 means keep the regular speed, while higher numbers make the sprint speed faster. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub sprint_speed_multiplier: Option<f32>,
    /// `walk_speed_multiplier`: multiplier for walking speed. 1.0 means keep the regular speed, while higher numbers make the walking speed faster. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub walk_speed_multiplier: Option<f32>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl Goal for AvoidMobType {
    const TYPED: FormatVersion = known::V1_21_120;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
        c.sound("avoid_mob_sound", self.avoid_mob_sound.as_ref());
        c.entity_types("entity_types", self.entity_types.as_ref());
        c.trigger("on_escape_event", self.on_escape_event.as_ref());
        c.range("sound_interval", &mut self.sound_interval);
    }
}

impl AvoidMobType {
    /// The goal's Molang walk (`schema::molang_with`): its triggers' legacy conditions
    /// read once into its cache, and its descriptors' tags.
    fn legacy_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(
            &mut triggers,
            "on_escape_event",
            self.on_escape_event.as_ref(),
        );
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.defend_trusted_target`: allows the mob to target another mob that hurts an entity it trusts.
///
/// Deprecated members, reported and dropped: `attack_owner`, `must_reach`, `persist_time`, `reselect_targets`, `scan_interval`, `set_persistent`, `target_acquisition_probability`, `target_invisible_multiplier`, `target_search_height`, `target_sneak_visibility_multiplier`.
// `behavior.defend_trusted_target`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.defend_trusted_target")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang_with = Self::legacy_molang)]
pub struct DefendTrustedTarget {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `aggro_sound`: sound to occasionally play while defending. Default `undefined`.
    #[facet(schema::fallback = "undefined")]
    pub aggro_sound: Option<SoundEvent>,
    /// `attack_interval`: time in seconds between attacks. Default 0 to 0.
    #[facet(schema::fallback = (0.0, 0.0))]
    pub attack_interval: Option<FloatRange>,
    /// `entity_types`: list of entity types that this mob considers valid targets.
    pub entity_types: Option<EntityTypes>,
    /// `must_see`: if true, only entities in this mob's viewing range can be selected as targets. Default `false`.
    #[facet(schema::fallback = false)]
    pub must_see: Option<bool>,
    /// `must_see_forget_duration`: determines the amount of time in seconds that this mob will look for a target before forgetting about it and looking for a new one when the target isn't visible any more. Default 3; at least 0.
    #[facet(schema::fallback = 3.0, schema::min = 0.0)]
    pub must_see_forget_duration: Option<f32>,
    /// `on_defend_start`: trigger to be executed when defending begins.
    pub on_defend_start: Option<EventTrigger>,
    /// `within_radius`: distance in blocks that the target can be within to launch an attack. Default 0; at least 0.
    #[facet(schema::fallback = 0.0, schema::min = 0.0)]
    pub within_radius: Option<f32>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl Goal for DefendTrustedTarget {
    const TYPED: FormatVersion = known::V1_26_30;

    const DEPRECATED: &'static [Deprecated] = &[
        (
            "attack_owner",
            deprecated_text!("attack_owner"),
            known::V1_26_30,
        ),
        (
            "must_reach",
            deprecated_text!("must_reach"),
            known::V1_26_30,
        ),
        (
            "persist_time",
            deprecated_text!("persist_time"),
            known::V1_26_30,
        ),
        (
            "reselect_targets",
            deprecated_text!("reselect_targets"),
            known::V1_26_30,
        ),
        (
            "scan_interval",
            deprecated_text!("scan_interval"),
            known::V1_26_30,
        ),
        (
            "set_persistent",
            deprecated_text!("set_persistent"),
            known::V1_26_30,
        ),
        (
            "target_acquisition_probability",
            deprecated_text!("target_acquisition_probability"),
            known::V1_26_30,
        ),
        (
            "target_invisible_multiplier",
            deprecated_text!("target_invisible_multiplier"),
            known::V1_26_30,
        ),
        (
            "target_search_height",
            deprecated_text!("target_search_height"),
            known::V1_26_30,
        ),
        (
            "target_sneak_visibility_multiplier",
            deprecated_text!("target_sneak_visibility_multiplier"),
            known::V1_26_30,
        ),
    ];

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
        c.sound("aggro_sound", self.aggro_sound.as_ref());
        c.range("attack_interval", &mut self.attack_interval);
        c.entity_types("entity_types", self.entity_types.as_ref());
        c.trigger("on_defend_start", self.on_defend_start.as_ref());
    }
}

impl DefendTrustedTarget {
    /// The goal's Molang walk (`schema::molang_with`): its triggers' legacy conditions
    /// read once into its cache, and its descriptors' tags.
    fn legacy_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(
            &mut triggers,
            "on_defend_start",
            self.on_defend_start.as_ref(),
        );
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.defend_village_target`: allows the entity to stay in a village and defend the village from aggressors. If a player is in bad standing with the village this goal will cause the entity to attack the player regardless of filter conditions.
// `behavior.defend_village_target`: typed from 1.26.0.
#[doc(alias = "minecraft:behavior.defend_village_target")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct DefendVillageTarget {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `attack_chance`: the percentage chance that the entity has to attack aggressors of its village, where 1.0 = 100%. Default 0.05; 0 to 1.
    #[facet(schema::fallback = 0.05, schema::min = 0.0, schema::max = 1.0)]
    pub attack_chance: Option<f32>,
    /// `attack_owner`: if true, this entity can attack its owner. Default `false`.
    #[facet(schema::fallback = false)]
    pub attack_owner: Option<bool>,
    /// `entity_types`: filters which types of targets are valid for this entity.
    pub entity_types: Option<EntityTypes>,
    /// `must_reach`: if true, this entity requires a path to the target. Default `false`.
    #[facet(schema::fallback = false)]
    pub must_reach: Option<bool>,
    /// `must_see`: determines if target-validity requires this entity to be in range only, or both in range and in sight. Default `false`.
    #[facet(schema::fallback = false)]
    pub must_see: Option<bool>,
    /// `must_see_forget_duration`: time (in seconds) the target must not be seen by this entity to become invalid. Used only if `must_see` is true. Default 3; at least 0.
    #[facet(schema::fallback = 3.0, schema::min = 0.0)]
    pub must_see_forget_duration: Option<f32>,
    /// `persist_time`: time (in seconds) this entity can continue attacking the target after the target is no longer valid. Default 0; at least 0.
    #[facet(schema::fallback = 0.0, schema::min = 0.0)]
    pub persist_time: Option<f32>,
    /// `within_radius`: maximum distance this entity can be from the target when following it, otherwise the target becomes invalid. This value is only used if the entity doesn't declare `minecraft:follow_range`. Default 0; at least 0.
    #[facet(schema::fallback = 0.0, schema::min = 0.0)]
    pub within_radius: Option<f32>,
}

impl Goal for DefendVillageTarget {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
        // Bounds by hand, not schema::min/max: a bound here follows another check, and the
        // derived check runs first, which would reorder the findings.
        c.entity_types("entity_types", self.entity_types.as_ref());
    }
}

/// `minecraft:behavior.hurt_by_target`: allows the mob to target another mob that hurts them.
///
/// Deprecated members, reported and dropped: `attack_owner`, `must_reach`, `must_see`, `must_see_forget_duration`, `persist_time`, `within_radius`.
// `behavior.hurt_by_target`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.hurt_by_target")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct HurtByTarget {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `entity_types`: list of entity types that this mob can target when hurt by them.
    pub entity_types: Option<EntityTypes>,
    /// `hurt_owner`: if true, the mob will hurt its owner and other mobs with the same owner as itself. Default `false`.
    #[facet(schema::fallback = false)]
    pub hurt_owner: Option<bool>,
}

impl Goal for HurtByTarget {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    const DEPRECATED: &'static [Deprecated] = &[
        (
            "attack_owner",
            deprecated_text!("attack_owner"),
            known::V1_26_30,
        ),
        (
            "must_reach",
            deprecated_text!("must_reach"),
            known::V1_26_30,
        ),
        ("must_see", deprecated_text!("must_see"), known::V1_26_30),
        (
            "must_see_forget_duration",
            deprecated_text!("must_see_forget_duration"),
            known::V1_26_30,
        ),
        (
            "persist_time",
            deprecated_text!("persist_time"),
            known::V1_26_30,
        ),
        (
            "within_radius",
            deprecated_text!("within_radius"),
            known::V1_26_30,
        ),
    ];

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
        c.entity_types("entity_types", self.entity_types.as_ref());
    }
}

/// `minecraft:behavior.nearest_attackable_target`: allows an entity to attack the closest target within a given subset of specific target types.
// `behavior.nearest_attackable_target`: typed from 1.26.0.
#[doc(alias = "minecraft:behavior.nearest_attackable_target")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct NearestAttackableTarget {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `attack_interval`: time range (in seconds) between searching for an attack target, range is in (0, `attack_interval`\]. Only used if `attack_interval` is greater than 0, otherwise `scan_interval` is used. Default 0 to 0.
    #[facet(schema::fallback = (0.0, 0.0))]
    pub attack_interval: Option<FloatRange>,
    /// `attack_owner`: if true, this entity can attack its owner. Default `false`.
    #[facet(schema::fallback = false)]
    pub attack_owner: Option<bool>,
    /// `entity_types`: filters which types of targets are valid for this entity.
    pub entity_types: Option<EntityTypes>,
    /// `must_reach`: if true, this entity requires a path to the target. Default `false`.
    #[facet(schema::fallback = false)]
    pub must_reach: Option<bool>,
    /// `must_see`: determines if target-validity requires this entity to be in range only, or both in range and in sight. Default `false`.
    #[facet(schema::fallback = false)]
    pub must_see: Option<bool>,
    /// `must_see_forget_duration`: time (in seconds) the target must not be seen by this entity to become invalid. Used only if `must_see` is true. Default 3; at least 0.
    #[facet(schema::fallback = 3.0, schema::min = 0.0)]
    pub must_see_forget_duration: Option<f32>,
    /// `persist_time`: time (in seconds) this entity can continue attacking the target after the target is no longer valid. Default 0; at least 0.
    #[facet(schema::fallback = 0.0, schema::min = 0.0)]
    pub persist_time: Option<f32>,
    /// `reselect_targets`: allows the attacking entity to update the nearest target, otherwise a target is only reselected after each `scan_interval` or `attack_interval`. Default `false`.
    #[facet(schema::fallback = false)]
    pub reselect_targets: Option<bool>,
    /// `scan_interval`: if `attack_interval` is 0 or isn't declared, then between attacks: scanning for a new target occurs every amount of ticks equal to `scan_interval`, minimum value is 1. Values under 10 can affect performance. Default 10; at least 1.
    #[facet(schema::fallback = 10, schema::min = 1)]
    pub scan_interval: Option<i32>,
    /// `set_persistent`: allows the actor to be set to persist upon targeting a player. Default `false`.
    #[facet(schema::fallback = false)]
    pub set_persistent: Option<bool>,
    /// `target_acquisition_probability`: probability (0.0 to 1.0) that this entity will accept a found target. Checked each time a valid target is found during scanning. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub target_acquisition_probability: Option<f32>,
    /// `target_invisible_multiplier`: multiplied with the target's armor coverage percentage to modify `max_dist` when detecting an invisible target. Default 0.7.
    #[facet(schema::fallback = 0.7)]
    pub target_invisible_multiplier: Option<f32>,
    /// `target_search_height`: maximum vertical target-search distance, if it's greater than the target type's `max_dist`. A negative value defaults to `entity_types` greatest `max_dist`. Default -1; at least -1.
    #[facet(schema::fallback = -1.0, schema::min = -1.0)]
    pub target_search_height: Option<f32>,
    /// `target_sneak_visibility_multiplier`: multiplied with the target type's `max_dist` when trying to detect a sneaking target. Default 0.8.
    #[facet(schema::fallback = 0.8)]
    pub target_sneak_visibility_multiplier: Option<f32>,
    /// `within_radius`: maximum distance this entity can be from the target when following it, otherwise the target becomes invalid. This value is only used if the entity doesn't declare `minecraft:follow_range`. Default 0; at least 0.
    #[facet(schema::fallback = 0.0, schema::min = 0.0)]
    pub within_radius: Option<f32>,
}

impl Goal for NearestAttackableTarget {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
        c.range("attack_interval", &mut self.attack_interval);
        c.entity_types("entity_types", self.entity_types.as_ref());
    }
}

/// `minecraft:behavior.nearest_prioritized_attackable_target`: allows the mob to check for and pursue the nearest valid target.
///
/// Deprecated members, reported and dropped: `attack_owner`, `target_acquisition_probability`, `target_invisible_multiplier`, `target_sneak_visibility_multiplier`.
// `behavior.nearest_prioritized_attackable_target`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.nearest_prioritized_attackable_target")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct NearestPrioritizedAttackableTarget {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `attack_interval`: time in seconds before selecting a target. Default 0 to 0.
    #[facet(schema::fallback = (0.0, 0.0))]
    pub attack_interval: Option<FloatRange>,
    /// `entity_types`: list of entity types that this mob considers valid targets.
    pub entity_types: Option<EntityTypes>,
    /// `hurt_owner`: if true, this mob will also attack mobs that hurt its owner. Default `false`.
    #[facet(schema::fallback = false)]
    pub hurt_owner: Option<bool>,
    /// `must_reach`: if true, only entities that this mob can path to can be selected as targets. Default `false`.
    #[facet(schema::fallback = false)]
    pub must_reach: Option<bool>,
    /// `must_see`: if true, only entities in this mob's viewing range can be selected as targets. Default `false`.
    #[facet(schema::fallback = false)]
    pub must_see: Option<bool>,
    /// `must_see_forget_duration`: determines the amount of time in seconds that this mob will look for a target before forgetting about it and looking for a new one when the target isn't visible any more. Default 3; at least 0.
    #[facet(schema::fallback = 3.0, schema::min = 0.0)]
    pub must_see_forget_duration: Option<f32>,
    /// `persist_time`: time in seconds for a valid target to stay targeted when it becomes an invalid target. Default 0; at least 0.
    #[facet(schema::fallback = 0.0, schema::min = 0.0)]
    pub persist_time: Option<f32>,
    /// `reselect_targets`: if true, the target will change to the current closest entity whenever a different entity is closer. Default `false`.
    #[facet(schema::fallback = false)]
    pub reselect_targets: Option<bool>,
    /// `scan_interval`: how many ticks to wait between scanning for a target. Default 10.
    #[facet(schema::fallback = 10)]
    pub scan_interval: Option<i32>,
    /// `set_persistent`: allows the actor to be set to persist upon targeting a player. Default `false`.
    #[facet(schema::fallback = false)]
    pub set_persistent: Option<bool>,
    /// `target_search_height`: height in blocks to search for a target mob. -`1.0f` means the height does not matter. Default -1.
    #[facet(schema::fallback = -1.0)]
    pub target_search_height: Option<f32>,
    /// `within_radius`: distance in blocks that the target can be within to launch an attack. Default 0; at least 0.
    #[facet(schema::fallback = 0.0, schema::min = 0.0)]
    pub within_radius: Option<f32>,
}

impl Goal for NearestPrioritizedAttackableTarget {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    const DEPRECATED: &'static [Deprecated] = &[
        (
            "attack_owner",
            deprecated_text!("attack_owner"),
            known::V1_26_30,
        ),
        (
            "target_acquisition_probability",
            deprecated_text!("target_acquisition_probability"),
            known::V1_26_30,
        ),
        (
            "target_invisible_multiplier",
            deprecated_text!("target_invisible_multiplier"),
            known::V1_26_30,
        ),
        (
            "target_sneak_visibility_multiplier",
            deprecated_text!("target_sneak_visibility_multiplier"),
            known::V1_26_30,
        ),
    ];

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
        c.range("attack_interval", &mut self.attack_interval);
        c.entity_types("entity_types", self.entity_types.as_ref());
    }
}

/// `minecraft:behavior.owner_hurt_by_target`: allows the mob to target another mob that hurts their owner.
///
/// Deprecated members, reported and dropped: `attack_owner`, `must_reach`, `must_see`, `must_see_forget_duration`, `persist_time`, `within_radius`.
// `behavior.owner_hurt_by_target`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.owner_hurt_by_target")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct OwnerHurtByTarget {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `entity_types`: list of entity types that this mob can target if they hurt their owner.
    pub entity_types: Option<EntityTypes>,
}

impl Goal for OwnerHurtByTarget {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    const DEPRECATED: &'static [Deprecated] = &[
        (
            "attack_owner",
            deprecated_text!("attack_owner"),
            known::V1_26_30,
        ),
        (
            "must_reach",
            deprecated_text!("must_reach"),
            known::V1_26_30,
        ),
        ("must_see", deprecated_text!("must_see"), known::V1_26_30),
        (
            "must_see_forget_duration",
            deprecated_text!("must_see_forget_duration"),
            known::V1_26_30,
        ),
        (
            "persist_time",
            deprecated_text!("persist_time"),
            known::V1_26_30,
        ),
        (
            "within_radius",
            deprecated_text!("within_radius"),
            known::V1_26_30,
        ),
    ];

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
        c.entity_types("entity_types", self.entity_types.as_ref());
    }
}

/// `minecraft:behavior.owner_hurt_target`: allows the mob to target a mob that is hurt by their owner.
///
/// Deprecated members, reported and dropped: `attack_owner`, `must_reach`, `must_see`, `must_see_forget_duration`, `persist_time`, `within_radius`.
// `behavior.owner_hurt_target`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.owner_hurt_target")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct OwnerHurtTarget {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `entity_types`: list of entity types that this entity can target if the potential target is hurt by this mob's owner.
    pub entity_types: Option<EntityTypes>,
}

impl Goal for OwnerHurtTarget {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    const DEPRECATED: &'static [Deprecated] = &[
        (
            "attack_owner",
            deprecated_text!("attack_owner"),
            known::V1_26_30,
        ),
        (
            "must_reach",
            deprecated_text!("must_reach"),
            known::V1_26_30,
        ),
        ("must_see", deprecated_text!("must_see"), known::V1_26_30),
        (
            "must_see_forget_duration",
            deprecated_text!("must_see_forget_duration"),
            known::V1_26_30,
        ),
        (
            "persist_time",
            deprecated_text!("persist_time"),
            known::V1_26_30,
        ),
        (
            "within_radius",
            deprecated_text!("within_radius"),
            known::V1_26_30,
        ),
    ];

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
        c.entity_types("entity_types", self.entity_types.as_ref());
    }
}

/// `minecraft:behavior.target_when_pushed`: allows the mob to target another mob when it is pushed by that mob.
// `behavior.target_when_pushed`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.target_when_pushed")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct TargetWhenPushed {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `entity_types`: list of entity types that can trigger this mob to begin targeting.
    pub entity_types: Option<EntityTypes>,
    /// `percent_chance`: the probability that the mob will target another mob when pushed. A value of 100 is 100%. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub percent_chance: Option<f32>,
}

impl Goal for TargetWhenPushed {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
        c.entity_types("entity_types", self.entity_types.as_ref());
    }
}

/// `minecraft:behavior.wither_target_highest_damage`: allows the wither to focus its attacks on whichever mob has dealt the most damage to it.
///
/// Deprecated members, reported and dropped: `attack_owner`, `must_reach`, `must_see`, `must_see_forget_duration`, `persist_time`, `within_radius`.
// `behavior.wither_target_highest_damage`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.wither_target_highest_damage")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct WitherTargetHighestDamage {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `entity_types`: list of entity types the wither takes into account to find who dealt the most damage to it.
    pub entity_types: Option<EntityTypes>,
}

impl Goal for WitherTargetHighestDamage {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    const DEPRECATED: &'static [Deprecated] = &[
        (
            "attack_owner",
            deprecated_text!("attack_owner"),
            known::V1_26_30,
        ),
        (
            "must_reach",
            deprecated_text!("must_reach"),
            known::V1_26_30,
        ),
        ("must_see", deprecated_text!("must_see"), known::V1_26_30),
        (
            "must_see_forget_duration",
            deprecated_text!("must_see_forget_duration"),
            known::V1_26_30,
        ),
        (
            "persist_time",
            deprecated_text!("persist_time"),
            known::V1_26_30,
        ),
        (
            "within_radius",
            deprecated_text!("within_radius"),
            known::V1_26_30,
        ),
    ];

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
        c.entity_types("entity_types", self.entity_types.as_ref());
    }
}
