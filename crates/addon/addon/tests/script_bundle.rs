//! The script bundle (`bedrock_addon::families::script::ScriptBundle`, `families/script_bundle`): every row
//! of its fixtures through the stack accessor `scripts()`, the normaliser against the
//! observed table, the validation rules that no fixture row reaches, and the costs.

#![cfg(feature = "script")]

#[path = "common/responder.rs"]
mod responder;

#[path = "script_bundle/cases.rs"]
mod cases;
#[path = "../../families/script/tests/script_bundle/common.rs"]
mod common;
#[path = "script_bundle/stack.rs"]
mod stack;
#[path = "script_bundle/stacks.rs"]
mod stacks;
