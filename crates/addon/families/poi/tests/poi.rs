//! The POI family (`bedrock_addon::families::poi`): its rows and rules, the chains, the
//! deviations, round trips and the group's loader.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "poi/chain.rs"]
mod chain;

#[path = "poi/common.rs"]
mod common;

#[path = "poi/located.rs"]
mod located;

#[path = "poi/round_trip.rs"]
mod round_trip;

#[path = "poi/rules.rs"]
mod rules;
