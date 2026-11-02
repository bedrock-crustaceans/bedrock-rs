//! Helpers of the feature tests.

#![allow(dead_code)]

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_family_feature::Feature;

/// A third-party behaviour pack at the engine version.
pub fn cx() -> LoadContext {
    LoadContext::detached()
}

/// A third-party behaviour pack on an engine of `version`.
pub fn engine(major: u16, minor: u16, patch: u16) -> LoadContext {
    let mut cx = LoadContext::detached();
    cx.engine_version = FormatVersion::new(major, minor, patch);
    cx
}

/// A base-game behaviour pack.
pub fn base_game() -> LoadContext {
    let mut cx = LoadContext::detached();
    cx.is_base_game_pack = true;
    cx
}

/// A fixture file under `tests/fixtures/worldgen/feature/` of bedrock_addon. This file is built
/// into the family crate's tests and into bedrock_addon's, so it looks in both places.
pub fn fixture(name: &str) -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = [".", "../../addon"]
        .iter()
        .map(|base| {
            root.join(base)
                .join("tests/fixtures/worldgen/feature")
                .join(name)
        })
        .find(|path| path.exists())
        .unwrap_or_else(|| panic!("{name}: no such fixture"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Loads `json` as a feature file.
pub fn load(json: &str, cx: &LoadContext) -> Result<Document<Feature>, Rejected> {
    Feature::parse(
        &PackPath::from_static("features/test.json"),
        json.as_bytes(),
        cx,
    )
}

/// Loads `json`, which must load.
#[track_caller]
pub fn loads(json: &str, cx: &LoadContext) -> Document<Feature> {
    match load(json, cx) {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {:#?}", rejected.diagnostics),
    }
}

/// Loads `json`, which must be rejected; its findings.
#[track_caller]
pub fn rejects(json: &str, cx: &LoadContext) -> Diagnostics {
    match load(json, cx) {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// A document of `version` with the payload `key` holding `members` (a JSON object's members
/// after the description of `demo:x`).
pub fn doc(version: &str, key: &str, members: &str) -> String {
    let sep = if members.trim().is_empty() { "" } else { ", " };
    format!(
        r#"{{"format_version": "{version}", "minecraft:{key}": {{"description": {{"identifier": "demo:x"}}{sep}{members}}}}}"#
    )
}

/// The findings whose code is `name` at `severity`.
pub fn with<'a>(d: &'a Diagnostics, name: &str, severity: Severity) -> Vec<&'a Diagnostic> {
    d.iter()
        .filter(|f| f.code.name() == name && f.severity == severity)
        .collect()
}

/// The single error of `d` with code `name`, at a path ending in `tail`.
#[track_caller]
pub fn error_at(d: &Diagnostics, name: &str, tail: &str) {
    let found = with(d, name, Severity::Error);
    assert!(
        found
            .iter()
            .any(|f| f.location.json_path.to_string().ends_with(tail)),
        "no {name} error at …{tail}: {d:#?}"
    );
}

/// A kept error of code `name` at a path ending in `tail`: a level-3 line of the game after
/// which the feature loads.
#[track_caller]
pub fn kept_at(d: &Diagnostics, name: &str, tail: &str) {
    let found = with(d, name, Severity::Error);
    assert!(
        found
            .iter()
            .any(|f| f.kept && f.location.json_path.to_string().ends_with(tail)),
        "no {name} kept error at …{tail}: {d:#?}"
    );
}

/// Whether some finding's message is exactly `text`.
pub fn has_text(d: &Diagnostics, text: &str) -> bool {
    d.iter().any(|f| f.code.to_string() == text)
}

/// The findings above info.
pub fn above_info(d: &Diagnostics) -> Vec<&Diagnostic> {
    d.iter().filter(|f| f.severity > Severity::Info).collect()
}
