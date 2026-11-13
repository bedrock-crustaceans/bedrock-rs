//! What the POI tests share: the fixture files and loading one document. The packs and stacks
//! built from them are in `bedrock_addon`'s `tests/poi/stacks.rs`.
//!
//! A fixture is `tests/fixtures/poi/poi/<name>/`: one behaviour-pack tree (`poi/…`) or, for a
//! stack, `stack/<n>_<pack>/poi/…` with `0_…` the highest priority. Unless a test says
//! otherwise the `poi` experiment is on.

#![allow(dead_code, reason = "each test module uses part of the helpers")]

use std::fmt::Write as _;
use std::path::PathBuf;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::{Experiment, Experiments, LoadContext, LoadOptions};
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::pack_identity::PackType;
use bedrock_addon_core::version::MinEngineVersion;
use uuid::Uuid;

/// The fixture directory `tests/fixtures/poi/poi/<name>`.
pub fn fixture_dir(name: &str) -> PathBuf {
    bedrock_addon_test_support::fixture::root(env!("CARGO_MANIFEST_DIR"))
        .join("poi/poi")
        .join(name)
}

/// A fixture file's text.
pub fn fixture(rest: &str) -> String {
    let path = fixture_dir(rest);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The world options with the `poi` experiment on.
pub fn poi_options() -> LoadOptions {
    let mut experiments = Experiments::new();
    experiments.insert(Experiment::Poi);
    LoadOptions {
        experiments,
        ..LoadOptions::default()
    }
}

/// A behaviour-pack context loaded with `options`.
pub fn context_with(options: &LoadOptions, base_game: bool) -> LoadContext {
    LoadContext::for_pack(
        options,
        Uuid::from_u128(7),
        PackType::Behavior,
        Some(MinEngineVersion::new(1, 26, 40)),
        base_game,
    )
}

/// A behaviour-pack context at the engine version with the `poi` experiment on.
pub fn context() -> LoadContext {
    context_with(&poi_options(), false)
}

/// Loads `text` as the document of type `T` at `poi/<file>` with `cx`.
pub fn load_with<T: Content>(text: &str, cx: &LoadContext) -> Result<Document<T>, Rejected> {
    T::parse(
        &PackPath::new("poi/document.json").expect("a pack path"),
        text.as_bytes(),
        cx,
    )
}

/// Loads `text` as the document of type `T`, the `poi` experiment on.
pub fn load<T: Content>(text: &str) -> Result<Document<T>, Rejected> {
    load_with(text, &context())
}

/// Loads `text` and requires it to load.
#[track_caller]
pub fn loads<T: Content>(text: &str) -> Document<T> {
    match load::<T>(text) {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {}", describe(rejected.diagnostics.iter())),
    }
}

/// Loads `text` and requires it to be rejected.
#[track_caller]
pub fn rejected<T: Content + std::fmt::Debug>(text: &str) -> Diagnostics {
    match load::<T>(text) {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// Loads `text`, requires it to be rejected and returns the code names of its errors.
#[track_caller]
pub fn errors<T: Content + std::fmt::Debug>(text: &str) -> Vec<&'static str> {
    names(rejected::<T>(text).iter(), Severity::Error)
}

/// Whether `finding` is the closing error "Failed to load POI Type" every failed file ends
/// with. A test of one rule looks at the findings before it; `rules.rs` pins the
/// closing line itself.
pub fn is_closing(finding: &Diagnostic) -> bool {
    finding.is(&bedrock_addon_family_poi::codes::FAILED_TO_LOAD)
}

/// The code names of the findings at `severity`, the closing line left out.
pub fn names<'a>(
    findings: impl IntoIterator<Item = &'a Diagnostic>,
    severity: Severity,
) -> Vec<&'static str> {
    findings
        .into_iter()
        .filter(|d| d.severity == severity && !is_closing(d))
        .map(|d| d.code.id())
        .collect()
}

/// The findings, one per line: severity, code, file, JSON path, text.
pub fn describe<'a>(findings: impl IntoIterator<Item = &'a Diagnostic>) -> String {
    findings.into_iter().fold(String::new(), |mut out, f| {
        write!(
            out,
            "\n  {} {} {:?} {} {}",
            f.severity,
            f.code.id(),
            f.location.path.as_ref().map(PackPath::as_str),
            f.location.json_path,
            f.code
        )
        .unwrap();
        out
    })
}

/// A `minecraft:poi_block` document of `version` with `payload`.
pub fn block_document(version: &str, payload: &str) -> String {
    format!(r#"{{"format_version": "{version}", "minecraft:poi_block": {payload}}}"#)
}

/// A `minecraft:poi_block` document at 1.26.40 with `payload`.
pub fn block(payload: &str) -> String {
    block_document("1.26.40", payload)
}

/// A `minecraft:poi_tag` document at 1.26.40 with `payload`.
pub fn tag(payload: &str) -> String {
    format!(r#"{{"format_version": "1.26.40", "minecraft:poi_tag": {payload}}}"#)
}

/// A minimal type document `identifier` of `blocks` (a JSON array) with `rest` (further
/// members, each ending in a comma) before `blocks`.
pub fn block_of(identifier: &str, rest: &str, blocks: &str) -> String {
    block(&format!(
        r#"{{"description": {{"identifier": "{identifier}"}}, {rest} "blocks": {blocks}}}"#
    ))
}

/// A tag document `identifier` with `rest` (further members, each starting with a comma).
pub fn tag_of(identifier: &str, rest: &str) -> String {
    tag(&format!(
        r#"{{"description": {{"identifier": "{identifier}"}}{rest}}}"#
    ))
}
