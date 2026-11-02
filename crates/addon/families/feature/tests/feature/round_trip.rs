//! Write-back at the Value level: parse, write, parse again and get the same
//! feature, with no member the author did not write; the history path written as read.

use bedrock_addon_core::document::Content;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_feature::{Feature, history};

use super::common::{base_game, cx, doc, engine, fixture, loads};

/// Parses `json`, writes it, parses the text again: the same feature, and the rewrite of the
/// rewrite is identical.
#[track_caller]
fn round_trip(json: &str, cx: &bedrock_addon_core::load::LoadContext) -> String {
    let first = loads(json, cx);
    let written = first.value().to_json().unwrap();
    let second = loads(&written, cx);
    assert_eq!(first.value(), second.value(), "{written}");
    assert_eq!(second.value().to_json().unwrap(), written);
    written
}

#[test]
fn every_type_round_trips() {
    let docs = [
        doc(
            "1.13.0",
            "aggregate_feature",
            r#""features": ["demo:a", "demo:b"], "early_out": "first_success""#,
        ),
        doc("1.13.0", "sequence_feature", r#""features": ["demo:a"]"#),
        doc(
            "1.13.0",
            "weighted_random_feature",
            r#""features": [["demo:a", 2], ["demo:b", 0.5]]"#,
        ),
        doc(
            "1.13.0",
            "search_feature",
            r#""places_feature": "demo:a", "search_volume": {"min": [-1, 0, -1], "max": [1, 4, 1]}, "search_axis": "-y", "required_successes": 2"#,
        ),
        doc(
            "1.13.0",
            "surface_relative_threshold_feature",
            r#""feature_to_place": "demo:a", "minimum_distance_below_surface": 3"#,
        ),
        doc(
            "1.13.0",
            "ore_feature",
            r#""count": 4, "discard_chance_on_air_exposure": 0.5, "replace_rules": [{"places_block": {"name": "minecraft:stone", "states": {"stone_type": "granite"}}, "may_replace": [{"tags": "q.any_tag('stone')"}]}]"#,
        ),
        doc(
            "1.21.40",
            "single_block_feature",
            r#""places_block": [{"block": "minecraft:stone", "weight": 2.5}], "enforce_placement_rules": true, "enforce_survivability_rules": false, "randomize_rotation": true, "may_attach_to": {"min_sides_must_attach": 2, "top": ["minecraft:stone"], "diagonal": "minecraft:dirt"}"#,
        ),
        doc(
            "1.13.0",
            "multiface_feature",
            r#""places_block": "minecraft:glow_lichen", "search_range": 20, "can_place_on_floor": true, "can_place_on_ceiling": true, "can_place_on_wall": true, "chance_of_spreading": 0.5, "can_place_on": ["minecraft:stone"]"#,
        ),
        doc(
            "1.13.0",
            "fossil_feature",
            r#""ore_block": "minecraft:coal_ore", "max_empty_corners": 4"#,
        ),
        doc(
            "1.13.0",
            "growing_plant_feature",
            r#""height_distribution": [[{"range_min": 1, "range_max": 12}, 2], [1, 1]], "growth_direction": "DOWN", "age": {"range_min": 17, "range_max": 26}, "body_blocks": [["minecraft:cave_vines", 4], [{"name": "minecraft:cave_vines_body_with_berries"}, 1]], "head_blocks": [["minecraft:cave_vines", 4]], "allow_water": true"#,
        ),
        doc(
            "1.13.0",
            "partially_exposed_blob_feature",
            r#""placement_radius_around_floor": 1, "placement_probability_per_valid_position": 0.5, "exposed_face": "down", "places_block": "minecraft:magma""#,
        ),
        doc(
            "1.13.0",
            "vegetation_patch_feature",
            r#""replaceable_blocks": ["minecraft:stone"], "ground_block": "minecraft:moss_block", "vegetation_feature": "demo:a", "surface": "ceiling", "depth": 1, "vertical_range": 5, "horizontal_radius": {"range_min": 4, "range_max": 7}, "extra_edge_column_chance": 0.3, "waterlogged": false"#,
        ),
        doc(
            "1.13.0",
            "cave_carver_feature",
            r#""fill_with": "minecraft:air", "width_modifier": 0.5, "skip_carve_chance": 15, "height_limit": 3, "y_scale": [0.1, 0.9], "floor_level": [-1, -0.4]"#,
        ),
        doc(
            "1.13.0",
            "underwater_cave_carver_feature",
            r#""fill_with": "minecraft:water", "replace_air_with": "minecraft:flowing_water", "width_modifier": "q.noise(1, 2)""#,
        ),
        doc(
            "1.26.30",
            "structure_template_feature",
            r#""structure_name": "demo:ruin", "adjustment_radius": 4, "facing_direction": "west", "rotate_around_center": true, "ground_level": 2, "constraints": {"grounded": {}, "unburied": {}, "block_intersection": {"block_allowlist": ["minecraft:air"], "only_check_intersection_for_motion_blocking_blocks": false}, "leveled": {"max_steepness": 3}}"#,
        ),
        doc(
            "1.26.50",
            "snap_to_surface_feature",
            r#""feature_to_snap": "demo:a", "search_range": 8, "surface": "random_horizontal", "allow_air_placement": false, "allowed_surface_blocks": ["minecraft:stone"], "embed_in_surface": true"#,
        ),
        doc(
            "1.26.30",
            "height_difference_filter_feature",
            r#""places_feature": "demo:a", "search_radius": 2, "max_allowed_downward_height_diff": 4"#,
        ),
        doc(
            "1.26.40",
            "multi_block_feature",
            r#""places_block": "demo:lamp", "enforce_placement_rules": true, "may_replace": ["minecraft:air"]"#,
        ),
        doc(
            "1.26.50",
            "multipart_block_column_feature",
            r#""tip_block": "minecraft:pointed_dripstone", "frustum_block": "minecraft:pointed_dripstone", "middle_block": "minecraft:pointed_dripstone", "base_block": "minecraft:pointed_dripstone", "height_range": [2, 6], "weighted_heights": [{"value": 3, "weight": 2}], "direction": "DOWN""#,
        ),
        doc(
            "1.13.0",
            "conditional_list",
            r#""conditional_features": [{"places_feature": "demo:a", "condition": "q.noise(1, 2) > 0"}], "early_out_scheme": "placement_success""#,
        ),
    ];
    for json in &docs {
        round_trip(json, &cx());
    }
    for json in [
        fixture("f01_scatter_1_13_0_to_distribution.json"),
        fixture("f13_tree_variants.json"),
        fixture("f21_orange_poplar_tree_feature.json"),
        fixture("f18_shelf_mushroom_feature.json"),
    ] {
        round_trip(&json, &engine(1, 26, 50));
    }
    let internal = [
        doc(
            "1.13.0",
            "rect_layout",
            r#""ratio_of_empty_space": 0.5, "feature_areas": [{"feature": "demo:a", "area_dimensions": [4, 4]}]"#,
        ),
        doc(
            "1.13.0",
            "scan_surface",
            r#""scan_surface_feature": "demo:a""#,
        ),
        doc(
            "1.13.0",
            "beards_and_shavers",
            r#""places_feature": "demo:a", "bounding_box_min": [0, 0, 0], "bounding_box_max": [8, 4, 8], "y_delta": 0.5, "surface_block_type": "minecraft:grass_block", "subsurface_block_type": "minecraft:dirt", "beard_raggedness_min": 0.1, "beard_raggedness_max": 0.6"#,
        ),
        doc(
            "1.13.0",
            "sculk_patch_feature",
            r#""can_place_sculk_patch_on": ["minecraft:stone"], "central_block": "minecraft:sculk_catalyst", "central_block_placement_chance": 0.5, "cursor_count": 10, "charge_amount": 32, "spread_attempts": 64, "growth_rounds": 0, "spread_rounds": 1, "extra_growth_chance": {"range_min": 1, "range_max": 3}"#,
        ),
    ];
    for json in &internal {
        round_trip(json, &base_game());
    }
}

#[test]
fn absent_members_are_not_written() {
    let written = round_trip(
        &doc(
            "1.21.40",
            "single_block_feature",
            r#""enforce_placement_rules": true, "enforce_survivability_rules": true, "may_attach_to": {}"#,
        ),
        &cx(),
    );
    for absent in [
        "randomize_rotation",
        "min_sides_must_attach",
        "auto_rotate",
        "diagonal",
        "places_block",
    ] {
        assert!(!written.contains(absent), "{absent} written: {written}");
    }
    assert!(written.contains(r#""may_attach_to": {}"#), "{written}");
}

#[test]
fn an_older_document_is_written_at_its_types_newest_format() {
    let written = loads(&fixture("f01_scatter_1_13_0_to_distribution.json"), &cx())
        .value()
        .to_json()
        .unwrap();
    assert!(
        written.contains(r#""format_version": "1.21.10""#),
        "{written}"
    );
    assert!(written.contains(r#""distribution""#));
}

#[test]
fn the_history_path_writes_as_read_and_upgrades_to_the_same_feature() {
    let json = fixture("f01_scatter_1_13_0_to_distribution.json");
    let path = PackPath::from_static("features/f01.json");
    let as_written = Feature::parse_as_written(&path, json.as_bytes(), &cx()).unwrap();
    assert!(matches!(
        as_written.value(),
        history::Any::ScatterV1_13_0(_)
    ));
    let text = as_written.value().to_json().unwrap();
    assert!(text.contains(r#""format_version": "1.13.0""#));
    let again = Feature::parse_as_written(&path, text.as_bytes(), &cx()).unwrap();
    assert_eq!(as_written.value(), again.value());
    let upgraded = history::upgrade(as_written, &cx()).unwrap();
    let parsed = Feature::parse(&path, json.as_bytes(), &cx()).unwrap();
    assert_eq!(upgraded.value(), parsed.value());
    assert_eq!(upgraded.version().parsed_as, parsed.version().parsed_as);
}

#[test]
fn a_document_at_its_newest_struct_is_latest_in_history() {
    let json = doc("1.13.0", "ore_feature", r#""count": 2"#);
    let path = PackPath::from_static("features/ore.json");
    let as_written = Feature::parse_as_written(&path, json.as_bytes(), &cx()).unwrap();
    assert!(matches!(
        as_written.value(),
        history::Any::Latest(Feature::Ore(_))
    ));
    assert_eq!(as_written.value().identifier().as_str(), "demo:x");
}

#[test]
fn history_reports_the_version_of_the_struct_that_parsed_it() {
    use bedrock_addon_core::version::FormatVersion;
    let path = PackPath::from_static("features/f01.json");
    let json = fixture("f01_scatter_1_13_0_to_distribution.json");
    let old = Feature::parse_as_written(&path, json.as_bytes(), &cx()).unwrap();
    assert_eq!(old.value().parsed_version(), FormatVersion::new(1, 13, 0));
    let json = doc(
        "1.26.0",
        "single_block_feature",
        r#""enforce_placement_rules": true, "enforce_survivability_rules": true"#,
    );
    let new = Feature::parse_as_written(&path, json.as_bytes(), &cx()).unwrap();
    assert_eq!(new.value().parsed_version(), FormatVersion::new(1, 21, 40));
}

#[test]
fn an_upgraded_form_the_newest_schema_rejects_is_written_at_its_own_version() {
    // A 1.13.0 `{tags}` places_block has no 1.21.40 form (f06), and a 1.13.0 scatter without
    // `x` none at 1.21.10 (f02e): both are written in the layout that accepts them.
    for json in [
        doc(
            "1.13.0",
            "single_block_feature",
            r#""places_block": {"tags": "q.any_tag('stone')"}, "enforce_placement_rules": true, "enforce_survivability_rules": true, "may_attach_to": {"top": "minecraft:stone"}"#,
        ),
        doc(
            "1.13.0",
            "scatter_feature",
            r#""places_feature": "demo:y", "iterations": 1, "y": 0, "z": 0"#,
        ),
    ] {
        let written = round_trip(&json, &cx());
        assert!(
            written.contains(r#""format_version": "1.13.0""#),
            "{written}"
        );
    }
}

#[test]
fn an_integral_real_and_an_alias_are_written_as_the_author_wrote_them() {
    // the author's `2.0` stays a real token, `block_whitelist` keeps its key.
    let geode = doc(
        "1.13.0",
        "geode_feature",
        r#""filler": "minecraft:air", "inner_layer": "minecraft:amethyst_block",
        "alternate_inner_layer": "minecraft:budding_amethyst", "middle_layer": "minecraft:calcite",
        "outer_layer": "minecraft:smooth_basalt", "min_outer_wall_distance": 4,
        "max_outer_wall_distance": 7, "min_distribution_points": 3, "max_distribution_points": 5,
        "min_point_offset": 1, "max_point_offset": 3, "max_radius": 16,
        "crack_point_offset": 2.0, "generate_crack_chance": 0.95, "base_crack_size": 2.0,
        "noise_multiplier": 0.025, "use_potential_placements_chance": 0.35,
        "use_alternate_layer0_chance": 0.083, "placements_require_layer0_alternate": true,
        "invalid_blocks_threshold": 1"#,
    );
    let written = round_trip(&geode, &cx());
    assert!(
        written.contains(r#""crack_point_offset": 2.0"#),
        "{written}"
    );
    assert!(written.contains(r#""max_radius": 16,"#), "{written}");
    let template = doc(
        "1.13.0",
        "structure_template_feature",
        r#""structure_name": "demo:ruin", "constraints": {"block_intersection": {"block_whitelist": ["minecraft:air"]}}"#,
    );
    let written = round_trip(&template, &cx());
    assert!(written.contains("block_whitelist"), "{written}");
    assert!(!written.contains("block_allowlist"), "{written}");
}
