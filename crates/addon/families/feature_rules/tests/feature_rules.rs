#![allow(clippy::pedantic, missing_docs)]
//! The feature-rule family (`feature_rules/`, `minecraft:feature_rules`): the fixture rows, the
//! validation rules, the chain, round trips, Molang and the stack accessor.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "feature_rules/common.rs"]
mod common;

#[path = "feature_rules/identifier.rs"]
mod identifier;

#[path = "feature_rules/input.rs"]
mod input;

#[path = "feature_rules/molang.rs"]
mod molang;

#[path = "feature_rules/range.rs"]
mod range;

#[path = "feature_rules/round_trip.rs"]
mod round_trip;

#[path = "feature_rules/rows.rs"]
mod rows;

#[path = "feature_rules/scatter.rs"]
mod scatter;

#[path = "feature_rules/structure.rs"]
mod structure;

#[path = "feature_rules/validation.rs"]
mod validation;

#[path = "feature_rules/engine.rs"]
mod engine;
