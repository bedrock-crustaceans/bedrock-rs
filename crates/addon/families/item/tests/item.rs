//! The item family (`items/`, `minecraft:item`): the chain, its steps, round trips, Molang and
//! the network NBT.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "item/common.rs"]
mod common;

#[path = "item/upgrade_steps.rs"]
mod upgrade_steps;

#[path = "item/routing.rs"]
mod routing;

#[path = "item/upgrade.rs"]
mod upgrade;

#[path = "item/replaced.rs"]
mod replaced;

#[path = "item/molang.rs"]
mod molang;

#[path = "item/beta.rs"]
mod beta;

#[path = "item/oracle.rs"]
mod oracle;

#[path = "item/round_trip.rs"]
mod round_trip;

#[path = "item/legacy_audit.rs"]
mod legacy_audit;

#[path = "item/molang_legacy.rs"]
mod molang_legacy;

#[path = "item/deviations.rs"]
mod deviations;

#[path = "item/v1_26_50.rs"]
mod v1_26_50;
