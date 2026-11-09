//! Helpers of the aim-assist tests.

use bedrock_addon::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon::document::{Content, Document, Rejected};
use bedrock_addon::families::aim_assist::{AimAssistCategories, AimAssistPreset};
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::pack::{Behavior, PackStack};
use bedrock_addon::storage::PackPath;

use super::pack_common::bp;

/// The fixture folder of this family.
pub const FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/camera/aim_assist"
);

/// The text of the fixture file `path` (relative to the family's fixture folder).
pub fn fixture(path: &str) -> String {
    std::fs::read_to_string(format!("{FIXTURES}/{path}"))
        .unwrap_or_else(|error| panic!("{path}: {error}"))
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

/// The document of an outcome that must have loaded.
#[track_caller]
pub fn loads<T: std::fmt::Debug>(outcome: Result<Document<T>, Rejected>) -> Document<T> {
    match outcome {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {:#?}", rejected.diagnostics),
    }
}

/// The findings of an outcome that must have been rejected.
#[track_caller]
pub fn rejects<T: std::fmt::Debug>(outcome: Result<Document<T>, Rejected>) -> Diagnostics {
    match outcome {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
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

/// A text that lives as long as the test binary (the pack helper takes `'static` files).
pub fn leak(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

/// A behaviour stack of packs, top first; each pack is its `(path, text)` files.
pub fn stack(packs: &[&[(&'static str, String)]]) -> PackStack<Behavior> {
    let packs: Vec<_> = packs
        .iter()
        .enumerate()
        .map(|(index, files)| {
            let mut spec = bp(index as u128 + 1);
            for (path, text) in *files {
                spec = spec.file(path, leak(text.clone()).as_bytes());
            }
            spec.open()
        })
        .collect();
    PackStack::<Behavior>::of(packs, LoadOptions::default()).unwrap()
}
