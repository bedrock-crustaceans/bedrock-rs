//! Helpers of the deferred-settings tests.

#![allow(dead_code)]

use bedrock_addon::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon::document::{Content, Document, Rejected};
use bedrock_addon::json::Value;
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::pack::{PackStack, Resource};
use bedrock_addon::storage::PackPath;

use super::pack_common::rp;

/// The fixture folder of this family.
pub const FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/render/render_configs"
);

/// The text of the fixture file `path` (relative to the family's fixture folder).
pub fn fixture(path: &str) -> String {
    std::fs::read_to_string(format!("{FIXTURES}/{path}"))
        .unwrap_or_else(|error| panic!("{path}: {error}"))
}

pub const LIGHTING: &str = "minecraft:lighting_settings";
pub const WATER: &str = "minecraft:water_settings";
pub const ATMOSPHERE: &str = "minecraft:atmosphere_settings";
pub const GRADING: &str = "minecraft:color_grading_settings";
pub const CUBEMAP: &str = "minecraft:cubemap_settings";
pub const SHADOWS: &str = "minecraft:shadow_settings";
pub const PBR: &str = "minecraft:pbr_fallback_settings";
pub const LOCAL: &str = "minecraft:local_light_settings";

/// A document at `version` whose payload `key` has `members` (a JSON object's members).
pub fn doc(version: &str, key: &str, members: &str) -> String {
    format!(r#"{{"format_version": "{version}", "{key}": {{{members}}}}}"#)
}

/// `members` after a `description` naming `identifier`.
pub fn named(identifier: &str, members: &str) -> String {
    let comma = if members.is_empty() { "" } else { ", " };
    format!(r#""description": {{"identifier": "{identifier}"}}{comma}{members}"#)
}

/// Loads `json` as content `T`.
pub fn parse<T: Content>(json: &str) -> Result<Document<T>, Rejected> {
    parse_with(json, &LoadContext::detached())
}

/// Loads `json` as content `T` with `cx`.
pub fn parse_with<T: Content>(json: &str, cx: &LoadContext) -> Result<Document<T>, Rejected> {
    T::parse(&PackPath::from_static("test.json"), json.as_bytes(), cx)
}

/// The document of `json`, which must load.
#[track_caller]
pub fn loads<T: Content + std::fmt::Debug>(json: &str) -> Document<T> {
    match parse::<T>(json) {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {json}\n{:#?}", rejected.diagnostics),
    }
}

/// The findings of `json`, which must be rejected.
#[track_caller]
pub fn rejects<T: Content + std::fmt::Debug>(json: &str) -> Diagnostics {
    match parse::<T>(json) {
        Ok(document) => panic!("loaded: {json}\n{:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// `(code name, JSON path, message)` of each finding of `severity`.
pub fn found<'a>(
    d: impl IntoIterator<Item = &'a Diagnostic>,
    severity: Severity,
) -> Vec<(&'static str, String, String)> {
    d.into_iter()
        .filter(|f| f.severity == severity)
        .map(|f| {
            (
                f.code.id(),
                f.location.json_path.to_string(),
                f.code.to_string(),
            )
        })
        .collect()
}

/// The errors of `d`.
pub fn errors<'a>(
    d: impl IntoIterator<Item = &'a Diagnostic>,
) -> Vec<(&'static str, String, String)> {
    found(d, Severity::Error)
}

/// The warnings of `d`.
pub fn warnings<'a>(
    d: impl IntoIterator<Item = &'a Diagnostic>,
) -> Vec<(&'static str, String, String)> {
    found(d, Severity::Warning)
}

/// The infos of `d`.
pub fn infos<'a>(
    d: impl IntoIterator<Item = &'a Diagnostic>,
) -> Vec<(&'static str, String, String)> {
    found(d, Severity::Info)
}

/// `(code name, JSON path)` of each entry.
pub fn at<'a>(found: &'a [(&'static str, String, String)]) -> Vec<(&'static str, &'a str)> {
    found
        .iter()
        .map(|(code, path, _)| (*code, path.as_str()))
        .collect()
}

/// One expected finding.
pub fn finding(code: &'static str, path: &str, message: &str) -> (&'static str, String, String) {
    (code, path.to_owned(), message.to_owned())
}

/// The one error of `json` loaded as `T`: `(code name, JSON path, message)`.
#[track_caller]
pub fn one_error<T: Content + std::fmt::Debug>(json: &str) -> (&'static str, String, String) {
    let d = rejects::<T>(json);
    let mut errors = errors(&d);
    assert_eq!(errors.len(), 1, "{json}\n{errors:#?}");
    errors.remove(0)
}

/// The JSON of `text`.
#[track_caller]
pub fn json(text: &str) -> Value {
    facet_json::from_str::<Value>(text).unwrap_or_else(|error| panic!("{text}: {error}"))
}

/// Writes `value`, reads the text back strictly (no finding at all) and requires the second
/// writing to be the same text; returns the text.
#[track_caller]
pub fn round_trip<T: Content + std::fmt::Debug + PartialEq>(value: &T) -> String {
    let written = value.to_json().expect("written");
    let again = match parse::<T>(&written) {
        Ok(document) => document,
        Err(rejected) => panic!("{written}\n{:#?}", rejected.diagnostics),
    };
    assert!(
        again.diagnostics().is_empty(),
        "{written}\n{:#?}",
        again.diagnostics()
    );
    assert_eq!(again.value(), value, "{written}");
    assert_eq!(again.to_json().expect("rewritten"), written);
    written
}

/// A text that lives as long as the test binary (the pack helper takes `'static` files).
pub fn leak(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

/// A resource stack of packs, top first; each pack is its `(path, text)` files.
pub fn stack(packs: &[&[(&str, String)]]) -> PackStack<Resource> {
    let packs: Vec<_> = packs
        .iter()
        .enumerate()
        .map(|(index, files)| {
            let mut spec = rp(index as u128 + 1);
            for (path, text) in *files {
                spec = spec.file(leak((*path).to_owned()), leak(text.clone()).as_bytes());
            }
            spec.open()
        })
        .collect();
    PackStack::<Resource>::of(packs, LoadOptions::default()).unwrap()
}

/// `(pack index, path)` of a document's origin.
pub fn origin<T>(document: &Document<T>) -> (Option<usize>, &str) {
    (
        document.origin().pack.index(),
        document.origin().path.as_str(),
    )
}

/// `(code name, pack index, file path, message)` of each finding of `severity`.
pub fn located<'a>(
    d: impl IntoIterator<Item = &'a Diagnostic>,
    severity: Severity,
) -> Vec<(&'static str, Option<usize>, String, String)> {
    d.into_iter()
        .filter(|f| f.severity == severity)
        .map(|f| {
            (
                f.code.id(),
                f.location
                    .pack
                    .as_ref()
                    .and_then(bedrock_addon::pack::PackRef::index),
                f.location
                    .path
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                f.code.to_string(),
            )
        })
        .collect()
}
