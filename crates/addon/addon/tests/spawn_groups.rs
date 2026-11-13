//! The spawn-groups family (`bedrock_addon::families::spawn_groups::SpawnGroup`), read from the
//! app data only.

#![cfg(all(feature = "spawn_rules", feature = "spawn_groups"))]

#[path = "../../families/spawn_rules/tests/spawn_rules/common.rs"]
mod common;

#[path = "spawn_rules/groups.rs"]
mod groups;

#[path = "spawn_groups/discovery.rs"]
mod discovery;

#[path = "spawn_groups/molang.rs"]
mod molang;

#[path = "spawn_groups/rows.rs"]
mod rows;
