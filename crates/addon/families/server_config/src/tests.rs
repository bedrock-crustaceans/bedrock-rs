//! Unit tests of the crate-private parts, and the cost of every reader: linear in its input
//! (`bedrock_addon_core::test_cost`: the run on four times the input takes under eight times as long).

use std::borrow::Cow;
use std::fmt::Write as _;

use bytes::Bytes;
use uuid::Uuid;

use bedrock_addon_core::json::first_wins;
use indexmap::IndexMap;

use super::{
    ConfigFolder, NetModuleConfig, PackPermissions, SERVER_NET_MODULE, ScriptConfig,
    ScriptConfigFiles, ServerNet, ServerProperties, UnreadableFile,
};
use bedrock_addon_core::diagnostic::{Diagnostics, Kind};
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_core::test_cost::{assert_each_linear_at, assert_linear_at};

// ---- first_wins ----

#[test]
fn first_wins_blanks_the_later_member_and_keeps_every_offset() {
    let text = "{\"a\": 1, \"b\": 2,\n \"a\": [3, {\"x\": 1}], \"c\": 4}";
    let clean = first_wins(text);
    assert_eq!(
        clean,
        format!("{{\"a\": 1, \"b\": 2 \n{}, \"c\": 4}}", " ".repeat(19))
    );
    assert_eq!(clean.len(), text.len());
    assert_eq!(clean.find("\"c\""), text.find("\"c\""));
    // The last member of an object goes with the comma before it.
    assert_eq!(first_wins(r#"{"a": 1, "a": 2}"#), r#"{"a": 1        }"#);
    // Three of a kind: the first stands.
    assert_eq!(
        first_wins(r#"{"a": 1, "a": 2, "a": 3}"#),
        r#"{"a": 1                }"#
    );
}

#[test]
fn first_wins_works_at_every_level_and_only_on_keys() {
    assert_eq!(
        first_wins(r#"{"o": {"k": 1, "k": 2}, "l": [{"k": 1, "k": 2}]}"#),
        r#"{"o": {"k": 1        }, "l": [{"k": 1        }]}"#
    );
    // Equal strings that are values, elements, or keys of different objects are left.
    for text in [
        r#"{"a": "a", "b": "a"}"#,
        r#"["a", "a"]"#,
        r#"{"a": {"a": 1}, "b": {"a": 2}}"#,
        r#"[{"a": 1}, {"a": 2}]"#,
    ] {
        assert!(matches!(first_wins(text), Cow::Borrowed(_)), "{text}");
    }
    // Keys compare by their value; a quote or a brace inside a string is text.
    // (The second key is written with an escape: backslash, `u0061`.)
    let text = r#"{"a\"}": 1, "U+0061\"}": 2, "x": "}"}"#.replace("U+", "\\u");
    assert_eq!(text.matches("\\u0061").count(), 1);
    assert_eq!(
        first_wins(&text),
        format!(r#"{{"a\"}}": 1{}, "x": "}}"}}"#, " ".repeat(16))
    );
}

#[test]
fn first_wins_reads_past_comments_and_leaves_what_is_not_json() {
    let text = "{\"a\": 1, // \"a\": 9,\n \"a\": 2 /* } */, \"b\": 3}";
    // The blank runs from the comma before the repeated member to the one after it: the
    // comments between them go with it, the line break stays.
    assert_eq!(
        first_wins(text),
        format!(
            "{{\"a\": 1{}\n{}, \"b\": 3}}",
            " ".repeat(12),
            " ".repeat(15)
        )
    );
    for broken in [
        "{\"a\": 1, \"a\": 2",
        "{\"a\": 1, \"a\": 2]",
        "{\"a",
        "/ {}",
        "{\"a\": 1} /*",
    ] {
        assert!(matches!(first_wins(broken), Cow::Borrowed(_)), "{broken}");
    }
}

#[test]
fn the_known_keys_of_server_properties_are_sorted_for_the_search() {
    // `KNOWN_KEYS` is searched by bisection: every key the shipped file sets must be found.
    let text = "transport=raknet\nview-distance=32\nallow-cheats=false\ntick-distance=4\n\
                server-name=x\nenable-script=true\nemit-server-telemetry=true\n";
    let properties = ServerProperties::parse(text.as_bytes());
    assert!(
        properties.diagnostics().is_empty(),
        "{:?}",
        properties.diagnostics()
    );
    assert_eq!(properties.entries.len(), 7);
}

// ---- the stack ----

#[test]
#[allow(
    clippy::disallowed_methods,
    reason = "the test needs a thread of a known stack size"
)]
fn values_nested_to_the_depth_limit_load_write_and_drop_on_a_small_stack() {
    // A megabyte of stack is what a world load has (crates/addon/std/tests/small_stack.rs),
    // and `MAX_JSON_DEPTH` (128) is what a file may nest: every reader of this module takes a
    // file at the limit there, and every writer writes it back.
    let run = || {
        // A value `above` levels below its file's root, nested so that the file has 128.
        let deep = |above: usize| {
            let arrays = bedrock_addon_core::json::MAX_JSON_DEPTH - above;
            format!("{}1{}", "[".repeat(arrays), "]".repeat(arrays))
        };
        let member = deep(1);
        let mut folder = ConfigFolder::new("default");
        folder.variables = Some(Bytes::from(format!("{{\"v\": {member}}}")));
        // A secret that is no string is skipped with its warning, however deep.
        folder.secrets = Some(Bytes::from(format!("{{\"s\": {member}}}")));
        folder.permissions = Some(Bytes::from(format!(
            "{{\"module_permissions\": {{\"@acme/other\": {{\"v\": {}}}, \
             \"@minecraft/server-net\": {{\"allowed_uris\": {}}}}}, \"note\": {member}}}",
            deep(3),
            deep(3),
        )));
        let files = ScriptConfigFiles {
            folders: vec![folder],
            commands: Some(Bytes::from(format!("{{\"note\": {member}}}"))),
            ..ScriptConfigFiles::default()
        };
        let config = ScriptConfig::parse(&files);
        let codes: Vec<&str> = config.diagnostics().iter().map(|f| f.code.id()).collect();
        assert!(
            config
                .diagnostics()
                .iter()
                .all(|f| f.kind() != Kind::Syntax),
            "{codes:?}"
        );
        assert_eq!(config.default.variables.len(), 1);
        assert!(config.default.secrets.is_empty());
        assert_eq!(config.default.permissions.module_permissions.len(), 2);
        assert_eq!(config.default.permissions.extra.iter().count(), 1);
        assert_eq!(config.commands.as_ref().unwrap().extra.iter().count(), 1);
        // Each file is written back and reads again to the same.
        let again = ScriptConfig::parse(&ScriptConfigFiles {
            folders: vec![{
                let mut folder = ConfigFolder::new("default");
                folder.variables = Some(Bytes::from(config.default.variables_json().unwrap()));
                folder.permissions =
                    Some(Bytes::from(config.default.permissions.to_json().unwrap()));
                folder
            }],
            commands: Some(Bytes::from(
                config.commands.as_ref().unwrap().to_json().unwrap(),
            )),
            ..ScriptConfigFiles::default()
        });
        assert_eq!(again.default.variables, config.default.variables);
        assert_eq!(again.default.permissions, config.default.permissions);
        assert_eq!(again.commands, config.commands);
        drop(config.clone());
        // One level more is refused.
        let mut folder = ConfigFolder::new("default");
        folder.variables = Some(Bytes::from(format!("{{\"v\": [{member}]}}")));
        let refused = ScriptConfig::parse(&ScriptConfigFiles {
            folders: vec![folder],
            ..ScriptConfigFiles::default()
        });
        assert!(refused.default.variables.is_empty());
        assert_eq!(
            refused.diagnostics().first_error().map(|f| f.code.id()),
            Some("server_config/config_parse_failed_malformed_input")
        );
    };
    std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap();
}

/// The too-deep message counts its byte from after a byte-order mark, as the reader did
/// when it stripped the mark: the mark blanked for the spans does not move the number.
#[test]
fn the_too_deep_byte_of_a_file_with_a_bom_counts_from_after_the_mark() {
    let depth = bedrock_addon_core::json::MAX_JSON_DEPTH;
    for (bom, name) in [(&b""[..], "plain"), (&b"\xEF\xBB\xBF"[..], "bom")] {
        let mut bytes = bom.to_vec();
        bytes.extend("[".repeat(depth + 1).bytes());
        let mut folder = ConfigFolder::new("default");
        folder.variables = Some(Bytes::from(bytes));
        let config = ScriptConfig::parse(&ScriptConfigFiles {
            folders: vec![folder],
            ..ScriptConfigFiles::default()
        });
        let message = config.diagnostics().first_error().map(|f| {
            f.syntax_message()
                .unwrap_or_else(|| panic!("{name}: {:?}", f.code))
                .to_owned()
        });
        assert_eq!(
            message,
            Some(format!(
                "JSON nesting deeper than the limit at byte {depth}"
            )),
            "{name}"
        );
    }
}

/// Permissions whose held server-net entry has `allowed_uris` nested `arrays` arrays deep,
/// with `server_net` as given: the entry nests `arrays` + 1 levels, and a `permissions.json`
/// that holds it two more (the root and `module_permissions`).
fn permissions_with_a_deep_net_entry(arrays: usize, server_net: ServerNet) -> PackPermissions {
    let mut value = Value::from(1_i64);
    for _ in 0..arrays {
        let mut array = facet_value::VArray::new();
        array.push(value);
        value = Value::from(array);
    }
    let mut permissions = PackPermissions {
        server_net,
        ..PackPermissions::default()
    };
    permissions.module_permissions.insert(
        SERVER_NET_MODULE.to_owned(),
        IndexMap::from([("allowed_uris".to_owned(), value)]),
    );
    permissions
}

#[test]
#[allow(
    clippy::disallowed_methods,
    reason = "the test needs a thread of a known stack size"
)]
fn a_held_server_net_entry_is_written_as_held_only_while_its_file_stays_within_the_depth_limit() {
    let refused_at = |result: Result<String, EncodeError>| match result {
        Err(EncodeError::NotRepresentable { at, .. }) => at.to_string(),
        other => panic!("{other:?}"),
    };
    let run = move || {
        // 128 levels in the file: the entry is one the reader refuses (denied), and it is
        // written as held; the file reads back.
        let at_the_limit = permissions_with_a_deep_net_entry(125, ServerNet::Denied);
        let text = at_the_limit.to_json().unwrap();
        let mut folder = ConfigFolder::new("default");
        folder.permissions = Some(Bytes::from(text));
        let again = ScriptConfig::parse(&ScriptConfigFiles {
            folders: vec![folder],
            ..ScriptConfigFiles::default()
        });
        assert_eq!(again.default.permissions, at_the_limit);
        let codes: Vec<&str> = again.diagnostics().iter().map(|f| f.code.id()).collect();
        assert_eq!(codes, ["core/invalid_type", "core/net_config_rejected"]);
        // 129 levels: the file would be refused whole, so the entry is not read as what
        // denies, and a denied server-net without such an entry has no JSON form.
        let past_it = permissions_with_a_deep_net_entry(126, ServerNet::Denied);
        assert_eq!(
            refused_at(past_it.to_json()),
            "module_permissions/@minecraft~1server-net"
        );
        // With a configuration beside it, the configuration is what is written.
        let edited = permissions_with_a_deep_net_entry(
            126,
            ServerNet::Configured(NetModuleConfig::default()),
        );
        assert_eq!(
            edited.to_json().unwrap().replace([' ', '\n'], ""),
            r#"{"module_permissions":{"@minecraft/server-net":{}}}"#
        );
    };
    std::thread::Builder::new()
        .stack_size(bedrock_addon_core::load::RECOMMENDED_JOB_STACK)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap();
}

/// `1` inside `arrays` arrays.
fn nested(arrays: usize) -> Value {
    let mut value = Value::from(1_i64);
    for _ in 0..arrays {
        let mut array = facet_value::VArray::new();
        array.push(value);
        value = Value::from(array);
    }
    value
}

#[test]
#[allow(
    clippy::disallowed_methods,
    reason = "the test needs a thread of a known stack size"
)]
fn no_writer_writes_a_file_the_reader_refuses_for_its_depth() {
    use super::CommandsConfig;
    // The refusal names the place that is too deep: the held value's member, then the
    // indices down to the level past the limit.
    let refused_below = |result: Result<String, EncodeError>, member: &str| match result {
        Err(EncodeError::NotRepresentable {
            at,
            what: bedrock_addon_core::json::Unrepresentable::TooDeep,
        }) => {
            let text = at.to_string();
            assert!(
                text.starts_with(&format!("{member}/")),
                "{member}: {}",
                &text[..40]
            );
        }
        Err(other) => panic!("{other:?}"),
        Ok(text) => panic!("written, {} bytes", text.len()),
    };
    let reads = |permissions: Option<String>, variables: Option<String>| {
        let config = parse(permissions, variables, None);
        let codes: Vec<&str> = config.diagnostics().iter().map(|f| f.code.id()).collect();
        assert!(
            config
                .diagnostics()
                .iter()
                .all(|f| f.kind() != Kind::Syntax),
            "{codes:?}"
        );
        config
    };
    let run = move || {
        // Another module's settings: the member's value lies three levels down (the root,
        // `module_permissions`, the module's object), so 125 arrays are a file of 128
        // levels and 126 one the reader refuses whole.
        let other = |arrays: usize| {
            let mut permissions = PackPermissions::default();
            permissions.module_permissions.insert(
                "@minecraft/other".to_owned(),
                IndexMap::from([("setting".to_owned(), nested(arrays))]),
            );
            permissions
        };
        let at_the_limit = other(125);
        let again = reads(Some(at_the_limit.to_json().unwrap()), None);
        assert_eq!(again.default.permissions, at_the_limit);
        refused_below(
            other(126).to_json(),
            "module_permissions/@minecraft~1other/setting",
        );
        // A configuration beside it is not what gets lost in a file that does not load.
        let mut beside = other(126);
        beside.server_net = ServerNet::Configured(NetModuleConfig::default());
        refused_below(
            beside.to_json(),
            "module_permissions/@minecraft~1other/setting",
        );
        // An unknown member: one level down.
        let unknown = |arrays: usize| {
            let mut permissions = PackPermissions::default();
            permissions.extra.insert("note".to_owned(), nested(arrays));
            permissions
        };
        let again = reads(Some(unknown(127).to_json().unwrap()), None);
        assert_eq!(again.default.permissions, unknown(127));
        refused_below(unknown(128).to_json(), "note");
        // A variable: one level down.
        let variable = |arrays: usize| {
            let mut config = super::PackScriptConfig::default();
            config.variables.insert("deep".to_owned(), nested(arrays));
            config
        };
        let again = reads(None, Some(variable(127).variables_json().unwrap()));
        assert_eq!(again.default.variables, variable(127).variables);
        refused_below(variable(128).variables_json(), "deep");
        // `commands.json`: an unknown member.
        let commands = |arrays: usize| {
            let mut commands = CommandsConfig::default();
            commands.extra.insert("note".to_owned(), nested(arrays));
            commands
        };
        let again = ScriptConfig::parse(&ScriptConfigFiles {
            commands: Some(Bytes::from(commands(127).to_json().unwrap())),
            ..ScriptConfigFiles::default()
        });
        assert_eq!(again.commands, Some(commands(127)));
        refused_below(commands(128).to_json(), "note");
    };
    std::thread::Builder::new()
        .stack_size(bedrock_addon_core::load::RECOMMENDED_JOB_STACK)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap();
}

// ---- cost ----

fn uuid(index: usize) -> Uuid {
    Uuid::from_u128(0xa000_0000_0000_4000_8000_0000_0000_0000 + index as u128)
}

fn parse(
    permissions: Option<String>,
    variables: Option<String>,
    secrets: Option<String>,
) -> ScriptConfig {
    let mut folder = ConfigFolder::new("default");
    folder.permissions = permissions.map(Bytes::from);
    folder.variables = variables.map(Bytes::from);
    folder.secrets = secrets.map(Bytes::from);
    ScriptConfig::parse(&ScriptConfigFiles {
        folders: vec![folder],
        ..ScriptConfigFiles::default()
    })
}

#[test]
fn parsing_a_config_of_many_folders_is_linear() {
    let input = |n: usize| {
        let mut files = ScriptConfigFiles::default();
        for index in 0..n {
            // In reverse, so the sort has work; every folder lacks what `default` has, has a
            // finding of its own and a file that could not be read.
            let name = uuid(n - index).to_string();
            let mut folder = ConfigFolder::new(name.as_str());
            folder.permissions = Some(Bytes::from_static(b"{\"allowed_modules\": 5}"));
            folder.stray_commands = true;
            files.unreadable.push(UnreadableFile {
                folder: Some(name.into()),
                file: "secrets.json".into(),
                kind: std::io::ErrorKind::PermissionDenied,
            });
            files.folders.push(folder);
        }
        let mut default = ConfigFolder::new("default");
        default.permissions = Some(Bytes::from_static(b"{\"allowed_modules\": [\"*\"]}"));
        default.variables = Some(Bytes::from_static(b"{\"v\": 1}"));
        files.folders.push(default);
        files
    };
    assert_linear_at("ScriptConfig::parse, n folders", 2_000, input, |files| {
        let config = ScriptConfig::parse(files);
        assert_eq!(config.packs.len(), files.folders.len() - 1);
        // InvalidType, ReadError, IgnoredFile and one PackConfigShadowsDefault per folder.
        assert_eq!(config.diagnostics().len(), 4 * config.packs.len());
    });
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "one input and one step per file kind"
)]
fn parsing_long_files_is_linear() {
    struct Texts {
        n: usize,
        modules: String,
        module_permissions: String,
        net: String,
        variables: String,
        secrets: String,
        commands: String,
    }
    let input = |n: usize| {
        let mut texts = Texts {
            n,
            modules: "{\"allowed_modules\": [".to_owned(),
            module_permissions: "{\"module_permissions\": {".to_owned(),
            net: "{\"module_permissions\": {\"@minecraft/server-net\": {\"allowed_uris\": ["
                .to_owned(),
            variables: "{".to_owned(),
            secrets: "{".to_owned(),
            commands: "{\"permission_levels\": {".to_owned(),
        };
        for index in 0..n {
            let comma = if index == 0 { "" } else { ", " };
            // A near miss, a non-string and a wildcard among the names: a finding per entry.
            write!(
                texts.modules,
                "{comma}\"@minecraft/server@{index}.0.0\", {index}, \"*\""
            )
            .unwrap();
            write!(
                texts.module_permissions,
                "{comma}\"@acme/m{index}\": {{\"k\": {index}}}, \"@acme/n{index}\": 5"
            )
            .unwrap();
            write!(texts.net, "{comma}\"http://h{index}.example/*\"").unwrap();
            // Every name twice: the last stands.
            write!(
                texts.variables,
                "{comma}\"v{index}\": [1e999], \"v{}\": {{\"a\": {index}}}",
                index / 2
            )
            .unwrap();
            write!(
                texts.secrets,
                "{comma}\"s{}\": {index}, \"t{index}\": \"x\"",
                n - index
            )
            .unwrap();
            // Every command twice: the first stands.
            write!(
                texts.commands,
                "{comma}\"c{index}\": \"any\", \"c{}\": \"internal\"",
                index / 2
            )
            .unwrap();
        }
        texts.modules.push_str("]}");
        texts.module_permissions.push_str("}}");
        texts.net.push_str("], \"session_headers\": {");
        for index in 0..n {
            let comma = if index == 0 { "" } else { ", " };
            write!(texts.net, "{comma}\"x-{index}\": \"v\"").unwrap();
        }
        texts.net.push_str("}}}}");
        texts.variables.push('}');
        texts.secrets.push('}');
        texts.commands.push_str("}}");
        texts
    };
    assert_each_linear_at(
        5_000,
        input,
        &[
            (
                "permissions.json, n allowed modules with findings",
                &|texts: &Texts| {
                    let config = parse(Some(texts.modules.clone()), None, None);
                    assert_eq!(
                        config.default.permissions.allowed_modules.len(),
                        2 * texts.n
                    );
                    assert!(config.diagnostics().len() > 2 * texts.n);
                },
            ),
            (
                "permissions.json, n module_permissions entries",
                &|texts: &Texts| {
                    let config = parse(Some(texts.module_permissions.clone()), None, None);
                    assert_eq!(config.default.permissions.module_permissions.len(), texts.n);
                    assert_eq!(config.diagnostics().len(), 2 * texts.n);
                },
            ),
            (
                "permissions.json, a server-net entry of n URIs and n headers",
                &|texts: &Texts| {
                    let config = parse(Some(texts.net.clone()), None, None);
                    assert_eq!(config.diagnostics().len(), texts.n);
                    let super::ServerNet::Configured(net) = &config.default.permissions.server_net
                    else {
                        panic!("the entry loads");
                    };
                    assert_eq!(net.session_headers.as_ref().unwrap().len(), texts.n);
                },
            ),
            ("variables.json, n names written twice", &|texts: &Texts| {
                let config = parse(None, Some(texts.variables.clone()), None);
                assert_eq!(config.default.variables.len(), texts.n);
            }),
            (
                "secrets.json, n secrets and n values that are no strings",
                &|texts: &Texts| {
                    let config = parse(None, None, Some(texts.secrets.clone()));
                    assert_eq!(config.default.secrets.len(), texts.n);
                    assert_eq!(config.diagnostics().len(), texts.n);
                },
            ),
            (
                "commands.json, n commands written twice",
                &|texts: &Texts| {
                    let config = ScriptConfig::parse(&ScriptConfigFiles {
                        commands: Some(Bytes::from(texts.commands.clone())),
                        ..ScriptConfigFiles::default()
                    });
                    assert_eq!(
                        config.commands.as_ref().unwrap().permission_levels.len(),
                        texts.n
                    );
                },
            ),
            (
                "permissions.json read and PackPermissions::to_json, n entries",
                &|texts: &Texts| {
                    let config = parse(Some(texts.module_permissions.clone()), None, None);
                    assert!(config.default.permissions.to_json().unwrap().len() > texts.n);
                },
            ),
        ],
    );
}

#[test]
fn first_wins_is_linear() {
    let input = |n: usize| {
        // Every key twice, at two levels, with escaped keys among them.
        let mut text = String::from("{\"o\": {");
        for index in 0..n {
            let comma = if index == 0 { "" } else { ", " };
            write!(
                text,
                "{comma}\"k{index}\": [1, {{\"a\": 1, \"a\": 2}}], \"k{}\": \"k{index}\", \"e\\\\{index}\": 0",
                index / 2
            )
            .unwrap();
        }
        text.push_str("}}");
        text
    };
    assert_linear_at(
        "first_wins, 3 n members, n written twice",
        40_000,
        input,
        |text| {
            let clean = first_wins(text);
            assert!(matches!(clean, Cow::Owned(_)));
            assert_eq!(clean.len(), text.len());
        },
    );
}

/// The plugin of the pack `name` (header `header`) whose script module `module` depends on
/// `dependencies` (`(module name, optional)`), as discovery leaves one.
fn plugin(
    name: &str,
    header: Uuid,
    module: Uuid,
    dependencies: Vec<(String, bool)>,
) -> bedrock_addon_family_script::ScriptPlugin {
    use bedrock_addon_family_script::{ResolvedDependency, ScriptPlugin, ScriptVersion};
    // `ScriptPlugin::new` sets the runtime, the entry, the main file and the flags this
    // plugin has; the name and the dependencies are set by hand.
    let mut plugin = ScriptPlugin::new(name, header, module, ScriptVersion::new(1, 0, 0));
    plugin.name = format!("{name} - 1.0.0");
    plugin.dependencies = dependencies
        .into_iter()
        .map(|(module_name, optional)| {
            let mut dependency = ResolvedDependency::new(module_name, ScriptVersion::new(1, 0, 0));
            dependency.optional = optional;
            dependency
        })
        .collect();
    plugin
}

#[test]
#[allow(clippy::too_many_lines, reason = "one stack and one step per check")]
fn the_stack_checks_are_linear() {
    use bedrock_addon_family_script::ScriptPlugin;
    struct Stack {
        config: ScriptConfig,
        plugins: Vec<ScriptPlugin>,
        greedy: ScriptPlugin,
    }
    let input = |n: usize| {
        let mut files = ScriptConfigFiles::default();
        let mut plugins = Vec::with_capacity(n);
        let mut allowed = String::from("{\"allowed_modules\": [");
        let mut dependencies = Vec::with_capacity(2 * n);
        for index in 0..n {
            // A third of the folders are a module's, a third a header's, a third nobody's.
            let module = uuid(3 * index);
            let header = uuid(3 * index + 1);
            let name = match index % 3 {
                0 => module.to_string(),
                1 => header.to_string(),
                _ => module.simple().to_string(),
            };
            let mut folder = ConfigFolder::new(name.as_str());
            folder.permissions = Some(Bytes::from_static(
                b"{\"allowed_modules\": [\"@minecraft/server\"]}",
            ));
            files.folders.push(folder);
            plugins.push(plugin(
                &format!("p{index}"),
                header,
                module,
                vec![("@minecraft/server".to_owned(), false)],
            ));
            write!(
                allowed,
                "{}\"@acme/m{index}\"",
                if index == 0 { "" } else { ", " }
            )
            .unwrap();
            for (prefix, optional) in [("m", false), ("x", true)] {
                dependencies.push((format!("@acme/{prefix}{index}"), optional));
            }
        }
        allowed.push_str("]}");
        let mut default = ConfigFolder::new("default");
        default.permissions = Some(Bytes::from(allowed));
        files.folders.push(default);
        Stack {
            config: ScriptConfig::parse(&files),
            plugins,
            greedy: plugin(
                "greedy",
                Uuid::nil(),
                Uuid::from_u128(u128::MAX),
                dependencies,
            ),
        }
    };
    assert_each_linear_at(
        20_000,
        input,
        &[
            (
                "ScriptConfig::filter, n allowed modules and 2 n dependencies",
                &|stack: &Stack| {
                    let verdict = stack.config.filter(&stack.greedy);
                    assert!(verdict.runs);
                    assert_eq!(verdict.dropped.len(), stack.plugins.len());
                },
            ),
            (
                "ScriptConfig::filter_stack, n plugins, most on a default of n modules",
                &|stack: &Stack| {
                    let verdicts = stack.config.filter_stack(&stack.plugins);
                    assert_eq!(verdicts.len(), stack.plugins.len());
                    let running = verdicts.iter().filter(|verdict| verdict.runs).count();
                    // A plugin with its own folder runs; the others fall to `default`.
                    assert_eq!(running, stack.plugins.len().div_ceil(3));
                },
            ),
            (
                "ScriptConfig::unmatched_folders, n folders and n plugins",
                &|stack: &Stack| {
                    let unmatched = stack.config.unmatched_folders(&stack.plugins);
                    assert_eq!(
                        unmatched.len(),
                        stack.plugins.len() - stack.plugins.len().div_ceil(3)
                    );
                },
            ),
            (
                "ScriptConfig::for_module for each of n modules",
                &|stack: &Stack| {
                    let own = stack
                        .plugins
                        .iter()
                        .filter(|p| {
                            stack.config.for_module(p.module_uuid).source
                                != super::ConfigSource::Default
                        })
                        .count();
                    assert_eq!(own, stack.plugins.len().div_ceil(3));
                },
            ),
        ],
    );
}

#[test]
fn server_properties_are_linear_in_their_lines() {
    type Input = (usize, String, ServerProperties);
    let input = |n: usize| {
        let mut text = String::new();
        for index in 0..n {
            // Known and unknown keys, a key repeated on every line, refused values, skipped
            // lines: a finding for most.
            writeln!(text, "script-watchdog-spike-threshold={index}ms").unwrap();
            writeln!(text, "content-log-level=level{index}").unwrap();
            writeln!(text, "my-key-{index}=x=y").unwrap();
            writeln!(text, " transport=raknet").unwrap();
            writeln!(text, "# comment {index}").unwrap();
        }
        (n, text.clone(), ServerProperties::parse(text.as_bytes()))
    };
    assert_each_linear_at(
        10_000,
        input,
        &[
            (
                "ServerProperties::parse, 5 n lines",
                &|(n, text, _): &Input| {
                    let properties = ServerProperties::parse(text.as_bytes());
                    assert_eq!(properties.entries.len(), 3 * n);
                    assert_eq!(properties.diagnostics().len(), 3 * n);
                },
            ),
            (
                "ServerProperties::script, 3 n applied lines",
                &|(n, _, properties): &Input| {
                    let mut findings = Diagnostics::default();
                    let script = properties.script(&mut findings);
                    assert_eq!(script.watchdog.spike_ms, i32::try_from(n - 1).ok());
                    assert_eq!(findings.len(), 2 * n);
                },
            ),
            (
                "ServerProperties::to_text, 3 n entries",
                &|(n, _, properties): &Input| {
                    assert_eq!(properties.to_text().unwrap().lines().count(), 3 * n);
                },
            ),
        ],
    );
}

#[test]
fn the_findings_of_server_properties_are_one_per_line_without_a_bound() {
    // Nothing folds or stops the findings of a file: a line that earns one gets one, so
    // their number grows with the file and no faster, and so does the time.
    let input = |n: usize| {
        let mut bytes = Vec::new();
        for index in 0..n {
            // An unknown key (one finding), and one that is not UTF-8 as well (two).
            bytes.extend_from_slice(format!("k{index}=v\n").as_bytes());
            bytes.extend_from_slice(b"k\xff=v\n");
        }
        (n, bytes)
    };
    assert_linear_at(
        "ServerProperties::parse, 2 n lines with 3 n findings",
        10_000,
        input,
        |(n, bytes)| {
            let properties = ServerProperties::parse(bytes);
            assert_eq!(properties.entries.len(), 2 * n);
            assert_eq!(properties.diagnostics().len(), 3 * n);
        },
    );
}

#[test]
fn secret_string_converts_from_text() {
    let owned: super::SecretString = String::from("hunter2").into();
    let borrowed: super::SecretString = "hunter2".into();
    assert_eq!(owned, borrowed);
    assert_eq!(owned.expose(), "hunter2");
}

#[test]
fn config_files_compare_equal_when_their_bytes_do() {
    let files = ScriptConfigFiles::default();
    assert_eq!(files.clone(), files);
    let folder = super::ConfigFolder {
        name: "default".into(),
        permissions: None,
        variables: None,
        secrets: None,
        stray_commands: false,
    };
    assert_eq!(folder.clone(), folder);
}

#[test]
fn a_context_carries_the_permission_list_the_configuration_gives_the_module() {
    let own = uuid(1);
    let folder = |name: &str, permissions: &str| {
        let mut folder = ConfigFolder::new(name);
        folder.permissions = Some(Bytes::copy_from_slice(permissions.as_bytes()));
        folder
    };
    let config = ScriptConfig::parse(&ScriptConfigFiles {
        folders: vec![
            folder("default", r#"{"allowed_modules": ["@minecraft/server"]}"#),
            folder(&own.to_string(), r#"{"allowed_modules": ["*"]}"#),
        ],
        ..ScriptConfigFiles::default()
    });
    let gates = bedrock_addon_family_script::ScriptContext {
        beta_apis: true,
        is_package_origin: true,
        ..bedrock_addon_family_script::ScriptContext::default()
    };
    // The module's own folder, and every other gate as it was given.
    let cx = config.context(gates, own);
    assert_eq!(cx.allowed_modules, Some(&["*".to_owned()][..]));
    assert!(cx.beta_apis && cx.is_package_origin && !cx.editor_world && !cx.allow_internal);
    // A module with no folder of its own takes `default`'s list.
    let cx = config.context(gates, uuid(2));
    assert_eq!(
        cx.allowed_modules,
        Some(&["@minecraft/server".to_owned()][..])
    );
}
