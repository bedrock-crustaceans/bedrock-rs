//! `minecraft:template_pool`: a template pool.

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::{FeatureId, ProcessorListId, StructureId, TemplatePoolId};
use bedrock_addon_core::json::StrictInt;

/// A template pool: a weighted list of structure templates (or features) a jigsaw block draws
/// from (`minecraft:template_pool`, in `worldgen/template_pools/`).
///
/// JSON: `{"format_version": "1.21.20", "minecraft:template_pool": {"description", "elements",
/// "fallback"?}}`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[doc(alias = "minecraft:template_pool")]
#[doc(alias = "JigsawStructureTemplatePool")]
pub struct TemplatePool {
    /// The pool's identifier (JSON `description`).
    pub description: PoolDescription,
    /// The weighted elements; may be empty (JSON `elements`).
    pub elements: Vec<TemplatePoolElement>,
    /// The pool used when no element fits (JSON `fallback`, default `minecraft:empty`).
    #[facet(default)]
    pub fallback: Option<TemplatePoolId>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:template_pool/description`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct PoolDescription {
    /// The pool's identifier, `namespace:name` (JSON `identifier`).
    pub identifier: TemplatePoolId,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One weighted entry of a template pool (`elements[]`).
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct TemplatePoolElement {
    /// What the entry places (JSON `element`).
    pub element: PoolElement,
    /// The entry's weight, 1 to 200 (JSON `weight`, default 1).
    #[facet(default)]
    pub weight: Option<StrictInt<u8>>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// What a template pool entry places, by its `element_type`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(tag = "element_type")]
#[repr(u8)]
#[non_exhaustive]
pub enum PoolElement {
    /// Nothing (JSON `minecraft:empty_pool_element`).
    #[facet(rename = "minecraft:empty_pool_element")]
    Empty {
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
    /// A structure template (JSON `minecraft:single_pool_element`).
    #[facet(rename = "minecraft:single_pool_element")]
    Single {
        /// The structure file, relative to the pack's `structures/` folder, at most 256 bytes
        /// (JSON `location`).
        location: StructureId,
        /// The processor list applied to the template (JSON `processors`, default `""`: none).
        #[facet(default, skip_serializing_if = Option::is_none)]
        processors: Option<ProcessorListId>,
        /// How the template follows the terrain (JSON `projection`, default `rigid`).
        #[facet(default, skip_serializing_if = Option::is_none)]
        projection: Option<Projection>,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
    /// A feature (JSON `minecraft:feature_pool_element`, engine 1.26.20).
    #[facet(rename = "minecraft:feature_pool_element")]
    Feature {
        /// The feature to place (JSON `feature`).
        feature: FeatureId,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
}

/// How a template follows the terrain (`projection`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, facet::Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum Projection {
    /// Placed as it is (JSON `rigid`), the default.
    #[default]
    Rigid,
    /// Follows the terrain (JSON `terrain_matching`).
    TerrainMatching,
}
