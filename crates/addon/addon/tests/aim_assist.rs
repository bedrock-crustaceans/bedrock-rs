#![allow(clippy::pedantic, missing_docs)]
//! The aim-assist family (`aim_assist/`): the fixture rows, the validation rules, the chains,
//! round trips and the stack accessor.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "aim_assist")]

#[path = "pack/common.rs"]
mod pack_common;

#[path = "aim_assist/common.rs"]
mod common;

#[path = "aim_assist/rows.rs"]
mod rows;

#[path = "aim_assist/stack.rs"]
mod stack;

#[path = "aim_assist/chain.rs"]
mod chain;
