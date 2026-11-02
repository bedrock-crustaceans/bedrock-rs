//! The link tests of entity and spawn references.

#![allow(dead_code, reason = "each feature set uses part of the helpers")]

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticDetail, Diagnostics, Phase, Severity};

#[cfg(feature = "entity")]
mod audit;
#[cfg(feature = "entity")]
mod entity_animation;
#[cfg(feature = "entity")]
mod entity_block;
#[cfg(feature = "entity")]
mod entity_entity;
#[cfg(feature = "entity")]
mod entity_item;
#[cfg(feature = "entity")]
mod entity_loot;
#[cfg(feature = "entity")]
mod entity_particle;
#[cfg(feature = "entity")]
mod entity_poi;
#[cfg(feature = "entity")]
mod entity_trade;
#[cfg(all(feature = "spawn_rules", feature = "spawn_groups"))]
mod spawn;

/// The path of the entity file every fixture of this group uses.
pub const FILE: &str = "entities/pet.json";

/// An entity document of format `version` named `id` whose `components` hold `components`
/// (JSON members) and whose payload has the further members `more` (JSON members, each
/// starting with a comma, or empty).
pub fn entity_with(version: &str, id: &str, components: &str, more: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "minecraft:entity": {{"description": {{"identifier": "{id}"}}, "components": {{{components}}}{more}}}}}"#
    )
}

/// The entity `demo:pet` in the newest format with `components`.
pub fn pet(components: &str) -> String {
    entity_with("1.26.50", "demo:pet", components, "")
}

/// A custom block `id`.
pub fn block(id: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.20", "minecraft:block": {{"description": {{"identifier": "{id}"}}, "components": {{}}}}}}"#
    )
}

/// A custom item `id`.
pub fn item(id: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.30", "minecraft:item": {{"description": {{"identifier": "{id}"}}, "components": {{}}}}}}"#
    )
}

/// What a finding says: the code's name, the severity, the file, the JSON path, the kind and
/// the name as written.
pub type Said = (&'static str, Severity, String, String, &'static str, String);

/// [`Said`] of one finding; it must be a finding of the link phase told at pack `bp 1`.
pub fn said(finding: &Diagnostic) -> Said {
    assert_eq!(finding.phase, Phase::Link, "{finding:?}");
    let pack = finding.location.pack.as_ref().expect("a pack");
    assert_eq!(pack.index(), Some(0), "{finding:?}");
    assert_eq!(pack.id().uuid, uuid::Uuid::from_u128(1), "{finding:?}");
    let (kind, id) = match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference { kind, id, .. }) => (*kind, id.clone()),
        _ => ("", String::new()),
    };
    (
        finding.code.name(),
        finding.severity,
        finding
            .location
            .path
            .as_ref()
            .map(|path| path.as_str().to_owned())
            .unwrap_or_default(),
        finding.location.json_path.to_string(),
        kind,
        id,
    )
}

/// [`said`] of every finding about a reference of `kind`.
pub fn said_of(findings: &Diagnostics, kind: &str) -> Vec<Said> {
    findings
        .iter()
        .filter(|finding| {
            matches!(finding.detail.as_deref(), Some(DiagnosticDetail::Reference { kind: k, .. }) if *k == kind)
        })
        .map(said)
        .collect()
}

/// One expected finding in the entity file.
pub fn miss(
    code: &'static str,
    severity: Severity,
    path: &str,
    kind: &'static str,
    id: &str,
) -> Said {
    (
        code,
        severity,
        FILE.to_owned(),
        path.to_owned(),
        kind,
        id.to_owned(),
    )
}

/// An `UnresolvedReference` information in the entity file at `path` below
/// `minecraft:entity`: a miss the game reads without a content-log line.
pub fn unresolved(path: &str, kind: &'static str, id: &str) -> Said {
    miss(
        "UnresolvedReference",
        Severity::Info,
        &format!("minecraft:entity/{path}"),
        kind,
        id,
    )
}

/// An `UnresolvedReference` kept error in the entity file at `path` below `minecraft:entity`:
/// a miss the game logs at the content log's error level and goes on from. The `kept` flag is asserted by the test.
pub fn kept_unresolved(path: &str, kind: &'static str, id: &str) -> Said {
    miss(
        "UnresolvedReference",
        Severity::Error,
        &format!("minecraft:entity/{path}"),
        kind,
        id,
    )
}
