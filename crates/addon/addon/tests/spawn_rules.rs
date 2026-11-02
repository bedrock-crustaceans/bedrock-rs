//! The spawn-rules family (`bedrock_addon::families::spawn_rules`, both shapes): the routing
//! gates, the chains, round trips, Molang and the stack accessor.

#![cfg(all(feature = "spawn_rules", feature = "spawn_groups"))]
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "spawn_rules/chain.rs"]
mod chain;
#[path = "spawn_rules/common.rs"]
mod common;
#[path = "spawn_rules/engine.rs"]
mod engine;
#[path = "spawn_rules/legacy.rs"]
mod legacy;
#[path = "spawn_rules/molang.rs"]
mod molang;
#[path = "spawn_rules/routing.rs"]
mod routing;
#[path = "spawn_rules/simplified.rs"]
mod simplified;
#[path = "spawn_rules/stack.rs"]
mod stack;
