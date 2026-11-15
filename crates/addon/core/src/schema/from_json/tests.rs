//! Unit tests of the reader: one per behaviour of the derived and core reads.

use facet::Facet;
use indexmap::IndexMap;

use super::FromJson;
use crate::diagnostic::{Diagnostics, JsonPath};
use crate::document::Extra;
use crate::json::kit::{Node, Order, Read};
use crate::json::{JsonSlice, Value, from_str_jsonc};
use crate::schema::{self, Schema};
use crate::versioning::{MemberFailure, NullHandling, Reading, Scalars};

/// Reads `text` as `T` at `root` under `reading` in `order`: the value and each finding as
/// `severity path code`.
fn read_as<T: FromJson>(text: &str, reading: Reading, order: Order) -> (Option<T>, Vec<String>) {
    let json: Value = from_str_jsonc(text).expect("JSON");
    let node = Node::with_text(&json, JsonPath::of_keys(["root"]), JsonSlice::new(text));
    let mut found = Diagnostics::default();
    let mut read = Read::new(reading, &mut found).order(order);
    let value = T::from_json(&node, &mut read);
    read.finish();
    let told = found
        .iter()
        .map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code.id()))
        .collect();
    (value, told)
}

fn door<T: FromJson>(text: &str) -> (Option<T>, Vec<String>) {
    read_as(text, Reading::DOCUMENT, Order::Door)
}

fn as_read<T: FromJson>(text: &str) -> (Option<T>, Vec<String>) {
    read_as(text, Reading::DOCUMENT, Order::AsRead)
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Lamp {
    level: u8,
    #[facet(schema::fallback = false)]
    lit: Option<bool>,
    #[facet(default = 7)]
    radius: u32,
    #[facet(rename = "colour")]
    color: Option<String>,
}

#[test]
fn a_struct_reads_its_members_by_name_absent_ones_by_their_declaration() {
    let (lamp, told) = door::<Lamp>(r#"{"level": 3, "colour": "red"}"#);
    assert!(told.is_empty(), "{told:?}");
    assert_eq!(
        lamp,
        Some(Lamp {
            level: 3,
            lit: None,
            radius: 7,
            color: Some("red".to_owned())
        })
    );
}

#[test]
fn a_missing_required_member_fails_its_owner() {
    let (lamp, told) = door::<Lamp>(r#"{"lit": true}"#);
    assert_eq!(lamp, None);
    assert_eq!(told, ["Error root/level core/missing_required_field"]);
}

#[test]
fn the_door_order_stops_at_the_first_failure_in_file_order() {
    let text = r#"{"radius": "far", "level": true, "lit": 1}"#;
    let (lamp, told) = door::<Lamp>(text);
    assert_eq!(lamp, None);
    assert_eq!(told, ["Error root/radius core/invalid_type"]);
    // Read as read, every failure is told, in file order.
    let (_, told) = as_read::<Lamp>(text);
    assert_eq!(
        told,
        [
            "Error root/radius core/invalid_type",
            "Error root/level core/invalid_type",
            "Error root/lit core/invalid_type"
        ]
    );
}

#[test]
fn unknown_members_are_told_after_the_read_in_the_door_order_and_where_met_as_read() {
    let text = r#"{"zz": 1, "level": "x"}"#;
    // The door tells only its failure.
    assert_eq!(door::<Lamp>(text).1, ["Error root/level core/invalid_type"]);
    assert_eq!(
        as_read::<Lamp>(text).1,
        [
            "Error root/zz core/unrecognized_field",
            "Error root/level core/invalid_type"
        ]
    );
    let text = r#"{"level": 1, "zz": 1, "lit": "true"}"#;
    // Converted, then told after the unknown member, as the door's passes run.
    assert_eq!(
        door::<Lamp>(text).1,
        [
            "Error root/zz core/unrecognized_field",
            "Error root/lit core/invalid_type"
        ]
    );
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Marked {
    #[facet(schema::fallback = 0)]
    v: Option<u8>,
    #[facet(skip, rename = "\u{0}extra")]
    extra: Extra,
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Holder {
    #[facet(schema::fallback = 0)]
    v: Option<u8>,
    marked: Option<Marked>,
    #[facet(schema::unknown = "keep")]
    kept: Option<Plain>,
    plain: Option<Plain>,
    boxed: Option<Box<Marked>>,
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Plain {
    #[facet(schema::fallback = 0)]
    v: Option<u8>,
}

#[test]
fn a_struct_that_keeps_puts_its_unknown_members_in_the_table_by_the_door_s_rule() {
    let text = r#"{"marked": {"a": 1}, "kept": {"b": 2}, "plain": {"c": 3}, "boxed": {"d": 4}}"#;
    let json: Value = from_str_jsonc(text).unwrap();
    let node = Node::with_text(&json, JsonPath::new(), JsonSlice::new(text));
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT, &mut found).order(Order::Door);
    assert!(Holder::from_json(&node, &mut read).is_some());
    read.finish();
    let kept: Vec<String> = found
        .unknown()
        .iter()
        .map(|(at, key, value)| format!("{at}/{key}={value:?}"))
        .collect();
    // Kept where marked or worded `keep`; reported in a plain struct and behind a box.
    assert_eq!(kept, ["kept/b=2", "marked/a=1"]);
    let told: Vec<String> = found
        .iter()
        .map(|f| format!("{} {}", f.location.json_path, f.code.id()))
        .collect();
    assert_eq!(
        told,
        [
            "marked/a core/unrecognized_field",
            "kept/b core/unrecognized_field",
            "plain/c core/unrecognized_field",
            "boxed/d core/unrecognized_field"
        ]
    );
}

#[test]
fn null_is_read_by_the_reading_s_null_rule() {
    let text = r#"{"level": 1, "lit": null}"#;
    let (lamp, told) = door::<Lamp>(text);
    assert_eq!(lamp, None);
    assert_eq!(told, ["Error root/lit core/invalid_type"]);
    let (lamp, told) = read_as::<Lamp>(text, Reading::NULL_ABSENT, Order::Door);
    assert_eq!(lamp.map(|lamp| lamp.lit), Some(None));
    assert_eq!(told, ["Info root/lit core/value_rewritten"]);
    let (lamp, told) = read_as::<Lamp>(
        text,
        Reading::DOCUMENT.with_null(NullHandling::Own),
        Order::Door,
    );
    assert_eq!(lamp.map(|lamp| lamp.lit), Some(None));
    assert!(told.is_empty());
    // Under the absent rule a required member written `null` is missing.
    let (lamp, told) = read_as::<Lamp>(r#"{"level": null}"#, Reading::NULL_ABSENT, Order::Door);
    assert_eq!(lamp, None);
    assert_eq!(told, ["Error root/level core/missing_required_field"]);
}

#[test]
fn a_null_error_waits_for_the_read_and_yields_to_a_failure() {
    // `lit: null` is refused after the read; the failure after it in the file is what is told.
    let (_, told) = door::<Lamp>(r#"{"lit": null, "level": "x"}"#);
    assert_eq!(told, ["Error root/level core/invalid_type"]);
    // Without one, the `null` errors are all that is told.
    let (_, told) = door::<Lamp>(r#"{"lit": null, "level": 1, "zz": 0}"#);
    assert_eq!(told, ["Error root/lit core/invalid_type"]);
}

#[test]
fn default_and_tell_resets_a_failed_member_that_can_take_its_default() {
    let reading = Reading::DOCUMENT.with_on_member_failure(MemberFailure::DefaultAndTell);
    let (lamp, told) = read_as::<Lamp>(
        r#"{"level": 2, "radius": "far", "lit": 3}"#,
        reading,
        Order::AsRead,
    );
    assert_eq!(
        lamp,
        Some(Lamp {
            level: 2,
            lit: None,
            radius: 7,
            color: None
        })
    );
    assert_eq!(
        told,
        [
            "Info root/radius core/invalid_type",
            "Info root/lit core/invalid_type"
        ]
    );
    // A required member has no default: its owner fails.
    let (lamp, _) = read_as::<Lamp>(r#"{"level": "x"}"#, reading, Order::AsRead);
    assert_eq!(lamp, None);
}

#[test]
fn the_door_defaults_a_refused_scalar_member_as_its_pre_pass_does() {
    let reading = Reading::DOCUMENT
        .with_scalars(Scalars::Strict)
        .with_on_member_failure(MemberFailure::DefaultAndTell);
    let (lamp, told) = read_as::<Lamp>(r#"{"level": 2, "radius": 1.5}"#, reading, Order::Door);
    assert_eq!(lamp.map(|lamp| lamp.radius), Some(7));
    assert_eq!(told, ["Info root/radius core/invalid_value"]);
    // A required one is then missing.
    let (lamp, told) = read_as::<Lamp>(r#"{"level": 2.5}"#, reading, Order::Door);
    assert_eq!(lamp, None);
    assert_eq!(told, ["Error root/level core/missing_required_field"]);
}

#[test]
fn scalars_read_by_every_profile() {
    let profiles = [
        Scalars::Strict,
        Scalars::JsonCpp,
        Scalars::NonStrict,
        Scalars::ConvertAndTell,
    ];
    let read = |text: &str| -> Vec<Option<u8>> {
        profiles
            .iter()
            .map(|scalars| {
                read_as::<u8>(text, Reading::DOCUMENT.with_scalars(*scalars), Order::Door).0
            })
            .collect()
    };
    assert_eq!(read("3"), [Some(3); 4]);
    assert_eq!(read("2.7"), [None, Some(2), Some(2), None]);
    assert_eq!(read("3.0"), [None, Some(3), Some(3), Some(3)]);
    assert_eq!(read("257"), [None, None, Some(1), None]);
    assert_eq!(read("true"), [None, None, Some(1), None]);
    // Converted, read, and told as a wrong type.
    let (value, told) = door::<u8>(r#""4""#);
    assert_eq!(value, Some(4));
    assert_eq!(told, ["Error root core/invalid_type"]);
    // Out of range for the plain read: an invalid value.
    assert_eq!(door::<u8>("256").1, ["Error root core/invalid_value"]);
}

#[test]
fn a_float_beyond_its_width_reads_the_infinity_and_is_told() {
    let (value, told) = door::<f32>("1e39");
    assert_eq!(value, Some(f32::INFINITY));
    assert_eq!(told, ["Info root core/beyond_float_member"]);
    let (value, told) = door::<f64>("1e39");
    assert_eq!(value, Some(1e39));
    assert!(told.is_empty());
}

#[test]
fn a_list_fails_with_an_element_or_drops_it_by_the_reading() {
    let (list, told) = as_read::<Vec<u8>>(r#"[1, "x", 3, true]"#);
    assert_eq!(list, None);
    assert_eq!(
        told,
        [
            "Error root/1 core/invalid_type",
            "Error root/3 core/invalid_type"
        ]
    );
    let reading = Reading::DOCUMENT.with_on_member_failure(MemberFailure::DropElement);
    let (list, _) = read_as::<Vec<u8>>(r#"[1, "x", 3]"#, reading, Order::AsRead);
    assert_eq!(list, Some(vec![1, 3]));
}

#[test]
fn maps_arrays_and_tuples_read_their_json_forms() {
    let (map, _) = door::<IndexMap<String, u8>>(r#"{"b": 2, "a": 1}"#);
    let map = map.unwrap();
    assert_eq!(map.keys().collect::<Vec<_>>(), ["b", "a"]);
    assert_eq!(door::<[u8; 2]>("[1, 2]").0, Some([1, 2]));
    assert_eq!(door::<[u8; 2]>("[1]").1, ["Error root core/invalid_type"]);
    assert_eq!(
        door::<(u8, String)>(r#"[1, "a"]"#).0,
        Some((1, "a".to_owned()))
    );
    assert_eq!(
        door::<IndexMap<String, u8>>("[]").1,
        ["Error root core/invalid_type"]
    );
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
enum Mode {
    Fast,
    SlowMotion,
}

#[test]
fn a_string_enum_reads_its_names() {
    assert_eq!(door::<Mode>(r#""slow_motion""#).0, Some(Mode::SlowMotion));
    assert_eq!(
        door::<Mode>(r#""Fast""#).1,
        ["Error root core/invalid_enum_value"]
    );
    assert_eq!(door::<Mode>("3").1, ["Error root core/invalid_enum_value"]);
    assert_eq!(door::<Mode>("[]").1, ["Error root core/invalid_type"]);
    assert_eq!(
        door::<Mode>(r#"{"fast": 1}"#).1,
        ["Error root/fast core/invalid_enum_value"]
    );
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(untagged)]
#[repr(u8)]
enum Amount {
    Count(u8),
    Name(String),
    Range { min: u8, max: u8 },
}

#[test]
fn an_untagged_enum_reads_the_arm_the_door_chooses() {
    assert_eq!(door::<Amount>("3").0, Some(Amount::Count(3)));
    assert_eq!(
        door::<Amount>(r#""many""#).0,
        Some(Amount::Name("many".to_owned()))
    );
    assert_eq!(
        door::<Amount>(r#"{"min": 1, "max": 2}"#).0,
        Some(Amount::Range { min: 1, max: 2 })
    );
    // The only object arm, its member missing.
    assert_eq!(
        door::<Amount>(r#"{"min": 1}"#).1,
        ["Error root/max core/missing_required_field"]
    );
    assert_eq!(door::<Amount>("[]").1, ["Error root core/invalid_type"]);
    // Under the strict profile a scalar no arm takes is refused at the enum.
    let strict = Reading::DOCUMENT.with_scalars(Scalars::Strict);
    assert_eq!(
        read_as::<Amount>("true", strict, Order::Door).1,
        ["Error root core/invalid_type"]
    );
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(tag = "type", rename_all = "snake_case")]
#[repr(u8)]
enum Shape {
    Circle { radius: u8 },
    Point,
}

#[test]
fn an_internally_tagged_enum_reads_the_variant_its_tag_names() {
    assert_eq!(
        door::<Shape>(r#"{"type": "circle", "radius": 2}"#).0,
        Some(Shape::Circle { radius: 2 })
    );
    assert_eq!(door::<Shape>(r#"{"type": "point"}"#).0, Some(Shape::Point));
    assert_eq!(
        door::<Shape>(r#"{"type": "square"}"#).1,
        ["Error root/type core/invalid_enum_value"]
    );
    assert_eq!(
        door::<Shape>(r#"{"radius": 2}"#).1,
        ["Error root/type core/missing_required_field"]
    );
    assert_eq!(
        door::<Shape>(r#"{"type": "point", "radius": 1}"#).1,
        ["Error root/radius core/unrecognized_field"]
    );
}

#[derive(Clone, Copy, Debug, PartialEq, Facet, Schema)]
#[repr(u8)]
enum Anchor {
    #[facet(rename = "absolute")]
    Absolute(i16),
    #[facet(rename = "bottom")]
    Bottom,
}

#[test]
fn an_externally_tagged_enum_reads_its_one_member() {
    assert_eq!(
        door::<Anchor>(r#"{"absolute": -4}"#).0,
        Some(Anchor::Absolute(-4))
    );
    assert_eq!(door::<Anchor>(r#""bottom""#).0, Some(Anchor::Bottom));
    assert_eq!(
        door::<Anchor>(r#"{"above": 1}"#).1,
        ["Error root/above core/invalid_enum_value"]
    );
    assert_eq!(
        door::<Anchor>(r#"{"absolute": 1, "bottom": 2}"#).1,
        ["Error root core/invalid_type"]
    );
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Outer {
    name: String,
    #[facet(flatten)]
    inner: Plain,
}

#[test]
fn a_flattened_struct_reads_its_members_from_its_owner_s_object() {
    let (outer, told) = door::<Outer>(r#"{"name": "a", "v": 3}"#);
    assert!(told.is_empty(), "{told:?}");
    assert_eq!(
        outer,
        Some(Outer {
            name: "a".to_owned(),
            inner: Plain { v: Some(3) }
        })
    );
    assert_eq!(
        door::<Outer>(r#"{"name": "a", "w": 3}"#).1,
        ["Error root/w core/unrecognized_field"]
    );
}

/// A level read as a word: `read_with`'s function reads the member.
fn level_word(node: &Node<'_>, read: &mut Read<'_>) -> Option<u8> {
    match read.string(node)? {
        "low" => Some(1),
        "high" => Some(9),
        _ => {
            read.refuse(node, (&crate::diagnostic::codes::INVALID_VALUE).into());
            None
        }
    }
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Worded {
    #[facet(schema::read_with = level_word)]
    level: u8,
    #[facet(schema::read_with = level_word)]
    other: Option<u8>,
}

#[test]
fn read_with_reads_a_member_by_the_family_s_function() {
    assert_eq!(
        door::<Worded>(r#"{"level": "high", "other": "low"}"#).0,
        Some(Worded {
            level: 9,
            other: Some(1)
        })
    );
    assert_eq!(
        door::<Worded>(r#"{"level": "mid"}"#).1,
        ["Error root/level core/invalid_value"]
    );
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(transparent)]
struct Meters(f32);

#[test]
fn a_transparent_newtype_reads_as_its_field() {
    assert_eq!(door::<Meters>("2.5").0, Some(Meters(2.5)));
}

#[test]
fn every_finding_spans_the_value_it_is_about() {
    let text = r#"{"level": "x"}"#;
    let json: Value = from_str_jsonc(text).unwrap();
    let node = Node::with_text(&json, JsonPath::new(), JsonSlice::new(text));
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT, &mut found).order(Order::Door);
    assert!(Lamp::from_json(&node, &mut read).is_none());
    let span = found.iter().next().unwrap().location.span.unwrap();
    assert_eq!(&text[span.offset as usize..][..span.len as usize], r#""x""#);
}

#[test]
fn unknown_first_tells_an_object_s_unknown_members_before_its_members() {
    let text = r#"{"level": "x", "zz": 1, "inner": 2}"#;
    let (_, told) = read_as::<Lamp>(text, Reading::DOCUMENT, Order::UnknownFirst);
    assert_eq!(
        told,
        [
            "Error root/zz core/unrecognized_field",
            "Error root/inner core/unrecognized_field",
            "Error root/level core/invalid_type"
        ]
    );
    // As read, where each is met.
    let (_, told) = as_read::<Lamp>(text);
    assert_eq!(
        told,
        [
            "Error root/level core/invalid_type",
            "Error root/zz core/unrecognized_field",
            "Error root/inner core/unrecognized_field"
        ]
    );
}

#[test]
fn a_fork_is_committed_or_dropped_whole() {
    let json: Value = from_str_jsonc(r#"{"zz": 1}"#).unwrap();
    let node = Node::new(&json, JsonPath::of_keys(["root"]));
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT.with_keeps_unknown(true), &mut found);
    // An attempt that keeps an unknown member and fails: dropped, nothing of it stays.
    let mut attempt = read.fork();
    assert!(Lamp::from_json(&node, &mut attempt).is_none());
    drop(attempt);
    assert!(!read.failed());
    // One that is committed: its findings and its kept member are the read's.
    let mut attempt = read.fork();
    assert!(Plain::from_json(&node, &mut attempt).is_some());
    read.commit(attempt);
    assert!(read.failed());
    assert_eq!(found.len(), 1);
    assert!(found.unknown().contains(&JsonPath::of_keys(["root"]), "zz"));
}
