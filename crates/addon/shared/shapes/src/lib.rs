//! The small value shapes several content families share, defined once: each is a type with
//! no rules of its own beyond reading and writing its JSON forms, and the families compose it.
//! A shape depends on `bedrock_addon_core` only, never on a family.
//!
//! `bedrock_addon::shared` re-exports every item here beside core's shared types (ranges,
//! filters, event triggers), so the public path is that one module.
//!
//! - [`descriptor`]: [`ItemDescriptor`], [`BlockDescriptor`] and [`BlockSpecifier`], how a
//!   document names an item or a block.
//! - [`menu`]: [`MenuCategory`] and [`CreativeCategory`], an item's or block's creative-menu
//!   placement.
//! - [`png`]: the PNG header ([`png_size`]).

pub mod descriptor;
pub mod menu;
pub mod png;

pub use descriptor::{
    BlockDescriptor, BlockSpecifier, ItemDescriptor, ItemDescriptorForm, StateValue,
};
pub use menu::{CreativeCategory, MenuCategory};
pub use png::png_size;
