//! The dialogue family (`bedrock_addon::families::dialogue`): every row of the fixtures,
//! The validation rules, the walk, round trips and the registry's combine rule.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "dialogue")]

#[path = "dialogue/common.rs"]
mod common;

#[path = "dialogue/discovery.rs"]
mod discovery;

#[path = "dialogue/round_trip.rs"]
mod round_trip;

#[path = "dialogue/rows.rs"]
mod rows;

#[path = "dialogue/rules.rs"]
mod rules;
