//! 20 W1 (the features a feature names), W3 (its blocks and their states), W4
//! (`single_block_feature.places_block`), W5 (`multi_block_feature.places_block`) and W6 (the
//! structure of a `structure_template_feature`). W2 and W7 are audited in `audit.rs`.

#![allow(unused_imports, reason = "which rows compile depends on the features")]

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{LinkOptions, StackSet};

use super::super::common::{hosted, stacks};
use super::{finding, said, stacks_with_game, unresolved_error, unresolved_info};

/// A feature document of the type `kind` (its payload key without `minecraft:`) at `version`,
/// named `id`, with `members` after its description.
pub fn feature(version: &str, kind: &str, id: &str, members: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "minecraft:{kind}": {{"description": {{"identifier": "{id}"}}{members}}}}}"#
    )
}

/// An ore feature `id`: a feature that names no other.
pub fn vein(id: &str) -> String {
    feature("1.13.0", "ore_feature", id, r#", "count": 8"#)
}

/// Requires every feature file of the world to have loaded: a rejected file is linked by no
/// rule, and a test that expects no finding from it would prove nothing.
#[track_caller]
fn all_loaded(set: &StackSet, files: usize) {
    let features = set.behavior.features();
    assert_eq!(features.len(), files, "{:#?}", features.rejected());
}

// ---- W1 --------------------------------------------------------------------------------------

/// A row of [`naming`]: the file, the document, the JSON path of the member, and whether
/// only the game's own packs may use the type.
type NamingRow = (&'static str, String, &'static str, bool);

/// One document per member of the row, each naming the feature `target`: the file, the
/// document, the JSON path of the member, and whether the type is one only the game's own
/// packs may use.
fn naming(target: &str) -> Vec<NamingRow> {
    let mut rows = naming_of_lists_and_places(target);
    rows.extend(naming_of_layouts_and_trees(target));
    rows
}

/// A feature document of `kind`, called `demo:<kind>`.
fn f(version: &str, kind: &str, members: &str) -> String {
    feature(version, kind, &format!("demo:{kind}"), members)
}

/// The first rows of [`naming`]: the lists, and the members that place one feature.
fn naming_of_lists_and_places(target: &str) -> Vec<NamingRow> {
    vec![
        (
            "features/aggregate.json",
            f(
                "1.13.0",
                "aggregate_feature",
                &format!(r#", "features": ["demo:vein", "{target}"]"#),
            ),
            "minecraft:aggregate_feature/features/1",
            false,
        ),
        (
            "features/sequence.json",
            f(
                "1.13.0",
                "sequence_feature",
                &format!(r#", "features": ["{target}"]"#),
            ),
            "minecraft:sequence_feature/features/0",
            false,
        ),
        (
            "features/scatter.json",
            f(
                "1.21.10",
                "scatter_feature",
                &format!(
                    r#", "places_feature": "{target}", "distribution": {{"iterations": 1, "x": 0, "y": 0, "z": 0}}"#
                ),
            ),
            "minecraft:scatter_feature/places_feature",
            false,
        ),
        (
            "features/snap.json",
            f(
                "1.26.50",
                "snap_to_surface_feature",
                &format!(r#", "feature_to_snap": "{target}", "search_range": 4"#),
            ),
            "minecraft:snap_to_surface_feature/feature_to_snap",
            false,
        ),
        (
            "features/search.json",
            f(
                "1.13.0",
                "search_feature",
                &format!(
                    r#", "places_feature": "{target}", "search_volume": {{"min": [0, 0, 0], "max": [1, 1, 1]}}, "search_axis": "-y""#
                ),
            ),
            "minecraft:search_feature/places_feature",
            false,
        ),
        (
            "features/rect.json",
            f(
                "1.13.0",
                "rect_layout",
                &format!(
                    r#", "feature_areas": [{{"feature": "{target}", "area_dimensions": [1, 1]}}]"#
                ),
            ),
            "minecraft:rect_layout/feature_areas/0/feature",
            true,
        ),
        (
            "features/scan.json",
            f(
                "1.13.0",
                "scan_surface",
                &format!(r#", "scan_surface_feature": "{target}""#),
            ),
            "minecraft:scan_surface/scan_surface_feature",
            true,
        ),
        (
            "features/weighted.json",
            f(
                "1.13.0",
                "weighted_random_feature",
                &format!(r#", "features": [["demo:vein", 1], ["{target}", 2]]"#),
            ),
            "minecraft:weighted_random_feature/features/1/0",
            false,
        ),
    ]
}

/// The last rows of [`naming`]: the layouts, the patch and the trees.
fn naming_of_layouts_and_trees(target: &str) -> Vec<NamingRow> {
    vec![
        (
            "features/conditional.json",
            f(
                "1.13.0",
                "conditional_list",
                &format!(
                    r#", "conditional_features": [{{"places_feature": "{target}", "condition": "1"}}]"#
                ),
            ),
            "minecraft:conditional_list/conditional_features/0/places_feature",
            true,
        ),
        (
            "features/relative.json",
            f(
                "1.13.0",
                "surface_relative_threshold_feature",
                &format!(r#", "feature_to_place": "{target}""#),
            ),
            "minecraft:surface_relative_threshold_feature/feature_to_place",
            false,
        ),
        (
            "features/beards.json",
            f(
                "1.13.0",
                "beards_and_shavers",
                &format!(
                    r#", "places_feature": "{target}", "bounding_box_min": [0, 0, 0], "bounding_box_max": [1, 1, 1], "y_delta": 1.0, "surface_block_type": "minecraft:grass_block", "subsurface_block_type": "minecraft:dirt""#
                ),
            ),
            "minecraft:beards_and_shavers/places_feature",
            true,
        ),
        (
            "features/patch.json",
            f(
                "1.13.0",
                "vegetation_patch_feature",
                &format!(
                    r#", "replaceable_blocks": ["minecraft:stone"], "ground_block": "minecraft:moss_block", "vegetation_feature": "{target}", "depth": 1, "vertical_range": 5, "horizontal_radius": 2"#
                ),
            ),
            "minecraft:vegetation_patch_feature/vegetation_feature",
            false,
        ),
        (
            "features/fallen.json",
            feature(
                "1.13.0",
                "tree_feature",
                "demo:fallen",
                &format!(
                    r#", "fallen_trunk": {{"log_length": 3, "trunk_block": "minecraft:oak_log", "log_decoration_feature": "{target}"}}"#
                ),
            ),
            "minecraft:tree_feature/fallen_trunk/log_decoration_feature",
            false,
        ),
        (
            "features/poplar.json",
            feature(
                "1.26.20",
                "tree_feature",
                "demo:poplar",
                &format!(
                    r#", "poplar_trunk": {{"trunk_height": {{"range_min": 7, "range_max": 11}}, "trunk_block": "minecraft:oak_log", "remaining_trunk_height_above_branches": {{"range_min": 4, "range_max": 4}}, "amount_of_foliage_support_branches": {{"range_min": 1, "range_max": 4}}, "log_decoration_feature": "{target}"}}, "poplar_canopy": {{"radius": [{{"value": 5, "weight": 5}}], "height": {{"range_min": 5, "range_max": 6}}, "side_hole_chance": 0.15, "leaf_block": "minecraft:oak_leaves", "branch_block": "minecraft:oak_log"}}"#
                ),
            ),
            "minecraft:tree_feature/poplar_trunk/log_decoration_feature",
            false,
        ),
    ]
}

/// The world of [`naming`]: the documents of the game's own types in a pack the game ships,
/// the others in a pack of the world, and `more` in the world's pack too.
fn world_naming(target: &str, more: &[(&'static str, &str)], options: LoadOptions) -> StackSet {
    let documents = naming(target);
    let mut world: Vec<(&'static str, &str)> = more.to_vec();
    let mut game = Vec::new();
    for (path, text, _, internal) in &documents {
        if *internal {
            game.push((*path, text.as_str()));
        } else {
            world.push((*path, text.as_str()));
        }
    }
    let set = stacks_with_game(&world, &game, options);
    all_loaded(&set, documents.len() + more.len());
    set
}

#[test]
fn w1_every_member_that_names_a_feature_nothing_defines_is_an_information() {
    let vein = vein("demo:vein");
    let set = world_naming(
        "demo:gone",
        &[("features/vein.json", vein.as_str())],
        LoadOptions::default(),
    );
    let mut expected: Vec<_> = naming("demo:gone")
        .iter()
        .map(|(path, _, at, _)| unresolved_info(path, at, "feature", "demo:gone"))
        .collect();
    let mut found = said(&set.link(), &["feature"]);
    expected.sort();
    found.sort();
    assert_eq!(found, expected);
    // Ours: feature references are lazy and the game logs nothing at load: an
    // information, so a lint.
    assert_eq!(
        said(
            &set.link_with(&LinkOptions {
                lints: false,
                ..Default::default()
            }),
            &["feature"]
        ),
        []
    );
}

#[test]
fn w1_a_feature_a_pack_defines_resolves_whatever_the_order_of_the_files() {
    // `features/zz_late.json` is read after every file that names it: a forward reference.
    let late = vein("demo:late");
    let set = world_naming(
        "demo:late",
        &[
            ("features/vein.json", &vein("demo:vein")),
            ("features/zz_late.json", late.as_str()),
        ],
        LoadOptions::default(),
    );
    assert_eq!(said(&set.link(), &["feature"]), []);
}

#[test]
fn w1_a_host_answers_for_a_feature_of_the_game() {
    let set = world_naming(
        "minecraft:oak_tree_feature",
        &[],
        hosted(&[
            ("feature", "minecraft:oak_tree_feature"),
            ("feature", "demo:vein"),
            ("block", "minecraft:stone"),
            ("block", "minecraft:moss_block"),
            ("block", "minecraft:oak_log"),
            ("block", "minecraft:oak_leaves"),
            ("block", "minecraft:grass_block"),
            ("block", "minecraft:dirt"),
        ]),
    );
    assert_eq!(said(&set.link(), &["feature"]), []);
    // Without a host the crate has no list of the game's features: the name is missing.
    let set = world_naming("minecraft:oak_tree_feature", &[], LoadOptions::default());
    assert_eq!(
        said(&set.link(), &["feature"]).len(),
        naming("x").len() + 2,
        "each member, and `demo:vein` of the aggregate and of the weighted list"
    );
}

/// A feature of the app's own definitions (which are of the `minecraft` namespace) is a feature
/// like any other.
#[test]
fn w1_a_feature_of_the_apps_definitions_resolves() {
    use bedrock_addon::load::AppData;
    use bedrock_addon::storage::PackPath;
    let app = AppData::from_files([(
        PackPath::from_static("definitions/features/own.json"),
        vein("minecraft:own").into_bytes(),
    )]);
    let sequence = feature(
        "1.13.0",
        "sequence_feature",
        "demo:sequence",
        r#", "features": ["minecraft:own", "demo:gone"]"#,
    );
    let set = stacks(
        &[("features/sequence.json", sequence.as_str())],
        &[],
        LoadOptions {
            app_data: Some(app),
            ..LoadOptions::default()
        },
    );
    assert_eq!(
        said(&set.link(), &["feature"]),
        [unresolved_info(
            "features/sequence.json",
            "minecraft:sequence_feature/features/1",
            "feature",
            "demo:gone"
        )]
    );
}

// ---- W3 --------------------------------------------------------------------------------------

/// Documents that name the block `block` in members of every shape the row has: a descriptor
/// (string and object), a list of descriptors, one descriptor or a list under one name, a
/// `[block, weight]` pair, a specifier, and members of nested parts. Each with the JSON paths
/// of its members.
#[cfg(feature = "block")]
fn naming_blocks(block: &str) -> Vec<(&'static str, String, Vec<&'static str>)> {
    vec![
        (
            "features/ore.json",
            feature(
                "1.13.0",
                "ore_feature",
                "demo:ore",
                &format!(
                    r#", "count": 8, "replace_rules": [{{"places_block": {{"name": "{block}"}}, "may_replace": ["minecraft:stone", "{block}"]}}]"#
                ),
            ),
            vec![
                "minecraft:ore_feature/replace_rules/0/places_block/name",
                "minecraft:ore_feature/replace_rules/0/may_replace/1",
            ],
        ),
        (
            "features/single.json",
            feature(
                "1.21.40",
                "single_block_feature",
                "demo:single",
                &format!(
                    r#", "places_block": "minecraft:stone", "enforce_placement_rules": true, "enforce_survivability_rules": true, "may_attach_to": {{"top": "{block}", "sides": ["minecraft:stone", "{block}"]}}, "may_not_attach_to": {{"bottom": ["{block}"]}}, "may_replace": ["{block}"]"#
                ),
            ),
            vec![
                "minecraft:single_block_feature/may_attach_to/top",
                "minecraft:single_block_feature/may_attach_to/sides/1",
                "minecraft:single_block_feature/may_not_attach_to/bottom/0",
                "minecraft:single_block_feature/may_replace/0",
            ],
        ),
        (
            "features/plant.json",
            feature(
                "1.13.0",
                "growing_plant_feature",
                "demo:plant",
                &format!(
                    r#", "height_distribution": [[{{"range_min": 1, "range_max": 3}}, 1]], "growth_direction": "UP", "body_blocks": [["minecraft:stone", 1], ["{block}", 2]], "head_blocks": [[{{"name": "{block}"}}, 1]]"#
                ),
            ),
            vec![
                "minecraft:growing_plant_feature/body_blocks/1/0",
                "minecraft:growing_plant_feature/head_blocks/0/0/name",
            ],
        ),
        (
            "features/carver.json",
            feature(
                "1.13.0",
                "underwater_cave_carver_feature",
                "demo:carver",
                &format!(r#", "fill_with": "{block}", "replace_air_with": {{"name": "{block}"}}"#),
            ),
            vec![
                "minecraft:underwater_cave_carver_feature/fill_with",
                "minecraft:underwater_cave_carver_feature/replace_air_with/name",
            ],
        ),
        (
            "features/template.json",
            feature(
                "1.13.0",
                "structure_template_feature",
                "demo:template",
                &format!(
                    r#", "structure_name": "mystructure:hut", "constraints": {{"block_intersection": {{"block_allowlist": ["{block}"]}}}}"#
                ),
            ),
            vec![
                "minecraft:structure_template_feature/constraints/block_intersection/block_allowlist/0",
            ],
        ),
        (
            "features/tree.json",
            feature(
                "1.13.0",
                "tree_feature",
                "demo:tree",
                &format!(
                    r#", "base_block": ["minecraft:dirt", "{block}"], "may_grow_on": ["{block}"], "fallen_trunk": {{"log_length": 3, "trunk_block": "{block}", "trunk_decoration": {{"decoration_chance": 0.5, "decoration_block": "{block}"}}}}"#
                ),
            ),
            vec![
                "minecraft:tree_feature/base_block/1",
                "minecraft:tree_feature/may_grow_on/0",
                "minecraft:tree_feature/fallen_trunk/trunk_block",
                "minecraft:tree_feature/fallen_trunk/trunk_decoration/decoration_block",
            ],
        ),
    ]
}

/// The world of [`naming_blocks`] with `more` files.
#[cfg(feature = "block")]
fn world_naming_blocks(
    block: &str,
    more: &[(&'static str, &str)],
    options: LoadOptions,
) -> StackSet {
    let documents = naming_blocks(block);
    let mut files: Vec<(&'static str, &str)> = more.to_vec();
    files.extend(
        documents
            .iter()
            .map(|(path, text, _)| (*path, text.as_str())),
    );
    let set = stacks(&files, &[], options);
    all_loaded(&set, documents.len());
    set
}

#[cfg(feature = "block")]
#[test]
fn w3_every_member_that_names_a_block_nothing_defines_is_an_information() {
    let set = world_naming_blocks("demo:lamp", &[], LoadOptions::default());
    let mut expected: Vec<_> = naming_blocks("demo:lamp")
        .iter()
        .flat_map(|(path, _, members)| {
            members
                .iter()
                .map(|at| unresolved_info(path, at, "block", "demo:lamp"))
                .collect::<Vec<_>>()
        })
        .collect();
    let mut found = said(&set.link(), &["block"]);
    expected.sort();
    found.sort();
    assert_eq!(found, expected);
}

#[cfg(feature = "block")]
#[test]
fn w3_a_block_a_pack_defines_or_the_game_has_resolves() {
    let lamp = super::block("demo:lamp");
    let set = world_naming_blocks(
        "demo:lamp",
        &[("blocks/lamp.json", lamp.as_str())],
        LoadOptions::default(),
    );
    assert_eq!(said(&set.link(), &["block"]), []);
    // The crate's list of the game's blocks, then a host.
    let set = world_naming_blocks("minecraft:gravel", &[], LoadOptions::default());
    assert_eq!(said(&set.link(), &["block"]), []);
    let set = world_naming_blocks(
        "demo:lamp",
        &[],
        hosted(&[
            ("block", "demo:lamp"),
            ("block", "minecraft:stone"),
            ("block", "minecraft:dirt"),
        ]),
    );
    assert_eq!(said(&set.link(), &["block"]), []);
}

/// K112: the block names of older versions the game's own features write (`minecraft:grass`
/// in `clay_with_dripleaves_feature.json`, `minecraft:planks` in `sculk_vein_feature.json`,
/// `minecraft:tallgrass`) are blocks of the game in every member a feature names a block
/// by, with a host or without one.
#[cfg(feature = "block")]
#[test]
fn w3_w4_a_legacy_block_name_of_the_games_own_features_resolves() {
    for name in ["minecraft:grass", "minecraft:planks", "minecraft:tallgrass"] {
        let set = world_naming_blocks(name, &[], LoadOptions::default());
        assert_eq!(said(&set.link(), &["block"]), [], "{name}");
        let host = hosted(&[("block", "minecraft:stone"), ("block", "minecraft:dirt")]);
        let set = world_naming_blocks(name, &[], host);
        assert_eq!(said(&set.link(), &["block"]), [], "{name}, with a host");
    }
    // A name that is neither a block nor a legacy name is still told.
    let set = world_naming_blocks("minecraft:oak_button", &[], LoadOptions::default());
    assert!(!said(&set.link(), &["block"]).is_empty());
}

/// A state of a block of the stack must be one of the block's, and its value one the state
/// can take; the states of the game's own blocks are not known and not checked.
#[cfg(feature = "block")]
#[test]
fn w3_a_state_the_block_does_not_have_and_a_value_it_cannot_take_are_informations() {
    let lamp = r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:lamp", "states": {"demo:lit": [false, true]}}, "components": {}}}"#;
    let ore = feature(
        "1.13.0",
        "ore_feature",
        "demo:ore",
        r#", "count": 8, "replace_rules": [{"places_block": {"name": "demo:lamp", "states": {"demo:lit": true}}, "may_replace": [{"name": "demo:lamp", "states": {"demo:dim": true}}, {"name": "demo:lamp", "states": {"demo:lit": 3}}, {"name": "minecraft:stone", "states": {"anything": 1}}]}]"#,
    );
    let set = stacks(
        &[("blocks/lamp.json", lamp), ("features/ore.json", &ore)],
        &[],
        LoadOptions::default(),
    );
    all_loaded(&set, 1);
    let rule = "minecraft:ore_feature/replace_rules/0/may_replace";
    assert_eq!(
        said(&set.link(), &["block", "block state"]),
        [
            // The states are the crate's own check, which the game does not make:
            // informations.
            finding(
                "core/unresolved_reference",
                Severity::Info,
                "features/ore.json",
                &format!("{rule}/0/states/demo:dim"),
                "block state",
                "demo:dim"
            ),
            finding(
                "core/invalid_reference",
                Severity::Info,
                "features/ore.json",
                &format!("{rule}/1/states/demo:lit"),
                "block state",
                "demo:lit"
            ),
        ]
    );
}

/// Without the `block` module nobody knows a pack's blocks.
#[cfg(not(feature = "block"))]
#[test]
fn w3_without_the_block_module_no_block_of_a_feature_is_reported() {
    let ore = feature(
        "1.13.0",
        "ore_feature",
        "demo:ore",
        r#", "count": 8, "replace_rules": [{"places_block": "demo:lamp"}]"#,
    );
    let set = stacks(&[("features/ore.json", &ore)], &[], LoadOptions::default());
    all_loaded(&set, 1);
    assert_eq!(said(&set.link(), &["block"]), []);
}

// ---- W4 --------------------------------------------------------------------------------------

#[cfg(feature = "block")]
#[test]
fn w4_a_block_single_block_places_that_nothing_defines_is_a_kept_error_with_vanillas_text() {
    const TEXT: &str = "Failed to resolve block for 'places_block' property.";

    let single = |places: &str| {
        feature(
            "1.21.40",
            "single_block_feature",
            "demo:single",
            &format!(
                r#", "places_block": {places}, "enforce_placement_rules": true, "enforce_survivability_rules": true"#
            ),
        )
    };
    for (places, expected) in [
        (r#""demo:lamp""#, vec![("", "demo:lamp")]),
        (r#"{"name": "demo:lamp"}"#, vec![("/name", "demo:lamp")]),
        (
            r#"[{"block": "minecraft:stone", "weight": 1}, {"block": "demo:lamp", "weight": 2}, {"block": {"name": "demo:lantern"}, "weight": 1}]"#,
            vec![("/1/block", "demo:lamp"), ("/2/block/name", "demo:lantern")],
        ),
        (r#""minecraft:stone""#, vec![]),
    ] {
        let text = single(places);
        let set = stacks(
            &[("features/single.json", &text)],
            &[],
            LoadOptions::default(),
        );
        all_loaded(&set, 1);
        let found = set.link();
        let expected: Vec<_> = expected
            .iter()
            .map(|(member, id)| {
                unresolved_error(
                    "features/single.json",
                    &format!("minecraft:single_block_feature/places_block{member}"),
                    "block",
                    id,
                )
            })
            .collect();
        assert_eq!(said(&found, &["block"]), expected, "{places}");
        // The setter logs at level 3 and returns
        // without the block, the feature kept: a kept error.
        assert!(
            found
                .iter()
                .filter(|finding| finding.code.id() == "core/unresolved_reference_worded")
                .all(|finding| finding.kept),
            "{found:#?}"
        );
        assert!(
            super::texts(&found, "block")
                .iter()
                .all(|text| text == TEXT),
            "{found:#?}"
        );
    }
}

/// A document below format 1.21.40 has a descriptor there, which the upgrade carries.
#[cfg(feature = "block")]
#[test]
fn w4_an_old_single_block_feature_is_held_to_the_same_rule() {
    let old = feature(
        "1.13.0",
        "single_block_feature",
        "demo:single",
        r#", "places_block": {"name": "demo:lamp"}, "enforce_placement_rules": true, "enforce_survivability_rules": true"#,
    );
    let set = stacks(
        &[("features/single.json", &old)],
        &[],
        LoadOptions::default(),
    );
    all_loaded(&set, 1);
    assert_eq!(
        said(&set.link(), &["block"]),
        [unresolved_error(
            "features/single.json",
            "minecraft:single_block_feature/places_block/name",
            "block",
            "demo:lamp"
        )]
    );
}

// ---- W5 --------------------------------------------------------------------------------------

#[cfg(feature = "block")]
const DOOR: &str = r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:door", "traits": {"minecraft:multi_block": {"enabled_states": ["minecraft:multi_block_part"], "direction": "up"}}}, "components": {"minecraft:movable": {"movement_type": "popped"}}}}"#;

#[cfg(feature = "block")]
fn multi_block(places: &str) -> String {
    feature(
        "1.26.40",
        "multi_block_feature",
        "demo:place",
        &format!(r#", "places_block": {places}"#),
    )
}

/// The W5 findings of a world with the door, a plain block and a feature placing `places`.
#[cfg(feature = "block")]
fn multi_block_findings(places: &str, options: LoadOptions) -> (Vec<super::Said>, Vec<String>) {
    let (lamp, text) = (super::block("demo:lamp"), multi_block(places));
    let set = stacks(
        &[
            ("blocks/door.json", DOOR),
            ("blocks/lamp.json", lamp.as_str()),
            ("features/place.json", text.as_str()),
        ],
        &[],
        LoadOptions {
            upcoming_creator_features: true,
            ..options
        },
    );
    all_loaded(&set, 1);
    assert_eq!(
        set.behavior.blocks().len(),
        2,
        "{:#?}",
        set.behavior.blocks().rejected()
    );
    let found = set.link();
    let said: Vec<_> = found
        .iter()
        .map(super::said_of)
        .filter(|said| said.0 == "core/invalid_reference_worded")
        .collect();
    let texts = found
        .iter()
        .filter(|finding| finding.code.id() == "core/invalid_reference_worded")
        .map(|finding| finding.code.to_string())
        .collect();
    (said, texts)
}

#[cfg(feature = "block")]
#[test]
fn w5_the_starting_part_of_a_multi_block_is_what_a_multi_block_feature_places() {
    for places in [
        r#""demo:door""#,
        r#"{"name": "demo:door"}"#,
        r#"{"name": "demo:door", "states": {"minecraft:multi_block_part": 0}}"#,
        // A block of the game: its traits are not known here, and nothing is told.
        r#""minecraft:oak_door""#,
        // A descriptor that names no block is the game's to evaluate.
        r#"{"tags": "q.any_tag('door')"}"#,
    ] {
        let (found, _) = multi_block_findings(places, LoadOptions::default());
        assert_eq!(found, [], "{places}");
    }
}

#[cfg(feature = "block")]
#[test]
fn w5_a_block_that_is_no_multi_block_is_an_error_with_vanillas_text() {
    for (places, member) in [
        (r#""demo:lamp""#, ""),
        (r#"{"name": "demo:lamp"}"#, "/name"),
    ] {
        let (found, texts) = multi_block_findings(places, LoadOptions::default());
        assert_eq!(
            found,
            [finding(
                "core/invalid_reference_worded",
                Severity::Error,
                "features/place.json",
                &format!("minecraft:multi_block_feature/places_block{member}"),
                "block",
                "demo:lamp"
            )],
            "{places}"
        );
        assert_eq!(
            texts,
            ["Must place a multi-block in a 'minecraft:multi_block_feature'"]
        );
    }
}

#[cfg(feature = "block")]
#[test]
fn w5_a_part_other_than_the_starting_one_is_an_error_with_vanillas_text() {
    let (found, texts) = multi_block_findings(
        r#"{"name": "demo:door", "states": {"minecraft:multi_block_part": 1}}"#,
        LoadOptions::default(),
    );
    assert_eq!(
        found,
        [finding(
            "core/invalid_reference_worded",
            Severity::Error,
            "features/place.json",
            "minecraft:multi_block_feature/places_block/states/minecraft:multi_block_part",
            "block state",
            "minecraft:multi_block_part"
        )]
    );
    assert_eq!(
        texts,
        [
            "If trying to place a multi-block in a 'minecraft:multi_block_feature' only the starting part may be used in field 'places_block'"
        ]
    );
}

// ---- W6 --------------------------------------------------------------------------------------

#[cfg(feature = "structure")]
fn template(name: &str, more: &str) -> String {
    feature(
        "1.13.0",
        "structure_template_feature",
        "demo:template",
        &format!(r#", "structure_name": "{name}", "constraints": {{}}{more}"#),
    )
}

/// A world with a structure three blocks high at `structures/demo/tower.mcstructure`, one at
/// `structures/hut.mcstructure`, and the feature `text`.
#[cfg(feature = "structure")]
fn world_with_structures(text: &str, options: LoadOptions) -> StackSet {
    use bedrock_addon::families::structure::Structure;

    use super::super::common::{behavior_pack_of, leak, stacks_of};
    let bytes = |size: [i32; 3]| -> &'static [u8] {
        Box::leak(
            Structure::new(size)
                .to_bytes()
                .expect("a structure encodes")
                .into_boxed_slice(),
        )
    };
    let pack = behavior_pack_of(
        1,
        &[
            ("structures/demo/tower.mcstructure", bytes([1, 3, 1])),
            ("structures/hut.mcstructure", bytes([1, 1, 1])),
            ("features/template.json", leak(text)),
        ],
    );
    let set = stacks_of(vec![pack], Vec::new(), options);
    all_loaded(&set, 1);
    set
}

#[cfg(feature = "structure")]
#[test]
fn w6_the_structure_of_a_template_feature_resolves_by_the_name_structure_load_takes() {
    for name in ["demo:tower", "mystructure:hut"] {
        let set = world_with_structures(&template(name, ""), LoadOptions::default());
        assert_eq!(said(&set.link(), &["structure"]), [], "{name}");
    }
    let set = world_with_structures(
        &template("minecraft:igloo", ""),
        hosted(&[("structure", "minecraft:igloo")]),
    );
    assert_eq!(said(&set.link(), &["structure"]), []);
}

#[cfg(feature = "structure")]
#[test]
fn w6_a_structure_nothing_has_is_an_information() {
    // A file directly in `structures/` is `mystructure:<name>`, not `<name>` alone.
    for name in ["demo:gone", "hut", "minecraft:igloo"] {
        let set = world_with_structures(&template(name, ""), LoadOptions::default());
        assert_eq!(
            said(&set.link(), &["structure"]),
            [unresolved_info(
                "features/template.json",
                "minecraft:structure_template_feature/structure_name",
                "structure",
                name
            )],
            "{name}"
        );
    }
}

/// A `ground_level` above the height of the structure is
/// clamped to it, with vanilla's warning. Both sides of the bound.
#[cfg(feature = "structure")]
#[test]
fn w6_a_ground_level_above_the_height_of_the_structure_is_a_warning() {
    let found = |level: i32| {
        let text = template("demo:tower", &format!(r#", "ground_level": {level}"#));
        let set = world_with_structures(&text, LoadOptions::default());
        set.link()
            .iter()
            .filter(|finding| finding.code.id() == "core/invalid_value_worded")
            .map(|finding| (super::said_of(finding), finding.code.to_string()))
            .collect::<Vec<_>>()
    };
    assert_eq!(found(3), []);
    assert_eq!(
        found(4),
        [(
            finding(
                "core/invalid_value_worded",
                Severity::Warning,
                "features/template.json",
                "minecraft:structure_template_feature/ground_level",
                "",
                ""
            ),
            "minecraft:structure_template_feature: ground_level (4) exceeds structure height (3). Clamped to 3."
                .to_owned()
        )]
    );
}

/// Without the `structure` module nobody knows a pack's structures.
#[cfg(not(feature = "structure"))]
#[test]
fn w6_without_the_structure_module_no_structure_of_a_feature_is_reported() {
    let text = feature(
        "1.13.0",
        "structure_template_feature",
        "demo:template",
        r#", "structure_name": "demo:gone", "constraints": {}"#,
    );
    let set = stacks(
        &[("features/template.json", &text)],
        &[],
        LoadOptions::default(),
    );
    all_loaded(&set, 1);
    assert_eq!(said(&set.link(), &["structure"]), []);
}
