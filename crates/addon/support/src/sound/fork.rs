//! The JSON rules of the game's JSON reader that the three resource-pack sound files are read
//! with: the scalar coercions, the two member lookups and the stack merge.
//!
//! The game merges the copies of a sound file as JSON and reads the merged document with
//! `asFloat` / `asBool` / `asInt` / `asString`, so the readers of this module work on a
//! [`Value`] and apply the reader's coercions exactly.

use bedrock_addon_core::json::{Coerce, Coercing, IsIntRule, Value, WideInt};
use facet_value::VObject;

/// The value the const `operator[]` gives for a missing member or a non-object.
static NULL: Value = Value::NULL;

/// The JSON type the reader stores a value as.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Stored {
    /// `null`.
    Null,
    /// An integer literal (`1`): the reader's int and uint.
    Integer,
    /// A literal with a fraction or an exponent (`1.0`): the reader's real.
    Real,
    /// A string.
    String,
    /// `true` or `false`.
    Bool,
    /// An array.
    Array,
    /// An object.
    Object,
}

/// How the reader stores `value`. A number is an integer or a real by its spelling, so `1` and
/// `1.0` differ as they do for `isInt()` and `isDouble()`.
pub fn stored(value: &Value) -> Stored {
    if value.is_null() {
        Stored::Null
    } else if value.is_bool() {
        Stored::Bool
    } else if let Some(number) = value.as_number() {
        if number.is_float() {
            Stored::Real
        } else {
            Stored::Integer
        }
    } else if value.is_array() {
        Stored::Array
    } else if value.is_object() {
        Stored::Object
    } else {
        Stored::String
    }
}

/// The text of a string value.
pub fn text(value: &Value) -> Option<&str> {
    value.as_string().map(facet_value::VString::as_str)
}

/// The number of a numeric value, as a double.
fn number(value: &Value) -> Option<f64> {
    value.as_number().map(facet_value::VNumber::to_f64_lossy)
}

/// `asFloat(default)`: a number converted, a bool 0.0 or 1.0, `null` the default, and a
/// string, array or object **0.0** (not the default).
#[allow(
    clippy::cast_possible_truncation,
    reason = "vanilla narrows the double to a float"
)]
pub fn as_float(value: &Value, default: f32) -> f32 {
    match stored(value) {
        Stored::Null => default,
        Stored::Bool => f32::from(u8::from(value.is_true())),
        Stored::Integer | Stored::Real => number(value).unwrap_or(0.0) as f32,
        Stored::String | Stored::Array | Stored::Object => 0.0,
    }
}

/// `asBool(default)`: a bool as is, a number whether it is not zero, a string, array or object
/// whether it is not empty (so `"false"` is true), `null` the default.
pub fn as_bool(value: &Value, default: bool) -> bool {
    match stored(value) {
        Stored::Null => default,
        Stored::Bool => value.is_true(),
        Stored::Integer | Stored::Real => number(value).is_some_and(|n| n != 0.0),
        Stored::String => text(value).is_some_and(|text| !text.is_empty()),
        Stored::Array => value.as_array().is_some_and(|array| !array.is_empty()),
        Stored::Object => value.as_object().is_some_and(|object| !object.is_empty()),
    }
}

/// `asInt(default)`: an integer as is, a real truncated toward zero, a bool 0 or 1, `null` the
/// default, and a string, array or object **0** (not the default).
pub fn as_int(value: &Value, default: i32) -> i32 {
    // 0 for an integer token past `i64` ([`WideInt::Zero`]). Only the `as_int` half of the
    // `Coercing` profile applies: the `IsIntRule` is never read here.
    const PROFILE: Coercing = Coercing::new(IsIntRule::Token, WideInt::Zero);
    value.as_int(default, PROFILE)
}

/// `asString()`: a string as is, a bool `"true"` or `"false"`, and `null`, a number, an array
/// or an object `""`.
pub fn to_text(value: &Value) -> String {
    match stored(value) {
        Stored::String => text(value).unwrap_or_default().to_owned(),
        Stored::Bool => if value.is_true() { "true" } else { "false" }.to_owned(),
        _ => String::new(),
    }
}

/// The const `operator[]`: the member `key` of an object, and `null` for a missing member or
/// a value that is no object. Nothing is changed.
pub fn member<'a>(value: &'a Value, key: &str) -> &'a Value {
    value
        .as_object()
        .and_then(|object| object.get(key))
        .unwrap_or(&NULL)
}

/// Whether `value` is an object with a member `key` (`isMember`).
pub fn has_member(value: &Value, key: &str) -> bool {
    value
        .as_object()
        .is_some_and(|object| object.contains_key(key))
}

/// The non-const `operator[]`: a value that is no object is first
/// replaced by `{}`, then the member `key` is found or inserted as `null`.
pub fn index_mut<'a>(value: &'a mut Value, key: &str) -> &'a mut Value {
    if !value.is_object() {
        *value = Value::from(VObject::new());
    }
    let Some(object) = value.as_object_mut() else {
        unreachable!("the value was made an object above")
    };
    if !object.contains_key(key) {
        object.insert(key, Value::NULL);
    }
    let Some(member) = object.get_mut(key) else {
        unreachable!("the member was inserted above")
    };
    member
}

/// The member names of an object in the reader's iteration order (its map is sorted bytewise);
/// none for another value.
pub fn member_names(value: &Value) -> Vec<String> {
    let mut names: Vec<String> = value
        .as_object()
        .into_iter()
        .flat_map(VObject::keys)
        .map(|key| key.as_str().to_owned())
        .collect();
    names.sort_unstable();
    names
}

/// What a `begin()` / `end()` walk visits: the elements of an array, the member values of an
/// object in sorted key order, and nothing for a scalar. Vanilla walks its lists without an
/// `isArray()` check.
pub fn elements(value: &Value) -> Vec<&Value> {
    if let Some(array) = value.as_array() {
        array.as_slice().iter().collect()
    } else if value.is_object() {
        member_names(value)
            .iter()
            .map(|name| member(value, name))
            .collect()
    } else {
        Vec::new()
    }
}

/// The predicate of a merge: handles one level itself and says so by returning `true`.
pub(crate) type MergePredicate = fn(&mut Value, &Value) -> bool;

/// The recursive merge: unless the predicate handled the level, every member
/// of `src` is merged into `dst`: object into object recursively, anything else replaces what
/// `dst` had (arrays whole, `null` included). `dst` is indexed with the non-const
/// `operator[]`, so a `dst` that is no object becomes one first. Nothing is ever removed.
pub fn recursive_merge(dst: &mut Value, src: &Value, predicate: Option<MergePredicate>) {
    if predicate.is_some_and(|predicate| predicate(dst, src)) {
        return;
    }
    for name in member_names(src) {
        let from = member(src, &name);
        let both_objects = from.is_object() && member(dst, &name).is_object();
        let into = index_mut(dst, &name);
        if both_objects {
            recursive_merge(into, from, predicate);
        } else {
            *into = from.clone();
        }
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, reason = "the coercions give exact values")]
mod tests {
    use super::*;

    fn json(text: &str) -> Value {
        bedrock_addon_core::json::from_str::<Value>(text).expect("valid JSON")
    }

    #[test]
    fn as_float_takes_numbers_and_bools_and_zeroes_other_types() {
        assert_eq!(as_float(&json("1"), 9.0), 1.0);
        assert_eq!(as_float(&json("2.5"), 9.0), 2.5);
        assert_eq!(as_float(&json("true"), 9.0), 1.0);
        assert_eq!(as_float(&json("false"), 9.0), 0.0);
        assert_eq!(as_float(&json("null"), 9.0), 9.0);
        assert_eq!(as_float(&json("\"3\""), 9.0), 0.0);
        assert_eq!(as_float(&json("[3]"), 9.0), 0.0);
        assert_eq!(as_float(&json("{\"a\": 3}"), 9.0), 0.0);
    }

    #[test]
    fn as_bool_reads_any_non_empty_string_as_true() {
        assert!(as_bool(&json("\"false\""), false));
        assert!(!as_bool(&json("\"\""), true));
        assert!(as_bool(&json("2"), false));
        assert!(!as_bool(&json("0"), true));
        assert!(!as_bool(&json("0.0"), true));
        assert!(as_bool(&json("[0]"), false));
        assert!(!as_bool(&json("[]"), true));
        assert!(as_bool(&json("{\"a\": 0}"), false));
        assert!(!as_bool(&json("{}"), true));
        assert!(as_bool(&json("null"), true));
        assert!(!as_bool(&json("null"), false));
        assert!(!as_bool(&json("false"), true));
    }

    #[test]
    fn as_int_truncates_reals_and_zeroes_strings() {
        assert_eq!(as_int(&json("2.9"), 7), 2);
        assert_eq!(as_int(&json("-2.9"), 7), -2);
        assert_eq!(as_int(&json("5"), 7), 5);
        assert_eq!(as_int(&json("true"), 7), 1);
        assert_eq!(as_int(&json("\"2\""), 7), 0);
        assert_eq!(as_int(&json("[2]"), 7), 0);
        assert_eq!(as_int(&json("null"), 7), 7);
    }

    #[test]
    fn to_text_spells_bools_and_empties_numbers() {
        assert_eq!(to_text(&json("\"a\"")), "a");
        assert_eq!(to_text(&json("true")), "true");
        assert_eq!(to_text(&json("false")), "false");
        assert_eq!(to_text(&json("5")), "");
        assert_eq!(to_text(&json("null")), "");
        assert_eq!(to_text(&json("[\"a\"]")), "");
    }

    #[test]
    fn a_number_is_an_integer_or_a_real_by_its_spelling() {
        assert_eq!(stored(&json("16")), Stored::Integer);
        assert_eq!(stored(&json("16.0")), Stored::Real);
    }

    #[test]
    fn index_mut_replaces_a_non_object_and_inserts_null() {
        let mut value = json("\"text\"");
        assert!(index_mut(&mut value, "a").is_null());
        assert_eq!(value, json("{\"a\": null}"));
        let mut value = json("{\"a\": 1}");
        assert_eq!(*index_mut(&mut value, "a"), json("1"));
        assert_eq!(value, json("{\"a\": 1}"));
    }

    #[test]
    fn member_never_changes_the_value() {
        let value = json("[1]");
        assert!(member(&value, "a").is_null());
        assert_eq!(value, json("[1]"));
        assert!(!has_member(&value, "a"));
        assert!(has_member(&json("{\"a\": null}"), "a"));
    }

    #[test]
    fn elements_walks_arrays_and_object_values_and_no_scalar() {
        assert_eq!(elements(&json("[1, 2]")).len(), 2);
        let object = json("{\"b\": 2, \"a\": 1}");
        assert_eq!(elements(&object), vec![&json("1"), &json("2")]);
        assert!(elements(&json("\"ab\"")).is_empty());
        assert!(elements(&json("null")).is_empty());
    }

    #[test]
    fn merge_recurses_into_objects_and_replaces_arrays_and_nulls() {
        let mut dst = json(r#"{"a": {"x": 1, "y": [1, 2]}, "b": 5, "keep": true}"#);
        let src = json(r#"{"a": {"y": [3], "z": null}, "b": null, "c": {"n": 1}}"#);
        recursive_merge(&mut dst, &src, None);
        assert_eq!(
            dst,
            json(r#"{"a": {"x": 1, "y": [3], "z": null}, "b": null, "keep": true, "c": {"n": 1}}"#)
        );
    }

    #[test]
    fn merge_replaces_an_object_by_a_scalar_and_a_scalar_by_an_object() {
        let mut dst = json(r#"{"a": {"x": 1}, "b": 2}"#);
        recursive_merge(&mut dst, &json(r#"{"a": 7, "b": {"y": 1}}"#), None);
        assert_eq!(dst, json(r#"{"a": 7, "b": {"y": 1}}"#));
        // An empty object replaces a scalar, and is copied where nothing was.
        let mut dst = json(r#"{"a": 7}"#);
        recursive_merge(&mut dst, &json(r#"{"a": {}, "b": {}}"#), None);
        assert_eq!(dst, json(r#"{"a": {}, "b": {}}"#));
    }

    #[test]
    fn merge_into_a_non_object_makes_it_an_object_first() {
        let mut dst = json("null");
        recursive_merge(&mut dst, &json(r#"{"a": 1}"#), None);
        assert_eq!(dst, json(r#"{"a": 1}"#));
        // A source without members leaves the target as it was.
        let mut dst = json("3");
        recursive_merge(&mut dst, &json("{}"), None);
        assert_eq!(dst, json("3"));
    }

    #[test]
    fn merge_stops_at_a_level_the_predicate_handled() {
        fn replace(dst: &mut Value, src: &Value) -> bool {
            if as_bool(member(src, "replace"), false) && has_member(src, "replace") {
                *dst = src.clone();
                return true;
            }
            false
        }
        let mut dst = json(r#"{"game": {"a": 1, "b": 2}, "menu": {"a": 1}}"#);
        let src = json(r#"{"game": {"replace": true, "b": 3}, "menu": {"b": 2}}"#);
        recursive_merge(&mut dst, &src, Some(replace));
        assert_eq!(
            dst,
            json(r#"{"game": {"replace": true, "b": 3}, "menu": {"a": 1, "b": 2}}"#)
        );
    }
}
