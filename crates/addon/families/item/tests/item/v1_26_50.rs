//! The 1.26.50 item format of 1.26.60: the newest struct, the
//! step from 1.26.30 that turns the `LevelSoundEvent` of the four sound members into its name,
//! and the fixtures: the engine's gtest items (`families/fixtures/item/`, the
//! `<LevelSoundEvent N>` placeholders filled with level sound event names) and the
//! vanilla JSON items that run the step.

use bedrock_addon_core::document::Content;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_family_item::Item;
use bedrock_addon_family_item::component::{SwingSounds, UseModifiers};

use super::common::*;

/// A fixture file under `tests/fixtures/item/item/`.
fn fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../addon/tests/fixtures/item/item")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The gtest fixture `name` with each `<LevelSoundEvent N>` placeholder replaced by the name
/// `SharedTypes::Legacy::bindLevelSoundEventType` binds to `N`: 615 = 0x267 is
/// `undefined`, 17 `hurt`, 3 `fly`, 4 `jump`
/// (`xtask/src/script_meta/game/gen/level_sound_event_ids.tsv`).
fn gtest(name: &str) -> String {
    fixture(&format!("engine/{name}.json"))
        .replace("<LevelSoundEvent 17>", "hurt")
        .replace("<LevelSoundEvent 615>", "undefined")
        .replace("<LevelSoundEvent 3>", "fly")
        .replace("<LevelSoundEvent 4>", "jump")
}

fn version(major: u16, minor: u16, patch: u16) -> FormatVersion {
    FormatVersion::new(major, minor, patch)
}

/// The written text's `format_version`.
fn written_version(item: &Item) -> String {
    let text = item.to_json().unwrap();
    let value: Value = bedrock_addon_core::json::from_str(&text).unwrap();
    value.as_object().unwrap()["format_version"]
        .as_string()
        .unwrap()
        .as_str()
        .to_owned()
}

#[test]
fn the_newest_format_is_1_26_50_and_1_26_30_is_upgraded_to_it() {
    let at = |v: &str| {
        loads(
            &item(v, "demo:x", r#""minecraft:icon": "stick""#),
            &third_party(),
        )
    };
    for (written, parsed_as, upgraded) in [
        ("1.26.30", version(1, 26, 30), true),
        ("1.26.40", version(1, 26, 30), true),
        ("1.26.50", version(1, 26, 50), false),
        ("1.26.60", version(1, 26, 50), false),
    ] {
        let document = at(written);
        assert_eq!(document.version().parsed_as, parsed_as, "{written}");
        assert_eq!(document.version().upgraded, upgraded, "{written}");
        assert_eq!(written_version(document.value()), "1.26.50", "{written}");
    }
}

/// `UseModifiersItemComponentServerTests::srv_UseModifiersItemWithLegacyStartSound_Load_
/// UpgradesEnumToTheSameSound`: a 1.20.50 item's enum start sound survives the
/// upgrade as the same sound.
#[test]
fn a_legacy_start_sound_upgrades_to_the_same_sound() {
    let document = loads(&gtest("use_modifiers-2"), &third_party());
    assert!(document.version().upgraded);
    let modifiers = document.components.get::<UseModifiers>().unwrap();
    assert_eq!(modifiers.start_sound.as_ref().unwrap().as_str(), "hurt");
    assert_eq!(written_version(document.value()), "1.26.50");
}

/// `srv_UseModifiersItemWithCustomStartSoundName_Load_StartSoundIsTheCustomSoundName`: at
/// 1.26.50 a name that is no level sound event is kept as written.
#[test]
fn a_1_26_50_start_sound_takes_any_sound_name() {
    let document = loads(&gtest("use_modifiers-1"), &third_party());
    assert!(!document.version().upgraded);
    let modifiers = document.components.get::<UseModifiers>().unwrap();
    assert_eq!(
        modifiers.start_sound.as_ref().unwrap().as_str(),
        "test_pack:custom_start"
    );
}

/// `SwingSoundsItemComponentServerTests` -1/-2 (1.21.130, the enum) and -3 (1.26.50, a sound
/// name): the 1.21.130 sounds upgrade to the same names; the 1.26.50 one is kept.
#[test]
fn swing_sounds_upgrade_to_the_same_names_and_take_any_name_at_1_26_50() {
    let custom = loads(&gtest("swing_sounds-2"), &third_party());
    let swing = custom.components.get::<SwingSounds>().unwrap();
    let names: Vec<&str> = [
        &swing.attack_miss,
        &swing.attack_hit,
        &swing.attack_critical_hit,
    ]
    .iter()
    .map(|name| name.as_ref().unwrap().as_str())
    .collect();
    assert_eq!(names, ["undefined", "fly", "jump"]);
    assert_eq!(written_version(custom.value()), "1.26.50");
    let default = loads(&gtest("swing_sounds-1"), &third_party());
    let swing = default.components.get::<SwingSounds>().unwrap();
    assert_eq!(swing.attack_miss.as_ref().unwrap().as_str(), "undefined");
    assert!(swing.attack_hit.is_none() && swing.attack_critical_hit.is_none());
    let string = loads(&gtest("swing_sounds-3"), &third_party());
    let swing = string.components.get::<SwingSounds>().unwrap();
    assert_eq!(
        swing.attack_hit.as_ref().unwrap().as_str(),
        "test_pack:custom_swing"
    );
}

/// The loaded value without the legacy reading (derived from the JSON each side holds).
fn value_of(item: &Item) -> Item {
    let mut item = item.clone();
    item.legacy.shared.clear();
    item
}

/// `vanilla_1.20.50/items/apple.json`, the one 1.26.30 JSON item, runs the step as an
/// identity: the item written at 1.26.50 reads back as the same item, with no finding.
#[test]
fn the_vanilla_1_26_30_apple_runs_the_step_unchanged() {
    let text = fixture("vanilla/apple_1_26_30.json");
    let mut base = third_party();
    base.is_base_game_pack = true;
    let document = loads(&text, &base);
    assert!(
        above_info(document.diagnostics()).is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(document.version().parsed_as, version(1, 26, 30));
    let written = document.to_json().unwrap();
    let again = loads(&written, &base);
    assert!(
        above_info(again.diagnostics()).is_empty(),
        "{:#?}",
        again.diagnostics()
    );
    assert!(!again.version().upgraded);
    assert_eq!(
        value_of(again.value()),
        value_of(document.value()),
        "{written}"
    );
    assert_eq!(written_version(document.value()), "1.26.50");
}

/// The vanilla spears (1.21.100) name their sounds by level sound event; the step
/// carries the names into 1.26.50.
#[test]
fn the_vanilla_spear_carries_its_sound_names_into_1_26_50() {
    let mut base = third_party();
    base.is_base_game_pack = true;
    let document = loads(&fixture("vanilla/diamond_spear_1_21_100.json"), &base);
    // (a 1.21.60–1.21.109 item's absent category is told at `description`)
    assert!(
        document.diagnostics().iter().all(|f| !f
            .location
            .json_path
            .to_string()
            .contains("components")),
        "{:#?}",
        document.diagnostics()
    );
    let modifiers = document.components.get::<UseModifiers>().unwrap();
    assert_eq!(
        modifiers.start_sound.as_ref().unwrap().as_str(),
        "item.diamond_spear.use"
    );
    let swing = document.components.get::<SwingSounds>().unwrap();
    assert_eq!(
        swing.attack_hit.as_ref().unwrap().as_str(),
        "item.diamond_spear.attack_hit"
    );
    assert_eq!(written_version(document.value()), "1.26.50");
}

/// A document of `version` whose `members` (one component each) name `sound`.
fn sounding(version: &str, component: &str) -> String {
    item(version, "demo:x", component)
}

/// The components that name `sound` in each of the four sound members, one per entry with
/// the member's path below `components`.
fn sound_members(sound: &str) -> Vec<(String, &'static str)> {
    vec![
        (
            format!(
                r#""minecraft:use_modifiers": {{"use_duration": 1.0, "start_sound": "{sound}"}}"#
            ),
            "minecraft:use_modifiers/start_sound",
        ),
        (
            format!(r#""minecraft:swing_sounds": {{"attack_hit": "{sound}"}}"#),
            "minecraft:swing_sounds/attack_hit",
        ),
        (
            format!(
                r#""minecraft:durability": {{"max_durability": 10}}, "minecraft:durability_sensor": {{"durability_thresholds": [{{"durability": 5, "sound_event": "{sound}"}}]}}"#
            ),
            "minecraft:durability_sensor/durability_thresholds/0/sound_event",
        ),
    ]
}

/// Whether `d` has an `InvalidEnumValue` error at `minecraft:item/components/<path>`.
fn enum_error_at(d: &bedrock_addon_core::diagnostic::Diagnostics, path: &str) -> bool {
    d.iter().any(|f| {
        f.severity == bedrock_addon_core::diagnostic::Severity::Error
            && matches!(
                f.code,
                bedrock_addon_core::diagnostic::DiagnosticCode::InvalidEnumValue { .. }
            )
            && f.location.json_path.to_string() == format!("minecraft:item/components/{path}")
    })
}

/// Below 1.26.50 the four sound members bind `LevelSoundEvent`
/// (`bindLevelSoundEventType` in `v1_26_30::UseModifiersItemComponent`,
/// `v1_21_10::DurabilityThreshold`, `v1_21_90::SwingSounds`): a name that is
/// no level sound event is an enum error that rejects the item; from 1.26.50 any name loads.
#[test]
fn below_1_26_50_a_sound_member_takes_only_level_sound_event_names() {
    for version in ["1.21.90", "1.26.0", "1.26.30", "1.26.40"] {
        for (component, path) in sound_members("demo.custom") {
            let d = rejects(&sounding(version, &component), &third_party());
            assert!(enum_error_at(&d, path), "{version} {path}: {d:#?}");
        }
        for (component, path) in sound_members("hurt") {
            loads(&sounding(version, &component), &third_party());
            let _ = path;
        }
    }
    for (component, _) in sound_members("demo.custom") {
        loads(&sounding("1.26.50", &component), &third_party());
    }
    // the gtest fixture -1 with its placeholder left in is no level sound event
    let d = rejects(&fixture("engine/swing_sounds-1.json"), &third_party());
    assert!(
        enum_error_at(&d, "minecraft:swing_sounds/attack_miss"),
        "{d:#?}"
    );
}

/// The stable `record.sound_event` is a `LevelSoundEvent` at every stable
/// version, 1.26.50 included (`v1_20_50::RecordItemComponent`); only the beta struct
/// re-binds it as a sound name.
#[test]
fn a_stable_record_takes_only_level_sound_event_names() {
    let record = |sound: &str| format!(r#""minecraft:record": {{"sound_event": "{sound}"}}"#);
    for version in ["1.20.50", "1.26.30", "1.26.50"] {
        let d = rejects(&sounding(version, &record("demo.custom")), &third_party());
        assert!(
            enum_error_at(&d, "minecraft:record/sound_event"),
            "{version}: {d:#?}"
        );
        loads(&sounding(version, &record("record.cat")), &third_party());
    }
}
