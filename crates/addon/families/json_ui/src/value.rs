//! Helpers over the raw JSON the UI loader works on: jsoncpp's member order and its string
//! conversion.

use bedrock_addon_core::json::Value;

pub(crate) use bedrock_addon_core::json::jsoncpp::{member, number};

/// The members of `value` in the order jsoncpp iterates an object: by key, bytewise (its
/// objects are a `std::map`). Empty for anything but an object.
// A jsoncpp object iterates its keys in sorted order.
pub(crate) fn sorted_members(value: &Value) -> Vec<(&str, &Value)> {
    let mut members: Vec<(&str, &Value)> = value
        .as_object()
        .map(|object| object.iter().map(|(k, v)| (k.as_str(), v)).collect())
        .unwrap_or_default();
    members.sort_by(|a, b| a.0.cmp(b.0));
    members
}

/// The key of the first member of an object as jsoncpp sees it: the smallest key.
pub(crate) fn first_key(value: &Value) -> Option<&str> {
    value
        .as_object()?
        .keys()
        .map(facet_value::VString::as_str)
        .min()
}

/// A key up to its first `@`: the short name of a `name@base` key.
pub(crate) fn short_name(key: &str) -> &str {
    key.split_once('@').map_or(key, |(name, _)| name)
}

/// The control name of an element of a `controls` array: the short name of its first member,
/// when the element is a non-empty object.
pub(crate) fn element_name(element: &Value) -> Option<&str> {
    first_key(element).map(short_name)
}

/// The string of a value read with `asString("")`: the string itself, `""` for anything else.
pub(crate) fn string_or_empty(value: Option<&Value>) -> &str {
    value
        .and_then(Value::as_string)
        .map_or("", |text| text.as_str())
}
