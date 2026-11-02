//! The script bundle (`bedrock_addon::families::script::ScriptBundle`, families/script_bundle): every row
//! of its fixtures through the stack accessor `scripts()`, the normaliser against the
//! observed table, the validation rules that no fixture row reaches, and the costs.

#![cfg(feature = "script")]
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "../src/test_cost.rs"]
mod cost;
#[path = "common/responder.rs"]
mod responder;

#[path = "script_bundle/cases.rs"]
mod cases;
#[path = "script_bundle/common.rs"]
mod common;
#[path = "script_bundle/normalize.rs"]
mod normalize;
#[path = "script_bundle/parser.rs"]
mod parser;
#[path = "script_bundle/rules.rs"]
mod rules;
#[path = "script_bundle/stack.rs"]
mod stack;
