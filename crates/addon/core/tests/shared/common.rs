//! Helpers for the shared-type tests: read a value from JSON text the way a family reads a
//! member (the jsonc entry point), write it back, and compare at the value level.

#![allow(clippy::disallowed_methods, clippy::disallowed_types, reason = "tests read fixtures and call facet-json directly")]
#![allow(dead_code)] // each test file uses part of the helpers

use bedrock_addon_core::json::{Indent, Value, WriteStyle, write_json};
use facet::Facet;

/// Wraps the value in `{ "v": … }`, the way a family holds a member.
#[derive(Debug, Facet)]
pub struct Holder<T> {
    pub v: T,
}

/// Reads `json` as a `T` member.
pub fn read<T: for<'a> Facet<'a>>(json: &str) -> Result<T, facet_json::DeserializeError> {
    let text = format!(r#"{{ "v": {json} }}"#);
    facet_json::from_str_borrowed_jsonc::<Holder<T>>(&text).map(|holder| holder.v)
}

/// Writes `value` as one line of JSON.
pub fn compact<'f, T: Facet<'f>>(value: &T) -> String {
    let mut out = Vec::new();
    let style = WriteStyle {
        indent: Indent::Compact,
        trailing_newline: false,
    };
    write_json(value, &mut out, style).expect("the value has a JSON form");
    String::from_utf8(out).unwrap()
}

/// `json` re-written as one line, so two texts compare without whitespace.
pub fn normalised(json: &str) -> String {
    let value: Value = facet_json::from_str_jsonc(json).expect("valid JSON");
    compact(&value)
}

/// Reads `json`, requires the written value to be `json` itself (member order, number tags),
/// and the written text to read back to an equal value.
pub fn round_trip<T: for<'a> Facet<'a> + PartialEq + std::fmt::Debug>(json: &str) -> T {
    let value: T = read(json).unwrap_or_else(|e| panic!("{json} does not read: {e}"));
    let written = compact(&value);
    assert_eq!(
        written,
        normalised(json),
        "written form differs from the input"
    );
    let again: T = read(&written).expect("the written form reads back");
    assert_eq!(again, value, "the written form reads back to another value");
    value
}
