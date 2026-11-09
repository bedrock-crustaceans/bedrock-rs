//! The single-step cases: each runs exactly one
//! `Upgrade::upgrade` on `upgrade/<name>.before.json` and compares the written result with
//! `<name>.after.json`, then checks the findings by code (and text where the row quotes it).

#![allow(
    clippy::disallowed_methods,
    reason = "calls facet-json directly: parses a fixture or written text into a json Value to compare it; not yet moved to the door"
)]

use bedrock_addon::diagnostic::codes;
use bedrock_addon::diagnostic::{Code, Diagnostics, JsonPath, Kind, Severity};
use bedrock_addon::families::block::history::{
    v1_16_0, v1_16_100, v1_18_0, v1_19_10, v1_19_20, v1_19_30, v1_19_40, v1_19_80, v1_20_10,
    v1_20_60, v1_21_30, v1_21_60, v1_21_90, v1_26_0,
};
use bedrock_addon::json::{JsonSlice, Value};
use bedrock_addon::load::LoadContext;
use bedrock_addon::vanilla::ActorType;
use bedrock_addon::version::{FormatVersion, FormatVersionInput};
use bedrock_addon::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct};

use super::common::{allow_listed, context, document, fixture, has, load, loads, rejected};

/// One step on a document's text: the written result (the whole document) and the findings.
fn step<S>(text: &str, cx: &LoadContext) -> (Value, Diagnostics)
where
    S: Upgrade + VersionStruct,
    S::Next: VersionStruct,
{
    step_from::<S>(text, cx, None)
}

/// One step on a document's text, the original version `original` (the document's own when
/// `None`): what a document of that version that reached this step gives.
fn step_from<S>(
    text: &str,
    cx: &LoadContext,
    original: Option<FormatVersion>,
) -> (Value, Diagnostics)
where
    S: Upgrade + VersionStruct,
    S::Next: VersionStruct,
{
    let root: Value = facet_json::from_str(text).expect("fixture JSON");
    let written = root
        .as_object()
        .and_then(|o| o.get("format_version"))
        .and_then(Value::as_string)
        .and_then(|v| FormatVersionInput::parse(v.as_str()))
        .and_then(|v| v.version)
        .expect("a format_version");
    let version = original.unwrap_or(written);
    let parse = ParseCx {
        load: cx,
        written: FormatVersionInput {
            version: Some(version),
            beta_literal: false,
            prerelease: false,
        },
        original_version: version,
        at: JsonPath::new(),
    };
    let mut d = Diagnostics::default();
    let parsed =
        S::parse(JsonSlice::new(text), &parse, &mut d).expect("the before document parses");
    let next = parsed.upgrade(&mut UpgradeCx {
        load: cx,
        original_version: version,
        identifier: "demo:x",
        actor_type: ActorType::Mob,
        skip_section_a: false,
        reading: cx.reading,
        diagnostics: &mut d,
    });
    let written = next.write().expect("written");
    let root = if written
        .as_object()
        .is_some_and(|o| o.contains_key("minecraft:block"))
    {
        written
    } else {
        let mut root = facet_value::VObject::new();
        root.insert("format_version", "1.26.20");
        root.insert("minecraft:block", written);
        root.into()
    };
    (root, d)
}

/// Runs the step of `S` on fixture `name` and compares with its `.after.json`.
#[track_caller]
fn compare<S>(name: &str, cx: &LoadContext) -> Diagnostics
where
    S: Upgrade + VersionStruct,
    S::Next: VersionStruct,
{
    let before = fixture(&format!("upgrade/{name}.before.json"));
    let after: Value =
        facet_json::from_str(&fixture(&format!("upgrade/{name}.after.json"))).expect("after JSON");
    let (out, d) = step::<S>(&before, cx);
    assert_eq!(
        bedrock_addon::json::to_json(&out).unwrap(),
        bedrock_addon::json::to_json(&after).unwrap(),
        "{name}: the step's output differs from the .after.json; findings: {d:#?}"
    );
    d
}

/// The codes of the findings.
fn codes(d: &Diagnostics) -> Vec<String> {
    d.iter().map(|f| f.code.id().to_owned()).collect()
}

/// How many findings have `code`.
fn count(d: &Diagnostics, code: fn(&Code) -> bool) -> usize {
    d.iter().filter(|f| code(&f.code)).count()
}

#[test]
fn s00_1_16_0_description_flags() {
    let d = compare::<v1_16_0::Block>("s00_1_16_0_description_flags", &context());
    assert_eq!(
        count(&d, |c| matches!(c.id(), "core/dropped_field")),
        2,
        "{d:#?}"
    );
    assert_eq!(d.len(), 2, "{d:#?}");
    // Without a format_version the document parses as 1.16.0, with the missing-version
    // information (the game says nothing).
    let text = fixture("upgrade/s00_1_16_0_description_flags.before.json").replacen(
        "\"format_version\": \"1.16.0\",",
        "",
        1,
    );
    let block = loads(&text, &context());
    assert_eq!(block.version().parsed_as, FormatVersion::new(1, 16, 0));
    assert!(
        block
            .diagnostics()
            .iter()
            .any(|f| matches!(f.code.id(), "core/missing_format_version")
                && f.severity == Severity::Info)
    );
}

#[test]
fn s01_1_16_100_renames_and_filter_clamp() {
    let d = compare::<v1_16_100::Block>("s01_1_16_100_renames_and_filter_clamp", &context());
    // The `grid_size: 3` the step removes is noted; the absorption 7.8 is
    // truncated to 7 (told) and 16 is clamped to 15.
    assert_eq!(
        codes(&d),
        [
            "core/dropped_field",
            "core/invalid_value",
            "core/clamped_value"
        ],
        "{d:#?}"
    );
}

#[test]
fn absorption_input_check() {
    for name in ["s01b_absorption_input_check", "s01b_grid_size_input_check"] {
        let text = fixture(&format!("upgrade/{name}.before.json"));
        let (_, d) = step::<v1_16_100::Block>(&text, &context());
        assert!(
            has(
                &d,
                |c| matches!(c.id(), "block/invalid_component"),
                Severity::Error,
                "invalid component"
            ),
            "{d:#?}"
        );
        assert!(has(
            &d,
            |c| matches!(c.id(), "block/upgrade_failed"),
            Severity::Error,
            "Failed to upgrade to version '1.18.0' from document version '1.16.100'"
        ));
        assert!(has(
            &d,
            |c| matches!(c.id(), "block/final_upgrade_failed"),
            Severity::Error,
            "Final json version"
        ));
        assert!(
            load(&text, &context()).is_err(),
            "{name}: the block is not loaded"
        );
    }
}

#[test]
fn s02_1_18_0_unwrap_all() {
    let d = compare::<v1_18_0::Block>("s02_1_18_0_unwrap_all", &context());
    assert!(
        has(&d, |c| c.is(&codes::DROPPED_FIELD), Severity::Info, ""),
        "{d:#?}"
    );
    // The unwrap's text is its own, not the collision upgrade's.
    assert!(
        !d.iter().any(|f| f
            .code
            .to_string()
            .contains("minecraft:block_collision will be thrown away")),
        "{d:#?}"
    );
    assert!(has(
        &d,
        |c| matches!(c.id(), "block/component_missing_field"),
        Severity::Error,
        "Component minecraft:flammable is missing required field burn_odds, component will be discarded"
    ));
}

#[test]
fn unwrap_missing_field() {
    let d = compare::<v1_18_0::Block>("s02b_unwrap_missing_field", &context());
    assert!(
        has(
            &d,
            |c| matches!(c.id(), "block/component_missing_field"),
            Severity::Error,
            "Component minecraft:loot is missing required field loot_table, component will be discarded"
        ),
        "{d:#?}"
    );
    assert_eq!(
        count(&d, |c| matches!(c.id(), "core/clamped_value")),
        1,
        "{d:#?}"
    );
}

#[test]
fn s03_1_19_10_menu_friction_destructible() {
    let d = compare::<v1_19_10::Block>("s03_1_19_10_menu_friction_destructible", &context());
    assert_eq!(codes(&d), ["core/clamped_value"], "{d:#?}");
}

#[test]
fn friction_input_check() {
    let text = fixture("upgrade/s03b_friction_input_check.before.json");
    let (_, d) = step::<v1_19_10::Block>(&text, &context());
    assert!(
        has(
            &d,
            |c| matches!(c.id(), "block/invalid_component"),
            Severity::Error,
            "minecraft:friction:  invalid component"
        ),
        "{d:#?}"
    );
    assert!(load(&text, &context()).is_err(), "the block is not loaded");
}

#[test]
fn permutation_creative_category() {
    let d = compare::<v1_19_10::Block>("s03c_permutation_creative_category", &context());
    // The base's category is replaced by the permutation's (the last one wins): noted at
    // the base's component.
    let told: Vec<String> = d
        .iter()
        .map(|f| format!("{:?} at {}", f.severity, f.location.json_path))
        .collect();
    assert_eq!(codes(&d), ["core/dropped_field"], "{d:#?}");
    assert_eq!(
        told,
        ["Info at minecraft:block/components/minecraft:creative_category"]
    );
}

#[test]
fn s04_1_19_20_display_name_key() {
    let d = compare::<v1_19_20::Block>("s04_1_19_20_display_name_key", &context());
    assert!(d.is_empty(), "{d:#?}");
}

#[test]
fn s05_1_19_30_menu_category_rules() {
    type Row<'a> = (Option<&'a str>, Option<&'a str>, bool, Option<&'a str>);

    let d = compare::<v1_19_30::Block>("s05_1_19_30_menu_category_rules", &context());
    assert!(d.is_empty(), "{d:#?}");
    // The seven rows: (written, category out, is_hidden_in_commands, logged).
    let rows: [Row<'_>; 7] = [
        (None, None, false, None),
        (
            Some(""),
            Some("none"),
            true,
            Some("Block Menu Category should not have an empty value"),
        ),
        (
            Some("bogus"),
            Some("none"),
            true,
            Some("Block Menu Category has an invalid value of bogus"),
        ),
        (Some("all"), Some("none"), true, None),
        (Some("commands"), Some("none"), false, None),
        (Some("none"), Some("none"), true, None),
        (Some("Nature"), Some("Nature"), false, None),
    ];
    for (written, category, hidden, logged) in rows {
        let menu = match written {
            Some(w) => format!(r#"{{"category": "{w}"}}"#),
            None => r#"{"category": 7}"#.to_owned(),
        };
        let text = document(
            "1.19.30",
            &format!(
                r#"{{"description": {{"identifier": "demo:x", "menu_category": {menu}}}, "components": {{}}}}"#
            ),
        );
        let (out, d) = step::<v1_19_30::Block>(&text, &context());
        let out_menu = out
            .as_object()
            .and_then(|o| o.get("minecraft:block"))
            .and_then(|p| p.as_object()?.get("description"))
            .and_then(|p| p.as_object()?.get("menu_category"))
            .and_then(Value::as_object)
            .expect("menu_category");
        let out_category = out_menu
            .get("category")
            .and_then(Value::as_string)
            .map(|s| s.as_str().to_owned());
        if let Some(category) = category {
            assert_eq!(out_category.as_deref(), Some(category), "{written:?}");
        } else {
            assert!(
                out_menu.get("category").is_some_and(|c| !c.is_string()),
                "a non-string is left alone"
            );
        }
        assert_eq!(
            out_menu
                .get("is_hidden_in_commands")
                .and_then(Value::as_bool),
            Some(hidden),
            "{written:?}"
        );
        match logged {
            Some(text) => assert!(
                has(
                    &d,
                    |c| c.kind() == Kind::InvalidValue,
                    Severity::Error,
                    text
                ),
                "{written:?}: {d:#?}"
            ),
            // `commands` and `all` are written `none` with no log of the game's: our note.
            None if matches!(written, Some("commands" | "all")) => {
                assert_eq!(d.len(), 1, "{d:#?}");
                assert!(
                    has(&d, |c| c.kind() == Kind::Deprecated, Severity::Info, ""),
                    "{d:#?}"
                );
            }
            None => assert!(d.is_empty(), "{written:?}: {d:#?}"),
        }
    }
}

#[test]
fn states_gate() {
    let with_states = |version: &str, states: &str, permutations: bool| {
        let permutations = if permutations {
            r#", "permutations": [{"condition": "q.block_state('demo:b') == 1", "components": {}}]"#
        } else {
            ""
        };
        document(
            version,
            &format!(
                r#"{{"description": {{"identifier": "demo:x", "states": {states}}}, "components": {{}}{permutations}}}"#
            ),
        )
    };
    // 1.19.50: the states are dropped.
    let block = loads(
        &with_states("1.19.50", r#"{"demo:b": [false, true]}"#, false),
        &context(),
    );
    assert!(block.description.states.is_empty());
    assert!(
        block
            .diagnostics()
            .iter()
            .any(|f| matches!(f.code.id(), "core/dropped_field"))
    );
    // 1.19.60: non-empty states reject the block; empty ones pass.
    let d = rejected(
        &with_states("1.19.60", r#"{"demo:b": [false, true]}"#, false),
        &context(),
    );
    assert!(
        has(
            &d,
            |c| matches!(c.id(), "block/custom_properties_version"),
            Severity::Error,
            "To use custom block properties, use format version 1.19.70 or higher"
        ),
        "{d:#?}"
    );
    loads(&with_states("1.19.60", "{}", false), &context());
    // 1.19.60 permutations are skipped with vanilla's text (typo kept).
    let block = loads(&with_states("1.19.60", "{}", true), &context());
    assert!(block.permutations.is_empty());
    assert!(
        block
            .diagnostics()
            .iter()
            .any(|f| f.code.to_string().contains("format verison 1.19.70"))
    );
    // With the legacy creator features everything loads.
    for (version, permutations) in [("1.19.50", false), ("1.19.60", false), ("1.19.60", true)] {
        let block = loads(
            &with_states(version, r#"{"demo:b": [false, true]}"#, permutations),
            &allow_listed(),
        );
        assert_eq!(block.description.states.len(), 1, "{version}");
        assert_eq!(
            block.permutations.len(),
            usize::from(permutations),
            "{version}"
        );
    }
}

#[test]
fn s06_1_19_40_rotation_part_visibility() {
    let d = compare::<v1_19_40::Block>("s06_1_19_40_rotation_part_visibility", &context());
    assert!(
        d.iter().any(|f| f
            .code
            .args_of(&codes::DUPLICATE_COMPONENT_VERSION)
            .is_some_and(
                |args| args[0] == "minecraft:rotation" && args[1] == "minecraft:transformation"
            )),
        "{d:#?}"
    );
    assert!(has(
        &d,
        |c| matches!(c.id(), "block/part_visibility_deprecated"),
        Severity::Info,
        "The part_visibility component has been deprecated"
    ));
    assert!(has(
        &d,
        |c| matches!(c.id(), "block/part_visibility_unsupported"),
        Severity::Info,
        "The part_visibility component is no longer supported"
    ));
}

#[test]
fn s07_1_19_80_bone_visibility_bools() {
    let d = compare::<v1_19_80::Block>("s07_1_19_80_bone_visibility_bools", &context());
    assert_eq!(d.len(), 1, "{d:#?}");
    assert!(has(
        &d,
        |c| matches!(c.id(), "block/bone_visibility_boolean"),
        Severity::Error,
        "bone_visibility | lid: Only boolean values are supported for bone visibilities in format versions before 1.20.10."
    ));
}

#[test]
fn s08_1_20_10_unit_cube_sections() {
    let d = compare::<v1_20_10::Block>("s08_1_20_10_unit_cube_sections", &context());
    assert_eq!(d.len(), 1, "{d:#?}");
    assert!(has(
        &d,
        |c| matches!(c.id(), "block/unit_cube_and_geometry"),
        Severity::Error,
        "A block cannot have both a unit cube and a geometry component."
    ));
}

#[test]
fn unit_cube_deprecated() {
    let text = document(
        "1.20.60",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:unit_cube": {}}}"#,
    );
    let block = loads(&text, &context());
    assert!(block.deprecated.unit_cube.is_none(), "no side-car");
    assert!(
        block
            .diagnostics()
            .iter()
            .any(|f| matches!(f.code.id(), "block/component_not_valid_from")
                && f.code
                    .to_string()
                    .contains("minecraft:unit_cube component is not valid from 1.20.60 onward.")),
        "{:#?}",
        block.diagnostics()
    );
}

#[test]
fn property_to_state() {
    let d = compare::<v1_20_10::Block>("s08c_property_to_state", &context());
    // The fixture's own `states.demo:lit` is replaced by the property of that name, which
    // lists other values: noted at the state.
    let told: Vec<String> = d
        .iter()
        .map(|f| format!("{:?} at {}", f.severity, f.location.json_path))
        .collect();
    assert_eq!(codes(&d), ["core/dropped_field"], "{d:#?}");
    assert_eq!(
        told,
        ["Info at minecraft:block/description/states/demo:lit"]
    );
    // The same input at 1.20.20: unchanged except `properties`, dropped with `DroppedField`.
    let before = fixture("upgrade/s08c_property_to_state.before.json");
    let at_1_20_20 = before.replacen(
        "\"format_version\": \"1.20.10\"",
        "\"format_version\": \"1.20.20\"",
        1,
    );
    let (out, d) = step::<v1_20_10::Block>(&at_1_20_20, &context());
    let text = bedrock_addon::json::to_json(&out).unwrap();
    assert!(
        !text.contains("properties")
            && text.contains("block_property")
            && !text.contains("set_block_state"),
        "{text}"
    );
    assert_eq!(codes(&d), ["core/dropped_field"], "{d:#?}");
    // A 1.19.50 document with `properties` and no legacy creator features: converted, then the
    // states gate drops the states.
    let old = document(
        "1.19.50",
        r#"{"description": {"identifier": "demo:x", "properties": {"demo:open": [false, true]}}, "components": {}}"#,
    );
    let block = loads(&old, &context());
    assert!(block.description.states.is_empty());
    assert!(
        block
            .diagnostics()
            .iter()
            .any(|f| matches!(f.code.id(), "core/dropped_field")
                && f.location
                    .json_path
                    .to_string()
                    // Told at the member the file has (`properties`), not at the
                    // `states` the 1.20.10 step made of it.
                    .ends_with("description/properties")),
        "{:#?}",
        block.diagnostics()
    );
}

#[test]
fn s09_1_20_60_redstone_synthesis() {
    let d = compare::<v1_20_60::Block>("s09_1_20_60_redstone_synthesis", &context());
    assert_eq!(
        codes(&d),
        ["core/dropped_field"],
        "item_specific_speeds dropped: {d:#?}"
    );
    let d = compare::<v1_20_60::Block>("s09_opaque_without_unit_cube", &context());
    assert!(d.is_empty(), "{d:#?}");
}

#[test]
fn s10_1_21_30_group_namespace() {
    let d = compare::<v1_21_30::Block>("s10_1_21_30_group_namespace", &context());
    // The namespaced group is told as the item's is.
    let told: Vec<String> = d
        .iter()
        .map(|f| format!("{} {:?} {}", f.code.id(), f.severity, f.location.json_path))
        .collect();
    assert_eq!(
        told,
        ["core/deprecated_member Info minecraft:block/description/menu_category/group"]
    );
    compare::<v1_21_30::Block>("s10_numeric_group", &context());
}

#[test]
fn s11_1_21_60_custom_components_v1() {
    let d = compare::<v1_21_60::Block>("s11_1_21_60_custom_components_v1", &context());
    assert!(d.is_empty(), "{d:#?}");
}

#[test]
fn custom_components_deprecated() {
    let text = document(
        "1.21.90",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:custom_components": ["demo:glow"]}}"#,
    );
    let block = loads(&text, &context());
    assert!(
        !block.components.iter().any(|c| matches!(
            c,
            bedrock_addon::document::ComponentRef::Unknown { .. }
                | bedrock_addon::document::ComponentRef::Custom(_)
        )),
        "the list is removed"
    );
    assert!(block.diagnostics().iter().any(|f| {
        matches!(f.code.id(), "block/component_not_valid_from")
            && f.code
                .to_string()
                .contains("minecraft:custom_components component is not valid from 1.21.90 onward.")
    }));
}

#[test]
fn s12_1_21_90_liquid_clipping() {
    let d = compare::<v1_21_90::Block>("s12_1_21_90_liquid_clipping", &context());
    assert!(d.is_empty(), "{d:#?}");
}

#[test]
fn s13_1_26_0_conversion() {
    // At 1.26.0 the deprecated `unit_cube` is removed (from 1.20.60) before the
    // side-car, the triggers and `breathability` move into it.
    let d = compare::<v1_26_0::Block>("s13_1_26_0_conversion", &allow_listed());
    assert_eq!(count(&d, |c| c.kind() == Kind::Deprecated), 3, "{d:#?}");
    assert!(
        d.iter().any(|f| f.code.to_string()
            == "minecraft:unit_cube component is not valid from 1.20.60 onward.")
    );
    // A document below 1.20.60 that reached this step keeps the unit cube in its side-car.
    let before = fixture("upgrade/s13_1_26_0_conversion.before.json");
    let (out, _) = step_from::<v1_26_0::Block>(
        &before,
        &allow_listed(),
        Some(FormatVersion::new(1, 20, 10)),
    );
    let after: Value =
        facet_json::from_str(&fixture("upgrade/s13_unit_cube_below_1_20_60.after.json")).unwrap();
    assert_eq!(
        bedrock_addon::json::to_json(&out).unwrap(),
        bedrock_addon::json::to_json(&after).unwrap()
    );
    let block = loads(
        &fixture("upgrade/s13_1_26_0_conversion.before.json"),
        &allow_listed(),
    );
    assert_eq!(block.source_format_version, FormatVersion::new(1, 26, 0));
}

#[test]
fn events_without_requirement() {
    // The legacy `breathability` of the same input needs the legacy creator features too, so
    // the block is not loaded; the events and the trigger are still reported.
    let d = rejected(
        &fixture("upgrade/s13_1_26_0_conversion.before.json"),
        &context(),
    );
    assert!(
        d.iter().any(|f| f.code.is(&codes::MISSING_TOGGLES)
            && f.severity == Severity::Error
            && f.location
                .json_path
                .to_string()
                .ends_with("components/minecraft:breathability")),
        "{d:#?}"
    );
    let d = &d;
    assert!(
        d.iter().any(|f| matches!(f.code.id(), "core/dropped_field")
            && f.location.json_path.to_string().ends_with("events")),
        "{d:#?}"
    );
    assert!(
        d.iter()
            .any(|f| matches!(f.code.id(), "core/unrecognized_component")
                && f.location
                    .json_path
                    .to_string()
                    .ends_with("minecraft:on_step_on")),
        "{d:#?}"
    );
}

#[test]
fn chest_obstruction() {
    use bedrock_addon::families::block::component::{ChestObstruction, ObstructionRule};
    let text = fixture("upgrade/s13c_chest_obstruction.before.json");
    let rule = |block: &bedrock_addon::families::block::Block, permutation: Option<usize>| {
        let map = match permutation {
            None => &block.components,
            Some(i) => &block.permutations[i].components,
        };
        map.get::<ChestObstruction>()
            .and_then(|c| c.obstruction_rule)
    };
    let block = loads(&text, &allow_listed());
    assert_eq!(rule(&block, None), Some(ObstructionRule::Always));
    assert_eq!(rule(&block, Some(0)), Some(ObstructionRule::Never));
    let later = text.replacen(
        "\"format_version\": \"1.26.0\"",
        "\"format_version\": \"1.26.10\"",
        1,
    );
    let block = loads(&later, &allow_listed());
    assert_eq!(rule(&block, None), None, "nothing inserted at 1.26.10");
}

/// After the 1.19.40 fixer keeps the author's case, the 1.26.20 parse reads the category
/// case-insensitively.
#[test]
fn a_kept_category_in_another_case_loads() {
    let text = document(
        "1.19.30",
        r#"{"description": {"identifier": "demo:x", "menu_category": {"category": "Nature"}}, "components": {}}"#,
    );
    let block = loads(&text, &context());
    assert_eq!(
        block
            .description
            .menu_category
            .as_ref()
            .and_then(|m| m.category),
        Some(bedrock_addon::shared::CreativeCategory::Nature)
    );
}

/// `block_light_emission {}` does not read its `emission` as 0: the game's 1.19.10
/// upgrade removes the component if `emission` is missing, before the unwrap: the component goes, in the base and in a permutation, each with the
/// game's level-3 line, a kept error.
#[test]
fn an_emission_object_without_emission_is_discarded_with_the_game_line() {
    let text = document(
        "1.18.0",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:block_light_emission": {}},
            "permutations": [{"condition": "1", "components": {"minecraft:block_light_emission": {}}}]}"#,
    );
    let (out, d) = step::<v1_18_0::Block>(&text, &context());
    let text = bedrock_addon::json::to_json(&out).unwrap();
    assert!(!text.contains("light_emission"), "{text}");
    let told: Vec<&bedrock_addon::diagnostic::Diagnostic> = d
        .iter()
        .filter(|f| {
            f.kept
                && f.code.to_string()
                    == "Component minecraft:block_light_emission is missing required field emission, component will be discarded"
        })
        .collect();
    assert_eq!(told.len(), 2, "{d:#?}");
    assert_eq!(d.len(), 2, "{d:#?}");
}

/// A `ticking` without `range` names `range` in vanilla's text.
#[test]
fn a_ticking_without_range_names_range() {
    let text = document(
        "1.18.0",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:ticking": {"on_tick": {"event": "e"}}}}"#,
    );
    let (_, d) = step::<v1_18_0::Block>(&text, &context());
    assert!(d.iter().any(|f| matches!(f.code.id(), "block/component_missing_field")
        && f.code.to_string()
            == "Component minecraft:ticking is missing required field range, component will be discarded"));
}

/// The game's missing-field removal looks only at a component that is an
/// object and only for the member's presence:
/// a `ticking` that is no object, a `range` that is no array and a `flammable` odds member
/// written `null` are not discarded with the missing-field line.
#[test]
fn only_an_object_without_the_member_is_discarded_with_the_missing_field_line() {
    for components in [
        r#""minecraft:ticking": 7"#,
        r#""minecraft:ticking": {"on_tick": {"event": "e"}, "range": 3}"#,
        r#""minecraft:flammable": {"flame_odds": null, "burn_odds": 3}"#,
    ] {
        let text = document(
            "1.18.0",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{{components}}}, "events": {{"e": {{}}}}}}"#
            ),
        );
        let (out, d) = step::<v1_18_0::Block>(&text, &context());
        assert!(
            !d.iter()
                .any(|f| f.code.to_string().contains("is missing required field")),
            "{components}: {d:#?}"
        );
        let text = bedrock_addon::json::to_json(&out).unwrap();
        assert!(
            text.contains("queued_ticking") || text.contains("flammable"),
            "{components}: {text}"
        );
    }
    // The `null` odds the typed read takes as the default: told at the member (ours).
    let text = document(
        "1.18.0",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:flammable": {"flame_odds": null, "burn_odds": 3}}}"#,
    );
    let (_, d) = step::<v1_18_0::Block>(&text, &context());
    assert!(
        d.iter().any(|f| matches!(f.code.id(), "core/invalid_type")
            && f.severity == Severity::Info
            && f.location.json_path.to_string()
                == "minecraft:block/components/minecraft:flammable/flame_odds"),
        "{d:#?}"
    );
}
