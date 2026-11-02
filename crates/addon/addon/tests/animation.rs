//! The animation family on both stacks (`bedrock_addon::families::animation`) and the behaviour-
//! pack controller rows: the behaviour-pack side (`behavior`), the resource-pack side
//! (`resource`), the deviations, round trips and Molang.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(all(feature = "animation", feature = "animation_controller"))]

#[path = "animation/common.rs"]
mod common;
#[path = "common/responder.rs"]
mod responder;

#[path = "animation/behavior.rs"]
mod behavior;
#[path = "animation/deviations.rs"]
mod deviations;
#[path = "animation/molang.rs"]
mod molang;
#[path = "animation/resource.rs"]
mod resource;
