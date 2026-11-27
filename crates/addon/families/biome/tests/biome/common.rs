//! What the biome tests share: loading one document, contexts, packs, the fixture files.

#![allow(dead_code, reason = "each test module uses part of the helpers")]

use std::path::PathBuf;

use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, Severity};
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::{LoadContext, LoadOptions};
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::pack_identity::PackType;
use bedrock_addon_core::version::MinEngineVersion;
use bedrock_addon_family_biome::{Beta, Biome};
use uuid::Uuid;

#[allow(unused_imports, reason = "each test file uses part of the helpers")]
pub use bedrock_addon_test_support::errors;

/// A behaviour-pack context of a pack other than a base-game pack: the engine version, no
/// toggles, no world version.
pub fn context() -> LoadContext {
    LoadContext::for_pack(
        &LoadOptions::default(),
        Uuid::from_u128(7),
        PackType::Behavior,
        Some(MinEngineVersion::new(1, 26, 30)),
        false,
    )
}

/// The context of a base-game pack.
pub fn base_game() -> LoadContext {
    let mut cx = context();
    cx.is_base_game_pack = true;
    cx
}

/// The context with the Beta APIs toggle on.
pub fn beta_apis() -> LoadContext {
    let options = LoadOptions {
        beta_apis: true,
        ..LoadOptions::default()
    };
    LoadContext::for_pack(
        &options,
        Uuid::from_u128(7),
        PackType::Behavior,
        Some(MinEngineVersion::new(1, 26, 30)),
        false,
    )
}

/// The context with the Molang compile check on.
#[cfg(feature = "molang")]
pub fn checked(mut cx: LoadContext) -> LoadContext {
    cx.molang = Some(std::sync::Arc::new(
        bedrock_addon_core::molang::MolangCheck::new(),
    ));
    cx
}

/// The path every single-file test loads from.
pub fn path() -> PackPath {
    PackPath::from_static("biomes/demo.biome.json")
}

/// Loads `text` as a biome.
pub fn load(text: &str, cx: &LoadContext) -> Result<Document<Biome>, Rejected> {
    Biome::parse(&path(), text.as_bytes(), cx)
}

/// Loads `text` and requires it to load.
#[track_caller]
pub fn loads(text: &str, cx: &LoadContext) -> Document<Biome> {
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

/// Loads `text` as a beta biome.
pub fn load_beta(text: &str, cx: &LoadContext) -> Result<Document<Beta>, Rejected> {
    Beta::parse(&path(), text.as_bytes(), cx)
}

/// A document of `version` with `payload` as `minecraft:biome`.
pub fn document(version: &str, payload: &str) -> String {
    format!(r#"{{"format_version": "{version}", "minecraft:biome": {payload}}}"#)
}

/// A payload named `identifier` with `components` (a JSON object's members).
pub fn payload(identifier: &str, components: &str) -> String {
    format!(
        r#"{{"description": {{"identifier": "{identifier}"}}, "components": {{{components}}}}}"#
    )
}

/// A document of `version` named `identifier` with `components`.
pub fn biome(version: &str, identifier: &str, components: &str) -> String {
    document(version, &payload(identifier, components))
}

/// The findings whose code matches `code`.
pub fn with<'a>(
    diagnostics: &'a Diagnostics,
    code: impl Fn(&Code) -> bool + 'a,
) -> Vec<&'a Diagnostic> {
    diagnostics.iter().filter(move |d| code(&d.code)).collect()
}

/// Whether a finding has `code`, `severity` and exactly the text `text`.
pub fn has(
    diagnostics: &Diagnostics,
    code: impl Fn(&Code) -> bool,
    severity: Severity,
    text: &str,
) -> bool {
    diagnostics
        .iter()
        .any(|d| code(&d.code) && d.severity == severity && d.code.to_string() == text)
}

/// The texts of the findings at `severity`, for assertion messages.
pub fn texts(diagnostics: &Diagnostics, severity: Severity) -> Vec<String> {
    diagnostics
        .iter()
        .filter(|d| d.severity == severity)
        .map(|d| format!("{} {}", d.location.json_path, d.code))
        .collect()
}

/// The fixture directory `tests/fixtures/biome/biome/<rest>`.
pub fn fixture_dir(rest: &str) -> PathBuf {
    bedrock_addon_test_support::fixture::root(env!("CARGO_MANIFEST_DIR"))
        .join("biome/biome")
        .join(rest)
}

/// A fixture file's text.
pub fn fixture(rest: &str) -> String {
    let path = fixture_dir(rest);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Every file of a fixture directory, sorted by name: `(file name, text)`.
pub fn fixtures(rest: &str) -> Vec<(String, String)> {
    let mut files: Vec<_> = std::fs::read_dir(fixture_dir(rest))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    files.sort();
    files
        .into_iter()
        .map(|path| {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            (name, std::fs::read_to_string(&path).unwrap())
        })
        .collect()
}
