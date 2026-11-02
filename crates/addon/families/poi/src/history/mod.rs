//! The POI documents as written, and their chains.
//!
//! Both documents (`minecraft:poi_block`, `minecraft:poi_tag`) have one struct each, at 1.26.40,
//! read from 1.26.40 up to the engine version: no older shape exists and there is no upgrade
//! step.

use super::{PoiTag, PoiType};

/// The `poi_block` family of the version-chain machinery (hidden: only the loaders name it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct PoiTypeFamily;

/// The `poi_tag` family.
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct PoiTagFamily;

bedrock_addon_core::version_chain! {
    family = PoiTypeFamily;
    /// A `minecraft:poi_block` document parsed by the struct its `format_version` selects.
    #[derive(Clone, Debug, PartialEq)]
    pub enum PoiTypeAny {
        /// Format 1.26.40 and later: the newest document.
        V1_26_40(PoiType),
    }
    molang = none;
}

bedrock_addon_core::version_chain! {
    family = PoiTagFamily;
    /// A `minecraft:poi_tag` document parsed by the struct its `format_version` selects.
    #[derive(Clone, Debug, PartialEq)]
    pub enum PoiTagAny {
        /// Format 1.26.40 and later: the newest document.
        V1_26_40(PoiTag),
    }
    molang = none;
}
