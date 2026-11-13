//! The behavior-tree family (`behavior_trees/`, `minecraft:behavior_tree`): every row of
//! The fixtures, the validation rules, the node catalogue of
//! The schema, the stack rules, write-back and the deviations.

#![cfg(feature = "behavior_tree")]

#[path = "../../families/behavior_tree/tests/behavior_tree/common.rs"]
mod common;

#[path = "behavior_tree/stacks.rs"]
mod stacks;

#[path = "common/responder.rs"]
mod responder;

#[path = "common/requests.rs"]
mod requests;

#[path = "behavior_tree/rows.rs"]
mod rows;

#[path = "behavior_tree/stack.rs"]
mod stack;

#[path = "behavior_tree/round_trip.rs"]
mod round_trip;

#[path = "behavior_tree/link.rs"]
mod link;
