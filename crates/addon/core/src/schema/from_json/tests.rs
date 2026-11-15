//! Unit tests of the reader: one per behaviour of the derived and core reads.

use facet::Facet;
use indexmap::IndexMap;

use super::FromJson;
use crate::diagnostic::{Diagnostics, JsonPath};
use crate::document::Extra;
use crate::json::kit::{Node, Profile, Read, Recovery, Texts};
use crate::json::{JsonSlice, Value, from_str_jsonc};
use crate::schema::{self, Schema};
use crate::versioning::{MemberFailure, NullHandling, RangeMembers, Reading, Scalars};

/// Reads `text` as `T` at `root` under `reading` by `profile`: the value and each finding as
/// `severity path code`.
fn read_as<T: FromJson>(
    text: &str,
    reading: Reading,
    profile: Profile,
) -> (Option<T>, Vec<String>) {
    let json: Value = from_str_jsonc(text).expect("JSON");
    let node = Node::with_text(&json, JsonPath::of_keys(["root"]), JsonSlice::new(text));
    let mut found = Diagnostics::default();
    let mut read = Read::new(reading, &mut found).profile(profile);
    let value = T::from_json(&node, &mut read);
    read.finish();
    let told = found
        .iter()
        .map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code.id()))
        .collect();
    (value, told)
}

fn door<T: FromJson>(text: &str) -> (Option<T>, Vec<String>) {
    read_as(text, Reading::DOCUMENT, Profile::FIRST_FAILURE)
}

fn as_read<T: FromJson>(text: &str) -> (Option<T>, Vec<String>) {
    read_as(text, Reading::DOCUMENT, Profile::EVERY_FAILURE)
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
    let mut read = Read::new(Reading::DOCUMENT, &mut found).profile(Profile::FIRST_FAILURE);
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
    let (lamp, told) = read_as::<Lamp>(text, Reading::NULL_ABSENT, Profile::FIRST_FAILURE);
    assert_eq!(lamp.map(|lamp| lamp.lit), Some(None));
    assert_eq!(told, ["Info root/lit core/value_rewritten"]);
    let (lamp, told) = read_as::<Lamp>(
        text,
        Reading::DOCUMENT.with_null(NullHandling::Own),
        Profile::FIRST_FAILURE,
    );
    assert_eq!(lamp.map(|lamp| lamp.lit), Some(None));
    assert!(told.is_empty());
    // Under the absent rule a required member written `null` is missing.
    let (lamp, told) = read_as::<Lamp>(
        r#"{"level": null}"#,
        Reading::NULL_ABSENT,
        Profile::FIRST_FAILURE,
    );
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
        Profile::EVERY_FAILURE,
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
    let (lamp, _) = read_as::<Lamp>(r#"{"level": "x"}"#, reading, Profile::EVERY_FAILURE);
    assert_eq!(lamp, None);
}

#[test]
fn the_door_defaults_a_refused_scalar_member_as_its_pre_pass_does() {
    let reading = Reading::DOCUMENT
        .with_scalars(Scalars::Strict)
        .with_on_member_failure(MemberFailure::DefaultAndTell);
    let (lamp, told) = read_as::<Lamp>(
        r#"{"level": 2, "radius": 1.5}"#,
        reading,
        Profile::FIRST_FAILURE,
    );
    assert_eq!(lamp.map(|lamp| lamp.radius), Some(7));
    assert_eq!(told, ["Info root/radius core/invalid_value"]);
    // A required one is then missing.
    let (lamp, told) = read_as::<Lamp>(r#"{"level": 2.5}"#, reading, Profile::FIRST_FAILURE);
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
                read_as::<u8>(
                    text,
                    Reading::DOCUMENT.with_scalars(*scalars),
                    Profile::FIRST_FAILURE,
                )
                .0
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
    let (list, _) = read_as::<Vec<u8>>(r#"[1, "x", 3]"#, reading, Profile::EVERY_FAILURE);
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
    // facet read a `null` as a variant that is none.
    assert_eq!(
        door::<Mode>("null").1,
        ["Error root core/invalid_enum_value"]
    );
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
        read_as::<Amount>("true", strict, Profile::FIRST_FAILURE).1,
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
    // As the door tells it (schema_read_door's every_kind_of_member grid): a wrong type at
    // the enum's object.
    assert_eq!(
        door::<Shape>(r#"{"type": "square"}"#).1,
        ["Error root core/invalid_type"]
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
    let mut read = Read::new(Reading::DOCUMENT, &mut found).profile(Profile::FIRST_FAILURE);
    assert!(Lamp::from_json(&node, &mut read).is_none());
    let span = found.iter().next().unwrap().location.span.unwrap();
    assert_eq!(&text[span.offset as usize..][..span.len as usize], r#""x""#);
}

#[test]
fn unknown_first_tells_an_object_s_unknown_members_before_its_members() {
    let text = r#"{"level": "x", "zz": 1, "inner": 2}"#;
    let (_, told) = read_as::<Lamp>(text, Reading::DOCUMENT, Profile::UNKNOWN_FIRST);
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

/// The members of a component storage: a name that is none is an unrecognized component.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "component")]
struct Components {
    #[facet(rename = "minecraft:fly")]
    fly: Option<bool>,
}

#[test]
fn an_unknown_member_of_a_component_struct_is_an_unrecognized_component_and_not_kept() {
    let (value, told) = as_read::<Components>(r#"{"minecraft:fly": true, "minecraft:swim": {}}"#);
    assert_eq!(value, Some(Components { fly: Some(true) }));
    assert_eq!(
        told,
        ["Error root/minecraft:swim core/unrecognized_component"]
    );
    // The door's order tells it after the read, as it tells any unknown member.
    let (_, told) = door::<Components>(r#"{"minecraft:swim": {}}"#);
    assert_eq!(
        told,
        ["Error root/minecraft:swim core/unrecognized_component"]
    );
    // The text of an unknown field does not reword it: it is the game's own line.
    let json: Value = from_str_jsonc(r#"{"minecraft:swim": {}}"#).unwrap();
    let node = Node::new(&json, JsonPath::of_keys(["root"]));
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT, &mut found).texts(Texts {
        unknown: Some("not in the schema"),
        ..Texts::CODES
    });
    assert!(Components::from_json(&node, &mut read).is_some());
    assert!(
        found
            .iter()
            .all(|f| f.code.id() == "core/unrecognized_component")
    );
    // A reading that keeps unknown members does not keep it.
    let node = Node::new(&json, JsonPath::of_keys(["root"]));
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT.with_keeps_unknown(true), &mut found);
    assert!(Components::from_json(&node, &mut read).is_some());
    assert!(
        !found
            .unknown()
            .contains(&JsonPath::of_keys(["root"]), "minecraft:swim")
    );
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Projection {
    #[facet(schema::required)]
    field_of_view: Option<f32>,
    near: Option<f32>,
}

#[test]
fn a_required_option_must_be_written_and_reads_as_some() {
    let (value, told) = as_read::<Projection>(r#"{"field_of_view": 66, "near": 0.5}"#);
    assert!(told.is_empty(), "{told:?}");
    assert_eq!(
        value,
        Some(Projection {
            field_of_view: Some(66.0),
            near: Some(0.5)
        })
    );
    let (value, told) = as_read::<Projection>(r#"{"near": 0.5}"#);
    assert_eq!(value, None);
    assert_eq!(
        told,
        ["Error root/field_of_view core/missing_required_field"]
    );
}

/// A struct that holds itself, with members enough to make each level's read a large frame.
#[derive(Debug, PartialEq, Facet, Schema)]
struct Deep {
    a: Option<String>,
    b: Option<String>,
    c: Option<String>,
    d: Option<String>,
    e: Option<String>,
    f: Option<String>,
    g: Option<String>,
    h: Option<String>,
    i: Option<Vec<String>>,
    j: Option<Vec<String>>,
    k: Option<IndexMap<String, String>>,
    l: Option<IndexMap<String, String>>,
    next: Option<Vec<Deep>>,
}

/// `{"next": [` `levels` times around `{}`, closed: a text that nests twice `levels` deep.
fn deep_text(levels: usize) -> String {
    let mut text = "{\"next\": [".repeat(levels);
    text.push_str("{}");
    text.push_str(&"]}".repeat(levels));
    text
}

/// A payload nested to the parser's limit takes the derived read more stack than a thread of
/// 384 KiB has (some 4 KiB a level here, unoptimised): the read makes the stack it needs, as
/// the typed door's did.
#[test]
fn a_payload_as_deep_as_the_parser_takes_reads_on_a_small_thread() {
    use crate::load::LoadContext;
    use crate::version::{FormatVersion, FormatVersionInput};
    use crate::versioning::{ParseCx, read_payload};
    // The parser takes 128 levels: 64 objects, each holding an array.
    let text = deep_text(63);
    let read = std::thread::Builder::new()
        .stack_size(384 * 1024)
        .spawn(move || {
            let load = LoadContext::detached();
            let cx = ParseCx {
                load: &load,
                written: FormatVersionInput {
                    version: Some(FormatVersion::new(1, 0, 0)),
                    beta_literal: false,
                    prerelease: false,
                },
                original_version: FormatVersion::new(1, 0, 0),
                at: JsonPath::new(),
            };
            let mut d = Diagnostics::default();
            let deep: Option<Deep> = read_payload(JsonSlice::new(&text), &cx, &mut d);
            (deep.is_some(), d.len())
        })
        .expect("a thread")
        .join()
        .expect("no overflow");
    assert_eq!(read, (true, 0));
}

#[derive(Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
struct Keeper {
    a: u8,
}

/// The members a read keeps go to the table only when it reads: a document that fails keeps
/// nothing, as the door's read absorbs its kept members only after a typed read that worked.
#[test]
fn a_read_that_fails_keeps_no_unknown_member() {
    for profile in [Profile::FIRST_FAILURE, Profile::EVERY_FAILURE] {
        for (text, read) in [
            (r#"{"zz": 1, "a": "x"}"#, false),
            (r#"{"zz": 1, "a": 2}"#, true),
        ] {
            let json: Value = from_str_jsonc(text).unwrap();
            let node = Node::with_text(&json, JsonPath::new(), JsonSlice::new(text));
            let mut found = Diagnostics::default();
            found
                .unknown_mut()
                .put(&JsonPath::of_keys(["before"]), "k", Value::from(1));
            let mut reader = Read::new(Reading::DOCUMENT, &mut found).profile(profile);
            let value: Option<Keeper> = super::read_document(&node, &mut reader);
            assert_eq!(value.is_some(), read, "{profile:?} {text}");
            let kept: Vec<String> = found
                .unknown()
                .iter()
                .map(|(at, key, _)| format!("{at}/{key}"))
                .collect();
            let expected: &[&str] = if read {
                &["/zz", "before/k"]
            } else {
                &["before/k"]
            };
            assert_eq!(kept, expected, "{profile:?} {text}");
        }
    }
}

/// Each axis of a profile changes one thing: how many failures, the order of the members, the
/// place of the unknown ones, and whether what fails nothing waits for the end.
#[test]
fn each_axis_of_a_profile_changes_one_thing() {
    use crate::json::kit::{Failures, MemberOrder, Passes, UnknownAt};
    let text = r#"{"radius": "far", "zz": 1, "level": true}"#;
    let told = |profile: Profile| -> Vec<String> {
        read_as::<Lamp>(text, Reading::DOCUMENT, profile)
            .1
            .iter()
            .map(|line| line.rsplit_once(' ').map_or("", |(at, _)| at).to_owned())
            .collect()
    };
    let every = Profile::EVERY_FAILURE;
    assert_eq!(
        told(every),
        ["Error root/radius", "Error root/zz", "Error root/level"]
    );
    assert_eq!(
        told(Profile::DECLARED_ORDER),
        ["Error root/zz", "Error root/level", "Error root/radius"]
    );
    assert_eq!(
        told(every.with_members(MemberOrder::Declaration)),
        ["Error root/level", "Error root/radius", "Error root/zz"]
    );
    assert_eq!(
        told(every.with_unknown(UnknownAt::Last)),
        ["Error root/radius", "Error root/level", "Error root/zz"]
    );
    assert_eq!(
        told(every.with_passes(Passes::AfterRead)),
        ["Error root/radius", "Error root/level", "Error root/zz"]
    );
    assert_eq!(
        told(Profile::FIRST_FAILURE.with_members(MemberOrder::Declaration)),
        ["Error root/level"]
    );
    assert_eq!(
        told(every.with_failures(Failures::First)),
        ["Error root/radius"]
    );
}

/// A shade read only by its field's function: it has no `FromJson`.
#[derive(Clone, Debug, PartialEq, Facet)]
struct Shade(u8);

crate::impl_schema_leaf!(Shade);
crate::impl_no_molang!(Shade);

fn shade(node: &Node<'_>, read: &mut Read<'_>) -> Option<Shade> {
    level_word(node, read).map(Shade)
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Shaded {
    #[facet(schema::read_with = level_word)]
    level: Option<u8>,
    #[facet(schema::read_with = shade)]
    shade: Option<Shade>,
}

/// An `Option` field read by its own function takes a `null` by the reading's null rule, as
/// every `Option` field does, before the function sees anything: `None` where the reading
/// takes it, the type's wrong type where it refuses it.
#[test]
fn an_option_field_read_by_its_function_reads_null_by_the_null_rule() {
    let text = r#"{"level": null, "shade": null}"#;
    let (value, told) = read_as::<Shaded>(text, Reading::OWN, Profile::EVERY_FAILURE);
    assert!(told.is_empty(), "{told:?}");
    assert_eq!(
        value,
        Some(Shaded {
            level: None,
            shade: None
        })
    );
    let (value, told) = read_as::<Shaded>(text, Reading::DOCUMENT, Profile::EVERY_FAILURE);
    assert_eq!(value, None);
    assert_eq!(
        told,
        [
            "Error root/level core/invalid_type",
            "Error root/shade core/invalid_type"
        ]
    );
    // The door's null pass, as for any `Option` member.
    let (_, told) = read_as::<Shaded>(text, Reading::DOCUMENT, Profile::FIRST_FAILURE);
    assert_eq!(
        told,
        [
            "Error root/level core/invalid_type",
            "Error root/shade core/invalid_type"
        ]
    );
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Pairs {
    #[facet(schema::unknown = "keep")]
    pair: Option<[Plain; 1]>,
}

/// A struct inside a fixed array tells its unknown members and keeps none: the door's walk
/// cannot reach into a fixed array to keep them.
#[test]
fn a_struct_in_a_fixed_array_keeps_no_unknown_member() {
    let text = r#"{"pair": [{"c": 3}]}"#;
    let json: Value = from_str_jsonc(text).unwrap();
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT, &mut found);
    let value: Option<Pairs> = super::read_document(&Node::new(&json, JsonPath::new()), &mut read);
    assert!(value.is_some());
    assert!(found.unknown().is_empty(), "{:?}", found.unknown());
    assert_eq!(found.len(), 1);
}

/// A list that leaves a failed element out fails nothing: the read is as failed as before it.
#[test]
fn a_list_that_drops_an_element_leaves_the_read_unfailed() {
    let json: Value = from_str_jsonc(r#"[1, "x", 3]"#).unwrap();
    let mut found = Diagnostics::default();
    let reading = Reading::DOCUMENT.with_on_member_failure(MemberFailure::DropElement);
    let mut read = Read::new(reading, &mut found);
    let list = Vec::<u8>::from_json(&Node::new(&json, JsonPath::new()), &mut read);
    assert_eq!(list, Some(vec![1, 3]));
    assert!(!read.failed());
    let mut read = Read::new(reading, &mut found);
    let list = super::read_list(
        &Node::new(&json, JsonPath::new()),
        &mut read,
        super::ElementFailure::FailList,
        u8::from_json,
    );
    assert_eq!(list, None);
    assert!(read.failed());
}

/// A committed attempt brings the moves of the lists it shortened: a finding told later at an
/// element it renumbered is named as the file has it. A fork of a read that has stopped reads
/// nothing.
#[test]
fn a_committed_fork_keeps_its_moves_and_a_stopped_read_forks_stopped() {
    let json: Value = from_str_jsonc(r#"[1, "x", 3]"#).unwrap();
    let node = Node::new(&json, JsonPath::of_keys(["list"]));
    let mut found = Diagnostics::default();
    let reading = Reading::DOCUMENT.with_on_member_failure(MemberFailure::DropElement);
    let mut read = Read::new(reading, &mut found);
    let mut attempt = read.fork();
    assert_eq!(Vec::<u8>::from_json(&node, &mut attempt), Some(vec![1, 3]));
    read.commit(attempt);
    // The element held at 1 is the file's 2.
    let held = JsonPath::of_keys(["list"]).index(1);
    read.findings_mut()
        .push(crate::diagnostic::Diagnostic::of(&crate::diagnostic::codes::INVALID_VALUE).at(held));
    drop(read);
    found.name_as_the_file_has_it();
    let at: Vec<String> = found
        .iter()
        .map(|f| f.location.json_path.to_string())
        .collect();
    assert_eq!(at, ["list/1", "list/2"]);

    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT, &mut found).profile(Profile::FIRST_FAILURE);
    assert_eq!(String::from_json(&node, &mut read), None);
    assert!(read.stopped());
    let mut attempt = read.fork();
    assert_eq!(Vec::<Value>::from_json(&node, &mut attempt), None);
    assert!(attempt.stopped());
}

#[test]
fn an_object_given_to_a_string_enum_spans_its_first_member() {
    let text = r#"{"fast": 1}"#;
    let json: Value = from_str_jsonc(text).unwrap();
    let node = Node::with_text(&json, JsonPath::new(), JsonSlice::new(text));
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT, &mut found);
    assert_eq!(Mode::from_json(&node, &mut read), None);
    let span = found.iter().next().and_then(|f| f.location.span);
    assert_eq!(span.map(|s| (s.offset, s.len)), Some((9, 1)));
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(rename_all = "snake_case", schema::string_only)]
#[repr(u8)]
enum Bound {
    Strict,
    Loose,
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Coded {
    #[facet(schema::wrong_type = crate::diagnostic::codes::DROPPED_FIELD)]
    count: Option<u8>,
    #[facet(schema::missing = crate::diagnostic::codes::IGNORED_MEMBER)]
    name: String,
    #[facet(schema::invalid = crate::diagnostic::codes::INVALID_VALUE)]
    bound: Option<Bound>,
    #[facet(schema::finite)]
    far: Option<f32>,
    list: Option<Vec<u8>>,
}

/// A field's codes word its member's own findings, not those of a value inside it; `finite`
/// reads a float beyond its width as its largest; a string-only enum takes a string alone.
#[test]
fn a_field_words_its_member_s_findings_and_reads_its_float_finite() {
    let (value, told) =
        as_read::<Coded>(r#"{"count": [1], "bound": "neither", "far": -1e39, "list": [1, "x"]}"#);
    assert_eq!(value, None);
    assert_eq!(
        told,
        [
            "Error root/count core/dropped_field",
            "Error root/bound core/invalid_value",
            "Info root/far core/invalid_value",
            "Error root/list/1 core/invalid_type",
            "Error root/name core/ignored_member",
        ]
    );
    let (value, told) = as_read::<Coded>(r#"{"name": "n", "far": 1e39}"#);
    assert!(told.iter().all(|line| line.starts_with("Info")), "{told:?}");
    assert_eq!(value.and_then(|coded| coded.far), Some(f32::MAX));
    // A string-only enum: a number is a wrong type, where the plain enum tells an invalid
    // name; a field's `invalid` code words only what the enum refuses.
    let (_, told) = as_read::<Coded>(r#"{"name": "n", "bound": 3}"#);
    assert_eq!(told, ["Error root/bound core/invalid_type"]);
    assert_eq!(
        as_read::<Mode>("3").1,
        ["Error root core/invalid_enum_value"]
    );
}

/// A member's `wrong_type` code words a `null` its type refuses as it words any other wrong
/// type, told where the read tells its nulls (after it).
#[test]
fn a_null_a_member_refuses_is_told_under_its_wrong_type_code() {
    let (_, told) = door::<Coded>(r#"{"name": "n", "count": null}"#);
    assert_eq!(told, ["Info root/count core/dropped_field"]);
    let (_, told) = as_read::<Coded>(r#"{"name": "n", "count": null}"#);
    assert_eq!(told, ["Error root/count core/dropped_field"]);
}

/// The reading of the fixed-path readers (`json::kit::fixed_path::reader`): every finding an
/// information, nothing failing.
fn lenient() -> Reading {
    Reading::OWN.with_on_member_failure(MemberFailure::DefaultAndTell)
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Listed {
    #[facet(schema::iterated)]
    names: Option<Vec<String>>,
    #[facet(schema::elements = "drop")]
    counts: Option<Vec<u8>>,
    #[facet(alias = "colour")]
    color: Option<String>,
}

/// A list jsoncpp iterates reads whatever its member is: an array's elements, an object's
/// values in key order, nothing for another value, the member told as no array; a list whose
/// elements drop leaves a failed one out; an alias is a second name.
#[test]
fn a_list_reads_as_its_keys_say() {
    let read = |text: &str| read_as::<Listed>(text, lenient(), Profile::EVERY_FAILURE);
    let (value, told) = read(r#"{"names": {"b": "y", "a": "x"}, "counts": [1, "x", 3]}"#);
    assert_eq!(
        value,
        Some(Listed {
            names: Some(vec!["x".to_owned(), "y".to_owned()]),
            counts: Some(vec![1, 3]),
            color: None,
        })
    );
    assert_eq!(
        told,
        [
            "Info root/names core/invalid_type",
            "Info root/counts/1 core/invalid_type"
        ]
    );
    for text in [r#"{"names": 3}"#, r#"{"names": null}"#] {
        let (value, told) = read(text);
        assert_eq!(value.and_then(|v| v.names), Some(Vec::new()), "{text}");
        assert_eq!(told, ["Info root/names core/invalid_type"], "{text}");
    }
    let (value, told) = read(r#"{"colour": "red"}"#);
    assert!(told.is_empty(), "{told:?}");
    assert_eq!(value.and_then(|v| v.color).as_deref(), Some("red"));
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Texted {
    message: Option<String>,
}

/// jsoncpp's `asString`: a bool its name, anything else empty, told as a wrong type.
#[test]
fn a_string_read_as_jsoncpp_s_as_string_takes_any_value() {
    let reading = lenient().with_strings(crate::versioning::Strings::AsString);
    for (text, read) in [
        (r#"{"message": true}"#, "true"),
        (r#"{"message": 5}"#, ""),
        (r#"{"message": [1]}"#, ""),
    ] {
        let (value, told) = read_as::<Texted>(text, reading, Profile::EVERY_FAILURE);
        assert_eq!(
            value.and_then(|v| v.message).as_deref(),
            Some(read),
            "{text}"
        );
        assert_eq!(told, ["Info root/message core/invalid_type"], "{text}");
    }
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "silent")]
struct Quiet {
    name: Option<String>,
    extra: crate::document::UnknownMembers,
}

/// A struct that says `silent` keeps its unknown members and tells none.
#[test]
fn a_silent_struct_keeps_its_unknown_members_and_tells_nothing() {
    let (value, told) = read_as::<Quiet>(
        r#"{"name": "n", "k": [1]}"#,
        lenient(),
        Profile::EVERY_FAILURE,
    );
    let value = value.expect("read");
    assert_eq!(value.name.as_deref(), Some("n"));
    assert_eq!(
        value.extra.iter().map(|(key, _)| key).collect::<Vec<_>>(),
        ["k"]
    );
    assert!(told.is_empty(), "{told:?}");
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Nulled {
    #[facet(schema::reads_null)]
    text: Option<String>,
    #[facet(schema::reads_null)]
    any: Option<Value>,
    plain: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct NulledKept {
    #[facet(schema::reads_null, schema::or_kept = "silent")]
    size: Option<u8>,
    #[facet(schema::or_kept = "silent")]
    shade: Option<u8>,
    extra: crate::document::UnknownMembers,
}

/// A `reads_null` field that keeps what it cannot read keeps a `null` too, as written.
#[test]
fn a_reads_null_field_that_keeps_what_it_cannot_read_keeps_a_null() {
    let (value, told) = read_as::<NulledKept>(
        r#"{"size": null, "shade": null}"#,
        lenient(),
        Profile::EVERY_FAILURE,
    );
    let value = value.expect("read");
    assert_eq!((value.size, value.shade), (None, None));
    let kept: Vec<&str> = value.extra.iter().map(|(key, _)| key).collect();
    assert_eq!(kept, ["size"]);
    assert!(told.is_empty(), "{told:?}");
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "silent")]
struct Skinned {
    #[facet(schema::or_kept = "silent")]
    depth: Option<u8>,
    #[facet(schema::or_kept = "silent")]
    size: Option<u8>,
    extra: crate::document::UnknownMembers,
}

/// A read that tells the unknown members after the members keeps the members it could not read
/// first, in the order of the fields, as a walk that took each member in turn did.
#[test]
fn a_member_kept_as_written_comes_before_the_unknown_members_that_are_told_last() {
    let profile = Profile::DECLARED_ORDER.with_unknown(crate::json::kit::UnknownAt::Last);
    let (value, told) = read_as::<Skinned>(
        r#"{"zz": 1, "size": "big", "depth": "deep", "yy": 2}"#,
        lenient(),
        profile,
    );
    let value = value.expect("read");
    let kept: Vec<&str> = value.extra.iter().map(|(key, _)| key).collect();
    assert_eq!(kept, ["depth", "size", "zz", "yy"]);
    assert!(told.is_empty(), "{told:?}");
}

/// A `reads_null` field takes a `null` as a value of its type: `Some`, read by the type's own
/// read (here jsoncpp's `asString`, which tells it), where any other `Option` field reads it by
/// the null rule as `None`.
#[test]
fn a_reads_null_field_reads_a_null_as_a_value() {
    let reading = lenient().with_strings(crate::versioning::Strings::AsString);
    let (value, told) = read_as::<Nulled>(
        r#"{"text": null, "any": null, "plain": null}"#,
        reading,
        Profile::EVERY_FAILURE,
    );
    assert_eq!(
        value,
        Some(Nulled {
            text: Some(String::new()),
            any: Some(Value::NULL),
            plain: None,
        })
    );
    assert_eq!(told, ["Info root/text core/invalid_type"]);
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(untagged)]
#[repr(u8)]
enum Message {
    Text(String),
    Conditional {
        message: String,
        condition: Option<String>,
    },
    #[facet(schema::otherwise)]
    Other(Value),
}

/// An untagged enum's `otherwise` arm takes a value the chosen arm fails on, whose findings
/// stand, and a value no arm takes, without a word.
#[test]
fn an_untagged_enum_falls_back_to_its_otherwise_arm_keeping_what_was_told() {
    let read = |text: &str| read_as::<Message>(text, lenient(), Profile::EVERY_FAILURE);
    let (value, told) = read(r#"{"condition": "c"}"#);
    assert!(matches!(value, Some(Message::Other(_))), "{value:?}");
    assert_eq!(told, ["Info root/message core/missing_required_field"]);
    let (value, told) = read("[1]");
    assert!(matches!(value, Some(Message::Other(_))), "{value:?}");
    assert!(told.is_empty(), "{told:?}");
    let (value, _) = read(r#"{"message": "m"}"#);
    assert_eq!(
        value,
        Some(Message::Conditional {
            message: "m".to_owned(),
            condition: None
        })
    );
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Kept {
    #[facet(schema::or_kept = "told")]
    size: Option<u8>,
    #[facet(schema::or_kept = "silent")]
    shade: Option<u8>,
    extra: crate::document::UnknownMembers,
}

/// A field that keeps what it cannot read is `None`, the value kept as written beside the
/// typed ones, its failure told or not, and fails nothing.
#[test]
fn a_field_that_keeps_what_it_cannot_read_fails_nothing() {
    let (value, told) = read_as::<Kept>(
        r#"{"size": "big", "shade": "dark", "zz": 1}"#,
        Reading::DOCUMENT,
        Profile::EVERY_FAILURE,
    );
    let value = value.expect("read");
    assert_eq!((value.size, value.shade), (None, None));
    let kept: Vec<&str> = value.extra.iter().map(|(key, _)| key).collect();
    assert_eq!(kept, ["zz", "size", "shade"]);
    assert_eq!(
        told,
        [
            "Error root/size core/invalid_type",
            "Error root/zz core/unrecognized_field"
        ]
    );
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Versioned {
    #[facet(schema::since_format = "1.21.0")]
    late: Option<u8>,
    #[facet(schema::since_engine = "1.21.0")]
    new: Option<u8>,
}

/// A member read from a later format than the document's is unknown there; one read from a
/// later engine than the pack's is read and told.
#[test]
fn a_member_read_from_a_version_on_is_judged_by_the_document_s_versions() {
    use crate::json::kit::Versions;
    use crate::version::FormatVersion;
    let text = r#"{"late": 1, "new": 2}"#;
    let json: Value = from_str_jsonc(text).unwrap();
    let read = |format: (u16, u16), engine: (u16, u16)| {
        let mut found = Diagnostics::default();
        let mut read = Read::new(Reading::DOCUMENT, &mut found).versions(Versions {
            format: Some(FormatVersion::new(1, format.0, format.1)),
            engine: Some(FormatVersion::new(1, engine.0, engine.1)),
        });
        let value = Versioned::from_json(&Node::new(&json, JsonPath::new()), &mut read);
        let told: Vec<String> = found
            .iter()
            .map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code.id()))
            .collect();
        (value, told)
    };
    let (value, told) = read((20, 0), (21, 0));
    assert_eq!(
        value,
        Some(Versioned {
            late: None,
            new: Some(2)
        })
    );
    assert_eq!(told, ["Error late core/unrecognized_field"]);
    let (value, told) = read((21, 0), (20, 40));
    assert_eq!(
        value,
        Some(Versioned {
            late: Some(1),
            new: Some(2)
        })
    );
    assert_eq!(told, ["Info new core/field_since_engine"]);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Facet, Schema)]
#[repr(u8)]
enum Burn {
    Slow,
    Fast,
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Wick {
    size: u8,
    burn: Option<Burn>,
}

#[test]
fn the_texts_name_the_codes_of_a_missing_member_an_unknown_one_and_a_bad_enum_name() {
    use crate::diagnostic::Severity;
    use crate::diagnostic::codes;
    let json: Value = from_str_jsonc(r#"{"burn": "never", "zz": 1}"#).unwrap();
    let node = Node::new(&json, JsonPath::of_keys(["root"]));
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::OWN, &mut found)
        .unknown_severity(Severity::Error)
        .texts(Texts {
            missing_named: Some(&codes::REQUIRED_CHILD),
            unknown_named: Some(&codes::CHILD_NOT_VALID),
            invalid_enum: Some(&codes::INVALID_VALUE),
            ..Texts::CODES
        });
    assert!(Wick::from_json(&node, &mut read).is_none());
    read.finish();
    let told: Vec<String> = found
        .iter()
        .map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code))
        .collect();
    assert_eq!(
        told,
        [
            "Error root/burn invalid value",
            "Error root/zz child 'zz' not valid here.",
            "Error root/size Required child size not found",
        ]
    );
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Trio {
    a: u8,
    b: u8,
    n: Option<u8>,
}

#[test]
fn a_read_of_the_first_failure_only_goes_on_to_tell_the_nulls_and_the_unknown_members() {
    let text = r#"{"a": "x", "b": "y", "zz": 1, "n": null}"#;
    let (value, told) = read_as::<Trio>(text, Reading::NULL_ABSENT, Profile::FIRST_THEN_PASSES);
    assert_eq!(value, None);
    assert_eq!(
        told,
        [
            "Error root/a core/invalid_type",
            "Info root/n core/value_rewritten",
            "Warning root/zz core/unrecognized_field",
        ],
        "the second failure is not told, the passes are"
    );
    // The first failure's read stops there and tells it alone.
    let (_, told) = read_as::<Trio>(text, Reading::NULL_ABSENT, Profile::FIRST_FAILURE);
    assert_eq!(told, ["Error root/a core/invalid_type"]);
    // Every failure tells both.
    let (_, told) = read_as::<Trio>(text, Reading::NULL_ABSENT, Profile::EVERY_FAILURE);
    assert_eq!(told.len(), 4, "{told:?}");
}

#[test]
fn a_read_of_the_first_failure_only_tells_a_refused_null_only_of_a_read_that_stands() {
    // The door's null pass is of a document that read: after a failure the unknown members are
    // told and the `null` is not.
    let text = r#"{"a": "x", "b": 1, "zz": 1, "n": null}"#;
    let (value, told) = read_as::<Trio>(text, Reading::DOCUMENT, Profile::FIRST_THEN_PASSES);
    assert_eq!(value, None);
    assert_eq!(
        told,
        [
            "Error root/a core/invalid_type",
            "Error root/zz core/unrecognized_field"
        ]
    );
    // A read that stands tells the refused `null` alone.
    let text = r#"{"a": 1, "b": 1, "zz": 1, "n": null}"#;
    let (value, told) = read_as::<Trio>(text, Reading::DOCUMENT, Profile::FIRST_THEN_PASSES);
    assert_eq!(value, None);
    assert_eq!(told, ["Error root/n core/invalid_type"]);
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Gauge {
    a: f32,
    b: u8,
}

#[test]
fn a_read_of_the_first_failure_only_tells_a_clamped_number_of_a_read_that_stands() {
    // A number beyond its float is clamped and told (a pass of its own); a failure elsewhere
    // takes the note away.
    let reading = Reading::NULL_ABSENT;
    let (value, told) = read_as::<Gauge>(
        r#"{"a": 1e39, "b": 1}"#,
        reading,
        Profile::FIRST_THEN_PASSES,
    );
    assert_eq!(told, ["Info root/a core/beyond_float_member"]);
    assert!(value.is_some());
    let (value, told) = read_as::<Gauge>(
        r#"{"a": 1e39, "b": "y"}"#,
        reading,
        Profile::FIRST_THEN_PASSES,
    );
    assert_eq!(value, None);
    assert_eq!(told, ["Error root/b core/invalid_type"]);
}

/// [`read_as`] through the root read ([`read_document`](crate::schema::read_document)), with
/// `texts`.
fn read_root<T: FromJson>(
    text: &str,
    reading: Reading,
    profile: Profile,
    texts: Texts,
) -> (Option<T>, Vec<String>) {
    let json: Value = from_str_jsonc(text).expect("JSON");
    let node = Node::with_text(&json, JsonPath::of_keys(["root"]), JsonSlice::new(text));
    let mut found = Diagnostics::default();
    let mut read = Read::new(reading, &mut found).profile(profile).texts(texts);
    let value = crate::schema::read_document::<T>(&node, &mut read);
    let told = found
        .iter()
        .map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code.id()))
        .collect();
    (value, told)
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Taper {
    #[facet(default = 1)]
    length: u8,
    lit: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Torch {
    wick: Option<Taper>,
    #[facet(default)]
    wicks: Vec<Taper>,
    sizes: Option<IndexMap<String, u8>>,
    #[facet(default = 2)]
    count: u8,
}

const DEFAULTED: Texts = Texts {
    defaulted: Some(&crate::diagnostic::codes::INVALID_VALUE),
    ..Texts::CODES
};

const LENIENT: Reading = Reading::NULL_ABSENT.with_on_member_failure(MemberFailure::DefaultAndTell);

#[test]
fn the_value_recovery_gives_up_the_innermost_value_that_fails() {
    let text = r#"{"wick": {"length": "x", "lit": true}, "wicks": [{"length": 3}, 4, {"lit": 1}], "sizes": {"a": 1, "b": "big"}, "count": 3}"#;
    let profile = Profile::EVERY_FAILURE.with_recovery(Recovery::Value);
    let (torch, told) = read_root::<Torch>(text, LENIENT, profile, DEFAULTED);
    let torch = torch.expect("read");
    assert_eq!(
        torch.wick,
        Some(Taper {
            length: 1,
            lit: Some(true)
        })
    );
    assert_eq!(
        torch.wicks,
        [
            Taper {
                length: 3,
                lit: None
            },
            Taper {
                length: 1,
                lit: None
            }
        ]
    );
    assert_eq!(torch.sizes.map(|sizes| sizes.len()), Some(1));
    assert_eq!(torch.count, 3);
    assert_eq!(
        told,
        [
            "Info root/wick/length core/invalid_value",
            "Info root/wicks/1 core/invalid_value",
            "Info root/wicks/2/lit core/invalid_value",
            "Info root/sizes/b core/invalid_value",
        ]
    );
}

#[test]
fn the_top_level_recovery_gives_up_a_member_or_element_whole_and_tells_it_dropped() {
    let text = r#"{"wick": {"length": "x", "lit": 1}, "wicks": [{"length": 3}, {"lit": 1, "length": "y"}], "sizes": {"b": "big"}, "count": "z"}"#;
    let profile = Profile::EVERY_FAILURE.with_recovery(Recovery::TopLevel);
    let (torch, told) = read_root::<Torch>(text, LENIENT, profile, DEFAULTED);
    let torch = torch.expect("read");
    assert_eq!(torch.wick, None);
    assert_eq!(
        torch.wicks,
        [Taper {
            length: 3,
            lit: None
        }]
    );
    assert_eq!(torch.sizes, None);
    assert_eq!(torch.count, 2);
    assert_eq!(
        told,
        [
            // The first failure of each, the second one of `wick` not told.
            "Info root/wick/length core/invalid_value",
            "Info root/wick core/dropped_field",
            "Info root/wicks/1/lit core/invalid_value",
            "Info root/wicks/1 core/dropped_field",
            "Info root/sizes/b core/invalid_value",
            "Info root/sizes core/dropped_field",
            "Info root/count core/invalid_value",
        ]
    );
    // Without the texts each failure keeps its own code.
    let (_, told) = read_root::<Torch>(text, LENIENT, profile, Texts::CODES);
    assert_eq!(told[0], "Info root/wick/length core/invalid_type");
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Needs {
    name: String,
}
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Holds {
    #[facet(default)]
    needs: Vec<Needs>,
}

#[test]
fn a_defaulted_missing_member_is_told_at_the_object_that_lacks_it() {
    let profile = Profile::EVERY_FAILURE.with_recovery(Recovery::Value);
    let (holds, told) = read_root::<Holds>(
        r#"{"needs": [{}, {"name": "a"}]}"#,
        LENIENT,
        profile,
        DEFAULTED,
    );
    assert_eq!(holds.map(|holds| holds.needs.len()), Some(1));
    assert_eq!(told, ["Info root/needs/0 core/invalid_value"]);
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Spread {
    width: Option<crate::shared::FloatRange>,
    depth: Option<crate::shared::IntRange>,
}

#[test]
fn a_read_that_asks_tells_the_members_a_range_does_not_read() {
    let (spread, told) = read_root::<Spread>(
        r#"{"width": {"min": 1, "a": 2}, "depth": [1, 2, 3]}"#,
        Reading::NULL_ABSENT.with_range_members(RangeMembers::Unrecognized),
        Profile::FIRST_FAILURE,
        Texts::CODES,
    );
    assert!(spread.is_some());
    assert_eq!(
        told,
        [
            "Warning root/width/a core/unrecognized_field",
            "Warning root/depth/2 core/unrecognized_field",
        ]
    );
    // A read that does not ask tells none.
    let (_, told) = read_as::<Spread>(
        r#"{"width": {"min": 1, "a": 2}}"#,
        Reading::NULL_ABSENT,
        Profile::FIRST_FAILURE,
    );
    assert!(told.is_empty(), "{told:?}");
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
struct Chance {
    percent: crate::shared::IntRange,
}

#[test]
fn a_range_tells_the_members_it_does_not_read_when_the_reading_says_so() {
    let text = r#"{"percent": {"min": 1, "max": 5, "zz": 3, "yy": 4}, "other": 1}"#;
    let told = |reading: Reading| {
        let (value, told) = read_as::<Chance>(text, reading, Profile::FIRST_FAILURE);
        assert!(value.is_some());
        told
    };
    assert_eq!(
        told(Reading::NULL_ABSENT),
        ["Warning root/other core/unrecognized_field"],
        "silent by default"
    );
    // After the unknown members of the pass before, in file order; at the unknown severity.
    assert_eq!(
        told(Reading::NULL_ABSENT.with_range_members(RangeMembers::NotKept)),
        [
            "Warning root/other core/unrecognized_field",
            "Warning root/percent/zz core/member_not_kept",
            "Warning root/percent/yy core/member_not_kept"
        ]
    );
    assert_eq!(
        told(Reading::DOCUMENT.with_range_members(RangeMembers::NotKept))[1],
        "Error root/percent/zz core/member_not_kept"
    );
    // A range of the array form has no member to tell, and the ends of the other spelling are
    // no unread member.
    let (_, told) = read_as::<Chance>(
        r#"{"percent": {"range_min": 1, "range_max": 2, "min": 9}}"#,
        Reading::DOCUMENT.with_range_members(RangeMembers::NotKept),
        Profile::FIRST_FAILURE,
    );
    assert_eq!(told, ["Error root/percent/min core/member_not_kept"]);
    // The elements of the list form past its two ends are told the same way.
    let (_, told) = read_as::<Spread>(
        r#"{"depth": [1, 2, 3]}"#,
        Reading::NULL_ABSENT.with_range_members(RangeMembers::NotKept),
        Profile::FIRST_FAILURE,
    );
    assert_eq!(told, ["Warning root/depth/2 core/member_not_kept"]);
}

#[test]
fn a_strict_bool_takes_a_bool_and_no_other_value() {
    use crate::json::StrictBool;
    let (value, told) = door::<StrictBool>("true");
    assert_eq!((value, told.len()), (Some(StrictBool(true)), 0));
    for text in ["1", r#""true""#, "null"] {
        let (value, told) = door::<StrictBool>(text);
        assert_eq!(value, None, "{text}");
        assert_eq!(told, ["Error root core/invalid_type"], "{text}");
    }
}

#[test]
fn a_value_in_memory_reads_as_the_plain_typed_read_does() {
    use crate::json::{MAX_JSON_DEPTH, Value};
    let json = |text: &str| -> Value { from_str_jsonc(text).expect("JSON") };
    let lamp = |text: &str| schema::read_value::<Lamp>(&json(text));
    assert_eq!(
        lamp(r#"{"level": 3, "zz": 1}"#).map(|lamp| lamp.level),
        Some(3)
    );
    // Strict scalars, and a required member.
    assert_eq!(lamp(r#"{"level": 3.5}"#), None);
    assert_eq!(lamp(r#"{"level": "3"}"#), None);
    assert_eq!(lamp(r#"{"lit": true}"#), None);
    // A number beyond its float is no read.
    assert_eq!(schema::read_value::<f32>(&json("1e39")), None);
    // Nested as deep as a document may be, and no deeper.
    let nested = |levels: usize| {
        let mut deep = Value::from(0_i64);
        for _ in 0..levels {
            deep = Value::from(facet_value::VArray::from_iter([deep]));
        }
        deep
    };
    assert!(schema::read_value::<Value>(&nested(MAX_JSON_DEPTH)).is_some());
    assert!(schema::read_value::<Value>(&nested(MAX_JSON_DEPTH + 1)).is_none());
}

/// A Molang number beyond `f32` is a wrong type, unless the read clamps it: then the largest
/// finite float of its sign and an information at the number, after the read.
#[test]
fn a_molang_number_beyond_f32_is_clamped_where_the_read_says() {
    use crate::molang::Molang;
    let text = "[1e39, -1e999, 0.5]";
    let (value, told) = door::<Vec<Molang>>(text);
    assert_eq!(value, None);
    assert_eq!(told, ["Error root/0 core/invalid_type"]);
    let json: Value = from_str_jsonc(text).expect("JSON");
    let node = Node::with_text(&json, JsonPath::of_keys(["root"]), JsonSlice::new(text));
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT, &mut found)
        .profile(Profile::FIRST_FAILURE)
        .clamp_molang();
    let value = Vec::<Molang>::from_json(&node, &mut read);
    read.finish();
    assert_eq!(
        value,
        Some(vec![
            Molang::Const(f32::MAX),
            Molang::Const(f32::MIN),
            Molang::Const(0.5)
        ])
    );
    let told: Vec<String> = found
        .iter()
        .map(|f| {
            format!(
                "{:?} {} {:?}",
                f.severity, f.location.json_path, f.location.span
            )
        })
        .collect();
    assert_eq!(
        told,
        [
            "Info root/0 Some(Span { offset: 1, len: 4 })",
            "Info root/1 Some(Span { offset: 7, len: 6 })"
        ]
    );
}

/// A kept `Value` holds the infinity a number beyond a double is, unless the read clamps it: then
/// the largest finite double of its sign and an information at the number.
#[test]
fn a_kept_number_beyond_a_double_is_clamped_where_the_read_says() {
    let text = r#"[1.5, {"a": -1e999}, 1e999]"#;
    let json: Value = from_str_jsonc(text).expect("JSON");
    let node = Node::with_text(&json, JsonPath::of_keys(["root"]), JsonSlice::new(text));
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT, &mut found)
        .profile(Profile::FIRST_FAILURE)
        .clamp_kept();
    let value = Value::from_json(&node, &mut read).expect("a value");
    read.finish();
    let again = value.as_array().expect("a list");
    assert_eq!(
        again[2].as_number().and_then(facet_value::VNumber::to_f64),
        Some(f64::MAX)
    );
    let told: Vec<String> = found
        .iter()
        .map(|f| {
            format!(
                "{:?} {} {:?}",
                f.severity, f.location.json_path, f.location.span
            )
        })
        .collect();
    assert_eq!(
        told,
        [
            "Info root/1/a Some(Span { offset: 12, len: 6 })",
            "Info root/2 Some(Span { offset: 21, len: 5 })"
        ]
    );
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT, &mut found).profile(Profile::FIRST_FAILURE);
    let kept = Value::from_json(&node, &mut read).expect("a value");
    read.finish();
    assert!(found.is_empty());
    assert_ne!(
        kept.as_array().expect("a list")[2]
            .as_number()
            .and_then(facet_value::VNumber::to_f64),
        Some(f64::MAX)
    );
}
