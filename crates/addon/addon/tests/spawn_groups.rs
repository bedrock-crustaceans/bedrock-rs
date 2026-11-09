//! The spawn-groups family (`bedrock_addon::families::spawn_groups::SpawnGroup`), read from the
//! app data only.

#![cfg(all(feature = "spawn_rules", feature = "spawn_groups"))]

#[path = "spawn_rules/common.rs"]
mod common;

#[path = "spawn_groups/discovery.rs"]
mod discovery;

#[path = "spawn_groups/molang.rs"]
mod molang;

#[path = "spawn_groups/rows.rs"]
mod rows;
