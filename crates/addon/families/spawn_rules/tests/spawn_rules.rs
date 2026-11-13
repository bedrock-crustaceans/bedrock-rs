//! The spawn-rules family (`bedrock_addon::families::spawn_rules`, both shapes): the routing
//! gates, the chains, round trips and Molang.

#[path = "spawn_rules/chain.rs"]
mod chain;
#[path = "spawn_rules/common.rs"]
mod common;
#[path = "spawn_rules/legacy.rs"]
mod legacy;
#[path = "spawn_rules/molang.rs"]
mod molang;
#[path = "spawn_rules/routing.rs"]
mod routing;
#[path = "spawn_rules/simplified.rs"]
mod simplified;
