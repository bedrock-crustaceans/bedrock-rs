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
        facet_json::from_str::<Value>(r#"[{"name": "demo:shake", "data": {}}]"#).unwrap()
    );
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
        Some(facet_json::from_str::<Value>(r#"[{"name": "demo:shake", "data": {}}]"#).unwrap())
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
