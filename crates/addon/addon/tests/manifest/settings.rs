//! The `manifest::Setting` rules (format 3 manifest settings and legacy manifests).

use bedrock_addon::diagnostic::{Code, CodeMeta, Diagnostics, Severity};
use bedrock_addon::document::Document;
use bedrock_addon::manifest::{DropdownOption, Manifest, Setting};
use bedrock_addon_core::diagnostic::codes as core_codes;

use super::{findings, fixture, has, parse};

fn mv(name: &str) -> Result<Document<Manifest>, bedrock_addon::document::Rejected> {
    parse(&fixture(&format!("pack_settings/{name}.json")))
}

fn loaded(name: &str) -> Document<Manifest> {
    mv(name).unwrap_or_else(|rejected| panic!("{name}: {:#?}", rejected.diagnostics))
}

fn refused(name: &str) -> Diagnostics {
    match mv(name) {
        Ok(document) => panic!("{name} loaded: {:#?}", document.settings),
        Err(rejected) => rejected.diagnostics,
    }
}

/// The codes of the pack settings definitions of core among the findings, in order.
fn texts(diagnostics: &Diagnostics) -> Vec<Code> {
    let settings: [&CodeMeta; 6] = [
        core_codes::DUPLICATE_PACK_SETTING_NAME.meta(),
        core_codes::SLIDER_MAX_NOT_ABOVE_MIN.meta(),
        core_codes::SLIDER_STEP_TOO_LARGE.meta(),
        core_codes::SLIDER_DEFAULT_OUT_OF_RANGE.meta(),
        core_codes::DROPDOWN_DEFAULT_NOT_IN_OPTIONS.meta(),
        core_codes::MULTISELECT_DEFAULT_NOT_IN_OPTIONS.meta(),
    ];
    diagnostics
        .iter()
        .filter_map(|d| match &d.code {
            code if settings.iter().any(|meta| meta.id() == code.id()) => Some(code.clone()),
            _ => None,
        })
        .collect()
}

fn invalid_at(diagnostics: &Diagnostics, path: &str) -> bool {
    has(diagnostics, Severity::Error, path, |c| {
        c.id() == "core/invalid_value"
    })
}

fn fruit() -> Vec<DropdownOption> {
    ["apple", "banana", "cherry"]
        .into_iter()
        .map(|name| DropdownOption {
            name: name.to_owned(),
            text: format!("{}{}", name[..1].to_uppercase(), &name[1..]),
        })
        .collect()
}

#[test]
fn mv01_all_types_load_without_a_finding() {
    let document = loaded("mv01_all_types");
    assert!(
        document.diagnostics().is_empty(),
        "{:?}",
        document.diagnostics()
    );
    assert_eq!(
        document.settings,
        [
            Setting::Label {
                text: "Settings".to_owned()
            },
            Setting::Slider {
                name: "banana:slider".to_owned(),
                text: "Slider".to_owned(),
                min: 1.0,
                max: 10.0,
                step: 1.0,
                default: 3.0,
            },
            Setting::Toggle {
                name: "banana:toggle".to_owned(),
                text: "Toggle".to_owned(),
                default: true,
            },
            Setting::Dropdown {
                name: "banana:dropdown".to_owned(),
                text: "Dropdown".to_owned(),
                options: fruit(),
                default: "cherry".to_owned(),
            },
        ]
    );
}

#[test]
fn mv02_fifty_settings_are_valid() {
    assert_eq!(loaded("mv02_fifty").settings.len(), 50);
}

#[test]
fn mv03_fifty_one_settings_are_a_container_size_error() {
    assert!(invalid_at(&refused("mv03_fifty_one"), "settings"));
}

#[test]
fn mv04_a_name_without_namespace_is_an_error() {
    assert!(invalid_at(&refused("mv04_no_namespace"), "settings/0/name"));
}

#[test]
fn mv05_the_minecraft_namespace_is_an_error() {
    assert!(invalid_at(
        &refused("mv05_minecraft_namespace"),
        "settings/0/name"
    ));
}

#[test]
fn mv06_the_minecraft_namespace_in_upper_case_is_an_error() {
    assert!(invalid_at(
        &refused("mv06_minecraft_upper"),
        "settings/0/name"
    ));
}

#[test]
fn mv07_duplicate_names_are_an_error() {
    assert_eq!(
        texts(&refused("mv07_duplicate_names")),
        [core_codes::DUPLICATE_PACK_SETTING_NAME.with(["banana:slider".to_owned()])]
    );
}

#[test]
fn mv08_slider_max_not_above_min() {
    let texts = texts(&refused("mv08_slider_min_max"));
    assert_eq!(
        texts[0],
        core_codes::SLIDER_MAX_NOT_ABOVE_MIN.with(["banana:slider".to_owned()])
    );
}

#[test]
fn mv09_slider_step_above_the_range() {
    assert_eq!(
        texts(&refused("mv09_slider_step")),
        [core_codes::SLIDER_STEP_TOO_LARGE.with(["banana:slider".to_owned(), "5".to_owned()])]
    );
}

#[test]
fn mv10_slider_default_out_of_range() {
    assert_eq!(
        texts(&refused("mv10_slider_default")),
        [core_codes::SLIDER_DEFAULT_OUT_OF_RANGE
            .with(["banana:slider".to_owned(), "11".to_owned()])]
    );
}

#[test]
fn mv11_slider_step_zero_is_a_constraint_error() {
    assert!(invalid_at(
        &refused("mv11_slider_step_zero"),
        "settings/0/step"
    ));
}

#[test]
fn mv12_dropdown_default_not_in_options() {
    assert_eq!(
        texts(&refused("mv12_dropdown_default")),
        [core_codes::DROPDOWN_DEFAULT_NOT_IN_OPTIONS
            .with(["banana:dropdown".to_owned(), "durian".to_owned()])]
    );
}

#[test]
fn mv13_a_dropdown_with_one_option_is_an_error() {
    assert!(invalid_at(
        &refused("mv13_dropdown_one_option"),
        "settings/0/options"
    ));
}

#[test]
fn mv14_a_dropdown_with_21_options_is_an_error() {
    assert!(invalid_at(
        &refused("mv14_dropdown_21_options"),
        "settings/0/options"
    ));
}

#[test]
fn mv15_multiselect_with_no_and_two_defaults_is_valid() {
    let document = loaded("mv15_multiselect");
    assert!(
        matches!(&document.settings[0], Setting::Multiselect { defaults, .. } if defaults.is_empty())
    );
    assert!(
        matches!(&document.settings[1], Setting::Multiselect { defaults, .. } if defaults == &["apple", "cherry"])
    );
    // families/pack_settings metadata: multiselect needs 1.26.50; this pack declares 1.26.20.
    // The game says nothing of it: an information of ours.
    assert!(has(
        document.diagnostics(),
        Severity::Info,
        "settings/0",
        |c| c.id() == "core/field_since_engine"
    ));
}

#[test]
fn mv16_multiselect_default_not_in_options() {
    assert_eq!(
        texts(&refused("mv16_multiselect_bad_default")),
        [core_codes::MULTISELECT_DEFAULT_NOT_IN_OPTIONS
            .with(["banana:fruits".to_owned(), "durian".to_owned()])]
    );
}

#[test]
fn mv17_a_legacy_manifest_keeps_only_labels() {
    let document = loaded("mv17_legacy_manifest_settings");
    assert_eq!(
        document.settings,
        [Setting::Label {
            text: "Settings".to_owned()
        }]
    );
    assert!(has(
        document.diagnostics(),
        Severity::Info,
        "settings/1",
        |c| c.is(&core_codes::LEGACY_SETTING_IGNORED)
    ));
}

#[test]
fn mv18_the_shape_not_the_type_decides_the_kind() {
    let document = loaded("mv18_shape_dispatch");
    assert_eq!(
        document.settings,
        [
            Setting::Label {
                text: "x".to_owned()
            },
            Setting::Toggle {
                name: "banana:t".to_owned(),
                text: "t".to_owned(),
                default: true,
            },
        ]
    );
    for path in ["settings/0/type", "settings/1/type"] {
        assert!(has(document.diagnostics(), Severity::Info, path, |c| c
            .is(&core_codes::SETTING_TYPE_MISMATCH)));
    }
}

#[test]
fn mv19_an_entry_that_fits_no_kind_rejects_the_manifest() {
    let diagnostics = refused("mv19_fits_nothing");
    assert!(has(&diagnostics, Severity::Error, "settings/0/step", |c| c
        .id()
        == "core/missing_required_field"));
    assert!(has(
        &diagnostics,
        Severity::Error,
        "settings/0/default",
        |c| c.id() == "core/missing_required_field"
    ));
}

#[test]
fn mv20_one_slider_can_fail_all_three_checks_in_order() {
    let name = || "banana:slider".to_owned();
    let diagnostics = refused("mv20_slider_all_three");
    let texts = texts(&diagnostics);
    assert_eq!(
        texts,
        [
            core_codes::SLIDER_MAX_NOT_ABOVE_MIN.with([name()]),
            core_codes::SLIDER_STEP_TOO_LARGE.with([name(), "2".to_owned()]),
            core_codes::SLIDER_DEFAULT_OUT_OF_RANGE.with([name(), "6".to_owned()]),
        ]
    );
    // The numbers print as `{}` prints an `f32` (`5`, not `5.0`).
    let rendered: Vec<String> = texts.iter().map(ToString::to_string).collect();
    assert_eq!(
        rendered,
        [
            "slider banana:slider max must be greater than min",
            "slider banana:slider step 2 must be <= to max - min",
            "slider banana:slider default 6 is out of range",
        ]
    );
}

#[test]
fn every_manifest_validation_fixture_is_a_test() {
    let mut names: Vec<String> = std::fs::read_dir(super::fixtures().join("pack_settings"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    assert_eq!(names.len(), 20);
    // Each fixture parses to a result (none panics the reader).
    for name in names {
        let result = parse(&fixture(&format!("pack_settings/{name}")));
        let _ = findings(&result);
    }
}

// ---- `ScriptPackSettingsTests` (1.26.60): the only format-3 manifests the build embeds ----

/// The text of `script_pack_settings/<name>`, a file copied verbatim from the build.
fn template(name: &str) -> String {
    String::from_utf8(fixture(&format!("script_pack_settings/{name}"))).unwrap()
}

/// `template` (a C++ `fmt` template: `{}` a parameter, `{{` / `}}` a brace) with `args` in its
/// parameters, as the engine test fills it.
fn fill(template: &str, args: &[&str]) -> String {
    let mut out = String::new();
    let mut args = args.iter();
    let mut rest = template;
    while let Some(at) = rest.find(['{', '}']) {
        out.push_str(&rest[..at]);
        let tail = &rest[at..];
        if let Some(after) = tail.strip_prefix("{{") {
            out.push('{');
            rest = after;
        } else if let Some(after) = tail.strip_prefix("}}") {
            out.push('}');
            rest = after;
        } else if let Some(after) = tail.strip_prefix("{}") {
            out.push_str(args.next().expect("an argument per parameter"));
            rest = after;
        } else {
            panic!("a lone brace in the template");
        }
    }
    out.push_str(rest);
    assert!(args.next().is_none(), "every argument used");
    out
}

/// The manifest template (`format_version` the integer 3, two
/// `dependencies` objects with trailing commas) around one setting definition, with the
/// UUIDs of `ScriptPackSettingsTests-multiselect-filled.json`.
fn script_pack_settings_manifest(setting: &str) -> String {
    fill(
        &template("ScriptPackSettingsTests-manifest.json"),
        &[
            "aa6516db-e06e-47de-afa2-6363e2efa31b",
            "00000000-0000-4000-8000-000000000001",
            "1.0.0",
            setting,
        ],
    )
}

/// Each `ScriptPackSettingsTests` manifest loads in the game
/// (`srv_MultiplePacks_DefineDifferentSettings_GetCorrectSettingsForPack` asserts
/// the settings it defines), as format 3.
#[test]
fn script_pack_settings_tests_manifests_load_as_format_3() {
    let multiselect = fill(
        &template("ScriptPackSettingsTests-multiselect.json"),
        &["e:multiselect", ""],
    );
    for (setting, kind) in [
        (
            fill(
                &template("ScriptPackSettingsTests-slider.json"),
                &["a:slider"],
            ),
            "slider",
        ),
        (
            fill(
                &template("ScriptPackSettingsTests-toggle.json"),
                &["b:toggle"],
            ),
            "toggle",
        ),
        (
            fill(
                &template("ScriptPackSettingsTests-dropdown.json"),
                &["c:dropdown"],
            ),
            "dropdown",
        ),
        (multiselect, "multiselect"),
    ] {
        let text = script_pack_settings_manifest(&setting);
        let document = parse(text.as_bytes())
            .unwrap_or_else(|rejected| panic!("{kind}: {:#?}", rejected.diagnostics));
        assert_eq!(
            document.format,
            bedrock_addon::manifest::ManifestFormat::V3,
            "{kind}"
        );
        assert_eq!(document.settings.len(), 1, "{kind}");
        assert_eq!(document.settings[0].type_name(), kind);
        assert!(
            document
                .diagnostics()
                .iter()
                .all(|d| d.severity < Severity::Error),
            "{kind}: {:#?}",
            document.diagnostics()
        );
    }
}

/// `MultiSelectSetting` binds an optional `selection_text` string;
/// the engine test inserts it
/// with its two fragments, and `ScriptPackSettingsTests-multiselect-filled.json` is that
/// manifest assembled.
#[test]
fn a_multiselect_s_selection_text_loads() {
    let setting = fill(
        &template("ScriptPackSettingsTests-multiselect.json"),
        &[
            "e:multiselect",
            &format!(
                "{}select something{}",
                template("ScriptPackSettingsTests-multiselect-selection_text-fragment.txt"),
                template("ScriptPackSettingsTests-multiselect-fragment2.txt").trim_end()
            ),
        ],
    );
    let filled = template("ScriptPackSettingsTests-multiselect-filled.json");
    for text in [script_pack_settings_manifest(&setting), filled] {
        let document =
            parse(text.as_bytes()).unwrap_or_else(|rejected| panic!("{:#?}", rejected.diagnostics));
        assert!(
            document
                .diagnostics()
                .iter()
                .all(|d| d.severity < Severity::Error),
            "{:#?}",
            document.diagnostics()
        );
        assert_eq!(document.settings[0].type_name(), "multiselect");
        assert!(
            matches!(
                &document.settings[0],
                Setting::Multiselect { selection_text: Some(t), defaults, .. }
                    if t == "select something" && defaults == &["one", "two", "three"]
            ),
            "{:#?}",
            document.settings
        );
        // Written and read back to the same manifest, written the same.
        let written = document.to_json().unwrap();
        assert!(
            written.contains("\"selection_text\": \"select something\""),
            "{written}"
        );
        let again = parse(written.as_bytes()).unwrap();
        assert_eq!(again.value(), document.value());
        assert_eq!(again.to_json().unwrap(), written);
    }
    // Left out: `None`, and nothing is written.
    let text = script_pack_settings_manifest(&fill(
        &template("ScriptPackSettingsTests-multiselect.json"),
        &["e:multiselect", ""],
    ));
    let document = parse(text.as_bytes()).unwrap();
    assert!(matches!(
        &document.settings[0],
        Setting::Multiselect {
            selection_text: None,
            ..
        }
    ));
    assert!(!document.to_json().unwrap().contains("selection_text"));
    // Another JSON type, `null` included (rule N2), fails the alternative, and no kind fits.
    for value in ["1", "null", "[]"] {
        let text = script_pack_settings_manifest(&fill(
            &template("ScriptPackSettingsTests-multiselect.json"),
            &["e:multiselect", &format!(r#""selection_text": {value},"#)],
        ));
        let rejected = parse(text.as_bytes()).unwrap_err();
        assert!(
            has(&rejected.diagnostics, Severity::Error, "settings/0", |c| {
                c.id() == "core/invalid_type"
            }),
            "{value}: {:#?}",
            rejected.diagnostics
        );
    }
}

/// `rejectEmpty` on `DropdownOption.name` fails the cereal read
/// first, so the validator's "… has an option with an empty name" is never reached; the
/// finding is the constraint's, at the name.
#[test]
fn an_option_with_an_empty_name_is_the_constraint_s_error_not_v3_or_v7() {
    for (setting, kind) in [
        (
            r#"{ "type": "dropdown", "name": "c:d", "text": "t", "default": "a",
                 "options": [{ "name": "", "text": "x" }, { "name": "a", "text": "y" }] }"#,
            "dropdown",
        ),
        (
            r#"{ "type": "multiselect", "name": "e:m", "text": "t", "defaults": [],
                 "options": [{ "name": "", "text": "x" }, { "name": "a", "text": "y" }] }"#,
            "multiselect",
        ),
    ] {
        let rejected = parse(script_pack_settings_manifest(setting).as_bytes()).unwrap_err();
        assert!(
            invalid_at(&rejected.diagnostics, "settings/0/options/0/name"),
            "{kind}: {:#?}",
            rejected.diagnostics
        );
        // Every error is the constraint's text-less `InvalidValue`: no validator text (V3/V7).
        assert!(
            rejected
                .diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Error)
                .all(|d| d.is(&core_codes::INVALID_VALUE)),
            "{kind}: {:#?}",
            rejected.diagnostics
        );
    }
}
