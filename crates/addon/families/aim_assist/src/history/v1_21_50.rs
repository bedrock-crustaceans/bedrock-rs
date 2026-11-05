//! The aim-assist preset at `[1.21.50, 1.21.120)` and its step to 1.21.120.

use indexmap::IndexMap;

use super::super::ExclusionSettings;
use bedrock_addon_core::id::{AimAssistCategoryId, AimAssistPresetId, BlockId, EntityId, ItemId};
use bedrock_addon_core::schema::Schema;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx, Versioned};
use bedrock_addon_support::chain_file::SourceVersion;

/// An aim-assist preset as written below format 1.21.120:
/// [`super::super::AimAssistPreset`] with one flat `exclusion_list` in place of
/// `exclusion_settings`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct AimAssistPreset {
    /// JSON `identifier`.
    pub identifier: AimAssistPresetId,
    /// The block or entity identifiers aim assist never targets (JSON `exclusion_list`).
    pub exclusion_list: Option<Vec<String>>,
    /// JSON `liquid_targeting_list`.
    pub liquid_targeting_list: Option<Vec<ItemId>>,
    /// JSON `item_settings`.
    pub item_settings: Option<IndexMap<String, AimAssistCategoryId>>,
    /// JSON `default_item_settings`.
    pub default_item_settings: Option<AimAssistCategoryId>,
    /// JSON `hand_settings`.
    pub hand_settings: Option<AimAssistCategoryId>,
    /// The `format_version` the document was read at, 1.21.50 or later; the struct is
    /// written back at it (not a JSON member of the payload).
    // A skipped field still takes the JSON member of its name in facet rc.7: a NUL-prefixed
    // name no document has.
    #[facet(skip, rename = "\u{0}source_format_version")]
    pub source_format_version: SourceVersion,
}

impl Versioned for AimAssistPreset {
    const VERSION: FormatVersion = FormatVersion::new(1, 21, 50);
}

/// Every same-named member is copied; each entry of
/// `exclusion_list` becomes both a block and an entity exclusion, in order, duplicates kept, the
/// text as written; `block_tags` and `entity_type_families` are empty lists. A document without
/// `exclusion_list` gets four empty lists. The game consumes the list without a message.
impl Upgrade for AimAssistPreset {
    type Next = super::super::AimAssistPreset;

    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Self::Next {
        let list = self.exclusion_list.unwrap_or_default();
        super::super::AimAssistPreset {
            identifier: self.identifier,
            exclusion_settings: Some(ExclusionSettings {
                blocks: Some(list.iter().map(|id| BlockId::parse(id.as_str())).collect()),
                entities: Some(list.iter().map(|id| EntityId::parse(id.as_str())).collect()),
                block_tags: Some(Vec::new()),
                entity_type_families: Some(Vec::new()),
            }),
            liquid_targeting_list: self.liquid_targeting_list,
            item_settings: self.item_settings,
            default_item_settings: self.default_item_settings,
            hand_settings: self.hand_settings,
        }
    }
}
