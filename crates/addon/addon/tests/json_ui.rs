//! JSON UI (`ui/_ui_defs.json`, the files it lists, `ui/_global_variables.json`): the rows of
//! its fixtures, the rules, the round trip of the file kinds and the deviations.

#![allow(
    clippy::float_cmp,
    reason = "the values are read from JSON, not computed"
)]
#![cfg(feature = "json_ui")]

// The crate's one protocol for cost tests, and this binary's one lock.
use bedrock_addon_core::test_cost as cost;

#[path = "json_ui/common.rs"]
mod common;

#[path = "common/responder.rs"]
mod responder;

#[path = "json_ui/stack.rs"]
mod stack;

#[path = "json_ui/loader.rs"]
mod loader;

#[path = "json_ui/lint.rs"]
mod lint;

#[path = "json_ui/located.rs"]
mod located;

#[path = "json_ui/content.rs"]
mod content;
