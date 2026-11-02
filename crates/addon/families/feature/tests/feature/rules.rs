//! Validation rules without a fixture row of their own.

use super::common::{cx, doc, engine, error_at, loads, rejects};

#[test]
fn rule_4_embed_in_surface_needs_engine_1_26_20() {
    let json = doc(
        "1.13.0",
        "snap_to_surface_feature",
        r#""feature_to_snap": "demo:y", "vertical_search_range": 4, "embed_in_surface": true"#,
    );
    error_at(
        &rejects(&json, &engine(1, 26, 10)),
        "UnrecognizedField",
        "embed_in_surface",
    );
    loads(&json, &engine(1, 26, 20));
}

#[test]
fn rule_16_a_search_volume_corner_has_exactly_three_elements() {
    let json = doc(
        "1.13.0",
        "search_feature",
        r#""places_feature": "demo:y", "search_volume": {"min": [0, 0], "max": [1, 1, 1, 1]}, "search_axis": "+y""#,
    );
    let d = rejects(&json, &cx());
    error_at(&d, "InvalidValue", "search_volume/min");
    error_at(&d, "InvalidValue", "search_volume/max");
    assert!(super::common::has_text(&d, "Array too small (2 < 3)"));
    assert!(super::common::has_text(&d, "Array too large (4 > 3)"));
}

#[test]
fn rule_27_horizontal_tree_decoration_needs_places_block() {
    let d = rejects(
        &doc(
            "1.26.40",
            "horizontal_tree_decoration_feature",
            r#""bark_side_only": true"#,
        ),
        &cx(),
    );
    error_at(&d, "MissingRequiredField", "places_block");
}

#[test]
fn rule_28_ground_level_and_adjustment_radius_ranges() {
    let template = |members: &str| {
        doc(
            "1.26.30",
            "structure_template_feature",
            &format!(r#""structure_name": "demo:ruin", "constraints": {{}}, {members}"#),
        )
    };
    error_at(
        &rejects(&template(r#""ground_level": -1"#), &cx()),
        "InvalidValue",
        "ground_level",
    );
    error_at(
        &rejects(&template(r#""adjustment_radius": 17"#), &cx()),
        "InvalidValue",
        "adjustment_radius",
    );
    // `ground_level` arrived with engine 1.26.30.
    error_at(
        &rejects(&template(r#""ground_level": 1"#), &engine(1, 26, 20)),
        "UnrecognizedField",
        "ground_level",
    );
}

#[test]
fn rule_29_max_steepness_is_bounded_from_engine_1_26_50() {
    let json = doc(
        "1.26.30",
        "structure_template_feature",
        r#""structure_name": "demo:ruin", "constraints": {"leveled": {"max_steepness": 9}}"#,
    );
    error_at(
        &rejects(&json, &cx()),
        "InvalidValue",
        "constraints/leveled/max_steepness",
    );
    loads(&json, &engine(1, 26, 40));
}

#[test]
fn rule_30_poplar_trunk_height_and_block() {
    let low = doc(
        "1.26.20",
        "tree_feature",
        r#""poplar_trunk": {"trunk_height": {"range_min": 0, "range_max": 4}, "trunk_block": "minecraft:poplar_log"}"#,
    );
    error_at(
        &rejects(&low, &cx()),
        "InvalidValue",
        "poplar_trunk/trunk_height",
    );
    let no_block = doc(
        "1.26.20",
        "tree_feature",
        r#""poplar_trunk": {"trunk_height": 8}"#,
    );
    error_at(
        &rejects(&no_block, &cx()),
        "MissingRequiredField",
        "poplar_trunk/trunk_block",
    );
}

// The identifier is read from the copy that stands. A
// payload written twice with the identifier only in its first copy has no identifier.
#[test]
fn a_doubled_payload_is_identified_by_its_last_copy() {
    let text = r#"{"format_version": "1.13.0", "minecraft:single_block_feature": {"description": {"identifier": "demo:first"}, "places_block": "minecraft:stone"}, "minecraft:single_block_feature": {"description": {}, "places_block": "minecraft:stone"}}"#;
    let d = super::common::rejects(text, &super::common::cx());
    assert!(
        super::common::has_text(
            &d,
            "[Feature Registry] Missing description identifier for JSON in the following file: features/test.json"
        ),
        "{d:#?}"
    );
}
