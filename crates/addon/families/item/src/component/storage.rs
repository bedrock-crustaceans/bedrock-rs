//! Bundles: `bundle_interaction`, `storage_item`, `storage_weight_limit`,
//! `storage_weight_modifier`.

use bedrock_addon_shared_shapes::ItemDescriptor;
use facet::Facet;

use super::visit_item_descriptors;
use bedrock_addon_core::schema::{self, Schema};

/// `minecraft:bundle_interaction`: the bundle user interface (needs `storage_item`).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct BundleInteraction {
    /// The slots shown, 1–64 (JSON `num_viewable_slots`; default 12).
    #[facet(default = 12, schema::min = 1, schema::max = 64)]
    pub num_viewable_slots: i32,
}

/// `minecraft:storage_item`: the item stores other items (needs a stack size of 1).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct StorageItem {
    /// The number of slots, at most 64 (JSON `max_slots`; default 64).
    #[facet(default = 64)]
    pub max_slots: i32,
    /// Other storage items may go in (JSON `allow_nested_storage_items`; default true).
    #[facet(default = true)]
    pub allow_nested_storage_items: bool,
    /// Items that may not go in (JSON `banned_items`).
    #[facet(default, skip_serializing_if = Vec::is_empty, schema::molang_with = visit_item_descriptors)]
    pub banned_items: Vec<ItemDescriptor>,
    /// Only these items may go in; empty means any (JSON `allowed_items`).
    #[facet(default, skip_serializing_if = Vec::is_empty, schema::molang_with = visit_item_descriptors)]
    pub allowed_items: Vec<ItemDescriptor>,
}

/// `minecraft:storage_weight_limit`: the total weight a storage item holds (from 1.21.60).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct StorageWeightLimit {
    /// The limit, at most 64 (JSON `max_weight_limit`; default 64).
    #[facet(default = 64, schema::max = 64)]
    pub max_weight_limit: i32,
}

/// `minecraft:storage_weight_modifier`: the weight of this item inside a storage item (from
/// 1.21.60).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct StorageWeightModifier {
    /// The weight, at least 0; 0 means it cannot be stored (JSON `weight_in_storage_item`;
    /// default 4).
    #[facet(default = 4, schema::min = 0)]
    pub weight_in_storage_item: i32,
}
