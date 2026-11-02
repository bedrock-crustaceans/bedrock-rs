#![allow(clippy::pedantic, missing_docs)]
//! The feature-rule family (`feature_rules/`, `minecraft:feature_rules`): the fixture rows, the
//! validation rules, the chain, round trips, Molang and the stack accessor.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "feature_rules")]

#[path = "../../families/feature_rules/tests/feature_rules/common.rs"]
mod common;

#[path = "feature_rules/envelope.rs"]
mod envelope;

#[path = "feature_rules/link.rs"]
mod link;

#[path = "pack/common.rs"]
mod pack_common;

#[path = "feature_rules/stack.rs"]
mod stack;
