#![allow(clippy::pedantic, missing_docs)]
//! The biome family (`bedrock_addon::families::biome`): every row of its fixtures, the steps, the
//! chain, the validation rules, round trips, Molang and the stack's registry.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "biome")]

#[path = "biome/chain.rs"]
mod chain;

#[path = "biome/common.rs"]
mod common;

#[path = "biome/engine.rs"]
mod engine;

#[path = "biome/molang.rs"]
mod molang;

#[path = "biome/rows.rs"]
mod rows;

#[path = "biome/rules.rs"]
mod rules;

#[path = "biome/stack.rs"]
mod stack;
