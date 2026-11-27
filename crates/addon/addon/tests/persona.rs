//! The persona-piece family (`bedrock_addon::families::persona`): the rows of its fixtures and
//! its rules. The fixture packs are built in memory by the tests.

#![cfg(feature = "persona")]

#[path = "../../families/persona/tests/persona/common.rs"]
mod common;

#[path = "persona/packs.rs"]
mod packs;

#[path = "persona/meta.rs"]
mod meta;

#[path = "persona/files.rs"]
mod files;

#[path = "persona/strict.rs"]
mod strict;

#[path = "persona/stacks.rs"]
mod stacks;
