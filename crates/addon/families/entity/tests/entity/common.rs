//! Shared helpers of the entity tests: loading a document, its findings by code, fixtures.
#![allow(dead_code)]

use std::path::PathBuf;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon_core::document::{Content, Document};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::versioning::{LoadOutcome, Loaded, load};
use bedrock_addon_family_entity::Entity;

/// The fixtures of the entity family.
pub fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../addon/tests/fixtures/entity/entity")
}

/// A fixture's text.
pub fn fixture(name: &str) -> String {
    let path = fixtures().join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// A vanilla file copied for a test (`tests/fixtures/vanilla_entity/<name>`, from
/// `vanilla_unpacked/behavior_packs/vanilla_<version>/entities/`). Kept apart from the
/// family's fixture folder: the writable probes read every small file there, and a whole
/// vanilla entity is many places to probe.
pub fn vanilla(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../addon/tests/fixtures/vanilla_entity")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// A load context of a pack outside any stack.
pub fn cx() -> LoadContext {
    LoadContext::detached()
}

/// Loads `text` through the whole chain.
pub fn outcome(text: &str, cx: &LoadContext) -> LoadOutcome<Entity> {
    load::<Entity>(JsonSlice::new(text), cx)
}

/// The newest struct `text` loads to, with its findings; panics when it is rejected.
pub fn stable(text: &str, cx: &LoadContext) -> (Entity, Diagnostics) {
    let outcome = outcome(text, cx);
    match outcome.value {
        Some(Loaded::Stable(entity)) => (entity, outcome.diagnostics),
        other => panic!(
            "expected a stable entity, got {other:?} with {:#?}",
            outcome.diagnostics
        ),
    }
}

/// `Content::parse` of `text`.
pub fn parse(text: &str) -> Result<Document<Entity>, bedrock_addon_core::document::Rejected> {
    Entity::parse(
        &PackPath::new("entities/test.json").unwrap(),
        text.as_bytes(),
        &cx(),
    )
}

/// The names of the codes of `diagnostics` at `severity` or above, in order.
pub fn codes(diagnostics: &Diagnostics, severity: Severity) -> Vec<&'static str> {
    diagnostics
        .iter()
        .filter(|d| d.severity >= severity)
        .map(|d| d.code.id())
        .collect()
}

/// The findings whose code is `name`.
pub fn with_code<'a>(diagnostics: &'a Diagnostics, name: &str) -> Vec<&'a Diagnostic> {
    diagnostics.iter().filter(|d| d.code.id() == name).collect()
}

/// `text` as a JSON value.
pub fn json(text: &str) -> Value {
    facet_json::from_str_jsonc::<Value>(text).unwrap_or_else(|e| panic!("{e}: {text}"))
}

/// A 1.26.50 document whose payload is `payload` (JSON text of the `minecraft:entity` object).
pub fn doc(version: &str, payload: &str) -> String {
    format!(r#"{{"format_version": "{version}", "minecraft:entity": {payload}}}"#)
}

/// Writes `entity` and reads it back.
pub fn reparse(entity: &Entity) -> Entity {
    let text = entity.to_json().expect("the entity writes");
    stable(&text, &cx()).0
}

/// `components` (a JSON object) as the newest struct reads and writes it: the shape the last
/// step (1.26.40 → 1.26.50, the typed conversion) produces, with the typed defaults written.
pub fn latest_components(components: &str) -> Value {
    let text = doc(
        "1.26.50",
        &format!(r#"{{"description": {{"identifier": "demo:x"}}, "components": {components}}}"#),
    );
    let (entity, found) = stable(&text, &cx());
    assert!(
        codes(&found, Severity::Warning).is_empty(),
        "{components}: {found:#?}"
    );
    json(&entity.to_json().expect("the entity writes"))
        .as_object()
        .and_then(|d| d.get("minecraft:entity"))
        .and_then(Value::as_object)
        .and_then(|p| p.get("components"))
        .cloned()
        .expect("components")
}
