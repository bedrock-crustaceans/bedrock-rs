//! The client entity family (`bedrock_addon::families::client_entity::ClientEntity`): every row
//! of its fixtures, the steps, the chain, round trips, Molang and the stack resolution.

#![cfg(feature = "client_entity")]

#[path = "../../families/client_entity/tests/client_entity/common.rs"]
mod common;

#[path = "common/responder.rs"]
mod responder;

#[path = "client_entity/link.rs"]
mod link;

#[path = "client_entity/molang.rs"]
mod molang;

#[path = "client_entity/round_trip.rs"]
mod round_trip;

#[path = "client_entity/rows.rs"]
mod rows;

#[path = "client_entity/stack.rs"]
mod stack;

#[path = "client_entity/stacks.rs"]
mod stacks;
