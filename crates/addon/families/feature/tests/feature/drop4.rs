//! The features of the vanilla pack `experimental_y_2026_drop_4` of 1.26.60 (copied under
//! `tests/fixtures/worldgen/feature/drop4/`): the 13 `minecraft:conditional_list` "hosted ore"
//! features and the two `snap_to_surface_feature` files;
//! and the engine's `snap_to_surface_feature` test bodies (under `engine/`).

use bedrock_addon_core::document::Content;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_family_feature::{Feature, Surface};

use super::common::{above_info, base_game, cx, error_at, fixture, loads, rejects};

/// The 13 conditional lists of the pack.
const CONDITIONAL_LISTS: &[&str] = &[
    "hosted_coal_ore_feature.json",
    "hosted_coal_ore_upper_feature.json",
    "hosted_copper_ore_feature.json",
    "hosted_diamond_ore_buried_feature.json",
    "hosted_diamond_ore_feature.json",
    "hosted_diamond_ore_large_feature.json",
    "hosted_gold_ore_feature.json",
    "hosted_gold_ore_lower_feature.json",
    "hosted_iron_ore_feature.json",
    "hosted_iron_ore_small_feature.json",
    "hosted_lapis_ore_buried_feature.json",
    "hosted_lapis_ore_feature.json",
    "hosted_redstone_ore_feature.json",
];

/// Loads every file of [`CONDITIONAL_LISTS`] in `cx`, each a conditional list with nothing
/// above information; how many.
#[track_caller]
fn every_conditional_list_loads(cx: &LoadContext) -> usize {
    let mut lists = 0;
    for name in CONDITIONAL_LISTS {
        let json = fixture(&format!("drop4/{name}"));
        let document = loads(&json, cx);
        assert!(
            above_info(document.diagnostics()).is_empty(),
            "{name}: {:#?}",
            document.diagnostics()
        );
        assert!(
            matches!(document.value(), Feature::ConditionalList(_)),
            "{name}"
        );
        lists += 1;
    }
    lists
}

/// `condition` is optional: "Condition for placing associated Feature. If omitted, always
/// places." (`_buildSchema<ConditionalListFeature>`, `addChild<ExpressionNode>("condition",
/// required = false)`; the pack's lists end with an entry without one.
#[test]
fn the_drop_4_conditional_lists_load_in_their_base_game_pack() {
    assert_eq!(every_conditional_list_loads(&base_game()), 13);
}

#[test]
fn an_entry_without_a_condition_holds_none_and_is_written_without_one() {
    let document = loads(&fixture("drop4/hosted_coal_ore_feature.json"), &base_game());
    let Feature::ConditionalList(list) = document.value() else {
        panic!("not a conditional list");
    };
    let final_entry = list.conditional_features.last().expect("an entry");
    assert_eq!(final_entry.condition, None);
    let written = document.to_json().expect("written");
    assert_eq!(written.matches("\"condition\"").count(), 2, "{written}");
}

/// `minecraft:conditional_list` is registered with `FeatureTypeVersion(InitialRelease,
/// internal = false)`: any pack may use it.
#[test]
fn the_drop_4_conditional_lists_load_in_a_third_party_pack() {
    assert_eq!(every_conditional_list_loads(&cx()), 13);
}

/// The two snap features of the pack: `ice_crystal_…` at 1.26.40 (`vertical_search_range`, the
/// three flags) and `snow_layer_…` at 1.26.50 (`search_range`), both read into the newest
/// struct with their values.
#[test]
fn the_drop_4_snap_features_load_at_both_formats() {
    for (name, range) in [
        ("ice_crystal_snap_to_floor_feature.json", 32),
        ("snow_layer_snap_to_floor_feature.json", 32),
    ] {
        let document = loads(&fixture(&format!("drop4/{name}")), &base_game());
        assert!(
            above_info(document.diagnostics()).is_empty(),
            "{name}: {:#?}",
            document.diagnostics()
        );
        let Feature::SnapToSurface(snap) = document.value() else {
            panic!("{name}: not a snap feature");
        };
        assert_eq!(snap.search_range.0, range, "{name}");
        assert_eq!(snap.surface, Some(Surface::Floor), "{name}");
    }
    let ice = loads(
        &fixture("drop4/ice_crystal_snap_to_floor_feature.json"),
        &base_game(),
    );
    let Feature::SnapToSurface(snap) = ice.value() else {
        panic!("not a snap feature");
    };
    assert_eq!(snap.allow_air_placement, Some(true));
    assert_eq!(snap.allow_underwater_placement, Some(false));
    assert_eq!(snap.embed_in_surface, Some(false));
    assert_eq!(snap.allow_non_air_placement, None);
}

/// A payload body of the engine's unit-test library (`fixtures/feature/case-<n>.json`
/// of 1.26.60, copied under `engine/`) in a document of `version`, with a
/// description added.
fn engine_snap(n: u32, version: &str) -> String {
    let body = fixture(&format!("engine/case-{n}.json"));
    let body = body.trim();
    let inner = body
        .strip_prefix('{')
        .and_then(|b| b.strip_suffix('}'))
        .expect("an object");
    format!(
        r#"{{"format_version": "{version}", "minecraft:snap_to_surface_feature": {{"description": {{"identifier": "test:snap"}}, {inner}}}}}"#
    )
}

/// The engine's snap bodies: the `vertical_search_range` ones read at 1.26.40, the
/// `search_range` one at 1.26.50, and the body without a range (case-30) misses the
/// required member.
#[test]
fn the_engine_snap_bodies_load_at_their_formats() {
    for n in [2, 6, 24, 34] {
        let document = loads(&engine_snap(n, "1.26.40"), &cx());
        assert!(
            above_info(document.diagnostics()).is_empty(),
            "case-{n}: {:#?}",
            document.diagnostics()
        );
    }
    let document = loads(&engine_snap(10, "1.26.50"), &cx());
    let Feature::SnapToSurface(snap) = document.value() else {
        panic!("not a snap feature");
    };
    assert_eq!(snap.surface, Some(Surface::Ceiling));
    let d = rejects(&engine_snap(30, "1.26.50"), &cx());
    error_at(&d, "core/required_child", "search_range");
}

/// Parses `json`, writes it, parses the written text again: the same feature, and the second
/// write identical to the first (as `round_trip.rs`). Returns the written text.
#[track_caller]
fn round_trip(name: &str, json: &str, cx: &LoadContext) -> String {
    let first = loads(json, cx);
    let written = first.to_json().expect("written");
    let second = loads(&written, cx);
    assert_eq!(first.value(), second.value(), "{name}: {written}");
    assert_eq!(second.to_json().expect("written again"), written, "{name}");
    written
}

/// The feature of `json` written in the captures' style: one line, no spaces, no trailing
/// newline.
fn compact(json: &str, cx: &LoadContext) -> String {
    let style = bedrock_addon_core::json::WriteStyle {
        indent: bedrock_addon_core::json::Indent::Compact,
        trailing_newline: false,
    };
    let mut out = Vec::new();
    loads(json, cx)
        .value()
        .write_json(&mut out, style)
        .expect("written compact");
    String::from_utf8(out).expect("UTF-8")
}

/// Every drop-4 conditional list round-trips, and written in the captures' style it is the
/// capture byte for byte but for its `format_version`: the type's one struct is `v1_13_0`, and
/// the writer stamps the struct's version.
#[test]
fn the_drop_4_conditional_lists_round_trip_to_the_capture() {
    for name in CONDITIONAL_LISTS {
        let capture = fixture(&format!("drop4/{name}"));
        round_trip(name, &capture, &cx());
        let expected = capture.replace(
            r#""format_version":"1.26.40""#,
            r#""format_version":"1.13.0""#,
        );
        assert_eq!(compact(&capture, &cx()), expected, "{name}");
    }
}

/// The snap features round-trip. The 1.26.50 file is written as the capture byte for byte.
/// The 1.26.40 file is written at the newest format, `format_version` 1.26.50 and its
/// `vertical_search_range` as `search_range`, every other member as read: the same JSON value,
/// not byte for byte: the writer puts `allowed_surface_blocks` in the struct's order (after
/// `allow_underwater_placement`), where the capture has it before `vertical_search_range`
#[test]
fn the_drop_4_snap_features_round_trip() {
    let snow = fixture("drop4/snow_layer_snap_to_floor_feature.json");
    round_trip("snow", &snow, &base_game());
    assert_eq!(compact(&snow, &base_game()), snow);

    let ice = fixture("drop4/ice_crystal_snap_to_floor_feature.json");
    round_trip("ice", &ice, &base_game());
    let expected = ice
        .replace(
            r#""format_version":"1.26.40""#,
            r#""format_version":"1.26.50""#,
        )
        .replace(r#""vertical_search_range""#, r#""search_range""#);
    let written = compact(&ice, &base_game());
    assert_ne!(written, expected, "now byte for byte: tighten this test");
    let written: bedrock_addon_core::json::Value = facet_json::from_str(&written).expect("JSON");
    let expected: bedrock_addon_core::json::Value = facet_json::from_str(&expected).expect("JSON");
    assert_eq!(written, expected);
}
