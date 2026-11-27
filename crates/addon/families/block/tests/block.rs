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

#[path = "block/common.rs"]
mod common;

#[path = "block/chain.rs"]
mod chain;

#[path = "block/component_scalars.rs"]
mod component_scalars;

#[path = "block/explosion_scalars.rs"]
mod explosion_scalars;

#[path = "block/fallbacks.rs"]
mod fallbacks;

#[path = "block/legacy_reader.rs"]
mod legacy_reader;

#[path = "block/members_of_ours.rs"]
mod members_of_ours;

#[path = "block/presence.rs"]
mod presence;

#[path = "block/replaced.rs"]
mod replaced;

#[path = "block/rows.rs"]
mod rows;

#[path = "block/rules.rs"]
mod rules;

#[path = "block/upgrade.rs"]
mod upgrade;

#[path = "block/latest.rs"]
mod latest;
