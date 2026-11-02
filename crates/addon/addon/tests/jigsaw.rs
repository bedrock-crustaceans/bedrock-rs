#![allow(clippy::pedantic, missing_docs)]
//! The jigsaw family (`bedrock_addon::families::jigsaw`): the fixture rows, the record
//! documents, the chains, round trips and the group's loader.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "jigsaw")]

#[path = "jigsaw/chain.rs"]
mod chain;

#[path = "jigsaw/common.rs"]
mod common;

#[path = "jigsaw/input.rs"]
mod input;

#[path = "jigsaw/link.rs"]
mod link;

#[path = "jigsaw/on_placed.rs"]
mod on_placed;

#[path = "jigsaw/records.rs"]
mod records;

#[path = "jigsaw/registry.rs"]
mod registry;

#[path = "jigsaw/round_trip.rs"]
mod round_trip;

#[path = "jigsaw/rows.rs"]
mod rows;

#[path = "jigsaw/rules.rs"]
mod rules;

#[path = "jigsaw/engine.rs"]
mod engine;
