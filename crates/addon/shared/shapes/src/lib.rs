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
//! - [`event`]: [`event::children`], the nested lists of an event node, and the game's
//!   finding for a node whose `sequence` is ignored.
//! - [`flag`]: [`TrueOnly`], a member whose one allowed value is `true`.
//! - [`keyframes`]: [`Keyframes`], a value that may change over the day.
//! - [`menu`]: [`MenuCategory`] and [`CreativeCategory`], an item's or block's creative-menu
//!   placement.
//! - [`named`]: [`NamedCondition`], a name or `{ name: condition }` of which the game reads one
//!   member, and its two lists' entries [`ControllerUse`] and [`AnimateEntry`].
//! - [`one_or_many`]: [`OneOrMany`], one value or an array of them, as written.
//! - [`png`]: the PNG header ([`png::header`], [`png_size`]).
//! - [`texture`]: the files of a stack as the texture rules see them ([`texture::files`]) and
//!   the headers of the image formats the game reads ([`texture::image`]).
//! - [`vec`](mod@vec): [`Vec3`] and [`Vec2`], a vector as an array or an object, as written.

pub mod codes;
pub mod color;
pub mod descriptor;
pub mod event;
pub mod flag;
pub mod keyframes;
pub mod menu;
pub mod named;
pub mod one_or_many;
pub mod png;
pub mod texture;
pub mod vec;

pub use color::{Channels, HexOr};
pub use descriptor::{
    BlockDescriptor, BlockSpecifier, ItemDescriptor, ItemDescriptorForm, StateValue,
};
pub use flag::{NotTrue, TrueOnly};
pub use keyframes::{Keyframe, Keyframes};
pub use menu::{CreativeCategory, MenuCategory};
pub use named::{
    AnimateEntry, ControllerUse, Name, NamedCondition, NamedConditionProxy, NotANamedCondition,
};
pub use one_or_many::OneOrMany;
pub use png::{PNG_SIGNATURE, PngHeader, png_size};
pub use vec::{Coordinate, Vec2, Vec3, VecForm, VectorError};
