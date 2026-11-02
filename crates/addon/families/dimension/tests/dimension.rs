//! The dimension family (`dimensions/`, `minecraft:dimension`): the fixture rows, the
//! validation rules, the chain, round trips and the stack accessor.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "dimension/chain.rs"]
mod chain;

#[path = "dimension/common.rs"]
mod common;

#[path = "dimension/engine.rs"]
mod engine;

#[path = "dimension/rows.rs"]
mod rows;
