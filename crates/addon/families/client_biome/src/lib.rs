//! Client biomes: what a biome looks and sounds like on the client, from the resource packs.
//! `ClientBiome` is `biomes/*.json` (`minecraft:client_biome`), one biome per file, at the
//! newest format (1.21.130); older files are upgraded when they load, and `history` has their
//! shapes; `Beta` is the `"format_version": "beta"` document.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![recursion_limit = "512"]

pub mod component;
pub mod history;

mod color;
mod environment;
mod load;
mod new;
mod parse;
mod texts;
mod validate;
mod write;

pub use color::{Color255Rgb, Color255Rgba};
pub use environment::{
    BoolAttribute, BoolOperation, ColorAttribute, ColorOperation, EnvironmentAttributes,
    FloatAttribute, FloatOperation,
};

#[doc(hidden)]
pub use load::LOADER;
#[doc(hidden)]
pub use load::identifier_in;

use bedrock_addon_core::document::{ComponentMap, Extra};
use bedrock_addon_core::id::BiomeId;

use component::ClientBiomeComponent;

/// A client biome (`minecraft:client_biome`), at the newest format version.
///
/// JSON: `{"format_version": "1.21.130", "minecraft:client_biome": {"description":
/// {"identifier"}, "components": {…}}}`. A document of an older version is upgraded to this
/// shape when it loads: the components vanilla's own older files left to code are inserted.
/// The client applies it only to a biome the server names.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:client_biome")]
#[doc(alias = "client_biome.json")]
pub struct ClientBiome {
    /// The biome's name (JSON `description`).
    pub description: Description,
    /// The components (JSON `components`, may be empty).
    pub components: ComponentMap<ClientBiomeComponent>,
    /// Unknown members of `minecraft:client_biome`.
    pub extra: Extra,
}

/// `minecraft:client_biome/description`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct Description {
    /// The biome's identifier (JSON `identifier`): `namespace:name` in lower case from format
    /// 1.21.90 (matching `^[a-z0-9._%+-]+:[a-z0-9._%+-]+$`); an older document may name a
    /// vanilla biome without its namespace, which the upgrade adds.
    pub identifier: BiomeId,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// A beta client biome (`"format_version": "beta"`): the newest components plus
/// `environment_attributes`. Loaded only from a base-game pack or with the Beta APIs toggle,
/// and never upgraded.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:client_biome")]
pub struct Beta {
    /// The biome's name (JSON `description`).
    pub description: Description,
    /// The components (JSON `components`, may be empty).
    pub components: ComponentMap<ClientBiomeComponent>,
    /// Experimental environment attributes (JSON `environment_attributes`).
    pub environment_attributes: Option<EnvironmentAttributes>,
    /// Unknown members of `minecraft:client_biome`.
    pub extra: Extra,
}
