//! `textures/ui/*.json`, the side files of UI images: the reader, the writer and the stack's
//! highest-pack rule.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "ui_texture_info")]

#[path = "common/resource_stacks.rs"]
mod common;

#[path = "ui_texture_info/rows.rs"]
mod rows;

#[path = "ui_texture_info/content.rs"]
mod content;
