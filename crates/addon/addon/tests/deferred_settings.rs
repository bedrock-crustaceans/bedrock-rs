//! The deferred renderer's settings (`bedrock_addon::families::deferred_settings`): the rows of
//! its fixtures, the rules, the upgrade steps, the chains, round trips and the stack accessor.

#![cfg(feature = "deferred_settings")]

#[path = "pack/common.rs"]
mod pack_common;

#[path = "../../families/deferred_settings/tests/deferred_settings/common.rs"]
mod common;

#[path = "deferred_settings/stack.rs"]
mod stack;

#[path = "deferred_settings/stacks.rs"]
mod stacks;
