//! One `deviation` test per deliberate deviation from vanilla (the others:
//! `upgrade.rs::deviation_i18_custom_components_from_the_list_are_also_keyed`,
//! `upgrade.rs::deviation_i18b_the_legacy_use_duration_keeps_tick_precision`).

use bedrock_addon_core::document::{ComponentRef, Content};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::pack_identity::PackType;
use bedrock_addon_family_item::component::{Digger, UseModifiers};

use super::common::*;

/// 1.20.40 → 1.20.50 `default emit_vibrations`: a `use_modifiers` built only by the
/// moves takes the type's defaults (unverified in vanilla).
#[test]
fn deviation_a_use_modifiers_built_by_the_1_20_50_moves_emits_vibrations() {
    let json = item(
        "1.20.40",
        "minecraft:x",
        r#""minecraft:chargeable": {"movement_modifier": 0.5}"#,
    );
    let document = loads(&json, &allowlisted());
    let modifiers = document.components.get::<UseModifiers>().unwrap();
    assert!(modifiers.emit_vibrations);
    assert!(modifiers.use_duration.abs() < f32::EPSILON);
    assert_eq!(modifiers.movement_modifier, Some(0.5));
}

/// 1.21.50 → 1.21.60 `custom_component_names_to_entries`: each listed name gets
/// `data: {}` (unverified in vanilla).
#[test]
fn deviation_a_listed_custom_component_name_gets_empty_data() {
    let json = item(
        "1.21.50",
        "demo:x",
        r#""minecraft:custom_components": ["demo:shake"]"#,
    );
    let document = loads(&json, &third_party());
    let list = document.deprecated.custom_components_v1.clone().unwrap();
    assert_eq!(
        list,
        bedrock_addon_core::json::from_str_jsonc::<Value>(
            r#"[{"name": "demo:shake", "data": {}}]"#
        )
        .unwrap()
    );
}

/// A listed custom component name that is no string: a number is the text facet makes of it
/// (`1.0` is `1`, `1e2` is `100`) and a `null` is the empty name, each told as an information;
/// a bool or a list is a type error that refuses the item (unverified in vanilla).
#[test]
fn deviation_a_listed_custom_component_name_that_is_no_string_reads_as_facet_made_it() {
    let json = item(
        "1.21.50",
        "demo:x",
        r#""minecraft:custom_components": ["demo:a", 7, 1.5, 1.0, 1e2, null]"#,
    );
    let document = loads(&json, &third_party());
    let names: Vec<String> = document
        .deprecated
        .custom_components_v1
        .as_ref()
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .map(|entry| {
            let name = entry.as_object().unwrap().get("name").unwrap();
            name.as_string().unwrap().as_str().to_owned()
        })
        .collect();
    assert_eq!(names, ["demo:a", "7", "1.5", "1", "100", ""]);
    let told = document
        .diagnostics()
        .iter()
        .filter(|f| f.code.id() == "core/invalid_type")
        .map(|f| f.location.json_path.to_string())
        .collect::<Vec<_>>();
    let at = |i: usize| format!("minecraft:item/components/minecraft:custom_components/{i}");
    assert_eq!(told, [at(1), at(2), at(3), at(4), at(5)]);
    for element in ["true", "[1]"] {
        let json = item(
            "1.21.50",
            "demo:x",
            &format!(r#""minecraft:custom_components": ["demo:a", {element}]"#),
        );
        let d = rejects(&json, &third_party());
        assert!(
            d.iter().any(|f| f.code.id() == "core/invalid_type"
                && f.severity == bedrock_addon_core::diagnostic::Severity::Error
                && f.location.json_path.to_string() == at(1)),
            "{element}: {d:#?}"
        );
    }
}

/// 1.26.0 → 1.26.30 `digger_to_legacy_on_dig_trigger`: a
/// triggered digger is copied to our side-car key `legacy_on_dig_trigger` (vanilla's 1.26.30
/// side-car has no JSON key), and stays in the components.
#[test]
fn deviation_a_triggered_digger_is_copied_to_legacy_on_dig_trigger() {
    let json = item(
        "1.26.0",
        "demo:x",
        r#""minecraft:digger": {"__deprecated_on_dig": {"event": "demo:dug"}, "destroy_speeds": [{"block": "minecraft:dirt", "speed": 2}]}"#,
    );
    let document = loads(&json, &third_party());
    assert!(document.deprecated.legacy_on_dig.is_some());
    assert!(
        document
            .components
            .get::<Digger>()
            .unwrap()
            .deprecated_on_dig
            .is_some()
    );
    // The 1.26.50 format binds neither the side-car nor the digger's trigger, and the
    // 1.26.0 root binds the side-car as a deprecated name the game refuses: the
    // item has no text that reads back, and is refused at the side-car.
    match document.to_json() {
        Err(bedrock_addon_core::json::EncodeError::NotRepresentable { at, .. }) => {
            assert_eq!(at.to_string(), "minecraft:item/__deprecated_components");
        }
        other => panic!("{other:?}"),
    }
}

/// 1.26.0 → 1.26.30 `custom_components_list_to_side_car`: the
/// list moves to our side-car key `custom_components_v1`; the newest writer leaves it out while
/// the keyed entries made from it spell it out again.
#[test]
fn deviation_the_custom_component_list_moves_to_custom_components_v1() {
    let json = item(
        "1.21.80",
        "demo:x",
        // A registered component beside the list: an item of custom components alone has
        // no text in the newest format (`round_trip.rs`, the `k117_` tests).
        r#""minecraft:custom_components": ["demo:shake"], "minecraft:max_stack_size": 16"#,
    );
    let mut cx = third_party();
    cx.beta_apis = true;
    let document = loads(&json, &cx);
    assert!(!document.components.iter().any(
        |e| matches!(e, ComponentRef::Unknown { name, .. } if name == "minecraft:custom_components")
    ));
    assert_eq!(
        document.deprecated.custom_components_v1,
        Some(
            bedrock_addon_core::json::from_str_jsonc::<Value>(
                r#"[{"name": "demo:shake", "data": {}}]"#
            )
            .unwrap()
        )
    );
    assert!(
        document.components.iter().any(
            |e| matches!(e, ComponentRef::Custom(c) if c.name == "demo:shake" && c.from_v1_list)
        )
    );
    let written = document.to_json().unwrap();
    assert!(!written.contains("\"custom_components_v1\""), "{written}");
}

/// A resource-pack `items/` file below 1.16.100 is parsed for validation
/// (vanilla's client loads it); the item registry is the behaviour stack's only.
#[test]
fn deviation_a_resource_pack_item_below_1_16_100_is_parsed_not_registered() {
    let mut rp = third_party();
    rp.pack_type = PackType::Resource;
    let document = loads(&item("1.10", "demo:x", r#""minecraft:icon": "x""#), &rp);
    assert_eq!(
        document.legacy.format,
        bedrock_addon_family_item::ItemFormat::Legacy
    );
    assert_eq!(
        <bedrock_addon_family_item::Item as Content>::SIDES,
        &[bedrock_addon_core::pack_identity::PackSide::Behavior]
    );
}

/// Texts the game does not give are ours (the foil text below
/// 1.20.20 without the legacy creator features).
#[test]
fn deviation_texts_ours_where_vanillas_was_not_recovered() {
    let json = item(
        "1.20.10",
        "demo:x",
        r#""minecraft:foil": true, "minecraft:max_stack_size": 1"#,
    );
    let document = loads(&json, &third_party());
    assert!(has_text(
        document.diagnostics(),
        "minecraft:foil is ignored for this pack; use minecraft:glint"
    ));
    // The text is ours and the game says nothing: an information.
    assert!(
        document
            .diagnostics()
            .iter()
            .filter(|f| f.code.id() == "item/foil_ignored")
            .all(|f| f.severity == bedrock_addon_core::diagnostic::Severity::Info)
    );
}
