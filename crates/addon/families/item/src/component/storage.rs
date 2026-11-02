//! Bundles: `bundle_interaction`, `storage_item`, `storage_weight_limit`,
//! `storage_weight_modifier`.

use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::shared::ItemDescriptor;
use facet::Facet;

use super::visit_item_descriptor;
use bedrock_addon_core::document::Extra;

/// `minecraft:bundle_interaction`: the bundle user interface (needs `storage_item`).
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct BundleInteraction {
    /// The slots shown, 1–64 (JSON `num_viewable_slots`; default 12).
    #[facet(default = 12)]
    pub num_viewable_slots: i32,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for BundleInteraction {
    fn default() -> Self {
        Self {
            num_viewable_slots: 12,
            extra: Extra::default(),
        }
    }
}

/// `minecraft:storage_item`: the item stores other items (needs a stack size of 1).
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct StorageItem {
    /// The number of slots, at most 64 (JSON `max_slots`; default 64).
    #[facet(default = 64)]
    pub max_slots: i32,
    /// Other storage items may go in (JSON `allow_nested_storage_items`; default true).
    #[facet(default = true)]
    pub allow_nested_storage_items: bool,
    /// Items that may not go in (JSON `banned_items`).
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub banned_items: Vec<ItemDescriptor>,
    /// Only these items may go in; empty means any (JSON `allowed_items`).
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub allowed_items: Vec<ItemDescriptor>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for StorageItem {
    fn default() -> Self {
        Self {
            max_slots: 64,
            allow_nested_storage_items: true,
            banned_items: Vec::new(),
            allowed_items: Vec::new(),
            extra: Extra::default(),
        }
    }
}

impl MolangFields for StorageItem {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        for (key, list) in [
            ("banned_items", &mut self.banned_items),
            ("allowed_items", &mut self.allowed_items),
        ] {
            visitor.enter(key.into());
            for (i, item) in list.iter_mut().enumerate() {
                visitor.enter(i.into());
                visit_item_descriptor(item, visitor);
                visitor.leave();
            }
            visitor.leave();
        }
    }
}

/// `minecraft:storage_weight_limit`: the total weight a storage item holds (from 1.21.60).
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct StorageWeightLimit {
    /// The limit, at most 64 (JSON `max_weight_limit`; default 64).
    #[facet(default = 64)]
    pub max_weight_limit: i32,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for StorageWeightLimit {
    fn default() -> Self {
        Self {
            max_weight_limit: 64,
            extra: Extra::default(),
        }
    }
}

/// `minecraft:storage_weight_modifier`: the weight of this item inside a storage item (from
/// 1.21.60).
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct StorageWeightModifier {
    /// The weight, at least 0; 0 means it cannot be stored (JSON `weight_in_storage_item`;
    /// default 4).
    #[facet(default = 4)]
    pub weight_in_storage_item: i32,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for StorageWeightModifier {
    fn default() -> Self {
        Self {
            weight_in_storage_item: 4,
            extra: Extra::default(),
        }
    }
}
