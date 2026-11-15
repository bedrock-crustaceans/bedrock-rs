//! The derived read under the game's schema node tree (`Profile::TREE`, `Texts::TREE`): one
//! schema of every kind of member declared as `Schema` types, read from a spread of documents
//! under each of the tree's rule sets (the animation loaders', the render controller's,
//! geometry's strict and lenient ones). The findings (code, text, severity, path, span, in
//! order), the respelled members and the value are pinned in `schema_read_tree.pins`: what the
//! node-tree walker the read replaced (`json::node_tree`) told and read for each, the same in
//! every line when the walker was deleted.

use std::fmt::Write as _;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Severity, Span};
use bedrock_addon_core::json::kit::{Lossy as KitLossy, Names as KitNames, Node, Profile, Read};
use bedrock_addon_core::json::kit::{MemberOrder, Recovery, Texts};
use bedrock_addon_core::json::parse::{Options, parse};
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::schema::{self, KeyPattern, Keys, Schema, read_document};
use bedrock_addon_core::versioning::{BeyondWidth, MemberFailure, NullHandling, Reading, Scalars};
use facet::Facet;
use indexmap::IndexMap;

const ANIMATIONS: Keys = Keys::new(KeyPattern::Prefixed {
    prefix: "animation.",
    extra: b"_.-",
    regex: r"animation\.[a-zA-Z0-9_.-]+",
})
.required();
const BONES: Keys = Keys::new(KeyPattern::IDENTIFIER).required();

fn is_time(key: &str) -> bool {
    !key.is_empty() && key.bytes().all(|b| b.is_ascii_digit() || b == b'.')
}
const TIMES: Keys = Keys::new(KeyPattern::Custom {
    matches: is_time,
    regex: "[0-9.]+",
});

#[derive(Debug, PartialEq, Facet, Schema)]
struct Root {
    format_version: String,
    #[facet(schema::keys = ANIMATIONS)]
    animations: IndexMap<String, Anim>,
}

#[derive(Debug, PartialEq, Facet, Schema)]
struct Anim {
    r#loop: Option<bool>,
    length: Option<f32>,
    blend: Option<Molang>,
    #[facet(schema::keys = BONES)]
    bones: Option<IndexMap<String, Bone>>,
    items: Option<Vec<Item>>,
    tag: Option<String>,
}

#[derive(Debug, Default, PartialEq, Facet, Schema)]
struct Bone {
    name: String,
    scale: Option<f32>,
    #[facet(schema::keys = TIMES)]
    frames: Option<IndexMap<String, f32>>,
}

#[derive(Debug, Default, PartialEq, Facet, Schema)]
struct Item {
    a: f32,
    b: Option<bool>,
}

// ---- the rule sets -------------------------------------------------------------------------------

const REMOVED: &str = "child '{}' not valid here (removed)";

/// One of the tree's rule sets: the derived read's profile, reading and texts for it.
struct Config {
    name: &'static str,
    profile: Profile,
    reading: Reading,
    texts: Texts,
    unknown: Severity,
}

fn configs() -> Vec<Config> {
    let strict = Reading::OWN
        .with_null(NullHandling::AsValue)
        .with_scalars(Scalars::Strict)
        .with_beyond_width(BeyondWidth::Finite);
    vec![
        Config {
            name: "animation",
            profile: Profile::TREE.with_names(KitNames::IgnoreAsciiCase),
            reading: strict,
            texts: Texts::TREE,
            unknown: Severity::Error,
        },
        Config {
            name: "render controller",
            profile: Profile::TREE
                .with_members(MemberOrder::Sorted)
                .with_lossy(KitLossy::Told),
            reading: strict,
            texts: Texts::TREE,
            unknown: Severity::Error,
        },
        Config {
            name: "geometry",
            profile: Profile::TREE.with_lossy(KitLossy::Told),
            reading: strict,
            texts: Texts::TREE,
            unknown: Severity::Error,
        },
        Config {
            name: "lenient geometry",
            profile: Profile::TREE
                .with_lossy(KitLossy::Told)
                .with_recovery(Recovery::Value),
            reading: strict.with_on_member_failure(MemberFailure::DefaultAndTell),
            texts: Texts {
                unknown_named: None,
                unknown: Some(REMOVED),
                ..Texts::TREE
            },
            unknown: Severity::Info,
        },
    ]
}

/// The documents: every kind of member right, wrong, missing, unknown, respelled, doubled.
const DOCUMENTS: &[&str] = &[
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {"loop": true, "length": 2.5, "blend": "q.x", "tag": "t"}}}"#,
    r#"{"format_version": "1.8.0", "animations": {}}"#,
    r#"{"animations": {"animation.a": {}}}"#,
    "{}",
    "[]",
    r#"{"format_version": 1, "animations": {"animation.a": {}}}"#,
    r#"{"format_version": "1.8.0", "animations": []}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation:a": {}, "animation.b": {}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {"loops": true}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {"loop": 1, "length": "x", "blend": [], "tag": 3}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {"loop": null, "length": null, "blend": null, "tag": null}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {"length": 0.123456789012, "blend": 1e39}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {"length": 1e39}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {"LOOP": true, "Length": 1}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {"loop": true, "Loop": false}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {"loop": true, "LOOP": true}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"Animation.A": {}, "ANIMATION.b": {"tag": "x"}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {"bones": {}}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {"bones": {"left arm": {"name": "x"}, "arm": {"name": "y"}}}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {"bones": {"arm": {}}}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {"bones": {"arm": {"name": 1, "scale": true}}}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {"bones": {"arm": {"name": "a", "frames": {"0": 1, "0.5": "x", "x": 2}}}}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {"bones": {"arm": {"name": "a", "frames": {"0": 1, "1": 2.5}}}}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {"bones": []}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {"items": [{"a": 1}, {"a": 2, "b": false}]}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {"items": [{"a": 1}, {"b": true}, 3, {"a": "x", "c": 1}]}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {"items": {}}}}"#,
    r#"{"zeta": 1, "format_version": "1.8.0", "animations": {"animation.b": {"tag": 1}, "animation.a": {"tag": 2}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {"tag": "x", "blend": 0.5, "zz": 1, "aa": 2, "loop": "no"}}}"#,
    r#"{"format_version": "1.8.0", "extra": [], "animations": {"animation.a": {"bones": {"arm": {"name": "a", "x": 1}}}}}"#,
    r#"{"format_version": "1.8.0", "animations": {"animation.a": {"bones": {"arm": {"Name": "a", "NAME": "b"}}}}}"#,
];

/// A finding as the pins have it.
fn line(finding: &Diagnostic) -> String {
    let span = finding.location.span.map_or_else(
        || "-".to_owned(),
        |Span { offset, len }| format!("{offset}+{len}"),
    );
    format!(
        "{:?} {} \"{}\" at {} {span}",
        finding.severity,
        finding.code.id(),
        finding.code,
        finding.location.json_path
    )
}

fn lines(findings: &Diagnostics) -> Vec<String> {
    findings.iter().map(line).collect()
}

/// What one reader made of a document: the value, the findings, the respelled members.
#[derive(Debug, PartialEq)]
struct Outcome {
    value: Option<Root>,
    findings: Vec<String>,
    respelled: Vec<(String, Option<Span>)>,
}

fn derived(config: &Config, text: &str) -> Outcome {
    let parsed = parse(text, Options::default()).unwrap();
    let node = Node::parsed(&parsed, JsonPath::new());
    let mut found = Diagnostics::default();
    let mut read = Read::new(config.reading, &mut found)
        .profile(config.profile)
        .texts(config.texts)
        .unknown_severity(config.unknown);
    let value = read_document::<Root>(&node, &mut read);
    let respelled = read
        .take_respelled()
        .into_iter()
        .map(|(path, span)| (path.to_string(), span))
        .collect();
    drop(read);
    Outcome {
        value,
        findings: lines(&found),
        respelled,
    }
}

/// The pinned outcomes, one block per rule set and document.
fn render(outcome: &Outcome, name: &str, text: &str) -> String {
    let mut out = format!("== {name} {text}\n");
    for line in &outcome.findings {
        out.push_str(line);
        out.push('\n');
    }
    for (path, span) in &outcome.respelled {
        writeln!(out, "respelled {path} {span:?}").expect("a String takes it");
    }
    writeln!(out, "{:?}", outcome.value).expect("a String takes it");
    out
}

#[test]
fn the_derived_read_of_a_tree_tells_what_the_walker_told() {
    let mut out = String::new();
    for config in configs() {
        for text in DOCUMENTS {
            out.push_str(&render(&derived(&config, text), config.name, text));
        }
    }
    let pinned = include_str!("schema_read_tree.pins");
    for (line, (got, want)) in out.lines().zip(pinned.lines()).enumerate() {
        assert_eq!(got, want, "line {}", line + 1);
    }
    assert_eq!(out.lines().count(), pinned.lines().count());
}

#[test]
fn the_documents_reach_every_kind_of_finding() {
    // The spread is worth its name only if it tells what the tree tells.
    let config = &configs()[0];
    let ids: Vec<String> = DOCUMENTS
        .iter()
        .flat_map(|text| derived(config, text).findings)
        .collect();
    for id in [
        "core/required_child",
        "core/child_not_valid",
        "core/unknown_child_option",
        "core/dropped_field",
        "core/invalid_value",
    ] {
        assert!(ids.iter().any(|line| line.contains(id)), "{id}");
    }
}
