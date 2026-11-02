//! The attachable family (`bedrock_addon::families::attachable::Attachable`, families/attachable):
//! every row of its fixtures, the rules it shares with client entities, the upgrade steps, the
//! chain, round trips, Molang and the stack resolution.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "attachable")]

#[path = "attachable/bindings.rs"]
mod bindings;

#[path = "attachable/common.rs"]
mod common;

#[path = "common/responder.rs"]
mod responder;

#[path = "attachable/chain.rs"]
mod chain;

#[path = "attachable/conditions.rs"]
mod conditions;

#[path = "attachable/depth.rs"]
mod depth;

#[path = "../../families/attachable/tests/attachable/generate.rs"]
mod generate;

#[path = "attachable/generated.rs"]
mod generated;

#[path = "attachable/link.rs"]
mod link;

#[path = "attachable/molang.rs"]
mod molang;

#[path = "attachable/nulls.rs"]
mod nulls;

#[path = "attachable/numbers.rs"]
mod numbers;

#[path = "attachable/round_trip.rs"]
mod round_trip;

#[path = "attachable/rows.rs"]
mod rows;

#[path = "attachable/shared.rs"]
mod shared;

#[path = "attachable/stack.rs"]
mod stack;

#[path = "attachable/upgrade_steps.rs"]
mod upgrade_steps;

#[path = "attachable/write_back.rs"]
mod write_back;
