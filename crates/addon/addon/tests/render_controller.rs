//! The render controller family (`bedrock_addon::families::render_controller`): the rows of
//! its fixtures.

#![cfg(feature = "render_controller")]

use bedrock_addon_core::test_cost as cost;

#[path = "render_controller/common.rs"]
mod common;

#[path = "common/requests.rs"]
mod requests;

#[path = "common/responder.rs"]
mod responder;

#[path = "render_controller/generated.rs"]
mod generated;

#[path = "render_controller/large.rs"]
mod large;

#[path = "render_controller/molang.rs"]
mod molang;

#[path = "render_controller/rows.rs"]
mod rows;

#[path = "render_controller/stack.rs"]
mod stack;

#[path = "render_controller/located.rs"]
mod located;
