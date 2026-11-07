//! The link tests of world generation, the small behaviour-pack families and cameras (20 rows W, M5 to M11, C).
//! One module per source family; each world is built from literals in the test.

#![allow(dead_code, reason = "each feature set uses part of the helpers")]

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticDetail, Diagnostics, Phase, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackOrigin, PackRepository, PackSelection, StackSet, WorldPacks};

use super::common::behavior_pack;

#[cfg(feature = "aim_assist")]
mod aim_assist;
mod audit;
#[cfg(feature = "behavior_tree")]
mod behavior_tree;
#[cfg(feature = "biome")]
mod biome;
#[cfg(feature = "dimension")]
mod dimension;
#[cfg(feature = "feature")]
mod feature;
#[cfg(all(feature = "feature_rules", feature = "feature"))]
mod feature_rules;
#[cfg(feature = "jigsaw")]
mod jigsaw;
#[cfg(feature = "poi")]
mod poi;
#[cfg(feature = "structure")]
mod structure;

/// What a link finding says: the code's name, the severity, the file, the JSON path of the
/// member, and the kind and the name of its reference detail as written (`("", "")` without
/// one).
pub type Said = (&'static str, Severity, String, String, &'static str, String);

/// [`Said`] of one finding. Every finding of the pass is of the link phase and names its pack.
#[track_caller]
pub fn said_of(finding: &Diagnostic) -> Said {
    assert_eq!(finding.phase, Phase::Link, "{finding:?}");
    assert!(finding.location.pack.is_some(), "{finding:?}");
    let (kind, id) = match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference { kind, id, .. }) => (*kind, id.clone()),
        _ => ("", String::new()),
    };
    (
        finding.code.id(),
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

/// [`Said`] of every finding of `findings` whose reference is of one of `kinds`, in order: a
/// test of one row looks at the kinds of its row, whatever other rules say of the same world.
#[track_caller]
pub fn said(findings: &Diagnostics, kinds: &[&str]) -> Vec<Said> {
    findings
        .iter()
        .map(said_of)
        .filter(|said| kinds.contains(&said.4))
        .collect()
}

/// One expected finding.
pub fn finding(
    code: &'static str,
    severity: Severity,
    file: &str,
    path: &str,
    kind: &'static str,
    id: &str,
) -> Said {
    (
        code,
        severity,
        file.to_owned(),
        path.to_owned(),
        kind,
        id.to_owned(),
    )
}

/// An `UnresolvedReference` warning.
pub fn unresolved_warning(file: &str, path: &str, kind: &'static str, id: &str) -> Said {
    finding(
        "core/unresolved_reference_worded",
        Severity::Warning,
        file,
        path,
        kind,
        id,
    )
}

/// An `UnresolvedReference` information: a reference of ours that the game's link pass has no
/// line for.
pub fn unresolved_info(file: &str, path: &str, kind: &'static str, id: &str) -> Said {
    finding(
        "core/unresolved_reference",
        Severity::Info,
        file,
        path,
        kind,
        id,
    )
}

/// An `UnresolvedReference` error: a reference the game's link pass holds and logs at the
/// content log's error level.
pub fn unresolved_error(file: &str, path: &str, kind: &'static str, id: &str) -> Said {
    finding(
        "core/unresolved_reference_worded",
        Severity::Error,
        file,
        path,
        kind,
        id,
    )
}

/// The texts of the findings of `findings` whose reference is of `kind`.
pub fn texts(findings: &Diagnostics, kind: &str) -> Vec<String> {
    findings
        .iter()
        .filter(|finding| {
            matches!(finding.detail.as_deref(), Some(DiagnosticDetail::Reference { kind: found, .. }) if *found == kind)
        })
        .map(|finding| finding.code.to_string())
        .collect()
}

/// A custom block `id` (format 1.26.20, no state).
pub fn block(id: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.20", "minecraft:block": {{"description": {{"identifier": "{id}"}}, "components": {{}}}}}}"#
    )
}

/// A custom entity `id`.
pub fn entity(id: &str) -> String {
    format!(
        r#"{{"format_version": "1.21.0", "minecraft:entity": {{"description": {{"identifier": "{id}", "is_spawnable": true, "is_summonable": true, "is_experimental": false}}, "components": {{}}}}}}"#
    )
}

/// A custom item `id`.
pub fn item(id: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.30", "minecraft:item": {{"description": {{"identifier": "{id}"}}, "components": {{}}}}}}"#
    )
}

/// The stacks of a world with two behaviour packs: `world` on top, a pack of the world
/// (`bp 1`), and below it `game`, a pack the game ships (`bp 2`, a base-game pack).
pub fn stacks_with_game(
    world: &[(&'static str, &str)],
    game: &[(&'static str, &str)],
    options: LoadOptions,
) -> StackSet {
    let packs = [
        (behavior_pack(1, world), PackOrigin::World),
        (behavior_pack(2, game), PackOrigin::Package),
    ];
    let behavior = packs
        .iter()
        .map(|(pack, _)| PackSelection {
            pack_id: pack.id().uuid,
            version: pack.id().version.clone(),
            subpack: None,
        })
        .collect();
    let mut repository = PackRepository::new();
    for (pack, origin) in packs {
        repository.add_pack(pack, origin);
    }
    repository.stacks(
        &WorldPacks {
            behavior,
            resource: Vec::new(),
        },
        options,
    )
}
