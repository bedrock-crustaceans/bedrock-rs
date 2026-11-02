//! The function family (`bedrock_addon::families::function`, families/function): every row of its
//! The fixtures through the stack accessors `bp.functions()` and `bp.tick_functions()`, the line
//! rules, the command-table lints, the tick list's merge and the write side.

#![cfg(feature = "function")]
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "common/responder.rs"]
mod responder;

#[path = "function/common.rs"]
mod common;
#[path = "function/grammar.rs"]
mod grammar;
#[path = "function/lines.rs"]
mod lines;
#[path = "function/lint.rs"]
mod lint;
#[path = "function/loader.rs"]
mod loader;
#[path = "function/names.rs"]
mod names;
#[path = "function/tick.rs"]
mod tick;
#[path = "function/write.rs"]
mod write;
