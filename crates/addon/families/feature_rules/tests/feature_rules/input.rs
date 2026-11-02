//! The input the game reads a rule from (`MinecraftDocumentInput`;
//! the slice's mask 0xfbf8): a member written as `null` where a value is read rejects the
//! document.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::document::Content;

use super::common::*;

/// The rule with `distribution` (a JSON object's text).
fn with_distribution(distribution: &str) -> String {
    rule("1.21.20", "surface_pass", Some(distribution))
}

/// `null` at the member ending in `tail` is an `InvalidType` error and the rule is rejected.
#[track_caller]
fn null_rejects(json: &str, tail: &str) {
    let d = rejects(json);
    assert!(
        with_code(&d, "InvalidType")
            .iter()
            .any(|f| f.severity == Severity::Error
                && f.location.json_path.to_string().ends_with(tail)),
        "no InvalidType at …{tail}: {d:#?}"
    );
}

#[test]
fn a_null_coordinate_eval_order_rejects_the_rule() {
    let json = with_distribution(&format!(
        r#"{{"iterations": 1, "coordinate_eval_order": null, "x": {UNIFORM}, "y": 0, "z": {UNIFORM}}}"#
    ));
    null_rejects(&json, "coordinate_eval_order");
    // Only the absent member takes the default `xzy`.
    let absent = with_distribution(&distribution("1", UNIFORM, "0", UNIFORM));
    let document = loads(&absent);
    let params = document
        .value()
        .distribution
        .as_ref()
        .expect("a distribution");
    assert_eq!(params.coordinate_eval_order, None);
}

#[test]
fn a_null_scatter_chance_rejects_the_rule() {
    let json = with_distribution(&format!(
        r#"{{"iterations": 1, "scatter_chance": null, "x": {UNIFORM}, "y": 0, "z": {UNIFORM}}}"#
    ));
    null_rejects(&json, "scatter_chance");
}

#[test]
fn a_null_member_of_a_coordinate_object_rejects_the_rule() {
    let json = with_distribution(&distribution(
        "1",
        r#"{"distribution": "fixed_grid", "extent": [0, 15], "step_size": null}"#,
        "0",
        UNIFORM,
    ));
    null_rejects(&json, "x/step_size");
}

#[test]
fn a_null_member_of_a_fraction_rejects_the_rule() {
    let json = with_distribution(&format!(
        r#"{{"iterations": 1, "scatter_chance": {{"numerator": null, "denominator": 2}}, "x": {UNIFORM}, "y": 0, "z": {UNIFORM}}}"#
    ));
    null_rejects(&json, "scatter_chance/numerator");
}

#[test]
fn a_null_biome_filter_rejects_the_rule() {
    let json = r#"{"format_version": "1.21.20", "minecraft:feature_rules": {"description": {"identifier": "demo:test", "places_feature": "demo:feature"}, "conditions": {"placement_pass": "surface_pass", "minecraft:biome_filter": null}}}"#;
    null_rejects(json, "minecraft:biome_filter");
}

#[test]
fn an_unknown_member_stays_a_warning() {
    // Mask 0xfbf8: an unknown member (0x4) is a warning, the rule kept.
    let json = with_distribution(&format!(
        r#"{{"iterations": 1, "x": {UNIFORM}, "y": 0, "z": {UNIFORM}, "spread": 2}}"#
    ));
    let document = loads(&json);
    let found = with_code(document.diagnostics(), "UnrecognizedField");
    assert_eq!(found.len(), 1, "{:#?}", document.diagnostics());
    assert_eq!(found[0].severity, Severity::Warning);
    assert!(
        found[0]
            .location
            .json_path
            .to_string()
            .ends_with("minecraft:feature_rules/distribution/spread"),
        "{:#?}",
        document.diagnostics()
    );
}

/// The 51 rules of the vanilla pack `experimental_y_2026_drop_4` of 1.26.60 (formats 1.26.40
/// and 1.26.50, copied under `drop4/`) load from a base-game pack with nothing
/// above information.
#[test]
fn the_drop_4_rules_load() {
    let mut cx = bedrock_addon_core::load::LoadContext::detached();
    cx.is_base_game_pack = true;
    let mut count = 0;
    let mut names: Vec<_> = std::fs::read_dir(fixtures().join("drop4"))
        .expect("the drop4 folder")
        .map(|entry| entry.expect("an entry").file_name())
        .collect();
    names.sort();
    for name in names {
        let name = name.to_string_lossy().into_owned();
        let document = loads_with(&fixture(&format!("drop4/{name}")), &cx);
        assert!(
            above_info(document.diagnostics()).is_empty(),
            "{name}: {:#?}",
            document.diagnostics()
        );
        count += 1;
    }
    assert_eq!(count, 51);
}

/// The style of the captures: one line, no spaces, no trailing newline.
const COMPACT: bedrock_addon_core::json::WriteStyle = bedrock_addon_core::json::WriteStyle {
    indent: bedrock_addon_core::json::Indent::Compact,
    trailing_newline: false,
};

/// The drop-4 rules whose `distribution` writes `scatter_chance` before `iterations`: the
/// writer puts the members in the struct's order (`iterations`, `scatter_chance`,
/// `coordinate_eval_order`, `x`, `y`, `z`), so these are written as the same JSON value, not
/// byte for byte.
const REORDERED: &[&str] = &[
    "overworld_underground_andesite_upper_feature.json",
    "overworld_underground_diorite_upper_feature.json",
    "overworld_underground_granite_upper_feature.json",
    "overworld_underground_intrusive_andesite_upper_feature.json",
    "overworld_underground_intrusive_diorite_upper_feature.json",
    "overworld_underground_intrusive_granite_upper_feature.json",
];

/// Every drop-4 rule round-trips: parse, write, parse the written text again, the same rule,
/// and the second write identical to the first; written in the captures' compact style, the
/// rule is the capture byte for byte.
#[test]
fn the_drop_4_rules_round_trip_to_the_capture() {
    let mut cx = bedrock_addon_core::load::LoadContext::detached();
    cx.is_base_game_pack = true;
    let mut names: Vec<_> = std::fs::read_dir(fixtures().join("drop4"))
        .expect("the drop4 folder")
        .map(|entry| entry.expect("an entry").file_name())
        .collect();
    names.sort();
    assert_eq!(names.len(), 51);
    let mut reordered: Vec<String> = Vec::new();
    for name in names {
        let name = name.to_string_lossy().into_owned();
        let capture = fixture(&format!("drop4/{name}"));
        let first = loads_with(&capture, &cx);
        let written = first.value().to_json().expect("written");
        let second = loads_with(&written, &cx);
        assert_eq!(first.value(), second.value(), "{name}: {written}");
        assert_eq!(
            second.value().to_json().expect("written again"),
            written,
            "{name}"
        );
        let mut compact = Vec::new();
        second
            .value()
            .write_json(&mut compact, COMPACT)
            .expect("written compact");
        let compact = String::from_utf8(compact).expect("UTF-8");
        if compact != capture {
            // Not byte for byte: the same JSON value, members in the struct's order.
            let as_written: bedrock_addon_core::json::Value =
                facet_json::from_str(&compact).expect("JSON");
            let as_captured: bedrock_addon_core::json::Value =
                facet_json::from_str(&capture).expect("JSON");
            assert_eq!(as_written, as_captured, "{name}");
            reordered.push(name);
        }
    }
    assert_eq!(reordered, REORDERED);
}

/// A coordinate written as a bare `null` (read from its own slice, `parse::typed`).
#[test]
fn a_null_coordinate_rejects_the_rule() {
    let json = with_distribution(&distribution("1", "null", "0", UNIFORM));
    null_rejects(&json, "minecraft:feature_rules/distribution/x");
}

/// `iterations` written as `null` (a Molang member of the payload shell).
#[test]
fn a_null_iterations_rejects_the_rule() {
    let json = with_distribution(&distribution("null", UNIFORM, "0", UNIFORM));
    null_rejects(&json, "minecraft:feature_rules/distribution/iterations");
}

/// `distribution` written as `null` (an optional object of the payload shell).
#[test]
fn a_null_distribution_rejects_the_rule() {
    let json = rule("1.21.20", "surface_pass", Some("null"));
    null_rejects(&json, "minecraft:feature_rules/distribution");
}
