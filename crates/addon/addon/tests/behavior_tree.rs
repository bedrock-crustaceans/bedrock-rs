//! The behavior-tree family (`behavior_trees/`, `minecraft:behavior_tree`) on a stack: every row
//! of its fixtures, which file's tree a name gets and the walk, write-back, and the link pass.
//! The tests that read one document are the family crate's (`families/behavior_tree/tests`).

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
