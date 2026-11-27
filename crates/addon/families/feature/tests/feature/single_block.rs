//! `single_block_feature`: the 1.21.40 schema, its step and the constructor defaults (fixtures
//! f04–f06, f25).

use bedrock_addon_family_feature::{Feature, PlacesBlock, SingleBlockFeature};
use bedrock_addon_shared_shapes::BlockDescriptor;

use super::common::{cx, doc, error_at, loads, rejects};

/// The single block feature of a loaded document.
#[track_caller]
fn single_block(json: &str) -> SingleBlockFeature {
    match loads(json, &cx()).into_value() {
        Feature::SingleBlock(feature) => feature,
        other => panic!("not a single block feature: {other:?}"),
    }
}

const ENFORCE: &str = r#""enforce_placement_rules": true, "enforce_survivability_rules": false"#;

#[test]
fn f04_the_1_21_40_members_are_unknown_in_a_1_21_30_document() {
    for (member, tail) in [
        (r#""randomize_rotation": true"#, "randomize_rotation"),
        (r#""may_not_attach_to": {}"#, "may_not_attach_to"),
        (
            r#""may_attach_to": {"diagonal": "minecraft:stone"}"#,
            "may_attach_to/diagonal",
        ),
    ] {
        let json = doc(
            "1.21.30",
            "single_block_feature",
            &format!("{ENFORCE}, {member}"),
        );
        let d = rejects(&json, &cx());
        error_at(&d, "core/child_not_valid", tail);
    }
}

#[test]
fn f04_a_weighted_places_block_is_the_wrong_type_in_a_1_21_30_document() {
    let json = doc(
        "1.21.30",
        "single_block_feature",
        &format!(r#"{ENFORCE}, "places_block": [{{"block": "minecraft:stone", "weight": 1}}]"#),
    );
    let d = rejects(&json, &cx());
    error_at(&d, "core/invalid_type", "places_block");
}

#[test]
fn f04b_enforce_placement_rules_is_required() {
    let json = doc(
        "1.21.40",
        "single_block_feature",
        r#""enforce_survivability_rules": true, "places_block": "minecraft:stone""#,
    );
    let d = rejects(&json, &cx());
    error_at(&d, "core/required_child", "enforce_placement_rules");
}

#[test]
fn f04b_places_block_is_not_required() {
    let feature = single_block(&doc("1.21.40", "single_block_feature", ENFORCE));
    assert_eq!(feature.places_block, None);
    let feature = single_block(&doc("1.13.0", "single_block_feature", ENFORCE));
    assert_eq!(feature.places_block, None);
}

#[test]
fn f04b_min_sides_must_attach_above_4_is_invalid() {
    let json = doc(
        "1.21.40",
        "single_block_feature",
        &format!(r#"{ENFORCE}, "may_attach_to": {{"min_sides_must_attach": 5}}"#),
    );
    let d = rejects(&json, &cx());
    error_at(
        &d,
        "core/outside_range",
        "may_attach_to/min_sides_must_attach",
    );
}

#[test]
fn f05_the_step_keeps_a_tags_descriptor_and_adds_no_members() {
    let json = doc(
        "1.21.30",
        "single_block_feature",
        &format!(
            r#"{ENFORCE}, "places_block": {{"tags": "q.any_tag('stone')"}}, "may_attach_to": {{"top": "minecraft:stone"}}"#
        ),
    );
    let feature = single_block(&json);
    let Some(PlacesBlock::Descriptor(BlockDescriptor::Tags { tags, .. })) = &feature.places_block
    else {
        panic!("not a tags descriptor: {:?}", feature.places_block);
    };
    assert_eq!(tags.as_str(), "q.any_tag('stone')");
    let attach = feature.may_attach_to.as_ref().unwrap();
    // `diagonal: []` in the document: no diagonal neighbours, written as an absent member.
    assert_eq!(attach.diagonal, None);
    assert_eq!(feature.may_not_attach_to, None);
    assert_eq!(feature.randomize_rotation, None);
    assert!(!feature.randomizes_rotation());
}

#[test]
fn f06_weighted_and_specifier_forms_parse_at_1_21_40() {
    let weighted = single_block(&doc(
        "1.21.40",
        "single_block_feature",
        &format!(
            r#"{ENFORCE}, "places_block": [{{"block": "minecraft:stone", "weight": 2.5}}, {{"block": {{"name": "minecraft:dirt"}}, "weight": 1}}]"#
        ),
    ));
    let Some(PlacesBlock::Weighted(entries)) = &weighted.places_block else {
        panic!("not weighted");
    };
    assert_eq!(entries.len(), 2);
    assert!((entries[0].weight - 2.5).abs() < f32::EPSILON);
    assert_eq!(entries[1].block.name.as_str(), "minecraft:dirt");
    let specifier = single_block(&doc(
        "1.21.40",
        "single_block_feature",
        &format!(
            r#"{ENFORCE}, "places_block": {{"name": "minecraft:stone", "states": {{"stone_type": "granite"}}}}"#
        ),
    ));
    let Some(PlacesBlock::Specifier(block)) = &specifier.places_block else {
        panic!("not a specifier");
    };
    assert_eq!(block.name.as_str(), "minecraft:stone");
}

#[test]
fn f06_a_tags_places_block_is_rejected_at_1_21_40() {
    let json = doc(
        "1.21.40",
        "single_block_feature",
        &format!(r#"{ENFORCE}, "places_block": {{"tags": "q.any_tag('stone')"}}"#),
    );
    let d = rejects(&json, &cx());
    // A specifier has no tag form; the option matches no shape.
    error_at(&d, "core/invalid_type", "places_block");
}

#[test]
fn f25_an_empty_may_attach_to_has_the_constructor_defaults() {
    let feature = single_block(&doc(
        "1.21.40",
        "single_block_feature",
        &format!(r#"{ENFORCE}, "may_attach_to": {{}}"#),
    ));
    let attach = feature.may_attach_to.as_ref().unwrap();
    assert_eq!(attach.min_sides(), 4);
    assert!(attach.rotates());
    assert!(!feature.randomizes_rotation());
}

/// a descriptor keeps a state's value as written; a number no double holds is the
/// largest finite double there, with a warning at the number, and the feature is written.
#[test]
fn a_state_value_no_double_holds_is_the_largest_finite_double_and_the_feature_is_written() {
    for (number, held) in [("1e999", f64::MAX), ("-1e999", f64::MIN)] {
        let json = doc(
            "1.13.0",
            "single_block_feature",
            &format!(
                r#""places_block": {{"name": "minecraft:cocoa", "states": {{"age": {number}}}}}, {ENFORCE}"#
            ),
        );
        let document = loads(&json, &cx());
        let at = "minecraft:single_block_feature/places_block/states/age";
        let told: Vec<_> = document
            .diagnostics()
            .iter()
            .filter(|f| f.location.json_path.to_string() == at)
            .map(|f| (f.code.id(), f.severity))
            .collect();
        assert_eq!(
            told,
            [(
                "core/invalid_value",
                bedrock_addon_core::diagnostic::Severity::Info
            )],
            "{number}"
        );
        let written = document.to_json().expect("a feature that loads is written");
        assert!(
            written.contains(if held > 0.0 {
                "1.7976931348623157e308"
            } else {
                "-1.7976931348623157e308"
            }),
            "{written}"
        );
        let again = loads(&written, &cx());
        assert_eq!(again.to_json().ok(), Some(written), "{number}");
    }
    // The largest double itself is held without the warning of the clamp.
    let json = doc(
        "1.13.0",
        "single_block_feature",
        &format!(
            r#""places_block": {{"name": "minecraft:cocoa", "states": {{"age": 1.7976931348623157e308}}}}, {ENFORCE}"#
        ),
    );
    let document = loads(&json, &cx());
    assert!(
        document.diagnostics().iter().all(|f| !f
            .location
            .json_path
            .to_string()
            .ends_with("states/age")),
        "{:?}",
        document.diagnostics()
    );
}

/// Features are read through `JsonCppInput`, where a
/// member written as `null` is absent: a required one is missing, not of the wrong type.
#[test]
fn a_null_required_member_is_missing() {
    let json = doc(
        "1.21.40",
        "single_block_feature",
        r#""enforce_placement_rules": null, "enforce_survivability_rules": true, "places_block": "minecraft:stone""#,
    );
    let d = rejects(&json, &cx());
    error_at(&d, "core/required_child", "enforce_placement_rules");
}

/// `early_out_scheme: null` reads as the member left out (the
/// game's default for both is `none`).
#[test]
fn a_null_early_out_scheme_reads_as_an_absent_one() {
    let list = |tail: &str| {
        let json = doc(
            "1.13.0",
            "conditional_list",
            &format!(
                r#""conditional_features": [{{"places_feature": "demo:a", "condition": "1"}}]{tail}"#
            ),
        );
        match super::common::loads(&json, &super::common::base_game()).into_value() {
            Feature::ConditionalList(feature) => feature,
            other => panic!("not a conditional list: {other:?}"),
        }
    };
    let null = list(r#", "early_out_scheme": null"#);
    assert_eq!(null, list(""));
    assert_eq!(null.early_out_scheme, None);
    // Absent and `null` both mean the constructor's `none`.
    assert_eq!(
        null.early_out_scheme.unwrap_or_default(),
        bedrock_addon_family_feature::EarlyOutScheme::None
    );
}

/// Declared by the family (`InputKind::JsonCpp`): an optional member written
/// as `null` loads as absent, and the information that marks it for the write-back is told
/// once, at the member.
#[test]
fn a_null_optional_member_is_told_once() {
    let json = doc(
        "1.21.40",
        "single_block_feature",
        &format!(r#""places_block": "minecraft:stone", {ENFORCE}, "randomize_rotation": null"#),
    );
    let document = loads(&json, &cx());
    let told: Vec<_> = document
        .diagnostics()
        .iter()
        .filter(|f| f.code.id() == "core/value_rewritten")
        .collect();
    assert_eq!(told.len(), 1, "{:#?}", document.diagnostics());
    assert!(
        told[0]
            .location
            .json_path
            .to_string()
            .ends_with("randomize_rotation")
    );
    match document.into_value() {
        Feature::SingleBlock(feature) => assert_eq!(feature.randomize_rotation, None),
        other => panic!("not a single block feature: {other:?}"),
    }
}
