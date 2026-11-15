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
//!   the `null` as written); a flattened struct's members are the struct's own. A family
//!   with such a member whose inside is read through `JsonCpp` handles it itself: in the proxy's
//!   conversion (read a `null` bound as the bound left out), or by taking the `null` members
//!   out of that object before the typed read; and states which it chose.

use facet::{Def, Facet, Field, Peek, Shape, StructKind, Type, UserType, Variant};

use super::Value;
use super::slice::{JsonSlice, SliceError, parse_without, span_at};
use super::walk::Walk;
use crate::diagnostic::codes as core_codes;
use crate::diagnostic::{
    Code, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, JsonPathSegment, Location, Phase,
    Severity,
};
use crate::json::arms::{Lookup, find_field, struct_fields, untagged_arm};

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
    Walk::new(T::SHAPE, json)
        .typed(Some(Peek::new(value)))
        .run()
        .nulls
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
            Code::from(&core_codes::INVALID_TYPE),
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
/// absent because they were written as `null` ([`remove_null_members`], [`null_members_of_shape`]): an
/// information `ValueRewritten` with the note [`NULL_MEMBER_READ_AS_ABSENT`]. The game says
/// nothing there; the crate writes the member's default back where the file has `null`, and a
/// place the write-back changes carries a finding, of the lowest severity, a lint.
pub fn tell_null_members(found: &[JsonPath], at: &JsonPath, d: &mut Diagnostics) {
    for path in found {
        let mut full = at.clone();
        full.append(path);
        let mut finding = Diagnostic::new(
            Code::from(&core_codes::VALUE_REWRITTEN),
            Severity::Info,
            Phase::Parse,
            Location::new(None, None, full, None),
        );
        finding.detail = Some(Box::new(DiagnosticDetail::Note(NULL_MEMBER_READ_AS_ABSENT)));
        d.push(finding);
    }
}

/// [`null_members_of_shape`] as the walk names them.
pub(crate) fn member_paths(shape: &'static Shape, json: &Value) -> Vec<JsonPath> {
    Walk::new(shape, json).run().null_members
}

/// Removes from `json` the members [`null_members_of_shape`] finds, and returns their paths.
pub fn remove_null_members(shape: &'static Shape, json: &mut Value) -> Vec<JsonPath> {
    let found = null_members_of_shape(shape, json);
    remove_members(json, &found);
    found
}

/// Every struct member of `json`, read as `shape`, written as `null` (rule N1's absent
/// members), in document order: the members of each struct the shape reads an object as,
/// through `Option`s, pointers, transparent newtypes, flattened structs (whose members are the
/// struct's own), list elements, map values and enum variants (the arm an untagged enum takes,
/// the variant a tag or a single key names). Map entries, list elements and the inside of an
/// untyped value or a proxied shape are not members.
///
/// Runs on a stack the depth of `json` can take; a value nested past `MAX_JSON_DEPTH` has none.
pub fn null_members_of_shape(shape: &'static Shape, json: &Value) -> Vec<JsonPath> {
    let Ok(levels) = super::write::levels_of(json) else {
        return Vec::new();
    };
    super::stack::with_stack_for(levels, || {
        let mut found = Vec::new();
        nulls_in(shape, json, &mut JsonPath::new(), &mut found);
        found
    })
}

/// The null members of `json` read as `shape`, at `path`.
fn nulls_in(shape: &'static Shape, json: &Value, path: &mut JsonPath, found: &mut Vec<JsonPath>) {
    if shape.effective_proxy(None).is_some() || shape.id == Value::SHAPE.id {
        return;
    }
    let element = match shape.def {
        Def::Option(option) => {
            if !json.is_null() {
                nulls_in(option.t(), json, path, found);
            }
            return;
        }
        Def::Pointer(pointer) => {
            if let Some(pointee) = pointer.pointee() {
                nulls_in(pointee, json, path, found);
            }
            return;
        }
        Def::Map(map) => {
            for (key, item) in json.as_object().into_iter().flatten() {
                path.push(key.as_str());
                nulls_in(map.v(), item, path, found);
                path.pop();
            }
            return;
        }
        Def::List(list) => Some(list.t()),
        Def::Array(array) => Some(array.t()),
        Def::Slice(slice) => Some(slice.t()),
        Def::Set(set) => Some(set.t()),
        _ => None,
    };
    if let Some(element) = element {
        for (index, item) in json.as_array().into_iter().flatten().enumerate() {
            path.push_index(index);
            nulls_in(element, item, path, found);
            path.pop();
        }
        return;
    }
    if shape.scalar_type().is_some() {
        return;
    }
    match shape.ty {
        Type::User(UserType::Struct(st)) if shape.is_transparent() && st.fields.len() == 1 => {
            nulls_in(st.fields[0].shape(), json, path, found);
        }
        Type::User(UserType::Struct(st)) => nulls_of_members(st.fields, json, path, found),
        Type::User(UserType::Enum(en)) => nulls_of_enum(shape, en.variants, json, path, found),
        _ => {}
    }
}

/// The null members of an object read as the named `fields`.
fn nulls_of_members(
    fields: &'static [Field],
    json: &Value,
    path: &mut JsonPath,
    found: &mut Vec<JsonPath>,
) {
    for (key, item) in json.as_object().into_iter().flatten() {
        path.push(key.as_str());
        null_member(fields, key.as_str(), item, path, found);
        path.pop();
    }
}

/// The member `key: item` of an object read as `fields`: a field's (null itself, and the null
/// members below it), or a flattened struct's, at the same path.
fn null_member(
    fields: &'static [Field],
    key: &str,
    item: &Value,
    path: &mut JsonPath,
    found: &mut Vec<JsonPath>,
) {
    match find_field(fields, key) {
        Lookup::Field(_, field) => {
            if item.is_null() {
                found.push(path.clone());
            }
            if field.proxy().is_none() {
                nulls_in(field.shape(), item, path, found);
            }
        }
        Lookup::Flattened(_, flattened) => {
            if let Some(inner) = struct_fields(flattened.shape()) {
                null_member(inner, key, item, path, found);
            }
        }
        Lookup::Unknown => {}
    }
}

/// The null members of an enum's payload: the arm an untagged enum reads `json` as, the
/// variant an internal tag names (its fields beside the tag), or the one key of an external tag.
fn nulls_of_enum(
    shape: &'static Shape,
    variants: &'static [Variant],
    json: &Value,
    path: &mut JsonPath,
    found: &mut Vec<JsonPath>,
) {
    let by_name = |name: &str| variants.iter().find(|v| v.effective_name() == name);
    if shape.is_untagged() {
        if let Some(index) = untagged_arm(variants, json) {
            nulls_of_variant(&variants[index], json, path, found);
        }
        return;
    }
    let Some(object) = json.as_object() else {
        return;
    };
    if let Some(tag) = shape.get_tag_attr() {
        let Some(variant) = object
            .get(tag)
            .and_then(Value::as_string)
            .and_then(|name| by_name(name.as_str()))
        else {
            return;
        };
        let mut rest = object.clone();
        rest.remove(tag);
        nulls_of_variant(variant, &Value::from(rest), path, found);
        return;
    }
    let mut members = object.iter();
    if let (Some((name, payload)), None) = (members.next(), members.next())
        && let Some(variant) = by_name(name.as_str())
    {
        path.push(name.as_str());
        nulls_of_variant(variant, payload, path, found);
        path.pop();
    }
}

/// A variant's payload: a struct variant's members, or a newtype variant's one field.
fn nulls_of_variant(
    variant: &'static Variant,
    json: &Value,
    path: &mut JsonPath,
    found: &mut Vec<JsonPath>,
) {
    let fields = variant.data.fields;
    match variant.data.kind {
        StructKind::Struct => nulls_of_members(fields, json, path, found),
        StructKind::TupleStruct | StructKind::Tuple if fields.len() == 1 => {
            nulls_in(fields[0].shape(), json, path, found);
        }
        _ => {}
    }
}

/// Removes from `json` the struct members at `found` ([`member_paths`]).
pub(crate) fn remove_members(json: &mut Value, found: &[JsonPath]) {
    for path in found {
        let Some(key) = path.last_key() else {
            continue;
        };
        let mut node = Some(&mut *json);
        for step in path.segments().take(path.len() - 1) {
            node = node.and_then(|value| match step {
                JsonPathSegment::Key(key) => value.as_object_mut()?.get_mut(key),
                JsonPathSegment::Index(index) => value.as_array_mut()?.get_mut(index),
            });
        }
        if let Some(object) = node.and_then(Value::as_object_mut) {
            object.remove(key);
        }
    }
}

/// [`parse_at`](super::parse_at) under rule N1 ([`NullRule::Absent`]): every struct member
/// [`null_members_of_shape`] finds is read as absent, so it keeps its default and a required one is
/// `MissingRequiredField`. The text is read with those members blanked to white space (and the
/// comma that joined one to its neighbour), so offsets and spans are the file's.
///
/// # Errors
///
/// Returns `SliceError` when `slice` does not read as `T` once its null members are blanked, with the span and path of the fault.
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
        assert!(found[0].is(&core_codes::INVALID_TYPE));
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
            code.id() == "core/missing_required_field",
            "{code:?} {error:?}"
        );
        assert_eq!(error.path.to_string(), "inner/required");
    }

    #[test]
    fn null_members_are_struct_members_only() {
        let text = r#"{"days": null, "inner": {"required": 1, "count": null}, "any": {"k": null}}"#;
        assert_eq!(
            paths(&null_members_of_shape(Bare::SHAPE, &json(text))),
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

    #[derive(Debug, Facet)]
    struct Shaped {
        #[facet(default)]
        a: Option<u8>,
        #[facet(default, rename = "b_written", alias = "b_alias")]
        b: Option<Inner>,
        #[facet(flatten)]
        flat: Flat,
        #[facet(default)]
        wrapped: Option<Wrapped>,
        #[facet(default)]
        tagged: Option<Tagged>,
        #[facet(default)]
        external: Option<External>,
        #[facet(default)]
        either: Vec<Either>,
        #[facet(default)]
        boxed: Option<Box<Inner>>,
        #[facet(default)]
        map: IndexMap<String, Vec<Inner>>,
        #[facet(skip)]
        skipped: Option<u8>,
    }

    #[derive(Debug, Default, Facet)]
    struct Flat {
        #[facet(default)]
        f: Option<u8>,
    }

    #[derive(Debug, Facet)]
    #[facet(transparent)]
    struct Wrapped(Inner);

    #[derive(Debug, Facet)]
    #[repr(u8)]
    #[facet(tag = "type")]
    #[allow(dead_code)]
    enum Tagged {
        One { x: Option<u8> },
        Two { y: Option<u8> },
    }

    #[derive(Debug, Facet)]
    #[repr(u8)]
    #[allow(dead_code)]
    enum External {
        Named { x: Option<u8> },
        Newtype(Inner),
    }

    #[derive(Debug, Facet)]
    #[repr(u8)]
    #[facet(untagged)]
    #[allow(dead_code)]
    enum Either {
        Left { l: Option<u8> },
        Right { r: Option<u8>, more: Option<u8> },
        Many(Vec<Inner>),
    }

    /// The shape's own finder and the walk find the same members, in the same order.
    #[test]
    fn the_null_members_of_a_shape_are_the_walks() {
        let cases: &[(&str, &[&str])] = &[
            ("{}", &[]),
            ("null", &[]),
            (
                r#"{"a": null, "b_written": null, "b_alias": {"size": null}}"#,
                &["a", "b_written", "b_alias/size"],
            ),
            (r#"{"f": null, "skipped": null, "unknown": null}"#, &["f"]),
            (
                r#"{"wrapped": {"size": null}, "boxed": {"size": null}}"#,
                &["wrapped/size", "boxed/size"],
            ),
            (
                r#"{"tagged": {"type": "Two", "y": null, "x": null}}"#,
                &["tagged/y"],
            ),
            (r#"{"tagged": {"type": "None", "y": null}}"#, &[]),
            (
                r#"{"external": {"Named": {"x": null}}}"#,
                &["external/Named/x"],
            ),
            (
                r#"{"external": {"Newtype": {"size": null}}}"#,
                &["external/Newtype/size"],
            ),
            (
                r#"{"external": {"Named": {"x": null}, "Newtype": {}}}"#,
                &[],
            ),
            (
                r#"{"either": [{"l": null}, {"r": 1, "more": null}, [{"size": null}], 3]}"#,
                &["either/0/l", "either/1/more", "either/2/0/size"],
            ),
            (
                r#"{"map": {"k": [{"size": null}, null], "j": null}}"#,
                &["map/k/0/size"],
            ),
            (
                r#"{"a": 1, "b_written": [null], "wrapped": null, "map": []}"#,
                &["wrapped"],
            ),
        ];
        for (text, expected) in cases {
            let value = json(text);
            let found = null_members_of_shape(Shaped::SHAPE, &value);
            assert_eq!(paths(&found), *expected, "{text}");
            assert_eq!(
                found,
                member_paths(Shaped::SHAPE, &value),
                "{text}: the walk"
            );
        }
        for text in [
            r#"{"days": null, "inner": {"required": 1, "count": null}, "any": {"k": null}}"#,
            r#"{"flag": null, "inner": null, "any": null}"#,
        ] {
            let value = json(text);
            assert_eq!(
                null_members_of_shape(Bare::SHAPE, &value),
                member_paths(Bare::SHAPE, &value),
                "{text}"
            );
            assert_eq!(
                null_members_of_shape(Optional::SHAPE, &value),
                member_paths(Optional::SHAPE, &value),
                "{text}"
            );
        }
    }
}
