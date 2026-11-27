//! The script side of a manifest (`bedrock_addon::families::script::ScriptModule`,
//! `families/script_module`): every row of its fixtures as a pack folder that mirrors a BDS
//! 1.26.36.1 probe run, resolved against `catalog_bds_1_26_36.toml`.

#[path = "script_module/catalog.rs"]
mod catalog;
#[path = "script_module/common.rs"]
mod common;
#[path = "script_module/discovery.rs"]
mod discovery;
#[path = "script_module/resolution.rs"]
mod resolution;
#[path = "script_module/small_stack.rs"]
mod small_stack;
#[path = "script_module/stage1.rs"]
mod stage1;
