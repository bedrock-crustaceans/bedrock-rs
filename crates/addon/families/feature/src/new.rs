//! Constructors: a description from an identifier, and feature values with every optional
//! member absent.

use super::{Description, TreeFeature};
use bedrock_addon_core::id::FeatureId;

impl Description {
    /// The description of the feature `identifier`.
    pub fn new(identifier: FeatureId) -> Self {
        Self { identifier }
    }
}

impl TreeFeature {
    /// A tree with this identifier and no parts yet.
    pub fn new(identifier: FeatureId) -> Self {
        Self {
            description: Description::new(identifier),
            base_block: None,
            base_cluster: None,
            may_grow_on: None,
            may_replace: None,
            may_grow_through: None,
            trunk: None,
            acacia_trunk: None,
            cherry_trunk: None,
            fallen_trunk: None,
            fancy_trunk: None,
            mangrove_trunk: None,
            mega_trunk: None,
            poplar_trunk: None,
            canopy: None,
            acacia_canopy: None,
            cherry_canopy: None,
            fancy_canopy: None,
            mangrove_canopy: None,
            mega_canopy: None,
            mega_pine_canopy: None,
            pine_canopy: None,
            roofed_canopy: None,
            spruce_canopy: None,
            random_spread_canopy: None,
            poplar_canopy: None,
            mangrove_roots: None,
        }
    }
}
