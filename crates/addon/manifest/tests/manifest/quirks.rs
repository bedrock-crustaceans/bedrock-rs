//! The manifest quirk table of 12, one test per quirk, and the format-3 reader's rules.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::pack_identity::PackType;
use bedrock_addon_core::version::{MinEngineVersion, SemVersion};
use bedrock_addon_manifest::{
    Capability, CapabilityRegistry, CapabilityValidation, Dependency, ManifestFormat, ModuleType,
    NativeModuleVersion, TrustLevel,
};
use uuid::Uuid;

use super::{format_2, format_3, has, loads, parse_str, rejected};

const HEADER: &str = "11111111-1111-1111-1111-111111111111";

/// A manifest of `format_version` `format` (as JSON text) with format-1/2 keys.
fn legacy_with_format(format: &str) -> String {
    format_2("", "").replacen(
        "\"format_version\": 2",
        &format!("\"format_version\": {format}"),
        1,
    )
}

// ---- 1. `format_version` u8 truncation and type coercion ----

#[test]
fn quirk_format_version_truncates_to_u8_and_coerces_other_types() {
    let format = |text: &str| loads(&legacy_with_format(text)).original_format;
    // A number below 3 is read as its text by the input (1.9 is 1.9.0, the legacy slice) and
    // then by the legacy parser's integer read, truncated.
    assert_eq!(format("1.9"), ManifestFormat::V1);
    assert_eq!(format("1"), ManifestFormat::V1);
    // A string version below 3.0.0 reaches the legacy parser, which reads a string as 0: the
    // deprecated layout, which this manifest does not have.
    let diagnostics = rejected(&legacy_with_format("\"2\""));
    assert!(has(
        &diagnostics,
        Severity::Error,
        "header/pack_id",
        |c| matches!(c.id(), "core/missing_property")
    ));
    // A number read as its text from 3.0.0 on is no legacy manifest; `true`, `[2]`,
    // `{}` and negative numbers give no version at all.
}

// ---- 2. comments ----

#[test]
fn quirk_comments_are_accepted() {
    let text = format_2("", "").replacen(
        "\"header\": {",
        "// a line comment\n  \"header\": { /* a block comment */",
        1,
    );
    assert!(loads(&text).diagnostics().is_empty());
}

// ---- 3. trailing commas (a deviation for formats 0, 1 and 2 only) ----

#[test]
fn deviation_quirk_trailing_commas_are_accepted_in_a_legacy_manifest() {
    // vanilla's jsoncpp rejects them in a legacy manifest; facet accepts them
    // silently (`vanilla = "parse error"`). Format 3 is not a deviation (below).
    let text = format_2(
        ",\n    \"description\": \"d\",",
        ",\n  \"capabilities\": [\"pbr\",],",
    );
    let document = loads(&text);
    assert_eq!(document.header.description.as_deref(), Some("d"));
    assert_eq!(document.capabilities, [Capability::Pbr]);
}

/// The engine's own format-3 manifest (`ScriptPackSettingsTests`) has a
/// trailing comma in each of its two `dependencies` objects and its test loads the packs, so
/// accepting them in format 3 is vanilla's behaviour.
#[test]
fn quirk_trailing_commas_in_a_format_3_manifest_are_accepted_as_vanilla_accepts_them() {
    let text = format_3(
        ",\n    \"description\": \"d\",",
        ",\n  \"dependencies\": [{ \"module_name\": \"@minecraft/server\", \"version\": \"1.0.0\", }],",
    );
    let document = loads(&text);
    assert_eq!(document.format, ManifestFormat::V3);
    assert_eq!(document.header.description.as_deref(), Some("d"));
    assert_eq!(document.dependencies.len(), 1);
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
}

// ---- 4. duplicate header / module UUIDs ----

#[test]
fn quirk_a_module_with_the_header_uuid_is_an_error_from_format_1() {
    let text = format_2("", "").replace("22222222-2222-2222-2222-222222222222", HEADER);
    let diagnostics = rejected(&text);
    assert!(has(
        &diagnostics,
        Severity::Error,
        "header/uuid",
        |c| matches!(c.id(), "core/duplicate_uuid")
    ));
}

#[test]
fn quirk_format_0_resolves_a_module_with_the_header_uuid_with_a_warning() {
    let text = format!(
        r#"{{ "format_version": 0, "header": {{ "name": "Old", "pack_id": "{HEADER}",
           "packs_version": [1, 0, 0],
           "modules": [ {{ "type": "data", "uuid": "{HEADER}", "version": [1, 0, 0] }} ] }} }}"#
    );
    let document = loads(&text);
    assert!(has(
        document.diagnostics(),
        Severity::Warning,
        "header/pack_id",
        |c| matches!(c.id(), "core/duplicate_uuid")
    ));
    assert_ne!(document.header.uuid, document.modules[0].uuid);
    assert_eq!(document.format, ManifestFormat::V1);
    assert_eq!(document.original_format, ManifestFormat::V0);
}

// ---- 5. the last module decides the pack type ----

#[test]
fn quirk_the_last_module_decides_the_pack_type() {
    let two = |first: &str, last: &str| {
        format!(
            r#"{{ "format_version": 2, "header": {{ "name": "x", "uuid": "{HEADER}", "version": [1, 0, 0] }},
               "modules": [
                 {{ "type": "{first}", "uuid": "22222222-2222-2222-2222-222222222222", "version": [1, 0, 0] }},
                 {{ "type": "{last}", "uuid": "33333333-3333-3333-3333-333333333333", "version": [1, 0, 0] }} ] }}"#
        )
    };
    assert_eq!(
        loads(&two("resources", "skin_pack")).pack_type(),
        PackType::Skins
    );
    assert_eq!(
        loads(&two("skin_pack", "persona_piece")).pack_type(),
        PackType::PersonaPiece
    );
    // Modules of different behaviour / resource kinds are refused (`MultipleModules`).
    let diagnostics = rejected(&two("data", "resources"));
    assert!(has(&diagnostics, Severity::Error, "modules", |c| matches!(
        c.id(),
        "core/multiple_modules"
    )));
}

// ---- 6. the 16-entry module-type table ----

#[test]
fn quirk_the_module_type_table_has_sixteen_names() {
    let table = [
        ("invalid", PackType::Invalid),
        ("addon", PackType::Addon),
        ("data", PackType::Behavior),
        ("plugin", PackType::Behavior),
        ("client_data", PackType::Behavior),
        ("interface", PackType::Behavior),
        ("behaviorpack", PackType::Behavior),
        ("script", PackType::Behavior),
        ("resourcepack", PackType::Resource),
        ("resources", PackType::Resource),
        ("client_script", PackType::Resource),
        ("world_template", PackType::WorldTemplate),
        ("worldtemplate", PackType::WorldTemplate),
        ("skin_pack", PackType::Skins),
        ("skinpack", PackType::Skins),
        ("persona_piece", PackType::PersonaPiece),
    ];
    assert_eq!(ModuleType::ALL.len(), 16);
    for (name, pack_type) in table {
        let ty = ModuleType::from_name(name).unwrap();
        assert_eq!(ty.name(), name);
        assert_eq!(ty.pack_type(), pack_type, "{name}");
    }
    assert_eq!(ModuleType::from_name("javascript"), None);
    let diagnostics =
        rejected(&format_2("", "").replace("\"type\": \"data\"", "\"type\": \"javascript\""));
    assert!(has(
        &diagnostics,
        Severity::Error,
        "modules/0/type",
        |c| matches!(c.id(), "core/manifest_invalid_value")
    ));
}

// ---- 7. version arrays truncate reals ----

#[test]
fn quirk_version_arrays_truncate_reals() {
    let document = loads(
        &format_2("", "").replace("\"version\": [1, 0, 0],", "\"version\": [1.9, 2.5, 0.99],"),
    );
    assert_eq!(document.header.version, SemVersion::new(1, 2, 0));
    // Ours: each truncated element is told, since the manifest is written with the
    // version read.
    for index in 0..3 {
        assert!(
            has(
                document.diagnostics(),
                Severity::Info,
                &format!("header/version/{index}"),
                |c| c.is(&core_codes::INVALID_VALUE)
            ),
            "{index}: {:?}",
            document.diagnostics()
        );
    }
}

/// The game takes every element convertible to an unsigned integer: a `null`
/// is 0 and a bool 0 or 1. Ours tells each at the element; a whole number written as
/// a real (`2.0`) is the number it spells and is not told.
#[test]
fn a_version_array_element_read_as_another_value_is_told_at_the_element() {
    let document = loads(
        &format_2("", "").replace("\"version\": [1, 0, 0],", "\"version\": [true, null, 2.0],"),
    );
    assert_eq!(document.header.version, SemVersion::new(1, 0, 2));
    for index in 0..2 {
        assert!(
            has(
                document.diagnostics(),
                Severity::Info,
                &format!("header/version/{index}"),
                |c| c.is(&core_codes::INVALID_TYPE)
            ),
            "{index}: {:?}",
            document.diagnostics()
        );
    }
    assert!(
        document
            .diagnostics()
            .iter()
            .all(|f| f.location.json_path.to_string() != "header/version/2"),
        "{:?}",
        document.diagnostics()
    );
    // An array of plain integers says nothing.
    let plain = loads(&format_2("", ""));
    assert!(
        plain.diagnostics().iter().all(|f| !f
            .location
            .json_path
            .to_string()
            .starts_with("header/version")),
        "{:?}",
        plain.diagnostics()
    );
}

// ---- 8. version arrays reject elements above 0xFFFF ----

#[test]
fn quirk_version_array_elements_above_0xffff_are_rejected() {
    let max =
        loads(&format_2("", "").replace("\"version\": [1, 0, 0],", "\"version\": [65535, 0, 0],"));
    assert_eq!(max.header.version, SemVersion::new(65535, 0, 0));
    let diagnostics = rejected(
        &format_2("", "").replace("\"version\": [1, 0, 0],", "\"version\": [65536, 0, 0],"),
    );
    assert!(has(
        &diagnostics,
        Severity::Error,
        "header/version",
        |c| matches!(c.id(), "core/manifest_wrong_type")
    ));
}

// ---- 9. the 1.12.0 cap on format ≤ 1 ----

#[test]
fn quirk_a_format_1_min_engine_version_is_capped_to_1_12_0() {
    let text = legacy_with_format("1").replace("[1, 26, 20]", "[1, 13, 0]");
    let document = loads(&text);
    assert_eq!(
        document.header.min_engine_version,
        Some(MinEngineVersion::FORMAT_1_CAP)
    );
    assert!(has(
        document.diagnostics(),
        Severity::Warning,
        "header/min_engine_version",
        |c| matches!(c.id(), "core/min_engine_version_cap_format_version1")
    ));
    // Below the cap: kept. Format 2 has no cap but a floor of 1.13.0.
    let low = loads(&legacy_with_format("1").replace("[1, 26, 20]", "[1, 12, 0]"));
    assert_eq!(
        low.header.min_engine_version,
        Some(MinEngineVersion::new(1, 12, 0))
    );
    let diagnostics = rejected(&format_2("", "").replace("[1, 26, 20]", "[1, 12, 0]"));
    assert!(has(
        &diagnostics,
        Severity::Error,
        "header/min_engine_version",
        |c| matches!(c.id(), "core/version_too_low")
    ));
}

// ---- 10. the 1.2.5 default on format ≤ 1 ----

#[test]
fn quirk_a_format_1_pack_without_min_engine_version_gets_1_2_5() {
    let text = legacy_with_format("1").replace(",\n    \"min_engine_version\": [1, 26, 20]", "");
    let document = loads(&text);
    assert_eq!(
        document.header.min_engine_version,
        Some(MinEngineVersion::FORMAT_1_DEFAULT)
    );
    assert!(document.diagnostics().is_empty());
    // Format 2 requires it from behaviour and resource packs.
    let diagnostics =
        rejected(&format_2("", "").replace(",\n    \"min_engine_version\": [1, 26, 20]", ""));
    assert!(has(
        &diagnostics,
        Severity::Error,
        "header/min_engine_version",
        |c| matches!(c.id(), "core/missing_property")
    ));
}

// ---- 11. an explicit nil UUID is malformed ----

#[test]
fn quirk_an_explicit_nil_uuid_is_malformed() {
    let text = format_2("", "").replace(HEADER, "00000000-0000-0000-0000-000000000000");
    let diagnostics = rejected(&text);
    assert!(has(
        &diagnostics,
        Severity::Error,
        "header/uuid",
        |c| matches!(c.id(), "core/malformed_property_uuid")
    ));
}

// ---- 12. ASCII-only case folding ----

#[test]
fn quirk_case_folding_is_ascii_only() {
    // UUIDs compare without ASCII case; a non-ASCII look-alike is malformed.
    let upper = loads(&format_2("", "").replace(HEADER, "AAAAAAAA-1111-1111-1111-111111111111"));
    let lower = loads(&format_2("", "").replace(HEADER, "aaaaaaaa-1111-1111-1111-111111111111"));
    assert_eq!(upper.header.uuid, lower.header.uuid);
    let fullwidth = format_2("", "").replace(HEADER, "ＡAAAAAAA-1111-1111-1111-111111111111");
    let diagnostics = rejected(&fullwidth);
    assert!(has(
        &diagnostics,
        Severity::Error,
        "header/uuid",
        |c| matches!(c.id(), "core/malformed_property_uuid")
    ));
    // Module types are matched exactly: no case folding at all.
    let diagnostics =
        rejected(&format_2("", "").replace("\"type\": \"data\"", "\"type\": \"DATA\""));
    assert!(has(
        &diagnostics,
        Severity::Error,
        "modules/0/type",
        |c| matches!(c.id(), "core/manifest_invalid_value")
    ));
}

// ---- 13. `(uuid, version)` is the pack identity ----

#[test]
fn quirk_the_identity_is_uuid_and_version() {
    let behavior = loads(&format_2("", ""));
    let resource =
        loads(&format_2("", "").replace("\"type\": \"data\"", "\"type\": \"resources\""));
    assert_ne!(behavior.identity().pack_type, resource.identity().pack_type);
    assert_eq!(behavior.identity(), resource.identity());
    let newer =
        loads(&format_2("", "").replace("\"version\": [1, 0, 0],", "\"version\": [1, 0, 1],"));
    assert_ne!(behavior.identity(), newer.identity());
    assert_eq!(behavior.identity().uuid, Uuid::parse_str(HEADER).unwrap());
}

// ---- format 3 ----

#[test]
fn a_format_3_manifest_loads_with_version_strings() {
    let document = loads(&format_3("", ""));
    assert_eq!(document.format, ManifestFormat::V3);
    assert_eq!(document.header.version, SemVersion::new(1, 0, 0));
    assert_eq!(
        document.header.min_engine_version,
        Some(MinEngineVersion::new(1, 26, 20))
    );
    assert_eq!(document.pack_type(), PackType::Resource);
    assert_eq!(
        document.version().written,
        Some(bedrock_addon_core::version::FormatVersion::new(3, 0, 0))
    );
    assert!(document.version().upgraded.eq(&false));
}

/// The game reads the manifest through a `MinecraftDocumentInput`,
/// which reads a number `format_version` as its text:
/// the integer 3 is version 3.0.0, the comprehensive slice, whose bound member is a `u8`;
/// the embedded `ScriptPackSettingsTests` manifests use it and load.
#[test]
fn an_integer_format_version_3_selects_the_comprehensive_reader() {
    let document = loads(&format_3("", ""));
    assert_eq!(document.format, ManifestFormat::V3);
    assert_eq!(document.original_format, ManifestFormat::V3);
    // A number whose text is a version below 3.0.0 stays with the legacy reader.
    let document = loads(&legacy_with_format("2"));
    assert_eq!(document.original_format, ManifestFormat::V2);
}

#[test]
fn format_3_rejects_unknown_members_and_array_versions() {
    let unknown = rejected(&format_3(",\n    \"author\": \"me\"", ""));
    assert!(has(
        &unknown,
        Severity::Error,
        "header/author",
        |c| matches!(c.id(), "core/unrecognized_field")
    ));
    let array = rejected(&format_3("", "").replace(
        "\"version\": \"1.0.0\",\n    \"min",
        "\"version\": [1, 0, 0],\n    \"min",
    ));
    assert!(has(
        &array,
        Severity::Error,
        "header/version",
        |c| matches!(c.id(), "core/invalid_type")
    ));
    let module = rejected(&format_3("", "").replace(
        "\"type\": \"resources\",",
        "\"type\": \"resources\", \"main\": \"x\",",
    ));
    assert!(has(
        &module,
        Severity::Error,
        "modules/0/main",
        |c| matches!(c.id(), "core/unrecognized_field")
    ));
    let legacy_engine = rejected(&format_3("", "").replace("\"1.26.20\"", "[1, 26, 20]"));
    assert!(has(
        &legacy_engine,
        Severity::Error,
        "header/min_engine_version",
        |c| matches!(c.id(), "core/invalid_type")
    ));
}

// A manifest member whose key holds `/` or `~` is one step of the
// finding's path, as in every other family.
#[test]
fn an_unknown_manifest_member_whose_key_holds_a_slash_is_one_step_of_the_path() {
    for (key, written) in [("a/b", "header/a~1b"), ("a~b", "header/a~0b")] {
        let unknown = rejected(&format_3(&format!(",\n    \"{key}\": \"me\""), ""));
        assert!(
            has(&unknown, Severity::Error, written, |c| matches!(
                c.id(),
                "core/unrecognized_field"
            )),
            "{key:?}: {unknown:#?}"
        );
    }
}

#[test]
fn format_3_native_module_dependencies() {
    let deps = r#",
  "dependencies": [
    { "module_name": "@minecraft/server", "version": "2.0", "optional": true },
    { "module_name": "@minecraft/server-ui", "version": "beta" }
  ]"#;
    let document = loads(&format_3("", deps));
    assert_eq!(
        document.dependencies,
        [
            Dependency::NativeModule {
                module_name: "@minecraft/server".to_owned(),
                version: NativeModuleVersion::Version(SemVersion::new(2, 0, 0)),
                optional: Some(true),
            },
            Dependency::NativeModule {
                module_name: "@minecraft/server-ui".to_owned(),
                version: NativeModuleVersion::LatestBeta,
                optional: None,
            },
        ]
    );
    for (version, code) in [
        ("\"abc\"", "malformed"),
        ("\"*\"", "malformed"),
        ("[2, 0, 0]", "type"),
    ] {
        let text = format_3("", &deps.replace("\"2.0\"", version));
        let diagnostics = rejected(&text);
        let found = if code == "type" {
            has(
                &diagnostics,
                Severity::Error,
                "dependencies/0/version",
                |c| matches!(c.id(), "core/invalid_type"),
            )
        } else {
            has(
                &diagnostics,
                Severity::Error,
                "dependencies/0/version",
                |c| matches!(c.id(), "core/dependency_invalid_version"),
            )
        };
        assert!(found, "{version}: {diagnostics:?}");
    }
    let both = format_3(
        "",
        &deps.replace(
            "\"optional\": true",
            "\"uuid\": \"33333333-3333-3333-3333-333333333333\"",
        ),
    );
    assert!(has(
        &rejected(&both),
        Severity::Error,
        "dependencies/0/uuid",
        |c| matches!(c.id(), "core/unrecognized_field")
    ));
}

#[test]
fn legacy_native_module_dependencies_drop_a_uuid_with_a_warning() {
    let deps = r#",
  "dependencies": [
    { "module_name": "@minecraft/server", "version": [2, 0, 0] },
    { "module_name": "@minecraft/server", "uuid": "33333333-3333-3333-3333-333333333333", "version": "2.0.0" }
  ]"#;
    let document = loads(&format_2("", deps));
    assert_eq!(document.dependencies.len(), 1);
    assert!(has(
        document.diagnostics(),
        Severity::Warning,
        "dependencies/1",
        |c| matches!(c.id(), "core/invalid_property")
    ));
    // A legacy native version must be a full SemVer: `*` and `2.0` are refused.
    for version in ["\"*\"", "\"2.0\""] {
        let diagnostics = rejected(&format_2("", &deps.replace("[2, 0, 0]", version)));
        assert!(
            has(
                &diagnostics,
                Severity::Error,
                "dependencies/0/version",
                |c| matches!(c.id(), "core/manifest_not_semver")
            ),
            "{version}"
        );
    }
}

#[test]
fn capabilities_are_checked_against_the_registry() {
    let text = format_2(
        "",
        ",\n  \"capabilities\": [\"chemistry\", \"script_eval\", \"experimental_custom_ui\"]",
    );
    let document = loads(&text);
    assert!(has(
        document.diagnostics(),
        Severity::Info,
        "capabilities/0",
        |c| matches!(c.id(), "core/invalid_capability")
    ));
    assert!(has(
        document.diagnostics(),
        Severity::Info,
        "capabilities/2",
        |c| matches!(c.id(), "core/invalid_capability")
    ));
    assert_eq!(
        document.capabilities[2],
        Capability::Other("experimental_custom_ui".to_owned())
    );
    // With the education toggle on, chemistry is granted, trusted.
    let education = bedrock_addon_manifest::Manifest::parse(
        &PackPath::from_static("manifest.json"),
        text.as_bytes(),
        &CapabilityRegistry::server(true),
    )
    .unwrap();
    assert!(!has(
        education.diagnostics(),
        Severity::Warning,
        "capabilities/0",
        |_| true
    ));
    let mut registry = CapabilityRegistry::empty();
    registry.insert("pbr", |_| CapabilityValidation::Succeeded {
        trust: TrustLevel::Trusted,
    });
    assert_eq!(registry.names().collect::<Vec<_>>(), ["pbr"]);
}

#[test]
fn a_syntax_error_and_an_empty_file_are_rejected() {
    assert!(has(
        &rejected("{ \"format_version\": 2, "),
        Severity::Error,
        "",
        |c| matches!(c.id(), "core/syntax")
    ));
    assert!(has(&rejected("  \n"), Severity::Error, "", |c| matches!(
        c.id(),
        "core/empty_document"
    )));
    assert!(parse_str("\u{feff}{}").is_err());
}

/// The format-3 manifest is read by the game's strict schema reader (through a
/// `MinecraftDocumentInput`), where an optional member written as `null` is present and
/// fails its type (`0x8`), an error under 0xfffc: the manifest is refused.
#[test]
fn format_3_a_null_member_is_a_type_error_that_rejects() {
    let mut not_refused = Vec::new();
    for (header_extra, root_extra, path) in [
        (",\n    \"description\": null", "", "header/description"),
        (",\n    \"pack_scope\": null", "", "header/pack_scope"),
        (
            ",\n    \"base_game_version\": null",
            "",
            "header/base_game_version",
        ),
        (
            ",\n    \"platform_locked\": null",
            "",
            "header/platform_locked",
        ),
        (
            ",\n    \"lock_template_options\": null",
            "",
            "header/lock_template_options",
        ),
        ("", ",\n  \"metadata\": null", "metadata"),
        (
            "",
            ",\n  \"metadata\": { \"authors\": null }",
            "metadata/authors",
        ),
        ("", ",\n  \"metadata\": { \"url\": null }", "metadata/url"),
        (
            "",
            ",\n  \"metadata\": { \"license\": null }",
            "metadata/license",
        ),
        (
            "",
            ",\n  \"metadata\": { \"generated_with\": null }",
            "metadata/generated_with",
        ),
        ("", ",\n  \"dependencies\": null", "dependencies"),
        ("", ",\n  \"capabilities\": null", "capabilities"),
        ("", ",\n  \"settings\": null", "settings"),
        ("", ",\n  \"subpacks\": null", "subpacks"),
        (
            "",
            ",\n  \"has_education_metadata\": null",
            "has_education_metadata",
        ),
    ] {
        let refused = match parse_str(&format_3(header_extra, root_extra)) {
            Ok(_) => false,
            Err(rejected) => has(&rejected.diagnostics, Severity::Error, path, |c| {
                matches!(c.id(), "core/invalid_type")
            }),
        };
        if !refused {
            not_refused.push(path);
        }
    }
    assert!(
        not_refused.is_empty(),
        "not refused as a type error: {not_refused:?}"
    );
}

/// The legacy reader keeps jsoncpp's reading: a `null` metadata member is absent.
#[test]
fn legacy_a_null_metadata_member_stays_absent() {
    for root_extra in [
        ",\n  \"metadata\": null",
        ",\n  \"metadata\": { \"authors\": null, \"url\": null, \"generated_with\": null }",
    ] {
        let document = loads(&format_2("", root_extra));
        assert!(
            document
                .diagnostics()
                .iter()
                .all(|d| d.severity < Severity::Warning),
            "{root_extra}: {:#?}",
            document.diagnostics()
        );
    }
}

/// The game reads the manifest through a `MinecraftDocumentInput`, which picks the first
/// textual `format_version`; a value that gives no version is `0x10` "Malformed format
/// version" in the input's log, and the loader returns
/// an invalid result without running a slice: no legacy or comprehensive
/// reader sees the manifest. Absent stays the caller's 0.0.0, the legacy reader.
#[test]
fn a_format_version_that_gives_no_version_is_malformed_on_both_paths() {
    for (value, builder) in [
        ("true", 2),
        ("false", 2),
        ("null", 2),
        ("{}", 2),
        ("\"abc\"", 2),
        ("\"*\"", 2),
        ("\"03\"", 2),
        ("\"3.0.0.0\"", 3),
        ("[3, 0]", 3),
        ("[2]", 2),
        ("[-1, 0, 0]", 2),
        ("-1", 2),
        ("-255", 2),
    ] {
        let text = if builder == 2 {
            legacy_with_format(value)
        } else {
            format_3_at(value)
        };
        let diagnostics = rejected(&text);
        assert!(
            has(
                &diagnostics,
                Severity::Error,
                "format_version",
                |c| matches!(c.id(), "core/malformed_format_version")
            ),
            "{value}: {diagnostics:#?}"
        );
        assert_eq!(
            diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Error)
                .count(),
            1,
            "{value}: nothing else is read"
        );
    }
    // Absent: the legacy reader at format 0 (the caller's version), as before.
    let absent = rejected(&legacy_with_format("2").replacen("\"format_version\": 2,", "", 1));
    assert!(has(
        &absent,
        Severity::Error,
        "header/pack_id",
        |c| matches!(c.id(), "core/missing_property")
    ));
}

/// The comprehensive slice ends at 3.0.0: the game's loader
/// keeps `VERSION < currGameVers ? currGameVers : VERSION`, i.e.
/// max(3.0.0, 1.26.60) = 3.0.0, so the slice is [3.0.0, 3.0.0] and no parser takes a later
/// version: "unsupported version: no parser available for version <v>" (the loader),
/// the manifest refused. `"beta"` has no Beta slice either.
#[test]
fn a_format_version_above_3_has_no_parser() {
    let three = format_3("", "");
    assert_eq!(loads(&three).format, ManifestFormat::V3);
    for (value, shown) in [
        ("4", "4.0.0"),
        ("\"3.0.1\"", "3.0.1"),
        ("3.1", "3.1.0"),
        ("258", "258.0.0"),
        ("\"beta\"", "beta"),
    ] {
        let diagnostics = rejected(&three.replacen(
            "\"format_version\": 3",
            &format!("\"format_version\": {value}"),
            1,
        ));
        let text = format!("unsupported version: no parser available for version {shown}");
        assert!(
            diagnostics.iter().any(|d| d.severity == Severity::Error
                && d.location.json_path.to_string() == "format_version"
                && matches!(d.code.id(), "core/no_parser_for_version")
                && d.code.to_string() == text),
            "{value}: {diagnostics:#?}"
        );
        assert_eq!(
            diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Error)
                .count(),
            1,
            "{value}"
        );
    }
}

/// `PackManifestDocument::format_version` is a `u8`, read by
/// the game's strict schema reader: only an unsigned integer below 256 binds; a string, an
/// array or a real that the input read as 3.0.0 fails the bind, a type error under 0xfffc.
#[test]
fn a_format_3_manifest_s_format_version_is_the_integer_3() {
    assert_eq!(loads(&format_3("", "")).format, ManifestFormat::V3);
    for value in ["\"3.0.0\"", "\"3\"", "\"3.0.0+build\"", "[3, 0, 0]", "3.0"] {
        let diagnostics = rejected(&format_3_at(value));
        assert!(
            has(
                &diagnostics,
                Severity::Error,
                "format_version",
                |c| matches!(c.id(), "core/invalid_type")
            ),
            "{value}: {diagnostics:#?}"
        );
    }
}

/// [`format_3`] with `format_version` written as `value`.
fn format_3_at(value: &str) -> String {
    format_3("", "").replacen(
        "\"format_version\": 3",
        &format!("\"format_version\": {value}"),
        1,
    )
}
