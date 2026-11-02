//! Loot functions: what an item entry does to its stack.

use super::{LootCondition, RandomValueBounds};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::{EntityId, LootTablePath};
use bedrock_addon_core::shared::IntRange;

/// One function of an item entry.
///
/// JSON: `{"function": name, "conditions"?: [Condition], …parameters}`. Everything up to and
/// including the last `:` of `function` is not part of the name, so `"minecraft:set_count"`
/// and `"set_count"` are the same function; a name vanilla does not know drops the function.
#[derive(Clone, Debug, PartialEq)]
pub struct LootFunction {
    /// The text of JSON `function` up to and including its last `:` (`"minecraft:"`), kept for
    /// write-back; `None` when the name has no `:`.
    pub namespace: Option<String>,
    /// The function (the rest of JSON `function`) and its parameters.
    pub kind: FunctionKind,
    /// JSON `conditions`: all must hold for the function to apply.
    pub conditions: Option<Vec<LootCondition>>,
    /// Members the function does not read (vanilla's own `set_count.add`,
    /// `looting_enchant.limit`), kept for write-back.
    pub extra: Extra,
}

/// The functions of vanilla's registry, with their parameters (in the game's order).
///
/// Every parameter is optional; the default vanilla applies is on the accessor or in the field's
/// documentation.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum FunctionKind {
    /// `set_count`: the stack size.
    SetCount {
        /// JSON `count` (default 0..0).
        count: Option<RandomValueBounds>,
    },
    /// `set_data`: the item's data (aux) value.
    SetData(ItemData),
    /// `set_damage`: the damage, as a fraction of the durability.
    SetDamage {
        /// JSON `damage` (default 0..0).
        damage: Option<RandomValueBounds>,
    },
    /// `looting_enchant`: more items per level of Looting.
    LootingEnchant {
        /// JSON `count`: added per level (default 0..0).
        count: Option<RandomValueBounds>,
    },
    /// `enchant_with_levels`: enchants as an enchanting table would at these levels.
    EnchantWithLevels {
        /// JSON `levels` (default 0..0).
        levels: Option<RandomValueBounds>,
        /// JSON `treasure`: allows treasure enchantments (default `false`).
        treasure: Option<bool>,
    },
    /// `enchant_book_for_trading`: a librarian's enchanted book and its price.
    EnchantBookForTrading {
        /// JSON `base_cost` (default 2).
        base_cost: Option<i32>,
        /// JSON `per_level_cost` (default 3).
        per_level_cost: Option<i32>,
        /// JSON `base_random_cost` (default 5).
        base_random_cost: Option<i32>,
        /// JSON `per_level_random_cost` (default 10).
        per_level_random_cost: Option<i32>,
        /// JSON `enchantments`: the enchantments to pick from; an option that names no
        /// enchantment is dropped.
        enchantments: Option<Vec<EnchantOption>>,
    },
    /// `enchant_randomly`: one random enchantment.
    EnchantRandomly {
        /// JSON `treasure` (default `false`).
        treasure: Option<bool>,
    },
    /// `furnace_smelt`: the smelted item instead.
    FurnaceSmelt,
    /// `set_data_from_color_index`: the data value from the looted entity's colour.
    SetDataFromColorIndex,
    /// `enchant_random_gear`: enchants equipment by the regional difficulty.
    EnchantRandomGear {
        /// JSON `chance` (default 0.0).
        chance: Option<f32>,
    },
    /// `random_aux_value`: the item's data value, by the same parser as `set_data`.
    RandomAuxValue(ItemData),
    /// `random_block_state`: a random value of a block state.
    RandomBlockState {
        /// JSON `block_state`: the state's name (default `""`).
        block_state: Option<String>,
        /// JSON `values` (default 0..0).
        values: Option<RandomValueBounds>,
    },
    /// `random_dye`: a random dye colour.
    RandomDye,
    /// `exploration_map`: a map to a structure.
    ExplorationMap {
        /// JSON `destination`: the structure (default `""`; not checked when loading).
        destination: Option<String>,
    },
    /// `set_banner_details`: a banner's type, base colour and patterns.
    SetBannerDetails {
        /// JSON `type`: `"default"`, `"illager_captain"` or 0..1 (default `Default`).
        banner_type: Option<BannerType>,
        /// JSON `base_color`: a colour name or 0..15 (default `White`).
        base_color: Option<ItemColor>,
        /// JSON `patterns`: at most six patterns, only on a default banner. A single object is
        /// read as one pattern.
        patterns: Option<Vec<BannerPattern>>,
    },
    /// `explosion_decay`: items lost to an explosion.
    ExplosionDecay,
    /// `set_name`: the item's custom name.
    SetName {
        /// JSON `name` (default `""`).
        name: Option<String>,
    },
    /// `set_lore`: the item's lore lines.
    SetLore {
        /// JSON `lore`: an array of strings or a single string (one line).
        lore: Option<Vec<String>>,
    },
    /// `specific_enchants`: these enchantments.
    SpecificEnchants {
        /// JSON `enchants`: an array (or a single value) of enchantment ids or `{id, level}`
        /// objects.
        enchants: Option<Vec<SpecificEnchant>>,
    },
    /// `fill_container`: fills the container item from another table.
    FillContainer {
        /// JSON `loot_table`: the table's path (default `""`; resolved when used).
        loot_table: Option<LootTablePath>,
    },
    /// `set_actor_id`: the entity of a spawn egg.
    SetActorId {
        /// JSON `id` (default `""`).
        id: Option<EntityId>,
    },
    /// `set_book_contents`: a written book. Vanilla keeps at most 16 bytes of the title and
    /// 256 bytes of each page.
    SetBookContents {
        /// JSON `author` (default `""`).
        author: Option<String>,
        /// JSON `title` (default `""`).
        title: Option<String>,
        /// JSON `pages`.
        pages: Option<Vec<String>>,
    },
    /// `set_stew_effect`: a suspicious stew's effects.
    SetStewEffect {
        /// JSON `effects`: an array of `{id}` objects or a single one.
        effects: Option<Vec<StewEffect>>,
    },
    /// `set_ominous_bottle_amplifier`: an ominous bottle's amplifier.
    SetOminousBottleAmplifier {
        /// JSON `amplifier` (default 0..0).
        amplifier: Option<RandomValueBounds>,
    },
    /// `set_armor_trim`: an armour trim.
    SetArmorTrim {
        /// JSON `material` (default `""`; not checked when loading).
        material: Option<String>,
        /// JSON `pattern` (default `""`; not checked when loading).
        pattern: Option<String>,
    },
    /// `set_potion`: the potion type.
    SetPotion {
        /// JSON `id` (default `""`; not checked when loading).
        id: Option<String>,
    },
    /// `carry_over_block_entity_data` (engine 1.26.40, Upcoming Creator Features): copies the
    /// looted block entity's data to the stack.
    CarryOverBlockEntityData {
        /// JSON `dynamic_properties`: also copy the dynamic properties (default `false`).
        dynamic_properties: Option<bool>,
    },
}

/// The parameters of `set_data` and `random_aux_value`, which share vanilla's parser.
///
/// JSON `data` is read first, then `values`; with neither the range is 0..0 and vanilla warns.
/// Which key was used decides how vanilla applies the value ([`ItemData::kind`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemData {
    /// JSON `data`.
    pub data: Option<RandomValueBounds>,
    /// JSON `values`, read when `data` is absent.
    pub values: Option<RandomValueBounds>,
}

/// How vanilla applies an [`ItemData`]: the function type its parser stores.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ItemDataKind {
    /// `data` was given: sets the data value.
    SetItemData,
    /// `values` was given: a random aux value.
    RandomAuxValue,
    /// Neither: vanilla's sentinel type, range 0..0.
    Unset,
}

/// One enchantment a librarian's book may get.
///
/// JSON: `{"name": string, "min"?: int, "max"?: int}`. `min` and `max` default to the
/// enchantment's own lowest and highest level; vanilla clamps them into that range.
#[derive(Clone, Debug, PartialEq)]
pub struct EnchantOption {
    /// JSON `name`: the enchantment.
    pub name: String,
    /// JSON `min` (read only when an integer).
    pub min: Option<i32>,
    /// JSON `max` (read only when an integer).
    pub max: Option<i32>,
    /// Members vanilla does not read, kept for write-back.
    pub extra: Extra,
}

/// A banner's type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BannerType {
    /// `"default"` or 0.
    #[default]
    Default,
    /// `"illager_captain"` or 1: the ominous banner.
    IllagerCaptain,
}

/// One of the 16 item colours: a name or its index 0..15.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ItemColor {
    /// `"black"` or 0.
    Black,
    /// `"red"` or 1.
    Red,
    /// `"green"` or 2.
    Green,
    /// `"brown"` or 3.
    Brown,
    /// `"blue"` or 4.
    Blue,
    /// `"purple"` or 5.
    Purple,
    /// `"cyan"` or 6.
    Cyan,
    /// `"silver"` or 7.
    Silver,
    /// `"gray"` or 8.
    Gray,
    /// `"pink"` or 9.
    Pink,
    /// `"lime"` or 10.
    Lime,
    /// `"yellow"` or 11.
    Yellow,
    /// `"light_blue"` or 12.
    LightBlue,
    /// `"magenta"` or 13.
    Magenta,
    /// `"orange"` or 14.
    Orange,
    /// `"white"` or 15.
    #[default]
    White,
}

/// One banner pattern.
///
/// JSON: `{"pattern": string, "color"?: colour}`; a colour that is not one is white.
#[derive(Clone, Debug, PartialEq)]
pub struct BannerPattern {
    /// JSON `pattern`: the pattern's name.
    pub pattern: String,
    /// JSON `color` (default `White`).
    pub color: Option<ItemColor>,
    /// Members vanilla does not read, kept for write-back.
    pub extra: Extra,
}

/// One enchantment of `specific_enchants`.
///
/// JSON: an enchantment id (level 1) or `{"id": string, "level"?: IntRange = 1}`. An id that
/// names no enchantment is dropped.
#[derive(Clone, Debug, PartialEq)]
pub struct SpecificEnchant {
    /// The id (JSON the string, or `id`).
    pub id: String,
    /// JSON `level` (default 1); `None` for the string form.
    pub level: Option<IntRange>,
    /// Members vanilla does not read, kept for write-back.
    pub extra: Extra,
}

/// One effect of `set_stew_effect`.
///
/// JSON: `{"id": int = 0}`; the id is not checked when loading.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StewEffect {
    /// JSON `id`.
    pub id: Option<i32>,
    /// Members vanilla does not read, kept for write-back.
    pub extra: Extra,
}

#[cfg(test)]
mod hash_tests {
    use super::{BannerType, ItemDataKind};
    use std::hash::Hash;

    fn assert_hash<T: Hash + Eq>() {}

    #[test]
    fn item_data_kind_and_banner_type_hash() {
        assert_hash::<ItemDataKind>();
        assert_hash::<BannerType>();
    }
}
