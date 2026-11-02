//! The geometry family (`bedrock_addon::families::geometry`): its rows, the chain, round trips,
//! Molang and the pack stack.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "geometry")]

// The crate's one protocol for cost tests, and this binary's one lock.
#[path = "../src/test_cost.rs"]
mod cost;

#[path = "geometry/chain.rs"]
mod chain;
#[path = "geometry/common.rs"]
mod common;
#[path = "common/responder.rs"]
mod responder;

#[path = "geometry/deviations.rs"]
mod deviations;
#[path = "geometry/gates.rs"]
mod gates;
#[path = "geometry/generated.rs"]
mod generated;
#[path = "geometry/molang.rs"]
mod molang;
#[path = "geometry/round_trip.rs"]
mod round_trip;
#[path = "geometry/stack.rs"]
mod stack;
