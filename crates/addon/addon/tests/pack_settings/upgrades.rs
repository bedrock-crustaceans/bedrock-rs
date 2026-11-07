//! The `v1_21_100 → v1_26_40` step of the pack-settings chain run through `step_output`, and the
//! tests of the deliberate deviations from vanilla that this binary pins by name (each ends in
//! `deviation`, or is a manifest rule's).

use bedrock_addon::diagnostic::{Code as C, Diagnostics, Severity};
use bedrock_addon::document::{Document, Rejected};
use bedrock_addon::json::Value;
use bedrock_addon::manifest::{CapabilityRegistry, Manifest, Setting};
use bedrock_addon::pack::settings::history::v1_21_100;
use bedrock_addon::versioning::testing::{Checks, Run, step_output};
use bedrock_addon_core::diagnostic::codes as core_codes;

use super::{runtime, version};

/// Two entries: one with `last_used` and every kind of value, one without either.
const TWO_ENTRIES: &str = r#"{"settings": [
    {"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "last_used": 5120331,
     "values": {"banana:slider": 2.5, "banana:toggle": true, "banana:dropdown": "apple"}},
    {"pack_id": "8f4e6e3c-5b2a-4c1d-9e7f-0a1b2c3d4e5f", "values": {}}
]}"#;

/// The member `key` of every element of `settings` in `payload`.
fn members(payload: &Value, key: &str) -> Vec<Option<Value>> {
    payload
        .as_object()
        .and_then(|o| o.get("settings"))
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .map(|entry| entry.as_object().and_then(|o| o.get(key)).cloned())
                .collect()
        })
        .unwrap_or_default()
}

/// The `v1_21_100 → v1_26_40` step moves every member of every element unchanged: each element
/// keeps each member (an absent `last_used` stays absent), and the names keep their order.
#[test]
fn the_v1_21_100_step_keeps_every_member_of_every_element() {
    let input = facet_json::from_str::<Value>(TWO_ENTRIES).expect("the input");
    let checks = Checks::default();
    let output = step_output::<v1_21_100::SavedSettings>(&input, Run::default(), &checks)
        .expect("the step runs");
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    for key in ["pack_id", "last_used", "values"] {
        let before = members(&input, key);
        assert_eq!(before.len(), 2, "{key}");
        assert_eq!(members(&output.payload, key), before, "{key}");
    }
    let names: Vec<String> = members(&output.payload, "values")[0]
        .as_ref()
        .and_then(Value::as_object)
        .map(|o| o.iter().map(|(k, _)| k.as_str().to_owned()).collect())
        .unwrap_or_default();
    assert_eq!(names, ["banana:slider", "banana:toggle", "banana:dropdown"]);
}

// ---- deliberate deviations from vanilla ------------------------------------------------------

/// A manifest fixture of the `pack_settings` rows.
fn manifest(name: &str) -> Result<Document<Manifest>, Rejected> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/manifest/pack_settings")
        .join(format!("{name}.json"));
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    Manifest::parse(
        &bedrock_addon::storage::PackPath::new("manifest.json").unwrap(),
        &bytes,
        &CapabilityRegistry::default(),
    )
}

/// The findings of `severity` at `path` whose code satisfies `code`.
fn count(d: &Diagnostics, severity: Severity, path: &str, code: fn(&C) -> bool) -> usize {
    d.iter()
        .filter(|f| {
            f.severity == severity && f.location.json_path.to_string() == path && code(&f.code)
        })
        .count()
}

/// M2a: the members decide the kind; a `type` naming another kind is a warning.
fn manifest_setting_type_mismatch() {
    let document = manifest("mv18_shape_dispatch").expect("the manifest loads");
    assert!(matches!(document.settings[0], Setting::Label { .. }));
    assert!(matches!(document.settings[1], Setting::Toggle { .. }));
    for path in ["settings/0/type", "settings/1/type"] {
        let found = count(document.diagnostics(), Severity::Info, path, |c| {
            c.is(&core_codes::SETTING_TYPE_MISMATCH)
        });
        assert_eq!(found, 1, "{path}");
    }
    // A `type` that names the kind the members fit is silent.
    let plain = manifest("mv01_all_types").unwrap();
    assert!(
        !plain
            .diagnostics()
            .iter()
            .any(|f| f.is(&core_codes::SETTING_TYPE_MISMATCH))
    );
}

/// A multiselect in a pack whose `min_engine_version` is below 1.26.50 loads with
/// a `FieldSinceEngine` information (ours).
fn manifest_multiselect_since_engine() {
    let document = manifest("mv15_multiselect").expect("the manifest loads");
    assert!(matches!(document.settings[0], Setting::Multiselect { .. }));
    let found = count(document.diagnostics(), Severity::Info, "settings/0", |c| {
        c.id() == "core/field_since_engine"
    });
    assert_eq!(found, 1);
}

/// Legacy manifests: a format-2 manifest keeps its labels and reports each
/// other element it drops.
fn manifest_legacy_setting_ignored() {
    let document = manifest("mv17_legacy_manifest_settings").expect("the manifest loads");
    assert_eq!(document.settings.len(), 1);
    assert!(matches!(document.settings[0], Setting::Label { .. }));
    let found = count(document.diagnostics(), Severity::Info, "settings/1", |c| {
        c.is(&core_codes::LEGACY_SETTING_IGNORED)
    });
    assert_eq!(found, 1);
}

/// M1–M6: a constraint failure is our `InvalidValue` at the member, without
/// vanilla's generic text.
fn manifest_constraint_texts() {
    let rejected = manifest("mv11_slider_step_zero").expect_err("step 0 is refused");
    let found = count(
        &rejected.diagnostics,
        Severity::Error,
        "settings/0/step",
        |c| c.is(&core_codes::INVALID_VALUE),
    );
    assert_eq!(found, 1);
}

/// A1–A5: each saved value that is set aside is reported.
fn saved_value_findings() {
    runtime::rt03_invalid_saved_values_fall_back_to_the_defaults_with_one_finding_each();
    runtime::rt04_slider_clamp_clamps_and_does_not_snap_to_the_step();
    runtime::rt05_unknown_saved_name_is_ignored_and_gone_after_the_save();
    runtime::a5_multiselect_values_need_every_entry_to_be_an_option();
}

/// README: `last_used` of global entries.
#[test]
fn global_last_used_not_stamped_deviation() {
    runtime::rt11_world_no_timestamp_and_the_global_file_keeps_its_own_deviation();
    runtime::rt11_a_global_entry_our_tools_add_has_no_last_used_deviation();
}

#[test]
fn format_version_array_info_deviation() {
    version::ps08_array_version_loads_with_an_info();
}

#[test]
fn saved_value_findings_deviation() {
    saved_value_findings();
}

#[test]
fn manifest_setting_type_mismatch_deviation() {
    manifest_setting_type_mismatch();
}

#[test]
fn manifest_multiselect_since_engine_deviation() {
    manifest_multiselect_since_engine();
}

#[test]
fn manifest_legacy_setting_ignored_deviation() {
    manifest_legacy_setting_ignored();
}

#[test]
fn manifest_constraint_texts_deviation() {
    manifest_constraint_texts();
}
