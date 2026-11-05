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
mod read;
#[doc(hidden)]
pub mod texts;
mod write;

use crate::read::parse_table;
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::document::unversioned;
use bedrock_addon_core::document::{
    Content, Document, Files, FiniteText, JsonContent, Origin, Rejected,
};
use bedrock_addon_core::id::{ItemId, LootTablePath};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{self, CollectionLoader, FileCopy, LoadContext, Walk};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};

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
    pub extra: UnknownMembers,
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
    pub extra: UnknownMembers,
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
    pub extra: UnknownMembers,
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
    pub extra: UnknownMembers,
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

// A hand `Content`: a loot table is an unversioned document (vanilla reads no
// `format_version`), so it has no family chain for `JsonDocument`.
impl bedrock_addon_core::__private::Sealed for LootTable {}

impl Content for LootTable {
    const NAME: &'static str = "loot table";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// Loads one loot table file.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = Origin::file(PackRef::app_data(), PackSide::Behavior, path.clone());
        // A number no double holds is read as the largest finite one, with a warning.
        let finite = FiniteText::of(bytes);
        let (table, mut diagnostics) = parse_table(path, finite.bytes(), cx);
        finite.settle(&mut diagnostics, None);
        diagnostics.locate(&origin);
        match table {
            // Vanilla reads no version: none is written, effective or parsed.
            Some(table) => Ok(Document::new(
                table,
                origin,
                unversioned::version(),
                diagnostics,
            )),
            None => Err(Rejected::new(origin, diagnostics)),
        }
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }
}

impl JsonContent for LootTable {}

// ---- the collection ----------------------------------------------------------------------

/// `loot_tables/**/*.json`, packs bottom-up.
const WALK: Walk = Walk::json_tree_in(&["loot_tables"]);

/// The loader of `bp.loot_tables()`: `loot_tables/**/*.json` of every behaviour pack, keyed by
/// path, the top pack's copy winning, and a copy that fails leaves the path without a table
/// (vanilla loads the one file the pack stack resolves the path to, and caches nothing when it
/// is invalid).
#[doc(hidden)]
pub const LOADER: CollectionLoader<Files<LootTable>, FileCopy<LootTable>> =
    load::files::<LootTable>(WALK);

impl Pool {
    /// The rolls: JSON `rolls`, 0..0 when absent.
    pub fn rolls(&self) -> RandomValueBounds {
        self.rolls.unwrap_or(RandomValueBounds::Exact(0.0))
    }

    /// The entries; none when JSON `entries` is absent.
    pub fn entries(&self) -> &[Entry] {
        self.entries.as_deref().unwrap_or_default()
    }
}

impl Tiers {
    /// JSON `initial_range`, 0 when absent.
    pub fn initial_range(&self) -> i32 {
        self.initial_range.unwrap_or(0)
    }

    /// JSON `bonus_rolls`, 0 when absent.
    pub fn bonus_rolls(&self) -> i32 {
        self.bonus_rolls.unwrap_or(0)
    }

    /// JSON `bonus_chance`, 0.0 when absent.
    pub fn bonus_chance(&self) -> f32 {
        self.bonus_chance.unwrap_or(0.0)
    }
}

impl Entry {
    /// An entry of `kind` with no other member.
    pub fn new(kind: EntryKind) -> Self {
        Self {
            kind,
            weight: None,
            quality: None,
            conditions: None,
            pools: None,
            extra: UnknownMembers::default(),
        }
    }

    /// JSON `weight`, 1 when absent.
    pub fn weight(&self) -> i32 {
        self.weight.unwrap_or(1)
    }

    /// JSON `quality`, 0 when absent.
    pub fn quality(&self) -> i32 {
        self.quality.unwrap_or(0)
    }

    /// The table a `loot_table` entry references: its `name` without everything up to and
    /// including the last `:` (`"minecraft:loot_tables/x.json"` references
    /// `loot_tables/x.json`). Vanilla compares the path without case when it looks it up.
    pub fn table(&self) -> Option<LootTablePath> {
        match &self.kind {
            EntryKind::LootTable { name: Some(name) } => Some(referenced(name)),
            _ => None,
        }
    }
}

/// `name` without everything up to and including its last `:`.
pub(crate) fn referenced(name: &LootTablePath) -> LootTablePath {
    // The game scans back for the last ':'.
    match name.as_str().rsplit_once(':') {
        Some((_, path)) => LootTablePath::parse(path.to_owned()),
        None => name.clone(),
    }
}

impl RandomValueBounds {
    /// Both bounds `min..max` (a `Range` with both written).
    pub fn new(min: f32, max: f32) -> Self {
        Self::Range {
            min: Some(min),
            max: Some(max),
        }
    }

    /// The lower bound: the number, or `min` (0.0 when absent).
    pub fn min(&self) -> f32 {
        match *self {
            Self::Exact(x) => x,
            Self::Range { min, .. } => min.unwrap_or(0.0),
        }
    }

    /// The upper bound: the number, or `max` (0.0 when absent).
    pub fn max(&self) -> f32 {
        match *self {
            Self::Exact(x) => x,
            Self::Range { max, .. } => max.unwrap_or(0.0),
        }
    }
}

impl LootFunction {
    /// The function `kind` without a namespace, conditions or other members.
    pub fn new(kind: FunctionKind) -> Self {
        Self {
            namespace: None,
            kind,
            conditions: None,
            extra: UnknownMembers::default(),
        }
    }

    /// The JSON `function` text: the namespace as written and the function's name.
    pub fn function_name(&self) -> String {
        format!(
            "{}{}",
            self.namespace.as_deref().unwrap_or_default(),
            self.kind.name()
        )
    }
}

impl FunctionKind {
    /// The function's name in vanilla's registry (`"set_count"`).
    pub fn name(&self) -> &'static str {
        match self {
            Self::SetCount { .. } => "set_count",
            Self::SetData(_) => "set_data",
            Self::SetDamage { .. } => "set_damage",
            Self::LootingEnchant { .. } => "looting_enchant",
            Self::EnchantWithLevels { .. } => "enchant_with_levels",
            Self::EnchantBookForTrading { .. } => "enchant_book_for_trading",
            Self::EnchantRandomly { .. } => "enchant_randomly",
            Self::FurnaceSmelt => "furnace_smelt",
            Self::SetDataFromColorIndex => "set_data_from_color_index",
            Self::EnchantRandomGear { .. } => "enchant_random_gear",
            Self::RandomAuxValue(_) => "random_aux_value",
            Self::RandomBlockState { .. } => "random_block_state",
            Self::RandomDye => "random_dye",
            Self::ExplorationMap { .. } => "exploration_map",
            Self::SetBannerDetails { .. } => "set_banner_details",
            Self::ExplosionDecay => "explosion_decay",
            Self::SetName { .. } => "set_name",
            Self::SetLore { .. } => "set_lore",
            Self::SpecificEnchants { .. } => "specific_enchants",
            Self::FillContainer { .. } => "fill_container",
            Self::SetActorId { .. } => "set_actor_id",
            Self::SetBookContents { .. } => "set_book_contents",
            Self::SetStewEffect { .. } => "set_stew_effect",
            Self::SetOminousBottleAmplifier { .. } => "set_ominous_bottle_amplifier",
            Self::SetArmorTrim { .. } => "set_armor_trim",
            Self::SetPotion { .. } => "set_potion",
            Self::CarryOverBlockEntityData { .. } => "carry_over_block_entity_data",
        }
    }
}

impl ItemData {
    /// The function type vanilla's parser stores: by `data`, else by `values`.
    pub fn kind(&self) -> ItemDataKind {
        match (&self.data, &self.values) {
            (Some(_), _) => ItemDataKind::SetItemData,
            (None, Some(_)) => ItemDataKind::RandomAuxValue,
            (None, None) => ItemDataKind::Unset,
        }
    }

    /// The range: `data`, else `values`, else 0..0.
    pub fn range(&self) -> RandomValueBounds {
        self.data
            .or(self.values)
            .unwrap_or(RandomValueBounds::Exact(0.0))
    }
}

impl BannerType {
    /// The JSON name (`"default"`, `"illager_captain"`).
    pub fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::IllagerCaptain => "illager_captain",
        }
    }
}

impl ItemColor {
    /// Every colour, by index.
    pub const ALL: [ItemColor; 16] = [
        Self::Black,
        Self::Red,
        Self::Green,
        Self::Brown,
        Self::Blue,
        Self::Purple,
        Self::Cyan,
        Self::Silver,
        Self::Gray,
        Self::Pink,
        Self::Lime,
        Self::Yellow,
        Self::LightBlue,
        Self::Magenta,
        Self::Orange,
        Self::White,
    ];

    /// The JSON name (`"light_blue"`).
    pub fn name(self) -> &'static str {
        match self {
            Self::Black => "black",
            Self::Red => "red",
            Self::Green => "green",
            Self::Brown => "brown",
            Self::Blue => "blue",
            Self::Purple => "purple",
            Self::Cyan => "cyan",
            Self::Silver => "silver",
            Self::Gray => "gray",
            Self::Pink => "pink",
            Self::Lime => "lime",
            Self::Yellow => "yellow",
            Self::LightBlue => "light_blue",
            Self::Magenta => "magenta",
            Self::Orange => "orange",
            Self::White => "white",
        }
    }

    /// The index 0..15.
    pub fn index(self) -> u8 {
        // `ALL` holds each colour once, in index order.
        Self::ALL
            .iter()
            .position(|c| *c == self)
            .and_then(|i| u8::try_from(i).ok())
            .unwrap_or(15)
    }

    /// The colour of a name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.name() == name)
    }
}

impl SpecificEnchant {
    /// The id at level 1 (the string form).
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            level: None,
            extra: UnknownMembers::default(),
        }
    }

    /// The levels: JSON `level`, 1..1 when absent.
    pub fn levels(&self) -> (i32, i32) {
        self.level.map_or((1, 1), |level| level.resolve(1, 1))
    }
}

impl LootCondition {
    /// The condition `kind` with no other member.
    pub fn new(kind: ConditionKind) -> Self {
        Self {
            kind,
            extra: UnknownMembers::default(),
        }
    }
}

impl ConditionKind {
    /// The condition's name in vanilla's factory map (`"random_chance"`).
    pub fn name(&self) -> &'static str {
        match self {
            Self::KilledByPlayer => "killed_by_player",
            Self::KilledByPlayerOrPets => "killed_by_player_or_pets",
            Self::KilledByEntity { .. } => "killed_by_entity",
            Self::EntityKilled { .. } => "entity_killed",
            Self::RandomChance { .. } => "random_chance",
            Self::RandomDifficultyChance { .. } => "random_difficulty_chance",
            Self::RandomChanceWithLooting { .. } => "random_chance_with_looting",
            Self::RandomRegionalDifficultyChance { .. } => "random_regional_difficulty_chance",
            Self::HasMarkVariant { .. } => "has_mark_variant",
            Self::HasVariant { .. } => "has_variant",
            Self::MatchTool { .. } => "match_tool",
            Self::DamagedByEntity { .. } => "damaged_by_entity",
            Self::PassengerOfEntity { .. } => "passenger_of_entity",
            Self::IsBaby => "is_baby",
            Self::HasProperty { .. } => "has_property",
            Self::BoolProperty { .. } => "bool_property",
            Self::IntProperty { .. } => "int_property",
            Self::FloatProperty { .. } => "float_property",
            Self::EnumProperty { .. } => "enum_property",
            Self::BiomeHasTag { .. } => "biome_has_tag",
        }
    }

    /// The chances of `random_difficulty_chance` for peaceful, easy, normal and hard: each one
    /// left out is `default_chance` (0.0 when that is absent too). `None` for another condition.
    pub fn difficulty_chances(&self) -> Option<[f32; 4]> {
        match *self {
            Self::RandomDifficultyChance {
                default_chance,
                peaceful,
                easy,
                normal,
                hard,
            } => {
                let default = default_chance.unwrap_or(0.0);
                Some([peaceful, easy, normal, hard].map(|c| c.unwrap_or(default)))
            }
            _ => None,
        }
    }
}
