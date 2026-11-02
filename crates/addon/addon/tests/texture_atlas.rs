//! The terrain and item atlases (`bedrock_addon::families::texture_atlas`): the two atlas files
//! and the link rules that check a block's or an item's texture against them.

#![cfg(feature = "texture_atlas")]
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "terrain_texture/common.rs"]
mod common;

#[path = "texture_atlas/item_texture/mod.rs"]
mod item_texture;

#[path = "texture_atlas/terrain_texture/mod.rs"]
mod terrain_texture;
