//! The block family (`bedrock_addon::families::block`): the fixture rows, the upgrade steps, the chain,
//! round trips, Molang, network NBT and the thin slice.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "block/upgrade_steps.rs"]
mod upgrade_steps;

#[path = "block/places.rs"]
mod places;

#[path = "block/bounds.rs"]
mod bounds;

#[path = "block/unknown_severity.rs"]
mod unknown_severity;
