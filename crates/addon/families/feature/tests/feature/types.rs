//! The other feature types' schema rules (fixtures f09, f13–f19, f21–f24).

use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_family_feature::{
    BlockList, Chance, EarlyOutScheme, Feature, StepDirection, Surface,
};

use super::common::{
    base_game, cx, doc, engine, error_at, fixture, has_text, kept_at, loads, rejects, with,
};

#[test]
fn f09_replace_rules_parse_at_1_13_0_and_1_26_0() {
    let members = r#""count": 8, "replace_rules": [{"places_block": "minecraft:diamond_ore", "may_replace": ["minecraft:stone"]}]"#;
    for version in ["1.13.0", "1.26.0"] {
        let Feature::Ore(ore) = loads(&doc(version, "ore_feature", members), &cx()).into_value()
        else {
            panic!("not an ore feature");
        };
        assert_eq!(ore.count.0, 8);
        assert_eq!(ore.replace_rules.as_ref().unwrap().len(), 1);
    }
}

#[test]
fn f09_root_places_block_and_may_replace_are_unknown() {
    let d = rejects(
        &doc(
            "1.13.0",
            "ore_feature",
            r#""count": 8, "places_block": "minecraft:diamond_ore", "may_replace": ["minecraft:stone"]"#,
        ),
        &cx(),
    );
    error_at(
        &d,
        "UnrecognizedField",
        "minecraft:ore_feature/places_block",
    );
    error_at(&d, "UnrecognizedField", "minecraft:ore_feature/may_replace");
}

#[test]
fn f09_count_0_and_empty_replace_rules_are_invalid() {
    let d = rejects(&doc("1.13.0", "ore_feature", r#""count": 0"#), &cx());
    error_at(&d, "InvalidValue", "count");
    assert!(has_text(
        &d,
        "Value '0' outside valid range [1, 2147483647]"
    ));
    let d = rejects(
        &doc(
            "1.13.0",
            "ore_feature",
            r#""count": 1, "replace_rules": []"#,
        ),
        &cx(),
    );
    error_at(&d, "InvalidValue", "replace_rules");
    assert!(has_text(&d, "Array too small (0 < 1)"));
}

#[test]
fn f13_tree_variants_parse() {
    let Feature::Tree(tree) = loads(&fixture("f13_tree_variants.json"), &cx()).into_value() else {
        panic!("not a tree feature");
    };
    let trunk = tree.acacia_trunk.as_ref().unwrap();
    let canopy = trunk
        .branches
        .as_ref()
        .unwrap()
        .branch_canopy
        .as_ref()
        .unwrap();
    assert_eq!(canopy.fancy_canopy.as_ref().unwrap().height.0, 2);
    assert!(tree.mangrove_roots.is_some());
    let decoration = tree
        .canopy
        .as_ref()
        .unwrap()
        .canopy_decoration
        .as_ref()
        .unwrap();
    assert_eq!(
        decoration
            .decoration_blocks_sequence
            .as_ref()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(decoration.step_direction, Some(StepDirection::Down));
    assert!(matches!(tree.base_block, Some(BlockList::Many(ref blocks)) if blocks.len() == 2));
}

#[test]
fn f13_a_cherry_trunk_base_of_1_is_invalid() {
    let json = doc(
        "1.13.0",
        "tree_feature",
        r#""cherry_trunk": {
            "trunk_block": "minecraft:cherry_log",
            "trunk_height": {"base": 1},
            "branches": {
                "branch_horizontal_length": {"range_min": 2, "range_max": 4},
                "branch_start_offset_from_top": {"range_min": -4, "range_max": -3},
                "branch_end_offset_from_top": {"range_min": -1, "range_max": 0}
            }
        }"#,
    );
    let d = rejects(&json, &cx());
    error_at(&d, "InvalidValue", "cherry_trunk/trunk_height/base");
}

#[test]
fn f14_geode_ranges() {
    let members = |wall: u32, crack: &str| {
        format!(
            r#""filler": "minecraft:air", "inner_layer": "minecraft:amethyst_block",
            "alternate_inner_layer": "minecraft:budding_amethyst", "middle_layer": "minecraft:calcite",
            "outer_layer": "minecraft:smooth_basalt",
            "min_outer_wall_distance": {wall}, "max_outer_wall_distance": 5,
            "min_distribution_points": 3, "max_distribution_points": 5,
            "min_point_offset": 1, "max_point_offset": 3, "max_radius": 16,
            "crack_point_offset": 2.0, "generate_crack_chance": 0.95, "base_crack_size": {crack},
            "noise_multiplier": 0.025, "use_potential_placements_chance": 0.35,
            "use_alternate_layer0_chance": 0.083, "placements_require_layer0_alternate": true,
            "invalid_blocks_threshold": 1"#
        )
    };
    let d = rejects(&doc("1.13.0", "geode_feature", &members(11, "2.0")), &cx());
    error_at(&d, "InvalidValue", "min_outer_wall_distance");
    loads(&doc("1.13.0", "geode_feature", &members(4, "5.0")), &cx());
}

const TEMPLATE: &str = r#""structure_name": "demo:ruin""#;

#[test]
fn f15_block_whitelist_is_an_alias_of_block_allowlist() {
    let json = doc(
        "1.13.0",
        "structure_template_feature",
        &format!(
            r#"{TEMPLATE}, "constraints": {{"block_intersection": {{"block_whitelist": ["minecraft:air"]}}}}"#
        ),
    );
    let Feature::StructureTemplate(feature) = loads(&json, &cx()).into_value() else {
        panic!("not a structure template feature");
    };
    let intersection = feature.constraints.block_intersection.as_ref().unwrap();
    assert_eq!(intersection.blocks().len(), 1);
    assert!(intersection.block_allowlist.is_none());
}

#[test]
fn f15_facing_direction_up_is_invalid() {
    let json = doc(
        "1.13.0",
        "structure_template_feature",
        &format!(r#"{TEMPLATE}, "facing_direction": "up", "constraints": {{}}"#),
    );
    let d = rejects(&json, &cx());
    error_at(&d, "InvalidValue", "facing_direction");
}

const CONDITIONAL: &str = r#""conditional_features": [{"places_feature": "demo:y", "condition": "1"}], "early_out_scheme": "none""#;

#[test]
fn f16_early_out_scheme_none_is_invalid_before_engine_1_26_50() {
    let mut cx = engine(1, 26, 20);
    cx.is_base_game_pack = true;
    let d = rejects(&doc("1.13.0", "conditional_list", CONDITIONAL), &cx);
    error_at(&d, "InvalidValue", "early_out_scheme");
    let Feature::ConditionalList(list) = loads(
        &doc("1.13.0", "conditional_list", CONDITIONAL),
        &base_game(),
    )
    .into_value() else {
        panic!("not a conditional list");
    };
    assert_eq!(list.early_out_scheme, Some(EarlyOutScheme::None));
}

#[test]
fn f17_no_height_difference_is_warned_and_the_feature_loads() {
    let json = doc(
        "1.26.30",
        "height_difference_filter_feature",
        r#""places_feature": "demo:y", "search_radius": 2"#,
    );
    let document = loads(&json, &engine(1, 26, 30));
    kept_at(
        document.diagnostics(),
        "MissingRequiredField",
        "minecraft:height_difference_filter_feature",
    );
    assert!(has_text(
        document.diagnostics(),
        "height_difference_filter_feature: No height differences specified"
    ));
}

#[test]
fn f17_a_missing_places_feature_loads_and_is_reported_at_placement() {
    // Vanilla logs "`height_difference_filter_feature` could not find feature
    // `places_feature`." in `place` (20 W2: run); loading reports nothing about it.
    let json = doc(
        "1.26.30",
        "height_difference_filter_feature",
        r#""places_feature": "test:missing", "search_radius": 2, "max_allowed_upward_height_diff": 3"#,
    );
    let document = loads(&json, &engine(1, 26, 30));
    assert!(
        with(
            document.diagnostics(),
            "UnresolvedReference",
            Severity::Warning
        )
        .is_empty()
    );
}

#[test]
fn f18_the_vanilla_shelf_mushroom_loads_from_engine_1_26_40() {
    let json = fixture("f18_shelf_mushroom_feature.json");
    let Feature::HorizontalTreeDecoration(feature) = loads(&json, &engine(1, 26, 40)).into_value()
    else {
        panic!("not a horizontal tree decoration");
    };
    assert_eq!(feature.bark_side_only.map(|b| b.0), Some(true));
    assert_eq!(feature.allow_adjacent.map(|b| b.0), Some(false));
    rejects(&json, &engine(1, 26, 30));
}

#[test]
fn f21_the_vanilla_orange_poplar_tree_loads_from_engine_1_26_40() {
    let json = fixture("f21_orange_poplar_tree_feature.json");
    let Feature::Tree(tree) = loads(&json, &engine(1, 26, 40)).into_value() else {
        panic!("not a tree");
    };
    let canopy = tree.poplar_canopy.as_ref().unwrap();
    assert_eq!(canopy.radius.len(), 4);
    assert_eq!(canopy.side_hole_chance.map(|v| v.0), Some(0.15));
    let trunk = tree.poplar_trunk.as_ref().unwrap();
    assert!(trunk.remaining_trunk_height_above_branches.is_some());
    assert_eq!(
        trunk
            .log_decoration_feature
            .as_ref()
            .map(bedrock_addon_core::id::Id::as_str),
        Some("minecraft:optional_shelf_mushroom_feature")
    );
    let d = rejects(&json, &engine(1, 26, 30));
    error_at(&d, "UnrecognizedField", "poplar_trunk");
}

#[test]
fn f21_poplar_canopy_bounds() {
    let base = fixture("f21_orange_poplar_tree_feature.json");
    for (from, to, tail) in [
        (
            r#""height":{"range_min":5,"range_max":6}"#,
            r#""height":{"range_min":3,"range_max":5}"#,
            "poplar_canopy/height",
        ),
        (
            r#""side_hole_chance":0.15"#,
            r#""side_hole_chance":1.5"#,
            "poplar_canopy/side_hole_chance",
        ),
        (
            r#""radius":[{"value":5,"weight":5},{"value":6,"weight":5},{"value":7,"weight":1},{"value":8,"weight":1}]"#,
            r#""radius":[]"#,
            "poplar_canopy/radius",
        ),
    ] {
        let compact = base.replace(['\n', ' '], "");
        assert!(compact.contains(from), "{from}");
        let d = rejects(&compact.replace(from, to), &engine(1, 26, 40));
        error_at(&d, "InvalidValue", tail);
    }
}

/// A type registered at `MultiBlockFeatureUpdate` (format 1.26.40) is in no schema below it: a
/// document there meets "Could not find registered feature type that matches name '%s'".
#[track_caller]
fn unknown_below_1_26_40(key: &str, members: &str) {
    let d = rejects(&doc("1.26.39", key, members), &cx());
    unknown_feature_type(&d, key);
}

/// `d` is the one unknown-type error, at the document root (the payload key the engine does not know).
#[track_caller]
fn unknown_feature_type(d: &bedrock_addon_core::diagnostic::Diagnostics, key: &str) {
    assert_eq!(d.len(), 1, "{d:#?}");
    let finding = d.iter().next().unwrap();
    assert!(
        matches!(finding.code, DiagnosticCode::UnknownPayloadKey { .. }),
        "{d:#?}"
    );
    assert_eq!(finding.severity, Severity::Error);
    assert_eq!(finding.location.json_path.segments().len(), 0, "{d:#?}");
    assert!(has_text(
        d,
        &format!("Could not find registered feature type that matches name 'minecraft:{key}'")
    ));
}

#[test]
fn f22_multi_block_below_its_floor_is_an_unknown_feature_type() {
    let members = r#""places_block": "demo:lamp""#;
    unknown_below_1_26_40("multi_block_feature", members);
    loads(&doc("1.26.40", "multi_block_feature", members), &cx());
    let d = rejects(&doc("1.26.40", "multi_block_feature", ""), &cx());
    error_at(&d, "MissingRequiredField", "places_block");
}

const SNAP: &str = r#""feature_to_snap": "demo:y""#;

#[test]
fn f23_vertical_search_range_is_renamed_search_range() {
    let old = doc(
        "1.26.20",
        "snap_to_surface_feature",
        &format!(r#"{SNAP}, "vertical_search_range": 12"#),
    );
    let Feature::SnapToSurface(feature) = loads(&old, &cx()).into_value() else {
        panic!("not a snap feature");
    };
    assert_eq!(feature.search_range.0, 12);
    let new = doc(
        "1.26.50",
        "snap_to_surface_feature",
        &format!(
            r#"{SNAP}, "search_range": 12, "allow_non_air_placement": true, "surface": "wall""#
        ),
    );
    let Feature::SnapToSurface(feature) = loads(&new, &cx()).into_value() else {
        panic!("not a snap feature");
    };
    assert_eq!(feature.surface, Some(Surface::Wall));
    let early = doc(
        "1.26.20",
        "snap_to_surface_feature",
        &format!(r#"{SNAP}, "search_range": 12"#),
    );
    let d = rejects(&early, &cx());
    error_at(&d, "UnrecognizedField", "search_range");
}

/// `_buildSchema<SnapToSurfaceFeature>` gates only the search-range name on
/// format 1.26.50; `allow_non_air_placement` and the `wall` surface (setter, no
/// version check) are in every schema.
#[test]
fn allow_non_air_placement_and_wall_are_read_below_1_26_50() {
    let old = doc(
        "1.26.20",
        "snap_to_surface_feature",
        &format!(
            r#"{SNAP}, "vertical_search_range": 12, "surface": "wall", "allow_non_air_placement": true"#
        ),
    );
    let document = loads(&old, &cx());
    assert!(
        super::common::above_info(document.diagnostics()).is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    let Feature::SnapToSurface(feature) = document.into_value() else {
        panic!("not a snap feature");
    };
    assert_eq!(feature.surface, Some(Surface::Wall));
    assert_eq!(feature.allow_non_air_placement.map(|b| b.0), Some(true));
    assert_eq!(feature.search_range.0, 12);
}

#[test]
fn f24_height_differences_are_at_least_0_and_search_radius_is_required() {
    let negative = doc(
        "1.26.30",
        "height_difference_filter_feature",
        r#""places_feature": "demo:y", "search_radius": 2, "min_required_upward_height_diff": -1"#,
    );
    let d = rejects(&negative, &cx());
    error_at(&d, "InvalidValue", "min_required_upward_height_diff");
    let no_radius = doc(
        "1.26.30",
        "height_difference_filter_feature",
        r#""places_feature": "demo:y", "min_required_upward_height_diff": 1"#,
    );
    let d = rejects(&no_radius, &cx());
    error_at(&d, "MissingRequiredField", "search_radius");
}

#[test]
fn chance_takes_a_ratio_or_a_number() {
    // Vanilla's own trees write `"decoration_chance": 0.25`.
    let json = doc(
        "1.13.0",
        "tree_feature",
        r#""trunk": {"trunk_height": 4, "trunk_block": "minecraft:oak_log",
            "trunk_decoration": {"decoration_chance": 0.25, "decoration_block": "minecraft:vine"}}"#,
    );
    let Feature::Tree(tree) = loads(&json, &cx()).into_value() else {
        panic!("not a tree");
    };
    let decoration = tree
        .trunk
        .as_ref()
        .unwrap()
        .trunk_decoration
        .as_ref()
        .unwrap();
    assert_eq!(decoration.decoration_chance, Chance::Probability(0.25));
    // No other JSON type is a chance, and the two numbers of a ratio are integers: the
    // proxy sees the token's type .
    for chance in [
        r#""5""#,
        "true",
        r#"{"numerator": "1", "denominator": 2}"#,
        r#"{"numerator": 1.5, "denominator": 2}"#,
        r#"{"numerator": 1}"#,
    ] {
        let d = rejects(&json.replace("0.25", chance), &cx());
        error_at(&d, "InvalidType", "trunk_decoration/decoration_chance");
    }
}

#[test]
fn f15_block_intersection_needs_one_of_its_two_names() {
    let json = doc(
        "1.13.0",
        "structure_template_feature",
        &format!(r#"{TEMPLATE}, "constraints": {{"block_intersection": {{}}}}"#),
    );
    let d = rejects(&json, &cx());
    error_at(
        &d,
        "MissingRequiredField",
        "block_intersection/block_allowlist",
    );
}

#[test]
fn an_all_req_container_is_optional_and_a_req_one_is_required() {
    // "{…} (all req)" marks the members, "{…} (req)" the container.
    let fancy = doc(
        "1.13.0",
        "tree_feature",
        r#""fancy_trunk": {"trunk_width": 1, "trunk_block": "minecraft:oak_log", "width_scale": 1, "foliage_altitude_factor": 0.3}"#,
    );
    loads(&fancy, &cx());
    let mangrove = doc(
        "1.13.0",
        "tree_feature",
        r#""mangrove_trunk": {"trunk_width": 1, "trunk_block": "minecraft:mangrove_log"}"#,
    );
    loads(&mangrove, &cx());
    let half = doc(
        "1.13.0",
        "tree_feature",
        r#""fancy_trunk": {"trunk_width": 1, "trunk_block": "minecraft:oak_log", "width_scale": 1, "foliage_altitude_factor": 0.3, "branches": {"slope": 1}}"#,
    );
    error_at(
        &rejects(&half, &cx()),
        "MissingRequiredField",
        "branches/density",
    );
    let cherry = doc(
        "1.13.0",
        "tree_feature",
        r#""cherry_trunk": {"trunk_block": "minecraft:cherry_log", "trunk_height": {"base": 4}}"#,
    );
    error_at(
        &rejects(&cherry, &cx()),
        "MissingRequiredField",
        "cherry_trunk/branches",
    );
}

const MULTIPART: &str = r#""tip_block": "minecraft:pointed_dripstone", "frustum_block": "minecraft:pointed_dripstone", "middle_block": "minecraft:pointed_dripstone", "base_block": "minecraft:pointed_dripstone", "height_range": [2, 6], "weighted_heights": [{"value": 3, "weight": 2}], "direction": "DOWN""#;

/// `multipart_block_column_feature` is registered at `MultiBlockFeatureUpdate` (the fifth feature update): its format floor is 1.26.40, as `multi_block_feature`'s, and an
/// engine of 1.26.40 has it.
#[test]
fn multipart_block_column_has_format_floor_1_26_40() {
    let at_floor = doc("1.26.40", "multipart_block_column_feature", MULTIPART);
    let Feature::MultipartBlockColumn(_) = loads(&at_floor, &engine(1, 26, 40)).into_value() else {
        panic!("not a multipart block column");
    };
    unknown_below_1_26_40("multipart_block_column_feature", MULTIPART);
    loads(
        &doc("1.26.40", "multipart_block_column_feature", MULTIPART),
        &cx(),
    );
}

/// The other side of the engine gate: an engine of 1.26.39 has not registered the
/// type, so a 1.26.40 document is the unknown-type error.
#[test]
fn multipart_block_column_is_unknown_to_an_engine_below_1_26_40() {
    let at_floor = doc("1.26.40", "multipart_block_column_feature", MULTIPART);
    let d = rejects(&at_floor, &engine(1, 26, 39));
    unknown_feature_type(&d, "multipart_block_column_feature");
}

/// The range-name gate at its edge (`getMinFormatVersion(6)`: 1.26.49 reads
/// `vertical_search_range` and refuses `search_range`; 1.26.50 the other way round.
#[test]
fn the_snap_range_name_changes_between_1_26_49_and_1_26_50() {
    let at = |version: &str, member: &str| {
        doc(
            version,
            "snap_to_surface_feature",
            &format!(r#"{SNAP}, "{member}": 12"#),
        )
    };
    loads(&at("1.26.49", "vertical_search_range"), &cx());
    loads(&at("1.26.50", "search_range"), &cx());
    for (version, member) in [
        ("1.26.49", "search_range"),
        ("1.26.50", "vertical_search_range"),
    ] {
        let d = rejects(&at(version, member), &cx());
        error_at(&d, "UnrecognizedField", member);
    }
}

/// The explicit `none` engine gate at its edge (validation rule 32): refused on an
/// engine of 1.26.49, read on 1.26.50.
#[test]
fn an_explicit_none_needs_engine_1_26_50() {
    let mut below = engine(1, 26, 49);
    below.is_base_game_pack = true;
    let d = rejects(&doc("1.13.0", "conditional_list", CONDITIONAL), &below);
    let found = with(&d, "InvalidValue", Severity::Error);
    assert_eq!(found.len(), 1, "{d:#?}");
    assert!(
        found[0]
            .location
            .json_path
            .to_string()
            .ends_with("minecraft:conditional_list/early_out_scheme"),
        "{d:#?}"
    );
    let Feature::ConditionalList(list) = loads(
        &doc("1.13.0", "conditional_list", CONDITIONAL),
        &engine(1, 26, 50),
    )
    .into_value() else {
        panic!("not a conditional list");
    };
    assert_eq!(list.early_out_scheme, Some(EarlyOutScheme::None));
}

/// An unknown `surface` at either
/// format.
#[test]
fn an_unknown_snap_surface_has_the_text_with_wall() {
    let text =
        "Bad value for surface - should be 'ceiling', 'floor', 'random_horizontal', or `wall`";
    for (version, range) in [
        ("1.26.20", "vertical_search_range"),
        ("1.26.50", "search_range"),
    ] {
        let json = doc(
            version,
            "snap_to_surface_feature",
            &format!(r#"{SNAP}, "{range}": 12, "surface": "side""#),
        );
        let d = rejects(&json, &cx());
        error_at(&d, "InvalidValue", "surface");
        assert!(has_text(&d, text), "{version}: {d:#?}");
    }
}
