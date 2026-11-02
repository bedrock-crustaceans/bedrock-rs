//! `config/commands.json` (fixtures sc33–sc36, sc48; validation C1–C4, C6).

use bedrock_addon::diagnostic::{DiagnosticCode, Severity, Span};
use bedrock_addon::families::script::config::{
    CommandPermissionLevel, CommandsConfig, ScriptConfig, ScriptConfigFiles, UnreadableFile,
};
use bedrock_addon::json::Value;
use bytes::Bytes;

use crate::common::{check_config, seen};

fn commands(text: &str) -> ScriptConfig {
    ScriptConfig::parse(&ScriptConfigFiles {
        commands: Some(Bytes::copy_from_slice(text.as_bytes())),
        ..ScriptConfigFiles::default()
    })
}

/// (code, severity, path) of each finding.
fn findings(config: &ScriptConfig) -> Vec<(String, String, String)> {
    seen(config.diagnostics())
        .into_iter()
        .map(|f| (f.code, f.severity, f.path))
        .collect()
}

fn one(code: &str, severity: &str, path: &str) -> Vec<(String, String, String)> {
    vec![(code.to_owned(), severity.to_owned(), path.to_owned())]
}

#[test]
fn an_absent_file_is_no_configuration_and_no_finding() {
    let config = ScriptConfig::parse(&ScriptConfigFiles::default());
    assert_eq!(config.commands, None);
    assert!(config.diagnostics().is_empty());
}

#[test]
fn sc33_commands_json_valid_keeps_the_unknown_member() {
    let config = check_config("sc33_commands_json_valid");
    let commands = config.commands.as_ref().unwrap();
    assert_eq!(commands.default_permission_level, None);
    assert_eq!(
        commands.permission_levels["list"],
        CommandPermissionLevel::Internal
    );
    assert_eq!(
        commands.extra.get("unknown_member"),
        Some(&Value::from(1_i64))
    );
}

#[test]
fn every_level_name_loads_as_the_default_level() {
    for level in CommandPermissionLevel::ALL {
        let text = format!(r#"{{"default_permission_level": "{}"}}"#, level.name());
        let config = commands(&text);
        assert!(config.diagnostics().is_empty());
        assert_eq!(
            config.commands,
            Some(CommandsConfig {
                default_permission_level: Some(level),
                ..CommandsConfig::default()
            })
        );
    }
}

#[test]
fn sc34_commands_json_bad_enum_refuses_the_whole_file() {
    let config = check_config("sc34_commands_json_bad_enum");
    assert_eq!(config.commands, None);
    assert_eq!(config.diagnostics().max_severity(), Some(Severity::Error));
}

#[test]
fn c2_a_bad_level_of_a_command_refuses_the_whole_file() {
    let text = r#"{"default_permission_level": "admin", "permission_levels": {"list": "internal", "say": "Admin", "op": 3}}"#;
    let config = commands(text);
    assert_eq!(config.commands, None);
    assert_eq!(
        findings(&config),
        [
            (
                "InvalidEnumValue".to_owned(),
                "error".to_owned(),
                "permission_levels/say".to_owned()
            ),
            (
                "InvalidType".to_owned(),
                "error".to_owned(),
                "permission_levels/op".to_owned()
            )
        ]
    );
    let offset = u32::try_from(text.find("\"Admin\"").unwrap()).unwrap();
    assert_eq!(
        config.diagnostics().iter().next().unwrap().location.span,
        Some(Span { offset, len: 7 })
    );
}

#[test]
fn c2_members_of_the_wrong_type_refuse_the_file() {
    for (text, path) in [
        (
            r#"{"default_permission_level": 1}"#,
            "default_permission_level",
        ),
        (r#"{"permission_levels": ["list"]}"#, "permission_levels"),
        (r#"{"permission_levels": []}"#, "permission_levels"),
        (r#"{"permission_levels": "internal"}"#, "permission_levels"),
        (r#"["list"]"#, ""),
        ("null", ""),
    ] {
        let config = commands(text);
        assert_eq!(config.commands, None, "{text}");
        assert_eq!(
            findings(&config),
            one("InvalidType", "error", path),
            "{text}"
        );
    }
}

#[test]
fn sc35_commands_json_syntax_is_refused() {
    let config = check_config("sc35_commands_json_syntax");
    assert_eq!(config.commands, None);
}

#[test]
fn an_empty_commands_json_is_refused() {
    let config = commands("");
    assert_eq!(config.commands, None);
    let finding = config.diagnostics().first_error().unwrap();
    assert!(matches!(finding.code, DiagnosticCode::EmptyDocument { .. }));
    assert_eq!(
        finding.code.to_string(),
        "Failed to parse JSON in file 'config/commands.json'\n:  malformed input"
    );
}

#[test]
fn an_unreadable_commands_json_is_a_read_error() {
    let config = ScriptConfig::parse(&ScriptConfigFiles {
        unreadable: vec![UnreadableFile {
            folder: None,
            file: "commands.json".into(),
            kind: std::io::ErrorKind::PermissionDenied,
        }],
        ..ScriptConfigFiles::default()
    });
    assert_eq!(config.commands, None);
    let finding = config.diagnostics().first_error().unwrap();
    assert!(matches!(finding.code, DiagnosticCode::ReadError { .. }));
    assert_eq!(
        finding.code.to_string(),
        "Failed to open file 'config/commands.json'"
    );
}

#[test]
fn sc36_commands_json_in_default_is_not_read() {
    let config = check_config("sc36_commands_json_in_default");
    assert_eq!(config.commands, None);
}

#[test]
fn sc48_commands_json_duplicate_keeps_the_first() {
    // The server keeps the first of two members here (`list` still runs), unlike the
    // per-folder files, where the last stands.
    let config = check_config("sc48_commands_json_duplicate");
    let commands = config.commands.as_ref().unwrap();
    assert_eq!(
        commands.permission_levels["list"],
        CommandPermissionLevel::Any
    );
    assert_eq!(commands.permission_levels.len(), 1);
}

#[test]
fn c6_the_first_of_two_members_stands_at_every_level() {
    let text = r#"{
  "default_permission_level": "host",
  "permission_levels": {"list": "any", "say": "admin", "list": "internal"},
  "default_permission_level": "owner", // a comment, with a , in it
  "note": {"a": 1, "a": 2},
  "permission_levels": {"list": "bogus"}
}"#;
    let config = commands(text);
    let commands = config.commands.as_ref().unwrap();
    assert_eq!(
        commands.default_permission_level,
        Some(CommandPermissionLevel::Host)
    );
    assert_eq!(
        commands
            .permission_levels
            .iter()
            .map(|(name, level)| (name.as_str(), *level))
            .collect::<Vec<_>>(),
        [
            ("list", CommandPermissionLevel::Any),
            ("say", CommandPermissionLevel::Admin)
        ]
    );
    let note = commands.extra.get("note").unwrap().as_object().unwrap();
    assert_eq!(note.get("a"), Some(&Value::from(1_i64)));
    // The later `permission_levels`, with its bad level, is never read.
    assert_eq!(findings(&config), one("UnrecognizedField", "info", "note"));
}

#[test]
fn c6_keys_compare_by_their_value_and_strings_are_not_keys() {
    // The second key is written with an escape (`l`, backslash, `u0069`, `st`) and spells
    // `list`; a value that looks like a key is no key.
    let escaped = |text: &str| text.replace("U+", "\\u");
    assert!(escaped("lU+0069st").starts_with("l\\u0069"));
    let config = commands(&escaped(
        r#"{"permission_levels": {"list": "any", "lU+0069st": "internal", "say": "list"}}"#,
    ));
    assert_eq!(config.commands, None);
    assert_eq!(
        findings(&config),
        one("InvalidEnumValue", "error", "permission_levels/say")
    );
    let config = commands(&escaped(
        r#"{"permission_levels": {"list": "any", "lU+0069st": "internal"}}"#,
    ));
    let commands = config.commands.as_ref().unwrap();
    assert_eq!(
        commands.permission_levels["list"],
        CommandPermissionLevel::Any
    );
}

#[test]
fn a_span_after_a_repeated_member_is_where_the_value_is() {
    let text = r#"{"permission_levels": {"list": "any"}, "permission_levels": {"x": "y"}, "default_permission_level": "bogus"}"#;
    let config = commands(text);
    let finding = config.diagnostics().first_error().unwrap();
    assert!(matches!(
        finding.code,
        DiagnosticCode::InvalidEnumValue { .. }
    ));
    let offset = u32::try_from(text.find("\"bogus\"").unwrap()).unwrap();
    assert_eq!(finding.location.span, Some(Span { offset, len: 7 }));
}

#[test]
fn c6_the_first_stands_also_when_content_follows_the_root() {
    // Whatever follows the root is ignored with a warning; it must not change which of two
    // members stands.
    let root = r#"{"permission_levels":{"list":"any"},"permission_levels":{"list":"internal"}}"#;
    for tail in [
        " ]", " }", " {", " \"", " /", " xyz", " [", " ,", " \"a\"", "\n// c", " /* c",
    ] {
        let config = commands(&format!("{root}{tail}"));
        let commands = config
            .commands
            .as_ref()
            .unwrap_or_else(|| panic!("tail {tail:?}: the file loads"));
        assert_eq!(
            commands.permission_levels["list"],
            CommandPermissionLevel::Any,
            "tail {tail:?}: the first of the two members stands"
        );
        assert_eq!(commands.permission_levels.len(), 1, "tail {tail:?}");
        // A whole comment is no content; an unclosed one is.
        let expected = if tail != "\n// c" {
            one("MalformedInput", "info", "")
        } else {
            Vec::new()
        };
        assert_eq!(findings(&config), expected, "tail {tail:?}");
    }
}

#[test]
fn c5_a_permission_level_for_a_command_the_game_does_not_have_is_a_note() {
    use bedrock_addon::diagnostic::{DiagnosticDetail, Phase};
    // 20 S10: the keys of `permission_levels` are command names, as typed: a name or an
    // alias (`tp` is `teleport`'s), compared byte for byte.
    let config = commands(
        r#"{"permission_levels": {"give": "admin", "tp": "host", "nosuchcommand": "any",
            "Give": "owner"}}"#,
    );
    assert_eq!(
        findings(&config),
        [
            (
                "UnresolvedReference".to_owned(),
                "info".to_owned(),
                "permission_levels/nosuchcommand".to_owned()
            ),
            (
                "UnresolvedReference".to_owned(),
                "info".to_owned(),
                "permission_levels/Give".to_owned()
            ),
        ]
    );
    let details: Vec<Option<DiagnosticDetail>> = config
        .diagnostics()
        .iter()
        .map(|finding| {
            // The table is the crate's own data: the note is told when the file is read,
            // not by a link pass.
            assert_eq!(finding.phase, Phase::Validate);
            assert_eq!(
                finding.location.path.as_ref().map(|path| path.as_str()),
                Some("config/commands.json")
            );
            finding.detail.as_deref().cloned()
        })
        .collect();
    let named = |id: &str| {
        Some(DiagnosticDetail::Reference {
            kind: "command",
            id: id.to_owned(),
            hint: None,
        })
    };
    assert_eq!(details, [named("nosuchcommand"), named("Give")]);
    // The file loads whole: the server keeps a level for a name it has no command for.
    let levels = &config
        .commands
        .as_ref()
        .expect("the file")
        .permission_levels;
    assert_eq!(
        levels.keys().map(String::as_str).collect::<Vec<_>>(),
        ["give", "tp", "nosuchcommand", "Give"]
    );
    assert_eq!(levels["nosuchcommand"], CommandPermissionLevel::Any);
}

#[test]
fn c5_a_refused_commands_json_tells_no_command_note() {
    // An invalid level refuses the whole file (C2): nothing of it is kept, and nothing of it
    // is looked up.
    let config = commands(r#"{"permission_levels": {"nosuchcommand": "bogus", "other": "any"}}"#);
    assert_eq!(config.commands, None);
    assert_eq!(
        findings(&config),
        one(
            "InvalidEnumValue",
            "error",
            "permission_levels/nosuchcommand"
        )
    );
}

#[test]
fn c1_a_syntax_fault_in_the_copy_of_a_member_that_loses_refuses_the_file() {
    // The first of two members stands (C6), and the server's reader still reads the whole
    // text: a later copy that is no JSON is the file's syntax error (C1), not a member that
    // silently lost.
    for text in [
        r#"{"default_permission_level": "any", "default_permission_level": bogus}"#,
        r#"{"permission_levels": {"list": "any"}, "permission_levels": {"list": tru}}"#,
        r#"{"permission_levels": {"list": "any", "list": [1 2]}}"#,
        // A missing comma after the copy that loses.
        r#"{"default_permission_level": "any", "default_permission_level": "host" "x": 1}"#,
    ] {
        let config = commands(text);
        assert_eq!(config.commands, None, "{text}");
        let found = findings(&config);
        assert_eq!(found.len(), 1, "{text}: {found:?}");
        assert_eq!(
            (found[0].0.as_str(), found[0].1.as_str()),
            ("MalformedInput", "error"),
            "{text}"
        );
        assert_eq!(
            config.diagnostics().iter().next().unwrap().code.to_string(),
            "Failed to parse JSON in file 'config/commands.json'\n:  malformed input",
            "{text}"
        );
    }
    // A well-formed copy that loses still loses in silence.
    let config = commands(
        r#"{"default_permission_level": "any", "default_permission_level": {"x": [1, 2]}}"#,
    );
    assert!(config.diagnostics().is_empty());
    assert_eq!(
        config.commands.unwrap().default_permission_level,
        Some(CommandPermissionLevel::Any)
    );
}
