//! The client entity family (`bedrock_addon::families::client_entity::ClientEntity`): every row
//! of its fixtures, the steps, the chain, round trips, Molang and the stack resolution.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "client_entity/common.rs"]
mod common;

#[path = "client_entity/drifts.rs"]
mod drifts;

#[path = "client_entity/generate.rs"]
mod generate;

#[path = "client_entity/generated.rs"]
mod generated;

#[path = "client_entity/steps.rs"]
mod steps;

#[path = "client_entity/located.rs"]
mod located;
