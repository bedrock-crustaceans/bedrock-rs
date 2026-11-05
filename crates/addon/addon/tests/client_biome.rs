//! The client-biome family (`bedrock_addon::families::client_biome`): every row of its
//! fixtures, the steps, the chain, the validation rules, round trips and the stack's registry.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "client_biome")]

#[path = "client_biome/chain.rs"]
mod chain;
#[path = "client_biome/common.rs"]
mod common;
#[path = "client_biome/engine.rs"]
mod engine;
#[path = "client_biome/rows.rs"]
mod rows;
#[path = "client_biome/rules.rs"]
mod rules;
#[path = "client_biome/stack.rs"]
mod stack;
#[path = "client_biome/steps.rs"]
mod steps;

#[path = "client_biome/located.rs"]
mod located;
