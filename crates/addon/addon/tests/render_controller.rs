#![allow(clippy::pedantic, missing_docs)]
//! The render controller family (`bedrock_addon::families::render_controller`): the rows of
//! its fixtures.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "render_controller")]

#[path = "../src/test_cost.rs"]
mod cost;

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
