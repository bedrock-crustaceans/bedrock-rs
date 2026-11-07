//! `Merged<T, L>`: a value folded from every pack's copy with the family's
//! merge strategy, the winning origin per entry and the layers top-first.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "tests read fixtures and call facet-json directly"
)]
use bedrock_addon_core::__private::Sealed;
use bedrock_addon_core::diagnostic::{
    Code, Diagnostic, Diagnostics, JsonPath, Location, Phase, Severity, codes,
};
use bedrock_addon_core::document::{
    Document, Merge, Merged, Origin, RawLayer, Rejected, VersionInfo,
};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::pack_identity::{PackIdVersion, PackPath, PackRef, PackSide, PackType};
use bedrock_addon_core::version::{FormatVersion, SemVersion};
use indexmap::IndexMap;
use uuid::Uuid;

/// A toy merged file: a translation table, later packs overriding keys.
#[derive(Clone, Debug, Default, PartialEq)]
struct Lang {
    entries: IndexMap<String, &'static str>,
}

impl Sealed for Lang {}

impl Merge for Lang {
    type Key = String;
}

fn pack(index: usize) -> PackRef {
    PackRef::new(
        index,
        PackIdVersion::new(
            Uuid::from_u128(index as u128 + 1),
            SemVersion::new(1, 0, 0),
            PackType::Resource,
        ),
        format!("pack {index}"),
    )
}

fn origin(pack_index: usize) -> Origin {
    Origin::new(
        pack(pack_index),
        PackSide::Resource,
        PackPath::from_static("texts/en_US.lang"),
        None,
        Vec::new(),
    )
}

fn layer(pack_index: usize, entries: &[(&str, &'static str)]) -> Document<Lang> {
    let v = FormatVersion::new(1, 0, 0);
    Document::new(
        Lang {
            entries: entries.iter().map(|(k, v)| ((*k).to_owned(), *v)).collect(),
        },
        origin(pack_index),
        VersionInfo::new(None, false, v, v, false),
        Diagnostics::default(),
    )
}

fn failed(pack_index: usize) -> Result<Document<Lang>, Rejected> {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(Diagnostic::new(
        Code::from(&codes::INVALID_TYPE),
        Severity::Error,
        Phase::Parse,
        Location::new(Some(pack(pack_index)), None, JsonPath::new(), None),
    ));
    Err(Rejected::new(origin(pack_index), diagnostics))
}

/// Bottom-up: later copies override keys, each key's origin is the last writer.
fn merge_lang(walk: Vec<Result<Document<Lang>, Rejected>>) -> Merged<Lang> {
    Merged::merge(walk, Lang::default(), |merged, copy, cx| {
        for (key, value) in &copy.entries {
            merged.entries.insert(key.clone(), value);
            cx.claim(key.clone());
        }
    })
}

#[test]
fn later_copies_override_and_each_entry_knows_its_winning_pack() {
    let merged = merge_lang(vec![
        Ok(layer(2, &[("a", "2a"), ("b", "2b")])),
        Ok(layer(1, &[("b", "1b"), ("c", "1c")])),
        Ok(layer(0, &[("c", "0c")])),
    ]);
    let entries: Vec<(&str, &str)> = merged
        .value()
        .entries
        .iter()
        .map(|(k, v)| (k.as_str(), *v))
        .collect();
    assert_eq!(entries, [("a", "2a"), ("b", "1b"), ("c", "0c")]);
    assert_eq!(
        merged.origin(&"a".to_owned()).map(|o| o.pack.clone()),
        Some(pack(2))
    );
    assert_eq!(
        merged.origin(&"b".to_owned()).map(|o| o.pack.clone()),
        Some(pack(1))
    );
    assert_eq!(
        merged.origin(&"c".to_owned()).map(|o| o.pack.clone()),
        Some(pack(0))
    );
    assert_eq!(merged.origin(&"z".to_owned()), None);
    // Deref reads through to the value.
    assert_eq!(merged.entries.len(), 3);
}

#[test]
fn layers_are_top_first() {
    let merged = merge_lang(vec![
        Ok(layer(2, &[])),
        Ok(layer(0, &[])),
        Ok(layer(1, &[])),
    ]);
    let packs: Vec<Option<usize>> = merged
        .layers()
        .iter()
        .map(|layer| layer.origin().pack.index())
        .collect();
    assert_eq!(packs, [Some(0), Some(1), Some(2)]);
}

#[test]
fn a_failed_copy_contributes_its_findings_only() {
    let merged = merge_lang(vec![Ok(layer(1, &[("a", "1a")])), failed(0)]);
    assert_eq!(merged.layers().len(), 1);
    assert_eq!(merged.entries.len(), 1);
    let codes: Vec<Code> = merged.diagnostics().map(|d| d.code.clone()).collect();
    assert_eq!(codes, [Code::from(&codes::INVALID_TYPE)]);
}

#[test]
fn a_strategy_can_release_an_entry_and_report() {
    let merged = Merged::merge(
        vec![Ok(layer(1, &[("a", "1a")])), Ok(layer(0, &[("a", "")]))],
        Lang::default(),
        |merged: &mut Lang, copy, cx| {
            for (key, value) in &copy.entries {
                if value.is_empty() {
                    merged.entries.shift_remove(key);
                    cx.release(key);
                    let location =
                        Location::new(Some(cx.origin().pack.clone()), None, JsonPath::new(), None);
                    cx.push_diagnostic(Diagnostic::new(
                        Code::from(&codes::SHADOWED_DEFINITION),
                        Severity::Info,
                        Phase::Validate,
                        location,
                    ));
                } else {
                    merged.entries.insert(key.clone(), value);
                    cx.claim(key.clone());
                }
            }
        },
    );
    assert!(merged.entries.is_empty());
    assert_eq!(merged.origin(&"a".to_owned()), None);
    assert_eq!(merged.diagnostics().count(), 1);
}

#[test]
fn raw_layers_merge_before_parsing() {
    let json = |text: &str| facet_json::from_str::<Value>(text).unwrap();
    let walk = vec![
        Ok(RawLayer::new(origin(1), json(r#"{"a": "1a"}"#))),
        Ok(RawLayer::new(origin(0), json(r#"{"b": "0b"}"#))),
    ];
    let merged: Merged<Lang, RawLayer> =
        Merged::merge(walk, Lang::default(), |merged, layer, cx| {
            let key = if layer.origin.pack.index() == Some(0) {
                "b"
            } else {
                "a"
            };
            merged.entries.insert(key.to_owned(), "raw");
            cx.claim(key.to_owned());
        });
    assert_eq!(merged.layers()[0].origin.pack, pack(0));
    assert_eq!(merged.layers()[1].json, json(r#"{"a": "1a"}"#));
    assert_eq!(
        merged.origin(&"a".to_owned()).map(|o| o.pack.clone()),
        Some(pack(1))
    );
}

#[test]
fn a_merged_value_from_its_parts() {
    let mut origins = IndexMap::new();
    origins.insert("a".to_owned(), origin(0));
    let merged: Merged<Lang> =
        Merged::new(Lang::default(), origins, Vec::new(), Diagnostics::default());
    assert_eq!(merged.origin(&"a".to_owned()), Some(&origin(0)));
    assert!(merged.layers().is_empty());
    assert_eq!(merged.diagnostics().count(), 0);
}
