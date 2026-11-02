//! Spawn rules: `SpawnRules`, `StandardSpawnRules`, `SimplifiedSpawnRules`, `Condition`.
//!
//! Spawn rules (`spawn_rules/**/*.json` of a behaviour pack, `minecraft:spawn_rules`). A
//! spawn-rules file is read in one of two shapes, never upgraded from one to the other: the 1.8.0
//! shape ([`StandardSpawnRules`]) always, the 1.21.60 shape ([`SimplifiedSpawnRules`]) when the
//! Simplified Spawn Rules experiment is on and the file is 1.21.60 or newer.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;

mod legacy;
mod load;
mod simplified;
mod texts;
mod validate;

pub use bedrock_addon_support::spawn::{
    AboveBlockFilter, Condition, DelayFilter, DensityLimit, Difficulty, DifficultyFilter, Herd,
    HerdInfo, MobEventFilter, MobSpawnRules, OneOrMany, Permutation, PopulationControl,
    RangeFilter, VillageFilter, Weight, WorldAgeFilter,
};
#[doc(hidden)]
pub use load::load_rules;
pub use simplified::{
    BiomeConditions, PermuteType, Placement, SimplifiedDescription, SimplifiedHerd,
    SimplifiedSpawnRules,
};

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::SpawnRulesId;
use bedrock_addon_core::version::FormatVersion;

/// How one entity spawns (`minecraft:spawn_rules`): the rules in the shape the file was read in.
///
/// A file is read as [`SpawnRules::Simplified`] when the Simplified Spawn Rules experiment is on
/// and its `format_version` is 1.21.60 or newer, and as [`SpawnRules::Standard`] otherwise; one
/// shape is never converted into the other.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:spawn_rules")]
#[doc(alias = "spawn_rules")]
pub enum SpawnRules {
    /// The 1.8.0 shape.
    Standard(StandardSpawnRules),
    /// The 1.21.60 shape (Simplified Spawn Rules experiment).
    Simplified(SimplifiedSpawnRules),
}

impl SpawnRules {
    /// The entity the rules are for (`description.identifier`).
    pub fn identifier(&self) -> &SpawnRulesId {
        match self {
            Self::Standard(rules) => &rules.description.identifier,
            Self::Simplified(rules) => &rules.description.identifier,
        }
    }
}

/// Spawn rules in the 1.8.0 shape: a spawn pool and a list of conditions, each a weight and the
/// filters and herds of one way the entity spawns.
///
/// JSON: `{"format_version": "1.8.0", "minecraft:spawn_rules": {"description": {"identifier",
/// "population_control"}, "conditions": [{"minecraft:weight": …, …}]}}`. Vanilla reads the whole
/// file with a hand-written parser: the envelope is not checked, a member of the wrong JSON type
/// takes the value vanilla's accessors give it, and `format_version` only selects the Molang
/// version of block descriptor `tags`. Spawn rules whose pool is missing or unknown load but are
/// not registered.
#[derive(Clone, Debug, PartialEq)]
pub struct StandardSpawnRules {
    /// The document's `format_version` (JSON `format_version`; `None` when absent, which reads
    /// as 1.8.0 with a warning): selects the Molang version of the block descriptors' `tags`.
    pub format_version: Option<FormatVersion>,
    /// The entity and its pool (JSON `description`).
    pub description: StandardDescription,
    /// The conditions (JSON `conditions`).
    pub conditions: Option<Vec<Condition>>,
    /// Unknown members of `minecraft:spawn_rules`.
    pub extra: Extra,
    /// Root members other than `format_version` and `minecraft:spawn_rules` (vanilla ignores
    /// them; each is an `UnrecognizedField` warning).
    pub root_extra: Extra,
}

/// `minecraft:spawn_rules/description` of standard spawn rules.
#[derive(Clone, Debug, PartialEq)]
pub struct StandardDescription {
    /// The entity (JSON `identifier`, required).
    pub identifier: SpawnRulesId,
    /// The spawn pool's name (JSON `population_control`); see
    /// [`StandardDescription::population_control`].
    pub population_control: Option<String>,
    /// Unknown members.
    pub extra: Extra,
}

impl StandardSpawnRules {
    /// The version the document is read at: its `format_version`, 1.8.0 when absent.
    pub fn effective_version(&self) -> FormatVersion {
        self.format_version.unwrap_or(FormatVersion::new(1, 8, 0))
    }
}

impl StandardDescription {
    /// The spawn pool: `population_control` after lower-casing, `None` when it is missing,
    /// empty or unknown (then vanilla does not register the rules).
    pub fn population_control(&self) -> Option<PopulationControl> {
        self.population_control
            .as_deref()
            .and_then(PopulationControl::from_name)
    }
}
