//! The link tests of the behaviour-pack content: block, item, loot, trade, recipe, catalogue and trim references (20 rows B, I, L, T, RC, M1 to M4).
//!
//! One file per source family. Every world is built in the test from literals; a rule is
//! tested on a target a pack defines, one the game has, one nothing defines, and on each
//! normalisation its row names.

#![allow(dead_code, reason = "each feature set uses part of the helpers")]

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticDetail, Diagnostics, Phase, Severity};

#[cfg(feature = "block")]
mod block;
#[cfg(all(feature = "item", feature = "crafting_catalog"))]
mod catalog;
#[cfg(feature = "item")]
mod item;
#[cfg(feature = "loot")]
mod loot;
#[cfg(feature = "recipe")]
mod recipe;
#[cfg(feature = "trade")]
mod trade;
#[cfg(all(
    feature = "loot",
    feature = "recipe",
    feature = "trade",
    feature = "item"
))]
mod vanilla;

/// What a finding says: the code's name, the severity, the file, the JSON path, and the kind
/// and name of the reference in its detail (empty without one).
pub type Said = (&'static str, Severity, String, String, &'static str, String);

/// [`Said`] of one finding. Every finding of the pass is of the link phase.
pub fn said_of(finding: &Diagnostic) -> Said {
    assert_eq!(finding.phase, Phase::Link, "{finding:#?}");
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

/// [`Said`] of every finding whose reference is of one of `kinds`, in order.
pub fn said(findings: &Diagnostics, kinds: &[&str]) -> Vec<Said> {
    findings
        .iter()
        .map(said_of)
        .filter(|said| kinds.contains(&said.4))
        .collect()
}

/// One expected finding, in the order of [`Said`].
pub fn one(
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

/// The vanilla text of a finding, as the game prints it; `None` for a finding of ours.
pub fn text(finding: &Diagnostic) -> Option<String> {
    use bedrock_addon::diagnostic::DiagnosticCode as Code;
    match &finding.code {
        Code::UnresolvedReference { text }
        | Code::InvalidReference { text }
        | Code::InvalidValue { text }
        | Code::NoEffect { text }
        | Code::MissingRequiredField { text }
        | Code::DocumentRejected { text } => text.as_ref().map(ToString::to_string),
        _ => None,
    }
}

/// The hint of a finding's reference.
pub fn hint(finding: &Diagnostic) -> Option<bedrock_addon::diagnostic::ReferenceHint> {
    match finding.detail.as_deref()? {
        DiagnosticDetail::Reference { hint, .. } => hint.clone(),
        _ => None,
    }
}

/// A custom block `id` of format 1.26.20 with the JSON members `components`.
pub fn block(id: &str, components: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.20", "minecraft:block": {{"description": {{"identifier": "{id}"}}, "components": {{{components}}}}}}}"#
    )
}

/// A custom item `id` of format 1.26.30 with the JSON members `components`.
pub fn item(id: &str, components: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.30", "minecraft:item": {{"description": {{"identifier": "{id}"}}, "components": {{{components}}}}}}}"#
    )
}

/// A custom entity `id`.
pub fn entity(id: &str) -> String {
    format!(
        r#"{{"format_version": "1.21.0", "minecraft:entity": {{"description": {{"identifier": "{id}", "is_spawnable": true, "is_summonable": true, "is_experimental": false}}, "components": {{}}}}}}"#
    )
}
