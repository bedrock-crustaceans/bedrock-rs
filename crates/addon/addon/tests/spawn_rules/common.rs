//! What the spawn tests share: contexts with and without the Simplified Spawn Rules experiment,
//! loading one document, and finding diagnostics by code, severity and text.

#![allow(dead_code, reason = "each test module uses part of the helpers")]

use std::path::PathBuf;

use bedrock_addon::diagnostic::{Code, Diagnostic, Diagnostics, Severity};
use bedrock_addon::document::{Content, Document, Rejected};
use bedrock_addon::families::spawn_groups::SpawnGroup;
use bedrock_addon::families::spawn_rules::{
    Condition, MobSpawnRules, SimplifiedSpawnRules, SpawnRules, StandardSpawnRules,
};
use bedrock_addon::load::{Experiment, Experiments, LoadContext, LoadOptions};
use bedrock_addon::pack::PackType;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::MinEngineVersion;
use uuid::Uuid;

/// A behaviour-pack context at engine 1.26.20, no experiments.
pub fn context() -> LoadContext {
    with_options(&LoadOptions::default())
}

/// A behaviour-pack context of a world with `options`.
pub fn with_options(options: &LoadOptions) -> LoadContext {
    let mut cx = LoadContext::for_pack(
        options,
        Uuid::from_u128(7),
        PackType::Behavior,
        Some(MinEngineVersion::new(1, 26, 20)),
        false,
    );
    cx.molang_version = bedrock_addon::molang::pack_molang_version(cx.min_engine_version.as_ref());
    cx
}

/// The options of a world with the Simplified Spawn Rules experiment on.
pub fn simplified_options() -> LoadOptions {
    let mut experiments = Experiments::new();
    experiments.insert(Experiment::SimplifiedSpawnRules);
    LoadOptions {
        experiments,
        ..LoadOptions::default()
    }
}

/// The context with the Simplified Spawn Rules experiment on.
pub fn experiment() -> LoadContext {
    with_options(&simplified_options())
}

/// Loads `text` as `spawn_rules/x.json`.
pub fn load(text: &str, cx: &LoadContext) -> Result<Document<SpawnRules>, Rejected> {
    SpawnRules::parse(
        &PackPath::from_static("spawn_rules/x.json"),
        text.as_bytes(),
        cx,
    )
}

/// Loads `text` and requires it to load.
#[track_caller]
pub fn loads(text: &str, cx: &LoadContext) -> Document<SpawnRules> {
    match load(text, cx) {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {:#?}", rejected.diagnostics),
    }
}

/// Loads `text` and requires it to be rejected.
#[track_caller]
pub fn rejected(text: &str, cx: &LoadContext) -> Diagnostics {
    match load(text, cx) {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// The standard rules of a document.
#[track_caller]
pub fn standard(document: &Document<SpawnRules>) -> &StandardSpawnRules {
    match document.value() {
        SpawnRules::Standard(rules) => rules,
        SpawnRules::Simplified(rules) => panic!("simplified: {rules:#?}"),
    }
}

/// The simplified rules of a document.
#[track_caller]
pub fn simplified(document: &Document<SpawnRules>) -> &SimplifiedSpawnRules {
    match document.value() {
        SpawnRules::Simplified(rules) => rules,
        SpawnRules::Standard(rules) => panic!("standard: {rules:#?}"),
    }
}

/// A 1.8.0 document for `x:mob` in the animal pool with `conditions` (the array's elements).
pub fn legacy(conditions: &str) -> String {
    format!(
        r#"{{"format_version": "1.8.0", "minecraft:spawn_rules": {{"description": {{"identifier": "x:mob", "population_control": "animal"}}, "conditions": [{conditions}]}}}}"#
    )
}

/// A document of `version` with `payload` as `minecraft:spawn_rules`.
pub fn document(version: &str, payload: &str) -> String {
    format!(r#"{{"format_version": "{version}", "minecraft:spawn_rules": {payload}}}"#)
}

/// Loads one legacy condition with body `body` (the members of the condition object): the
/// condition, what vanilla makes of it, and the document's findings.
#[track_caller]
pub fn condition(body: &str) -> (Condition, MobSpawnRules, Diagnostics) {
    let document = loads(&legacy(&format!("{{{body}}}")), &context());
    let rules = standard(&document);
    let condition = rules.conditions.as_ref().expect("conditions")[0].clone();
    let mob = condition.to_spawn_rules();
    (condition, mob, document.diagnostics().clone())
}

/// The findings whose code matches `code`.
pub fn with<'a>(
    diagnostics: &'a Diagnostics,
    code: impl Fn(&Code) -> bool + 'a,
) -> Vec<&'a Diagnostic> {
    diagnostics.iter().filter(|d| code(&d.code)).collect()
}

/// Whether a finding has `code`, `severity` and exactly `text` as its message.
pub fn has_text(
    diagnostics: &Diagnostics,
    code: impl Fn(&Code) -> bool,
    severity: Severity,
    text: &str,
) -> bool {
    diagnostics
        .iter()
        .any(|d| code(&d.code) && d.severity == severity && d.code.to_string() == text)
}

/// The code names of the findings at or above `severity`.
pub fn names(diagnostics: &Diagnostics, severity: Severity) -> Vec<&'static str> {
    diagnostics
        .at_least(severity)
        .map(|d| d.code.name())
        .collect()
}

/// The findings' paths, as text.
pub fn paths(diagnostics: &[&Diagnostic]) -> Vec<String> {
    diagnostics
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect()
}

/// Whether the rejection ends with vanilla's closing line `text`.
pub fn closes_with(diagnostics: &Diagnostics, text: &str) -> bool {
    diagnostics.iter().last().is_some_and(|d| {
        // By legacy name: the variant and the definitions that were it.
        d.code.name() == "DocumentRejected"
            && d.severity == Severity::Error
            && d.code.to_string() == text
    })
}

/// Writes `value` and requires the text to read back strictly to an equal value that writes the
/// same text again.
#[track_caller]
pub fn round_trip<T: Content + PartialEq + std::fmt::Debug>(
    value: &T,
    path: &str,
    cx: &LoadContext,
) -> String {
    let text = value.to_json().expect("written");
    let again = T::parse(&PackPath::new(path).expect("a path"), text.as_bytes(), cx)
        .unwrap_or_else(|r| panic!("re-read: {:#?}\n{text}", r.diagnostics));
    assert_eq!(again.value(), value, "{text}");
    assert_eq!(again.to_json().expect("rewritten"), text);
    text
}

/// Loads `text` as the spawn group `definitions/spawn_groups/x.json`.
pub fn load_group(text: &str) -> Result<Document<SpawnGroup>, Rejected> {
    SpawnGroup::parse(
        &PackPath::from_static("definitions/spawn_groups/x.json"),
        text.as_bytes(),
        &context(),
    )
}

/// The fixture directory `tests/fixtures/spawn/<family>/<rest>`.
pub fn fixture_path(family: &str, rest: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/spawn")
        .join(family)
        .join(rest)
}

/// A fixture file's text.
pub fn fixture(family: &str, rest: &str) -> String {
    let path = fixture_path(family, rest);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}
