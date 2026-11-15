//! The new reader (`versioning::read_payload`, the derived `FromJson`) against today's door
//! (`versioning::parse_payload`, facet-json's typed read and its passes): one struct per member
//! type, every token of a grid, under every reading. Where the two tell the same codes,
//! severities and paths (spans aside: the new reader spans every finding's value) and read the
//! same value, nothing is said; every difference is a line of `DIFFERENCES`, the pinned list of
//! what the new reader does otherwise and why.

use std::fmt::Write as _;

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon_core::id::BlockId;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::schema::{__read, FromJson, Schema};
use bedrock_addon_core::shared::{EventTrigger, Filter, FloatRange, IntRange};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    BeyondWidth, MemberFailure, ParseCx, Reading, Scalars, parse_payload, read_payload,
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
        let object = __read::Object::open(node, read)?;
        let disposal = object.disposal(read, None, false);
        let mut slot = __read::Slot::<FormatVersion>::new();
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
        /// Every probe of `token` under `reading`, as `(name, old, new)`.
        fn every(reading: Reading, token: &str) -> Vec<(&'static str, String, String)> {
            vec![
                $((stringify!($name), old::<$name>(reading, token), new::<$name>(reading, token)),)+
                ("OptMolang", old::<OptMolang>(reading, token), new::<OptMolang>(reading, token)),
                (
                    "OptFormatVersion",
                    old::<OptFormatVersion>(reading, token),
                    new::<OptFormatVersion>(reading, token),
                ),
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

fn readings() -> [(&'static str, Reading); 7] {
    [
        ("document", Reading::DOCUMENT),
        (
            "strict",
            Reading::DOCUMENT
                .with_scalars(Scalars::Strict)
                .with_beyond_width(BeyondWidth::Error),
        ),
        ("jsoncpp", Reading::DOCUMENT.with_scalars(Scalars::JsonCpp)),
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

fn old<T: for<'a> Facet<'a> + std::fmt::Debug>(reading: Reading, token: &str) -> String {
    let load = context(reading);
    let text = format!(r#"{{"v": {token}}}"#);
    let mut d = Diagnostics::default();
    let value: Option<T> = parse_payload(JsonSlice::new(&text), &parse_cx(&load), &mut d);
    outcome(value.as_ref(), &d)
}

fn new<T: FromJson + std::fmt::Debug>(reading: Reading, token: &str) -> String {
    let load = context(reading);
    let text = format!(r#"{{"v": {token}}}"#);
    let mut d = Diagnostics::default();
    let value: Option<T> = read_payload(JsonSlice::new(&text), &parse_cx(&load), &mut d);
    outcome(value.as_ref(), &d)
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

/// Why the reader tells `read` where the door tells `door`, when it is one of the differences
/// it makes on purpose.
fn explained(probe: &str, token: &str, door: &str, read: &str) -> Option<&'static str> {
    if at_the_member(door) == read {
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
            for (probe, old, new) in every(reading, token) {
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
            ("the member's path", 317),
            ("the untagged arm the profile takes", 4)
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
                parse_payload(JsonSlice::new(text), &parse_cx(&load), &mut d);
            let door = outcome(value.as_ref(), &d);
            let mut d = Diagnostics::default();
            let value: Option<Several> =
                read_payload(JsonSlice::new(text), &parse_cx(&load), &mut d);
            let read = outcome(value.as_ref(), &d);
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
                parse_payload(JsonSlice::new(text), &parse_cx(&load), &mut d);
            let door = outcome(value.as_ref(), &d);
            let mut d = Diagnostics::default();
            let value: Option<Several> =
                read_payload(JsonSlice::new(text), &parse_cx(&load), &mut d);
            let read = outcome(value.as_ref(), &d);
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
