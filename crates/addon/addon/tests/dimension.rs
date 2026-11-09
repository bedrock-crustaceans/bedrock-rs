//! The dimension family (`dimensions/`, `minecraft:dimension`): the fixture rows, the
//! validation rules, the chain, round trips and the stack accessor.

#![cfg(feature = "dimension")]

#[path = "../../families/dimension/tests/dimension/common.rs"]
mod common;

#[path = "pack/common.rs"]
mod pack_common;

#[path = "dimension/rules.rs"]
mod rules;

#[path = "dimension/stack.rs"]
mod stack;

#[path = "common/in_files.rs"]
mod in_files;
