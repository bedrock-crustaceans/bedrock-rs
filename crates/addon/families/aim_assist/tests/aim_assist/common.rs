//! Helpers of the aim-assist tests.

#![allow(
    dead_code,
    reason = "bedrock_addon's tests/aim_assist.rs includes this file too, and its stack tests leave fixtures, fixture, preset and categories unused"
)]

use bedrock_addon_core::diagnostic::{Diagnostic, Severity};
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_aim_assist::{AimAssistCategories, AimAssistPreset};

#[allow(unused_imports, reason = "each test file uses part of the helpers")]
pub use bedrock_addon_test_support::{loaded as loads, rejected as rejects};

/// The fixture folder of this family.
pub fn fixtures() -> std::path::PathBuf {
    bedrock_addon_test_support::fixture::root(env!("CARGO_MANIFEST_DIR")).join("camera/aim_assist")
}

/// The text of the fixture file `path` (relative to the family's fixture folder).
pub fn fixture(path: &str) -> String {
    std::fs::read_to_string(fixtures().join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// Loads `json` as a preset file.
pub fn preset(json: &str) -> Result<Document<AimAssistPreset>, Rejected> {
    AimAssistPreset::parse(
        &PackPath::from_static("aim_assist/presets/test.json"),
        json.as_bytes(),
        &LoadContext::detached(),
    )
}

/// Loads `json` as a categories file.
pub fn categories(json: &str) -> Result<Document<AimAssistCategories>, Rejected> {
    AimAssistCategories::parse(
        &PackPath::from_static("aim_assist/categories/test.json"),
        json.as_bytes(),
        &LoadContext::detached(),
    )
}

/// `(code name, JSON path, message)` of each error.
pub fn errors<'a>(
    d: impl IntoIterator<Item = &'a Diagnostic>,
) -> Vec<(&'static str, String, String)> {
    d.into_iter()
        .filter(|f| f.severity == Severity::Error)
        .map(|f| {
            (
                f.code.id(),
                f.location.json_path.to_string(),
                f.code.to_string(),
            )
        })
        .collect()
}

/// One expected finding.
pub fn finding(code: &'static str, path: &str, message: &str) -> (&'static str, String, String) {
    (code, path.to_owned(), message.to_owned())
}

/// A preset document at `version` with `members` (a JSON object's members).
pub fn preset_json(version: &str, members: &str) -> String {
    format!(r#"{{"format_version": "{version}", "minecraft:aim_assist_preset": {{{members}}}}}"#)
}

/// A categories document at `version` with `categories` (the elements of the JSON array).
pub fn categories_json(version: &str, categories: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "minecraft:aim_assist_categories": {{"categories": [{categories}]}}}}"#
    )
}

/// One category named `name` with `priorities` (a JSON object's members).
pub fn category(name: &str, priorities: &str) -> String {
    format!(r#"{{"name": "{name}", "priorities": {{{priorities}}}}}"#)
}
