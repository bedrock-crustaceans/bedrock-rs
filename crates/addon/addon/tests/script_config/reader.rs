//! The per-folder files: `permissions.json`, `variables.json`, `secrets.json` (fixtures
//! sc11–sc15, sc17–sc32, sc46; validation F1–F6, M1–M9, V1–V2, S1–S2).

use bedrock_addon::diagnostic::{DiagnosticCode, Severity, Span};
use bedrock_addon::families::script::config::{
    CommandPermissionLevel, ConfigFolder, ConfigSource, ScriptConfig, ScriptConfigFiles,
    UnreadableFile,
};
use bedrock_addon::json::Value;
use bytes::Bytes;

use crate::common::{MODULE, VARS, check_config, files};

/// A `config/` with one folder `folder` holding `permissions.json` = `text`.
fn permissions(folder: &str, text: &str) -> ScriptConfig {
    let mut entry = ConfigFolder::new(folder);
    entry.permissions = Some(Bytes::copy_from_slice(text.as_bytes()));
    ScriptConfig::parse(&ScriptConfigFiles {
        folders: vec![entry],
        ..ScriptConfigFiles::default()
    })
}

fn variables(text: &str) -> ScriptConfig {
    let mut entry = ConfigFolder::new("default");
    entry.variables = Some(Bytes::copy_from_slice(text.as_bytes()));
    ScriptConfig::parse(&ScriptConfigFiles {
        folders: vec![entry],
        ..ScriptConfigFiles::default()
    })
}

fn modules(config: &ScriptConfig, folder: &str) -> Vec<String> {
    config.packs[folder].permissions.allowed_modules.clone()
}

// ---- allowed_modules ----

#[test]
fn sc11_wildcard_is_kept_as_written() {
    let config = check_config("sc11_wildcard");
    assert_eq!(config.default.permissions.allowed_modules, ["*"]);
    assert_eq!(config.default.source, ConfigSource::Default);
}

#[test]
fn sc12_version_suffix_is_kept_and_warned() {
    let config = check_config("sc12_version_suffix");
    assert_eq!(
        config.default.permissions.allowed_modules,
        ["@minecraft/server@2.0.0", "@minecraft/server-ui"]
    );
}

#[test]
fn sc13_case_is_kept_and_warned() {
    let config = check_config("sc13_case");
    assert_eq!(
        config.default.permissions.allowed_modules,
        ["@Minecraft/Server"]
    );
}

#[test]
fn m4_a_name_that_is_no_near_miss_is_not_warned() {
    // `@minecraft/server-editor` is in the shipped file and not in the catalog; an unknown
    // name with a suffix is left alone too.
    let config = permissions(
        "default",
        r#"{"allowed_modules": ["@minecraft/server-editor", "@acme/tool@1.0.0", "@minecraft/server"]}"#,
    );
    assert!(
        config.diagnostics().is_empty(),
        "{:?}",
        config.diagnostics()
    );
}

#[test]
fn m5_a_wildcard_beside_other_modules_is_an_info() {
    let config = permissions(
        "default",
        r#"{"allowed_modules": ["@minecraft/server", "*"]}"#,
    );
    let findings: Vec<_> = config.diagnostics().iter().collect();
    assert_eq!(findings.len(), 1);
    assert!(matches!(findings[0].code, DiagnosticCode::NoEffect { .. }));
    assert_eq!(findings[0].severity, Severity::Info);
    assert_eq!(
        findings[0].location.json_path.to_string(),
        "allowed_modules/1"
    );
    // Alone it says everything there is to say.
    assert!(
        permissions("default", r#"{"allowed_modules": ["*"]}"#)
            .diagnostics()
            .is_empty()
    );
}

#[test]
fn sc14_string_not_array_is_an_empty_list() {
    let config = check_config("sc14_string_not_array");
    assert!(config.default.permissions.allowed_modules.is_empty());
}

#[test]
fn sc15_mixed_elements_keep_the_strings() {
    let config = check_config("sc15_mixed_elements");
    assert_eq!(
        config.default.permissions.allowed_modules,
        ["@minecraft/server", "@minecraft/server-admin"]
    );
}

// ---- the JSON reader ----

#[test]
fn sc17_syntax_error_permissions_names_the_right_file_deviation() {
    // Vanilla prints the sibling `variables.json`; ours names `permissions.json`.
    let config = check_config("sc17_syntax_error_permissions");
    assert!(modules(&config, MODULE).is_empty());
    let finding = config.diagnostics().first_error().unwrap();
    assert!(matches!(
        finding.code,
        DiagnosticCode::MalformedInput { .. }
    ));
    assert!(finding.code.to_string().ends_with("/permissions.json'"));
}

#[test]
fn sc18_empty_file_is_refused() {
    let config = check_config("sc18_empty_file");
    assert!(modules(&config, MODULE).is_empty());
    // White space alone is an empty file too.
    let blank = permissions("default", " \n\t");
    assert!(matches!(
        blank.diagnostics().first_error().unwrap().code,
        DiagnosticCode::EmptyDocument { .. }
    ));
}

#[test]
fn sc19_comments_load() {
    let config = check_config("sc19_comments");
    assert_eq!(modules(&config, MODULE), ["@minecraft/server"]);
}

#[test]
fn sc20_trailing_comma_loads_deviation() {
    // Vanilla: a parse error and the file dropped; the core rule accepts it.
    let config = check_config("sc20_trailing_comma");
    assert_eq!(
        modules(&config, MODULE),
        ["@minecraft/server", "@minecraft/server-ui"]
    );
}

#[test]
fn sc21_bom_is_skipped() {
    let config = check_config("sc21_bom");
    assert_eq!(modules(&config, MODULE), ["@minecraft/server"]);
}

#[test]
fn a_span_counts_from_the_start_of_the_file_the_bom_included() {
    let text = "\u{feff}{\"allowed_modules\": [5]}";
    let config = permissions("default", text);
    let finding = config.diagnostics().iter().next().unwrap();
    assert!(matches!(finding.code, DiagnosticCode::InvalidType { .. }));
    let offset = u32::try_from(text.find('5').unwrap()).unwrap();
    assert_eq!(finding.location.span, Some(Span { offset, len: 1 }));
}

#[test]
fn sc22_trailing_garbage_loads_with_a_warning() {
    let config = check_config("sc22_trailing_garbage");
    assert_eq!(modules(&config, MODULE), ["@minecraft/server"]);
    let finding = config.diagnostics().iter().next().unwrap();
    assert_eq!(finding.location.span, Some(Span { offset: 43, len: 3 }));
}

#[test]
fn sc23_root_array_leaves_each_part_empty() {
    let config = check_config("sc23_root_array");
    let pack = &config.packs[MODULE];
    assert!(pack.permissions.allowed_modules.is_empty());
    assert!(pack.variables.is_empty());
    assert!(pack.secrets.is_empty());
}

#[test]
fn sc24_null_root_is_empty() {
    let config = check_config("sc24_null_root");
    assert!(modules(&config, MODULE).is_empty());
}

#[test]
fn f5_a_string_or_number_root_is_a_warning_and_empty() {
    for text in ["\"text\"", "5", "true"] {
        let config = permissions("default", text);
        let findings: Vec<_> = config.diagnostics().iter().collect();
        assert_eq!(findings.len(), 1, "{text}");
        assert!(matches!(
            findings[0].code,
            DiagnosticCode::InvalidType { .. }
        ));
        assert_eq!(findings[0].severity, Severity::Info);
        assert_eq!(config.default, ScriptConfig::default().default);
    }
}

#[test]
fn sc25_unreadable_is_a_read_error_and_the_part_is_empty() {
    // The file exists and cannot be opened (mode 000 on the probe); the other files of the
    // folder are used.
    let name = "sc25_unreadable";
    let mut files = files(name);
    files.unreadable.push(UnreadableFile {
        folder: Some(MODULE.into()),
        file: "variables.json".into(),
        kind: std::io::ErrorKind::PermissionDenied,
    });
    let config = ScriptConfig::parse(&files);
    crate::common::assert_findings(name, &["parse"], config.diagnostics());
    assert!(config.packs[MODULE].variables.is_empty());
    assert_eq!(modules(&config, MODULE), ["@minecraft/server"]);
    let finding = config.diagnostics().first_error().unwrap();
    assert!(matches!(
        finding.code,
        DiagnosticCode::ReadError {
            kind: std::io::ErrorKind::PermissionDenied,
            ..
        }
    ));
}

#[test]
fn f2_an_unreadable_permissions_file_names_itself_deviation() {
    // Vanilla prints the sibling `variables.json` path for it.
    let unreadable = UnreadableFile::new(
        Some(MODULE),
        "permissions.json",
        std::io::ErrorKind::PermissionDenied,
    );
    assert_eq!(
        unreadable,
        UnreadableFile {
            folder: Some(MODULE.into()),
            file: "permissions.json".into(),
            kind: std::io::ErrorKind::PermissionDenied,
        }
    );
    let files = ScriptConfigFiles {
        folders: vec![ConfigFolder::new(MODULE)],
        unreadable: vec![unreadable],
        ..ScriptConfigFiles::default()
    };
    let config = ScriptConfig::parse(&files);
    let finding = config.diagnostics().first_error().unwrap();
    assert!(matches!(
        finding.code,
        DiagnosticCode::ReadError {
            kind: std::io::ErrorKind::PermissionDenied,
            ..
        }
    ));
    assert_eq!(
        finding.code.to_string(),
        format!("Failed to open file 'config/{MODULE}/permissions.json'")
    );
}

/// `variables.json` whose one variable is `arrays` arrays deep: the file nests `arrays` + 1
/// levels.
fn nested(arrays: usize) -> ScriptConfig {
    variables(&format!(
        "{{\"v\": {}1{}}}",
        "[".repeat(arrays),
        "]".repeat(arrays)
    ))
}

#[test]
fn f6_a_file_nested_128_levels_loads_and_one_nested_129_is_refused() {
    // On the test's own thread: the reader holds a file at the limit on any thread a load
    // runs on (a megabyte: `script::config::tests`, in-crate).
    // 128 levels: the root object and 127 arrays.
    let at_the_limit = nested(127);
    assert!(at_the_limit.diagnostics().is_empty());
    assert_eq!(at_the_limit.default.variables.len(), 1);
    // 129 levels.
    let past_it = nested(128);
    let finding = past_it.diagnostics().first_error().unwrap();
    assert!(matches!(
        finding.code,
        DiagnosticCode::MalformedInput { .. }
    ));
    assert_eq!(past_it.diagnostics().len(), 1);
    assert!(past_it.default.variables.is_empty());
}

#[test]
fn a_file_that_is_not_utf8_is_refused() {
    let mut entry = ConfigFolder::new("default");
    entry.variables = Some(Bytes::from_static(b"{\"a\": \"\xff\"}"));
    let config = ScriptConfig::parse(&ScriptConfigFiles {
        folders: vec![entry],
        ..ScriptConfigFiles::default()
    });
    assert!(matches!(
        config.diagnostics().first_error().unwrap().code,
        DiagnosticCode::MalformedInput { .. }
    ));
}

#[test]
fn sc26_extra_member_is_kept() {
    let config = check_config("sc26_extra_member");
    assert_eq!(
        config.default.permissions.allowed_modules,
        ["@minecraft/server"]
    );
    assert_eq!(
        config.default.permissions.extra.get("foo"),
        Some(&Value::from(1_i64))
    );
}

#[test]
fn a_member_named_extra_is_an_unknown_member() {
    let config = permissions(
        "default",
        r#"{"extra": {"a": 1}, "allowed_modules": ["*"]}"#,
    );
    assert_eq!(config.default.permissions.allowed_modules, ["*"]);
    assert!(config.default.permissions.extra.get("extra").is_some());
}

#[test]
fn folders_are_read_in_name_order_whatever_the_listing_order() {
    let folder = |name: &str| {
        let mut entry = ConfigFolder::new(name);
        entry.permissions = Some(Bytes::from_static(b"{ broken"));
        entry
    };
    let listing = |names: &[&str]| {
        ScriptConfig::parse(&ScriptConfigFiles {
            folders: names.iter().map(|name| folder(name)).collect(),
            ..ScriptConfigFiles::default()
        })
    };
    let one = listing(&["b", "default", "a"]);
    let other = listing(&["default", "a", "b"]);
    assert_eq!(one, other);
    assert_eq!(one.packs.keys().collect::<Vec<_>>(), ["a", "b"]);
    let files: Vec<String> = crate::common::seen(one.diagnostics())
        .into_iter()
        .map(|finding| finding.file)
        .collect();
    assert_eq!(
        files,
        [
            "config/a/permissions.json",
            "config/b/permissions.json",
            "config/default/permissions.json"
        ]
    );
}

// ---- module_permissions and commands_permission_level ----

#[test]
fn sc46_other_module_key_is_kept() {
    let config = check_config("sc46_other_module_key");
    let settings = &config.default.permissions.module_permissions["@minecraft/server"];
    assert_eq!(settings["a"], Value::from(1_i64));
}

#[test]
fn an_empty_array_is_no_object_at_any_map_level() {
    // jsoncpp's object reads refuse an array, an empty one too.
    for (text, path) in [
        (r#"{"module_permissions": []}"#, "module_permissions"),
        (
            r#"{"module_permissions": {"@acme/other": []}}"#,
            "module_permissions/@acme~1other",
        ),
        (
            r#"{"module_permissions": {"@minecraft/server-net": []}}"#,
            "module_permissions/@minecraft~1server-net",
        ),
        ("[]", ""),
    ] {
        let config = permissions("default", text);
        let findings = crate::common::seen(config.diagnostics());
        assert_eq!(findings.len(), 1, "{text}");
        assert_eq!(findings[0].code, "InvalidType", "{text}");
        assert_eq!(findings[0].severity, "info", "{text}");
        assert_eq!(findings[0].path, path, "{text}");
        assert!(config.default.permissions.module_permissions.is_empty());
    }
    for file in ["variables", "secrets"] {
        let mut entry = ConfigFolder::new("default");
        let bytes = Some(Bytes::from_static(b"[]"));
        if file == "variables" {
            entry.variables = bytes;
        } else {
            entry.secrets = bytes;
        }
        let config = ScriptConfig::parse(&ScriptConfigFiles {
            folders: vec![entry],
            ..ScriptConfigFiles::default()
        });
        let findings = crate::common::seen(config.diagnostics());
        assert_eq!(findings.len(), 1, "{file}");
        assert_eq!(findings[0].code, "InvalidType", "{file}");
        assert_eq!(findings[0].file, format!("config/default/{file}.json"));
    }
}

#[test]
fn m7_module_permissions_that_is_no_object_is_a_warning() {
    let config = permissions("default", r#"{"module_permissions": [1]}"#);
    let findings: Vec<_> = config.diagnostics().iter().collect();
    assert_eq!(findings.len(), 1);
    assert!(matches!(
        findings[0].code,
        DiagnosticCode::InvalidType { .. }
    ));
    assert_eq!(findings[0].severity, Severity::Info);
    assert_eq!(
        findings[0].location.json_path.to_string(),
        "module_permissions"
    );
    assert!(config.default.permissions.module_permissions.is_empty());
    // A module whose value is no object has no settings; the others keep theirs.
    let config = permissions(
        "default",
        r#"{"module_permissions": {"@minecraft/server": 5, "@minecraft/server-ui": {"k": true}}}"#,
    );
    let paths: Vec<String> = config
        .diagnostics()
        .iter()
        .filter(|d| matches!(d.code, DiagnosticCode::InvalidType { .. }))
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(paths, ["module_permissions/@minecraft~1server"]);
    assert_eq!(
        config
            .default
            .permissions
            .module_permissions
            .keys()
            .collect::<Vec<_>>(),
        ["@minecraft/server-ui"]
    );
}

#[test]
fn sc31_commands_level_names_parse_to_the_enum() {
    let config = check_config("sc31_commands_level_names");
    for level in CommandPermissionLevel::ALL {
        let folder = format!("l_{}", level.name());
        assert_eq!(
            config.packs[&folder].permissions.commands_permission_level,
            Some(level)
        );
    }
}

#[test]
fn the_level_names_are_the_six_of_the_schema_both_ways() {
    let names = [
        "any",
        "game_directors",
        "admin",
        "host",
        "owner",
        "internal",
    ];
    for (value, name) in names.into_iter().enumerate() {
        let level = CommandPermissionLevel::from_name(name).unwrap();
        assert_eq!(level as usize, value);
        assert_eq!(level.name(), name);
    }
    assert_eq!(CommandPermissionLevel::ALL.len(), names.len());
    for other in ["Any", "ANY", "operator", "", "game-directors"] {
        assert_eq!(CommandPermissionLevel::from_name(other), None, "{other}");
    }
}

#[test]
fn sc32_commands_level_invalid_is_unset_deviation() {
    // Vanilla is silent; ours warns.
    let config = check_config("sc32_commands_level_invalid");
    assert_eq!(
        config.packs["l_bogus"]
            .permissions
            .commands_permission_level,
        None
    );
    assert_eq!(
        config.packs["l_num"].permissions.commands_permission_level,
        None
    );
}

// ---- variables and secrets ----

#[test]
fn sc27_variable_types_are_kept_as_written() {
    let config = check_config("sc27_variable_types");
    let variables = &config.packs[VARS].variables;
    assert_eq!(
        variables.keys().collect::<Vec<_>>(),
        [
            "str", "num", "float", "neg", "big", "bool", "null", "arr", "nested", ""
        ]
    );
    assert_eq!(variables["str"], Value::from("a".to_owned()));
    assert_eq!(variables["num"].as_number().unwrap().to_i64(), Some(5));
    assert!(!variables["num"].as_number().unwrap().is_float());
    assert_eq!(variables["float"].as_number().unwrap().to_f64(), Some(1.5));
    assert_eq!(variables["neg"].as_number().unwrap().to_i64(), Some(-3));
    assert_eq!(
        variables["big"].as_number().unwrap().to_i64(),
        Some(12_345_678_901_234)
    );
    assert_eq!(variables["bool"].as_bool(), Some(true));
    assert!(variables["null"].is_null());
    assert_eq!(variables["arr"].as_array().unwrap().len(), 3);
    let nested = variables["nested"].as_object().unwrap();
    assert_eq!(nested.get("c"), Some(&Value::from("d".to_owned())));
    assert_eq!(variables[""], Value::from("empty key".to_owned()));
}

#[test]
fn sc28_duplicate_names_keep_the_last() {
    let config = check_config("sc28_duplicate_names");
    let pack = &config.packs[VARS];
    assert_eq!(pack.variables["a"].as_number().unwrap().to_i64(), Some(2));
    assert_eq!(pack.variables.len(), 1);
    assert_eq!(pack.secrets["s"].expose(), "two");
    assert_eq!(pack.secrets.len(), 1);
}

#[test]
fn v2_of_two_nested_objects_of_one_name_the_last_stands_whole() {
    let config = variables(r#"{"a": {"x": 1, "y": 2}, "a": {"z": 3}}"#);
    let a = config.default.variables["a"].as_object().unwrap();
    assert_eq!(a.keys().map(|k| k.as_str()).collect::<Vec<_>>(), ["z"]);
}

#[test]
fn a_number_too_large_for_a_double_is_held_to_the_largest_deviation() {
    // No finite double spells `1e999`; the value is kept as the largest one, with a
    // warning at the number.
    let text = r#"{"a": 1, "huge": [1e999, -1e999]}"#;
    let config = variables(text);
    let huge = config.default.variables["huge"].as_array().unwrap();
    assert_eq!(huge[0].as_number().unwrap().to_f64(), Some(f64::MAX));
    assert_eq!(huge[1].as_number().unwrap().to_f64(), Some(f64::MIN));
    let findings: Vec<_> = config.diagnostics().iter().collect();
    assert_eq!(findings.len(), 2);
    for (finding, path) in findings.iter().zip(["huge/0", "huge/1"]) {
        assert!(matches!(finding.code, DiagnosticCode::InvalidValue { .. }));
        assert_eq!(finding.severity, Severity::Info);
        assert_eq!(finding.location.json_path.to_string(), path);
    }
    let offset = u32::try_from(text.find("1e999").unwrap()).unwrap();
    assert_eq!(findings[0].location.span, Some(Span { offset, len: 5 }));
}

#[test]
fn an_integer_wider_than_64_bits_is_a_number() {
    let config = variables(r#"{"wide": 123456789012345678901234567890}"#);
    assert!(config.diagnostics().is_empty());
    let wide = config.default.variables["wide"].as_number().unwrap();
    assert_eq!(wide.to_f64_lossy(), 1.234_567_890_123_456_8e29);
}

#[test]
fn sc29_secret_types_keep_only_the_strings() {
    let config = check_config("sc29_secret_types");
    let secrets = &config.packs[VARS].secrets;
    assert_eq!(secrets.keys().collect::<Vec<_>>(), ["s_str"]);
    assert_eq!(secrets["s_str"].expose(), "v");
}

#[test]
fn sc30_secret_redaction_never_prints_the_value() {
    let name = "sc30_secret_redaction";
    let files = files(name);
    let config = check_config(name);
    let secret = &config.packs[VARS].secrets["token"];
    assert_eq!(secret.expose(), "hunter2");
    assert_eq!(format!("{secret:?}"), "<secret>");
    assert_eq!(format!("{secret}"), "<secret>");
    let printed = format!("{config:?} {config:#?} {files:?}");
    assert!(printed.contains("<secret>"));
    assert!(!printed.contains("hunter2"), "{printed}");
}

#[test]
fn s2_no_finding_about_a_secrets_file_quotes_its_content() {
    // A syntax error, a root that is no object, a value that is no string: none of the
    // findings carries the text of the file, in its message or through a span of the root.
    for text in [
        r#"{"token": "hunter2" "#,
        r#"["hunter2"]"#,
        r#"{"token": ["hunter2"]}"#,
        r#"{"token": hunter2}"#,
    ] {
        let mut entry = ConfigFolder::new("default");
        entry.secrets = Some(Bytes::copy_from_slice(text.as_bytes()));
        let config = ScriptConfig::parse(&ScriptConfigFiles {
            folders: vec![entry],
            ..ScriptConfigFiles::default()
        });
        assert!(!config.diagnostics().is_empty(), "{text}");
        let printed = format!("{config:?}");
        assert!(!printed.contains("hunter2"), "{text}: {printed}");
        for finding in config.diagnostics() {
            assert!(!finding.to_string().contains("hunter2"), "{finding}");
        }
    }
}

#[test]
fn s2_no_finding_about_a_secrets_file_has_a_span() {
    // A span addresses bytes of the file, and whoever prints a finding with its source would
    // print them: no finding about `secrets.json` has one, whatever it is about.
    let texts = [
        r#"{"token": "hunter2" "#,
        r#"{"k": "HUNTER2"#,
        r#"["hunter2"]"#,
        r#""hunter2""#,
        "482913",
        "null",
        r#"{"pin": 482913}"#,
        r#"{"token": ["hunter2"]}"#,
        r#"{"token": {"a": "hunter2"}}"#,
        r#"{"token": null, "b": true, "c": 1.5}"#,
        r#"{"token": hunter2}"#,
        r#"{"token": "hunter2"} hunter2"#,
        r#"{"token": "hunter2"} "hunter2""#,
        r#"{"token": "hunter2"}]"#,
        r#"{"token": 1e999}"#,
        r#"{"token": "a", "token": 5}"#,
        "\u{feff}{\"pin\": 482913} x",
        "",
        "   ",
        "{\"a\": \"hunter2\",",
    ];
    let deep = format!("{}1{}", "[".repeat(200), "]".repeat(200));
    let mut texts: Vec<&[u8]> = texts.iter().map(|text| text.as_bytes()).collect();
    texts.push(deep.as_bytes());
    texts.push(b"{\"token\": \"hunter\xff2\"}");
    let mut seen = 0;
    for text in texts {
        for folder in ["default", MODULE, "not-a-uuid"] {
            let mut entry = ConfigFolder::new(folder);
            entry.secrets = Some(Bytes::copy_from_slice(text));
            // A finding of another file of the folder keeps its span.
            entry.permissions = Some(Bytes::from_static(br#"{"allowed_modules": 5}"#));
            let config = ScriptConfig::parse(&ScriptConfigFiles {
                folders: vec![entry],
                ..ScriptConfigFiles::default()
            });
            let mut about_secrets = 0;
            for finding in config.diagnostics() {
                let file = finding.location.path.as_ref().unwrap().as_str();
                if file.ends_with("secrets.json") {
                    about_secrets += 1;
                    assert_eq!(finding.location.span, None, "{finding:?}");
                } else {
                    assert_eq!(finding.location.span, Some(Span { offset: 20, len: 1 }));
                }
            }
            assert!(about_secrets > 0, "{:?}", String::from_utf8_lossy(text));
            seen += about_secrets;
        }
    }
    assert!(seen >= 66, "{seen}");
}

#[test]
fn f3_a_syntax_fault_in_the_copy_of_a_member_that_loses_refuses_the_file() {
    // The last of two members stands (jsoncpp), and the server still reads the whole text:
    // an earlier copy that is no JSON is the file's syntax error (F3), the part empty.
    let refused = |config: &ScriptConfig, what: &str| {
        let found: Vec<(&str, Severity)> = config
            .diagnostics()
            .iter()
            .map(|f| (f.code.name(), f.severity))
            .collect();
        assert_eq!(found, [("MalformedInput", Severity::Error)], "{what}");
    };
    for text in [
        r#"{"allowed_modules": bogus, "allowed_modules": ["*"]}"#,
        r#"{"allowed_modules": ["a" "b"], "allowed_modules": ["*"]}"#,
        // In the server-net entry, whose last member stands too (N9).
        r#"{"module_permissions": {"@minecraft/server-net":
            {"force_tls": tru, "force_tls": true}}}"#,
        // A missing comma between the copies.
        r#"{"allowed_modules": ["x"] "allowed_modules": ["*"]}"#,
    ] {
        let config = permissions("default", text);
        refused(&config, text);
        assert!(
            config.default.permissions.allowed_modules.is_empty(),
            "{text}"
        );
        assert!(
            config.default.permissions.module_permissions.is_empty(),
            "{text}"
        );
    }
    let config = variables(r#"{"v": [1 2], "v": 3}"#);
    refused(&config, "variables");
    assert!(config.default.variables.is_empty());
    let mut folder = ConfigFolder::new("default");
    folder.secrets = Some(Bytes::from_static(br#"{"s": "a" x, "s": "b"}"#));
    let config = ScriptConfig::parse(&ScriptConfigFiles {
        folders: vec![folder],
        ..ScriptConfigFiles::default()
    });
    refused(&config, "secrets");
    assert!(config.default.secrets.is_empty());
    // A well-formed copy that loses still loses in silence.
    let config = permissions(
        "default",
        r#"{"allowed_modules": {"x": [1]}, "allowed_modules": ["*"]}"#,
    );
    assert!(config.diagnostics().is_empty());
    assert_eq!(config.default.permissions.allowed_modules, ["*"]);
}
