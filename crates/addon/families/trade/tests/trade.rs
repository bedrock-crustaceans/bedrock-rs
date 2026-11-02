#![allow(clippy::pedantic, missing_docs)]
//! The trade-table family (`bedrock_addon::families::trade`): its rows and rules, the upgrade
//! step, the chain, round trips and the stack accessor.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "trade/common.rs"]
mod common;

#[path = "trade/stamped.rs"]
mod stamped;

#[path = "trade/legacy.rs"]
mod legacy;

#[path = "trade/quantity.rs"]
mod quantity;

#[path = "trade/gates.rs"]
mod gates;

#[path = "trade/rules.rs"]
mod rules;

#[path = "trade/upgrade.rs"]
mod upgrade;

#[path = "trade/round_trip.rs"]
mod round_trip;

#[path = "trade/chain.rs"]
mod chain;

#[path = "trade/input.rs"]
mod input;
