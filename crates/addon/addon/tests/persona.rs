#![allow(clippy::pedantic, missing_docs)]
//! The persona-piece family (`bedrock_addon::families::persona`): the rows of its fixtures and
//! its rules. The fixture packs are built in memory by the tests.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "persona")]

#[path = "persona/common.rs"]
mod common;

#[path = "persona/packs.rs"]
mod packs;

#[path = "persona/meta.rs"]
mod meta;

#[path = "persona/files.rs"]
mod files;

#[path = "persona/write.rs"]
mod write;

#[path = "persona/vanilla.rs"]
mod vanilla;

#[path = "persona/strict.rs"]
mod strict;
