//! The reader (`versioning::read_payload`, the derived `FromJson`) against the door it replaced
//! (`versioning::parse_payload`, facet-json's typed read and its passes, now deleted): one struct
//! per member type, every token of a grid, under every reading. The door's outcomes are frozen
//! in `schema_read_door.pins`, one line per cell: `key<TAB>read` where the two agreed,
//! `key<TAB>read<TAB>door` where they differed. Each test holds the reader to the pinned `read`,
//! and every pinned difference must be one the reader makes on purpose (`explained`, the
//! documents' rules): codes, severities and paths (spans aside: the reader spans every
//! finding's value) and the value read. Set `BEDROCK_DOOR_PINS=<dir>` to write each test's cells
//! (the reader's outcome, the pinned door's) to `<dir>/<test>` instead.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::OnceLock;

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon_core::id::BlockId;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::schema::{self, FromJson, Schema, derive_support};
use bedrock_addon_core::shared::{EventTrigger, Filter, FloatRange, IntRange};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    BeyondWidth, MemberFailure, ParseCx, Reading, Scalars, read_payload,
};
use facet::Facet;
use indexmap::IndexMap;

#[derive(Debug, PartialEq, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
enum Unit {
    Alpha,
    Beta,
}

#[derive(Debug, PartialEq, Facet, Schema)]
#[facet(untagged)]
#[repr(u8)]
enum Untagged {
    Count(u8),
    Name(String),
    Pair { a: u8 },
}

#[derive(Debug, PartialEq, Facet, Schema)]
struct Inner {
    v: Option<u8>,
}

/// The Molang probe, by hand: the derive sees a Molang field by its type's name, which a macro
/// argument hides.
#[derive(Debug, PartialEq, Facet, Schema)]
struct OptMolang {
    v: Option<Molang>,
}

/// The format version probe, read by a hand `FromJson` over the derive's own slot (no
/// `Schema` type holds a format version, so it has no bound walk).
#[derive(Debug, Facet)]
struct OptFormatVersion {
    v: Option<FormatVersion>,
}

impl FromJson for OptFormatVersion {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let object = derive_support::Object::open(node, read)?;
        let disposal = object.disposal(read, None, false);
        let mut slot = derive_support::Slot::<FormatVersion>::new();
        for (key, member) in object.members() {
            match key {
                "v" => {
                    if !object.absent(&member, read) {
                        slot.read_option(&member, read, None);
                    }
                }
                _ => object.unknown(key, &member, read, disposal, None),
            }
        }
        let mut ok = true;
        let v = slot.optional(read, &mut ok);
        object.close(read);
        ok.then_some(Self { v })
    }
}

/// One struct per member type, `v` optional (`opt`) or required (`req`). The `Option` is
/// written here, not passed in: the derive knows an `Option` field by its spelling.
macro_rules! probes {
    ($($kind:ident $name:ident: $ty:ty;)+) => {
        $(probes!(@struct $kind $name $ty);)+
        /// Every probe of `token` under `reading`, as `(name, outcome)`.
        fn every(reading: Reading, token: &str) -> Vec<(&'static str, String)> {
            vec![
                $((stringify!($name), new::<$name>(reading, token)),)+
                ("OptMolang", new::<OptMolang>(reading, token)),
                ("OptFormatVersion", new::<OptFormatVersion>(reading, token)),
            ]
        }
    };
    (@struct opt $name:ident $ty:ty) => {
        #[derive(Debug, PartialEq, Facet, Schema)]
        struct $name {
            v: Option<$ty>,
        }
    };
    (@struct req $name:ident $ty:ty) => {
        #[derive(Debug, PartialEq, Facet, Schema)]
        struct $name {
            v: $ty,
        }
    };
}

probes! {
    opt OptU8: u8;
    req ReqU8: u8;
    opt OptI32: i32;
    opt OptF32: f32;
    opt OptF64: f64;
    opt OptBool: bool;
    opt OptString: String;
    req ReqString: String;
    opt OptVecU8: Vec<u8>;
    opt OptId: BlockId;
    opt OptFloatRange: FloatRange;
    opt OptIntRange: IntRange;
    opt OptFilter: Filter;
    opt OptEvent: EventTrigger;
    opt OptValue: Value;
    req ReqValue: Value;
    opt OptUnit: Unit;
    req ReqUnit: Unit;
    opt OptUntagged: Untagged;
    opt OptStruct: Inner;
    opt OptArray: [u8; 2];
    opt OptMap: IndexMap<String, u8>;
}

const TOKENS: &[&str] = &[
    "3",
    "\"3\"",
    "3.0",
    "0.5",
    "-1",
    "256",
    "true",
    "null",
    "1e39",
    "[]",
    "{}",
    "\"x\"",
    "[1]",
    "[1, 2]",
    "{\"a\": 1}",
    "{\"v\": 1}",
    "\"alpha\"",
    "\"true\"",
    "1e999",
    "-0.0",
    "18446744073709551616",
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
        (
            "non_strict",
            Reading::DOCUMENT.with_scalars(Scalars::NonStrict),
        ),
        ("null_absent", Reading::NULL_ABSENT),
        ("own", Reading::OWN),
        (
            "default_and_tell",
            Reading::DOCUMENT
                .with_scalars(Scalars::Strict)
                .with_on_member_failure(MemberFailure::DefaultAndTell),
        ),
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

fn parse_cx(load: &LoadContext) -> ParseCx<'_> {
    ParseCx {
        load,
        written: FormatVersionInput {
            version: Some(FormatVersion::new(1, 0, 0)),
            beta_literal: false,
            prerelease: false,
        },
        original_version: FormatVersion::new(1, 0, 0),
        at: JsonPath::of_keys(["p"]),
    }
}

/// The value (when no finding rejects the document: the pipeline drops it otherwise) and the
/// findings as `severity code path`.
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
    format!("{value} [{}]", found.join("; "))
}

fn new<T: FromJson + std::fmt::Debug>(reading: Reading, token: &str) -> String {
    let load = context(reading);
    let text = format!(r#"{{"v": {token}}}"#);
    let mut d = Diagnostics::default();
    let value: Option<T> = read_payload(JsonSlice::new(&text), &parse_cx(&load), &mut d);
    outcome(value.as_ref(), &d)
}

/// The frozen cells: key to `(read, door)`, `door` absent where it is `read`.
fn frozen() -> &'static HashMap<&'static str, (&'static str, Option<&'static str>)> {
    static FROZEN: OnceLock<HashMap<&'static str, (&'static str, Option<&'static str>)>> =
        OnceLock::new();
    FROZEN.get_or_init(|| {
        include_str!("schema_read_door.pins")
            .lines()
            .filter_map(|line| {
                let mut parts = line.split('\t');
                Some((parts.next()?, (parts.next()?, parts.next())))
            })
            .collect()
    })
}

/// The cell `key` of `test`: the reader's outcome `read` must be the pinned one; returns the
/// door's outcome as pinned. Under `BEDROCK_DOOR_PINS` the cell is written to that directory
/// instead.
fn cell(test: &str, key: &str, read: &str) -> String {
    let pinned = frozen().get(key).copied();
    let door = pinned.map_or(read, |(pinned_read, door)| door.unwrap_or(pinned_read));
    if let Ok(dir) = std::env::var("BEDROCK_DOOR_PINS") {
        use std::io::Write as _;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(std::path::Path::new(&dir).join(test))
            .expect("the pin directory");
        let line = if door == read {
            format!("{key}\t{read}\n")
        } else {
            format!("{key}\t{read}\t{door}\n")
        };
        file.write_all(line.as_bytes()).expect("a pin line");
        return door.to_owned();
    }
    let Some((pinned_read, _)) = pinned else {
        panic!("no pin for {key}");
    };
    assert_eq!(read, pinned_read, "the reader at {key}");
    door.to_owned()
}

/// The door's paths as the reader names them: a wrong type at the member whose value has it
/// (facet named the element or member inside it where it stopped: `v/0`, `v/1`, `v/a`), an
/// integer out of its member's range at the member (facet named the owner).
fn at_the_member(door: &str) -> String {
    let mut out = String::new();
    let mut rest = door;
    while let Some(at) = rest.find(" p/v/") {
        out.push_str(&rest[..at]);
        out.push_str(" p/v");
        rest = &rest[at + " p/v/".len()..];
        let end = rest.find([']', ';']).unwrap_or(rest.len());
        rest = &rest[end..];
    }
    out.push_str(rest);
    out.replace("core/invalid_value p]", "core/invalid_value p/v]")
}

/// The probes whose member the door's paths name a step inside of (an array or an object
/// given to a scalar, an identifier, a Molang value, a format version or an untagged enum of
/// scalars), or whose owner it names (an integer out of range): the ones [`at_the_member`]
/// applies to. Every other probe is held to the door's paths as they are.
const COLLAPSED: &[&str] = &[
    "OptBool",
    "OptF32",
    "OptF64",
    "OptFormatVersion",
    "OptI32",
    "OptId",
    "OptMolang",
    "OptString",
    "OptU8",
    "OptUntagged",
    "ReqString",
    "ReqU8",
];

/// Why the reader tells `read` where the door tells `door`, when it is one of the differences
/// it makes on purpose.
fn explained(probe: &str, token: &str, door: &str, read: &str) -> Option<&'static str> {
    if COLLAPSED.contains(&probe) && at_the_member(door) == read {
        return Some("the member's path");
    }
    // Under a profile other than the plain read, facet took the first arm that parses the
    // string (`Count(3)`, silently); the reader takes the first arm the profile takes.
    if probe == "OptUntagged" && token == "\"3\"" && read.contains("Name(\"3\")") {
        return Some("the untagged arm the profile takes");
    }
    None
}

#[test]
fn the_reader_tells_what_the_door_tells_but_where_it_is_pinned_otherwise() {
    let mut unexplained = String::new();
    let mut counts: Vec<(&'static str, usize)> = Vec::new();
    for (name, reading) in readings() {
        for token in TOKENS {
            for (probe, new) in every(reading, token) {
                let old = cell("grid", &format!("grid {name} {probe} {token}"), &new);
                if old == new {
                    continue;
                }
                match explained(probe, token, &old, &new) {
                    Some(rule) => match counts.iter_mut().find(|(r, _)| *r == rule) {
                        Some((_, count)) => *count += 1,
                        None => counts.push((rule, 1)),
                    },
                    None => {
                        let _ = writeln!(
                            unexplained,
                            "{name} {probe} {token}\n  door {old}\n  read {new}"
                        );
                    }
                }
            }
        }
    }
    assert!(
        unexplained.is_empty(),
        "--- unexplained ---\n{unexplained}--- end ---"
    );
    assert_eq!(
        counts,
        [
            ("the member's path", 264),
            ("the untagged arm the profile takes", 3)
        ]
    );
}

#[derive(Debug, PartialEq, Facet, Schema)]
struct Several {
    a: Option<u8>,
    b: Option<u8>,
    c: Option<f32>,
    inner: Option<Inner>,
    list: Option<Vec<Inner>>,
}

/// Documents of several members, each with more than one finding: the order of the door's
/// passes (the first failure alone; else the `null` errors alone; else the members read as
/// absent, the unknown members, the scalars read as another value).
const DOCUMENTS: &[&str] = &[
    r#"{"a": null, "zz": 1, "b": "3", "c": 1e39}"#,
    r#"{"zz": 1, "a": 1, "inner": {"yy": 2, "v": "4"}, "b": null}"#,
    r#"{"b": "x", "a": "y", "zz": 0}"#,
    r#"{"a": null, "b": true}"#,
    r#"{"list": [{"v": 1, "q": 1}, {"v": null}, {"w": 2}], "c": "2.5", "zz": null}"#,
    r#"{"inner": null, "list": null, "a": 300}"#,
    r#"{"c": 1e39, "inner": {"v": 1e39}, "list": [{"v": "1"}]}"#,
];

#[test]
fn a_document_of_several_findings_tells_them_in_the_door_s_order() {
    let mut unexplained = String::new();
    for (name, reading) in readings() {
        for text in DOCUMENTS {
            let load = context(reading);
            let mut d = Diagnostics::default();
            let value: Option<Several> =
                read_payload(JsonSlice::new(text), &parse_cx(&load), &mut d);
            let read = outcome(value.as_ref(), &d);
            let door = cell("several", &format!("several {name} {text}"), &read);
            // An integer out of range at its member (`a`), where facet named the owner.
            let at_a = door.replace("core/invalid_value p]", "core/invalid_value p/a]");
            if door != read && at_the_member(&door) != read && at_a != read {
                let _ = writeln!(unexplained, "{name} {text}\n  door {door}\n  read {read}");
            }
        }
    }
    assert!(
        unexplained.is_empty(),
        "--- unexplained ---\n{unexplained}--- end ---"
    );
}

#[test]
fn a_payload_that_is_no_object_is_the_door_s_syntax_error() {
    let mut unexplained = String::new();
    for (name, reading) in readings() {
        for text in ["5", r#""s""#, "[]", "[1, 2]", "null", "true"] {
            let load = context(reading);
            let mut d = Diagnostics::default();
            let value: Option<Several> =
                read_payload(JsonSlice::new(text), &parse_cx(&load), &mut d);
            let read = outcome(value.as_ref(), &d);
            let door = cell("no_object", &format!("no_object {name} {text}"), &read);
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

#[test]
fn a_read_may_take_another_null_handling_than_its_reading() {
    use bedrock_addon_core::versioning::{NullHandling, read_payload_with};
    let load = context(Reading::DOCUMENT);
    let text = r#"{"v": null}"#;
    let mut d = Diagnostics::default();
    let rejected: Option<OptU8> = read_payload(JsonSlice::new(text), &parse_cx(&load), &mut d);
    assert_eq!(rejected, None);
    assert_eq!(d.len(), 1);
    let mut d = Diagnostics::default();
    let own: Option<OptU8> =
        read_payload_with(JsonSlice::new(text), &parse_cx(&load), &mut d, |read| {
            read.null(NullHandling::Own)
        });
    assert_eq!(own, Some(OptU8 { v: None }));
    assert!(d.is_empty());
}

#[derive(Debug, PartialEq, Facet, Schema)]
struct Mapped {
    m: Option<IndexMap<String, u8>>,
    req: Option<Req>,
    list: Option<Vec<IndexMap<String, u8>>>,
}

#[derive(Debug, PartialEq, Facet, Schema)]
struct Req {
    m: IndexMap<String, u8>,
}

/// A map one of whose values its profile refuses: under `DefaultAndTell` the door's pre-pass
/// leaves that entry out and keeps the others (an information at it); every other reading
/// fails the map.
#[test]
fn a_map_with_a_refused_value_reads_as_the_door_reads_it() {
    let mut unexplained = String::new();
    for (name, reading) in readings() {
        for text in [
            r#"{"m": {"a": 1, "b": "x", "c": 3}}"#,
            r#"{"req": {"m": {"a": "x"}}}"#,
            r#"{"m": {"a": null}}"#,
            r#"{"list": [{"a": "x", "b": 2}, {"c": 3}]}"#,
        ] {
            let load = context(reading);
            let mut d = Diagnostics::default();
            let value: Option<Mapped> =
                read_payload(JsonSlice::new(text), &parse_cx(&load), &mut d);
            let read = outcome(value.as_ref(), &d);
            let door = cell("mapped", &format!("mapped {name} {text}"), &read);
            if door != read && at_the_member(&door) != read {
                let _ = writeln!(unexplained, "{name} {text}\n  door {door}\n  read {read}");
            }
        }
    }
    assert!(
        unexplained.is_empty(),
        "--- unexplained ---\n{unexplained}--- end ---"
    );
}

/// A format version written as a number is read as the door reads it, by the number's text:
/// `1.20` is version 1.20.0, not the 1.2.0 the double prints as.
#[test]
fn a_format_version_written_as_a_number_reads_as_the_door_reads_it() {
    let mut unexplained = String::new();
    for token in ["1.20", "1.2", "1.21", "2", "1.200", "1e2"] {
        let reading = Reading::DOCUMENT;
        let read = new::<OptFormatVersion>(reading, token);
        let door = cell("format_version", &format!("format_version {token}"), &read);
        if door != read {
            let _ = writeln!(unexplained, "{token}\n  door {door}\n  read {read}");
        }
    }
    assert!(
        unexplained.is_empty(),
        "--- unexplained ---\n{unexplained}--- end ---"
    );
}

#[derive(Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
struct Kept {
    v: Option<u8>,
    w: Option<String>,
}

#[derive(Debug, PartialEq, Facet, Schema)]
#[facet(tag = "type")]
#[repr(u8)]
enum Shape {
    #[facet(rename = "dot")]
    Dot,
    #[facet(rename = "box")]
    Box { size: u8 },
}

#[derive(Debug, PartialEq, Facet, Schema)]
#[repr(u8)]
enum Either {
    #[facet(rename = "none")]
    Nothing,
    #[facet(rename = "count")]
    Count(u8),
}

/// A document of every kind of member the derived read has: nested and kept structs, a list of
/// them, a map of them, a box, a tagged and an externally tagged enum, a `skip` field,
/// `default = x` and `rename_all`. (A tuple has a read and no check: no `Schema` type holds one
/// yet. A flattened struct makes facet's read buffer the object and fail at members it reads
/// whole: `a_flattened_struct_reads_its_members_from_its_owner_s_object` pins the reader's.)
#[derive(Debug, PartialEq, Facet, Schema)]
#[facet(rename_all = "camelCase")]
struct Everything {
    inner_one: Option<Inner>,
    kept: Option<Kept>,
    list: Option<Vec<Kept>>,
    map: Option<IndexMap<String, Kept>>,
    boxed: Option<Box<Kept>>,
    shape: Option<Shape>,
    either: Option<Either>,
    #[facet(default = 7)]
    level: u8,
    #[facet(skip)]
    skipped: u8,
}

const EVERYTHING: &[&str] = &[
    r#"{"innerOne": {"v": 1, "zz": 2}, "kept": {"v": 2, "zz": 3}}"#,
    r#"{"list": [{"v": 1, "a": 1}, {"w": "x", "b": 2}], "map": {"k": {"v": 3, "c": 3}}}"#,
    r#"{"list": [{"v": "x", "a": 1}, {"v": 2}], "level": "high"}"#,
    r#"{"boxed": {"v": 1, "d": 4}, "skipped": 1}"#,
    r#"{"shape": {"type": "box", "size": 2}, "either": {"count": 3}}"#,
    r#"{"shape": {"type": "box"}, "either": "none", "kept": {"zz": null}}"#,
    r#"{"shape": {"type": "ring"}, "either": {"count": 3, "x": 1}}"#,
    r#"{"shape": {"size": 1}, "either": "other"}"#,
    r#"{"shape": {"type": 5}, "inner_one": {}, "level": null}"#,
    r#"{"map": {"k": {"v": 300}}, "kept": {"v": null, "w": 5}}"#,
];

/// [`Everything`] read by both readers under every reading, the table of kept members
/// compared too.
#[test]
fn every_kind_of_member_reads_as_the_door_reads_it() {
    let mut unexplained = String::new();
    for (name, reading) in readings() {
        for text in EVERYTHING {
            let load = context(reading);
            let kept = |d: &Diagnostics| -> String {
                let kept: Vec<String> = d
                    .unknown()
                    .iter()
                    .map(|(at, key, value)| format!("{at}/{key}={value:?}"))
                    .collect();
                kept.join("; ")
            };
            let mut d = Diagnostics::default();
            let value: Option<Everything> =
                read_payload(JsonSlice::new(text), &parse_cx(&load), &mut d);
            let read = format!("{} kept [{}]", outcome(value.as_ref(), &d), kept(&d));
            let door = cell("everything", &format!("everything {name} {text}"), &read);
            // An integer out of range at its member, where facet named the document; a `skip`
            // field is not read, where facet read it from the member of its name.
            let door = door
                .replace("core/invalid_value p]", "core/invalid_value p/map/k/v]")
                .replace(", skipped: 1 }", ", skipped: 0 }");
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

#[derive(Debug, PartialEq, Facet, Schema)]
struct Probe {
    shape: Option<Shape>,
    shapes: Option<Vec<Shape>>,
    either: Option<Either>,
    eithers: Option<Vec<Either>>,
    outer: Option<Outer>,
}

#[derive(Debug, PartialEq, Facet, Schema)]
#[facet(tag = "t")]
#[repr(u8)]
enum Outer {
    #[facet(rename = "a")]
    A {
        shape: Option<Shape>,
        either: Option<Either>,
        shapes: Option<Vec<Shape>>,
    },
}

#[test]
fn probe_tags() {
    for text in [
        r#"{"shape": {"type":"ring"}}"#,
        r#"{"shapes": [{"type":"ring"}]}"#,
        r#"{"either": 5}"#,
        r#"{"either": null}"#,
        r#"{"either": true}"#,
        r#"{"either": "zz"}"#,
        r#"{"either": {"zz": 1}}"#,
        r#"{"either": []}"#,
        r#"{"eithers": [5]}"#,
        r#"{"outer": {"t": "a", "shape": {"type":"ring"}}}"#,
        r#"{"outer": {"t": "a", "shapes": [{"type":"ring"}]}}"#,
        r#"{"outer": {"t": "a", "either": 5}}"#,
        r#"{"outer": {"t": "a", "either": null}}"#,
        r#"{"outer": {"t": "a", "either": "zz"}}"#,
        r#"{"outer": {"t": "zz"}}"#,
    ] {
        for (name, reading) in [
            ("doc", Reading::DOCUMENT),
            ("null_absent", Reading::NULL_ABSENT),
        ] {
            let load = context(reading);
            let mut d = Diagnostics::default();
            let v: Option<Probe> = read_payload(JsonSlice::new(text), &parse_cx(&load), &mut d);
            let read = outcome(v.as_ref(), &d);
            eprintln!("{name} {text}\n  read {read}");
        }
    }
}
