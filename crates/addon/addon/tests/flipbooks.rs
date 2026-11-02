//! Flipbooks (`bedrock_addon::families::flipbooks`): every fixture row
//! through `rp.flipbooks()`. The rows that check an entry against the
//! terrain atlas or its image wait for the link pass.

#![cfg(feature = "flipbooks")]
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

// The crate's one protocol for cost tests, and this binary's one lock.
#[path = "../src/test_cost.rs"]
mod cost;

#[path = "terrain_texture/common.rs"]
mod common;

#[path = "flipbooks/rows.rs"]
mod rows;
