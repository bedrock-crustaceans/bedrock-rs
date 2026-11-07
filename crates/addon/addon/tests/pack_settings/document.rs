//! `families/pack_settings` fixtures.md "Document" rows `ps01`–`ps16` (validation.md "Save
//! document" D1–D10).

use bedrock_addon::diagnostic::{Code as C, Diagnostic, Diagnostics, Severity};
use bedrock_addon::pack::settings::{SavedSettings, SettingValue, Settings};
use bedrock_addon::version::FormatVersion;
use indexmap::IndexMap;
use uuid::Uuid;

use super::common::{at, loads, names, refused};

/// The vanilla unit tests' pack.
const PACK: Uuid = Uuid::from_u128(0xd334_9771_6782_4420_875a_fcd6_179a_8915);

const V1_21_100: FormatVersion = FormatVersion::new(1, 21, 100);
const V1_26_40: FormatVersion = FormatVersion::new(1, 26, 40);

/// An upgrade fixture pair: `before` parsed at 1.21.100 and upgraded equals `after` at 1.26.40.
fn upgrade_pair(name: &str) -> SavedSettings {
    let before = loads(&format!("{name}.before.json"));
    assert_eq!(before.version().parsed_as, V1_21_100);
    assert!(before.version().upgraded, "the step ran");
    let after = loads(&format!("{name}.after.json"));
    assert_eq!(after.version().parsed_as, V1_26_40);
    assert!(!after.version().upgraded);
    assert_eq!(before.value(), after.value());
    assert!(
        before.diagnostics().is_empty(),
        "{:#?}",
        before.diagnostics()
    );
    assert!(after.diagnostics().is_empty(), "{:#?}", after.diagnostics());
    before.into_value()
}

#[test]
fn ps01_v1_21_100_upgrade_keeps_every_value_and_last_used() {
    let settings = upgrade_pair("ps01_v1_21_100_upgrade");
    let values: IndexMap<String, SettingValue> = [
        ("banana:slider", SettingValue::Float(5.5)),
        ("banana:toggle", SettingValue::Bool(false)),
        ("banana:dropdown", SettingValue::String("apple".to_owned())),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_owned(), value))
    .collect();
    let mut entry = Settings::new(PACK, values);
    entry.last_used = Some(5_120_331);
    assert_eq!(
        settings,
        SavedSettings {
            settings: vec![entry]
        }
    );
}

#[test]
fn ps02_v1_21_100_no_last_used_stays_absent_after_the_step() {
    let settings = upgrade_pair("ps02_v1_21_100_no_last_used");
    assert_eq!(settings.settings.len(), 1);
    assert_eq!(settings.settings[0].last_used, None);
    assert_eq!(
        settings.settings[0].values["banana:toggle"],
        SettingValue::Bool(false)
    );
}

/// The closing finding of a rejected file: "Pack Settings failed to load [<name>]".
pub fn load_failed(name: &str) -> C {
    bedrock_addon::pack::settings::codes::LOAD_FAILED.with([name])
}

/// The errors of a rejected file end with exactly one closing finding naming it.
pub fn closes(diagnostics: &Diagnostics, name: &str) {
    let errors: Vec<&Diagnostic> = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    let last = errors.last().expect("an error");
    assert_eq!(last.code, load_failed(name), "{diagnostics:#?}");
    assert_eq!(
        last.code.to_string(),
        format!("Pack Settings failed to load [{name}]")
    );
    let closing = errors
        .iter()
        .filter(|d| matches!(d.code.id(), "pack_settings/load_failed"))
        .count();
    assert_eq!(closing, 1, "one closing finding per file");
}

#[test]
fn ps03_v1_26_40_string_list_loads() {
    let settings = loads("ps03_v1_26_40_string_list.json");
    assert!(
        settings.diagnostics().is_empty(),
        "{:#?}",
        settings.diagnostics()
    );
    assert_eq!(
        settings.settings[0].values["banana:fruits"],
        SettingValue::StringList(vec!["apple".to_owned(), "cherry".to_owned()])
    );
}

#[test]
fn ps04_array_at_1_21_100_is_a_type_error_and_fails_the_file() {
    let name = "ps04_array_at_1_21_100.json";
    let found = refused(name);
    assert_eq!(
        names(&found, Severity::Error),
        ["core/invalid_type", "pack_settings/load_failed"]
    );
    let typed = at(
        &found,
        "minecraft:pack_settings/settings/0/values/banana:fruits",
    );
    assert!(matches!(typed[..], [d] if matches!(d.code.id(), "core/invalid_type")));
    closes(&found, name);
}

#[test]
fn ps12_integer_value_reads_as_a_float() {
    let settings = loads("ps12_integer_value.json");
    assert_eq!(
        settings.settings[0].values["banana:slider"],
        SettingValue::Float(5.0)
    );
}

#[test]
fn ps13_empty_settings_loads_with_no_entries() {
    let settings = loads("ps13_empty_settings.json");
    assert!(settings.settings.is_empty());
    assert!(settings.diagnostics().is_empty());
}
