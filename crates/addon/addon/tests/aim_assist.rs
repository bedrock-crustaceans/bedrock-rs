//! The aim-assist family (`aim_assist/`): the fixture rows, the validation rules, the chains,
//! round trips and the stack accessor.

#![cfg(feature = "aim_assist")]

#[path = "pack/common.rs"]
mod pack_common;

#[path = "../../families/aim_assist/tests/aim_assist/common.rs"]
mod common;

#[path = "aim_assist/stack.rs"]
mod stack;

#[path = "aim_assist/stacks.rs"]
mod stacks;
