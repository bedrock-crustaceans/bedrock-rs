//! The new reader's components (`versioning::read_component`, `read_component_map`) against
//! the door's (`parse_component`, `parse_component_map`): one row per payload shape (a
//! `shorthand = scalar` row of a one-member struct, of an untagged enum, of a transparent
//! newtype, plain rows), every token of a grid, under every reading. Where the two tell the
//! same codes, severities and paths (spans aside: the new reader spans every finding) and read
//! the same value, nothing is said; a difference is one the reader makes on purpose, by rule.

use std::convert::Infallible;
use std::fmt::Write as _;

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::ComponentMap;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::schema::Schema;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BeyondWidth, ComponentTable, ConvertMode, Family, Loaded, MemberFailure, NoStable, Reading,
    Scalars, UpgradeCx, VersionChain, parse_component, parse_component_map, read_component,
    read_component_map,
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
    #[component(name = "minecraft:pair", parse = read_component::<Pair, Part>)]
    Pair(Pair),
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

fn readings() -> [(&'static str, Reading); 6] {
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
    load.scalar_rules = reading.scalar_rules();
    load.keeps_unknown = reading.keeps_unknown;
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

/// `text` read as `T` of `Part` by both readers under `reading`: `(door, read)`.
fn both<T>(reading: Reading, text: &str) -> (Outcome, Outcome)
where
    T: bedrock_addon_core::schema::FromJson + for<'a> Facet<'a> + Into<Part>,
{
    let cx = context(reading);
    let slice = JsonSlice::new(text);
    let mut d = Diagnostics::default();
    let door = parse_component::<T, Part>(slice, ConvertMode::Strict, &cx, &mut d);
    let door = outcome_of(door.as_ref(), &d);
    let mut d = Diagnostics::default();
    let read = read_component::<T, Part>(slice, ConvertMode::Strict, &cx, &mut d);
    (door, outcome_of(read.as_ref(), &d))
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

/// Why the reader tells `read` where the door tells `door`, when the difference is one of the
/// reader's rules:
/// - a finding at the member whose value it is about, where facet named a step inside it (a
///   key of an object given to a number) or the owner (an integer out of its range): class P;
/// - a wrong type where facet told a value of another JSON kind as its malformed input
///   (`core/syntax`, "unexpected token"), which the door's components do for a value given to
///   a struct, a transparent scalar or a shorthand's member: the reader words every wrong JSON
///   kind alike;
/// - one finding where the door told the same one twice (a `null` given to an untagged enum).
fn explained(door: &Outcome, read: &Outcome) -> bool {
    let mut told = door.told.clone();
    told.dedup();
    door.value == read.value
        && door.kept == read.kept
        && told.len() == read.told.len()
        && told.iter().zip(&read.told).all(|(door, read)| {
            let path = read.path == door.path
                || door.path.starts_with(&read.path)
                || (door.path.is_empty() && !read.path.contains('/'));
            let code = door.code == read.code
                || (door.code == "core/syntax" && read.code == "core/invalid_type");
            door.severity == read.severity && path && code
        })
}

#[test]
fn a_component_reads_as_the_door_reads_it() {
    let mut unexplained = String::new();
    let mut compared = 0;
    for (name, reading) in readings() {
        for token in TOKENS {
            let cases = [
                ("damage", both::<Damage>(reading, token)),
                ("explode", both::<Explode>(reading, token)),
                ("friction", both::<Friction>(reading, token)),
                ("count", both::<Count>(reading, token)),
                ("maybe", both::<Maybe>(reading, token)),
                ("blast", both::<Blast>(reading, token)),
                ("pair", both::<Pair>(reading, token)),
            ];
            for (row, (door, read)) in cases {
                compared += 1;
                if door != read && !explained(&door, &read) {
                    let _ = writeln!(
                        unexplained,
                        "{name} {row} {token}\n  door {door:?}\n  read {read:?}"
                    );
                }
            }
        }
    }
    assert!(compared > 800);
    assert!(
        unexplained.is_empty(),
        "--- unexplained ---\n{unexplained}--- end ---"
    );
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
    let mut unexplained = String::new();
    for (name, reading) in readings() {
        for text in texts {
            let cx = context(reading);
            let at = JsonPath::of_keys(["components"]);
            let raw: indexmap::IndexMap<String, facet_json::RawJson<'_>> =
                bedrock_addon_core::json::from_str_borrowed_jsonc(text).expect("raw members");
            let mut d = Diagnostics::default();
            let door = parse_component_map::<Shelf>(
                &raw,
                JsonSlice::new(text),
                &at,
                ConvertMode::Strict,
                is_custom,
                &cx,
                &mut d,
            );
            let door = map_outcome(&door, &d);
            let parsed = bedrock_addon_core::json::parse::parse(
                text,
                bedrock_addon_core::json::parse::Options::default(),
            )
            .expect("a text");
            let mut d = Diagnostics::default();
            let read = {
                let mut read = Read::new(reading, &mut d).load(&cx);
                read_component_map::<Shelf>(
                    &Node::parsed(&parsed, at.clone()),
                    &mut read,
                    ConvertMode::Strict,
                    is_custom,
                )
            };
            let read = map_outcome(&read.unwrap_or_default(), &d);
            if door != read {
                let _ = writeln!(unexplained, "{name} {text}\n  door {door}\n  read {read}");
            }
        }
    }
    assert!(
        unexplained.is_empty(),
        "--- unexplained ---\n{unexplained}--- end ---"
    );
}

fn map_outcome(map: &ComponentMap<Part>, d: &Diagnostics) -> String {
    let entries: Vec<String> = map.iter().map(|entry| format!("{entry:?}")).collect();
    format!("{} {}", entries.join(", "), outcome(Some(&()), d))
}
