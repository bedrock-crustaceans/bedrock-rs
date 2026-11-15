//! Unknown members, as the reader tells and keeps them: every key no field of the struct takes
//! is an unknown member, told with its JSON path (`description/color`, `events/1/target`), and
//! those of a struct marked with `#[facet(skip)] extra: Extra` are kept in the document's
//! `Unknown` table, by the path of their object, for write-back. An untagged enum is read
//! through the arm its keys choose (facet's solver, mirrored by the reader).

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, JsonPathSegment};
use bedrock_addon_core::document::{Extra, Unknown};
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::parse::{Options, parse};
use bedrock_addon_core::json::{Value, from_str_jsonc, write_json_with};
use bedrock_addon_core::schema::{FromJson, Schema, read_document};
use bedrock_addon_core::versioning::Reading;
use facet::Facet;
use indexmap::IndexMap;

fn value(text: &str) -> Value {
    from_str_jsonc(text).unwrap()
}

fn at(keys: &[&str]) -> JsonPath {
    JsonPath::of_keys(keys)
}

/// `text` read as `T` by the document reading: the value and the findings, the kept members
/// in their table.
fn read<T: FromJson>(text: &str) -> (Option<T>, Diagnostics) {
    let parsed = parse(text, Options::default()).expect("JSON");
    let mut d = Diagnostics::default();
    let value = {
        let mut read = Read::new(Reading::DOCUMENT, &mut d);
        read_document::<T>(&Node::parsed(&parsed, JsonPath::new()), &mut read)
    };
    (value, d)
}

/// The paths of the unknown members the read of `text` as `T` tells, in its order.
fn unknown_paths<T: FromJson>(text: &str) -> Vec<String> {
    let (_, d) = read::<T>(text);
    d.iter()
        .filter(|f| f.is(&core_codes::UNRECOGNIZED_FIELD))
        .map(|f| f.location.json_path.to_string())
        .collect()
}

/// The value at `path` of `json`.
fn value_at<'v>(json: &'v Value, path: &JsonPath) -> Option<&'v Value> {
    let mut at = json;
    for segment in path.segments() {
        at = match segment {
            JsonPathSegment::Key(key) => at.as_object()?.get(key)?,
            JsonPathSegment::Index(index) => at.as_array()?.get(index)?,
        };
    }
    Some(at)
}

/// The unknown members' values, in the read's order.
fn unknown_values<T: FromJson>(text: &str) -> Vec<Value> {
    let (_, d) = read::<T>(text);
    let json = value(text);
    d.iter()
        .filter(|f| f.is(&core_codes::UNRECOGNIZED_FIELD))
        .filter_map(|f| value_at(&json, &f.location.json_path).cloned())
        .collect()
}

/// `value` written with `unknown`'s members, as the crate writes a document.
fn written<'a, T: Facet<'a>>(value: &T, unknown: &Unknown) -> String {
    let mut out = Vec::new();
    let told = write_json_with(
        value,
        unknown,
        &mut out,
        bedrock_addon_core::json::WriteStyle::default(),
    )
    .unwrap();
    assert!(told.is_empty(), "{told:?}");
    String::from_utf8(out).unwrap()
}

#[derive(Debug, PartialEq, Facet, Schema)]
struct Description {
    #[facet(default)]
    identifier: String,
    #[facet(rename = "is_spawnable", default)]
    spawnable: bool,
    #[facet(skip)]
    extra: Extra,
}

#[derive(Debug, PartialEq, Facet, Schema)]
struct Component {
    #[facet(default)]
    value: f32,
    #[facet(skip)]
    extra: Extra,
}

#[derive(Debug, PartialEq, Facet, Schema)]
struct Document {
    #[facet(default)]
    description: Description,
    #[facet(default)]
    components: IndexMap<String, Component>,
    #[facet(default)]
    events: Vec<Event>,
    optional: Option<Description>,
    opaque: Option<Value>,
    strict: Option<i32>,
    #[facet(skip)]
    extra: Extra,
}

#[derive(Debug, PartialEq, Facet, Schema)]
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
    assert_eq!(
        unknown_paths::<Document>(DOCUMENT),
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
    // The read refuses it; it has no members to compare.
    let values = unknown_values::<Document>(r#"{ "description": [1], "events": {} }"#);
    assert!(values.is_empty());
}

#[test]
fn keep_unknown_members_keeps_them_at_each_level() {
    let (doc, d) = read::<Document>(DOCUMENT);
    assert!(doc.is_some());
    let found = d
        .iter()
        .filter(|f| f.is(&core_codes::UNRECOGNIZED_FIELD))
        .count();
    assert_eq!(found, 5);
    let kept = d.unknown();
    assert_eq!(
        kept.get(&at(&["description"]), "color"),
        Some(&Value::from("red"))
    );
    assert_eq!(kept.members_at(&at(&["events"]).index(0)).count(), 0);
    assert_eq!(
        kept.get(&at(&["events"]).index(1), "target"),
        Some(&Value::from("self"))
    );
    assert_eq!(
        kept.get(&at(&["optional"]), "weight"),
        Some(&Value::from(2i64))
    );
    assert_eq!(kept.get(&JsonPath::new(), "root_extra"), Some(&Value::NULL));
}

/// An unknown member inside a map entry is told but not kept: the map field says no `keep`.
#[test]
fn an_unknown_member_inside_a_map_value_is_reported_but_not_kept_deviation() {
    let (_, d) = read::<Document>(DOCUMENT);
    assert!(d.iter().any(|f| f.is(&core_codes::UNRECOGNIZED_FIELD)
        && f.location.json_path == at(&["components", "a", "unit"])));
    assert_eq!(d.unknown().members_at(&at(&["components", "a"])).count(), 0);
}

// --- untagged enums with an object arm -------------------------------------

#[derive(Debug, PartialEq, Facet, Schema)]
#[facet(untagged)]
#[repr(u8)]
#[allow(dead_code)]
enum Filter {
    Test(FilterTest),
    List(Vec<Filter>),
    Group(FilterGroup),
}

#[derive(Debug, PartialEq, Facet, Schema)]
struct FilterTest {
    #[facet(default)]
    test: String,
    value: Option<Value>,
    #[facet(skip)]
    extra: Extra,
}

#[derive(Debug, PartialEq, Facet, Schema)]
struct FilterGroup {
    all_of: Option<Vec<Filter>>,
    any_of: Option<Vec<Filter>>,
    #[facet(skip)]
    extra: Extra,
}

#[test]
fn o6_facet_reads_an_untagged_object_arm_with_an_unknown_key() {
    let (filter, _) = read::<Filter>(r#"{ "test": "is_family", "subject_x": 1 }"#);
    let filter = filter.expect("reads");
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
fn o6_keep_unknown_members_keeps_the_members_of_the_arm() {
    let text = r#"{ "any_of": [{ "test": "a", "u": 1 }], "w": 3 }"#;
    let (filter, d) = read::<Filter>(text);
    let filter = filter.expect("reads");
    let kept = d.unknown();
    let Filter::Group(group) = &filter else {
        panic!("{filter:?}");
    };
    assert_eq!(kept.get(&JsonPath::new(), "w"), Some(&Value::from(3i64)));
    let Filter::Test(_) = &group.any_of.as_ref().unwrap()[0] else {
        panic!("{group:?}");
    };
    assert_eq!(
        kept.get(&at(&["any_of"]).index(0), "u"),
        Some(&Value::from(1i64))
    );
}

// The reader picks an untagged object arm as facet did: by the first key, in document order,
// that leaves exactly one arm with that key (facet-solver `see_key`), not by the most known keys.

#[derive(Debug, Facet, Schema)]
#[facet(untagged)]
#[repr(u8)]
#[allow(dead_code)]
enum Split {
    A(Sa),
    B(Sb),
}

#[derive(Debug, PartialEq, Facet, Schema)]
struct Sa {
    #[facet(default)]
    a: u8,
    #[facet(skip)]
    extra: Extra,
}

#[derive(Debug, PartialEq, Facet, Schema)]
struct Sb {
    #[facet(default)]
    c: u8,
    #[facet(default)]
    d: u8,
    #[facet(skip)]
    extra: Extra,
}

const SPLIT: &str = r#"{ "a": 1, "c": 2, "d": 3 }"#;

#[test]
fn o6_facet_takes_the_arm_of_the_first_distinguishing_key() {
    let (split, _) = read::<Split>(SPLIT);
    let split = split.expect("reads");
    assert!(matches!(split, Split::A(_)), "{split:?}");
}

#[test]
fn o6_unknown_members_follows_facets_arm() {
    assert_eq!(unknown_paths::<Split>(SPLIT), ["c", "d"]);
}

#[test]
fn o6_keep_unknown_members_fills_the_arm_the_value_took_and_round_trips() {
    let (split, d) = read::<Split>(SPLIT);
    let split = split.expect("reads");
    let found = d
        .iter()
        .filter(|f| f.is(&core_codes::UNRECOGNIZED_FIELD))
        .count();
    assert_eq!(found, 2);
    let Split::A(_) = &split else {
        panic!("{split:?}");
    };
    let kept = d.unknown();
    assert_eq!(kept.get(&JsonPath::new(), "a"), None);
    assert_eq!(kept.get(&JsonPath::new(), "c"), Some(&Value::from(2i64)));
    let written = written(&split, kept);
    let again: Value = value(&written);
    assert_eq!(again, value(SPLIT));
    let (reread, _) = read::<Split>(&written);
    assert!(matches!(reread, Some(Split::A(Sa { a: 1, .. }))));
}

#[test]
fn o6_an_object_no_key_singles_out_reads_as_the_first_object_arm() {
    // No key leaves one arm: the first struct-like arm.
    let text = r#"{ "x": 1 }"#;
    let (split, _) = read::<Split>(text);
    assert!(matches!(split, Some(Split::A(_))));
    assert_eq!(unknown_paths::<Split>(text), ["x"]);
}

// A key several remaining arms share with different value types: the arms are narrowed by the
// scalar value's type (facet-format `solve_variant`, `KeyResult::Ambiguous` then
// `satisfy_at_path`); when no arm's type takes the value the first object arm is read.

#[derive(Debug, Facet, Schema)]
#[facet(untagged)]
#[repr(u8)]
#[allow(dead_code)]
enum Keyed {
    Num(KeyNum),
    Text(KeyText),
}

#[derive(Debug, PartialEq, Facet, Schema)]
struct KeyNum {
    #[facet(default)]
    key: u32,
    #[facet(default)]
    n: u8,
    #[facet(skip)]
    extra: Extra,
}

#[derive(Debug, PartialEq, Facet, Schema)]
struct KeyText {
    #[facet(default)]
    key: String,
    #[facet(default)]
    t: u8,
    #[facet(skip)]
    extra: Extra,
}

#[derive(Debug, Facet, Schema)]
#[facet(untagged)]
#[repr(u8)]
#[allow(dead_code)]
enum KeyedOptional {
    Num(KeyOptionalNum),
    Text(KeyText),
}

#[derive(Debug, PartialEq, Facet, Schema)]
struct KeyOptionalNum {
    key: Option<u32>,
    #[facet(default)]
    n: u8,
    #[facet(skip)]
    extra: Extra,
}

#[test]
fn o6_an_ambiguous_key_takes_the_arm_whose_type_takes_its_scalar() {
    // `key` is in both arms; a string value leaves `Text` before `n` would single out `Num`.
    let text = r#"{ "key": "s", "n": 1, "t": 2 }"#;
    let (keyed, _) = read::<Keyed>(text);
    let keyed = keyed.expect("reads");
    assert!(
        matches!(keyed, Keyed::Text(KeyText { t: 2, .. })),
        "{keyed:?}"
    );
    assert_eq!(unknown_paths::<Keyed>(text), ["n"]);
    // A number leaves `Num` before `t` would single out `Text`.
    let text = r#"{ "key": 5, "t": 2, "n": 1 }"#;
    let (keyed, _) = read::<Keyed>(text);
    let keyed = keyed.expect("reads");
    assert!(
        matches!(keyed, Keyed::Num(KeyNum { key: 5, n: 1, .. })),
        "{keyed:?}"
    );
    assert_eq!(unknown_paths::<Keyed>(text), ["t"]);
}

#[test]
fn o6_an_ambiguous_key_no_arm_type_takes_reads_as_the_first_object_arm() {
    // `Option<u32>` is not a scalar type to the solver and `String` does not take `5`: no arm
    // satisfies the value, so the narrowing stops and the first object arm is read, although
    // `t` would single out `Text`.
    let text = r#"{ "key": 5, "t": 2, "n": 1 }"#;
    let (keyed, _) = read::<KeyedOptional>(text);
    let keyed = keyed.expect("reads");
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
    assert_eq!(unknown_paths::<KeyedOptional>(text), ["t"]);
}
