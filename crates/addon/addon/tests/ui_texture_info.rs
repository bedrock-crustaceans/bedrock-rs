//! `textures/ui/*.json`, the side files of UI images: the reader, the writer and the stack's
//! highest-pack rule.

#![cfg(feature = "ui_texture_info")]

#[path = "common/resource_stacks.rs"]
mod common;

#[path = "ui_texture_info/rows.rs"]
mod rows;

#[path = "common/in_files.rs"]
mod in_files;
