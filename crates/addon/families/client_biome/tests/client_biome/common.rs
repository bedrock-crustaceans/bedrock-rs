//! What the client-biome tests share: loading one document, contexts, packs, fixture files.

#![allow(dead_code, reason = "each test module uses part of the helpers")]

use std::path::PathBuf;

use bedrock_addon_core::diagnostic::{Code, Diagnostics, Severity};
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::{LoadContext, LoadOptions};
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::pack_identity::PackType;
use bedrock_addon_core::version::{BaseGameVersion, MinEngineVersion, SemVersion};
use bedrock_addon_family_client_biome::{Beta, ClientBiome};
use uuid::Uuid;

#[allow(unused_imports, reason = "each test file uses part of the helpers")]
pub use bedrock_addon_test_support::{errors, json};

/// The file every single-file test loads from.
pub const PATH: &str = "biomes/demo.client_biome.json";

/// A resource-pack context of a pack other than a base-game pack: the engine version, no
/// toggles, no world version (every world-gated entry applies).
pub fn context() -> LoadContext {
    LoadContext::for_pack(
        &LoadOptions::default(),
        Uuid::from_u128(7),
        PackType::Resource,
        Some(MinEngineVersion::new(1, 26, 30)),
        false,
    )
}

/// The context in a world of base-game version `major.minor.patch`.
pub fn world(major: u16, minor: u16, patch: u16) -> LoadContext {
    let mut cx = context();
    cx.world_base_game_version = BaseGameVersion::new(SemVersion::new(major, minor, patch));
    cx
}

/// The context of a base-game pack.
pub fn base_game() -> LoadContext {
    let mut cx = context();
    cx.is_base_game_pack = true;
    cx
}

/// The context with the Beta APIs toggle on.
pub fn beta_apis() -> LoadContext {
    let mut cx = context();
    cx.beta_apis = true;
    cx
}

/// The path every single-file test loads from.
pub fn path() -> PackPath {
    PackPath::from_static(PATH)
}

/// Loads `text` as a client biome.
pub fn load(text: &str, cx: &LoadContext) -> Result<Document<ClientBiome>, Rejected> {
    ClientBiome::parse(&path(), text.as_bytes(), cx)
}

/// Loads `text` and requires it to load.
#[track_caller]
pub fn loads(text: &str, cx: &LoadContext) -> Document<ClientBiome> {
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

/// Loads `text` as a beta client biome.
pub fn load_beta(text: &str, cx: &LoadContext) -> Result<Document<Beta>, Rejected> {
    Beta::parse(&path(), text.as_bytes(), cx)
}

/// A document of `version` with `payload` as `minecraft:client_biome`.
pub fn document(version: &str, payload: &str) -> String {
    format!(r#"{{"format_version": "{version}", "minecraft:client_biome": {payload}}}"#)
}

/// A document of `version` named `identifier` with `components` (a JSON object's members).
pub fn biome(version: &str, identifier: &str, components: &str) -> String {
    document(
        version,
        &format!(
            r#"{{"description": {{"identifier": "{identifier}"}}, "components": {{{components}}}}}"#
        ),
    )
}

/// The document written back, as JSON.
pub fn written(document: &Document<ClientBiome>) -> facet_value::Value {
    json(&document.to_json().expect("the document writes"))
}

/// The `components` object a document writes back.
pub fn components(document: &Document<ClientBiome>) -> facet_value::Value {
    written(document)
        .as_object()
        .and_then(|root| root.get("minecraft:client_biome"))
        .and_then(|payload| payload.as_object())
        .and_then(|payload| payload.get("components"))
        .cloned()
        .expect("components")
}

/// The component `name` a document writes back.
pub fn component(document: &Document<ClientBiome>, name: &str) -> Option<facet_value::Value> {
    components(document)
        .as_object()
        .and_then(|components| components.get(name))
        .cloned()
}

/// How many findings have a code `code` accepts.
pub fn count(diagnostics: &Diagnostics, code: impl Fn(&Code) -> bool) -> usize {
    diagnostics.iter().filter(|d| code(&d.code)).count()
}

/// Whether a finding has `severity`, exactly the text `text` and a code `code` accepts.
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

/// Whether a finding at `at` has `severity` and a code `code` accepts.
pub fn has_at(
    diagnostics: &Diagnostics,
    code: impl Fn(&Code) -> bool,
    severity: Severity,
    at: &str,
) -> bool {
    diagnostics.iter().any(|d| {
        code(&d.code) && d.severity == severity && format!("/{}", d.location.json_path) == at
    })
}

/// The findings as text, for assertion messages.
pub fn texts(diagnostics: &Diagnostics) -> Vec<String> {
    diagnostics
        .iter()
        .map(|d| format!("{:?} {} {}", d.severity, d.location.json_path, d.code))
        .collect()
}

/// The `tests/fixtures` folder of `bedrock_addon`: this file is compiled by the family crate and
/// included by `bedrock_addon`'s own test root, which owns the folder.
fn fixtures_root() -> std::path::PathBuf {
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let own = manifest.join("tests/fixtures");
    if own.is_dir() {
        own
    } else {
        manifest.join("../../addon/tests/fixtures")
    }
}

/// The fixture directory `tests/fixtures/biome/biomes_client/<rest>`.
pub fn fixture_dir(rest: &str) -> PathBuf {
    fixtures_root().join("biome/biomes_client").join(rest)
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
