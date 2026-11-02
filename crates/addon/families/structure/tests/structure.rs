#![allow(clippy::pedantic, missing_docs)]
//! The structure family (`structures/**/*.mcstructure`, little-endian NBT): every row of
//! The fixtures, the exact round trip, the lookups by identifier, the stack
//! and the deviations.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "../../../addon/tests/../src/test_cost.rs"]
mod cost;

#[path = "structure/common.rs"]
mod common;

#[path = "structure/rows.rs"]
mod rows;

#[path = "structure/rules.rs"]
mod rules;

#[path = "structure/round_trip.rs"]
mod round_trip;

#[path = "structure/api.rs"]
mod api;

#[path = "structure/deviations.rs"]
mod deviations;

#[path = "structure/raw.rs"]
mod raw;

#[path = "structure/large.rs"]
mod large;
