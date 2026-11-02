#![allow(clippy::pedantic, missing_docs)]
//! The dimension family (`dimensions/`, `minecraft:dimension`): the fixture rows, the
//! validation rules, the chain, round trips and the stack accessor.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "dimension")]

#[path = "../../families/dimension/tests/dimension/common.rs"]
mod common;

#[path = "pack/common.rs"]
mod pack_common;

#[path = "dimension/rules.rs"]
mod rules;

#[path = "dimension/stack.rs"]
mod stack;
