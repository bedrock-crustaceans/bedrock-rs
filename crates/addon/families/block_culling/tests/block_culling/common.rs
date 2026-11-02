//! Helpers of the block-culling tests.

#![allow(dead_code)]

use bedrock_addon_core::diagnostic::{Diagnostics, Severity};
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_block_culling::CullingRules;

/// The fixture folder of this family.
pub const FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../addon/tests/fixtures/block/block_culling"
);

/// The path every test file is loaded under.
pub const PATH: &str = "block_culling/test.json";

/// The text of the fixture file `name`.
pub fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!("{FIXTURES}/{name}"))
        .unwrap_or_else(|error| panic!("{name}: {error}"))
}

/// The context of a pack that is not the base game's.
pub fn user_pack() -> LoadContext {
    LoadContext::detached()
}

/// The context of a base-game pack.
pub fn base_game() -> LoadContext {
    let mut cx = LoadContext::detached();
    cx.is_base_game_pack = true;
    cx
}

/// Loads `json` as [`PATH`] of a pack with `cx`.
pub fn load_in(json: &str, cx: &LoadContext) -> Result<Document<CullingRules>, Rejected> {
    CullingRules::parse(&PackPath::from_static(PATH), json.as_bytes(), cx)
}

/// Loads `json` from a user pack; it must load without any finding.
#[track_caller]
pub fn loads(json: &str) -> Document<CullingRules> {
    match load_in(json, &user_pack()) {
        Ok(document) => {
            assert!(
                document.diagnostics().is_empty(),
                "{json}\n{:#?}",
                document.diagnostics()
            );
            document
        }
        Err(rejected) => panic!("rejected: {json}\n{:#?}", rejected.diagnostics),
    }
}

/// Loads `json` from a pack with `cx`; it must be rejected. Its findings.
#[track_caller]
pub fn rejects_in(json: &str, cx: &LoadContext) -> Diagnostics {
    match load_in(json, cx) {
        Ok(document) => panic!("loaded: {json}\n{:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// Loads `json` from a user pack; it must be rejected. Its findings.
#[track_caller]
pub fn rejects(json: &str) -> Diagnostics {
    rejects_in(json, &user_pack())
}

/// `(code name, text, JSON path)` of every finding, all of which must be errors located in
/// [`PATH`].
#[track_caller]
pub fn errors(d: &Diagnostics) -> Vec<(&'static str, String, String)> {
    d.iter()
        .map(|f| {
            assert_eq!(f.severity, Severity::Error, "{f:#?}");
            assert_eq!(f.location.path.as_ref().map(PackPath::as_str), Some(PATH));
            (
                f.code.name(),
                f.code.to_string(),
                f.location.json_path.to_string(),
            )
        })
        .collect()
}

/// One expected finding of [`errors`].
pub fn error(code: &'static str, text: &str, path: &str) -> (&'static str, String, String) {
    (code, text.to_owned(), path.to_owned())
}

/// The payload key.
pub const KEY: &str = "minecraft:block_culling_rules";

/// "Missing object", as [`errors`] lists it for a finding with `code` at `path`.
pub fn missing_object(code: &'static str, path: &str) -> (&'static str, String, String) {
    error(
        code,
        "Failed to parse block culling file 'block_culling/test.json'. Missing object \"minecraft:block_culling_rules\"",
        path,
    )
}

/// The closing line after a version without a parser or a strict parse error.
pub fn failed_to_parse() -> (&'static str, String, String) {
    error(
        "DocumentRejected",
        "Failed to parse block culling file 'block_culling/test.json'.",
        "",
    )
}

/// The closing line after a failed identifier or rule check.
pub fn failed_internal() -> (&'static str, String, String) {
    error(
        "DocumentRejected",
        "Failed to parse block culling file 'block_culling/test.json' into internal format",
        "",
    )
}

/// A document at `version` naming `identifier` with `rules` (the elements of the array).
pub fn document(version: &str, identifier: &str, rules: &[&str]) -> String {
    format!(
        r#"{{"format_version": "{version}", "minecraft:block_culling_rules": {{"description": {{"identifier": "{identifier}"}}, "rules": [{}]}}}}"#,
        rules.join(", ")
    )
}

/// A 1.21.80 document `test:culling_parsing` with `rules`.
pub fn latest(rules: &[&str]) -> String {
    document("1.21.80", "test:culling_parsing", rules)
}

/// A 1.20.60 document `test:culling_parsing` with `rules`.
pub fn floor(rules: &[&str]) -> String {
    document("1.20.60", "test:culling_parsing", rules)
}

/// Rule 0 of vanilla's `PuVData` tests: `{bone1, cube 2, face east}` against north.
pub const BONE1_EAST: &str =
    r#"{"geometry_part": {"bone": "bone1", "cube": 2, "face": "east"}, "direction": "north"}"#;
