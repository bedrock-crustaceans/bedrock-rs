//! Write-back (Value level) and the writer's `format_version`.

use bedrock_addon::json::Value;
use bedrock_addon::pack::settings::{SavedSettings, SettingValue, Settings};
use bedrock_addon::version::FormatVersion;
use indexmap::IndexMap;
use uuid::Uuid;

use super::common::{WORLD, fixture, loads, parse_text};

/// The vanilla unit tests' pack.
const PACK: Uuid = Uuid::from_u128(0xd334_9771_6782_4420_875a_fcd6_179a_8915);

/// Parse, write, parse strictly again: equal values, and the second write is identical.
fn round_trip(name: &str) {
    let first = loads(name);
    let written = first.to_json().expect("writable");
    let second = parse_text(WORLD, &written).unwrap_or_else(|r| panic!("{:#?}", r.diagnostics));
    let codes = |d: &bedrock_addon::diagnostic::Diagnostics| {
        d.iter().map(|d| d.code.id()).collect::<Vec<_>>()
    };
    // The reread finds what the first read found, apart from the version infos (the writer
    // picks its own version).
    let first_codes: Vec<_> = codes(first.diagnostics())
        .into_iter()
        .filter(|c| {
            !matches!(
                *c,
                "core/vanilla_rejects_version" | "core/format_version_array"
            )
        })
        .collect();
    assert_eq!(codes(second.diagnostics()), first_codes, "{name}");
    assert_eq!(first.value(), second.value(), "{name}");
    assert_eq!(second.to_json().unwrap(), written, "{name}");
}

#[test]
fn every_loading_fixture_round_trips() {
    for name in [
        "ps01_v1_21_100_upgrade.before.json",
        "ps01_v1_21_100_upgrade.after.json",
        "ps02_v1_21_100_no_last_used.before.json",
        "ps03_v1_26_40_string_list.json",
        "ps12_integer_value.json",
        "ps13_empty_settings.json",
        "ps11_duplicate_pack_id.json",
    ] {
        round_trip(name);
    }
}

#[test]
fn the_writer_emits_the_members_in_vanilla_order_and_no_absent_last_used() {
    let written = loads("ps02_v1_21_100_no_last_used.before.json")
        .to_json()
        .unwrap();
    assert_eq!(
        written,
        "{\n  \"format_version\": \"1.21.100\",\n  \"minecraft:pack_settings\": {\n    \"settings\": [\n      {\n        \"pack_id\": \"d3349771-6782-4420-875a-fcd6179a8915\",\n        \"values\": {\n          \"banana:toggle\": false\n        }\n      }\n    ]\n  }\n}\n"
    );
}

#[test]
fn a_vanilla_written_file_reads_back_unchanged() {
    // The shape `_saveUserOverridesToFile` writes (the 1.21.100 example).
    let text = String::from_utf8(fixture("ps01_v1_21_100_upgrade.before.json")).unwrap();
    let loaded = parse_text(WORLD, &text).unwrap();
    let written = loaded.to_json().unwrap();
    let as_json = |text: &str| bedrock_addon::json::from_str::<Value>(text).unwrap();
    assert_eq!(as_json(&written), as_json(&text));
}

fn with_values(values: Vec<(&str, SettingValue)>) -> SavedSettings {
    let values: IndexMap<String, SettingValue> = values
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value))
        .collect();
    SavedSettings {
        settings: vec![Settings::new(PACK, values)],
    }
}

#[test]
pub fn rt14_writer_version_is_1_21_100_without_a_list_and_1_26_40_with_one_deviation() {
    let plain = with_values(vec![
        ("banana:slider", SettingValue::Float(2.5)),
        ("banana:dropdown", SettingValue::String("apple".to_owned())),
    ]);
    assert_eq!(plain.format_version(), FormatVersion::new(1, 21, 100));
    assert!(
        plain
            .to_json()
            .unwrap()
            .contains("\"format_version\": \"1.21.100\"")
    );
    let list = with_values(vec![(
        "banana:fruits",
        SettingValue::StringList(vec!["apple".to_owned()]),
    )]);
    assert_eq!(list.format_version(), FormatVersion::new(1, 26, 40));
    assert!(
        list.to_json()
            .unwrap()
            .contains("\"format_version\": \"1.26.40\"")
    );
    // Both read back as written.
    for settings in [plain, list] {
        let text = settings.to_json().unwrap();
        assert_eq!(parse_text(WORLD, &text).unwrap().value(), &settings);
    }
}

#[test]
fn rt14_a_list_value_in_any_entry_needs_1_26_40() {
    let plain = |id: u128| {
        let mut values = IndexMap::new();
        values.insert("banana:slider".to_owned(), SettingValue::Float(2.5));
        Settings::new(Uuid::from_u128(id), values)
    };
    let with_list = |id: u128| {
        let mut entry = plain(id);
        entry.values.insert(
            "banana:fruits".to_owned(),
            SettingValue::StringList(vec!["apple".to_owned()]),
        );
        entry
    };
    for settings in [
        vec![with_list(1), plain(2), plain(3)],
        vec![plain(1), with_list(2), plain(3)],
        vec![plain(1), plain(2), with_list(3)],
    ] {
        let document = SavedSettings { settings };
        assert_eq!(document.format_version(), FormatVersion::new(1, 26, 40));
        let text = document.to_json().unwrap();
        assert!(text.contains("\"format_version\": \"1.26.40\""));
        let reread = parse_text(WORLD, &text).unwrap_or_else(|r| panic!("{:#?}", r.diagnostics));
        assert_eq!(reread.value(), &document);
    }
    let document = SavedSettings {
        settings: vec![plain(1), plain(2), plain(3)],
    };
    assert_eq!(document.format_version(), FormatVersion::new(1, 21, 100));
}

#[test]
fn a_float_is_written_in_its_shortest_form() {
    let settings = with_values(vec![("banana:slider", SettingValue::Float(0.1))]);
    let text = settings.to_json().unwrap();
    assert!(text.contains("\"banana:slider\": 0.1\n"), "{text}");
}

#[test]
fn a_non_finite_float_cannot_be_written() {
    let settings = with_values(vec![("banana:slider", SettingValue::Float(f32::NAN))]);
    assert!(settings.to_json().is_err());
}
