//! Block descriptor `tags` in spawn groups: the fixture rows `sg50`–`sg52`.

#![cfg(feature = "molang")]

use std::sync::Arc;

use bedrock_addon::diagnostic::{DiagnosticDetail, Diagnostics, Severity};
use bedrock_addon::document::{Content, Document};
use bedrock_addon::families::spawn_groups::SpawnGroup;
use bedrock_addon::load::LoadContext;
use bedrock_addon::molang::{DiagCode, MolangCheck, MolangVersion, RawVersion};
use bedrock_addon::shared::BlockDescriptor;
use bedrock_addon::storage::PackPath;

use crate::common::context;

/// A context that compiles Molang.
fn checking() -> LoadContext {
    let mut cx = context();
    cx.molang = Some(Arc::new(MolangCheck::new()));
    cx
}

/// Loads a 1.8.0 group whose one condition is `{ body }`.
fn load(body: &str) -> Document<SpawnGroup> {
    let text = format!(
        r#"{{"format_version": "1.8.0", "minecraft:spawn_groups": {{"description": {{"identifier": "test:g"}}, "conditions": [{{ {body} }}]}}}}"#
    );
    SpawnGroup::parse(
        &PackPath::from_static("definitions/spawn_groups/x.json"),
        text.as_bytes(),
        &checking(),
    )
    .unwrap_or_else(|r| panic!("rejected: {:#?}", r.diagnostics))
}

/// The compiler codes of the Molang findings, with their severities.
fn molang(d: &Diagnostics) -> Vec<(DiagCode, Severity)> {
    d.iter()
        .filter_map(|f| match f.detail.as_deref() {
            Some(DiagnosticDetail::Molang(m)) => Some((m.code(), f.severity)),
            _ => None,
        })
        .collect()
}

#[test]
fn sg50_tags_compile() {
    let document = load(r#""minecraft:spawns_on_block_filter": {"tags": "q.any_tag('grass')"}"#);
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    let condition = &document.conditions.as_ref().unwrap()[0];
    let list = condition.spawns_on_block_filter.as_ref().unwrap();
    let BlockDescriptor::Tags { tags, .. } = &list.as_slice()[0] else {
        panic!("not tags");
    };
    assert_eq!(
        tags.raw_version(),
        Some(RawVersion::from(MolangVersion::V0))
    );
}

#[test]
fn sg51_tags_default_query() {
    let document = load(r#""minecraft:spawns_on_block_filter": {"tags": "q.is_baby"}"#);
    let found = molang(document.diagnostics());
    assert!(
        found.contains(&(DiagCode::UnknownQuery, Severity::Error)),
        "{found:?}"
    );
    assert!(
        found.contains(&(DiagCode::Syntax, Severity::Error)),
        "{found:?}"
    );
    assert!(!document.diagnostics().has_errors());
    assert_eq!(document.spawn_rules()[0].spawns_on_blocks.len(), 1);
}

#[test]
fn sg52_tags_shape_then_molang() {
    let document = load(
        r#""minecraft:spawns_above_block_filter": {"blocks": [5, {"tags": "1 +"}], "distance": 2}"#,
    );
    let d = document.diagnostics();
    assert!(
        d.iter()
            // By legacy name: the variant and the definitions that were it.
            .any(|f| f.code.id() == "shapes/block_descriptor_shape"
                && f.code.to_string().ends_with("Cannot parse 5")),
        "{d:#?}"
    );
    assert!(
        molang(d)
            .iter()
            .any(|(c, s)| *c == DiagCode::StatementForm && *s == Severity::Error),
        "{d:#?}"
    );
    assert!(!d.has_errors());
    let rules = &document.spawn_rules()[0];
    assert_eq!(
        (rules.spawns_above_blocks.len(), rules.above_block_distance),
        (1, 2)
    );
}
