//! `minecraft:template_pool`: a template pool.

use super::validate::validate_detached;
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::id::{FeatureId, ProcessorListId, StructureId, TemplatePoolId};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::schema::{self, Schema};

/// A template pool: a weighted list of structure templates (or features) a jigsaw block draws
/// from (`minecraft:template_pool`, in `worldgen/template_pools/`).
///
/// JSON: `{"format_version": "1.21.20", "minecraft:template_pool": {"description", "elements",
/// "fallback"?}}`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[doc(alias = "minecraft:template_pool")]
#[doc(alias = "JigsawStructureTemplatePool")]
#[facet(schema::unknown = "keep")]
pub struct TemplatePool {
    /// The pool's identifier (JSON `description`).
    pub description: PoolDescription,
    /// The weighted elements; may be empty (JSON `elements`).
    pub elements: Vec<TemplatePoolElement>,
    /// The pool used when no element fits (JSON `fallback`, default `minecraft:empty`).
    pub fallback: Option<TemplatePoolId>,
}

/// `minecraft:template_pool/description`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct PoolDescription {
    /// The pool's identifier, `namespace:name` (JSON `identifier`).
    pub identifier: TemplatePoolId,
}

/// One weighted entry of a template pool (`elements[]`).
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct TemplatePoolElement {
    /// What the entry places (JSON `element`).
    pub element: PoolElement,
    /// The entry's weight, 1 to 200 (JSON `weight`, default 1: [`TemplatePoolElement::weight`]).
    #[facet(schema::fallback = 1, schema::min = 1, schema::max = 200)]
    pub weight: Option<u8>,
}

/// What a template pool entry places, by its `element_type`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(tag = "element_type")]
#[repr(u8)]
#[non_exhaustive]
pub enum PoolElement {
    /// Nothing (JSON `minecraft:empty_pool_element`).
    #[facet(rename = "minecraft:empty_pool_element")]
    #[facet(schema::unknown = "keep")]
    Empty {},
    /// A structure template (JSON `minecraft:single_pool_element`).
    #[facet(rename = "minecraft:single_pool_element")]
    #[facet(schema::unknown = "keep")]
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
    },
    /// A feature (JSON `minecraft:feature_pool_element`, engine 1.26.20).
    #[facet(rename = "minecraft:feature_pool_element")]
    #[facet(schema::unknown = "keep")]
    Feature {
        /// The feature to place (JSON `feature`).
        feature: FeatureId,
    },
}

/// How a template follows the terrain (`projection`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, facet::Facet, Schema)]
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

/// The template pool `fallback` defaults to.
const EMPTY_POOL: TemplatePoolId = TemplatePoolId::from_static("minecraft:empty");

impl TemplatePool {
    /// A pool without elements, falling back to `minecraft:empty`.
    pub fn new(identifier: TemplatePoolId) -> TemplatePool {
        TemplatePool {
            description: PoolDescription { identifier },
            elements: Vec::new(),
            fallback: None,
        }
    }

    /// The fallback pool; `minecraft:empty` when absent.
    pub fn fallback(&self) -> TemplatePoolId {
        self.fallback.clone().unwrap_or(EMPTY_POOL)
    }

    /// Runs the family's parse constraints and registration checks as if the pool were loaded
    /// from a behaviour pack with `cx`. The pool is not changed.
    pub fn validate(&self, cx: &LoadContext) -> Diagnostics {
        validate_detached(self, cx)
    }
}

impl TemplatePoolElement {
    /// An entry placing `element` with the default weight.
    pub fn new(element: PoolElement) -> TemplatePoolElement {
        TemplatePoolElement {
            element,
            weight: None,
        }
    }
}

impl PoolElement {
    /// The processor list of a single element; `None` for no processors (absent or `""`) and
    /// for the other element types.
    pub fn processors(&self) -> Option<&ProcessorListId> {
        match self {
            PoolElement::Single {
                processors: Some(list),
                ..
            } if !list.as_str().is_empty() => Some(list),
            _ => None,
        }
    }

    /// The projection of a single element (`rigid` when absent); `None` for the other element
    /// types.
    pub fn projection(&self) -> Option<Projection> {
        match self {
            PoolElement::Single { projection, .. } => Some(projection.unwrap_or_default()),
            _ => None,
        }
    }
}

// The data enums walk nothing: their bounds and members are checked by hand (`read.rs`).
bedrock_addon_core::impl_schema_leaf!(PoolElement);
bedrock_addon_core::impl_no_molang!(PoolElement);
