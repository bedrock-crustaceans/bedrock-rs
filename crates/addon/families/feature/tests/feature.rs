//! The feature family (`features/`, one schema per feature type): the fixture rows, the
//! steps, the chains, round trips, Molang and the stack.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "feature/common.rs"]
mod common;

#[path = "feature/single_block.rs"]
mod single_block;

#[path = "feature/loader.rs"]
mod loader;

#[path = "feature/types.rs"]
mod types;

#[path = "feature/molang.rs"]
mod molang;

#[path = "feature/upgrade.rs"]
mod upgrade;

#[path = "feature/round_trip.rs"]
mod round_trip;

#[path = "feature/rules.rs"]
mod rules;

#[path = "feature/drop4.rs"]
mod drop4;

#[path = "feature/engine.rs"]
mod engine;
