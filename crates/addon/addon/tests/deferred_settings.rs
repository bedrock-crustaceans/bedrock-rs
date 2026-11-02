//! The deferred renderer's settings (`bedrock_addon::families::deferred_settings`): the rows of
//! its fixtures, the rules, the upgrade steps, the chains, round trips and the stack accessor.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "deferred_settings")]

#[path = "pack/common.rs"]
mod pack_common;

#[path = "deferred_settings/common.rs"]
mod common;

#[path = "deferred_settings/envelope.rs"]
mod envelope;

#[path = "deferred_settings/single.rs"]
mod single;

#[path = "deferred_settings/keyframes.rs"]
mod keyframes;

#[path = "deferred_settings/chains.rs"]
mod chains;

#[path = "deferred_settings/stack.rs"]
mod stack;

#[path = "deferred_settings/write.rs"]
mod write;

#[path = "deferred_settings/ranges.rs"]
mod ranges;

#[path = "../../families/deferred_settings/tests/deferred_settings/compare.rs"]
mod compare;

#[path = "deferred_settings/generated.rs"]
mod generated;
