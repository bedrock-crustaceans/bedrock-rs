//! Test JSON, read through the door.

use bedrock_addon_core::json::{Value, from_str_jsonc};

/// `text` read as a JSON value by the door's jsonc entry point; panics with the text and the
/// error when it is no JSON.
#[track_caller]
pub fn json(text: &str) -> Value {
    from_str_jsonc(text).unwrap_or_else(|e| panic!("{text}: {e}"))
}
