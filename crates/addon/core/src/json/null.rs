//! A member written as `null`, as the game's reader of a family reads it.
//!
//! facet reads a JSON `null` for an `Option` as `None`, the value an absent member gets, and
//! refuses it for a strict wrapper. The game decides by the reader its loader feeds
//! (the rules below):
//!
//! - **Rule N1, `JsonCpp` input** (entity, features, the item legacy
//!   component chain): the reader reports a member whose value is `null` as not found. A member
//!   written as `null` is an absent member: it keeps its default, and a
//!   required one is `missing required field`. [`NullRule::Absent`].
//! - **Rule N2, the Minecraft Document reader** (every other Cereal family: block, the modern
//!   item, biome, dimension, camera, trade, …): the strict reader finds the member, and its
//!   type reads `null` and fails: a type error that rejects the document. Only an untyped value
//!   (the game's dynamic value) holds a `null`. [`NullRule::Rejected`].
//!
//! # How a family opts in
//!
//! Nothing here runs by itself; a family calls the helper of its rule at each typed read
//! (the payload, each component, each slice it parses on its own):
//!
//! - N2: after the typed read, [`reject_nulls`] with the typed value and the slice it was read
//!   from. Every place where the value's shape reads a value and the JSON has `null` (an
//!   `Option` member read as `None`, and whatever else facet took from a `null`) is an
//!   `InvalidType` error at that member, which rejects the document. Not judged: a `null`
//!   where the shape is `Value`, `RawJson` or a proxied type itself (each is read and judged
//!   later by its own parse, or holds the `null`), a member with a field-level proxy, a unit,
//!   and `Option<Value>` (vanilla's dynamic value holds a `null`). Judged: `Option<RawJson>`
//!   and an `Option` of a proxied type (`Option<StrictBool>`, `Option<Molang>`): the `Option`
//!   is the member's own shape, and the game's `std::optional` rejects `null` too.
//! - N1: [`parse_at_null_absent`] in place of [`parse_at`](super::parse_at): every struct member
//!   written as `null` is read as absent (the text is read with those members blanked, so the
//!   spans still point into the file), and [`remove_null_members`] on the `Value` the family
//!   walks for unknown members, so that no `null` is told as a misread scalar. The paths it
//!   returns go to [`tell_null_members`]: the write-back writes the default where the file has
//!   `null`, and each such place carries an information.
//!
//!   The blanking follows the struct shapes only. It does not enter a proxied shape (a range
//!   written `{"min": null, "max": 3}`, a Molang field, a strict wrapper: the proxy receives
//!   the `null` as written) nor a flattened field (only the envelope shell has one). A family
//!   with such a member whose inside is read through `JsonCpp` handles it itself: in the proxy's
//!   conversion (read a `null` bound as the bound left out), or by taking the `null` members
//!   out of that object before the typed read; and states which it chose.

use facet::{Facet, Peek, Shape};

use super::Value;
use super::path::{Member, MemberPath};
use super::slice::{JsonSlice, SliceError, parse_without, span_at};
use super::unknown::walk_whole;
use crate::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Location, Phase, Severity,
};

/// How a family's reader takes a member written as `null`; the family's input kind decides
/// (module documentation).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum NullRule {
    /// `JsonCpp` input (rule N1): the member is absent.
    Absent,
    /// The Minecraft Document reader (rule N2): the member is present and its type rejects `null`.
    Rejected,
}

/// Every place in `json` where `value`, read from it, reads a value and the JSON has `null`:
/// an `Option` member left `None` by a `null`, and anything else facet took from one. Places
/// whose own shape the walk does not enter (`Value`, `RawJson`, a proxied type, a member with
/// a field-level proxy) are left out, and so is an `Option<Value>`, which holds the `null` as
/// written; an `Option<RawJson>` or an `Option` of a proxied type is found (module
/// documentation).
pub fn nulls_read<'f, T: Facet<'f>>(value: &T, json: &Value) -> Vec<JsonPath> {
    walk_whole(T::SHAPE, json, Some(Peek::new(value)), false).nulls
}

/// Rule N2 ([`NullRule::Rejected`]) for `value`, read from `slice`: each place [`nulls_read`]
/// finds is an `InvalidType` error at `at` followed by its path, with the span of the `null`.
/// Returns whether one was found; the errors reject the document.
pub fn reject_nulls<'f, T: Facet<'f>>(
    value: &T,
    slice: JsonSlice<'_>,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> bool {
    let Ok(json) = super::from_str_jsonc::<Value>(slice.text) else {
        return false;
    };
    let found = nulls_read(value, &json);
    for path in &found {
        let mut full = at.clone();
        full.append(path);
        d.push(Diagnostic::new(
            DiagnosticCode::InvalidType { text: None },
            Severity::Error,
            Phase::Parse,
            Location::new(None, None, full, span_at(slice, path)),
        ));
    }
    !found.is_empty()
}

/// The note of the information [`tell_null_members`] leaves at a member written as `null` that
/// rule N1 read as absent.
pub const NULL_MEMBER_READ_AS_ABSENT: &str = "the member written as `null` is read as absent; written back as its default, or left out for an optional member";

/// Rule N1's mark at each member in `found` (paths below `at`), the members a read took as
/// absent because they were written as `null` ([`remove_null_members`], [`null_members`]): an
/// information `ValueRewritten` with the note [`NULL_MEMBER_READ_AS_ABSENT`]. The game says
/// nothing there; the crate writes the member's default back where the file has `null`, and a
/// place the write-back changes carries a finding, of the lowest severity, a lint.
pub fn tell_null_members(found: &[JsonPath], at: &JsonPath, d: &mut Diagnostics) {
    for path in found {
        let mut full = at.clone();
        full.append(path);
        let mut finding = Diagnostic::new(
            DiagnosticCode::ValueRewritten { text: None },
            Severity::Info,
            Phase::Parse,
            Location::new(None, None, full, None),
        );
        finding.detail = Some(Box::new(DiagnosticDetail::Note(NULL_MEMBER_READ_AS_ABSENT)));
        d.push(finding);
    }
}

/// Every struct member of `json`, read as `shape`, written as `null` (rule N1's absent
/// members), in document order. Map entries, list elements and the inside of untyped values
/// are not members; the inside of a proxied shape or of a flattened field is not walked
/// (module documentation).
pub fn null_members(shape: &'static Shape, json: &Value) -> Vec<JsonPath> {
    member_paths(shape, json)
        .iter()
        .map(MemberPath::to_json_path)
        .collect()
}

/// [`null_members`] as the walk names them.
fn member_paths(shape: &'static Shape, json: &Value) -> Vec<MemberPath> {
    walk_whole(shape, json, None, false).null_members
}

/// Removes from `json` the members [`null_members`] finds, and returns their paths.
pub fn remove_null_members(shape: &'static Shape, json: &mut Value) -> Vec<JsonPath> {
    let found = member_paths(shape, json);
    for path in &found {
        let Some((Member::Key(key), within)) = path.steps().split_last() else {
            continue;
        };
        let mut node = Some(&mut *json);
        for step in within {
            node = node.and_then(|value| match step {
                Member::Key(key) => value.as_object_mut()?.get_mut(key),
                Member::Index(index) => value.as_array_mut()?.get_mut(*index),
            });
        }
        if let Some(object) = node.and_then(Value::as_object_mut) {
            object.remove(key);
        }
    }
    found.iter().map(MemberPath::to_json_path).collect()
}

/// [`parse_at`](super::parse_at) under rule N1 ([`NullRule::Absent`]): every struct member
/// [`null_members`] finds is read as absent, so it keeps its default and a required one is
/// `MissingRequiredField`. The text is read with those members blanked to white space (and the
/// comma that joined one to its neighbour), so offsets and spans are the file's.
pub fn parse_at_null_absent<'a, T: Facet<'a>>(slice: JsonSlice<'a>) -> Result<T, SliceError> {
    let absent = match super::from_str_jsonc::<Value>(slice.text) {
        Ok(json) => member_paths(T::SHAPE, &json),
        // Not JSON: the typed read reports it.
        Err(_) => Vec::new(),
    };
    parse_without(slice, &absent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::{StrictBool, StrictFloat, StrictInt, parse_at};
    use indexmap::IndexMap;

    /// A document of an MDI family in small: optional members of each kind.
    #[derive(Debug, Default, PartialEq, Facet)]
    struct Optional {
        #[facet(default)]
        flag: Option<StrictBool>,
        #[facet(default)]
        name: Option<String>,
        #[facet(default)]
        inner: Option<Inner>,
        #[facet(default)]
        list: Vec<Inner>,
        #[facet(default)]
        map: IndexMap<String, Inner>,
        #[facet(default)]
        any: Option<Value>,
        #[facet(default)]
        raw: Value,
    }

    #[derive(Debug, Default, PartialEq, Facet)]
    struct Inner {
        #[facet(default)]
        size: Option<StrictInt<u8>>,
    }

    /// A component of a `JsonCpp` family in small: bare strict members with defaults, and a
    /// required one.
    #[derive(Debug, PartialEq, Facet)]
    struct Bare {
        #[facet(default = StrictFloat(3.0_f32))]
        days: StrictFloat<f32>,
        #[facet(default)]
        flag: StrictBool,
        #[facet(default)]
        inner: Option<BareInner>,
        #[facet(default)]
        any: Value,
    }

    #[derive(Debug, PartialEq, Facet)]
    struct BareInner {
        required: StrictInt<i32>,
        #[facet(default = StrictInt(7))]
        count: StrictInt<i32>,
    }

    fn json(text: &str) -> Value {
        crate::json::from_str_jsonc(text).unwrap()
    }

    fn paths(found: &[JsonPath]) -> Vec<String> {
        found.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn facet_reads_a_null_option_member_as_an_absent_one() {
        // facet cannot tell a `null` member from an absent one.
        let absent: Optional = parse_at(JsonSlice::new("{}")).unwrap();
        let null: Optional = parse_at(JsonSlice::new(
            r#"{"flag": null, "name": null, "inner": null}"#,
        ))
        .unwrap();
        assert_eq!(
            null, absent,
            "facet tells a null Option member from an absent one"
        );
    }

    #[test]
    fn a_null_where_a_value_is_read_is_found_at_its_member() {
        let text = r#"{"flag": null, "name": "x", "inner": {"size": null},
            "list": [{"size": null}], "map": {"k": {"size": null}}, "any": null,
            "raw": {"deep": null}}"#;
        let value: Optional = parse_at(JsonSlice::new(text)).unwrap();
        assert_eq!(
            paths(&nulls_read(&value, &json(text))),
            ["flag", "inner/size", "list/0/size", "map/k/size"],
            "every null an Option took is a place; untyped values hold theirs"
        );
    }

    #[test]
    fn an_absent_member_is_no_null() {
        let value: Optional = parse_at(JsonSlice::new("{}")).unwrap();
        assert!(nulls_read(&value, &json("{}")).is_empty());
    }

    #[test]
    fn rejecting_nulls_is_an_invalid_type_error_at_the_member() {
        let text = r#"{"inner": {"size": null}}"#;
        let slice = JsonSlice::new(text);
        let value: Optional = parse_at(slice).unwrap();
        let mut d = Diagnostics::default();
        let mut at = JsonPath::new();
        at.push("minecraft:test");
        assert!(reject_nulls(&value, slice, &at, &mut d));
        let found: Vec<_> = d.iter().collect();
        assert_eq!(found.len(), 1, "{found:#?}");
        assert!(matches!(found[0].code, DiagnosticCode::InvalidType { .. }));
        assert_eq!(found[0].severity, Severity::Error);
        assert_eq!(
            found[0].location.json_path.to_string(),
            "minecraft:test/inner/size"
        );
        let span = found[0].location.span.expect("the null's span");
        assert_eq!(&text[span.offset as usize..][..4], "null");
    }

    #[test]
    fn a_bare_strict_member_refuses_null_without_the_rule() {
        // What rule N1 changes: facet's read refuses the `null`.
        let refused = parse_at::<Bare>(JsonSlice::new(r#"{"days": null}"#));
        assert!(refused.is_err(), "{refused:?}");
    }

    #[test]
    fn under_rule_n1_a_null_member_keeps_its_default() {
        for text in [
            r#"{"days": null}"#,
            r#"{"days": null, "flag": true}"#,
            r#"{"flag": true, "days": null}"#,
            r#"{"flag": true, "days": null, "inner": null}"#,
            "{\"days\" /* c, */ : null // d,\n , \"flag\": true}",
        ] {
            let bare: Bare = parse_at_null_absent(JsonSlice::new(text)).unwrap_or_else(|e| {
                panic!("{text}: {e:?}");
            });
            assert_eq!(bare.days, StrictFloat(3.0), "{text}");
            assert_eq!(bare.inner, None, "{text}");
        }
    }

    #[test]
    fn under_rule_n1_a_null_in_an_untyped_value_is_kept() {
        let bare: Bare = parse_at_null_absent(JsonSlice::new(r#"{"any": {"k": null}}"#)).unwrap();
        assert_eq!(bare.any, json(r#"{"k": null}"#));
    }

    #[test]
    fn under_rule_n1_a_null_required_member_is_missing() {
        let text = r#"{"inner": {"required": null, "count": null}}"#;
        let error = parse_at_null_absent::<Bare>(JsonSlice::new(text)).unwrap_err();
        let code =
            crate::diagnostic::code_for_deserialize_error_kind(&error.error.kind, true, false);
        assert!(
            matches!(code, DiagnosticCode::MissingRequiredField { .. }),
            "{code:?} {error:?}"
        );
        assert_eq!(error.path.to_string(), "inner/required");
    }

    #[test]
    fn null_members_are_struct_members_only() {
        let text = r#"{"days": null, "inner": {"required": 1, "count": null}, "any": {"k": null}}"#;
        assert_eq!(
            paths(&null_members(Bare::SHAPE, &json(text))),
            ["days", "inner/count"]
        );
        let mut value = json(text);
        let removed = remove_null_members(Bare::SHAPE, &mut value);
        assert_eq!(paths(&removed), ["days", "inner/count"]);
        assert_eq!(
            value,
            json(r#"{"inner": {"required": 1}, "any": {"k": null}}"#)
        );
    }

    #[test]
    fn under_rule_n1_a_borrowing_read_borrows_the_file_and_spans_are_the_files() {
        #[derive(Debug, Facet)]
        struct Shell<'a> {
            #[facet(default)]
            flag: StrictBool,
            raw: facet_json::RawJson<'a>,
        }
        let text = r#"{"flag": null, "raw": {"x": null}}"#;
        let slice = JsonSlice::new(text);
        let shell: Shell<'_> = parse_at_null_absent(slice).unwrap();
        let nested = slice
            .nested(&shell.raw)
            .expect("the raw slice borrows the file");
        assert_eq!(nested.text, r#"{"x": null}"#);
        let bad = r#"{"flag": null, "days": "x"}"#;
        let error = parse_at_null_absent::<Bare>(JsonSlice::new(bad)).unwrap_err();
        assert_eq!(error.path.to_string(), "days", "{error:?}");
        // The same place as the read of the text with a value of that length gives.
        let plain = parse_at::<Bare>(JsonSlice::new(r#"{"flag": true, "days": "x"}"#)).unwrap_err();
        assert_eq!(error.span, plain.span);
    }
}
