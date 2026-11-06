//! Which unknown members the door keeps in the document's table: a struct marked with
//! `extra: Extra`, or stating `schema::unknown = "keep"` (on itself or on the field that holds
//! it), keeps them; under a slice whose `Reading::keeps_unknown` is set every other struct
//! does too, unless it states `report`, `reject` or `ignore`. One not kept is told by the word
//! its struct states: `UnrecognizedField` (as every unknown member was told before the words
//! were read), `DroppedField` for `reject`, `IgnoredMember` for `ignore`. A member inside a map
//! value is never kept, whatever its struct says, as before.

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::{Extra, Unknown};
use bedrock_addon_core::json::{
    EncodeError, Indent, JsonSlice, Value, WriteStyle, write_json_with,
};
use bedrock_addon_core::load::{DocumentSource, LoadContext};
use bedrock_addon_core::schema;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    ComponentTable, Family, LoadOutcome, Loaded, ParseCx, Reading, StructIndex, UnknownMask,
    VersionStruct, Versioned, load, parse_payload, write_payload,
};
use facet::Facet;
use indexmap::IndexMap;

use crate::toy::ToyComponent;

/// Marked with `extra: Extra`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Marked {
    #[facet(default)]
    pub v: Option<u8>,
    #[facet(skip)]
    pub extra: Extra,
}

/// States `keep`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct Keeps {
    #[facet(default)]
    pub v: Option<u8>,
}

/// States nothing.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Plain {
    #[facet(default)]
    pub v: Option<u8>,
}

/// States `report`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "report")]
pub struct Reports {
    #[facet(default)]
    pub v: Option<u8>,
}

/// States `reject`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "reject")]
pub struct Rejects {
    #[facet(default)]
    pub v: Option<u8>,
}

/// States `ignore`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "ignore")]
pub struct Ignores {
    #[facet(default)]
    pub v: Option<u8>,
}

/// The payload: states nothing itself, so its own unknown members go by the reading.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Payload {
    #[facet(default)]
    pub marked: Option<Marked>,
    #[facet(default)]
    pub keeps: Option<Keeps>,
    #[facet(default)]
    pub plain: Option<Plain>,
    #[facet(default)]
    pub reports: Option<Reports>,
    #[facet(default)]
    pub rejects: Option<Rejects>,
    #[facet(default)]
    pub ignores: Option<Ignores>,
    /// `report` stated, under a reading that keeps.
    #[facet(default)]
    pub listed: Vec<Reports>,
    /// A map value is not kept, even of a struct that keeps.
    #[facet(default)]
    pub map: IndexMap<String, Keeps>,
    /// The field states `keep` for the struct it holds.
    #[facet(default, schema::unknown = "keep")]
    pub field_keeps: Option<Plain>,
    /// The field states `report` for a struct that keeps by its reading.
    #[facet(default, schema::unknown = "report")]
    pub field_reports: Option<Plain>,
}

impl Versioned for Payload {
    const VERSION: FormatVersion = FormatVersion::new(1, 0, 0);
}

impl VersionStruct for Payload {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        parse_payload(payload, cx, d)
    }
    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl bedrock_addon_core::molang::MolangFields for Payload {
    fn visit_molang(&mut self, _: &mut bedrock_addon_core::molang::MolangVisitor<'_>) {}
}

macro_rules! keep_toy {
    ($name:ident, $module:ident, $reading:expr) => {
        pub struct $name;
        pub mod $module {
            use super::*;
            bedrock_addon_core::version_chain! {
                family = $name;
                /// One struct.
                pub enum History {
                    V1_0_0(Payload),
                }
            }
        }
        impl Family for $name {
            const NAME: &'static str = "keep toy";
            const PAYLOAD_KEY: &'static str = "minecraft:toy";
            const FLOOR: FormatVersion = FormatVersion::new(1, 0, 0);
            type Latest = Payload;
            type Beta = std::convert::Infallible;
            type Component = ToyComponent;
            type History = $module::History;
            fn components() -> &'static ComponentTable<Self> {
                &ComponentTable::EMPTY
            }
            fn reading(_: StructIndex, _: DocumentSource) -> Reading {
                $reading
            }
        }
    };
}

keep_toy!(
    ReadingKeeps,
    reading_keeps,
    Reading::OWN.with_keeps_unknown(true)
);
keep_toy!(ReadingReports, reading_reports, Reading::OWN);
keep_toy!(
    StrictReports,
    strict_reports,
    Reading::OWN.with_unknown(UnknownMask::Error)
);

/// One unknown member `u` in every object of the payload, and `zz` in the payload itself.
const PAYLOAD: &str = r#"{
    "marked": {"v": 1, "u": 1},
    "keeps": {"v": 1, "u": 2},
    "plain": {"v": 1, "u": 3},
    "reports": {"v": 1, "u": 4},
    "rejects": {"v": 1, "u": 5},
    "ignores": {"v": 1, "u": 6},
    "listed": [{"v": 1, "u": 7}],
    "map": {"k": {"v": 1, "u": 8}},
    "field_keeps": {"v": 1, "u": 9},
    "field_reports": {"v": 1, "u": 10},
    "zz": 11
}"#;

fn document(payload: &str) -> String {
    format!(r#"{{"format_version": "1.0.0", "minecraft:toy": {payload}}}"#)
}

fn load_as<F: Family>(text: &str) -> LoadOutcome<F> {
    load::<F>(JsonSlice::new(text), &LoadContext::detached())
}

/// The kept members, as `path=value` below the payload.
fn kept(unknown: &Unknown) -> Vec<String> {
    unknown
        .iter()
        .map(|(at, key, value)| {
            format!(
                "{}={}",
                at.child(key),
                bedrock_addon_core::json::to_json(value).unwrap().trim_end()
            )
        })
        .collect()
}

/// The findings, as `path code severity`.
fn told(d: &Diagnostics) -> Vec<String> {
    d.iter()
        .map(|f| {
            // By legacy name: the variant and the definitions that were it.
            let code = match f.code.name() {
                name @ ("UnrecognizedField" | "DroppedField" | "IgnoredMember") => name,
                _ => "other",
            };
            format!("{} {code} {:?}", f.location.json_path, f.severity)
        })
        .collect()
}

/// What the told list of a reading that does not make unknown members errors is, kept or not.
fn told_at(severity: Severity) -> Vec<String> {
    [
        ("marked/u", "UnrecognizedField"),
        ("keeps/u", "UnrecognizedField"),
        ("plain/u", "UnrecognizedField"),
        ("reports/u", "UnrecognizedField"),
        ("rejects/u", "DroppedField"),
        ("ignores/u", "IgnoredMember"),
        ("listed/0/u", "UnrecognizedField"),
        ("map/k/u", "UnrecognizedField"),
        ("field_keeps/u", "UnrecognizedField"),
        ("field_reports/u", "UnrecognizedField"),
        ("zz", "UnrecognizedField"),
    ]
    .iter()
    .map(|(path, code)| format!("minecraft:toy/{path} {code} {severity:?}"))
    .collect()
}

#[test]
fn a_reading_that_keeps_keeps_every_struct_s_members_but_those_that_say_otherwise() {
    let outcome = load_as::<ReadingKeeps>(&document(PAYLOAD));
    assert!(outcome.value.is_some(), "{:?}", outcome.diagnostics);
    assert_eq!(
        kept(outcome.diagnostics.unknown()),
        [
            "minecraft:toy/zz=11",
            "minecraft:toy/field_keeps/u=9",
            "minecraft:toy/keeps/u=2",
            "minecraft:toy/marked/u=1",
            "minecraft:toy/plain/u=3",
        ]
    );
    assert_eq!(told(&outcome.diagnostics), told_at(Severity::Warning));
}

#[test]
fn a_reading_that_reports_keeps_only_the_structs_that_say_keep() {
    let outcome = load_as::<ReadingReports>(&document(PAYLOAD));
    assert!(outcome.value.is_some(), "{:?}", outcome.diagnostics);
    assert_eq!(
        kept(outcome.diagnostics.unknown()),
        [
            "minecraft:toy/field_keeps/u=9",
            "minecraft:toy/keeps/u=2",
            "minecraft:toy/marked/u=1",
        ]
    );
    assert_eq!(told(&outcome.diagnostics), told_at(Severity::Warning));
}

#[test]
fn under_a_strict_mask_every_disposition_is_an_error() {
    let outcome = load_as::<StrictReports>(&document(PAYLOAD));
    assert!(outcome.value.is_none());
    assert_eq!(told(&outcome.diagnostics), told_at(Severity::Error));
}

/// Read, write with the table, read again: the same value and the same table, and the text
/// written holds the kept members and no other unknown one.
#[test]
fn what_a_reading_keeps_is_written_back_and_read_the_same() {
    let first = load_as::<ReadingKeeps>(&document(PAYLOAD));
    let Some(Loaded::Stable(payload)) = &first.value else {
        panic!("{:?}", first.diagnostics);
    };
    let mut unknown = first.diagnostics.unknown().clone();
    unknown.rebase(&JsonPath::of_keys(["minecraft:toy"]), &JsonPath::new());
    let mut out = Vec::new();
    let style = WriteStyle {
        indent: Indent::Compact,
        trailing_newline: false,
    };
    let dropped = write_json_with(payload, &unknown, &mut out, style).unwrap();
    assert!(
        dropped.is_empty(),
        "every kept member has its object: {dropped:?}"
    );
    let written = String::from_utf8(out).unwrap();
    for member in [r#""u":1"#, r#""u":2"#, r#""u":3"#, r#""u":9"#, r#""zz":11"#] {
        assert!(written.contains(member), "{member} in {written}");
    }
    for member in [
        r#""u":4"#,
        r#""u":5"#,
        r#""u":6"#,
        r#""u":7"#,
        r#""u":8"#,
        r#""u":10"#,
    ] {
        assert!(!written.contains(member), "{member} in {written}");
    }
    let second = load_as::<ReadingKeeps>(&document(&written));
    assert_eq!(
        second.value.as_ref().map(stable),
        first.value.as_ref().map(stable)
    );
    assert_eq!(second.diagnostics.unknown(), first.diagnostics.unknown());
}

fn stable(loaded: &Loaded<ReadingKeeps>) -> &Payload {
    match loaded {
        Loaded::Stable(payload) => payload,
        Loaded::Beta(never) => match *never {},
    }
}

/// Pin, a question for the diagnostics table: the door tells a member its struct reports
/// (`schema::unknown = "report"`, or nothing under a reading that does not keep) with the code
/// of a kept one, `UnrecognizedField`, though the member is not kept; a hand reader's kit tells
/// the same `Report` as `MemberNotKept`. Today's disposition, pinned, not decided.
#[test]
fn pin_the_door_tells_a_reported_member_as_it_tells_a_kept_one() {
    let outcome = load_as::<ReadingReports>(&document(PAYLOAD));
    assert!(outcome.value.is_some(), "{:?}", outcome.diagnostics);
    let told = told(&outcome.diagnostics);
    let kept = kept(outcome.diagnostics.unknown());
    for (path, is_kept) in [("marked/u", true), ("reports/u", false), ("plain/u", false)] {
        assert!(
            told.contains(&format!("minecraft:toy/{path} UnrecognizedField Warning")),
            "{path}: {told:?}"
        );
        assert_eq!(
            kept.iter()
                .any(|member| member.starts_with(&format!("minecraft:toy/{path}="))),
            is_kept,
            "{path}: {kept:?}"
        );
    }
}

/// The door tells every unknown member at the reading's grade of an unknown member, whatever
/// its struct does with it (kept, reported, rejected or ignored): a strict slice grades them
/// to an error where they are raised, with no lift after the parse. Read here outside the
/// load, with the reading set to the strict mask, every line is an error.
#[test]
fn the_door_tells_every_unknown_member_at_the_reading_s_grade() {
    let mut load = LoadContext::detached();
    load.reading = load.reading.with_unknown(UnknownMask::Error);
    let cx = ParseCx {
        load: &load,
        written: FormatVersionInput {
            version: Some(Payload::VERSION),
            beta_literal: false,
            prerelease: false,
        },
        original_version: Payload::VERSION,
        at: JsonPath::of_keys(["minecraft:toy"]),
    };
    let mut d = Diagnostics::default();
    let payload: Option<Payload> = parse_payload(JsonSlice::new(PAYLOAD), &cx, &mut d);
    assert!(payload.is_some(), "{d:?}");
    let expected: Vec<String> = told_at(Severity::Warning)
        .into_iter()
        .map(|line| line.replace("Warning", "Error"))
        .collect();
    assert_eq!(told(&d), expected);
}
