//! The upgrade steps on their fixture pairs: each pair runs one
//! `Upgrade::upgrade` on `<name>.before.json` and compares the result with `<name>.after.json`
//! parsed by the target struct at the source's `original_version`; `legacy.shared` (the raw
//! values the legacy reader keeps) is compared apart.

use std::fmt::Debug;

use bedrock_addon_core::diagnostic::{Code, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::{ComponentRef, CustomComponent};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct};
use bedrock_addon_family_item::component::{
    BlockPlacer, Damage, Digger, Food, StartUsing, StorageItem, StorageWeightLimit,
    StorageWeightModifier, UseModifiers, Wearable,
};
use bedrock_addon_family_item::history::{self, Component, ItemDocument};
use bedrock_addon_family_item::{Item, ItemFormat};
use bedrock_addon_shared_shapes::CreativeCategory;
use facet_json::RawJson;
use indexmap::IndexMap;

use super::common::*;

/// A fixture file of `upgrade/`.
fn fixture(name: &str) -> String {
    let path = format!(
        "{}/../../addon/tests/fixtures/item/item/upgrade/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The payload text and the version of a document.
fn split(text: &str) -> (String, FormatVersion) {
    let root: IndexMap<String, RawJson<'_>> = facet_json::from_str_borrowed_jsonc(text).unwrap();
    let version: String = facet_json::from_str(root["format_version"].as_str()).unwrap();
    let parts: Vec<u16> = version.split('.').map(|p| p.parse().unwrap()).collect();
    (
        root["minecraft:item"].as_str().to_owned(),
        FormatVersion::new(parts[0], parts[1], parts[2]),
    )
}

/// The parse context of a document at `original`.
fn parse_cx(load: &LoadContext, original: FormatVersion) -> ParseCx<'_> {
    let mut at = JsonPath::new();
    at.push("minecraft:item");
    ParseCx {
        load,
        written: FormatVersionInput {
            version: Some(original),
            beta_literal: false,
            prerelease: false,
        },
        original_version: original,
        at,
    }
}

/// The struct parses `payload` at `original`.
fn parse<S: VersionStruct>(
    payload: &str,
    cx: &LoadContext,
    original: FormatVersion,
) -> (S, Diagnostics) {
    let mut d = Diagnostics::default();
    let value = S::parse(JsonSlice::new(payload), &parse_cx(cx, original), &mut d)
        .unwrap_or_else(|| panic!("rejected: {d:#?}"));
    (value, d)
}

/// Runs `S`'s step.
fn upgrade<S: Upgrade>(
    value: S,
    cx: &LoadContext,
    original: FormatVersion,
    identifier: &str,
) -> (S::Next, Diagnostics) {
    let mut d = Diagnostics::default();
    let next = value.upgrade(&mut UpgradeCx {
        load: cx,
        original_version: original,
        identifier,
        actor_type: bedrock_addon_core::vanilla::ActorType::Mob,
        skip_section_a: false,
        reading: cx.reading,
        diagnostics: &mut d,
    });
    (next, d)
}

/// The raw values the legacy reader keeps differ by what each side's JSON holds.
trait Shared {
    fn clear_shared(&mut self);
}

impl<const V: u32> Shared for ItemDocument<V> {
    fn clear_shared(&mut self) {
        self.legacy.shared.clear();
        self.legacy.category = None;
    }
}

/// The newest format reads no side-car (v18): the step's side-car is checked apart.
impl Shared for Item {
    fn clear_shared(&mut self) {
        self.legacy.shared.clear();
        self.legacy.category = None;
        self.deprecated = bedrock_addon_family_item::ItemDeprecatedData::default();
    }
}

/// The 1.26.30 struct reads no side-car either (v18).
impl Shared for history::v1_26_30::Item {
    fn clear_shared(&mut self) {
        self.legacy.shared.clear();
        self.legacy.category = None;
        self.deprecated = bedrock_addon_family_item::ItemDeprecatedData::default();
    }
}

/// Runs the step of fixture `name` from struct `S` at `original` (the before file's version
/// when `None`), compares it with the after file, and returns the result and the step's
/// findings.
#[track_caller]
fn step<S>(name: &str, original: Option<FormatVersion>, cx: &LoadContext) -> (S::Next, Diagnostics)
where
    S: Upgrade + VersionStruct,
    S::Next: VersionStruct + PartialEq + Debug + Shared + Clone,
{
    let (before, version) = split(&fixture(&format!("{name}.before.json")));
    let original = original.unwrap_or(version);
    let (after, _) = split(&fixture(&format!("{name}.after.json")));
    let (value, _) = parse::<S>(&before, cx, original);
    let (next, d) = upgrade(value, cx, original, "minecraft:x");
    let (mut expected, _) = parse::<S::Next>(&after, cx, original);
    let mut got = next.clone();
    got.clear_shared();
    expected.clear_shared();
    assert_eq!(got, expected, "{name}");
    (next, d)
}

/// The findings of `d` with code `name`.
fn codes(d: &Diagnostics, name: &str) -> usize {
    with_code(d, name).len()
}

#[test]
fn the_1_18_fixers_rename_and_map() {
    let (_, d) = step::<history::v1_16_100::Item>("i01_1_16_100_fixers_118", None, &allowlisted());
    // the string list of repair items is removed as a whole member
    assert_eq!(codes(&d, "core/dropped_field"), 1);
    // every saturation name, `high` included, and an unknown one
    for (name, expected) in [
        ("poor", 0.1),
        ("low", 0.3),
        ("normal", 0.6),
        ("good", 0.8),
        ("max", 1.0),
        ("supernatural", 1.2),
        ("high", 0.6),
        ("weird", 0.6),
    ] {
        let json = item(
            "1.16.100",
            "minecraft:x",
            &format!(r#""minecraft:food": {{"nutrition": 1, "saturation_modifier": "{name}"}}"#),
        );
        let document = loads(&json, &allowlisted());
        let food = document.components.get::<Food>().unwrap();
        assert!(
            (food.saturation_modifier.unwrap() - expected).abs() < 1e-6,
            "{name}"
        );
        let flagged = has_text(
            document.diagnostics(),
            "unknown saturation_modifier type 'weird'",
        );
        assert_eq!(flagged, name == "weird", "{name}");
    }
    // an object list is left intact
    let json = item(
        "1.16.100",
        "minecraft:x",
        r#""minecraft:repairable": {"repair_items": [{"items": ["minecraft:stick"], "repair_amount": 5}]}"#,
    );
    let document = loads(&json, &allowlisted());
    let repairable = document
        .components
        .get::<bedrock_addon_family_item::component::Repairable>()
        .unwrap();
    assert_eq!(repairable.repair_items.len(), 1);
}

#[test]
fn an_unknown_saturation_name_keeps_the_item() {
    // `FoodItem118Upgrade::upgradeToNext` logs it with `_contentLog(true, 3, 17)`
    // and goes on with 0.6: a kept error
    let json = item(
        "1.16.100",
        "minecraft:x",
        r#""minecraft:food": {"saturation_modifier": "weird"}"#,
    );
    let document = loads(&json, &allowlisted());
    let finding = document
        .diagnostics()
        .iter()
        .find(|f| f.code.id() == "item/saturation_modifier")
        .unwrap();
    assert_eq!((finding.severity, finding.kept), (Severity::Error, true));
}

#[test]
fn the_old_shapes_at_1_18_0_refuse_the_item() {
    // At 1.18 the slice binds the new shapes: a string `saturation_modifier` is a float of
    // another type (0x8), a durability without `max_durability` misses a required member
    // (0x80), `max_damage` is an unknown member (0x4, level 2). The read fails
    // and `makeError` refuses the item. The pre-1.18 name
    // list of `repair_items` is read by `alsoReadAs<mItems>` as the entries'
    // `items`: ours, an information, the value dropped as before (open: keep it).
    let json = item(
        "1.18.0",
        "minecraft:x",
        r#""minecraft:food": {"nutrition": 4, "saturation_modifier": "good"}, "minecraft:durability": {"max_damage": 250}, "minecraft:repairable": {"repair_items": ["minecraft:stick"]}"#,
    );
    let d = rejects(&json, &allowlisted());
    let told: Vec<(String, Severity)> = d
        .iter()
        .filter(|f| {
            f.code.id() == "core/invalid_value" || f.code.id() == "core/missing_required_field"
        })
        .map(|f| (f.location.json_path.to_string(), f.severity))
        .collect();
    assert_eq!(
        told,
        [
            (
                "minecraft:item/components/minecraft:food/saturation_modifier".to_owned(),
                Severity::Error
            ),
            (
                "minecraft:item/components/minecraft:durability/max_damage".to_owned(),
                Severity::Warning
            ),
            (
                "minecraft:item/components/minecraft:durability/max_durability".to_owned(),
                Severity::Error
            ),
            (
                "minecraft:item/components/minecraft:repairable/repair_items".to_owned(),
                Severity::Info
            ),
        ],
        "{d:#?}"
    );
}

#[test]
fn the_category_is_copied_and_a_missing_one_stays_none() {
    let (doc, _) =
        step::<history::v1_19_83::Item>("i02_1_19_83_typed_description", None, &allowlisted());
    assert_eq!(doc.description.category.as_deref(), Some("equipment"));
    let (doc, _) = step::<history::v1_19_83::Item>(
        "i02_1_19_83_typed_description_missing",
        None,
        &allowlisted(),
    );
    assert_eq!(doc.description.category, None);
}

#[test]
fn armor_becomes_wearable_protection_and_category_menu_category() {
    let (doc, d) = step::<history::v1_20_0::Item>("i03_1_20_0_armor_menu", None, &allowlisted());
    // `armor.texture_type`, and the fixture's `minecraft:foil`, which no format from 1.20.20
    // on reads: the step that leaves the formats that read it tells that it goes.
    assert_eq!(codes(&d, "core/dropped_field"), 2, "texture_type, foil");
    let Some(Component::Wearable(wearable)) = doc.components.get("minecraft:wearable") else {
        panic!("no wearable");
    };
    assert_eq!(wearable.protection, 3);
    assert!(!doc.components.contains_key("minecraft:armor"));
    assert_eq!(
        doc.description.menu_category.as_ref().unwrap().category,
        Some(CreativeCategory::Equipment)
    );
    // foil stays a legacy read: is_glint, and no glint is created
    assert_eq!(doc.legacy.is_glint, Some(true));
    assert!(!doc.components.contains_key("minecraft:glint"));
}

#[test]
fn armor_without_wearable_is_an_error_and_removed() {
    let (doc, d) =
        step::<history::v1_20_0::Item>("i03b_armor_without_wearable", None, &allowlisted());
    assert!(!doc.components.contains_key("minecraft:armor"));
    let error = d
        .iter()
        .find(|f| f.severity == Severity::Error)
        .expect("an error");
    assert_eq!(
        error.code.to_string(),
        "Item must have a 'minecraft:wearable' component to be equippable"
    );
}

#[test]
fn on_consume_moves_to_the_side_car() {
    let (doc, d) = step::<history::v1_20_20::Item>("i04_1_20_20_on_consume", None, &allowlisted());
    assert_eq!(
        doc.deprecated.on_consume.as_ref().unwrap().event.as_deref(),
        Some("demo:ate")
    );
    assert_eq!(codes(&d, "core/deprecated_member"), 1);
}

#[test]
fn the_six_triggers_and_render_offsets_move_whole() {
    // render_offsets reaches the step only from a document below its deprecation (1.20.10)
    let original = Some(FormatVersion::new(1, 20, 0));
    let (doc, d) = step::<history::v1_20_30::Item>(
        "i05_1_20_30_triggers_render_offsets",
        original,
        &allowlisted(),
    );
    let side = &doc.deprecated;
    assert_eq!(
        side.on_use.as_ref().unwrap().event.as_deref(),
        Some("demo:use")
    );
    assert_eq!(
        side.on_use_on.as_ref().unwrap().event.as_deref(),
        Some("demo:use_on")
    );
    assert_eq!(
        side.on_hurt_actor.as_ref().unwrap().event.as_deref(),
        Some("demo:hurt")
    );
    assert_eq!(
        side.on_hit_actor.as_ref().unwrap().event.as_deref(),
        Some("demo:miss")
    );
    assert_eq!(
        side.on_hit_block.as_ref().unwrap().event.as_deref(),
        Some("demo:block")
    );
    assert!(side.render_offsets.is_some());
    assert_eq!(codes(&d, "core/deprecated_member"), 6);
    for gone in [
        "minecraft:on_use",
        "minecraft:on_use_on",
        "minecraft:weapon",
        "minecraft:render_offsets",
    ] {
        assert!(!doc.components.contains_key(gone), "{gone}");
    }
}

#[test]
fn use_modifiers_chargeable_digger_and_tags() {
    let (doc, _) =
        step::<history::v1_20_40::Item>("i06_1_20_40_use_modifiers_digger", None, &allowlisted());
    let Some(Component::UseModifiers(modifiers)) = doc.components.get("minecraft:use_modifiers")
    else {
        panic!("no use_modifiers");
    };
    assert!((modifiers.use_duration - 1.6).abs() < 1e-6);
    assert_eq!(modifiers.movement_modifier, Some(0.35));
    // the defaults of a use_modifiers built by the moves (unverified)
    assert!(modifiers.emit_vibrations);
    assert!(!doc.components.contains_key("minecraft:chargeable"));
    assert_eq!(
        doc.deprecated
            .on_complete
            .as_ref()
            .unwrap()
            .event
            .as_deref(),
        Some("demo:charged")
    );
    let Some(Component::Digger(digger)) = doc.components.get("minecraft:digger") else {
        panic!("no digger");
    };
    assert_eq!(
        digger.deprecated_on_dig.as_ref().unwrap().event.as_deref(),
        Some("demo:dug")
    );
    assert_eq!(
        digger.destroy_speeds[0]
            .deprecated_on_dig
            .as_ref()
            .unwrap()
            .event
            .as_deref(),
        Some("demo:dirt")
    );
    let Some(Component::Tags(tags)) = doc.components.get("minecraft:tags") else {
        panic!("no tags");
    };
    assert_eq!(tags.tags, ["demo:blunt"]);
    assert!(doc.legacy.tags.is_empty());
}

#[test]
fn icon_texture_becomes_textures_default() {
    let (doc, _) =
        step::<history::v1_20_50::Item>("i07_1_20_50_icon_textures", None, &allowlisted());
    let Some(Component::Icon(icon)) = doc.components.get("minecraft:icon") else {
        panic!("no icon");
    };
    assert_eq!(icon.default_texture(), Some("demo_sword"));
}

/// A plain-copy step: the output is the input.
fn copies<S>(version: &str, components: &str)
where
    S: Upgrade + VersionStruct + Clone,
    S::Next: VersionStruct + Debug,
{
    let json = item(version, "minecraft:x", components);
    let (payload, original) = split(&json);
    let (value, _) = parse::<S>(&payload, &allowlisted(), original);
    let written = value.write().unwrap();
    let (next, d) = upgrade(value, &allowlisted(), original, "minecraft:x");
    assert!(d.is_empty(), "{version}: {d:#?}");
    assert_eq!(next.write().unwrap(), written, "{version}");
}

#[test]
fn the_plain_copies_change_nothing_but_replace_block_item() {
    let components = r#""minecraft:hand_equipped": true, "minecraft:max_stack_size": 16"#;
    copies::<history::v1_20_60::Item>("1.20.60", components);
    copies::<history::v1_20_80::Item>("1.20.80", components);
    copies::<history::v1_21_10::Item>("1.21.10", components);
    copies::<history::v1_21_40::Item>("1.21.40", components);
    copies::<history::v1_21_60::Item>("1.21.60", components);
    copies::<history::v1_21_80::Item>("1.21.80", components);
    // 1.21.30 → 1.21.40: the block placer gains replace_block_item: false
    let json = item(
        "1.21.30",
        "minecraft:x",
        r#""minecraft:block_placer": {"block": "minecraft:stone"}"#,
    );
    let (payload, original) = split(&json);
    let (value, _) = parse::<history::v1_21_30::Item>(&payload, &allowlisted(), original);
    let (next, _) = upgrade(value, &allowlisted(), original, "minecraft:x");
    let Some(Component::BlockPlacer(placer)) = next.components.get("minecraft:block_placer") else {
        panic!("no block placer");
    };
    assert_eq!(placer.replace_block_item, Some(false));
}

#[test]
fn menu_category_custom_components_and_storage_split() {
    let (doc, d) =
        step::<history::v1_21_50::Item>("i13_1_21_50_menu_custom_storage", None, &allowlisted());
    assert_eq!(codes(&d, "core/deprecated_member"), 1, "the group prefix");
    let menu = doc.description.menu_category.as_ref().unwrap();
    assert_eq!(menu.category, Some(CreativeCategory::Items));
    assert_eq!(
        menu.group.as_deref(),
        Some("minecraft:itemGroup.name.bundles")
    );
    let Some(Component::StorageItem(storage)) = doc.components.get("minecraft:storage_item") else {
        panic!("no storage item");
    };
    assert_eq!(storage.max_slots, 8);
    assert!(matches!(
        doc.components.get("minecraft:storage_weight_limit"),
        Some(Component::StorageWeightLimit(StorageWeightLimit {
            max_weight_limit: 32,
            ..
        }))
    ));
    assert!(matches!(
        doc.components.get("minecraft:storage_weight_modifier"),
        Some(Component::StorageWeightModifier(StorageWeightModifier {
            weight_in_storage_item: 2,
            ..
        }))
    ));
    // the weight components are created even at their defaults
    let json = item(
        "1.21.50",
        "demo:x",
        r#""minecraft:max_stack_size": 1, "minecraft:storage_item": {}"#,
    );
    let latest = loads(&json, &third_party());
    assert_eq!(
        latest
            .components
            .get::<StorageWeightLimit>()
            .unwrap()
            .max_weight_limit,
        64
    );
    assert_eq!(
        latest
            .components
            .get::<StorageWeightModifier>()
            .unwrap()
            .weight_in_storage_item,
        4
    );
    assert!(latest.components.contains::<StorageItem>());
}

#[test]
fn an_absent_menu_category_becomes_items_and_a_namespaced_group_is_prefixed() {
    let absent = loads(
        &item("1.21.50", "demo:x", r#""minecraft:hand_equipped": true"#),
        &third_party(),
    );
    assert_eq!(
        absent.description.menu_category.category,
        Some(CreativeCategory::Items)
    );
    let json = r#"{"format_version": "1.21.50", "minecraft:item": {"description": {"identifier": "demo:x", "menu_category": {"group": "demo:group"}}, "components": {"minecraft:hand_equipped": true}}}"#;
    let namespaced = loads(json, &third_party());
    assert_eq!(
        namespaced.description.menu_category.category,
        Some(CreativeCategory::Items)
    );
    assert_eq!(
        namespaced.description.menu_category.group.as_deref(),
        Some("minecraft:demo:group")
    );
}

#[test]
fn an_absent_or_none_category_of_1_21_90_stays_undefined() {
    for menu in ["{}", r#"{"category": "none"}"#] {
        let json = format!(
            r#"{{"format_version": "1.21.90", "minecraft:item": {{"description": {{"identifier": "demo:x", "menu_category": {menu}}}, "components": {{"minecraft:hand_equipped": true}}}}}}"#
        );
        let document = loads(&json, &third_party());
        assert_eq!(
            document.description.menu_category.category,
            Some(CreativeCategory::Undefined),
            "{menu}"
        );
        let later = json.replace("1.21.90", "1.21.110");
        let document = loads(&later, &third_party());
        assert_eq!(
            document.description.menu_category.category,
            Some(CreativeCategory::ItemCommandOnly),
            "{menu} at 1.21.110"
        );
    }
    let absent = loads(
        &item("1.21.90", "demo:x", r#""minecraft:hand_equipped": true"#),
        &third_party(),
    );
    assert_eq!(
        absent.description.menu_category.category,
        Some(CreativeCategory::Undefined)
    );
    let items = r#"{"format_version": "1.21.90", "minecraft:item": {"description": {"identifier": "demo:x", "menu_category": {"category": "items"}}, "components": {"minecraft:hand_equipped": true}}}"#;
    assert_eq!(
        loads(items, &third_party())
            .description
            .menu_category
            .category,
        Some(CreativeCategory::Items)
    );
}

#[test]
fn damage_is_kept_and_aligned_placement_is_false() {
    let (doc, _) =
        step::<history::v1_21_110::Item>("i17_1_21_110_damage_placer", None, &third_party());
    assert!(matches!(
        doc.components.get("minecraft:damage"),
        Some(Component::Damage(Damage { value: 300, .. }))
    ));
    let Some(Component::BlockPlacer(placer)) = doc.components.get("minecraft:block_placer") else {
        panic!("no block placer");
    };
    assert_eq!(placer.aligned_placement, Some(false));
    for value in [200, 300] {
        let json = item(
            "1.21.110",
            "demo:x",
            &format!(r#""minecraft:damage": {value}"#),
        );
        assert_eq!(
            loads(&json, &third_party())
                .components
                .get::<Damage>()
                .unwrap()
                .value,
            value
        );
    }
}

#[test]
fn the_1_26_30_step() {
    let (doc, _) = step::<history::v1_26_0::Item>("i18_1_26_0_latest", None, &third_party());
    assert_eq!(
        doc.components.get::<UseModifiers>().unwrap().start_using,
        StartUsing::Always
    );
    // the digger keeps its trigger and is copied to the side-car
    assert!(
        doc.components.get::<Digger>().unwrap().destroy_speeds[0]
            .deprecated_on_dig
            .is_some()
    );
    assert!(doc.deprecated.legacy_on_dig.is_some());
    assert!(
        doc.components
            .iter()
            .any(|e| matches!(e, ComponentRef::Custom(c) if c.name == "demo:spin"))
    );
    // the same use_modifiers written at 1.26.30 starts if_first
    let json = item(
        "1.26.30",
        "demo:x",
        r#""minecraft:use_modifiers": {"use_duration": 1.0}"#,
    );
    assert_eq!(
        loads(&json, &third_party())
            .components
            .get::<UseModifiers>()
            .unwrap()
            .start_using,
        StartUsing::IfFirst
    );
    // a digger without triggers: no side-car entry
    let json = item(
        "1.26.0",
        "demo:x",
        r#""minecraft:digger": {"destroy_speeds": [{"block": "minecraft:dirt", "speed": 2}]}"#,
    );
    assert!(
        loads(&json, &third_party())
            .deprecated
            .legacy_on_dig
            .is_none()
    );
}

#[test]
fn the_custom_component_list_becomes_keyed_and_moves_to_the_side_car() {
    let json = item(
        "1.21.80",
        "demo:x",
        r#""minecraft:custom_components": ["demo:shake", "demo:roll"]"#,
    );
    let mut cx = third_party();
    cx.beta_apis = true;
    let document = loads(&json, &cx);
    let customs: Vec<&CustomComponent> = document
        .components
        .iter()
        .filter_map(|e| match e {
            ComponentRef::Custom(c) => Some(c),
            _ => None,
        })
        .collect();
    assert_eq!(
        customs.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
        ["demo:shake", "demo:roll"]
    );
    assert!(document.deprecated.custom_components_v1.is_some());
}

#[test]
fn deviation_i18_custom_components_from_the_list_are_also_keyed() {
    // vanilla keeps the list only in the side-car; ours also keys each entry (TOML vanilla =)
    let json = item(
        "1.21.60",
        "demo:x",
        r#""minecraft:custom_components": ["demo:shake"]"#,
    );
    let document = loads(&json, &third_party());
    assert!(
        document
            .components
            .iter()
            .any(|e| matches!(e, ComponentRef::Custom(c) if c.name == "demo:shake"))
    );
    let list = document.deprecated.custom_components_v1.clone().unwrap();
    assert_eq!(
        list,
        facet_json::from_str::<Value>(r#"[{"name": "demo:shake", "data": {}}]"#).unwrap()
    );
}

#[test]
fn a_legacy_use_duration_is_rebuilt_into_use_modifiers() {
    let json = item("1.21.0", "demo:x", r#""minecraft:use_duration": 1.6"#);
    let document = loads(&json, &third_party());
    let modifiers = document.components.get::<UseModifiers>().unwrap();
    assert!((modifiers.use_duration - 1.6).abs() < 1e-6);
    assert!(modifiers.emit_vibrations);
    assert_eq!(modifiers.start_using, StartUsing::IfFirst);
    let both = item(
        "1.21.0",
        "demo:x",
        r#""minecraft:use_duration": 2.0, "minecraft:use_modifiers": {"use_duration": 0.5}"#,
    );
    let modifiers = loads(&both, &third_party())
        .components
        .get::<UseModifiers>()
        .unwrap()
        .clone();
    assert!((modifiers.use_duration - 2.0).abs() < 1e-6);
    assert_eq!(modifiers.start_using, StartUsing::Always);
}

#[test]
fn deviation_i18b_the_legacy_use_duration_keeps_tick_precision() {
    // vanilla divides the ticks by 20 as integers: 1.6 s → 32 ticks → 1.0 s; ours 1.6 s
    let json = item("1.21.0", "demo:x", r#""minecraft:use_duration": 1.6"#);
    let modifiers = loads(&json, &third_party())
        .components
        .get::<UseModifiers>()
        .unwrap()
        .clone();
    assert!((modifiers.use_duration - 1.0).abs() > 0.5);
}

#[test]
fn vanilla_apple_keeps_its_use_duration() {
    // only use_modifiers: no legacy use duration overrides it
    let json = item(
        "1.21.0",
        "minecraft:apple",
        r#""minecraft:use_modifiers": {"use_duration": 1.6}"#,
    );
    let modifiers = loads(&json, &allowlisted())
        .components
        .get::<UseModifiers>()
        .unwrap()
        .clone();
    assert!((modifiers.use_duration - 1.6).abs() < 1e-6);
}

#[test]
fn a_1_16_100_item_through_every_step_to_latest() {
    let before = fixture("v06_full_chain_1_16_100.before.json");
    let after = fixture("v06_full_chain_1_16_100.after.json");
    let document = loads(&before, &allowlisted());
    assert!(document.version().upgraded);
    let mut got = document.value().clone();
    let mut expected = loads(&after, &allowlisted()).into_value();
    got.legacy.shared.clear();
    expected.legacy.shared.clear();
    assert_eq!(
        got.deprecated.on_consume.as_ref().unwrap().event.as_deref(),
        Some("demo:ate")
    );
    got.deprecated = bedrock_addon_family_item::ItemDeprecatedData::default();
    assert_eq!(got.legacy.format, ItemFormat::DataDriven);
    assert_eq!(got.legacy.category.as_deref(), Some("equipment"));
    got.legacy.category = None;
    expected.legacy.category = None;
    assert_eq!(got, expected);
}

#[allow(dead_code)]
fn unused(_: &BlockPlacer, _: &Wearable, _: &Code) {}
