//! The POI family (`bedrock_addon::families::poi`): its rows and rules, the chains, the
//! deviations, round trips and the group's loader.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "poi")]

#[path = "poi/common.rs"]
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

#[path = "poi/round_trip.rs"]
mod round_trip;

#[path = "poi/rows.rs"]
mod rows;

#[path = "poi/rules.rs"]
mod rules;

#[path = "poi/stack.rs"]
mod stack;

#[path = "poi/vanilla.rs"]
mod vanilla;
