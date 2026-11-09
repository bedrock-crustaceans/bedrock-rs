//! The feature-rule family (`feature_rules/`, `minecraft:feature_rules`): the fixture rows, the
//! validation rules, the chain, round trips, Molang and the stack accessor.

#![cfg(feature = "feature_rules")]

#[path = "../../families/feature_rules/tests/feature_rules/common.rs"]
#[allow(
    clippy::disallowed_methods,
    reason = "calls facet-json directly: parses a fixture into a json Value to compare it; not yet moved to the door"
)]
mod common;

#[path = "feature_rules/envelope.rs"]
mod envelope;

#[path = "feature_rules/link.rs"]
mod link;

#[path = "pack/common.rs"]
mod pack_common;

#[path = "feature_rules/stack.rs"]
mod stack;

#[path = "common/in_files.rs"]
mod in_files;
