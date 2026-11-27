//! The POI family (`bedrock_addon::families::poi`): its rows and rules, the chains, the
//! deviations, round trips and the group's loader.

#![cfg(feature = "poi")]

#[path = "../../families/poi/tests/poi/common.rs"]
mod common;

#[path = "poi/determinism.rs"]
mod determinism;

#[path = "poi/deviations.rs"]
mod deviations;

#[path = "poi/link.rs"]
mod link;

#[path = "poi/pins.rs"]
mod pins;

#[path = "common/responder.rs"]
mod responder;

#[path = "poi/rows.rs"]
mod rows;

#[path = "poi/stack.rs"]
mod stack;

#[path = "poi/stacks.rs"]
mod stacks;

#[path = "poi/vanilla.rs"]
mod vanilla;
