//! Unknown members: version structs have no flatten field, so
//! `unknown_members(T::SHAPE, &Value)` walks the struct shapes and a `Value` of the same text
//! together and reports every key no field takes; `keep_unknown_members` also stores them in the
//! `#[facet(skip)] extra: Extra` of the struct they belong to, for write-back.
//!
//! Each unknown member carries its JSON path (`description/color`, `events/1/target`).

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "tests read fixtures and call facet-json directly"
)]
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{
    MAX_JSON_DEPTH, StrictInt, Value, keep_unknown_members, to_json, too_deep_to_walk,
    unknown_members,
};
use facet::Facet;
use indexmap::IndexMap;

fn value(text: &str) -> Value {
    facet_json::from_str_borrowed_jsonc(text).unwrap()
}

/// The unknown members' values, in walk order.
fn unknown_values<'a, T: Facet<'a>>(text: &str) -> Vec<Value> {
    unknown_members(T::SHAPE, &value(text))
        .into_iter()
        .map(|(_, v)| v)
        .collect()
}

#[derive(Debug, Default, Facet)]
struct Description {
    identifier: String,
    #[facet(rename = "is_spawnable", default)]
    spawnable: bool,
    #[facet(skip)]
    extra: Extra,
}

#[derive(Debug, Default, Facet)]
struct Component {
    #[facet(default)]
    value: f32,
    #[facet(skip)]
    extra: Extra,
}

#[derive(Debug, Default, Facet)]
struct Document {
    description: Description,
    #[facet(default)]
    components: IndexMap<String, Component>,
    #[facet(default)]
    events: Vec<Event>,
    #[facet(default)]
    optional: Option<Description>,
    #[facet(default)]
    opaque: Option<Value>,
    #[facet(default)]
    strict: Option<StrictInt<i32>>,
    #[facet(skip)]
    extra: Extra,
}

#[derive(Debug, Default, Facet)]
struct Event {
    #[facet(default)]
    name: String,
    #[facet(skip)]
    extra: Extra,
}

const DOCUMENT: &str = r#"{
  "description": { "identifier": "x:y", "is_spawnable": true, "color": "red" },
  "components": { "a": { "value": 1, "unit": "m" } },
  "events": [{ "name": "e" }, { "name": "f", "target": "self" }],
  "optional": { "identifier": "z", "weight": 2 },
  "opaque": { "anything": [1, 2] },
  "strict": 4,
  "root_extra": null
}"#;

#[test]
fn a_document_without_unknown_members_reports_none() {
    let text = r#"{ "description": { "identifier": "x:y" } }"#;
    assert!(unknown_values::<Document>(text).is_empty());
}

#[test]
fn unknown_members_are_found_at_every_level() {
    let values = unknown_values::<Document>(DOCUMENT);
    assert_eq!(
        values,
        [
            Value::from("red"),
            Value::from("m"),
            Value::from("self"),
            Value::from(2i64),
            Value::NULL,
        ]
    );
}

#[test]
fn unknown_members_are_reported_with_their_json_paths() {
    let paths: Vec<String> = unknown_members(Document::SHAPE, &value(DOCUMENT))
        .into_iter()
        .map(|(path, _)| path.to_string())
        .collect();
    assert_eq!(
        paths,
        [
            "description/color",
            "components/a/unit",
            "events/1/target",
            "optional/weight",
            "root_extra",
        ]
    );
}

#[test]
fn a_renamed_field_is_known_by_its_json_name() {
    let values = unknown_values::<Description>(r#"{ "identifier": "a", "spawnable": true }"#);
    assert_eq!(values, [Value::TRUE]);
}

#[test]
fn a_skipped_field_is_not_a_member() {
    // `extra` itself is `#[facet(skip)]`: a JSON key "extra" is unknown.
    let values = unknown_values::<Description>(r#"{ "identifier": "a", "extra": 1 }"#);
    assert_eq!(values, [Value::from(1i64)]);
}

#[test]
fn opaque_values_and_strict_scalars_are_not_walked() {
    let values = unknown_values::<Document>(
        r#"{ "description": { "identifier": "a" }, "opaque": { "x": { "y": 1 } }, "strict": 1 }"#,
    );
    assert!(values.is_empty());
}

#[test]
fn a_member_of_the_wrong_json_type_is_not_walked() {
    // A typed parse rejects it; the walk has nothing to compare.
    let values = unknown_values::<Document>(r#"{ "description": [1], "events": {} }"#);
    assert!(values.is_empty());
}

#[test]
fn keep_unknown_members_fills_extra_at_each_level() {
    let mut doc: Document = facet_json::from_str_borrowed_jsonc(DOCUMENT).unwrap();
    let found = keep_unknown_members(&mut doc, &value(DOCUMENT));
    assert_eq!(found.len(), 5);
    assert_eq!(
        doc.description.extra.get("color"),
        Some(&Value::from("red"))
    );
    assert_eq!(doc.events[0].extra.iter().count(), 0);
    assert_eq!(
        doc.events[1].extra.get("target"),
        Some(&Value::from("self"))
    );
    assert_eq!(
        doc.optional.as_ref().unwrap().extra.get("weight"),
        Some(&Value::from(2i64))
    );
    assert_eq!(doc.extra.get("root_extra"), Some(&Value::NULL));
}

/// facet's reflection cannot reach a map value mutably (rc.7 `PokeMap` has no `get_mut`), so an
/// unknown member inside a map entry is reported but not kept.
#[test]
fn an_unknown_member_inside_a_map_value_is_reported_but_not_kept_deviation() {
    let mut doc: Document = facet_json::from_str_borrowed_jsonc(DOCUMENT).unwrap();
    let found = keep_unknown_members(&mut doc, &value(DOCUMENT));
    assert!(found.iter().any(|(_, v)| *v == Value::from("m")));
    assert!(doc.components["a"].extra.is_empty());
}

#[test]
fn keeping_twice_replaces_rather_than_duplicates() {
    let mut doc: Document = facet_json::from_str_borrowed_jsonc(DOCUMENT).unwrap();
    keep_unknown_members(&mut doc, &value(DOCUMENT));
    keep_unknown_members(&mut doc, &value(DOCUMENT));
    assert_eq!(doc.description.extra.iter().count(), 1);
}

// --- untagged enums with an object arm -------------------------------------

#[derive(Debug, Facet)]
#[facet(untagged)]
#[repr(u8)]
#[allow(dead_code)]
enum Filter {
    Test(FilterTest),
    List(Vec<Filter>),
    Group(FilterGroup),
}

#[derive(Debug, Default, Facet)]
struct FilterTest {
    test: String,
    #[facet(default)]
    value: Option<Value>,
    #[facet(skip)]
    extra: Extra,
}

#[derive(Debug, Default, Facet)]
struct FilterGroup {
    #[facet(default)]
    all_of: Option<Vec<Filter>>,
    #[facet(default)]
    any_of: Option<Vec<Filter>>,
    #[facet(skip)]
    extra: Extra,
}

#[test]
fn o6_facet_reads_an_untagged_object_arm_with_an_unknown_key() {
    let filter: Filter =
        facet_json::from_str_borrowed_jsonc(r#"{ "test": "is_family", "subject_x": 1 }"#).unwrap();
    assert!(matches!(filter, Filter::Test(_)), "{filter:?}");
}

#[test]
fn o6_unknown_members_descends_into_the_object_arm_of_an_untagged_enum() {
    let values = unknown_values::<Filter>(r#"{ "test": "is_family", "subject_x": 1 }"#);
    assert_eq!(values, [Value::from(1i64)]);
    let values = unknown_values::<Filter>(
        r#"{ "all_of": [{ "test": "a", "u": 1 }, [{ "test": "b", "v": 2 }]], "w": 3 }"#,
    );
    assert_eq!(
        values,
        [Value::from(1i64), Value::from(2i64), Value::from(3i64)]
    );
}

#[test]
fn o6_keep_unknown_members_fills_the_extra_of_the_arm() {
    let text = r#"{ "any_of": [{ "test": "a", "u": 1 }], "w": 3 }"#;
    let mut filter: Filter = facet_json::from_str_borrowed_jsonc(text).unwrap();
    keep_unknown_members(&mut filter, &value(text));
    let Filter::Group(group) = &filter else {
        panic!("{filter:?}");
    };
    assert_eq!(group.extra.get("w"), Some(&Value::from(3i64)));
    let Filter::Test(leaf) = &group.any_of.as_ref().unwrap()[0] else {
        panic!("{group:?}");
    };
    assert_eq!(leaf.extra.get("u"), Some(&Value::from(1i64)));
}

// facet picks an untagged object arm by the first key, in document order, that leaves exactly one
// arm with that key (facet-solver `see_key`), not by the most known keys.

#[derive(Debug, Facet)]
#[facet(untagged)]
#[repr(u8)]
#[allow(dead_code)]
enum Split {
    A(Sa),
    B(Sb),
}

#[derive(Debug, Default, Facet)]
struct Sa {
    a: u8,
    #[facet(skip)]
    extra: Extra,
}

#[derive(Debug, Default, Facet)]
struct Sb {
    c: u8,
    d: u8,
    #[facet(skip)]
    extra: Extra,
}

const SPLIT: &str = r#"{ "a": 1, "c": 2, "d": 3 }"#;

#[test]
fn o6_facet_takes_the_arm_of_the_first_distinguishing_key() {
    let split: Split = facet_json::from_str(SPLIT).unwrap();
    assert!(matches!(split, Split::A(_)), "{split:?}");
}

#[test]
fn o6_unknown_members_follows_facets_arm() {
    let paths: Vec<String> = unknown_members(Split::SHAPE, &value(SPLIT))
        .into_iter()
        .map(|(path, _)| path.to_string())
        .collect();
    assert_eq!(paths, ["c", "d"]);
}

#[test]
fn o6_keep_unknown_members_fills_the_arm_the_value_took_and_round_trips() {
    let mut split: Split = facet_json::from_str(SPLIT).unwrap();
    let found = keep_unknown_members(&mut split, &value(SPLIT));
    assert_eq!(found.len(), 2);
    let Split::A(a) = &split else {
        panic!("{split:?}");
    };
    assert_eq!(a.extra.get("a"), None);
    assert_eq!(a.extra.get("c"), Some(&Value::from(2i64)));
    let written = to_json(&split).unwrap();
    let again: Value = facet_json::from_str(&written).unwrap();
    assert_eq!(again, value(SPLIT));
    let reread: Split = facet_json::from_str(&written).unwrap();
    assert!(matches!(reread, Split::A(Sa { a: 1, .. })));
}

#[test]
fn o6_an_object_no_key_singles_out_reads_as_the_first_object_arm() {
    // No key leaves one arm: facet falls back to the first struct-like arm.
    let text = r#"{ "x": 1 }"#;
    let split: Split = facet_json::from_str(text).unwrap();
    assert!(matches!(split, Split::A(_)));
    let paths: Vec<String> = unknown_members(Split::SHAPE, &value(text))
        .into_iter()
        .map(|(path, _)| path.to_string())
        .collect();
    assert_eq!(paths, ["x"]);
}

// A key several remaining arms share with different value types: facet narrows the arms by the
// scalar value's type (facet-format `solve_variant`, `KeyResult::Ambiguous` then
// `satisfy_at_path`); when no arm's type takes the value it stops and reads the first object arm.

#[derive(Debug, Facet)]
#[facet(untagged)]
#[repr(u8)]
#[allow(dead_code)]
enum Keyed {
    Num(KeyNum),
    Text(KeyText),
}

#[derive(Debug, Default, Facet)]
struct KeyNum {
    key: u32,
    #[facet(default)]
    n: u8,
    #[facet(skip)]
    extra: Extra,
}

#[derive(Debug, Default, Facet)]
struct KeyText {
    key: String,
    #[facet(default)]
    t: u8,
    #[facet(skip)]
    extra: Extra,
}

#[derive(Debug, Facet)]
#[facet(untagged)]
#[repr(u8)]
#[allow(dead_code)]
enum KeyedOptional {
    Num(KeyOptionalNum),
    Text(KeyText),
}

#[derive(Debug, Default, Facet)]
struct KeyOptionalNum {
    #[facet(default)]
    key: Option<u32>,
    #[facet(default)]
    n: u8,
    #[facet(skip)]
    extra: Extra,
}

fn unknown_paths(shape: &'static facet::Shape, text: &str) -> Vec<String> {
    unknown_members(shape, &value(text))
        .into_iter()
        .map(|(path, _)| path.to_string())
        .collect()
}

#[test]
fn o6_an_ambiguous_key_takes_the_arm_whose_type_takes_its_scalar() {
    // `key` is in both arms; a string value leaves `Text` before `n` would single out `Num`.
    let text = r#"{ "key": "s", "n": 1, "t": 2 }"#;
    let keyed: Keyed = facet_json::from_str(text).unwrap();
    assert!(
        matches!(keyed, Keyed::Text(KeyText { t: 2, .. })),
        "{keyed:?}"
    );
    assert_eq!(unknown_paths(Keyed::SHAPE, text), ["n"]);
    // A number leaves `Num` before `t` would single out `Text`.
    let text = r#"{ "key": 5, "t": 2, "n": 1 }"#;
    let keyed: Keyed = facet_json::from_str(text).unwrap();
    assert!(
        matches!(keyed, Keyed::Num(KeyNum { key: 5, n: 1, .. })),
        "{keyed:?}"
    );
    assert_eq!(unknown_paths(Keyed::SHAPE, text), ["t"]);
}

#[test]
fn o6_an_ambiguous_key_no_arm_type_takes_reads_as_the_first_object_arm() {
    // `Option<u32>` is not a scalar type to the solver and `String` does not take `5`: no arm
    // satisfies the value, so facet stops narrowing and reads the first object arm, although `t`
    // would single out `Text`.
    let text = r#"{ "key": 5, "t": 2, "n": 1 }"#;
    let keyed: KeyedOptional = facet_json::from_str(text).unwrap();
    assert!(
        matches!(
            keyed,
            KeyedOptional::Num(KeyOptionalNum {
                key: Some(5),
                n: 1,
                ..
            })
        ),
        "{keyed:?}"
    );
    assert_eq!(unknown_paths(KeyedOptional::SHAPE, text), ["t"]);
}

/// A shell that holds members as raw slices of the text.
#[derive(Debug, Facet)]
struct RawShell<'a> {
    #[facet(default)]
    components: IndexMap<String, facet_json::RawJson<'a>>,
    #[facet(default)]
    raw: Option<facet_json::RawJson<'a>>,
    #[facet(skip)]
    extra: Extra,
}

/// A `RawJson` member is parsed on its own: its contents are no unknown members of the shell.
#[test]
fn members_inside_a_raw_json_are_not_reported() {
    let text = r#"{"components": {"a": {"x": 1}, "b": [{"y": 2}]}, "raw": {"z": 3}, "other": 4}"#;
    let paths: Vec<String> = unknown_members(RawShell::SHAPE, &value(text))
        .into_iter()
        .map(|(path, _)| path.to_string())
        .collect();
    assert_eq!(paths, ["other"]);
    let mut shell: RawShell<'_> = facet_json::from_str_borrowed_jsonc(text).unwrap();
    let kept = keep_unknown_members(&mut shell, &value(text));
    assert_eq!(kept.len(), 1);
    assert!(shell.extra.get("other").is_some());
}

/// A struct that holds itself: the walk goes a level down for each `a`.
#[derive(Debug, Default, Facet)]
struct Nested {
    #[facet(default)]
    a: Option<Box<Nested>>,
}

/// `{"zz": 1}` as the innermost of `levels` objects, each the `a` of the one around it; built
/// by hand, without a call for each level, since no reader reads a text past the limit.
fn nested_objects(levels: usize) -> Value {
    let mut innermost = facet_value::VObject::new();
    innermost.insert("zz", Value::from(1u64));
    let mut value: Value = innermost.into();
    for _ in 1..levels {
        let mut object = facet_value::VObject::new();
        object.insert("a", value);
        value = object.into();
    }
    value
}

/// The walk's limit is the readers': a value of `MAX_JSON_DEPTH` levels is walked to its
/// innermost member, and of a value one level deeper nothing is reported (a pin).
#[test]
fn a_value_at_the_depth_limit_is_walked_and_one_level_deeper_is_not() {
    let at_the_limit = nested_objects(MAX_JSON_DEPTH);
    let unknown = unknown_members(Nested::SHAPE, &at_the_limit);
    assert_eq!(unknown.len(), 1, "128 levels: the innermost member");
    let (path, value) = &unknown[0];
    assert_eq!(path.len(), MAX_JSON_DEPTH);
    assert_eq!(path.last_key(), Some("zz"));
    assert_eq!(value, &Value::from(1u64));

    let past_the_limit = nested_objects(MAX_JSON_DEPTH + 1);
    assert!(unknown_members(Nested::SHAPE, &past_the_limit).is_empty());
    let mut target = Nested::default();
    assert!(keep_unknown_members(&mut target, &past_the_limit).is_empty());
    assert!(target.a.is_none());
}

/// "Nothing reported" of a value past the limit is told apart from "no unknown member": the
/// walk says which values it does not walk, on both sides of the limit.
#[test]
fn a_value_the_walk_does_not_walk_is_told_apart_from_one_without_unknown_members() {
    let at_the_limit = nested_objects(MAX_JSON_DEPTH);
    assert!(!too_deep_to_walk(&at_the_limit), "128 levels are walked");
    let past_the_limit = nested_objects(MAX_JSON_DEPTH + 1);
    assert!(
        too_deep_to_walk(&past_the_limit),
        "129 levels are not walked, and the caller is not told"
    );
    assert!(!too_deep_to_walk(&Value::from(1u64)), "a scalar");
}
