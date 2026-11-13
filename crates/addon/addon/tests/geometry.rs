//! The geometry family (`bedrock_addon::families::geometry`): its rows, the chain, round trips,
//! Molang and the pack stack.

#![cfg(feature = "geometry")]

// The crate's one protocol for cost tests, and this binary's one lock.
use bedrock_addon_core::test_cost as cost;

#[path = "../../families/geometry/tests/geometry/common.rs"]
mod common;
#[path = "common/responder.rs"]
mod responder;

#[path = "geometry/deviations.rs"]
mod deviations;
#[path = "geometry/generated.rs"]
mod generated;
#[path = "geometry/stack.rs"]
mod stack;
#[path = "geometry/stacks.rs"]
mod stacks;
