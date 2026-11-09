//! The pack settings "Runtime" fixture rows `rt01`–`rt18`: the values of one pack
//! (`PackSettings`, rules A1–A5) and the save algorithm.

#![allow(
    clippy::disallowed_methods,
    reason = "calls facet-json directly: parses a fixture or written text into a json Value to compare it; not yet moved to the door"
)]

use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::json::Value;
use bedrock_addon::manifest::{CapabilityRegistry, Manifest};
use bedrock_addon::pack::settings::{
    FileState, PackSettings, SaveRefused, SavedSettings, SetError, SettingValue, Settings,
    SettingsFile,
};
use bedrock_addon::storage::PackPath;
use bedrock_addon_core::diagnostic::codes as core_codes;
use indexmap::IndexMap;
use uuid::Uuid;

use super::common::{fixture, names};

/// The vanilla unit tests' pack.
const PACK: Uuid = Uuid::from_u128(0xd334_9771_6782_4420_875a_fcd6_179a_8915);

/// `runtime/<row>/<file>`'s text.
fn text(row: &str, file: &str) -> String {
    String::from_utf8(fixture(&format!("runtime/{row}/{file}"))).unwrap()
}

/// The manifest `runtime/<row>/<file>`.
fn manifest_file(row: &str, file: &str) -> Manifest {
    let bytes = fixture(&format!("runtime/{row}/{file}"));
    Manifest::parse(
        &PackPath::new("manifest.json").unwrap(),
        &bytes,
        &CapabilityRegistry::default(),
    )
    .unwrap_or_else(|r| panic!("{row}: {:#?}", r.diagnostics))
    .into_value()
}

/// `runtime/<row>/manifest.json`.
fn manifest(row: &str) -> Manifest {
    manifest_file(row, "manifest.json")
}

/// The saved document `runtime/<row>/<file>`.
fn saved_file(row: &str, file: &str) -> SavedSettings {
    SavedSettings::parse_named(file, text(row, file).as_bytes())
        .unwrap_or_else(|r| panic!("{row}: {:#?}", r.diagnostics))
        .into_value()
}

/// `runtime/<row>/saved.json`.
fn saved(row: &str) -> SavedSettings {
    saved_file(row, "saved.json")
}

/// A JSON text as a value.
fn json(text: &str) -> Value {
    facet_json::from_str::<Value>(text).unwrap()
}

/// `runtime/<row>/expected.json`.
fn expected(row: &str) -> Value {
    json(&text(row, "expected.json"))
}

/// A JSON value as a setting value.
fn setting_value(value: &Value) -> SettingValue {
    if let Some(number) = value.as_number() {
        #[allow(clippy::cast_possible_truncation, reason = "the game reads a float")]
        return SettingValue::Float(number.to_f64().unwrap() as f32);
    }
    if let Some(flag) = value.as_bool() {
        return SettingValue::Bool(flag);
    }
    if let Some(text) = value.as_string() {
        return SettingValue::String(text.as_str().to_owned());
    }
    let list = value.as_array().expect("a list");
    SettingValue::StringList(
        list.iter()
            .map(|v| v.as_string().unwrap().as_str().to_owned())
            .collect(),
    )
}

/// An object of expected values.
fn values_of(value: &Value) -> IndexMap<String, SettingValue> {
    value
        .as_object()
        .expect("an object of values")
        .iter()
        .map(|(name, value)| (name.as_str().to_owned(), setting_value(value)))
        .collect()
}

/// The current values.
fn current(settings: &PackSettings) -> IndexMap<String, SettingValue> {
    settings
        .values()
        .iter()
        .map(|(name, entry)| (name.clone(), entry.value.clone()))
        .collect()
}

/// The values of the pack in `row`'s manifest over `global` (a resource pack's file).
fn global_values(row: &str, global: &SavedSettings) -> (PackSettings, Diagnostics) {
    let mut d = Diagnostics::default();
    let settings = PackSettings::for_pack(&manifest(row), None, global, &mut d)
        .expect("the pack has settings");
    (settings, d)
}

/// The values of the behaviour pack in `manifest` over `world`.
fn world_values(manifest: &Manifest, world: &SavedSettings) -> PackSettings {
    let mut d = Diagnostics::default();
    PackSettings::for_pack(manifest, Some(world), &SavedSettings::default(), &mut d)
        .expect("the pack has settings")
}

/// A document as JSON, for comparing with an expected file.
fn written(document: &SavedSettings) -> Value {
    json(&document.to_json().unwrap())
}

#[test]
fn rt01_defaults_only_gives_every_default() {
    let row = "rt01_defaults_only";
    let (settings, d) = global_values(row, &saved(row));
    assert_eq!(current(&settings), values_of(&expected(row)));
    assert!(d.is_empty(), "{d:#?}");
    for entry in settings.values().values() {
        assert_eq!(entry.value, entry.default);
    }
    assert_eq!(settings.pack_id(), PACK);
}

#[test]
fn rt02_label_only_has_no_values() {
    let row = "rt02_label_only";
    let (settings, _) = global_values(row, &saved(row));
    assert!(settings.values().is_empty());
    assert_eq!(current(&settings), values_of(&expected(row)));
}

#[test]
pub fn rt03_invalid_saved_values_fall_back_to_the_defaults_with_one_finding_each() {
    let row = "rt03_invalid_saved_values";
    let (settings, d) = global_values(row, &saved(row));
    assert_eq!(current(&settings), values_of(&expected(row)));
    // In manifest order: slider ("x"), toggle (1), dropdown ("balloon").
    // All three are ours: vanilla uses the default in silence.
    assert_eq!(
        names(&d, Severity::Info),
        [
            "core/setting_type_mismatch",
            "core/setting_type_mismatch",
            "core/setting_unknown_option"
        ]
    );
    // The second run of the vanilla test: "grapes".
    let grapes = text(row, "saved.json").replace("balloon", "grapes");
    let grapes = SavedSettings::parse(grapes.as_bytes())
        .unwrap()
        .into_value();
    let (settings, d) = global_values(row, &grapes);
    assert_eq!(
        settings.value("banana:dropdown"),
        Some(&SettingValue::String("cherry".to_owned()))
    );
    assert_eq!(
        names(&d, Severity::Info),
        [
            "core/setting_type_mismatch",
            "core/setting_type_mismatch",
            "core/setting_unknown_option"
        ]
    );
}

#[test]
pub fn rt04_slider_clamp_clamps_and_does_not_snap_to_the_step() {
    let row = "rt04_slider_clamp";
    let (settings, d) = global_values(row, &saved(row));
    assert_eq!(current(&settings), values_of(&expected(row)));
    assert_eq!(names(&d, Severity::Info), ["core/setting_clamped"]);
    for (saved_value, value) in [("-3", 1.0), ("2.5", 2.5)] {
        let text = text(row, "saved.json").replace(": 42", &format!(": {saved_value}"));
        let document = SavedSettings::parse(text.as_bytes()).unwrap().into_value();
        let (settings, _) = global_values(row, &document);
        assert_eq!(
            settings.value("banana:slider"),
            Some(&SettingValue::Float(value)),
            "{saved_value}"
        );
    }
}

#[test]
pub fn rt05_unknown_saved_name_is_ignored_and_gone_after_the_save() {
    let row = "rt05_unknown_saved_name";
    let world = saved(row);
    let mut d = Diagnostics::default();
    let settings = PackSettings::for_pack(
        &manifest(row),
        Some(&world),
        &SavedSettings::default(),
        &mut d,
    )
    .unwrap();
    assert!(settings.value("banana:gone").is_none());
    assert_eq!(names(&d, Severity::Info), ["core/setting_unknown_name"]);
    let found = d
        .iter()
        .find(|f| f.is(&core_codes::SETTING_UNKNOWN_NAME))
        .unwrap();
    assert_eq!(found.location.json_path.to_string(), "values/banana:gone");
    let written_doc =
        SavedSettings::save(SettingsFile::World, &FileState::Read(world), [&settings])
            .unwrap()
            .expect("the entry changed");
    assert_eq!(written(&written_doc), expected(row));
}

#[test]
fn rt06_behavior_needs_world_has_no_values_without_a_world() {
    let row = "rt06_behavior_needs_world";
    let manifest = manifest(row);
    let mut d = Diagnostics::default();
    assert!(PackSettings::for_pack(&manifest, None, &saved(row), &mut d).is_none());
    // With a world (even one without a file) it has values.
    assert!(
        PackSettings::for_pack(
            &manifest,
            Some(&SavedSettings::default()),
            &saved(row),
            &mut d
        )
        .is_some()
    );
    // Other pack types never have settings files.
    assert_eq!(
        SettingsFile::for_pack_type(bedrock_addon::pack::PackType::Skins),
        None
    );
}

#[test]
fn rt07_pack_type_routing_sends_resource_packs_global_and_behavior_packs_to_the_world() {
    let row = "rt07_pack_type_routing";
    let resource = manifest(row);
    let behavior = manifest_file(row, "manifest_behavior.json");
    let world = SavedSettings::default();
    let global = saved(row);
    let mut d = Diagnostics::default();
    let mut resource_values =
        PackSettings::for_pack(&resource, Some(&world), &global, &mut d).unwrap();
    let mut behavior_values =
        PackSettings::for_pack(&behavior, Some(&world), &global, &mut d).unwrap();
    assert_eq!(
        resource_values.set("banana:slider", SettingValue::Float(4.0)),
        Ok(true)
    );
    assert_eq!(
        behavior_values.set("banana:toggle", SettingValue::Bool(true)),
        Ok(true)
    );
    let packs = [&resource_values, &behavior_values];
    let (resource, behavior) = (&resource, &behavior);
    let by_file = |file: SettingsFile| {
        packs
            .iter()
            .copied()
            .filter(|p| {
                let manifest = if p.pack_id() == PACK {
                    resource
                } else {
                    behavior
                };
                SettingsFile::for_pack_type(manifest.pack_type()) == Some(file)
            })
            .collect::<Vec<_>>()
    };
    let expected = expected(row);
    let expected = expected.as_object().unwrap();
    let global_doc = SavedSettings::save(
        SettingsFile::Global,
        &FileState::Missing,
        by_file(SettingsFile::Global),
    )
    .unwrap()
    .unwrap();
    assert_eq!(written(&global_doc), expected["global"]);
    let world_doc = SavedSettings::save(
        SettingsFile::World,
        &FileState::Missing,
        by_file(SettingsFile::World),
    )
    .unwrap()
    .unwrap();
    assert_eq!(written(&world_doc), expected["world"]);
}

#[test]
fn rt08_save_only_diffs_writes_exactly_the_changed_value() {
    let row = "rt08_save_only_diffs";
    let world = saved(row);
    let mut settings = world_values(&manifest(row), &world);
    assert_eq!(
        settings.set("banana:slider", SettingValue::Float(6.0)),
        Ok(true)
    );
    let document = SavedSettings::save(SettingsFile::World, &FileState::Read(world), [&settings])
        .unwrap()
        .unwrap();
    assert_eq!(written(&document), expected(row));
}

#[test]
fn rt09_back_to_default_leaves_a_file_with_no_entries() {
    let row = "rt09_back_to_default";
    let world = saved(row);
    let mut settings = world_values(&manifest(row), &world);
    settings
        .set("banana:slider", SettingValue::Float(6.0))
        .unwrap();
    let first = SavedSettings::save(SettingsFile::World, &FileState::Missing, [&settings])
        .unwrap()
        .expect("the first save writes");
    settings
        .set("banana:slider", SettingValue::Float(3.0))
        .unwrap();
    let second = SavedSettings::save(SettingsFile::World, &FileState::Read(first), [&settings])
        .unwrap()
        .expect("the second save writes");
    assert_eq!(written(&second), expected(row));
}

#[test]
fn rt10_no_change_no_file_writes_nothing() {
    let row = "rt10_no_change_no_file";
    let settings = world_values(&manifest(row), &SavedSettings::default());
    assert_eq!(
        SavedSettings::save(SettingsFile::World, &FileState::Missing, [&settings]),
        Ok(None)
    );
    assert_eq!(expected(row), Value::NULL);
}

#[test]
pub fn rt11_world_no_timestamp_and_the_global_file_keeps_its_own_deviation() {
    let row = "rt11_world_no_timestamp";
    let expected = expected(row);
    let expected = expected.as_object().unwrap();
    let global = saved(row);
    let (mut resource, _) = global_values(row, &global);
    assert_eq!(
        resource.last_used(),
        5_120_331,
        "the saved entry's last_used"
    );
    resource
        .set("banana:slider", SettingValue::Float(8.0))
        .unwrap();
    let global_doc =
        SavedSettings::save(SettingsFile::Global, &FileState::Read(global), [&resource])
            .unwrap()
            .unwrap();
    assert_eq!(written(&global_doc), expected["global"]);
    let world = saved_file(row, "saved_world.json");
    let mut behavior = world_values(&manifest_file(row, "manifest_behavior.json"), &world);
    behavior
        .set("banana:slider", SettingValue::Float(8.0))
        .unwrap();
    let world_doc = SavedSettings::save(SettingsFile::World, &FileState::Read(world), [&behavior])
        .unwrap()
        .unwrap();
    assert_eq!(written(&world_doc), expected["world"]);
}

#[test]
pub fn rt11_a_global_entry_our_tools_add_has_no_last_used_deviation() {
    let row = "rt11_world_no_timestamp";
    let (mut resource, _) = global_values(row, &SavedSettings::default());
    assert_eq!(resource.last_used(), 0);
    resource
        .set("banana:slider", SettingValue::Float(8.0))
        .unwrap();
    let document = SavedSettings::save(SettingsFile::Global, &FileState::Missing, [&resource])
        .unwrap()
        .unwrap();
    assert_eq!(document.settings[0].last_used, None);
}

/// The pack ids of `document`, as strings.
fn ids(document: &SavedSettings) -> Vec<String> {
    document
        .settings
        .iter()
        .map(|entry| entry.pack_id.to_string())
        .collect()
}

/// An expected list of pack ids.
fn expected_ids(row: &str) -> Vec<String> {
    expected(row)
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_string().unwrap().as_str().to_owned())
        .collect()
}

#[test]
fn rt12_prune_keeps_the_most_recently_used() {
    let row = "rt12_prune";
    let mut document = saved(row);
    document.prune(4);
    assert_eq!(ids(&document), expected_ids(row));
    // At or under the limit nothing moves.
    let mut document = saved(row);
    document.prune(8);
    assert_eq!(ids(&document), ids(&saved(row)));
}

#[test]
fn rt13_prune_missing_last_used_drops_those_first() {
    let row = "rt13_prune_missing_last_used";
    let mut document = saved(row);
    document.prune(2);
    assert_eq!(ids(&document), expected_ids(row));
}

#[test]
pub fn rt12_prune_ties_keep_file_order_deviation() {
    let tied = |id: u128| {
        let mut entry = Settings::new(Uuid::from_u128(id), IndexMap::new());
        entry.last_used = Some(7);
        entry
    };
    let mut document = SavedSettings {
        settings: vec![tied(3), tied(1), tied(2)],
    };
    document.prune(2);
    assert_eq!(
        document
            .settings
            .iter()
            .map(|e| e.pack_id)
            .collect::<Vec<_>>(),
        [Uuid::from_u128(3), Uuid::from_u128(1)]
    );
}

#[test]
fn rt12_prune_of_a_large_file_keeps_tied_entries_in_file_order_deviation() {
    // 150 entries whose `last_used` repeats 0, 1, 2: far more ties than a sort handles by
    // insertion. The newest 128 stay: every 2, every 1, then the first 28 of the 0s, each group
    // in file order.
    let mut previous = SavedSettings::default();
    for i in 0..150_u64 {
        let mut values = IndexMap::new();
        values.insert("banana:slider".to_owned(), SettingValue::Float(2.0));
        let mut entry = Settings::new(Uuid::from_u128(u128::from(i) + 1), values);
        entry.last_used = Some(i % 3);
        previous.settings.push(entry);
    }
    let mut expected = Vec::new();
    for used in [2, 1, 0] {
        expected.extend(
            previous
                .settings
                .iter()
                .filter(|entry| entry.last_used == Some(used))
                .map(|entry| entry.pack_id),
        );
    }
    expected.truncate(128);
    let mut pruned = previous.clone();
    pruned.prune(128);
    assert_eq!(
        pruned
            .settings
            .iter()
            .map(|e| e.pack_id)
            .collect::<Vec<_>>(),
        expected
    );
    // The same through a global save (the pack given has nothing to store).
    let (unchanged, _) = global_values("rt01_defaults_only", &SavedSettings::default());
    let saved = SavedSettings::save(
        SettingsFile::Global,
        &FileState::Read(previous),
        [&unchanged],
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        saved.settings.iter().map(|e| e.pack_id).collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn the_global_save_prunes_to_128_entries() {
    let mut previous = SavedSettings::default();
    for i in 0..130_u128 {
        let mut values = IndexMap::new();
        values.insert("banana:slider".to_owned(), SettingValue::Float(2.0));
        let mut entry = Settings::new(Uuid::from_u128(i + 1), values);
        entry.last_used = u64::try_from(i).ok();
        previous.settings.push(entry);
    }
    let row = "rt01_defaults_only";
    let (mut settings, _) = global_values(row, &SavedSettings::default());
    settings
        .set("banana:slider", SettingValue::Float(9.0))
        .unwrap();
    let document = SavedSettings::save(
        SettingsFile::Global,
        &FileState::Read(previous),
        [&settings],
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        document.settings.len(),
        bedrock_addon::pack::settings::MAX_GLOBAL_ENTRIES
    );
    // The new entry has no `last_used`, so it goes first.
    assert!(document.get(PACK).is_none());
}

#[test]
fn rt14_writer_version_follows_the_values() {
    let row = "rt14_writer_version";
    let expected = expected(row);
    let expected = expected.as_object().unwrap();
    let (mut settings, _) = global_values(row, &SavedSettings::default());
    settings
        .set("banana:slider", SettingValue::Float(5.0))
        .unwrap();
    let plain = SavedSettings::save(SettingsFile::Global, &FileState::Missing, [&settings])
        .unwrap()
        .unwrap();
    assert_eq!(
        Value::from(plain.format_version().to_string()),
        expected["plain"]
    );
    settings
        .set(
            "banana:fruits",
            SettingValue::StringList(vec!["apple".to_owned()]),
        )
        .unwrap();
    let list = SavedSettings::save(SettingsFile::Global, &FileState::Missing, [&settings])
        .unwrap()
        .unwrap();
    assert_eq!(
        Value::from(list.format_version().to_string()),
        expected["list"]
    );
}

#[test]
pub fn rt15_failed_file_not_overwritten_deviation() {
    let row = "rt15_failed_file_not_overwritten";
    let bytes = fixture(&format!("runtime/{row}/saved.json"));
    assert!(
        SavedSettings::parse(&bytes).is_err(),
        "the file fails to load"
    );
    let mut settings = world_values(&manifest(row), &SavedSettings::default());
    settings
        .set("banana:slider", SettingValue::Float(6.0))
        .unwrap();
    assert_eq!(
        SavedSettings::save(SettingsFile::World, &FileState::Failed, [&settings]),
        Err(SaveRefused::FileFailedToLoad)
    );
    // What vanilla writes over the file at this point (the save of a file it takes for
    // missing): the changed value alone, without anything the failed file held.
    let overwrite = SavedSettings::save(SettingsFile::World, &FileState::Missing, [&settings])
        .unwrap()
        .expect("a missing file is written");
    assert_eq!(overwrite.settings.len(), 1);
    assert_eq!(
        overwrite.settings[0].values.keys().collect::<Vec<_>>(),
        ["banana:slider"]
    );
    let held = String::from_utf8(bytes).unwrap();
    assert!(held.contains("\"comment\""), "the failed file holds more");
    assert!(!overwrite.to_json().unwrap().contains("comment"));
}

#[test]
fn rt15b_failed_file_no_changes_writes_nothing() {
    let row = "rt15b_failed_file_no_changes";
    assert!(SavedSettings::parse(&fixture(&format!("runtime/{row}/saved.json"))).is_err());
    let settings = world_values(&manifest(row), &SavedSettings::default());
    assert_eq!(
        SavedSettings::save(SettingsFile::World, &FileState::Failed, [&settings]),
        Ok(None)
    );
}

#[test]
pub fn rt16_set_value_checks_reject_what_the_definition_does_not_allow_deviation() {
    let row = "rt16_set_value_checks";
    let (mut settings, _) = global_values(row, &SavedSettings::default());
    let before = current(&settings);
    assert_eq!(before, values_of(&expected(row)));
    assert_eq!(
        settings.set("banana:slider", SettingValue::Bool(true)),
        Err(SetError::TypeMismatch)
    );
    assert_eq!(
        settings.set("banana:dropdown", SettingValue::String("durian".to_owned())),
        Err(SetError::UnknownOption)
    );
    assert_eq!(
        settings.set(
            "banana:fruits",
            SettingValue::StringList(vec!["apple".to_owned(), "durian".to_owned()])
        ),
        Err(SetError::UnknownOption)
    );
    assert_eq!(
        settings.set("banana:gone", SettingValue::Bool(true)),
        Err(SetError::UnknownName)
    );
    assert_eq!(current(&settings), before, "nothing changed");
    // A slider value is clamped; an equal value is no change.
    assert_eq!(
        settings.set("banana:slider", SettingValue::Float(99.0)),
        Ok(true)
    );
    assert_eq!(
        settings.value("banana:slider"),
        Some(&SettingValue::Float(10.0))
    );
    assert_eq!(
        settings.set("banana:slider", SettingValue::Float(10.0)),
        Ok(false)
    );
    assert_eq!(
        settings.set("banana:dropdown", SettingValue::String("apple".to_owned())),
        Ok(true)
    );
}

#[test]
fn rt17_label_only_pack_has_settings_and_saves_nothing() {
    let row = "rt17_label_only_pack";
    let settings = world_values(&manifest(row), &SavedSettings::default());
    assert!(settings.values().is_empty());
    assert_eq!(
        SavedSettings::save(SettingsFile::World, &FileState::Missing, [&settings]),
        Ok(None)
    );
    assert_eq!(expected(row), Value::NULL);
}

#[test]
fn a_manifest_with_an_empty_settings_list_has_no_pack_settings_deviation() {
    // Vanilla tests only that the manifest's `settings` optional is engaged; ours has a list,
    // and an empty one (a legacy manifest whose every element was dropped) is "no settings".
    let mut plain = manifest("rt17_label_only_pack");
    assert!(
        PackSettings::for_pack(
            &plain,
            Some(&SavedSettings::default()),
            &SavedSettings::default(),
            &mut Diagnostics::default()
        )
        .is_some()
    );
    plain.settings.clear();
    assert!(
        PackSettings::for_pack(
            &plain,
            Some(&SavedSettings::default()),
            &SavedSettings::default(),
            &mut Diagnostics::default()
        )
        .is_none()
    );
}

#[test]
fn rt18_slider_bounds_inclusive_keeps_values_at_the_bounds() {
    let row = "rt18_slider_bounds_inclusive";
    let (settings, d) = global_values(row, &saved(row));
    assert_eq!(current(&settings), values_of(&expected(row)));
    assert!(d.is_empty());
    for (bound, value) in [("1", 1.0), ("10", 10.0)] {
        let text = text(row, "saved.json").replace(": 3", &format!(": {bound}"));
        let document = SavedSettings::parse(text.as_bytes()).unwrap().into_value();
        let (settings, d) = global_values(row, &document);
        assert_eq!(
            settings.value("banana:slider"),
            Some(&SettingValue::Float(value))
        );
        assert!(d.is_empty(), "{bound}: {d:#?}");
    }
}

#[test]
pub fn a5_multiselect_values_need_every_entry_to_be_an_option() {
    let row = "multiselect_values";
    let (settings, d) = global_values(row, &saved(row));
    assert_eq!(current(&settings), values_of(&expected(row)));
    assert!(d.is_empty());
    let text = text(row, "saved.json").replace("\"banana\"", "\"durian\"");
    let document = SavedSettings::parse(text.as_bytes()).unwrap().into_value();
    let (settings, d) = global_values(row, &document);
    assert_eq!(
        settings.value("banana:fruits"),
        Some(&SettingValue::StringList(vec!["cherry".to_owned()]))
    );
    assert_eq!(names(&d, Severity::Info), ["core/setting_unknown_option"]);
    // A string where a list belongs is a type mismatch.
    let text = text_of_value("banana:fruits", "\"apple\"");
    let document = SavedSettings::parse(text.as_bytes()).unwrap().into_value();
    let (_, d) = global_values(row, &document);
    assert_eq!(names(&d, Severity::Info), ["core/setting_type_mismatch"]);
}

/// A 1.26.40 document holding one value for the vanilla tests' pack.
fn text_of_value(name: &str, value: &str) -> String {
    super::common::document(
        "\"1.26.40\"",
        &super::common::one_entry(&format!(r#"{{"{name}": {value}}}"#)),
    )
}

#[test]
fn a_repeated_name_keeps_its_first_definition() {
    use bedrock_addon::manifest::Setting;
    let mut definitions = manifest("rt01_defaults_only").settings;
    let first = definitions
        .iter()
        .find(|s| s.name() == Some("banana:slider"))
        .cloned()
        .unwrap();
    let mut second = first.clone();
    if let Setting::Slider { default, .. } = &mut second {
        *default = 9.0;
    }
    definitions.push(second);
    let mut d = Diagnostics::default();
    let settings = PackSettings::new(PACK, &definitions, None, &mut d);
    assert_eq!(
        settings.value("banana:slider"),
        Some(&SettingValue::Float(3.0))
    );
    assert_eq!(settings.values().len(), 3);
}
