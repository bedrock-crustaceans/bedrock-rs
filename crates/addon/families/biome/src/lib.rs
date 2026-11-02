//! Biomes: `Biome`, `component::*`.
//!
//! Server biomes: `biomes/*.json` of a behaviour pack (`minecraft:biome`). `Biome` is the
//! newest document (format 1.26.30); older files are upgraded when they load, and `history` has
//! the older shapes; `Beta` is the `"format_version": "beta"` document.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![recursion_limit = "512"]

pub mod component;
mod environment;
pub mod history;
mod load;
mod new;
mod parse;
mod texts;
mod validate;
mod write;

pub use environment::{
    BoolAttribute, BoolOperation, ColorAttribute, ColorOperation, EnvironmentAttributes,
    FloatAttribute, FloatOperation,
};

use bedrock_addon_core::document::{ComponentMap, Extra};
use bedrock_addon_core::id::BiomeId;

use component::BiomeComponent;

#[doc(hidden)]
pub use load::load_registry;
// Every block a component names, for the link pass.
#[doc(hidden)]
pub use validate::blocks as block_references;

/// A server biome (`minecraft:biome`), at the newest format version.
///
/// JSON: `{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier"},
/// "components": {…}}}`. A document of an older version is upgraded to this shape when it
/// loads; the components a vanilla biome needs are inserted for overrides of vanilla biomes
/// older than 1.26.0.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:biome")]
pub struct Biome {
    /// The biome's name (JSON `description`).
    pub description: Description,
    /// The components (JSON `components`, may be empty).
    pub components: ComponentMap<BiomeComponent>,
    /// Unknown members of `minecraft:biome`.
    pub extra: Extra,
}

/// `minecraft:biome/description`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct Description {
    /// The biome's identifier, `namespace:name` in lower case (JSON `identifier`, matching
    /// `^[a-z0-9._%+-]+:[a-z0-9._%+-]+$`).
    pub identifier: BiomeId,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// A beta biome (`"format_version": "beta"`): the newest components plus
/// `environment_attributes`. Loaded only from a base-game pack or with the Beta APIs toggle,
/// and never upgraded.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:biome")]
pub struct Beta {
    /// The biome's name (JSON `description`).
    pub description: Description,
    /// The components (JSON `components`, may be empty).
    pub components: ComponentMap<BiomeComponent>,
    /// Experimental environment attributes (JSON `environment_attributes`).
    pub environment_attributes: Option<EnvironmentAttributes>,
    /// Unknown members of `minecraft:biome`.
    pub extra: Extra,
}
