//! `minecraft:aim_assist_preset`: an aim-assist preset.

use indexmap::IndexMap;

use bedrock_addon_core::id::{AimAssistCategoryId, AimAssistPresetId, BlockId, EntityId, ItemId};
use bedrock_addon_core::json::StrictString;

/// The category an item without an `item_settings` entry uses when a preset names none.
pub(crate) const DEFAULT_CATEGORY: &str = "minecraft:default";
/// The category an empty hand uses when a preset names none.
pub(crate) const EMPTY_HAND_CATEGORY: &str = "minecraft:empty_hand";

/// An aim-assist preset (`minecraft:aim_assist_preset`, in `aim_assist/presets/`), at the
/// newest format version (1.21.120): which targets aim assist leaves out and which category of
/// priorities each held item uses.
///
/// JSON: `{"format_version": "1.21.120", "minecraft:aim_assist_preset": {"identifier":
/// "ns:name", "exclusion_settings"?: {…}, "liquid_targeting_list"?: [item, …],
/// "item_settings"?: {item: category, …}, "default_item_settings"?: category,
/// "hand_settings"?: category}}`. Only `identifier` is required; a member the document leaves
/// out is not written back. Any unknown member or wrong type rejects the file.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[doc(alias = "minecraft:aim_assist_preset")]
#[doc(alias = "aim_assist_preset")]
pub struct AimAssistPreset {
    /// The preset's name, `namespace:name` (JSON `identifier`).
    pub identifier: AimAssistPresetId,
    /// What aim assist never targets (JSON `exclusion_settings`).
    pub exclusion_settings: Option<ExclusionSettings>,
    /// The items that target liquid blocks while held (JSON `liquid_targeting_list`).
    pub liquid_targeting_list: Option<Vec<ItemId>>,
    /// The category each held item uses, by item identifier (JSON `item_settings`).
    pub item_settings: Option<IndexMap<String, AimAssistCategoryId>>,
    /// The category of an item not in `item_settings` (JSON `default_item_settings`; see
    /// [`AimAssistPreset::default_item_settings`]).
    pub default_item_settings: Option<AimAssistCategoryId>,
    /// The category of an empty hand (JSON `hand_settings`; see
    /// [`AimAssistPreset::hand_settings`]).
    pub hand_settings: Option<AimAssistCategoryId>,
}

impl AimAssistPreset {
    /// [`default_item_settings`](Self::default_item_settings), `minecraft:default` when the
    /// document leaves it out (what the game uses at run time).
    #[must_use]
    pub fn default_item_settings(&self) -> AimAssistCategoryId {
        self.default_item_settings
            .clone()
            .unwrap_or_else(|| AimAssistCategoryId::parse(DEFAULT_CATEGORY))
    }

    /// [`hand_settings`](Self::hand_settings), `minecraft:empty_hand` when the document leaves
    /// it out (what the game uses at run time).
    #[must_use]
    pub fn hand_settings(&self) -> AimAssistCategoryId {
        self.hand_settings
            .clone()
            .unwrap_or_else(|| AimAssistCategoryId::parse(EMPTY_HAND_CATEGORY))
    }
}

/// What aim assist never targets (`exclusion_settings`); every list is optional.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
pub struct ExclusionSettings {
    /// Blocks (JSON `blocks`).
    pub blocks: Option<Vec<BlockId>>,
    /// Entities; an entry may carry a spawn event, `ns:name<ns:event>` (JSON `entities`).
    pub entities: Option<Vec<EntityId>>,
    /// Block tags (JSON `block_tags`).
    pub block_tags: Option<Vec<StrictString>>,
    /// Entity type families (JSON `entity_type_families`).
    pub entity_type_families: Option<Vec<StrictString>>,
}
