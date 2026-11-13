//! The script side of a manifest (`bedrock_addon::families::script::ScriptModule`,
//! `families/script_module`): every row of its fixtures as a pack folder that mirrors a BDS
//! 1.26.36.1 probe run, resolved against `catalog_bds_1_26_36.toml`.

#![cfg(feature = "script")]

#[path = "../../families/script/tests/script_module/common.rs"]
#[allow(
    dead_code,
    reason = "the fixture runner is the family crate's; rules.rs, the one file here, uses part of it"
)]
mod common;
#[path = "script_module/rules.rs"]
mod rules;
