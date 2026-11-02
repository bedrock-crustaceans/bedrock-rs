#![allow(clippy::pedantic, missing_docs)]
//! `skins.json`, the one file a skin pack is described by.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "skin")]

// The crate's one protocol for cost tests, and this binary's one lock.
#[path = "../src/test_cost.rs"]
mod cost;

#[path = "common/resource_stacks.rs"]
mod common;

#[path = "skin/rows.rs"]
mod rows;
