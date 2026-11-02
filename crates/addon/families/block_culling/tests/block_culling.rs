#![allow(clippy::pedantic, missing_docs)]
//! The block-culling family (`block_culling/`, `minecraft:block_culling_rules`): every row of
//! its rules, the chain and its step, round trips and the stack accessor.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "block_culling/common.rs"]
mod common;

#[path = "block_culling/document.rs"]
mod document;

#[path = "block_culling/envelope.rs"]
mod envelope;

#[path = "block_culling/round_trip.rs"]
mod round_trip;

#[path = "block_culling/rules.rs"]
mod rules;

#[path = "block_culling/strict.rs"]
mod strict;
