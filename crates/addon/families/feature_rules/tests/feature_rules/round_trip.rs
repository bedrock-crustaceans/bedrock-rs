//! The round-trip at the Value level: a rule parsed, written and
//! parsed again is equal, and the second write is the first; unknown members are kept at their
//! level. The strongest fixtures are the retail 1.26.45 files, copied as shipped.

use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{RangeModel, Selection, select_versions};

use super::common::*;

/// Parses `text` at `path`, writes it, parses the output and writes again: the two values and
/// the two writes are equal. Returns the first write.
fn round_trip(path: &'static str, text: &str) -> String {
    let cx = LoadContext::detached();
    let first = load_at(path, text, &cx).unwrap_or_else(|r| panic!("{path}: {:#?}", r.diagnostics));
    let written = first.to_json().unwrap();
    let second =
        load_at(path, &written, &cx).unwrap_or_else(|r| panic!("{written}: {:#?}", r.diagnostics));
    assert_eq!(first.value(), second.value(), "{written}");
    assert_eq!(second.to_json().unwrap(), written);
    written
}

const SULFUR: &str =
    "feature_rules/sulfur_caves_before_surface_sulfur_spring_trail_to_surface_feature_rules.json";
const DAPPLED: &str = "feature_rules/scatter_tree_dappled_forest_feature_rules.json";

#[test]
fn r17_the_retail_1_26_10_and_1_26_30_files_load_clean_into_the_one_struct() {
    assert!(ENGINE_VERSION >= FormatVersion::new(1, 26, 30));
    for (path, version) in [
        (SULFUR, FormatVersion::new(1, 26, 10)),
        (DAPPLED, FormatVersion::new(1, 26, 30)),
    ] {
        let text = fixture(&format!("r17_1_26_x/{path}"));
        let document = load_at(path, &text, &LoadContext::detached())
            .unwrap_or_else(|r| panic!("{path}: {:#?}", r.diagnostics));
        assert!(
            above_info(document.diagnostics()).is_empty(),
            "{path}: {:#?}",
            document.diagnostics()
        );
        #[cfg(feature = "molang")]
        {
            let mut cx = pack(1, 21, 80);
            cx.molang = Some(std::sync::Arc::new(
                bedrock_addon_core::molang::MolangCheck::new(),
            ));
            let document = load_at(path, &text, &cx).unwrap();
            assert!(
                document.diagnostics().is_empty(),
                "{path}: {:#?}",
                document.diagnostics()
            );
        }
        assert_eq!(document.version().written, Some(version));
        assert_eq!(document.version().parsed_as, FormatVersion::new(1, 12, 0));
        assert!(!document.version().upgraded);
    }
}

#[test]
fn r17_with_an_engine_of_1_26_20_the_1_26_30_file_selects_no_parser() {
    let versions = [FormatVersion::new(1, 12, 0)];
    let at = |version| FormatVersionInput {
        version: Some(version),
        beta_literal: false,
        prerelease: false,
    };
    let engine = Some(FormatVersion::new(1, 26, 20));
    assert_eq!(
        select_versions(
            &versions,
            FormatVersion::new(1, 12, 0),
            RangeModel::ClosedToEngine,
            engine,
            &at(FormatVersion::new(1, 26, 30))
        ),
        Selection::Unsupported
    );
    assert_eq!(
        select_versions(
            &versions,
            FormatVersion::new(1, 12, 0),
            RangeModel::ClosedToEngine,
            engine,
            &at(FormatVersion::new(1, 26, 10))
        ),
        Selection::Struct(0)
    );
}

#[test]
fn a_rule_above_the_engine_is_unsupported_with_the_puv_text() {
    let d = rejects(&vanilla_input("without_distribution", "1.27.0"));
    let found = with_code(&d, "core/no_parser_for_version");
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(
        found[0].code.to_string(),
        "unsupported version: no parser available for version 1.27.0"
    );
}

#[test]
fn the_retail_files_round_trip() {
    for path in [SULFUR, DAPPLED] {
        let written = round_trip(path, &fixture(&format!("r17_1_26_x/{path}")));
        assert!(written.contains(r#""format_version": "1.26."#), "{written}");
    }
}

#[test]
fn every_form_round_trips_with_unknown_members_at_their_level() {
    let text = rule(
        "1.13.0",
        "surface_pass",
        Some(r#"{"iterations": {"expression": "2", "version": 0}, "scatter_chance": {"numerator": 1, "denominator": 3, "a": 1}, "coordinate_eval_order": "yxz", "x": {"distribution": "fixed_grid", "extent": ["v.a", 15], "step_size": 2, "grid_offset": 0, "b": 2}, "y": true, "z": "", "c": 3}"#),
    )
    .replace(r#""placement_pass": "surface_pass""#, r#""placement_pass": "surface_pass", "minecraft:biome_filter": {"all_of": [{"test": "has_biome_tag", "value": "a"}]}, "d": 4"#)
    .replace(r#""places_feature": "demo:feature""#, r#""places_feature": "demo:feature", "e": 5"#)
    .replace(r#""conditions""#, r#""f": 6, "conditions""#);
    let written = round_trip("feature_rules/test.json", &text);
    for member in [
        r#""a": 1"#,
        r#""b": 2"#,
        r#""c": 3"#,
        r#""d": 4"#,
        r#""e": 5"#,
        r#""f": 6"#,
    ] {
        assert!(written.contains(member), "{member}: {written}");
    }
}

#[test]
fn a_rule_without_distribution_writes_none() {
    let written = round_trip(
        "feature_rules/custom_feature_rule.json",
        &vanilla_input("without_distribution", "1.21.20"),
    );
    assert!(!written.contains("distribution"), "{written}");
    assert!(
        written.contains(r#""format_version": "1.21.20""#),
        "{written}"
    );
}
