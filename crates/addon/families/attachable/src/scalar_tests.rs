//! Every scalar member of an attachable's description, at each format version, read from every
//! JSON type: each line pins what the load says (code, severity, path, span) and, when the
//! document loads, the document written back. The table is what the members read before they
//! became plain scalars, and the oracle of that change.

use std::fmt::Write as _;

use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;

use super::Attachable;

/// The payload key the documents are written under.
const PAYLOAD: &str = "minecraft:attachable";

/// Loads `text` as an attachable file.
fn load(text: &str) -> Result<Document<Attachable>, Rejected> {
    Attachable::parse(
        &PackPath::from_static("attachables/wand.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
}

/// The values each member is read from.
const VALUES: &[&str] = &[
    "\"x\"",
    "\"5\"",
    "5",
    "0.5",
    "-1",
    "true",
    "null",
    "1e39",
    "18446744073709551616",
    "[]",
    "{}",
];

/// The format versions the table reads each place at.
const VERSIONS: &[&str] = &["1.8.0", "1.10.0", "1.26.0"];

/// The members of `description` the table reads, `@@` standing for the value.
const PLACES: &[(&str, &str)] = &[
    ("identifier", r#""identifier": @@"#),
    ("materials.default", r#""materials": {"default": @@}"#),
    ("queryable_geometry", r#""queryable_geometry": @@"#),
    ("spawn_egg.texture", r#""spawn_egg": {"texture": @@}"#),
    (
        "spawn_egg.texture_index",
        r#""spawn_egg": {"texture_index": @@}"#,
    ),
    ("spawn_egg.base_color", r#""spawn_egg": {"base_color": @@}"#),
    (
        "spawn_egg.overlay_color",
        r#""spawn_egg": {"overlay_color": @@}"#,
    ),
    (
        "scripts.variables",
        r#""scripts": {"variables": {"variable.x": @@}}"#,
    ),
    (
        "scripts.initialize[0]",
        r#""scripts": {"initialize": [@@]}"#,
    ),
    (
        "scripts.pre_animation[0]",
        r#""scripts": {"pre_animation": [@@]}"#,
    ),
    ("particle_emitters.x", r#""particle_emitters": {"x": @@}"#),
    (
        "animation_controllers[0].x",
        r#""animation_controllers": [{"x": @@}]"#,
    ),
    ("render_controllers[0]", r#""render_controllers": [@@]"#),
    ("enable_attachables", r#""enable_attachables": @@"#),
    (
        "held_item_ignores_lighting",
        r#""held_item_ignores_lighting": @@"#,
    ),
    ("hide_armor", r#""hide_armor": @@"#),
    ("held_item_scale", r#""held_item_scale": @@"#),
];

/// The document of `version` whose description holds `member` (and an identifier, unless
/// `member` is the identifier).
fn document(version: &str, member: &str) -> String {
    let identifier = if member.starts_with(r#""identifier""#) {
        String::new()
    } else {
        r#""identifier": "demo:wand", "#.to_owned()
    };
    format!(
        r#"{{"format_version": "{version}", "{PAYLOAD}": {{"description": {{{identifier}{member}}}}}}}"#
    )
}

/// `text` on one line: each line trimmed, the lines joined.
fn compact(text: &str) -> String {
    text.lines().map(str::trim).collect()
}

/// One line of the table: what loading `text` gave.
fn line(text: &str) -> String {
    let (loaded, diagnostics) = match load(text) {
        Ok(document) => (
            format!(
                "loaded {}",
                compact(&document.to_json().unwrap_or_default())
            ),
            document.diagnostics().clone(),
        ),
        Err(rejected) => ("rejected".to_owned(), rejected.diagnostics),
    };
    let findings: Vec<String> = diagnostics
        .iter()
        .map(|d| {
            let span = d
                .location
                .span
                .map_or("-".to_owned(), |s| format!("{}+{}", s.offset, s.len));
            format!(
                "{} {:?} {} {span}",
                d.code.id(),
                d.severity,
                d.location.json_path
            )
        })
        .collect();
    format!("{loaded} [{}]", findings.join("; "))
}

/// The whole table, one line per version, place and value.
pub(crate) fn table() -> String {
    let mut table = String::new();
    for version in VERSIONS {
        for (name, place) in PLACES {
            for value in VALUES {
                let text = document(version, &place.replace("@@", value));
                let _ = writeln!(table, "{version} {name} {value} => {}", line(&text));
            }
        }
    }
    table
}

#[test]
fn every_scalar_member_reads_as_pinned() {
    let table = table();
    let expected = include_str!("scalar_tests.expected");
    assert!(table == expected, "--- actual ---\n{table}--- end ---");
}
