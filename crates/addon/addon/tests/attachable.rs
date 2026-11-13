//! The attachable family (`bedrock_addon::families::attachable::Attachable`, families/attachable):
//! every row of its fixtures, the rules it shares with client entities, the upgrade steps, the
//! chain, round trips, Molang and the stack resolution.

#![cfg(feature = "attachable")]

#[path = "attachable/bindings.rs"]
mod bindings;

#[path = "../../families/attachable/tests/attachable/common.rs"]
mod common;

#[path = "common/responder.rs"]
mod responder;

#[path = "attachable/conditions.rs"]
mod conditions;

#[path = "attachable/depth.rs"]
mod depth;

#[path = "attachable/link.rs"]
mod link;

#[path = "attachable/molang.rs"]
mod molang;

#[path = "attachable/shared.rs"]
mod shared;

#[path = "attachable/stack.rs"]
mod stack;

#[path = "attachable/stacks.rs"]
mod stacks;

#[path = "attachable/write_back.rs"]
mod write_back;
