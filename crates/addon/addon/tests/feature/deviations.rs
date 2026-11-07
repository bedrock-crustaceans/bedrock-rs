//! Every deliberate difference from vanilla, pinned (the `vanilla =` notes of the fixtures and the
//! rules marked as ours).

use bedrock_addon::diagnostic::{Severity, codes};

use super::common::{base_game, cx, doc, engine, error_at, kept_at, loads, rejects, with};

#[test]
fn an_unparsed_carver_float_range_refuses_the_feature() {
    // Vanilla: a level-3 line, then the `FloatRange` node's `_validate` returns
    // false and the schema fails.
    let d = rejects(
        &doc(
            "1.13.0",
            "cave_carver_feature",
            r#""y_scale": [0.1, 0.5, 0.9]"#,
        ),
        &cx(),
    );
    error_at(&d, "InvalidValue", "y_scale");
    assert!(super::common::has_text(
        &d,
        "Float range array was not parsed. Float range arrays should have 2 values."
    ));
}

#[test]
fn no_height_difference_is_a_kept_error() {
    // Vanilla: a content-log error at level 3, the feature registered: a kept error
    let document = loads(
        &doc(
            "1.26.30",
            "height_difference_filter_feature",
            r#""places_feature": "demo:y", "search_radius": 1"#,
        ),
        &cx(),
    );
    assert_eq!(
        with(
            document.diagnostics(),
            "MissingRequiredField",
            Severity::Error
        )
        .len(),
        1
    );
    assert!(!document.diagnostics().has_errors());
}

#[test]
fn scatter_runtime_checks_are_kept_errors() {
    // Vanilla: content-log errors, the load continues with the replacement values: kept errors
    let document = loads(
        &doc(
            "1.21.10",
            "scatter_feature",
            r#""places_feature": "demo:y", "distribution": {"iterations": -1, "x": 0, "y": 0, "z": 0}"#,
        ),
        &cx(),
    );
    assert!(!document.diagnostics().has_errors());
    kept_at(document.diagnostics(), "ScatterParamValue", "iterations");
}

#[test]
fn a_duplicate_across_packs_is_an_information_deviation() {
    // Vanilla drops the lower pack's definition silently; ours: a
    // DuplicateIdentifier information without text.
    use bedrock_addon::load::LoadOptions;
    use bedrock_addon::pack::{Behavior, PackStack};
    const ORE: &[u8] = br#"{"format_version": "1.13.0", "minecraft:ore_feature": {"description": {"identifier": "demo:o"}, "count": 1}}"#;
    let top = super::pack_common::bp(1)
        .file("features/o.json", ORE)
        .open();
    let bottom = super::pack_common::bp(2)
        .file("features/o.json", ORE)
        .open();
    let stack = PackStack::<Behavior>::of([top, bottom], LoadOptions::default()).unwrap();
    assert!(
        stack
            .features()
            .diagnostics()
            .any(|f| f.severity == Severity::Info && f.is(&codes::DUPLICATE_IDENTIFIER))
    );
    assert!(
        stack
            .features()
            .diagnostics()
            .all(|f| f.severity < Severity::Warning),
        "{:#?}",
        stack.features().diagnostics().collect::<Vec<_>>()
    );
}

#[test]
fn poplar_parts_are_gated_at_engine_1_26_40_deviation() {
    // Vanilla's builder has no gate; ours: `since_engine` 1.26.40.
    let json = doc(
        "1.26.20",
        "tree_feature",
        r#""poplar_trunk": {"trunk_height": 8, "trunk_block": "minecraft:poplar_log"}"#,
    );
    error_at(
        &rejects(&json, &engine(1, 26, 30)),
        "UnrecognizedField",
        "poplar_trunk",
    );
    loads(&json, &engine(1, 26, 40));
}

#[test]
fn a_bounding_box_setter_message_is_a_kept_error() {
    // The setter logs its message at level 3 (`_contentLog(1,3,17)`) and the value
    // is kept: a kept error.
    let document = loads(
        &doc(
            "1.13.0",
            "beards_and_shavers",
            r#""places_feature": "demo:a", "bounding_box_min": [0, 0, 0], "bounding_box_max": [0, 4, 8], "y_delta": 0, "surface_block_type": "minecraft:dirt", "subsurface_block_type": "minecraft:dirt""#,
        ),
        &base_game(),
    );
    kept_at(document.diagnostics(), "InvalidValue", "bounding_box_max");
}

#[test]
fn an_unknown_setter_name_rejects_the_file_deviation() {
    // A name the setter does not know has no value to keep: ours rejects the file with the
    // setter's text.
    let d = rejects(
        &doc(
            "1.13.0",
            "growing_plant_feature",
            r#""height_distribution": [[1, 1]], "growth_direction": "SIDEWAYS", "body_blocks": [["minecraft:vine", 1]], "head_blocks": [["minecraft:vine", 1]]"#,
        ),
        &cx(),
    );
    assert!(super::common::has_text(
        &d,
        "Bad value for growth_direction - should be 'UP' or 'DOWN'"
    ));
}

#[test]
fn the_single_block_step_adds_no_diagonal_list_deviation() {
    // Vanilla's step gives `may_attach_to` an empty `diagonal: []`; ours
    // leaves the member absent, the same empty set, so write-back invents nothing.
    let document = loads(
        &doc(
            "1.21.30",
            "single_block_feature",
            r#""enforce_placement_rules": true, "enforce_survivability_rules": true, "may_attach_to": {"top": "minecraft:stone"}"#,
        ),
        &cx(),
    );
    let bedrock_addon::families::feature::Feature::SingleBlock(feature) = document.value() else {
        panic!("not a single block feature");
    };
    assert_eq!(feature.may_attach_to.as_ref().unwrap().diagonal, None);
    let written = bedrock_addon::document::Content::to_json(document.value()).unwrap();
    assert!(!written.contains("diagonal"), "{written}");
}

// Vanilla's sentence names the child by its key as written, whatever
// the key holds; the finding's path writes `/` and `~` in the key escaped.
#[test]
fn an_unknown_child_is_named_by_its_key_as_written() {
    for (key, written) in [("plain", "plain"), ("a~b", "a~0b"), ("a/b", "a~1b")] {
        let json = doc(
            "1.26.50",
            "single_block_feature",
            &format!(
                r#""places_block": "minecraft:stone", "enforce_placement_rules": true, "enforce_survivability_rules": true, "{key}": 1"#
            ),
        );
        let found = match super::common::load(&json, &cx()) {
            Ok(document) => document.diagnostics().clone(),
            Err(rejected) => rejected.diagnostics,
        };
        let told: Vec<(String, String)> = found
            .iter()
            // By legacy name: the variant and the definitions that were it.
            .filter(|f| matches!(f.code.name(), "UnrecognizedField"))
            .map(|f| (f.location.json_path.to_string(), f.code.to_string()))
            .collect();
        assert_eq!(
            told,
            [(
                format!("minecraft:single_block_feature/{written}"),
                format!("child '{key}' not valid here.")
            )],
            "{key:?}"
        );
    }
}
