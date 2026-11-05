//! Loot tables: `LootTable`, `Pool`, `Entry`, `LootFunction`, `LootCondition`.
//!
//! Loot tables: `loot_tables/**/*.json` of a behaviour pack. Vanilla parses them by hand
//! (the table, pool, entry, function and condition readers), lazily, and reads no
//! `format_version`: `LootTable` is one unversioned struct and loot is not a version
//! family.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub use condition::{ConditionKind, LootCondition, ToolEnchantment};
pub use function::{
    BannerPattern, BannerType, EnchantOption, FunctionKind, ItemColor, ItemData, ItemDataKind,
    LootFunction, SpecificEnchant, StewEffect,
};

mod condition;
mod function;
mod load;
mod new;
mod parse;
#[doc(hidden)]
pub mod texts;
mod write;

#[doc(hidden)]
pub use load::LOADER;

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::{ItemId, LootTablePath};

/// A loot table: the pools a drop, a chest or a gameplay event rolls.
///
/// JSON: `{"pools": [Pool]}`. Vanilla reads only `pools`; every other root member (a
/// `format_version`, the `type` of vanilla's own trade tables) is ignored and kept in
/// [`extra`](Self::extra). There is no version: every file loads with the same rules.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "loot_tables")]
pub struct LootTable {
    /// JSON `pools`: the pools, each rolled on its own. An object in their place has its member
    /// values read as the pools, as vanilla iterates it.
    pub pools: Option<Vec<Pool>>,
    /// Root members vanilla does not read, kept for write-back.
    pub extra: Extra,
}

/// One pool of a table: how often it rolls and what it picks from.
///
/// JSON: `{"rolls", "bonus_rolls"?, "conditions"?, "tiers"?, "entries"}`. A pool whose
/// conditions fail does not roll.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pool {
    /// JSON `rolls`: how many entries are picked; a number or `{min, max}`. Absent, the pool
    /// rolls 0 times ([`rolls`](Self::rolls)).
    pub rolls: Option<RandomValueBounds>,
    /// JSON `bonus_rolls`: extra rolls per point of luck; a number or `{min, max}`.
    pub bonus_rolls: Option<RandomValueBounds>,
    /// JSON `conditions`: all must hold for the pool to roll.
    pub conditions: Option<Vec<LootCondition>>,
    /// JSON `tiers`: picks by tier instead of by weight.
    pub tiers: Option<Tiers>,
    /// JSON `entries`: what the pool picks from. Vanilla requires an array; without one the
    /// pool is kept with no entries.
    pub entries: Option<Vec<Entry>>,
    /// Members vanilla does not read, kept for write-back.
    pub extra: Extra,
}

/// A pool's tier selection.
///
/// JSON: `{"initial_range": int = 0, "bonus_rolls": int = 0, "bonus_chance": float = 0.0}`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tiers {
    /// JSON `initial_range` (default 0, [`initial_range`](Self::initial_range)).
    pub initial_range: Option<i32>,
    /// JSON `bonus_rolls` (default 0, [`bonus_rolls`](Self::bonus_rolls)).
    pub bonus_rolls: Option<i32>,
    /// JSON `bonus_chance` (default 0.0, [`bonus_chance`](Self::bonus_chance)).
    pub bonus_chance: Option<f32>,
    /// Members vanilla does not read, kept for write-back.
    pub extra: Extra,
}

/// One entry of a pool.
///
/// JSON: `{"type": "item" | "loot_table" | "empty", "weight"?, "quality"?, "conditions"?,
/// "pools"?, …}`. An entry with another `type` is dropped. `weight` and `quality` are not
/// range-checked: vanilla accepts 0 and negative values.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    /// JSON `type` and the members it adds.
    pub kind: EntryKind,
    /// JSON `weight` (default 1, [`weight`](Self::weight)).
    pub weight: Option<i32>,
    /// JSON `quality` (default 0, [`quality`](Self::quality)).
    pub quality: Option<i32>,
    /// JSON `conditions`: all must hold for the entry to be picked.
    pub conditions: Option<Vec<LootCondition>>,
    /// JSON `pools`: an inline table nested in the entry (a sub-table), read with the rules of
    /// a table's root.
    pub pools: Option<Vec<Pool>>,
    /// Members vanilla does not read (vanilla's own `aux_val`, `auxVal`, `count`), kept for
    /// write-back.
    pub extra: Extra,
}

/// What an entry gives: the JSON `type` and its members.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum EntryKind {
    /// `"type": "item"`: an item stack.
    Item {
        /// JSON `name`: the item. A name that names no item keeps the entry (it still counts
        /// for the weights) and gives nothing.
        name: Option<ItemId>,
        /// JSON `functions`: applied to the stack in order.
        functions: Option<Vec<LootFunction>>,
    },
    /// `"type": "loot_table"`: the result of another table.
    LootTable {
        /// JSON `name`: the referenced table's path, as written. Everything up to and
        /// including the last `:` is not part of the path ([`Entry::table`]).
        name: Option<LootTablePath>,
    },
    /// `"type": "empty"`: nothing.
    Empty,
}

/// A float range a value is drawn from.
///
/// JSON: a number (both bounds) or `{"min": float = 0.0, "max": float = 0.0}`. The bounds are
/// floats and are not ordered: `{"min": 3, "max": 1}` is kept as written.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RandomValueBounds {
    /// A number: `min = max`.
    Exact(f32),
    /// `{"min", "max"}`; a bound left out is 0.0.
    Range {
        /// JSON `min` ([`RandomValueBounds::min`]).
        min: Option<f32>,
        /// JSON `max` ([`RandomValueBounds::max`]).
        max: Option<f32>,
    },
}
