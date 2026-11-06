//! What the two biome families share, defined once: `minecraft:biome` (behaviour packs) and
//! `minecraft:client_biome` (resource packs) are different documents that the game reads with
//! different rules, but their payload has the same frame.
//!
//! - [`environment`]: the beta document's `environment_attributes`, [`EnvironmentAttributes`],
//!   with its three attribute kinds and their operations.
//! - [`shell`]: the `{description, components}` payload object: one component's read re-rooted
//!   at its key ([`read_component`]), and the writers of the payload ([`payload_object`],
//!   [`beta_object`], [`write_components`]), generic over the family's description and
//!   component set.
//!
//! Each family keeps its own `Description`, the Facet shells it reads the payload through (see
//! [`shell`] for why they are not here), its component enum and the loop that reads
//! `components` by its game rules.

pub mod environment;
pub mod shell;

pub use environment::{
    BoolAttribute, BoolOperation, ColorAttribute, ColorOperation, EnvironmentAttributes,
    FloatAttribute, FloatOperation,
};
pub use shell::{beta_object, payload_object, read_component, write_components};
