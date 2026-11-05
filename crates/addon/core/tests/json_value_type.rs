//! `ValueExt::json_type`: the six JSON types of a `Value`, as jsoncpp's type tests see them.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use bedrock_addon_core::json::{Value, ValueExt, from_str_jsonc};
use bedrock_addon_core::molang::JsonType;

#[test]
fn json_type_names_each_of_the_six_types() {
    for (text, expected) in [
        ("null", JsonType::Null),
        ("true", JsonType::Bool),
        ("false", JsonType::Bool),
        ("0", JsonType::Number),
        ("-1.5", JsonType::Number),
        ("18446744073709551615", JsonType::Number),
        ("\"a\"", JsonType::String),
        ("[]", JsonType::Array),
        ("[1]", JsonType::Array),
        ("{}", JsonType::Object),
        ("{\"a\": 1}", JsonType::Object),
    ] {
        let value: Value = from_str_jsonc(text).unwrap();
        assert_eq!(value.json_type(), expected, "{text}");
    }
}
