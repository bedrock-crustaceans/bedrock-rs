//! Helpers of the feature-rule tests.

#![allow(dead_code)]

use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, Severity};
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::version::MinEngineVersion;
use bedrock_addon_family_feature_rules::FeatureRule;

#[allow(unused_imports, reason = "each test file uses part of the helpers")]
pub use bedrock_addon_test_support::with_code;

/// The fixture folder of this family, found from whichever crate includes this file: the
/// fixtures live in `bedrock_addon`'s `tests/fixtures`.
pub fn fixtures() -> std::path::PathBuf {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    [root.join("tests"), root.join("../../addon/tests")]
        .into_iter()
        .map(|tests| tests.join("fixtures/worldgen/feature_rules"))
        .find(|folder| folder.is_dir())
        .expect("the feature_rules fixture folder")
}

/// The text of the fixture file `path` (relative to the family's fixture folder).
pub fn fixture(path: &str) -> String {
    std::fs::read_to_string(fixtures().join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// A behaviour pack's context with `min_engine_version` `[major, minor, patch]`.
pub fn pack(major: u16, minor: u16, patch: u16) -> LoadContext {
    let mut cx = LoadContext::detached();
    let version = MinEngineVersion::new(major, minor, patch);
    cx.molang_version = version.molang_version().into();
    cx.min_engine_version = Some(version);
    cx
}

/// Loads `json` as the rule file `path` (`feature_rules/<stem>.json`).
pub fn load_at(
    path: &'static str,
    json: &str,
    cx: &LoadContext,
) -> Result<Document<FeatureRule>, Rejected> {
    FeatureRule::parse(&PackPath::from_static(path), json.as_bytes(), cx)
}

/// Loads `json` as `feature_rules/<name>.json`, where `<name>` is the part of the document's
/// identifier after its last `:` (so the file-name check passes).
pub fn load(json: &str, cx: &LoadContext) -> Result<Document<FeatureRule>, Rejected> {
    let value: bedrock_addon_core::json::Value = facet_json::from_str(json).unwrap_or_default();
    let identifier = value
        .as_object()
        .and_then(|o| o.get("minecraft:feature_rules"))
        .and_then(|p| p.as_object()?.get("description"))
        .and_then(|d| d.as_object()?.get("identifier"))
        .and_then(|i| i.as_string().map(|s| s.as_str().to_owned()))
        .unwrap_or_else(|| "test".to_owned());
    let name = identifier.rsplit(':').next().unwrap_or("test");
    let path = PackPath::new(&format!("feature_rules/{name}.json")).unwrap();
    FeatureRule::parse(&path, json.as_bytes(), cx)
}

/// Loads `json`, which must load.
#[track_caller]
pub fn loads(json: &str) -> Document<FeatureRule> {
    loads_with(json, &LoadContext::detached())
}

/// Loads `json` with `cx`, which must load.
#[track_caller]
pub fn loads_with(json: &str, cx: &LoadContext) -> Document<FeatureRule> {
    match load(json, cx) {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {:#?}", rejected.diagnostics),
    }
}

/// Loads `json`, which must be rejected; its findings.
#[track_caller]
pub fn rejects(json: &str) -> Diagnostics {
    match load(json, &LoadContext::detached()) {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// Whether a finding's message is exactly `text`.
pub fn has_text(d: &Diagnostics, text: &str) -> bool {
    d.iter().any(|f| f.code.to_string() == text)
}

/// The findings above info.
pub fn above_info(d: &Diagnostics) -> Vec<&Diagnostic> {
    d.iter().filter(|f| f.severity > Severity::Info).collect()
}

/// Whether `d` holds a finding whose code matches.
pub fn has(d: &Diagnostics, pred: impl Fn(&Code) -> bool) -> bool {
    d.iter().any(|f| pred(&f.code))
}

/// A rule document at `version` with `distribution` (a JSON object's text, or `None` for none)
/// and `placement_pass`.
pub fn rule(version: &str, pass: &str, distribution: Option<&str>) -> String {
    let distribution = distribution.map_or(String::new(), |d| format!(r#", "distribution": {d}"#));
    format!(
        r#"{{"format_version": "{version}", "minecraft:feature_rules": {{"description": {{"identifier": "demo:test", "places_feature": "demo:feature"}}, "conditions": {{"placement_pass": "{pass}"}}{distribution}}}}}"#
    )
}

/// A `distribution` object with `iterations` and the three coordinates as given (JSON texts).
pub fn distribution(iterations: &str, x: &str, y: &str, z: &str) -> String {
    format!(r#"{{"iterations": {iterations}, "x": {x}, "y": {y}, "z": {z}}}"#)
}

/// A uniform coordinate over `[0, 15]`.
pub const UNIFORM: &str = r#"{"distribution": "uniform", "extent": [0, 15]}"#;

/// The vanilla unit-test input `name` (`without_distribution`, `custom_feature_rule_v1_13`) at
/// `version`: the shapes of `AutomaticFeatureRules`'s unit tests, rebuilt from the
/// folder's description.
pub fn vanilla_input(name: &str, version: &str) -> String {
    fixture(&format!("vanilla_test_input/{name}.json")).replace("{version}", version)
}
