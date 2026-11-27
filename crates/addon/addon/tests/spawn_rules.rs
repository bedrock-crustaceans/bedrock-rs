//! The spawn-rules family (`bedrock_addon::families::spawn_rules`, both shapes): the routing
//! gates, the chains, round trips, Molang and the stack accessor.

#![cfg(all(feature = "spawn_rules", feature = "spawn_groups"))]

#[path = "../../families/spawn_rules/tests/spawn_rules/common.rs"]
mod common;
#[path = "spawn_rules/engine.rs"]
mod engine;
#[path = "spawn_rules/groups.rs"]
mod groups;
#[path = "spawn_rules/stack.rs"]
mod stack;
