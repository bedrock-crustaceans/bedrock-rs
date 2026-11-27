//! The reader's components (`versioning::read_component`, `read_component_map_by`) against the
//! old door's (`parse_component`, `parse_component_map`, now deleted): one row per payload shape (a
//! `shorthand = scalar` row of a one-member struct, of an untagged enum, of a transparent
//! newtype, plain rows), every token of a grid, under every reading. The old door's outcomes of the
//! grid and of the component maps are frozen in `schema_read_components.pins`, one line per
//! cell: `key<TAB>read` where the two agreed, `key<TAB>read<TAB>door` where they differed (each
//! such difference one the reader makes by rule, checked while the old door ran: a finding at the
//! member whose value it is about where facet named a step inside it or the owner, class P; one
//! finding where the old door told the same one twice, a `null` given to an untagged enum). The
//! tests hold the reader to the pinned `read`; `BEDROCK_DOOR_PINS=<dir>` writes each test's
//! cells to `<dir>/<test>` instead of checking them.

use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::OnceLock;

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::ComponentMap;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BeyondWidth, ComponentTable, ConvertMode, Family, KeyGate, Loaded, MemberFailure, NoStable,
    Reading, Scalars, UpgradeCx, VersionChain, read_component, read_component_map_by,
    read_component_with, read_slice, read_slice_with,
};
use bedrock_addon_derive::Components;
use facet::Facet;

struct Shelf;

enum ShelfHistory {}

impl VersionChain<Shelf> for ShelfHistory {
    const VERSIONS: &'static [FormatVersion] = &[];

    fn parse(
        _: JsonSlice<'_>,
        _: FormatVersion,
        _: &LoadContext,
        _: &mut Diagnostics,
    ) -> Option<Self> {
        None
    }

    fn upgrade_to_end(self, _: &mut UpgradeCx<'_>) -> Loaded<Shelf> {
        match self {}
    }
}

impl Family for Shelf {
    const NAME: &'static str = "shelf";
    const PAYLOAD_KEY: &'static str = "minecraft:shelf";
    const FLOOR: FormatVersion = FormatVersion::new(1, 16, 0);
    type Latest = NoStable;
    type Beta = Infallible;
    type Component = Part;
    type History = ShelfHistory;

    fn components() -> &'static ComponentTable<Self> {
        Part::TABLE
    }
}

/// `{"value": n}`, or `n`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Damage {
    #[facet(default)]
    pub value: i32,
}

/// `true`, `false` or `{"resistance": x}`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(untagged)]
#[repr(u8)]
pub enum Explode {
    Enabled(bool),
    Params(Params),
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Params {
    pub resistance: f32,
}

/// A bare number.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(transparent)]
pub struct Friction(pub f32);

/// `{"count": n}`, or `n`: a required member.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Count {
    pub count: u8,
}

/// `{"value": n}`, `{}` or `n`: an optional member.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Maybe {
    pub value: Option<i32>,
}

/// `{"mode": …}`, or the mode bare: a member that is an untagged enum.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Blast {
    pub mode: Explode,
}

/// A mood, by name.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
pub enum Mood {
    Calm,
    Angry,
}

/// `{"mood": …}`, or the mood bare: a member that is a unit enum.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Feeling {
    pub mood: Mood,
}

/// A member a later format adds.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Gated {
    pub old: Option<u8>,
    #[facet(schema::since_format = "1.21.0")]
    pub new: Option<u8>,
}

/// A list of names.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(transparent)]
pub struct Tags(pub Vec<String>);

/// Two members, no shorthand.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Pair {
    pub a: i32,
    pub b: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Components)]
#[component(family = Shelf, unknown = Unknown)]
pub enum Part {
    #[component(name = "minecraft:damage", shorthand = scalar, parse = read_component::<Damage, Part>)]
    Damage(Damage),
    #[component(name = "minecraft:explode", shorthand = scalar, parse = read_component::<Explode, Part>)]
    Explode(Explode),
    #[component(name = "minecraft:friction", shorthand = scalar, parse = read_component::<Friction, Part>)]
    Friction(Friction),
    #[component(name = "minecraft:count", shorthand = scalar, parse = read_component::<Count, Part>)]
    Count(Count),
    #[component(name = "minecraft:maybe", shorthand = scalar, parse = read_component::<Maybe, Part>)]
    Maybe(Maybe),
    #[component(name = "minecraft:blast", shorthand = scalar, parse = read_component::<Blast, Part>)]
    Blast(Blast),
    #[component(name = "minecraft:feeling", shorthand = scalar, parse = read_component::<Feeling, Part>)]
    Feeling(Feeling),
    #[component(name = "minecraft:tags", parse = read_component::<Tags, Part>)]
    Tags(Tags),
    #[component(name = "minecraft:pair", parse = read_component::<Pair, Part>)]
    Pair(Pair),
    #[component(name = "minecraft:gated", parse = read_component::<Gated, Part>)]
    Gated(Gated),
    Unknown(Value),
}

const TOKENS: &[&str] = &[
    "5",
    "\"5\"",
    "true",
    "null",
    "0.5",
    "300",
    "1e39",
    "[1]",
    "{}",
    r#"{"value": 5}"#,
    r#"{"value": "x"}"#,
    r#"{"value": null}"#,
    r#"{"count": 2}"#,
    r#"{"count": 300}"#,
    r#"{"resistance": 2}"#,
    r#"{"mode": true}"#,
    r#"{"mode": "no"}"#,
    r#"{"a": 1, "b": 2}"#,
    r#"{"a": 1, "zz": 2}"#,
    r#""no""#,
];

fn readings() -> [(&'static str, Reading); 7] {
    [
        ("document", Reading::DOCUMENT),
        (
            "strict",
            Reading::DOCUMENT
                .with_scalars(Scalars::Strict)
                .with_beyond_width(BeyondWidth::Error),
        ),
        ("null_absent", Reading::NULL_ABSENT),
        ("own", Reading::OWN),
        (
            "non_strict",
            Reading::OWN
                .with_scalars(Scalars::NonStrict)
                .with_beyond_width(BeyondWidth::Error),
        ),
        (
            "default_and_tell",
            Reading::DOCUMENT
                .with_scalars(Scalars::Strict)
                .with_on_member_failure(MemberFailure::DefaultAndTell),
        ),
        ("keeps", Reading::NULL_ABSENT.with_keeps_unknown(true)),
    ]
}

fn context(reading: Reading) -> LoadContext {
    let mut load = LoadContext::detached();
    load.reading = reading;
    load.null_rule = reading.null_rule();
    load
}

/// The value (when nothing rejects it) and the findings as `severity code path`, with the
/// table of kept members.
fn outcome<T: std::fmt::Debug>(value: Option<&T>, d: &Diagnostics) -> String {
    let rejected = d.iter().any(|f| f.severity >= Severity::Error);
    let value = if rejected {
        "rejected".to_owned()
    } else {
        format!("{value:?}")
    };
    let found: Vec<String> = d
        .iter()
        .map(|f| format!("{:?} {} {}", f.severity, f.code.id(), f.location.json_path))
        .collect();
    let kept: Vec<String> = d
        .unknown()
        .iter()
        .map(|(at, key, _)| format!("{at}/{key}"))
        .collect();
    format!("{value} [{}] kept [{}]", found.join("; "), kept.join("; "))
}

/// The frozen cells: key to `(read, door)`, `door` absent where it is `read`.
fn frozen() -> &'static HashMap<&'static str, (&'static str, Option<&'static str>)> {
    static FROZEN: OnceLock<HashMap<&'static str, (&'static str, Option<&'static str>)>> =
        OnceLock::new();
    FROZEN.get_or_init(|| {
        include_str!("schema_read_components.pins")
            .lines()
            .filter_map(|line| {
                let mut parts = line.split('\t');
                Some((parts.next()?, (parts.next()?, parts.next())))
            })
            .collect()
    })
}

/// The cell `key` of `test`: the reader's outcome `read` must be the pinned one. Under
/// `BEDROCK_DOOR_PINS` the cell (with the pinned old door's outcome) is written to that directory
/// instead.
fn cell(test: &str, key: &str, read: &str) {
    let pinned = frozen().get(key).copied();
    if let Ok(dir) = std::env::var("BEDROCK_DOOR_PINS") {
        use std::io::Write as _;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(std::path::Path::new(&dir).join(test))
            .expect("the pin directory");
        let line = match pinned.and_then(|(_, door)| door) {
            Some(door) => format!("{key}\t{read}\t{door}\n"),
            None => format!("{key}\t{read}\n"),
        };
        file.write_all(line.as_bytes()).expect("a pin line");
        return;
    }
    let Some((pinned_read, _)) = pinned else {
        panic!("no pin for {key}");
    };
    assert_eq!(read, pinned_read, "the reader at {key}");
}

/// `text` read as `T` of `Part` under `reading`.
fn read_as<T>(reading: Reading, text: &str) -> Outcome
where
    T: bedrock_addon_core::schema::FromJson + for<'a> Facet<'a> + Into<Part>,
{
    let cx = context(reading);
    let mut d = Diagnostics::default();
    let read = read_component::<T, Part>(JsonSlice::new(text), ConvertMode::Strict, &cx, &mut d);
    outcome_of(read.as_ref(), &d)
}

/// One finding as the comparison sees it.
#[derive(Clone, Debug, PartialEq)]
struct Told {
    severity: Severity,
    code: String,
    path: String,
}

/// An outcome: the value (or `rejected`), the findings, the kept members.
#[derive(Debug, PartialEq)]
struct Outcome {
    value: String,
    told: Vec<Told>,
    kept: Vec<String>,
}

fn outcome_of<T: std::fmt::Debug>(value: Option<&T>, d: &Diagnostics) -> Outcome {
    let rejected = d.iter().any(|f| f.severity >= Severity::Error);
    Outcome {
        value: if rejected {
            "rejected".to_owned()
        } else {
            format!("{value:?}")
        },
        told: d
            .iter()
            .map(|f| Told {
                severity: f.severity,
                code: f.code.id().to_owned(),
                path: f.location.json_path.to_string(),
            })
            .collect(),
        kept: d
            .unknown()
            .iter()
            .map(|(at, key, _)| format!("{at}/{key}"))
            .collect(),
    }
}

#[test]
fn a_component_reads_as_the_door_reads_it() {
    let mut compared = 0;
    for (name, reading) in readings() {
        for token in TOKENS {
            let cases = [
                ("damage", read_as::<Damage>(reading, token)),
                ("explode", read_as::<Explode>(reading, token)),
                ("friction", read_as::<Friction>(reading, token)),
                ("count", read_as::<Count>(reading, token)),
                ("maybe", read_as::<Maybe>(reading, token)),
                ("blast", read_as::<Blast>(reading, token)),
                ("pair", read_as::<Pair>(reading, token)),
                ("tags", read_as::<Tags>(reading, token)),
            ];
            for (row, read) in cases {
                compared += 1;
                let key = format!("component {name} {row} {token}");
                cell("component", &key, &format!("{read:?}"));
            }
        }
    }
    assert!(compared > 800);
}

/// `demo:*` keys are custom components.
fn is_custom(key: &str) -> bool {
    key.starts_with("demo:")
}

/// A `components` object read whole by both map readers: each component by its row, a failed
/// one dropped alone, a custom one kept, an unknown one kept and told.
#[test]
fn a_component_map_reads_as_the_door_reads_it() {
    let texts = [
        r#"{"minecraft:damage": 5, "minecraft:count": "x", "demo:glow": {"a": 1}, "minecraft:zz": 1}"#,
        r#"{"minecraft:pair": {"a": 1, "zz": 2}, "minecraft:maybe": null}"#,
        r#"{"minecraft:friction": 0.5, "minecraft:explode": {"resistance": "x"}}"#,
    ];
    for (name, reading) in readings() {
        for text in texts {
            let cx = context(reading);
            let at = JsonPath::of_keys(["components"]);
            let parsed = bedrock_addon_core::json::parse::parse(
                text,
                bedrock_addon_core::json::parse::Options::default(),
            )
            .expect("a text");
            let mut d = Diagnostics::default();
            let read = {
                let mut read = Read::new(reading, &mut d).load(&cx);
                read_component_map_by::<Shelf>(
                    &Node::parsed(&parsed, at.clone()),
                    &mut read,
                    ConvertMode::Strict,
                    is_custom,
                    |_| KeyGate::Read,
                )
            };
            let read = map_outcome(&read.unwrap_or_default(), &d);
            cell("map", &format!("map {name} {text}"), &read);
        }
    }
}

fn map_outcome(map: &ComponentMap<Part>, d: &Diagnostics) -> String {
    let entries: Vec<String> = map.iter().map(|entry| format!("{entry:?}")).collect();
    format!("{} {}", entries.join(", "), outcome(Some(&()), d))
}

/// A component whose JSON is of the wrong kind for its type's root is facet-json's malformed
/// input: `core/syntax`, "unexpected token", an error at the component spanning its text, in
/// facet-json's words.
#[test]
#[allow(
    clippy::too_many_lines,
    reason = "one case of the table per three lines"
)]
fn a_component_of_the_wrong_json_kind_is_the_door_s_syntax_error() {
    let told = |d: &Diagnostics| -> Vec<(Severity, String, String, bool, String)> {
        d.iter()
            .map(|f| {
                (
                    f.severity,
                    f.code.id().to_owned(),
                    f.location.json_path.to_string(),
                    f.location.span.is_some(),
                    f.code.to_string(),
                )
            })
            .collect()
    };
    let cases: [(&str, &str, &str); 12] = [
        (
            "pair",
            "[1]",
            "unexpected token: got sequence start, expected struct start",
        ),
        (
            "pair",
            "null",
            "unexpected token: got scalar, expected struct start",
        ),
        (
            "pair",
            "5",
            "unexpected token: got scalar, expected struct start",
        ),
        (
            "friction",
            "[1]",
            "unexpected token: got sequence start, expected scalar value",
        ),
        (
            "friction",
            "{}",
            "unexpected token: got node without argument, expected scalar value or node with argument",
        ),
        (
            "damage",
            "[1]",
            "unexpected token: got sequence start, expected scalar value",
        ),
        (
            "count",
            "[]",
            "unexpected token: got sequence start, expected scalar value",
        ),
        (
            "maybe",
            "[1]",
            "unexpected token: got sequence start, expected scalar value",
        ),
        (
            "damage",
            "[1, 2]",
            "unexpected token: got sequence start, expected scalar value",
        ),
        (
            "tags",
            "5",
            "unexpected token: got scalar, expected sequence start",
        ),
        (
            "tags",
            "null",
            "unexpected token: got scalar, expected sequence start",
        ),
        ("tags", "{}", "unexpected token: got object, expected array"),
    ];
    let cx = context(Reading::DOCUMENT);
    for (row, token, text) in cases {
        let slice = JsonSlice::new(token);
        let mut read = Diagnostics::default();
        match row {
            "pair" => {
                read_component::<Pair, Part>(slice, ConvertMode::Strict, &cx, &mut read);
            }
            "friction" => {
                read_component::<Friction, Part>(slice, ConvertMode::Strict, &cx, &mut read);
            }
            "damage" => {
                read_component::<Damage, Part>(slice, ConvertMode::Strict, &cx, &mut read);
            }
            "tags" => {
                read_component::<Tags, Part>(slice, ConvertMode::Strict, &cx, &mut read);
            }
            "count" => {
                read_component::<Count, Part>(slice, ConvertMode::Strict, &cx, &mut read);
            }
            _ => {
                read_component::<Maybe, Part>(slice, ConvertMode::Strict, &cx, &mut read);
            }
        }
        let want = vec![(
            Severity::Error,
            "core/syntax".to_owned(),
            String::new(),
            true,
            text.to_owned(),
        )];
        assert_eq!(told(&read), want, "the reader, {row} {token}");
    }
}

/// A bare array given to a shorthand's unit enum is facet-json's malformed input whatever the
/// slice's scalar rule says: facet's enum read takes a name or an object.
#[test]
fn an_array_given_to_a_shorthand_unit_enum_is_the_door_s_syntax_error() {
    for (name, reading) in readings() {
        let cx = context(reading);
        for token in ["[]", "[1]"] {
            let slice = JsonSlice::new(token);
            let mut read = Diagnostics::default();
            read_component::<Feeling, Part>(slice, ConvertMode::Strict, &cx, &mut read);
            let told = |d: &Diagnostics| -> Vec<String> {
                d.iter()
                    .map(|f| format!("{:?} {} {}", f.severity, f.code.id(), f.code))
                    .collect()
            };
            assert_eq!(
                told(&read),
                [
                    "Error core/syntax unexpected token: got sequence start, expected string or struct for enum"
                ],
                "{name} {token}"
            );
        }
    }
}

/// A member `schema::since_format` a later version is an unknown member of its component in a
/// document written at an earlier one (the component map knows the document's format).
#[test]
fn a_member_since_a_format_is_unknown_in_a_component_of_an_older_document() {
    let text = r#"{"minecraft:gated": {"old": 1, "new": 2}}"#;
    let cx = context(Reading::DOCUMENT);
    let parsed = bedrock_addon_core::json::parse::parse(
        text,
        bedrock_addon_core::json::parse::Options::default(),
    )
    .expect("a text");
    let at = JsonPath::of_keys(["components"]);
    let read_at = |format: Option<FormatVersion>| {
        let mut d = Diagnostics::default();
        let map = {
            let mut read = Read::new(Reading::DOCUMENT, &mut d)
                .load(&cx)
                .format(format);
            read_component_map_by::<Shelf>(
                &Node::parsed(&parsed, at.clone()),
                &mut read,
                ConvertMode::Strict,
                is_custom,
                |_| KeyGate::Read,
            )
        };
        let told: Vec<String> = d
            .iter()
            .map(|f| format!("{} {}", f.code.id(), f.location.json_path))
            .collect();
        (format!("{:?}", map.unwrap_or_default()), told)
    };
    let (older, told) = read_at(Some(FormatVersion::new(1, 20, 0)));
    assert_eq!(
        told,
        ["core/unrecognized_field components/minecraft:gated/new"]
    );
    assert!(older.contains("new: None"), "{older}");
    let (newer, told) = read_at(Some(FormatVersion::new(1, 21, 0)));
    assert!(told.is_empty(), "{told:?}");
    assert!(newer.contains("new: Some(2)"), "{newer}");
    // A read that knows no format reads every member.
    let (unknown, told) = read_at(None);
    assert!(told.is_empty(), "{told:?}");
    assert!(unknown.contains("new: Some(2)"), "{unknown}");
}

/// A value read from a slice is a component without a shorthand: the context's reading, every
/// finding at the value and spanning it.
#[test]
fn a_slice_reads_as_a_component_without_a_shorthand() {
    let cx = context(Reading::DOCUMENT);
    let read = |text: &str| {
        let mut d = Diagnostics::default();
        let value: Option<Friction> = read_slice(JsonSlice::new(text), &cx, &mut d);
        let told: Vec<String> = d
            .iter()
            .map(|f| format!("{:?} {} {:?}", f.severity, f.code.id(), f.location.span))
            .collect();
        (value, told)
    };
    assert_eq!(read("0.5"), (Some(Friction(0.5)), Vec::new()));
    let (value, told) = read(r#""slick""#);
    assert_eq!(value, None);
    assert_eq!(told.len(), 1, "{told:?}");
    assert!(
        told[0].starts_with("Error core/invalid_type Some(Span"),
        "{told:?}"
    );
    // A value of the wrong JSON kind for the root is facet-json's syntax error, as a component's.
    let (value, told) = read("[1]");
    assert_eq!(value, None);
    assert!(
        told[0].starts_with("Error core/syntax Some(Span"),
        "{told:?}"
    );
}

/// A family judges the keys first: a refused one is told before any row reads and left out, one
/// the slice has no row for is told after the rows and kept as an unknown component.
#[test]
fn a_key_gate_refuses_a_key_or_keeps_it_unread() {
    let text = r#"{"minecraft:damage": 5, "minecraft:count": {"count": 1}, "minecraft:friction": "x", "minecraft:pair": {"a": 1}}"#;
    let parsed = bedrock_addon_core::json::parse::parse(
        text,
        bedrock_addon_core::json::parse::Options::default(),
    )
    .expect("a text");
    let cx = context(Reading::DOCUMENT);
    let mut d = Diagnostics::default();
    let map = {
        let mut read = Read::new(Reading::DOCUMENT, &mut d).load(&cx);
        read_component_map_by::<Shelf>(
            &Node::parsed(&parsed, JsonPath::of_keys(["components"])),
            &mut read,
            ConvertMode::Strict,
            is_custom,
            |key| match key {
                "minecraft:count" => {
                    KeyGate::Refuse(bedrock_addon_core::diagnostic::Diagnostic::of(
                        &bedrock_addon_core::diagnostic::codes::DROPPED_FIELD,
                    ))
                }
                "minecraft:damage" => {
                    KeyGate::Unread(bedrock_addon_core::diagnostic::Diagnostic::of(
                        &bedrock_addon_core::diagnostic::codes::INVALID_TYPE,
                    ))
                }
                _ => KeyGate::Read,
            },
        )
    }
    .expect("an object");
    let told: Vec<String> = d
        .iter()
        .map(|f| {
            format!(
                "{} {} {}",
                f.code.id(),
                f.location.json_path,
                f.location.span.is_some()
            )
        })
        .collect();
    assert_eq!(
        told,
        [
            "core/dropped_field components/minecraft:count true",
            "core/invalid_type components/minecraft:friction true",
            "core/invalid_type components/minecraft:damage true",
        ]
    );
    let names: Vec<String> = map.iter().map(|entry| format!("{entry:?}")).collect();
    assert_eq!(names.len(), 2, "{names:?}");
    assert!(names[0].contains("Pair"), "{names:?}");
    assert!(names[1].contains("damage"), "{names:?}");
}

/// A key a family drops is told where the file has it, among the rows' own findings.
#[test]
fn a_key_gate_drops_a_key_in_the_files_order() {
    use bedrock_addon_core::diagnostic::{Diagnostic, codes};
    let text = r#"{"minecraft:count": {"count": 1, "zz": 1}, "minecraft:damage": 5, "minecraft:pair": {"a": 1, "yy": 1}}"#;
    let parsed = bedrock_addon_core::json::parse::parse(
        text,
        bedrock_addon_core::json::parse::Options::default(),
    )
    .expect("a text");
    let cx = context(Reading::DOCUMENT);
    let mut d = Diagnostics::default();
    let map = {
        let mut read = Read::new(Reading::DOCUMENT, &mut d).load(&cx);
        read_component_map_by::<Shelf>(
            &Node::parsed(&parsed, JsonPath::of_keys(["components"])),
            &mut read,
            ConvertMode::Strict,
            is_custom,
            |key| match key {
                "minecraft:damage" => KeyGate::Drop(Diagnostic::of(&codes::DROPPED_FIELD)),
                _ => KeyGate::Read,
            },
        )
    }
    .expect("an object");
    let told: Vec<String> = d
        .iter()
        .map(|f| format!("{} {}", f.code.id(), f.location.json_path))
        .collect();
    assert_eq!(
        told,
        [
            "core/unrecognized_field components/minecraft:count/zz",
            "core/dropped_field components/minecraft:damage",
            "core/unrecognized_field components/minecraft:pair/yy",
        ]
    );
    assert_eq!(map.iter().count(), 2);
}

#[test]
fn a_component_read_keeps_its_unknown_members_where_the_read_says_so_whatever_the_reading() {
    let cx = context(Reading::OWN);
    let text = JsonSlice::new(r#"{"value": 3, "zz": 1}"#);
    let kept_by = |keeps: bool| {
        let mut d = Diagnostics::default();
        let part =
            read_component_with::<Damage, Part>(text, ConvertMode::Strict, &cx, &mut d, |read| {
                read.keeps_unknown(keeps)
            });
        assert_eq!(part, Some(Part::Damage(Damage { value: 3 })));
        let codes: Vec<&str> = d.iter().map(|f| f.code.id()).collect();
        (d.unknown().iter().count(), codes.len())
    };
    // The reading keeps none: the member is told, and nothing is held for the write-back.
    assert_eq!(kept_by(false), (0, 1), "the reading's own disposal");
    // The read keeps: the member is held, and told as a kept one (the grade is the reading's).
    assert_eq!(kept_by(true), (1, 1), "the read's keeping");
    // A slice read the same way.
    let mut d = Diagnostics::default();
    let slice: Option<Damage> = read_slice_with(text, &cx, &mut d, |read| read.keeps_unknown(true));
    assert_eq!(slice, Some(Damage { value: 3 }));
    assert_eq!(d.unknown().iter().count(), 1);
}
