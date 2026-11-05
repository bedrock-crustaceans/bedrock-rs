//! The small value shapes several content families share, defined once: each is a type with
//! no rules of its own beyond reading and writing its JSON forms, and the families compose it.
//! A shape depends on `bedrock_addon_core` only, never on a family.
//!
//! `bedrock_addon::shared` re-exports every item here beside core's shared types (ranges,
//! filters, event triggers), so the public path is that one module.
//!
//! - [`color`]: the strict hex colour reader ([`color::is_hex`], [`color::parse_hex`]) and
//!   [`HexOr`], a colour written as that text or as an array of channels.
//! - [`descriptor`]: [`ItemDescriptor`], [`BlockDescriptor`] and [`BlockSpecifier`], how a
//!   document names an item or a block.
//! - [`menu`]: [`MenuCategory`] and [`CreativeCategory`], an item's or block's creative-menu
//!   placement.
//! - [`one_or_many`]: [`OneOrMany`], one value or an array of them, as written.
//! - [`png`]: the PNG header ([`png::header`], [`png_size`]).
//! - [`vec`](mod@vec): [`Vec3`] and [`Vec2`], a vector as an array or an object, as written.

pub mod color;
pub mod descriptor;
pub mod menu;
pub mod one_or_many;
pub mod png;
pub mod vec;

pub use color::{Channels, HexOr};
pub use descriptor::{
    BlockDescriptor, BlockSpecifier, ItemDescriptor, ItemDescriptorForm, StateValue,
};
pub use menu::{CreativeCategory, MenuCategory};
pub use one_or_many::OneOrMany;
pub use png::{PNG_SIGNATURE, PngHeader, png_size};
pub use vec::{Coordinate, Vec2, Vec3, VecForm, VectorError};
