//! Loot conditions: what must hold for a pool, an entry or a function to apply.

use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::{EntityId, ItemId};
use bedrock_addon_core::shared::IntRange;

/// One condition.
///
/// JSON: `{"condition": name, …parameters}`. The name is matched exactly (no namespace is
/// stripped); vanilla drops a condition it does not know, so the pool, entry or function it
/// would have guarded is not gated by it.
#[derive(Clone, Debug, PartialEq)]
pub struct LootCondition {
    /// The condition (JSON `condition`) and its parameters.
    pub kind: ConditionKind,
    /// Members the condition does not read, kept for write-back.
    pub extra: UnknownMembers,
}

/// The conditions of vanilla's factory map, with their parameters (in the game's order).
///
/// Every parameter is optional; vanilla reads each with its default and checks none.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ConditionKind {
    /// `killed_by_player`.
    KilledByPlayer,
    /// `killed_by_player_or_pets`.
    KilledByPlayerOrPets,
    /// `killed_by_entity`: the killer is this entity.
    KilledByEntity {
        /// JSON `entity_type` (default `""`).
        entity_type: Option<EntityId>,
    },
    /// `entity_killed`: the killed entity is this one.
    EntityKilled {
        /// JSON `entity_type` (default `""`).
        entity_type: Option<EntityId>,
    },
    /// `random_chance`.
    RandomChance {
        /// JSON `chance` (default 0.0).
        chance: Option<f32>,
    },
    /// `random_difficulty_chance`: a chance per difficulty.
    RandomDifficultyChance {
        /// JSON `default_chance` (default 0.0): the chance of each difficulty left out.
        default_chance: Option<f32>,
        /// JSON `peaceful`.
        peaceful: Option<f32>,
        /// JSON `easy`.
        easy: Option<f32>,
        /// JSON `normal`.
        normal: Option<f32>,
        /// JSON `hard`.
        hard: Option<f32>,
    },
    /// `random_chance_with_looting`.
    RandomChanceWithLooting {
        /// JSON `chance` (default 0.0).
        chance: Option<f32>,
        /// JSON `looting_multiplier` (default 0.0): added per level of Looting.
        looting_multiplier: Option<f32>,
    },
    /// `random_regional_difficulty_chance`.
    RandomRegionalDifficultyChance {
        /// JSON `max_chance` (default 0.0).
        max_chance: Option<f32>,
    },
    /// `has_mark_variant`.
    HasMarkVariant {
        /// JSON `value` (default 0).
        value: Option<i32>,
    },
    /// `has_variant`.
    HasVariant {
        /// JSON `value` (default 0).
        value: Option<i32>,
    },
    /// `match_tool`: the tool used.
    MatchTool {
        /// JSON `item` (default `""`).
        item: Option<ItemId>,
        /// JSON `count` (default every count).
        count: Option<IntRange>,
        /// JSON `durability` (default every durability).
        durability: Option<IntRange>,
        /// JSON `enchantments`.
        enchantments: Option<Vec<ToolEnchantment>>,
        /// JSON `minecraft:match_tool_filter_any`: item tags, any of which the tool has.
        filter_any: Option<Vec<String>>,
        /// JSON `minecraft:match_tool_filter_all`: item tags, all of which the tool has.
        filter_all: Option<Vec<String>>,
        /// JSON `minecraft:match_tool_filter_none`: item tags, none of which the tool has.
        filter_none: Option<Vec<String>>,
    },
    /// `damaged_by_entity`.
    DamagedByEntity {
        /// JSON `entity_type` (default `""`).
        entity_type: Option<EntityId>,
    },
    /// `passenger_of_entity`.
    PassengerOfEntity {
        /// JSON `entity_type` (default `""`).
        entity_type: Option<EntityId>,
    },
    /// `is_baby`.
    IsBaby,
    /// `has_property`: the entity has this actor property.
    HasProperty {
        /// JSON `domain`: the property name (default `""`).
        domain: Option<String>,
    },
    /// `bool_property`.
    BoolProperty {
        /// JSON `domain`: the property name (default `""`).
        domain: Option<String>,
        /// JSON `value` (default `false`).
        value: Option<bool>,
    },
    /// `int_property`.
    IntProperty {
        /// JSON `domain`: the property name (default `""`).
        domain: Option<String>,
        /// JSON `value` (default 0).
        value: Option<i32>,
    },
    /// `float_property`.
    FloatProperty {
        /// JSON `domain`: the property name (default `""`).
        domain: Option<String>,
        /// JSON `value` (default 0.0).
        value: Option<f32>,
    },
    /// `enum_property`.
    EnumProperty {
        /// JSON `domain`: the property name (default `""`).
        domain: Option<String>,
        /// JSON `value` (default `""`).
        value: Option<String>,
    },
    /// `biome_has_tag` (engine 1.26.30): the current biome has this tag.
    BiomeHasTag {
        /// JSON `tag` (default `""`).
        tag: Option<String>,
    },
}

/// One enchantment `match_tool` requires.
///
/// JSON: `{"enchantment": string, "levels": IntRange}`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToolEnchantment {
    /// JSON `enchantment`.
    pub enchantment: Option<String>,
    /// JSON `levels`.
    pub levels: Option<IntRange>,
    /// Members vanilla does not read, kept for write-back.
    pub extra: UnknownMembers,
}
