//! Spawn groups: `SpawnGroup`, `Condition`.
//!
//! Raid spawn groups (`definitions/spawn_groups/*.json` of the app data, `minecraft:spawn_groups`).
//! Spawn groups use the 1.8.0 condition ([`Condition`]) with a larger distance cap.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;

mod legacy;
mod load;
mod validate;

pub use bedrock_addon_support::spawn::{
    AboveBlockFilter, Condition, DelayFilter, DensityLimit, Difficulty, DifficultyFilter, Herd,
    HerdInfo, MobEventFilter, MobSpawnRules, OneOrMany, Permutation, PopulationControl,
    RangeFilter, VillageFilter, Weight, WorldAgeFilter,
};
#[doc(hidden)]
pub use load::load_groups;

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::SpawnGroupId;
use bedrock_addon_core::version::FormatVersion;

/// A raid spawn group (`minecraft:spawn_groups`): the conditions the server spawns as one batch
/// when a raid wave names the group (`minecraft:raid_group_<wave>`).
///
/// JSON: `{"format_version": "1.11.0", "minecraft:spawn_groups": {"description": {"identifier"},
/// "conditions": [{…}]}}`, read like standard spawn rules except that the spawn-distance cap is
/// 1024 and nothing is warned about the root, the payload or the description. Spawn groups come
/// only from the app data (`definitions/spawn_groups/*.json`), never from a pack.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:spawn_groups")]
#[doc(alias = "spawn_groups")]
pub struct SpawnGroup {
    /// The document's `format_version`: selects the Molang version of the block descriptors'
    /// `tags`.
    pub format_version: FormatVersion,
    /// The group (JSON `description`).
    pub description: GroupDescription,
    /// The conditions (JSON `conditions`); a group without any is never registered.
    pub conditions: Option<Vec<Condition>>,
    /// Other members of `minecraft:spawn_groups` (kept, ignored).
    pub extra: Extra,
    /// Root members other than `format_version` and `minecraft:spawn_groups` (kept, ignored).
    pub root_extra: Extra,
}

/// `minecraft:spawn_groups/description`.
#[derive(Clone, Debug, PartialEq)]
pub struct GroupDescription {
    /// The group (JSON `identifier`, required), kept verbatim.
    pub identifier: SpawnGroupId,
    /// Read and kept; unused by spawn groups (JSON `population_control`).
    pub population_control: Option<String>,
    /// Other members (kept, ignored).
    pub extra: Extra,
}

impl SpawnGroup {
    /// What the runtime keeps of the group: one
    /// [`MobSpawnRules`] per condition, in document order, with the spawn-distance cap 1024. A
    /// condition's weight and biome filter are not part of it.
    pub fn spawn_rules(&self) -> Vec<MobSpawnRules> {
        self.conditions
            .iter()
            .flatten()
            .map(|c| c.rules_with(bedrock_addon_support::spawn::condition::SPAWN_GROUPS))
            .collect()
    }
}
