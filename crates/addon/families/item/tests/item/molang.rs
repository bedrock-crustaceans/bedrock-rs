//! Molang in items: `repair_amount`, descriptor `tags`. The version of every item expression is the item's
//! `format_version` window; block descriptor `tags` compile at a constant 13.

#[cfg(feature = "molang")]
use bedrock_addon_core::molang::MolangSource;
use bedrock_addon_core::molang::{Molang, MolangVersion, RawVersion};
use bedrock_addon_family_item::component::{Digger, Repairable};
use bedrock_addon_shared_shapes::BlockDescriptor;

use super::common::*;

/// The `repair_amount` of the first repair entry.
fn repair_amount(
    json: &str,
    cx: &bedrock_addon_core::load::LoadContext,
) -> (Molang, bedrock_addon_core::diagnostic::Diagnostics) {
    let document = loads(json, cx);
    let repairable = document.components.get::<Repairable>().unwrap();
    (
        repairable.repair_items[0].repair_amount.clone().unwrap(),
        document.diagnostics().clone(),
    )
}

/// A 1.20.50 item repaired by a stick with `amount` (JSON text).
fn repairable(amount: &str) -> String {
    item(
        "1.20.50",
        "demo:x",
        &format!(
            r#""minecraft:repairable": {{"repair_items": [{{"items": ["minecraft:stick"], "repair_amount": {amount}}}]}}"#
        ),
    )
}

#[test]
fn a_number_is_a_constant_and_is_not_compiled() {
    let (amount, d) = repair_amount(&repairable("25"), &third_party());
    assert!(matches!(amount, Molang::Const(v) if (v - 25.0).abs() < f32::EPSILON));
    assert!(d.is_empty(), "{d:#?}");
}

#[cfg(feature = "molang")]
mod compiled {
    use std::sync::Arc;

    use bedrock_addon_core::diagnostic::{DiagnosticDetail, Diagnostics, Severity};
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::molang::{DiagCode, MolangCheck, MolangVersion, RawVersion};
    use bedrock_addon_shared_shapes::ItemDescriptor;

    use super::*;

    /// A third-party context that compiles Molang, its pack at Molang 0 (`[1, 16, 0]`).
    fn checking() -> LoadContext {
        let mut cx = third_party();
        cx.molang = Some(Arc::new(MolangCheck::new()));
        cx.molang_version = RawVersion::from(MolangVersion::V0);
        cx
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
    fn a_string_follows_the_format_version_window_not_the_pack() {
        let (_, d) = repair_amount(&repairable(r#""'a' + 1""#), &checking());
        let found = molang(&d);
        assert!(
            found.iter().any(|(c, _)| *c == DiagCode::StringMisuse),
            "{d:#?}"
        );
        assert!(
            d.iter()
                // By legacy name: the variant and the definitions that were it.
                .any(|f| matches!(f.code.id(), "core/invalid_molang"))
        );
        let fine = r#""context.other->query.remaining_durability + 0.05 * context.other->query.max_durability""#;
        let (_, d) = repair_amount(&repairable(fine), &checking());
        assert!(d.is_empty(), "{d:#?}");
    }

    #[test]
    fn a_rejected_expression_keeps_the_item_deviation() {
        // the parser logs `_contentLog(true, 3, 22)` and the item loads: a kept error.
        let (_, d) = repair_amount(&repairable(r#""'a' + 1""#), &checking());
        let finding = d
            .iter()
            // By legacy name: the variant and the definitions that were it.
            .find(|f| matches!(f.code.id(), "core/invalid_molang"))
            .unwrap();
        assert_eq!((finding.severity, finding.kept), (Severity::Error, true));
    }

    #[test]
    fn an_object_form_keeps_its_own_version() {
        let (amount, d) = repair_amount(
            &repairable(r#"{"expression": "'a' + 1", "version": 2}"#),
            &checking(),
        );
        assert!(d.is_empty(), "{d:#?}");
        let Molang::Expr(source) = amount else {
            panic!("not an expression");
        };
        assert_eq!(source.raw_version(), Some(RawVersion(2)));
    }

    #[test]
    fn out_of_range_object_versions() {
        let low = r#"{"expression": "query.max_durability * 0.25", "version": -1}"#;
        let (_, d) = repair_amount(&repairable(low), &checking());
        let found = molang(&d);
        assert!(
            found.contains(&(DiagCode::InvalidVersion, Severity::Info)),
            "{found:?}"
        );
        assert!(
            found
                .iter()
                .any(|(c, s)| *c == DiagCode::QueryDeprecated && *s == Severity::Info),
            "{found:?}"
        );
        assert!(
            found.iter().any(|(c, _)| *c == DiagCode::UnknownQuery),
            "{found:?}"
        );
        assert!(
            found.iter().any(|(c, _)| *c == DiagCode::Syntax),
            "{found:?}"
        );
        let high = r#"{"expression": "query.max_durability * 0.25", "version": 14}"#;
        let (_, d) = repair_amount(&repairable(high), &checking());
        let found = molang(&d);
        assert!(
            // molang's own version lint, an information
            found.contains(&(DiagCode::InvalidVersion, Severity::Info)),
            "{found:?}"
        );
        assert!(found.iter().any(|(_, s)| *s == Severity::Info), "{found:?}");
        assert!(
            found.iter().any(|(c, _)| *c == DiagCode::UnknownQuery),
            "{found:?}"
        );
        assert!(
            found.iter().any(|(c, _)| *c == DiagCode::Syntax),
            "{found:?}"
        );
    }

    #[test]
    fn descriptor_tags() {
        let tags = |expression: &str| {
            item(
                "1.20.50",
                "demo:x",
                &format!(
                    r#""minecraft:repairable": {{"repair_items": [{{"items": [{{"tags": "{expression}"}}], "repair_amount": 1}}]}}"#
                ),
            )
        };
        let document = loads(&tags("query.any_tag('planks')"), &checking());
        assert!(
            document.diagnostics().is_empty(),
            "{:#?}",
            document.diagnostics()
        );
        let repairable = document.components.get::<Repairable>().unwrap();
        let ItemDescriptor::Tags { tags: source, .. } = &repairable.repair_items[0].items[0] else {
            panic!("not a tags descriptor");
        };
        assert_eq!(
            source.raw_version(),
            Some(RawVersion::from(MolangVersion::V11))
        );
        let document = loads(&tags("query.is_baby"), &checking());
        let found = molang(document.diagnostics());
        assert!(
            found.iter().any(|(c, _)| *c == DiagCode::UnknownQuery),
            "{found:?}"
        );
        assert!(
            found.iter().any(|(c, _)| *c == DiagCode::Syntax),
            "{found:?}"
        );
        let repairable = document.components.get::<Repairable>().unwrap();
        assert!(matches!(
            repairable.repair_items[0].items[0],
            ItemDescriptor::Tags { .. }
        ));
    }

    /// The expressions of a `repair_items` entry of a file below 1.19.83, which a step
    /// types after the file was read, have the item's version like those of a newer file:
    /// the compile pass is given no expression without one.
    #[test]
    fn the_expressions_of_a_legacy_repair_entry_have_the_items_version() {
        let components = r#""minecraft:repairable": {"repair_items": [{"items": [{"tags": "query.any_tag('planks')"}], "repair_amount": "query.max_durability * 0.25"}]}"#;
        let versions = |version: &str, cx: &LoadContext| {
            let document = loads(&item(version, "demo:x", components), cx);
            let repairable = document.components.get::<Repairable>().unwrap();
            let ItemDescriptor::Tags { tags, .. } = &repairable.repair_items[0].items[0] else {
                panic!("not a tags descriptor");
            };
            let amount = repairable.repair_items[0].repair_amount.as_ref().unwrap();
            (
                tags.raw_version(),
                amount.source().and_then(MolangSource::raw_version),
            )
        };
        // Read without the compile pass, then with it.
        let mut checking = allowlisted();
        checking.molang = Some(Arc::new(MolangCheck::new()));
        checking.molang_version = RawVersion::from(MolangVersion::V0);
        for cx in [allowlisted(), checking] {
            // From 1.19.83 the newest type reads the list and the reading gives the
            // versions; below it a step types the list.
            let mut last = RawVersion(0);
            for version in ["1.16.100", "1.17.0", "1.18.0", "1.19.0", "1.19.83"] {
                let (tags, amount) = versions(version, &cx);
                assert!(tags.is_some(), "{version}: `tags` has no version");
                assert_eq!(tags, amount, "{version}");
                // The version of the file's format, which grows with it.
                let raw = tags.unwrap();
                assert!(raw.0 >= last.0, "{version}: {raw:?} after {last:?}");
                last = raw;
            }
            assert_eq!(last, RawVersion(7));
        }
    }

    /// What the load tells of a legacy `repair_items` entry behind entries the step that
    /// types the list leaves out (a rejected expression) is at the
    /// index the file has the entry at.
    #[test]
    fn a_legacy_repair_entry_the_slice_cannot_read_refuses_the_item_at_the_index_of_the_file() {
        let mut cx = allowlisted();
        cx.molang = Some(Arc::new(MolangCheck::new()));
        cx.molang_version = RawVersion::from(MolangVersion::V0);
        let json = item(
            "1.16.100",
            "demo:x",
            r#""minecraft:repairable": {"repair_items": [
                7,
                {"repair_amount": 1},
                {"items": [{"tags": "query.is_baby"}], "repair_amount": "1 +"},
                {"a": 1},
                {"items": ["minecraft:stick", {"tags": "query.is_baby"}], "repair_amount": 1}]}"#,
        );
        // An entry the 1.19.83 slice cannot read (`items` required; a number)
        // refuses the item, each error at the index the file has the entry at;
        // the Molang pass does not run on a refused document.
        let d = rejects(&json, &cx);
        let list = "minecraft:item/components/minecraft:repairable/repair_items/";
        let mut told: Vec<(String, &'static str, Severity)> = d
            .iter()
            .filter_map(|finding| {
                let path = finding.location.json_path.to_string();
                Some((
                    path.strip_prefix(list)?.to_owned(),
                    finding.code.id(),
                    finding.severity,
                ))
            })
            .collect();
        told.sort();
        told.dedup();
        assert_eq!(
            told,
            [
                ("0".to_owned(), "core/invalid_value", Severity::Error),
                ("1".to_owned(), "core/invalid_value", Severity::Error),
                ("3".to_owned(), "core/invalid_value", Severity::Error),
            ],
            "{:#?}",
            d.iter().map(ToString::to_string).collect::<Vec<_>>()
        );
    }
}

#[test]
fn block_descriptor_tags_are_version_13_whatever_the_format_version() {
    for version in ["1.20.50", "1.21.0"] {
        let json = item(
            version,
            "minecraft:x",
            r#""minecraft:digger": {"destroy_speeds": [{"block": {"tags": "query.any_tag('stone')"}, "speed": 4}]}"#,
        );
        let document = loads(&json, &allowlisted());
        let digger = document.components.get::<Digger>().unwrap();
        let BlockDescriptor::Tags { tags, .. } = &digger.destroy_speeds[0].block else {
            panic!("not a tags descriptor");
        };
        assert_eq!(
            tags.raw_version(),
            Some(RawVersion::from(MolangVersion::V13)),
            "{version}"
        );
    }
}

#[cfg(not(feature = "molang"))]
#[test]
fn without_the_molang_feature_expressions_are_not_checked() {
    let (_, d) = repair_amount(&repairable(r#""'a' + 1""#), &third_party());
    assert!(
        d.iter().any(|f| f.code.id() == "core/molang_not_checked"),
        "{d:#?}"
    );
}

#[cfg(feature = "molang")]
#[test]
fn without_a_compile_check_expressions_are_not_checked() {
    let (_, d) = repair_amount(&repairable(r#""'a' + 1""#), &third_party());
    assert!(
        d.iter().any(|f| f.code.id() == "core/molang_not_checked"),
        "{d:#?}"
    );
}
