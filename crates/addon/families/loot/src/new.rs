//! Constructors, names and the accessors that apply vanilla's defaults.

use super::{
    BannerType, ConditionKind, Entry, EntryKind, FunctionKind, ItemColor, ItemData, ItemDataKind,
    LootCondition, LootFunction, Pool, RandomValueBounds, SpecificEnchant, Tiers,
};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::LootTablePath;

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
            extra: Extra::default(),
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
            extra: Extra::default(),
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
            extra: Extra::default(),
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
            extra: Extra::default(),
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
