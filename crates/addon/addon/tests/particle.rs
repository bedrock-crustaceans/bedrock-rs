//! The particle family (`bedrock_addon::families::particle`): its rows, the validation rules at
//! their boundaries, the chain, round trips, cost, and the stack's registry.

#![cfg(feature = "particle")]

#[path = "../../families/particle/tests/particle/common.rs"]
mod common;
#[path = "particle/cost.rs"]
mod cost;
#[path = "particle/stack.rs"]
mod stack;
#[path = "particle/stacks.rs"]
mod stacks;
