//! The attachable family (`bedrock_addon::families::attachable::Attachable`, families/attachable):
//! every row of its fixtures, the rules it shares with client entities, the upgrade steps, the
//! chain, round trips, Molang and the stack resolution.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "attachable/generate.rs"]
mod generate;

#[path = "attachable/common.rs"]
mod common;

#[path = "attachable/chain.rs"]
mod chain;

#[path = "attachable/drifts.rs"]
mod drifts;

#[path = "attachable/generated.rs"]
mod generated;

#[path = "attachable/nulls.rs"]
mod nulls;

#[path = "attachable/numbers.rs"]
mod numbers;

#[path = "attachable/round_trip.rs"]
mod round_trip;

#[path = "attachable/rows.rs"]
mod rows;

#[path = "attachable/upgrade_steps.rs"]
mod upgrade_steps;

#[path = "attachable/located.rs"]
mod located;
