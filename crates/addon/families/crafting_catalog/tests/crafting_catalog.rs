#![allow(clippy::pedantic, missing_docs)]
//! The crafting item catalogue (`item_catalog/crafting_item_catalog.json`,
//! `minecraft:crafting_items_catalog`): its rules, the chain, round trips and the stack accessor.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "crafting_catalog/common.rs"]
mod common;

#[path = "crafting_catalog/construct.rs"]
mod construct;

#[path = "crafting_catalog/envelope.rs"]
mod envelope;

#[path = "crafting_catalog/guard.rs"]
mod guard;

#[path = "crafting_catalog/round_trip.rs"]
mod round_trip;

#[path = "crafting_catalog/rows.rs"]
mod rows;

#[path = "crafting_catalog/rules.rs"]
mod rules;

#[path = "crafting_catalog/strict.rs"]
mod strict;

#[path = "crafting_catalog/versions.rs"]
mod versions;
