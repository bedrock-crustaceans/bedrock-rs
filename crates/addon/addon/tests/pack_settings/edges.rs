//! Both ends of every range and both halves of every compound rule of the reader, the runtime
//! values and the save algorithm, beyond what the fixture rows pin.

use bedrock_addon::diagnostic::{DiagnosticCode as C, Diagnostics, Severity, codes};
use bedrock_addon::manifest::{DropdownOption, Setting};
use bedrock_addon::pack::settings::{
    FileState, GLOBAL_FILE, PackSettings, SavedSettings, SetError, SettingValue, Settings,
    SettingsFile, WORLD_FILE,
};
use bedrock_addon::pack::{PackSide, PackType};
use indexmap::IndexMap;
use uuid::Uuid;

use super::common::{WORLD, document, loads_text, names, one_entry, parse_text, refused_text};
use super::document::closes;

/// The vanilla unit tests' pack.
const PACK: Uuid = Uuid::from_u128(0xd334_9771_6782_4420_875a_fcd6_179a_8915);

/// A payload with one empty entry for [`PACK`].
const EMPTY_ENTRY: &str =
    r#"{"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "values": {}}]}"#;

// ---- the file's name and origin --------------------------------------------------------------

#[test]
fn parse_names_the_world_file() {
    let loaded = SavedSettings::parse(document("\"1.21.100\"", EMPTY_ENTRY).as_bytes()).unwrap();
    assert_eq!(loaded.origin().path.as_str(), WORLD_FILE);
    assert_eq!(loaded.origin().side, PackSide::Behavior);
    assert!(loaded.origin().pack.is_app_data());
    let rejected = SavedSettings::parse(b"{}").unwrap_err();
    closes(&rejected.diagnostics, WORLD_FILE);
    assert_eq!(rejected.origin.path.as_str(), WORLD_FILE);
}

#[test]
fn the_origin_is_the_last_segment_of_the_name_and_the_global_file_is_on_the_resource_side() {
    let text = document("\"1.21.100\"", EMPTY_ENTRY);
    for name in [
        GLOBAL_FILE.to_owned(),
        format!("/data/settings/{GLOBAL_FILE}"),
        format!("C:\\data\\settings\\{GLOBAL_FILE}"),
    ] {
        let loaded = parse_text(&name, &text).unwrap();
        assert_eq!(loaded.origin().path.as_str(), GLOBAL_FILE, "{name}");
        assert_eq!(loaded.origin().side, PackSide::Resource, "{name}");
    }
    let loaded = parse_text("worlds/a/my_settings.json", &text).unwrap();
    assert_eq!(loaded.origin().path.as_str(), "my_settings.json");
    assert_eq!(loaded.origin().side, PackSide::Behavior);
    // The closing finding names the file as the caller did.
    let name = "/data/worlds/a/world_behavior_pack_settings.json";
    let found = refused_text(name, "{}");
    closes(&found, name);
    for finding in &found {
        assert_eq!(finding.location.path.as_ref().unwrap().as_str(), WORLD_FILE);
        assert!(finding.location.pack.as_ref().unwrap().is_app_data());
    }
}

// ---- bytes that are no document ----------------------------------------------------------------

#[test]
fn a_bom_is_skipped() {
    let mut bytes = vec![0xef, 0xbb, 0xbf];
    bytes.extend_from_slice(document("\"1.21.100\"", EMPTY_ENTRY).as_bytes());
    assert_eq!(SavedSettings::parse(&bytes).unwrap().settings.len(), 1);
}

#[test]
fn bytes_that_are_not_utf8_or_nest_too_deep_or_are_no_json_fail_the_file() {
    let deep = format!("{}{}", "[".repeat(200), "]".repeat(200));
    for bytes in [
        &b"\xff\xfe{}"[..],
        deep.as_bytes(),
        b"{\"format_version\": ",
    ] {
        let rejected = SavedSettings::parse(bytes).unwrap_err();
        assert_eq!(
            names(&rejected.diagnostics, Severity::Error),
            ["MalformedInput", "DocumentRejected"]
        );
        closes(&rejected.diagnostics, WORLD_FILE);
    }
}

#[test]
fn d5a_a_syntax_error_carries_the_document_json_text() {
    // "Failed to parse the Minecraft Document JSON: {}" (`initAsJson`).
    for text in [
        r#"{"format_version": "1.21.100", "minecraft:pack_settings": {"settings": [}}"#,
        r#"["format_version", "1.21.100"]"#,
    ] {
        let found = refused_text(WORLD, text);
        let first = found.iter().next().unwrap();
        assert!(first.is(&codes::DOCUMENT_JSON), "{text}: {found:#?}");
        closes(&found, WORLD);
    }
}

#[test]
fn an_empty_file_fails_like_any_other() {
    let rejected = SavedSettings::parse(b"").unwrap_err();
    assert!(rejected.diagnostics.has_errors());
    closes(&rejected.diagnostics, WORLD_FILE);
}

// ---- version ends -------------------------------------------------------------------------------

#[test]
fn the_versions_next_to_each_range_end() {
    let at = |version: &str| parse_text(WORLD, &document(&format!("\"{version}\""), EMPTY_ENTRY));
    // Below, at and above the floor.
    assert!(at("1.21.99").is_err());
    assert!(at("1.21.100").unwrap().diagnostics().is_empty());
    assert_eq!(
        names(at("1.21.101").unwrap().diagnostics(), Severity::Info),
        ["VanillaRejectsVersion"]
    );
    // Below, at and above the newest struct.
    let below = at("1.26.39").unwrap();
    assert_eq!(
        names(below.diagnostics(), Severity::Info),
        ["VanillaRejectsVersion"]
    );
    assert_eq!(below.version().parsed_as.to_string(), "1.21.100");
    let newest = at("1.26.40").unwrap();
    assert!(newest.diagnostics().is_empty());
    assert_eq!(newest.version().parsed_as.to_string(), "1.26.40");
    assert!(at("1.26.41").is_err());
}

#[test]
fn an_array_version_below_the_floor_is_named_in_dotted_form_and_gets_no_array_info() {
    let found = refused_text(WORLD, &document("[1, 21, 90]", EMPTY_ENTRY));
    let version = found
        .iter()
        .find_map(|d| match &d.code {
            C::Specific { code } if code.is(&codes::NO_PARSER_FOR_VERSION) => {
                Some(code.args()[0].clone())
            }
            _ => None,
        })
        .expect("no parser");
    assert_eq!(version, "1.21.90");
    assert!(names(&found, Severity::Info).is_empty());
}

#[test]
fn a_list_value_under_an_array_version_of_the_old_struct_is_a_type_error() {
    let text = document("[1, 26, 39]", &one_entry(r#"{"banana:fruits": []}"#));
    assert!(parse_text(WORLD, &text).is_err());
    let text = document("[1, 26, 40]", &one_entry(r#"{"banana:fruits": []}"#));
    let loaded = parse_text(WORLD, &text).unwrap();
    assert_eq!(
        loaded.settings[0].values["banana:fruits"],
        SettingValue::StringList(Vec::new())
    );
    assert_eq!(
        names(loaded.diagnostics(), Severity::Info),
        ["FormatVersionArray"]
    );
}

// ---- the foreign payload key --------------------------------------------------------------------

/// Whether `found` holds a `PayloadKeyMismatch`.
fn mismatch(found: &Diagnostics) -> bool {
    found
        .iter()
        // By legacy name: the variant and the definitions that were it.
        .any(|d| matches!(d.code.name(), "PayloadKeyMismatch"))
}

#[test]
fn a_foreign_payload_key_is_not_looked_at_below_the_floor() {
    let text = format!(r#"{{"format_version": "1.21.99", "x": {EMPTY_ENTRY}}}"#);
    let found = refused_text(WORLD, &text);
    assert!(!mismatch(&found), "{found:#?}");
    assert_eq!(
        names(&found, Severity::Error),
        ["UnsupportedVersion", "DocumentRejected"]
    );
}

#[test]
fn a_foreign_payload_key_loads_under_an_array_version_and_in_either_member_order() {
    for text in [
        format!(r#"{{"format_version": [1, 21, 100], "x": {EMPTY_ENTRY}}}"#),
        format!(r#"{{"x": {EMPTY_ENTRY}, "format_version": "1.21.100"}}"#),
        format!("{{\"format_version\": \"1.26.40\", \"x\"\n\t :\n {EMPTY_ENTRY}}}"),
    ] {
        let loaded = parse_text(WORLD, &text).unwrap_or_else(|r| panic!("{:#?}", r.diagnostics));
        assert_eq!(loaded.settings[0].pack_id, PACK, "{text}");
        assert!(mismatch(loaded.diagnostics()), "{text}");
        let warning = loaded
            .diagnostics()
            .iter()
            // By legacy name: the variant and the definitions that were it.
            .find(|d| matches!(d.code.name(), "PayloadKeyMismatch"))
            .unwrap();
        let span = warning.location.span.unwrap();
        let start = usize::try_from(span.offset).unwrap();
        let end = start + usize::try_from(span.len).unwrap();
        assert_eq!(&text[start..end], "\"x\"", "the quoted name");
    }
}

#[test]
fn a_foreign_payload_key_with_an_escaped_quote_is_found_whole() {
    let text = format!(r#"{{"format_version": "1.21.100", "a\"b": {EMPTY_ENTRY}}}"#);
    let loaded = parse_text(WORLD, &text).unwrap_or_else(|r| panic!("{:#?}", r.diagnostics));
    let warning = loaded
        .diagnostics()
        .iter()
        // By legacy name: the variant and the definitions that were it.
        .find(|d| matches!(d.code.name(), "PayloadKeyMismatch"))
        .unwrap();
    assert_eq!(warning.location.json_path.to_string(), "a\"b");
    let span = warning.location.span.unwrap();
    let start = usize::try_from(span.offset).unwrap();
    let end = start + usize::try_from(span.len).unwrap();
    assert_eq!(&text[start..end], r#""a\"b""#);
}

#[test]
fn two_foreign_members_are_the_envelope_error_without_a_mismatch_warning() {
    let text = format!(r#"{{"format_version": "1.21.100", "x": {EMPTY_ENTRY}, "y": 1}}"#);
    let found = refused_text(WORLD, &text);
    assert!(!mismatch(&found));
    assert_eq!(
        names(&found, Severity::Error),
        ["MalformedInput", "DocumentRejected"]
    );
}

#[test]
fn the_right_payload_key_has_no_mismatch_warning() {
    assert!(!mismatch(
        loads_text(&document("\"1.21.100\"", EMPTY_ENTRY)).diagnostics()
    ));
}

// ---- members ------------------------------------------------------------------------------------

#[test]
fn last_used_takes_the_whole_unsigned_64_bit_range() {
    let with = |value: &str| {
        document(
            "\"1.21.100\"",
            &format!(
                r#"{{"settings": [{{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "last_used": {value}, "values": {{}}}}]}}"#
            ),
        )
    };
    assert_eq!(loads_text(&with("0")).settings[0].last_used, Some(0));
    assert_eq!(
        loads_text(&with("18446744073709551615")).settings[0].last_used,
        Some(u64::MAX)
    );
    assert!(parse_text(WORLD, &with("18446744073709551616")).is_err());
}

#[test]
fn an_entry_or_values_that_is_no_object_is_an_error() {
    for payload in [
        r#"{"settings": [5]}"#,
        r#"{"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "values": []}]}"#,
        r"[]",
    ] {
        let found = refused_text(WORLD, &document("\"1.21.100\"", payload));
        assert!(found.has_errors(), "{payload}");
        closes(&found, WORLD);
    }
}

#[test]
fn every_failure_of_one_file_is_reported() {
    let payload = r#"{"settings": [
        {"pack_id": 5, "values": {"a:x": null}, "note": 1},
        {"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "values": {"a:y": {}}}
    ], "extra": true}"#;
    let found = refused_text(WORLD, &document("\"1.21.100\"", payload));
    let errors: Vec<String> = found
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| format!("{} {}", d.code.name(), d.location.json_path))
        .collect();
    assert_eq!(
        errors,
        [
            "UnrecognizedField minecraft:pack_settings/extra",
            "UnrecognizedField minecraft:pack_settings/settings/0/note",
            "InvalidType minecraft:pack_settings/settings/0/pack_id",
            "InvalidType minecraft:pack_settings/settings/0/values/a:x",
            "InvalidType minecraft:pack_settings/settings/1/values/a:y",
            "DocumentRejected ",
        ]
    );
}

#[test]
fn two_malformed_pack_ids_are_both_nil_and_duplicates() {
    let payload = r#"{"settings": [
        {"pack_id": "one", "values": {}},
        {"pack_id": "d3349771-6782-4420-875a-fcd6179a891", "values": {}}
    ]}"#;
    let loaded = loads_text(&document("\"1.21.100\"", payload));
    assert!(loaded.settings.iter().all(|e| e.pack_id == Uuid::nil()));
    // Each malformed id is an information of ours, and so is the duplicate
    // (the game takes the first match in silence).
    assert_eq!(
        names(loaded.diagnostics(), Severity::Info),
        ["InvalidValue", "InvalidValue", "DuplicateIdentifier"]
    );
}

#[test]
fn three_entries_of_one_pack_warn_twice_and_distinct_packs_never() {
    let entry = |id: &str| format!(r#"{{"pack_id": "{id}", "values": {{}}}}"#);
    let a = "d3349771-6782-4420-875a-fcd6179a8915";
    let b = "8f4e6e3c-5b2a-4c1d-9e7f-0a1b2c3d4e5f";
    let payload = format!(
        r#"{{"settings": [{}, {}, {}, {}]}}"#,
        entry(a),
        entry(b),
        entry(a),
        entry(a)
    );
    let loaded = loads_text(&document("\"1.21.100\"", &payload));
    let paths: Vec<String> = loaded
        .diagnostics()
        .iter()
        // By legacy name: the variant and the definitions that were it.
        .filter(|d| matches!(d.code.name(), "DuplicateIdentifier"))
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(
        paths,
        [
            "minecraft:pack_settings/settings/2/pack_id",
            "minecraft:pack_settings/settings/3/pack_id"
        ]
    );
}

// ---- runtime values -----------------------------------------------------------------------------

fn slider(min: f32, max: f32, default: f32) -> Setting {
    Setting::Slider {
        name: "banana:slider".to_owned(),
        text: "Slider".to_owned(),
        min,
        max,
        step: 1.0,
        default,
    }
}

fn options() -> Vec<DropdownOption> {
    ["apple", "banana"]
        .into_iter()
        .map(|name| DropdownOption {
            name: name.to_owned(),
            text: name.to_owned(),
        })
        .collect()
}

fn entry_of(values: Vec<(&str, SettingValue)>) -> Settings {
    Settings::new(
        PACK,
        values
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value))
            .collect(),
    )
}

/// The slider's value and the infos for a saved number.
fn saved_slider(x: f32) -> (SettingValue, Vec<&'static str>) {
    let mut d = Diagnostics::default();
    let saved = entry_of(vec![("banana:slider", SettingValue::Float(x))]);
    let settings = PackSettings::new(PACK, &[slider(1.0, 10.0, 3.0)], Some(&saved), &mut d);
    assert!(names(&d, Severity::Warning).is_empty());
    (
        settings.value("banana:slider").unwrap().clone(),
        names(&d, Severity::Info),
    )
}

#[test]
fn a_saved_slider_value_is_clamped_only_outside_the_closed_range() {
    assert_eq!(
        saved_slider(0.999),
        (SettingValue::Float(1.0), vec!["SettingClamped"])
    );
    assert_eq!(saved_slider(1.0), (SettingValue::Float(1.0), vec![]));
    assert_eq!(saved_slider(1.001), (SettingValue::Float(1.001), vec![]));
    assert_eq!(saved_slider(9.999), (SettingValue::Float(9.999), vec![]));
    assert_eq!(saved_slider(10.0), (SettingValue::Float(10.0), vec![]));
    assert_eq!(
        saved_slider(10.001),
        (SettingValue::Float(10.0), vec!["SettingClamped"])
    );
}

#[test]
fn a_slider_default_outside_its_range_is_clamped_like_a_saved_value() {
    // The manifest check M10 rejects such a definition; vanilla's constructor clamps whichever
    // number it starts from.
    for (default, value) in [(11.0, 10.0), (0.0, 1.0)] {
        let mut d = Diagnostics::default();
        let settings = PackSettings::new(PACK, &[slider(1.0, 10.0, default)], None, &mut d);
        let entry = &settings.values()["banana:slider"];
        assert_eq!(entry.value, SettingValue::Float(value));
        assert_eq!(entry.default, SettingValue::Float(default));
        assert!(d.is_empty());
        // The same when a saved value of another kind is set aside for the default.
        let saved = entry_of(vec![("banana:slider", SettingValue::Bool(true))]);
        let settings = PackSettings::new(PACK, &[slider(1.0, 10.0, default)], Some(&saved), &mut d);
        assert_eq!(
            settings.value("banana:slider"),
            Some(&SettingValue::Float(value))
        );
        assert_eq!(names(&d, Severity::Info), ["SettingTypeMismatch"]);
    }
}

#[test]
fn each_kind_takes_only_its_own_kind_of_saved_value() {
    let definitions = [
        slider(1.0, 10.0, 3.0),
        Setting::Toggle {
            name: "banana:toggle".to_owned(),
            text: "Toggle".to_owned(),
            default: false,
        },
        Setting::Dropdown {
            name: "banana:dropdown".to_owned(),
            text: "Dropdown".to_owned(),
            options: options(),
            default: "apple".to_owned(),
        },
        Setting::Multiselect {
            name: "banana:fruits".to_owned(),
            text: "Fruits".to_owned(),
            options: options(),
            defaults: Vec::new(),
            selection_text: None,
        },
    ];
    let kinds = [
        SettingValue::Float(5.0),
        SettingValue::Bool(true),
        SettingValue::String("banana".to_owned()),
        SettingValue::StringList(vec!["banana".to_owned()]),
    ];
    let names_of = [
        "banana:slider",
        "banana:toggle",
        "banana:dropdown",
        "banana:fruits",
    ];
    for (definition_index, name) in names_of.iter().enumerate() {
        for (kind_index, value) in kinds.iter().enumerate() {
            let mut d = Diagnostics::default();
            let saved = entry_of(vec![(name, value.clone())]);
            let settings = PackSettings::new(PACK, &definitions, Some(&saved), &mut d);
            let entry = &settings.values()[*name];
            if definition_index == kind_index {
                assert_eq!(&entry.value, value, "{name}");
                assert!(d.is_empty(), "{name}: {d:#?}");
            } else {
                assert_eq!(entry.value, entry.default, "{name} with {value:?}");
                assert_eq!(names(&d, Severity::Info), ["SettingTypeMismatch"]);
                assert_eq!(d.len(), 1);
            }
            // The admin API refuses the same values.
            let mut fresh =
                PackSettings::new(PACK, &definitions, None, &mut Diagnostics::default());
            let outcome = fresh.set(name, value.clone());
            if definition_index == kind_index {
                assert_eq!(outcome, Ok(true), "{name}");
            } else {
                assert_eq!(
                    outcome,
                    Err(SetError::TypeMismatch),
                    "{name} with {value:?}"
                );
            }
        }
    }
}

#[test]
fn an_empty_multiselect_list_is_a_valid_value_and_one_unknown_entry_spoils_the_list() {
    let definitions = [Setting::Multiselect {
        name: "banana:fruits".to_owned(),
        text: "Fruits".to_owned(),
        options: options(),
        defaults: vec!["apple".to_owned()],
        selection_text: None,
    }];
    let list =
        |names: &[&str]| SettingValue::StringList(names.iter().map(|n| (*n).to_owned()).collect());
    for (saved_value, value, warnings) in [
        (list(&[]), list(&[]), 0),
        (list(&["banana", "apple"]), list(&["banana", "apple"]), 0),
        (list(&["durian"]), list(&["apple"]), 1),
        (list(&["banana", "durian"]), list(&["apple"]), 1),
        (list(&["durian", "banana"]), list(&["apple"]), 1),
    ] {
        let mut d = Diagnostics::default();
        let saved = entry_of(vec![("banana:fruits", saved_value.clone())]);
        let settings = PackSettings::new(PACK, &definitions, Some(&saved), &mut d);
        assert_eq!(
            settings.value("banana:fruits"),
            Some(&value),
            "{saved_value:?}"
        );
        assert_eq!(names(&d, Severity::Info).len(), warnings, "{saved_value:?}");
    }
}

#[test]
fn a_dropdown_value_is_compared_with_the_option_names_byte_for_byte() {
    let definitions = [Setting::Dropdown {
        name: "banana:dropdown".to_owned(),
        text: "Dropdown".to_owned(),
        options: options(),
        default: "apple".to_owned(),
    }];
    for (saved_value, taken) in [
        ("banana", true),
        ("Banana", false),
        ("banana ", false),
        ("", false),
    ] {
        let mut d = Diagnostics::default();
        let saved = entry_of(vec![(
            "banana:dropdown",
            SettingValue::String(saved_value.to_owned()),
        )]);
        let settings = PackSettings::new(PACK, &definitions, Some(&saved), &mut d);
        let expected = if taken { saved_value } else { "apple" };
        assert_eq!(
            settings.value("banana:dropdown"),
            Some(&SettingValue::String(expected.to_owned()))
        );
        assert_eq!(names(&d, Severity::Info).len(), usize::from(!taken));
    }
}

#[test]
fn a_saved_value_named_like_a_label_text_has_no_definition() {
    let definitions = [Setting::Label {
        text: "banana:label".to_owned(),
    }];
    let mut d = Diagnostics::default();
    let saved = entry_of(vec![("banana:label", SettingValue::Bool(true))]);
    let settings = PackSettings::new(PACK, &definitions, Some(&saved), &mut d);
    assert!(settings.values().is_empty());
    assert_eq!(names(&d, Severity::Info), ["SettingUnknownName"]);
}

#[test]
fn changed_lists_only_the_non_default_values_sorted_by_name() {
    let definitions = [
        Setting::Toggle {
            name: "zebra:on".to_owned(),
            text: "Z".to_owned(),
            default: false,
        },
        slider(1.0, 10.0, 3.0),
        Setting::Toggle {
            name: "apple:on".to_owned(),
            text: "A".to_owned(),
            default: false,
        },
    ];
    let mut settings = PackSettings::new(PACK, &definitions, None, &mut Diagnostics::default());
    assert!(settings.changed().is_empty());
    settings.set("zebra:on", SettingValue::Bool(true)).unwrap();
    settings.set("apple:on", SettingValue::Bool(true)).unwrap();
    assert_eq!(
        settings
            .changed()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["apple:on", "zebra:on"]
    );
}

#[test]
fn only_behavior_and_resource_packs_have_a_settings_file() {
    assert_eq!(
        SettingsFile::for_pack_type(PackType::Behavior),
        Some(SettingsFile::World)
    );
    assert_eq!(
        SettingsFile::for_pack_type(PackType::Resource),
        Some(SettingsFile::Global)
    );
    for other in [
        PackType::Invalid,
        PackType::Addon,
        PackType::Cached,
        PackType::CopyProtected,
        PackType::PersonaPiece,
        PackType::Skins,
        PackType::WorldTemplate,
    ] {
        assert_eq!(SettingsFile::for_pack_type(other), None, "{other:?}");
    }
    assert_eq!(
        SettingsFile::World.name(),
        "world_behavior_pack_settings.json"
    );
    assert_eq!(
        SettingsFile::Global.name(),
        "global_resource_pack_settings.json"
    );
}

// ---- the save algorithm -------------------------------------------------------------------------

fn changed_pack(id: Uuid) -> PackSettings {
    let mut settings = PackSettings::new(
        id,
        &[slider(1.0, 10.0, 3.0)],
        None,
        &mut Diagnostics::default(),
    );
    settings
        .set("banana:slider", SettingValue::Float(6.0))
        .unwrap();
    settings
}

#[test]
fn a_save_updates_the_first_entry_of_a_pack_and_keeps_a_later_one_untouched() {
    let mut first = entry_of(vec![("banana:slider", SettingValue::Float(2.0))]);
    first.last_used = Some(5);
    let mut later = entry_of(vec![("banana:old", SettingValue::Bool(true))]);
    later.last_used = Some(9);
    let previous = SavedSettings {
        settings: vec![first, later.clone()],
    };
    let document = SavedSettings::save(
        SettingsFile::Global,
        &FileState::Read(previous.clone()),
        [&changed_pack(PACK)],
    )
    .unwrap()
    .unwrap();
    // The global file is not over its limit, so the order stays.
    assert_eq!(document.settings.len(), 2);
    assert_eq!(
        document.settings[0].values["banana:slider"],
        SettingValue::Float(6.0)
    );
    assert_eq!(document.settings[0].last_used, Some(5));
    assert_eq!(document.settings[1], later);
    // Back at the default, only the first entry goes.
    let unchanged = PackSettings::new(
        PACK,
        &[slider(1.0, 10.0, 3.0)],
        None,
        &mut Diagnostics::default(),
    );
    let document = SavedSettings::save(
        SettingsFile::Global,
        &FileState::Read(previous),
        [&unchanged],
    )
    .unwrap()
    .unwrap();
    assert_eq!(document.settings, [later]);
}

#[test]
fn a_save_that_changes_nothing_writes_nothing() {
    let previous = SavedSettings {
        settings: vec![entry_of(vec![("banana:slider", SettingValue::Float(6.0))])],
    };
    for file in [SettingsFile::World, SettingsFile::Global] {
        assert_eq!(
            SavedSettings::save(
                file,
                &FileState::Read(previous.clone()),
                [&changed_pack(PACK)]
            ),
            Ok(None)
        );
    }
    // The packs the save was not given keep their entries.
    assert_eq!(
        SavedSettings::save(SettingsFile::World, &FileState::Read(previous), []),
        Ok(None)
    );
}

#[test]
fn the_world_save_is_never_pruned_and_the_global_one_only_above_128() {
    let many = |count: u128| SavedSettings {
        settings: (0..count)
            .map(|i| {
                let mut entry = Settings::new(Uuid::from_u128(1000 + i), IndexMap::new());
                entry
                    .values
                    .insert("banana:slider".to_owned(), SettingValue::Float(2.0));
                entry.last_used = u64::try_from(i).ok();
                entry
            })
            .collect(),
    };
    let pack = changed_pack(PACK);
    let world = SavedSettings::save(SettingsFile::World, &FileState::Read(many(200)), [&pack])
        .unwrap()
        .unwrap();
    assert_eq!(world.settings.len(), 201);
    // 127 entries and the new one are exactly the limit: nothing is cut and nothing reordered.
    let at_limit = SavedSettings::save(SettingsFile::Global, &FileState::Read(many(127)), [&pack])
        .unwrap()
        .unwrap();
    assert_eq!(at_limit.settings.len(), 128);
    assert_eq!(at_limit.settings[0].pack_id, Uuid::from_u128(1000));
    assert_eq!(at_limit.settings[127].pack_id, PACK);
    // One more: the newest 128 stay, newest first, and the entry without `last_used` goes.
    let over = SavedSettings::save(SettingsFile::Global, &FileState::Read(many(128)), [&pack])
        .unwrap()
        .unwrap();
    assert_eq!(over.settings.len(), 128);
    assert_eq!(over.settings[0].pack_id, Uuid::from_u128(1127));
    assert!(over.get(PACK).is_none());
}

#[test]
fn prune_to_one_fewer_drops_the_oldest_and_orders_newest_first() {
    let stamped = |id: u128, used: Option<u64>| {
        let mut entry = Settings::new(Uuid::from_u128(id), IndexMap::new());
        entry.last_used = used;
        entry
    };
    let mut document = SavedSettings {
        settings: vec![stamped(1, Some(0)), stamped(2, None), stamped(3, Some(9))],
    };
    document.prune(3);
    assert_eq!(
        document.settings[0].pack_id,
        Uuid::from_u128(1),
        "at the limit nothing moves"
    );
    document.prune(2);
    assert_eq!(
        document
            .settings
            .iter()
            .map(|e| e.pack_id)
            .collect::<Vec<_>>(),
        [Uuid::from_u128(3), Uuid::from_u128(1)],
        "a `last_used` of 0 still outranks none"
    );
    document.prune(0);
    assert!(document.settings.is_empty());
}

#[test]
fn a_failed_file_is_refused_for_either_file_and_a_missing_one_is_written() {
    let pack = changed_pack(PACK);
    for file in [SettingsFile::World, SettingsFile::Global] {
        assert!(SavedSettings::save(file, &FileState::Failed, [&pack]).is_err());
        assert!(
            SavedSettings::save(file, &FileState::Missing, [&pack])
                .unwrap()
                .is_some()
        );
        assert_eq!(SavedSettings::save(file, &FileState::Failed, []), Ok(None));
    }
}

#[test]
fn a_set_slider_value_is_clamped_at_both_ends() {
    let mut settings = PackSettings::new(
        PACK,
        &[slider(1.0, 10.0, 3.0)],
        None,
        &mut Diagnostics::default(),
    );
    assert_eq!(
        settings.set("banana:slider", SettingValue::Float(-5.0)),
        Ok(true)
    );
    assert_eq!(
        settings.value("banana:slider"),
        Some(&SettingValue::Float(1.0))
    );
    assert_eq!(
        settings.set("banana:slider", SettingValue::Float(0.5)),
        Ok(false)
    );
    assert_eq!(
        settings.set("banana:slider", SettingValue::Float(10.5)),
        Ok(true)
    );
    assert_eq!(
        settings.value("banana:slider"),
        Some(&SettingValue::Float(10.0))
    );
}
