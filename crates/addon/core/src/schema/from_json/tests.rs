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
    let parsed = crate::json::parse::parse_slice(
        JsonSlice::new(text),
        crate::json::parse::Options::default(),
    )
    .unwrap();
    let node = Node::parsed(&parsed, JsonPath::of_keys(["root"]));
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
    // The first-failure profile tells only its failure.
    assert_eq!(door::<Lamp>(text).1, ["Error root/level core/invalid_type"]);
    assert_eq!(
        as_read::<Lamp>(text).1,
        [
            "Error root/zz core/unrecognized_field",
            "Error root/level core/invalid_type"
        ]
    );
    let text = r#"{"level": 1, "zz": 1, "lit": "true"}"#;
    // Converted, then told after the unknown member, as the first-failure profile's passes run.
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
    #[facet(skip)]
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
    let parsed = crate::json::parse::parse_slice(
        JsonSlice::new(text),
        crate::json::parse::Options::default(),
    )
    .unwrap();
    let node = Node::parsed(&parsed, JsonPath::new());
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
    let profiles = [Scalars::Strict, Scalars::NonStrict, Scalars::ConvertAndTell];
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
    assert_eq!(read("3"), [Some(3); 3]);
    assert_eq!(read("2.7"), [None, Some(2), None]);
    assert_eq!(read("3.0"), [None, Some(3), Some(3)]);
    assert_eq!(read("257"), [None, Some(1), None]);
    assert_eq!(read("true"), [None, Some(1), None]);
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
fn a_list_fails_with_an_element() {
    let (list, told) = as_read::<Vec<u8>>(r#"[1, "x", 3, true]"#);
    assert_eq!(list, None);
    assert_eq!(
        told,
        [
            "Error root/1 core/invalid_type",
            "Error root/3 core/invalid_type"
        ]
    );
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
    // As facet-json told it (schema_read_door's every_kind_of_member grid): a wrong type at
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

/// The name of a stage's variant: the text at `kind` without everything up to its last `:`;
/// a name that is none is told at the stage, as the game words it.
fn stage_kind(node: &Node<'_>, read: &mut Read<'_>) -> Option<&'static str> {
    let written = node
        .value()
        .as_object()
        .and_then(|object| object.get("kind"))
        .and_then(|kind| kind.as_string().map(|text| text.as_str().to_owned()))
        .unwrap_or_default();
    let name = written.rsplit(':').next().unwrap_or_default();
    let found = <StageKind as super::derived::Fields>::VARIANTS
        .iter()
        .find(|variant| **variant == name);
    if found.is_none() {
        read.push(crate::diagnostic::Diagnostic::of(&STAGE_KIND).at(node.path().clone()));
    }
    found.copied()
}

/// The code of a stage of no known kind.
static STAGE_KIND: crate::diagnostic::CodeDef<0> = crate::diagnostic::CodeDef::new(
    "test/stage_kind",
    crate::diagnostic::Kind::InvalidValue,
    crate::diagnostic::Class::KeptError,
    "a stage of an unknown kind",
);

/// A rule over the variant just read, with the object as written: a weighted stage that
/// writes no weight is told.
fn stage_rule(kind: &mut StageKind, node: &Node<'_>, read: &mut Read<'_>) {
    if matches!(kind, StageKind::Weighted { weight: None }) {
        read.push(crate::diagnostic::Diagnostic::of(&STAGE_WEIGHT).at(node.path().clone()));
    }
}

/// The code of a weighted stage without a weight.
static STAGE_WEIGHT: crate::diagnostic::CodeDef<0> = crate::diagnostic::CodeDef::new(
    "test/stage_weight",
    crate::diagnostic::Kind::MissingRequired,
    crate::diagnostic::Class::Warning,
    "a weighted stage without a weight",
);

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(tag = "kind", schema::tag_with = stage_kind, schema::after_read = stage_rule)]
#[repr(u8)]
enum StageKind {
    #[facet(rename = "plain")]
    Plain,
    #[facet(rename = "weighted")]
    Weighted { weight: Option<u8> },
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Stage {
    before: Option<u8>,
    #[facet(flatten)]
    kind: StageKind,
    after: Option<u8>,
    extra: crate::document::UnknownMembers,
}

/// An internally tagged enum flattened into a struct is read where the struct declares it, so
/// the findings of the members before and after it come in declaration order; the name of the
/// variant is the function's; a member only another variant has is one the struct does not know
/// (told and kept by the struct, whatever the variant).
#[test]
fn a_flattened_tagged_enum_is_read_where_the_struct_declares_it() {
    let declared = Profile::DECLARED_ORDER.with_unknown(crate::json::kit::UnknownAt::Last);
    let read = |text: &str| read_as::<Stage>(text, lenient(), declared);
    let (stage, told) = read(r#"{"after": 2, "weight": 5, "kind": "x:weighted", "before": 1}"#);
    assert!(told.is_empty(), "{told:?}");
    assert_eq!(
        stage.map(|stage| (stage.before, stage.kind, stage.after)),
        Some((Some(1), StageKind::Weighted { weight: Some(5) }, Some(2)))
    );
    // `weight` is no member of a plain stage: unknown, kept in the struct's own.
    let (stage, told) = read(r#"{"kind": "plain", "weight": 5}"#);
    assert_eq!(told, ["Warning root/weight core/unrecognized_field"]);
    let extra = stage.expect("read").extra;
    assert_eq!(
        extra.iter().map(|(key, _)| key).collect::<Vec<_>>(),
        ["weight"]
    );
    // The members before it are told before its own and the rule's, those after it after.
    let (_, told) = read(r#"{"after": "a", "weight": "w", "kind": "weighted", "before": "b"}"#);
    assert_eq!(
        told,
        [
            "Info root/before core/invalid_type",
            "Info root/weight core/invalid_type",
            "Warning root test/stage_weight",
            "Info root/after core/invalid_type",
        ]
    );
}

/// A name the function refuses is told by it, in the tag's place, and fails the struct that
/// flattens the enum there: its members after the tag and its unknown ones are not told. With
/// no tag member the function is asked all the same.
#[test]
fn a_flattened_tagged_enum_the_function_refuses_fails_its_owner() {
    let declared = Profile::DECLARED_ORDER.with_unknown(crate::json::kit::UnknownAt::Last);
    let (stage, told) = read_as::<Stage>(
        r#"{"before": "b", "kind": "other", "after": "a", "k": 1}"#,
        lenient(),
        declared,
    );
    assert_eq!(stage, None);
    assert_eq!(
        told,
        [
            "Info root/before core/invalid_type",
            "Error root test/stage_kind"
        ]
    );
    let (stage, told) = read_as::<Stage>(r#"{"before": 1}"#, lenient(), declared);
    assert_eq!(stage, None);
    assert_eq!(told, ["Error root test/stage_kind"]);
}

/// A rule on the enum runs over the variant read, flattened or not, in the place the variant is
/// read: before the owner's members after it.
#[test]
fn a_tagged_enum_s_rule_runs_over_the_variant_it_read() {
    let declared = Profile::DECLARED_ORDER.with_unknown(crate::json::kit::UnknownAt::Last);
    let (_, told) = read_as::<Stage>(r#"{"kind": "weighted", "after": "a"}"#, lenient(), declared);
    assert_eq!(
        told,
        [
            "Warning root test/stage_weight",
            "Info root/after core/invalid_type"
        ]
    );
    let (kind, told) = read_as::<StageKind>(r#"{"kind": "weighted"}"#, lenient(), declared);
    assert_eq!(kind, Some(StageKind::Weighted { weight: None }));
    assert_eq!(told, ["Warning root test/stage_weight"]);
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
    let parsed = crate::json::parse::parse_slice(
        JsonSlice::new(text),
        crate::json::parse::Options::default(),
    )
    .unwrap();
    let node = Node::parsed(&parsed, JsonPath::new());
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
    // The first-failure profile tells it after the read, as any unknown member.
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
/// the old typed read did.
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
/// nothing, as the old read absorbed its kept members only after a typed read that worked.
#[test]
fn a_read_that_fails_keeps_no_unknown_member() {
    for profile in [Profile::FIRST_FAILURE, Profile::EVERY_FAILURE] {
        for (text, read) in [
            (r#"{"zz": 1, "a": "x"}"#, false),
            (r#"{"zz": 1, "a": 2}"#, true),
        ] {
            let parsed = crate::json::parse::parse_slice(
                JsonSlice::new(text),
                crate::json::parse::Options::default(),
            )
            .unwrap();
            let node = Node::parsed(&parsed, JsonPath::new());
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
    // The first-failure profile's null pass, as for any `Option` member.
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

/// A struct inside a fixed array tells its unknown members and keeps none: the old walk
/// could not reach into a fixed array to keep them.
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
    let mut read = Read::new(Reading::DOCUMENT, &mut found);
    let list = super::read_list(
        &Node::new(&json, JsonPath::new()),
        &mut read,
        super::ElementFailure::Drop,
        u8::from_json,
    );
    assert_eq!(list, Some(vec![1, 3]));
    assert!(!read.failed());
}

/// A committed attempt brings the moves of the lists it shortened: a finding told later at an
/// element it renumbered is named as the file has it. A fork of a read that has stopped reads
/// nothing.
#[test]
fn a_committed_fork_keeps_its_moves_and_a_stopped_read_forks_stopped() {
    let json: Value = from_str_jsonc(r#"[1, "x", 3]"#).unwrap();
    let node = Node::new(&json, JsonPath::of_keys(["list"]));
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT, &mut found);
    let mut attempt = read.fork();
    let list = super::read_list(
        &node,
        &mut attempt,
        super::ElementFailure::Drop,
        u8::from_json,
    );
    assert_eq!(list, Some(vec![1, 3]));
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
    let parsed = crate::json::parse::parse_slice(
        JsonSlice::new(text),
        crate::json::parse::Options::default(),
    )
    .unwrap();
    let node = Node::parsed(&parsed, JsonPath::new());
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

static WRONG_OF: crate::diagnostic::CodeDef<1> = crate::diagnostic::CodeDef::new(
    "test/wrong_of",
    crate::diagnostic::Kind::WrongType,
    crate::diagnostic::Class::Error,
    "a wrong type in {}",
);
static MISSING_OF: crate::diagnostic::CodeDef<1> = crate::diagnostic::CodeDef::new(
    "test/missing_of",
    crate::diagnostic::Kind::MissingRequired,
    crate::diagnostic::Class::Error,
    "a missing member in {}",
);
static INVALID_OF: crate::diagnostic::CodeDef<1> = crate::diagnostic::CodeDef::new(
    "test/invalid_of",
    crate::diagnostic::Kind::InvalidValue,
    crate::diagnostic::Class::Error,
    "an invalid value in {}",
);

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Subjected {
    #[facet(schema::wrong_type = WRONG_OF)]
    count: Option<u8>,
    #[facet(schema::missing = MISSING_OF)]
    name: String,
    #[facet(schema::invalid = INVALID_OF)]
    bound: Option<Bound>,
    #[facet(schema::wrong_type = crate::diagnostic::codes::DROPPED_FIELD)]
    plain: Option<u8>,
}

/// Reads `text` as `Subjected` for `subject`: each finding as `path code text`.
fn read_for(text: &str, subject: Option<&str>) -> Vec<String> {
    let json = from_str_jsonc(text).unwrap();
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT, &mut found);
    if let Some(subject) = subject {
        read = read.subject(subject);
    }
    let value = Subjected::from_json(&Node::new(&json, JsonPath::of_keys(["root"])), &mut read);
    assert!(value.is_none());
    read.finish();
    found
        .iter()
        .map(|f| format!("{} {} {}", f.location.json_path, f.code.id(), f.code))
        .collect()
}

/// A field's code of one argument is fed the read's subject; a read that names none tells its
/// own code there, and a code of no argument is told either way.
#[test]
fn a_member_code_of_one_argument_is_fed_the_read_s_subject() {
    let text = r#"{"count": "3", "bound": "neither", "plain": "4"}"#;
    assert_eq!(
        read_for(text, Some("minecraft:stone")),
        [
            "root/count test/wrong_of a wrong type in minecraft:stone",
            "root/bound test/invalid_of an invalid value in minecraft:stone",
            "root/plain core/dropped_field field dropped",
            "root/name test/missing_of a missing member in minecraft:stone",
        ]
    );
    let told = read_for(text, None);
    let codes: Vec<&str> = told
        .iter()
        .map(|line| line.split(' ').nth(1).unwrap())
        .collect();
    assert_eq!(
        codes,
        [
            "core/invalid_type",
            "core/invalid_enum_value",
            "core/dropped_field",
            "core/missing_required_field"
        ]
    );
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
    #[facet(alias = "colour")]
    color: Option<String>,
}

/// A list jsoncpp iterates reads whatever its member is: an array's elements, an object's
/// values in key order, nothing for another value, the member told as no array; an alias is a
/// second name.
#[test]
fn a_list_reads_as_its_keys_say() {
    let read = |text: &str| read_as::<Listed>(text, lenient(), Profile::EVERY_FAILURE);
    let (value, told) = read(r#"{"names": {"b": "y", "a": "x"}}"#);
    assert_eq!(
        value,
        Some(Listed {
            names: Some(vec!["x".to_owned(), "y".to_owned()]),
            color: None,
        })
    );
    assert_eq!(told, ["Info root/names core/invalid_type"]);
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
struct Part {
    weight: u8,
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Parts {
    #[facet(schema::drop)]
    dropped: Option<Vec<Part>>,
    plain: Option<Vec<Part>>,
}

/// A list that says `drop` leaves a failed element out alone, told by its own findings, whatever
/// the reading: the list does not fail, and the elements kept are renumbered for the findings
/// told after; a list that does not say it fails with its element.
#[test]
fn a_dropping_list_leaves_a_failed_element_out_alone() {
    let text = r#"{"dropped": [{"weight": 1}, 7, {"weight": "x"}, {"weight": 4}]}"#;
    let (value, told) = read_as::<Parts>(text, Reading::DOCUMENT, Profile::EVERY_FAILURE);
    let weights = value
        .expect("read")
        .dropped
        .map(|parts| parts.iter().map(|part| part.weight).collect::<Vec<_>>());
    assert_eq!(weights, Some(vec![1, 4]));
    assert_eq!(
        told,
        [
            "Error root/dropped/1 core/invalid_type",
            "Error root/dropped/2/weight core/invalid_type",
        ]
    );
    let (value, told) = read_as::<Parts>(
        r#"{"plain": [3]}"#,
        Reading::DOCUMENT,
        Profile::EVERY_FAILURE,
    );
    assert_eq!(value, None);
    assert_eq!(told, ["Error root/plain/0 core/invalid_type"]);
}

/// The elements a dropping list keeps are named as the file has them by the findings told after.
#[test]
fn a_dropping_list_names_the_elements_it_kept_as_the_file_has_them() {
    let json: Value = from_str_jsonc(r#"[{"weight": 1}, 7, {"weight": 3}]"#).unwrap();
    let node = Node::new(&json, JsonPath::of_keys(["list"]));
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT, &mut found);
    let list = crate::schema::derive_support::list::<Part>(&node, &mut read, false, true);
    assert_eq!(list.map(|parts| parts.len()), Some(2));
    read.findings_mut().push(
        crate::diagnostic::Diagnostic::of(&crate::diagnostic::codes::INVALID_VALUE)
            .at(JsonPath::of_keys(["list"]).index(1)),
    );
    drop(read);
    found.name_as_the_file_has_it();
    let at: Vec<String> = found
        .iter()
        .map(|f| f.location.json_path.to_string())
        .collect();
    assert_eq!(at, ["list/1", "list/2"]);
}

/// A list a game reader iterates without a type test reads as jsoncpp iterates it and tells
/// nothing of a member that is no array; the values an object gives are named by their keys for
/// the findings told after the read.
#[test]
fn a_list_iterated_silently_names_an_object_s_values_by_their_keys() {
    let element = |node: &Node<'_>, read: &mut Read<'_>| Part::from_json(node, read);
    for text in ["3", "null", "\"x\""] {
        let json: Value = from_str_jsonc(text).unwrap();
        let mut found = Diagnostics::default();
        let mut read = Read::new(Reading::DOCUMENT, &mut found);
        let node = Node::new(&json, JsonPath::of_keys(["list"]));
        let list = crate::schema::read_iterated_silently(
            &node,
            &mut read,
            crate::schema::ElementFailure::Drop,
            element,
        );
        assert_eq!(list, Some(Vec::new()), "{text}");
        drop(read);
        assert!(found.is_empty(), "{text}");
    }
    let json: Value =
        from_str_jsonc(r#"{"c": {"weight": 3}, "a": 7, "b": {"weight": 2}}"#).unwrap();
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT, &mut found);
    let node = Node::new(&json, JsonPath::of_keys(["list"]));
    let list = crate::schema::read_iterated_silently(
        &node,
        &mut read,
        crate::schema::ElementFailure::Drop,
        element,
    );
    let weights = list.map(|parts| parts.iter().map(|part| part.weight).collect::<Vec<_>>());
    assert_eq!(weights, Some(vec![2, 3]));
    for held in 0..2 {
        read.findings_mut().push(
            crate::diagnostic::Diagnostic::of(&crate::diagnostic::codes::INVALID_VALUE)
                .at(JsonPath::of_keys(["list"]).index(held).child("weight")),
        );
    }
    drop(read);
    found.name_as_the_file_has_it();
    let at: Vec<String> = found
        .iter()
        .map(|f| f.location.json_path.to_string())
        .collect();
    assert_eq!(at, ["list/a", "list/b/weight", "list/c/weight"]);
}

/// A list that stops at its first failed element fails there, the elements after it unread.
#[test]
fn a_stopping_list_is_given_up_at_its_first_failed_element() {
    let json: Value = from_str_jsonc(r#"[{"weight": 1}, 7, "x"]"#).unwrap();
    let mut found = Diagnostics::default();
    let mut read = Read::new(lenient(), &mut found);
    let node = Node::new(&json, JsonPath::of_keys(["list"]));
    let list = crate::schema::read_list(
        &node,
        &mut read,
        crate::schema::ElementFailure::Stop,
        Part::from_json,
    );
    assert_eq!(list, None);
    drop(read);
    let told: Vec<String> = found
        .iter()
        .map(|f| f.location.json_path.to_string())
        .collect();
    assert_eq!(told, ["list/1"]);
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Ended {
    #[facet(schema::ends)]
    first: String,
    second: Option<u8>,
    extra: crate::document::UnknownMembers,
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct KeptOrEnded {
    #[facet(schema::or_kept = "told", schema::ends)]
    list: Option<Vec<u8>>,
    after: Option<u8>,
    extra: crate::document::UnknownMembers,
}

/// A member that `ends` its struct's read fails the struct where it fails or is told missing:
/// nothing after it is read or told, the unknown members neither; with `or_kept` a value of
/// another JSON kind than the type takes is kept, and one of that kind that fails ends the read.
#[test]
fn a_member_that_ends_the_read_fails_its_struct_there() {
    let declared = Profile::DECLARED_ORDER.with_unknown(crate::json::kit::UnknownAt::Last);
    let read = |text: &str| read_as::<Ended>(text, lenient().strict(), declared);
    let (value, told) = read(r#"{"second": "x", "k": 1}"#);
    assert_eq!(value, None);
    assert_eq!(told, ["Info root/first core/missing_required_field"]);
    let (value, told) = read(r#"{"first": 3, "second": "x", "k": 1}"#);
    assert_eq!(value, None);
    assert_eq!(told, ["Info root/first core/invalid_type"]);
    let (value, told) = read(r#"{"first": "a", "second": "x", "k": 1}"#);
    assert_eq!(
        value.map(|v| (v.first, v.second)),
        Some(("a".to_owned(), None))
    );
    assert_eq!(
        told,
        [
            "Info root/second core/invalid_type",
            "Warning root/k core/unrecognized_field"
        ]
    );
    let read = |text: &str| read_as::<KeptOrEnded>(text, lenient().strict(), declared);
    let (value, told) = read(r#"{"list": 3, "after": 1}"#);
    let value = value.expect("read");
    assert_eq!((value.list, value.after), (None, Some(1)));
    assert_eq!(value.extra.get("list"), Some(&Value::from(3i64)));
    assert_eq!(told, ["Info root/list core/invalid_type"]);
    let (value, told) = read(r#"{"list": ["x"], "after": "y"}"#);
    assert_eq!(value, None);
    assert_eq!(told, ["Info root/list/0 core/invalid_type"]);
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::after_members = name_rule)]
struct Ruled {
    name: Option<String>,
    extra: crate::document::UnknownMembers,
}

fn name_rule(ruled: &mut Ruled, node: &Node<'_>, read: &mut Read<'_>) {
    if ruled.name.as_deref() == Some("bad") {
        read.push(
            crate::diagnostic::Diagnostic::of(&crate::diagnostic::codes::INVALID_VALUE)
                .severity(crate::diagnostic::Severity::Info)
                .at(node.path().child("name")),
        );
    }
}

/// A struct's rule after its members (`schema::after_members`) runs on the value they built,
/// before the unknown members a profile tells after them, which the value then keeps.
#[test]
fn a_rule_after_the_members_comes_before_the_unknown_members() {
    let declared = Profile::DECLARED_ORDER.with_unknown(crate::json::kit::UnknownAt::Last);
    let (value, told) = read_as::<Ruled>(r#"{"k": 1, "name": "bad"}"#, lenient(), declared);
    let value = value.expect("read");
    assert_eq!(value.extra.get("k"), Some(&Value::from(1i64)));
    assert_eq!(
        told,
        [
            "Info root/name core/invalid_value",
            "Warning root/k core/unrecognized_field"
        ]
    );
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
/// read (here the sound files' `asString`, which makes it the empty text), where any other
/// `Option` field reads it by the null rule as `None`.
#[test]
fn a_reads_null_field_reads_a_null_as_a_value() {
    let reading = lenient().with_scalars(Scalars::Fork);
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
    assert!(told.is_empty(), "{told:?}");
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
struct KeptRequired {
    #[facet(schema::or_kept = "told", schema::required)]
    size: Option<u8>,
    extra: crate::document::UnknownMembers,
}

/// A required member that keeps what it cannot read is there: its failure is the one finding,
/// the owner is read, and only a member the file lacks is told missing.
#[test]
fn a_required_field_that_keeps_what_it_cannot_read_is_not_missing() {
    let (value, told) = read_as::<KeptRequired>(
        r#"{"size": "big"}"#,
        Reading::DOCUMENT,
        Profile::EVERY_FAILURE,
    );
    let value = value.expect("read");
    assert_eq!(value.size, None);
    let kept: Vec<&str> = value.extra.iter().map(|(key, _)| key).collect();
    assert_eq!(kept, ["size"]);
    assert_eq!(told, ["Error root/size core/invalid_type"]);
    let (value, told) = read_as::<KeptRequired>("{}", Reading::DOCUMENT, Profile::EVERY_FAILURE);
    assert_eq!(value, None);
    assert_eq!(told, ["Error root/size core/missing_required_field"]);
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Versioned {
    #[facet(schema::since_format = "1.21.0")]
    late: Option<u8>,
}

/// A member read from a later format than the document's is unknown there.
#[test]
fn a_member_read_from_a_version_on_is_judged_by_the_document_s_format() {
    use crate::version::FormatVersion;
    let text = r#"{"late": 1}"#;
    let json: Value = from_str_jsonc(text).unwrap();
    let read = |format: (u16, u16)| {
        let mut found = Diagnostics::default();
        let mut read = Read::new(Reading::DOCUMENT, &mut found)
            .format(Some(FormatVersion::new(1, format.0, format.1)));
        let value = Versioned::from_json(&Node::new(&json, JsonPath::new()), &mut read);
        let told: Vec<String> = found
            .iter()
            .map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code.id()))
            .collect();
        (value, told)
    };
    let (value, told) = read((20, 0));
    assert_eq!(value, Some(Versioned { late: None }));
    assert_eq!(told, ["Error late core/unrecognized_field"]);
    let (value, told) = read((21, 0));
    assert_eq!(value, Some(Versioned { late: Some(1) }));
    assert!(told.is_empty(), "{told:?}");
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
    // The first-failure null pass is of a document that read: after a failure the
    // unknown members are told and the `null` is not.
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
    let parsed = crate::json::parse::parse_slice(
        JsonSlice::new(text),
        crate::json::parse::Options::default(),
    )
    .unwrap();
    let node = Node::parsed(&parsed, JsonPath::of_keys(["root"]));
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
    let parsed = crate::json::parse::parse_slice(
        JsonSlice::new(text),
        crate::json::parse::Options::default(),
    )
    .unwrap();
    let node = Node::parsed(&parsed, JsonPath::of_keys(["root"]));
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
    let parsed = crate::json::parse::parse_slice(
        JsonSlice::new(text),
        crate::json::parse::Options::default(),
    )
    .unwrap();
    let node = Node::parsed(&parsed, JsonPath::of_keys(["root"]));
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

/// A reader that returns `None` and tells nothing: the contract of `FromJson::from_json` broken.
fn silent_failure(_: &Node<'_>, _: &mut Read<'_>) -> Option<u8> {
    None
}

#[derive(Debug, PartialEq, Facet, Schema)]
struct SilentFailure {
    #[facet(schema::read_with = silent_failure)]
    level: Option<u8>,
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "told nothing")]
fn a_leaf_that_fails_without_telling_is_caught_under_the_tree() {
    drop(read_as::<SilentFailure>(
        r#"{"level": 1}"#,
        Reading::OWN.with_null(NullHandling::AsValue),
        Profile::TREE,
    ));
}

#[test]
fn a_leaf_that_fails_without_telling_is_no_one_s_business_off_the_tree() {
    let (quiet, told) =
        read_as::<SilentFailure>(r#"{"level": 1}"#, Reading::OWN, Profile::EVERY_FAILURE);
    assert_eq!(quiet, None);
    assert!(told.is_empty(), "{told:?}");
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Placed {
    first: u8,
    second: u8,
    third: Option<u8>,
}

/// A read that asks for the members one by one in declared order tells a missing required one
/// where it asks for it; a read in the file's order tells it after the members the file has.
#[test]
fn a_missing_required_member_is_told_in_its_place_in_declared_order() {
    let text = r#"{"third": "x"}"#;
    let (value, told) = read_as::<Placed>(text, Reading::DOCUMENT, Profile::DECLARED_ORDER);
    assert_eq!(value, None);
    assert_eq!(
        told,
        [
            "Error root/first core/missing_required_field",
            "Error root/second core/missing_required_field",
            "Error root/third core/invalid_type",
        ]
    );
    let (_, told) = read_as::<Placed>(text, Reading::DOCUMENT, Profile::EVERY_FAILURE);
    assert_eq!(
        told,
        [
            "Error root/third core/invalid_type",
            "Error root/first core/missing_required_field",
            "Error root/second core/missing_required_field",
        ]
    );
}

/// A struct's own `UnknownMembers` holds an unknown member as written, unless the read clamps
/// what it keeps: then the number no double holds is the largest finite one, told at its place
/// after the unknown member's own finding.
#[test]
fn an_unknown_member_kept_beyond_a_double_is_clamped_where_the_read_says() {
    #[derive(Debug, PartialEq, Facet, Schema)]
    struct Holder {
        known: Option<u8>,
        extra: crate::document::UnknownMembers,
    }
    let text = r#"{"known": 1, "more": [1e999]}"#;
    let parsed = crate::json::parse::parse_slice(
        JsonSlice::new(text),
        crate::json::parse::Options::default(),
    )
    .unwrap();
    let node = Node::parsed(&parsed, JsonPath::of_keys(["root"]));
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT, &mut found).clamp_kept();
    let holder = Holder::from_json(&node, &mut read).expect("a value");
    read.finish();
    let more = holder
        .extra
        .get("more")
        .and_then(Value::as_array)
        .expect("kept");
    assert_eq!(
        more[0].as_number().and_then(facet_value::VNumber::to_f64),
        Some(f64::MAX)
    );
    let told: Vec<String> = found
        .iter()
        .map(|f| format!("{} {}", f.location.json_path, f.code))
        .collect();
    assert_eq!(
        told,
        ["root/more unrecognized field", "root/more/0 invalid value"]
    );
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Resets {
    #[facet(default)]
    inner: Counted,
    #[facet(schema::unknown = "keep")]
    entries: Option<IndexMap<String, Counted>>,
}

/// A value that fails when its one member does, kept in a map or reset to its default.
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
struct Counted {
    n: u8,
}

/// The document `text` read as `T` under `lenient()` and `profile`: the value, and the paths and keys of
/// the unknown members the read kept.
fn kept_after<T: FromJson>(text: &str, profile: Profile) -> (Option<T>, Vec<String>, Diagnostics) {
    let parsed = crate::json::parse::parse_slice(
        JsonSlice::new(text),
        crate::json::parse::Options::default(),
    )
    .unwrap();
    let node = Node::parsed(&parsed, JsonPath::new());
    let mut found = Diagnostics::default();
    let mut read = Read::new(lenient(), &mut found).profile(profile);
    let value = super::read_document::<T>(&node, &mut read);
    let kept = found
        .unknown()
        .iter()
        .map(|(at, key, _)| format!("{at}/{key}"))
        .collect();
    (value, kept, found)
}

/// A member reset to its default under `DefaultAndTell` keeps none of the unknown members its
/// failed value held, and the default object written back carries none.
#[test]
fn a_member_reset_to_its_default_keeps_no_unknown_member_of_the_failed_value() {
    let (value, kept, found) = kept_after::<Resets>(
        r#"{"inner": {"extra": 1, "n": "x"}}"#,
        Profile::EVERY_FAILURE,
    );
    let value = value.expect("read");
    assert_eq!(value.inner, Counted::default());
    assert_eq!(kept, Vec::<String>::new());
    let mut out = Vec::new();
    let style = crate::json::WriteStyle {
        indent: crate::json::Indent::Compact,
        trailing_newline: false,
    };
    crate::json::write_json_with(&value, found.unknown(), &mut out, style).unwrap();
    assert_eq!(String::from_utf8(out).unwrap(), r#"{"inner":{"n":0}}"#);
}

/// A map entry left out keeps none of the unknown members its failed value held.
#[test]
fn a_map_entry_left_out_keeps_no_unknown_member_of_the_failed_value() {
    let (value, kept, _) = kept_after::<Resets>(
        r#"{"entries": {"gone": {"extra": 1, "n": "x"}, "stays": {"extra": 2, "n": 1}}}"#,
        Profile::EVERY_FAILURE.with_recovery(Recovery::Value),
    );
    let value = value.expect("read");
    assert_eq!(
        value
            .entries
            .map(|entries| entries.into_keys().collect::<Vec<_>>()),
        Some(vec!["stays".to_owned()])
    );
    assert_eq!(kept, ["entries/stays/extra"]);
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct KeptThenCounted {
    #[facet(schema::or_kept = "told")]
    size: Option<u8>,
    count: u8,
}

/// A told `or_kept` failure is no failure of the read: it neither stops a first-failure read nor
/// counts as the first failure, so the failure of a member after it is told as the first one.
#[test]
fn a_told_kept_failure_is_not_the_first_failure_of_the_read() {
    for profile in [Profile::FIRST_FAILURE, Profile::FIRST_THEN_PASSES] {
        let (value, told) = read_as::<KeptThenCounted>(
            r#"{"size": "big", "count": "x"}"#,
            Reading::DOCUMENT,
            profile,
        );
        assert_eq!(value, None, "{profile:?}");
        assert_eq!(
            told,
            [
                "Error root/size core/invalid_type",
                "Error root/count core/invalid_type"
            ],
            "{profile:?}"
        );
    }
}

/// A map read in the sorted order of its keys' bytes tells its entries' findings in that order;
/// in place, it holds the entries it read where the file has them.
#[test]
fn a_map_read_sorted_in_place_tells_by_key_and_holds_in_file_order() {
    let text = r#"{"d": 4, "b": "x", "c": 3, "a": "y"}"#;
    let reading = Reading::OWN
        .strict()
        .with_on_member_failure(MemberFailure::DefaultAndTell);
    let profile = Profile::EVERY_FAILURE.with_recovery(Recovery::Value);
    let keys =
        |map: Option<IndexMap<String, u8>>| -> Vec<String> { map.unwrap().into_keys().collect() };
    let (sorted, told) = read_as::<IndexMap<String, u8>>(
        text,
        reading,
        profile.with_members(crate::json::kit::MemberOrder::Sorted),
    );
    assert_eq!(
        told,
        [
            "Info root/a core/invalid_type",
            "Info root/b core/invalid_type"
        ]
    );
    assert_eq!(keys(sorted), ["c", "d"]);
    let (in_place, told) = read_as::<IndexMap<String, u8>>(
        text,
        reading,
        profile.with_members(crate::json::kit::MemberOrder::SortedInPlace),
    );
    assert_eq!(
        told,
        [
            "Info root/a core/invalid_type",
            "Info root/b core/invalid_type"
        ]
    );
    assert_eq!(keys(in_place), ["d", "c"]);
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(transparent)]
struct Weights(#[facet(schema::iterated, schema::drop)] Vec<u8>);

/// A transparent newtype reads its list by its field's keys: iterated as jsoncpp iterates its
/// value, a failed element left out alone.
#[test]
fn a_transparent_list_reads_as_its_field_s_keys_say() {
    let (value, told) = read_as::<Weights>(
        r#"{"b": 2, "a": "x", "c": 3}"#,
        lenient(),
        Profile::EVERY_FAILURE,
    );
    assert_eq!(value, Some(Weights(vec![2, 3])));
    assert_eq!(
        told,
        [
            "Info root core/invalid_type",
            "Info root/a core/invalid_type"
        ]
    );
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Point(u8, Option<String>);

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(
    schema::wrong_type = crate::diagnostic::codes::INVALID_VALUE,
    schema::arity = crate::diagnostic::codes::DROPPED_FIELD
)]
struct Entry(#[facet(default)] String, #[facet(default = 9)] u8);

/// A tuple struct reads an array, each field the element at its place: another length is a
/// wrong type, and a failed element fails the struct.
#[test]
fn a_tuple_struct_reads_an_array_by_position() {
    let (value, told) = as_read::<Point>(r#"[3, "a"]"#);
    assert!(told.is_empty(), "{told:?}");
    assert_eq!(value, Some(Point(3, Some("a".to_owned()))));
    for text in ["[3]", r#"[3, "a", 4]"#, "{}"] {
        let (value, told) = as_read::<Point>(text);
        assert_eq!(value, None, "{text}");
        assert_eq!(told, ["Error root core/invalid_type"], "{text}");
    }
    let (value, told) = as_read::<Point>(r#"["x", 2]"#);
    assert_eq!(value, None);
    assert_eq!(
        told,
        [
            "Error root/0 core/invalid_type",
            "Error root/1 core/invalid_type"
        ]
    );
}

/// A tuple struct's own codes: a value that is no array under `schema::wrong_type`; an array of
/// another length under `schema::arity`, told after its elements and read on, a missing element
/// absent and one past the fields not read.
#[test]
fn a_tuple_struct_tells_its_shape_under_its_own_codes() {
    let read = |text: &str| read_as::<Entry>(text, lenient(), Profile::EVERY_FAILURE);
    let (value, told) = read(r#"{"a": 1}"#);
    assert_eq!(value, None);
    assert_eq!(told, ["Info root core/invalid_value"]);
    let (value, told) = read(r#"["a"]"#);
    assert_eq!(value, Some(Entry("a".to_owned(), 9)));
    assert_eq!(told, ["Info root core/dropped_field"]);
    let (value, told) = read(r#"[[1], 2, "x"]"#);
    assert_eq!(value, Some(Entry(String::new(), 2)));
    assert_eq!(
        told,
        [
            "Info root/0 core/invalid_type",
            "Info root core/dropped_field"
        ]
    );
}

/// A member the writer writes and no file holds (geometry's `__legacy` side-car).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct WrittenOnly {
    #[facet(schema::fallback = 0)]
    v: Option<u8>,
    #[facet(rename = "__side", skip_deserializing)]
    side: Option<u8>,
}

#[test]
fn a_skip_deserializing_field_is_not_read_and_its_member_is_an_unknown_one() {
    let (value, told) = as_read::<WrittenOnly>(r#"{"v": 1, "__side": 2}"#);
    assert_eq!(
        value,
        Some(WrittenOnly {
            v: Some(1),
            side: None
        })
    );
    assert_eq!(told, ["Error root/__side core/unrecognized_field"]);
}

/// The size a crate absent its member gets: 7 where the `kind` read before it is `big`, told
/// as an information at the member it lacks; none otherwise.
fn size_absent(object: &Node<'_>, read: &mut Read<'_>) -> Option<u8> {
    let big = object
        .value()
        .as_object()
        .and_then(|members| members.get("kind"))
        .and_then(|kind| kind.as_string())
        .is_some_and(|kind| kind.as_str() == "big");
    let mut finding =
        crate::diagnostic::Diagnostic::of(&crate::diagnostic::codes::MISSING_PROPERTY)
            .severity(crate::diagnostic::Severity::Info)
            .at(object.path().child("size"));
    finding.location.span = object.span();
    read.push(finding);
    big.then_some(7)
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Crate {
    kind: Option<String>,
    #[facet(schema::absent_with = size_absent)]
    size: Option<u8>,
    label: Option<String>,
}

/// A member the object lacks is asked of the field's `absent_with` fn, in its place where the
/// read asks for members in declaration order (a `null` read as absent too), after the members
/// where it reads them in file order; a member that is written is read as ever.
#[test]
fn an_absent_member_is_what_its_absent_with_fn_says() {
    let declared = Reading::DOCUMENT
        .strict()
        .with_null(NullHandling::AbsentSilently);
    let (value, told) = read_as::<Crate>(
        r#"{"label": 5, "kind": "big"}"#,
        declared,
        Profile::DECLARED_ORDER,
    );
    assert_eq!(value.map(|c| c.size), None, "the label failed the crate");
    assert_eq!(
        told,
        [
            "Info root/size core/missing_property",
            "Error root/label core/invalid_type",
        ]
    );
    let (value, told) = read_as::<Crate>(
        r#"{"kind": "big", "size": null}"#,
        declared,
        Profile::DECLARED_ORDER,
    );
    assert_eq!(value.and_then(|c| c.size), Some(7));
    assert_eq!(told, ["Info root/size core/missing_property"]);
    let (value, told) = read_as::<Crate>(
        r#"{"label": 5, "kind": "small"}"#,
        Reading::DOCUMENT.strict(),
        Profile::EVERY_FAILURE,
    );
    assert!(value.is_none());
    assert_eq!(
        told,
        [
            "Error root/label core/invalid_type",
            "Info root/size core/missing_property",
        ]
    );
    let (value, told) = read_as::<Crate>(r#"{"size": 3}"#, declared, Profile::DECLARED_ORDER);
    assert_eq!(value.and_then(|c| c.size), Some(3));
    assert!(told.is_empty(), "{told:?}");
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Ticket {
    id: u8,
    note: Option<String>,
    #[facet(default)]
    seats: u8,
}

/// Under `DefaultAndRefuse` a member that fails is told as an error and takes its default: the
/// owner is read on and built, and fails only by a required member.
#[test]
fn a_member_that_fails_is_refused_and_left_at_its_default() {
    let reading = Reading::DOCUMENT
        .strict()
        .with_on_member_failure(MemberFailure::DefaultAndRefuse);
    let (value, told) = read_as::<Ticket>(
        r#"{"id": 1, "note": 5, "seats": "x"}"#,
        reading,
        Profile::EVERY_FAILURE,
    );
    assert_eq!(
        value,
        Some(Ticket {
            id: 1,
            note: None,
            seats: 0
        })
    );
    assert_eq!(
        told,
        [
            "Error root/note core/invalid_type",
            "Error root/seats core/invalid_type",
        ]
    );
    let (value, told) = read_as::<Ticket>(r#"{"id": "x"}"#, reading, Profile::EVERY_FAILURE);
    assert_eq!(value, None);
    assert_eq!(told, ["Error root/id core/invalid_type"]);
}

/// A loudness of its own, or the `volume` its table put in scope; `-1` where none did.
#[derive(Clone, Copy, Debug, PartialEq, Facet)]
struct Loudness(f64);

crate::impl_schema_leaf!(Loudness);
crate::impl_no_molang!(Loudness);

impl FromJson for Loudness {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let own = node
            .value()
            .as_number()
            .map(facet_value::VNumber::to_f64_lossy);
        let table = read.scope::<f64>("volume").copied();
        Some(Self(own.or(table).unwrap_or(-1.0)))
    }
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Table {
    #[facet(default = 1.0, schema::scope)]
    volume: f64,
    #[facet(default)]
    sounds: IndexMap<String, Loudness>,
    inner: Option<Box<Table>>,
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Tables {
    table: Table,
    loose: Loudness,
}

/// A `schema::scope` field puts its value in scope for the members read after it and every
/// read below them: its default where the member is absent, the nearest table's where tables
/// nest, and nothing once its struct is read.
#[test]
fn a_scope_field_is_what_the_reads_after_it_see() {
    let text = r#"{"table": {"volume": 0.5, "sounds": {"a": "x", "b": 2},
        "inner": {"sounds": {"c": "x"}}}, "loose": "x"}"#;
    let (value, told) = read_as::<Tables>(text, Reading::OWN, Profile::DECLARED_ORDER);
    assert!(told.is_empty(), "{told:?}");
    let value = value.expect("read");
    assert_eq!(value.table.sounds["a"], Loudness(0.5));
    assert_eq!(value.table.sounds["b"], Loudness(2.0));
    let inner = value.table.inner.expect("inner");
    assert_eq!(
        inner.sounds["c"],
        Loudness(1.0),
        "the inner table's default"
    );
    assert_eq!(value.loose, Loudness(-1.0), "no table is in scope after it");
}
