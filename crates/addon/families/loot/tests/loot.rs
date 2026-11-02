//! The loot table family (`loot_tables/`): every row of the fixtures, every rule
//! of the validation rules, the round trip, the stack accessor and the deviations.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "loot/common.rs"]
mod common;

#[path = "loot/rows.rs"]
mod rows;

#[path = "loot/schema.rs"]
mod schema;

#[path = "loot/rules.rs"]
mod rules;

#[path = "loot/round_trip.rs"]
mod round_trip;

#[path = "loot/deviations.rs"]
mod deviations;

#[path = "loot/edges.rs"]
mod edges;

#[path = "loot/kept.rs"]
mod kept;
