//! The client entity family (`bedrock_addon::families::client_entity::ClientEntity`): every row
//! of its fixtures, the steps, the chain, round trips, Molang and the stack resolution.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "client_entity")]

#[path = "client_entity/common.rs"]
mod common;

#[path = "client_entity/drifts.rs"]
mod drifts;

#[path = "common/responder.rs"]
mod responder;

#[path = "../../families/client_entity/tests/client_entity/generate.rs"]
mod generate;

#[path = "client_entity/generated.rs"]
mod generated;

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

#[path = "client_entity/steps.rs"]
mod steps;

#[path = "client_entity/located.rs"]
mod located;
