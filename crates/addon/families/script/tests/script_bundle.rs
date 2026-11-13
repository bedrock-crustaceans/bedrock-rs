//! The script bundle (`bedrock_addon::families::script::ScriptBundle`, `families/script_bundle`): every row
//! of its fixtures through the stack accessor `scripts()`, the normaliser against the
//! observed table, the validation rules that no fixture row reaches, and the costs.

use bedrock_addon_core::test_cost as cost;

#[path = "script_bundle/common.rs"]
mod common;
#[path = "script_bundle/normalize.rs"]
mod normalize;
#[path = "script_bundle/parser.rs"]
mod parser;
#[path = "script_bundle/rules.rs"]
mod rules;
