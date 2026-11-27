//! Writing the JSON files back: what is written reads back to the same value, equals the
//! input for a file that loaded without a rewrite finding (but for the named differences), and
//! is never text the reader takes for something else.

use std::collections::BTreeMap;

use bedrock_addon_core::diagnostic::{Kind, Severity};
use bedrock_addon_core::json::{self, EncodeError, Value};
use bedrock_addon_family_server_config::{
    CommandPermissionLevel, CommandsConfig, ConfigFolder, NetModuleConfig, PackPermissions,
    PackScriptConfig, ScriptConfig, ScriptConfigFiles, SecretString, ServerNet,
};
use bytes::Bytes;

use crate::common::{MODULE, VARS, config, seen};

/// The parsed `default` folder of a `config/` whose three files are the given texts.
fn folder(
    permissions: Option<&str>,
    variables: Option<&str>,
    secrets: Option<&str>,
) -> ScriptConfig {
    let bytes = |text: &str| Bytes::copy_from_slice(text.as_bytes());
    let mut folder = ConfigFolder::new("default");
    folder.permissions = permissions.map(bytes);
    folder.variables = variables.map(bytes);
    folder.secrets = secrets.map(bytes);
    ScriptConfig::parse(&ScriptConfigFiles {
        folders: vec![folder],
        ..ScriptConfigFiles::default()
    })
}

fn commands(text: &str) -> ScriptConfig {
    ScriptConfig::parse(&ScriptConfigFiles {
        commands: Some(Bytes::copy_from_slice(text.as_bytes())),
        ..ScriptConfigFiles::default()
    })
}

/// No finding says the reader changed or dropped something of the file: nothing at warning
/// level or above, and none of the reader's own notes, informations (a value of
/// the wrong type or form, one with no effect).
fn clean(config: &ScriptConfig) -> bool {
    config.diagnostics().max_severity() < Some(Severity::Warning)
        && !config.diagnostics().iter().any(|d| {
            matches!(
                d.code.kind(),
                Kind::WrongType | Kind::InvalidValue | Kind::InvalidEnumValue | Kind::NeverApplies
            )
        })
}

/// A JSON value in a form that compares member for member whatever the member order:
/// numbers as numbers (an integer and a fraction apart), keys sorted.
fn canon(value: &Value) -> String {
    if let Some(object) = value.as_object() {
        let members: BTreeMap<&str, String> = object
            .iter()
            .map(|(key, value)| (key.as_str(), canon(value)))
            .collect();
        format!("{members:?}")
    } else if let Some(array) = value.as_array() {
        format!("{:?}", array.iter().map(canon).collect::<Vec<_>>())
    } else if let Some(number) = value.as_number() {
        if number.is_float() {
            format!("f{:?}", number.to_f64_lossy())
        } else {
            format!("i{:?}", number.to_i128())
        }
    } else if let Some(text) = value.as_string() {
        format!("{:?}", text.as_str())
    } else if let Some(flag) = value.as_bool() {
        format!("{flag}")
    } else {
        "null".to_owned()
    }
}

fn read(text: &str) -> Value {
    json::from_str_jsonc(text).unwrap()
}

// ---- by hand ----

#[test]
fn a_permissions_file_is_written_as_it_was_read() {
    let text = r#"{
  "allowed_modules": [
    "@minecraft/server",
    "@minecraft/server-net"
  ],
  "module_permissions": {
    "@minecraft/server-net": {
      "allowed_uris": [
        "https://example.com/api"
      ],
      "force_https": true,
      "max_body_bytes": 1000
    },
    "@acme/other": {
      "k": [
        1,
        2.5,
        null
      ]
    }
  },
  "commands_permission_level": "game_directors",
  "note": "kept"
}
"#;
    let config = folder(Some(text), None, None);
    assert_eq!(config.default.permissions.to_json().unwrap(), text);
}

#[test]
fn the_fixtures_that_load_write_text_that_reads_back_equal() {
    for (name, key) in [
        ("sc02_module_folder", MODULE),
        ("sc19_comments", MODULE),
        ("sc20_trailing_comma", MODULE),
        ("sc21_bom", MODULE),
        ("sc27_variable_types", VARS),
        ("sc28_duplicate_names", VARS),
        ("sc29_secret_types", VARS),
        ("sc05_no_merge_variables", VARS),
    ] {
        let config = config(name);
        let pack = &config.packs[key];
        let again = folder(
            Some(&pack.permissions.to_json().unwrap()),
            Some(&pack.variables_json().unwrap()),
            Some(&pack.secrets_json().unwrap()),
        );
        assert!(
            again.diagnostics().is_empty(),
            "{name}: {:?}",
            again.diagnostics()
        );
        assert_eq!(again.default.permissions, pack.permissions, "{name}");
        assert_eq!(again.default.variables, pack.variables, "{name}");
        assert_eq!(again.default.secrets, pack.secrets, "{name}");
    }
}

#[test]
fn nothing_the_author_did_not_write_is_written() {
    let empty = PackScriptConfig::default();
    assert_eq!(empty.permissions.to_json().unwrap(), "{}\n");
    assert_eq!(empty.variables_json().unwrap(), "{}\n");
    assert_eq!(empty.secrets_json().unwrap(), "{}\n");
    assert_eq!(CommandsConfig::default().to_json().unwrap(), "{}\n");
}

#[test]
fn secrets_json_writes_the_values() {
    let mut pack = PackScriptConfig::default();
    pack.secrets
        .insert("token".to_owned(), SecretString::new("hunter2"));
    let text = pack.secrets_json().unwrap();
    assert_eq!(read(&text), read(r#"{"token": "hunter2"}"#));
}

#[test]
fn a_server_net_configuration_built_by_hand_is_written_as_its_entry() {
    let net = NetModuleConfig {
        allowed_uris: Some(vec!["https://example.com/".to_owned()]),
        force_tls: Some(true),
        max_body_bytes: Some(u32::MAX),
        max_concurrent_requests: Some(2),
        session_headers: Some([("x-a".to_owned(), "b".to_owned())].into_iter().collect()),
        max_websocket_connections: Some(u16::MAX),
        max_websocket_payload_size: Some(64),
    };
    let permissions = PackPermissions {
        allowed_modules: vec!["@minecraft/server-net".to_owned()],
        server_net: ServerNet::Configured(net.clone()),
        ..PackPermissions::default()
    };
    let config = folder(Some(&permissions.to_json().unwrap()), None, None);
    assert!(config.diagnostics().is_empty());
    assert_eq!(
        config.default.permissions.server_net,
        ServerNet::Configured(net)
    );
    // The members that are not set are not written.
    let bare = PackPermissions {
        server_net: ServerNet::Configured(NetModuleConfig::default()),
        ..PackPermissions::default()
    };
    assert_eq!(
        read(&bare.to_json().unwrap()),
        read(r#"{"module_permissions": {"@minecraft/server-net": {}}}"#)
    );
}

#[test]
fn a_written_pack_folder_gives_the_findings_it_gave() {
    // A pack folder beside a `default` that has a list: without the member the folder is
    // warned of (it allows nothing and hides `default`); with an empty list written on
    // purpose it is not. Writing the file back changes neither.
    let beside_default = |text: &str| {
        let bytes = |text: &str| Some(Bytes::copy_from_slice(text.as_bytes()));
        let mut default = ConfigFolder::new("default");
        default.permissions = bytes(r#"{"allowed_modules": ["@minecraft/server"]}"#);
        let mut pack = ConfigFolder::new(MODULE);
        pack.permissions = bytes(text);
        ScriptConfig::parse(&ScriptConfigFiles {
            folders: vec![default, pack],
            ..ScriptConfigFiles::default()
        })
    };
    for (text, findings, written) in [
        (r#"{"allowed_modules": []}"#, 0, r#"{"allowed_modules":[]}"#),
        ("{}", 1, "{}"),
        (
            r#"{"commands_permission_level": "host"}"#,
            1,
            r#"{"commands_permission_level":"host"}"#,
        ),
        (
            r#"{"allowed_modules": [], "note": 1}"#,
            1,
            r#"{"allowed_modules":[],"note":1}"#,
        ),
    ] {
        let config = beside_default(text);
        assert_eq!(config.diagnostics().len(), findings, "{text}");
        let permissions = &config.packs[MODULE].permissions;
        let out = permissions.to_json().unwrap();
        assert_eq!(canon(&read(&out)), canon(&read(written)), "{text}");
        let again = beside_default(&out);
        assert_eq!(&again.packs[MODULE].permissions, permissions, "{text}");
        // The same findings, at the offsets of the written text.
        assert!(
            seen(again.diagnostics()) == seen(config.diagnostics()),
            "{text}: {:?}",
            again.diagnostics()
        );
    }
    // A model built by hand writes the member when it is told the list was written.
    let mut by_hand = PackPermissions::default();
    assert_eq!(
        canon(&read(&by_hand.to_json().unwrap())),
        canon(&read("{}"))
    );
    by_hand.allowed_modules_written = true;
    assert_eq!(
        canon(&read(&by_hand.to_json().unwrap())),
        canon(&read(r#"{"allowed_modules": []}"#))
    );
}

#[test]
fn a_hand_built_list_with_the_flag_unset_does_not_equal_its_read_back() {
    // The flag takes part in `PartialEq`, and the reader sets it for a file with the
    // member, so a value built by hand with a list and the flag false is not its own read-back.
    let by_hand = PackPermissions {
        allowed_modules: vec!["@minecraft/server".to_owned()],
        ..PackPermissions::default()
    };
    assert!(!by_hand.allowed_modules_written);
    let read_back = folder(Some(&by_hand.to_json().unwrap()), None, None)
        .default
        .permissions;
    assert!(read_back.allowed_modules_written);
    assert_eq!(read_back.allowed_modules, by_hand.allowed_modules);
    assert_ne!(read_back, by_hand);
    // The reader's own value does round trip.
    let again = folder(Some(&read_back.to_json().unwrap()), None, None)
        .default
        .permissions;
    assert_eq!(again, read_back);
}

#[test]
fn an_edit_of_the_held_server_net_entry_alone_is_not_written() {
    // `to_json` writes `server_net` once it differs from what the held entry loads
    // to, so an edit of the raw entry alone is dropped, and the typed value is written.
    let text = r#"{"module_permissions": {"@minecraft/server-net": {"force_https": true}}}"#;
    let mut parsed = folder(Some(text), None, None).default.permissions;
    parsed
        .module_permissions
        .get_mut("@minecraft/server-net")
        .unwrap()
        .insert("max_body_bytes".to_owned(), Value::from(7_u64));
    let written = read(&parsed.to_json().unwrap());
    let modules = written.as_object().unwrap().get("module_permissions");
    let entry = modules
        .and_then(Value::as_object)
        .and_then(|modules| modules.get("@minecraft/server-net"))
        .and_then(Value::as_object)
        .unwrap();
    assert!(entry.get("max_body_bytes").is_none(), "{entry:?}");
    assert_eq!(
        entry.get("force_tls"),
        Some(&Value::from(true)),
        "{entry:?}"
    );
}

#[test]
fn an_edited_server_net_is_what_is_written() {
    let text = r#"{"module_permissions": {"@minecraft/server-net": {"force_https": true, "allowed_uris": ["https://a/"]}, "@acme/other": {"k": 1}}}"#;
    let parsed = folder(Some(text), None, None).default.permissions;
    let authored = NetModuleConfig {
        allowed_uris: Some(vec!["https://a/".to_owned()]),
        force_tls: Some(true),
        ..NetModuleConfig::default()
    };
    assert_eq!(parsed.server_net, ServerNet::Configured(authored.clone()));
    let net_entry = |permissions: &PackPermissions| {
        let written = read(&permissions.to_json().unwrap());
        let modules = written.as_object().unwrap().get("module_permissions");
        let modules = modules.and_then(Value::as_object).unwrap();
        // The other module is written whatever happens to server-net.
        assert_eq!(
            canon(modules.get("@acme/other").unwrap()),
            canon(&read(r#"{"k": 1}"#))
        );
        modules.get("@minecraft/server-net").map(canon)
    };
    // Not edited: the entry as its author wrote it, the legacy spelling included.
    assert_eq!(
        net_entry(&parsed),
        Some(canon(&read(
            r#"{"force_https": true, "allowed_uris": ["https://a/"]}"#
        )))
    );
    // Edited: the typed value, which reads back as the edit.
    let mut edited = parsed.clone();
    let wider = NetModuleConfig {
        allowed_uris: Some(vec!["https://a/".to_owned(), "https://b/".to_owned()]),
        max_body_bytes: Some(10),
        ..authored
    };
    edited.server_net = ServerNet::Configured(wider.clone());
    assert_eq!(
        net_entry(&edited),
        Some(canon(&read(
            r#"{"allowed_uris": ["https://a/", "https://b/"], "force_tls": true, "max_body_bytes": 10}"#
        )))
    );
    let again = folder(Some(&edited.to_json().unwrap()), None, None);
    assert_eq!(
        again.default.permissions.server_net,
        ServerNet::Configured(wider)
    );
    // Lifted: no entry.
    edited.server_net = ServerNet::Unrestricted;
    assert_eq!(net_entry(&edited), None);
    let again = folder(Some(&edited.to_json().unwrap()), None, None);
    assert_eq!(
        again.default.permissions.server_net,
        ServerNet::Unrestricted
    );
    // Denied over an entry that loads has no JSON form.
    edited.server_net = ServerNet::Denied;
    match edited.to_json() {
        Err(EncodeError::NotRepresentable { at, .. }) => {
            assert_eq!(at.to_string(), "module_permissions/@minecraft~1server-net");
        }
        other => panic!("{other:?}"),
    }
    // A refused entry is denied and written as authored; edited, the edit is written.
    let refused = r#"{"module_permissions": {"@minecraft/server-net": {"bogus": 1}, "@acme/other": {"k": 1}}}"#;
    let mut denied = folder(Some(refused), None, None).default.permissions;
    assert_eq!(denied.server_net, ServerNet::Denied);
    assert_eq!(net_entry(&denied), Some(canon(&read(r#"{"bogus": 1}"#))));
    denied.server_net = ServerNet::Configured(NetModuleConfig::default());
    assert_eq!(net_entry(&denied), Some(canon(&read("{}"))));
}

#[test]
fn what_would_read_back_as_something_else_is_refused() {
    let refused = |result: Result<String, EncodeError>| match result {
        Err(EncodeError::NotRepresentable { at, .. }) => at.to_string(),
        other => panic!("{other:?}"),
    };
    // A denied server-net without the entry that denied it has no JSON form.
    let denied = PackPermissions {
        server_net: ServerNet::Denied,
        ..PackPermissions::default()
    };
    assert_eq!(
        refused(denied.to_json()),
        "module_permissions/@minecraft~1server-net"
    );
    // The fixture whose entry is no object loads to that state.
    let config = config("sc45_net_not_object");
    assert_eq!(
        refused(config.default.permissions.to_json()),
        "module_permissions/@minecraft~1server-net"
    );
    // An unknown member named like a known one would be read as that member.
    for name in [
        "allowed_modules",
        "module_permissions",
        "commands_permission_level",
    ] {
        let mut permissions = PackPermissions::default();
        permissions
            .extra
            .insert(name.to_owned(), Value::from(1_i64));
        assert_eq!(refused(permissions.to_json()), name);
    }
    for name in ["default_permission_level", "permission_levels"] {
        let mut commands = CommandsConfig::default();
        commands.extra.insert(name.to_owned(), Value::from(1_i64));
        assert_eq!(refused(commands.to_json()), name);
    }
}

#[test]
fn a_commands_file_is_written_as_it_was_read() {
    let text = r#"{
  "default_permission_level": "admin",
  "permission_levels": {
    "list": "internal",
    "say": "any"
  },
  "unknown_member": 1
}
"#;
    let config = commands(text);
    assert_eq!(config.commands.as_ref().unwrap().to_json().unwrap(), text);
}

// ---- generated ----

/// xorshift64*: a fixed sequence per seed.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % n as u64).unwrap()
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len())]
    }

    fn chance(&mut self, of: usize) -> bool {
        self.below(of) == 0
    }
}

/// `{ members }` with each member on a line.
fn object(members: &[(String, String)]) -> String {
    let lines: Vec<String> = members
        .iter()
        .map(|(key, value)| format!("  {key:?}: {value}"))
        .collect();
    format!("{{\n{}\n}}\n", lines.join(",\n"))
}

/// Any JSON value, a few levels deep.
fn any(rng: &mut Rng, depth: usize) -> String {
    match rng.below(if depth == 0 { 9 } else { 11 }) {
        0 => "null".to_owned(),
        1 => "true".to_owned(),
        2 => "-3".to_owned(),
        3 => "1.5".to_owned(),
        4 => "12345678901234".to_owned(),
        5 => "123456789012345678901234567890".to_owned(),
        6 => "\"text \\\" \\u00e9\"".to_owned(),
        7 => "5.0".to_owned(),
        8 => "\"\"".to_owned(),
        9 => format!("[{}, {}]", any(rng, depth - 1), any(rng, depth - 1)),
        _ => format!(
            "{{\"a\": {}, \"b\": {}}}",
            any(rng, depth - 1),
            any(rng, depth - 1)
        ),
    }
}

/// One server-net entry, valid or not.
fn net_entry(rng: &mut Rng) -> String {
    let mut members = Vec::new();
    for (name, forms) in [
        (
            "allowed_uris",
            &[
                "[\"https://example.com/\"]",
                "[]",
                "[\"http://a.example/*\"]",
                "\"x\"",
            ][..],
        ),
        ("force_tls", &["true", "false", "\"yes\""][..]),
        ("force_https", &["true", "false"][..]),
        ("max_body_bytes", &["0", "4294967295", "-1", "1e999"][..]),
        (
            "session_headers",
            &["{\"x-a\": \"b\"}", "{}", "{\"x-a\": 1}"][..],
        ),
        ("max_websocket_connections", &["65535", "65536"][..]),
        ("max_websocket_payload_size", &["100"][..]),
        ("bogus_key", &["1"][..]),
    ] {
        if rng.chance(4) {
            members.push(format!("{name:?}: {}", rng.pick(forms)));
        }
    }
    format!("{{{}}}", members.join(", "))
}

fn permissions_document(rng: &mut Rng) -> String {
    let mut members: Vec<(String, String)> = Vec::new();
    let allowed = |rng: &mut Rng| {
        rng.pick(&[
            "[]",
            "[\"@minecraft/server\"]",
            "[\"@minecraft/server\", \"@minecraft/server-net\"]",
            "[\"*\"]",
            "[\"*\", \"@minecraft/server\"]",
            "[\"@acme/tool\"]",
            "\"@minecraft/server\"",
            "[5, \"@minecraft/server\"]",
            "[\"@Minecraft/Server\"]",
        ])
        .to_owned()
    };
    if !rng.chance(4) {
        members.push(("allowed_modules".to_owned(), allowed(rng)));
    }
    if rng.chance(2) {
        let value = match rng.below(7) {
            0 => "{}".to_owned(),
            1 => "[1]".to_owned(),
            2 => "{\"@minecraft/server-net\": [1]}".to_owned(),
            3 => format!("{{\"@acme/other\": {{\"k\": {}}}}}", any(rng, 2)),
            4 => format!(
                "{{\"@acme/other\": {{}}, \"@minecraft/server-net\": {}}}",
                net_entry(rng)
            ),
            _ => format!("{{\"@minecraft/server-net\": {}}}", net_entry(rng)),
        };
        members.push(("module_permissions".to_owned(), value));
    }
    if rng.chance(2) {
        let level = rng.pick(&["\"any\"", "\"internal\"", "\"host\"", "\"bogus\"", "1"]);
        members.push(("commands_permission_level".to_owned(), level.to_owned()));
    }
    if rng.chance(3) {
        members.push((rng.pick(&["foo", "extra", "note"]).to_owned(), any(rng, 2)));
    }
    if rng.chance(6) {
        // A member written twice: the last stands.
        members.insert(0, ("allowed_modules".to_owned(), allowed(rng)));
    }
    if rng.chance(3) {
        let at = rng.below(members.len() + 1);
        let len = members.len();
        members.rotate_left(at.min(len));
    }
    object(&members)
}

#[test]
fn generated_permissions_files_round_trip() {
    let mut rng = Rng(0x5eed_c0f1_6000_0002);
    let mut classes: BTreeMap<&'static str, usize> = BTreeMap::new();
    for _ in 0..4000 {
        let input = permissions_document(&mut rng);
        let config = folder(Some(&input), None, None);
        let permissions = &config.default.permissions;
        let written = match permissions.to_json() {
            Ok(written) => written,
            Err(EncodeError::NotRepresentable { at, .. }) => {
                // Only the denied server-net whose entry was no object.
                assert_eq!(at.to_string(), "module_permissions/@minecraft~1server-net");
                assert!(!clean(&config), "{input}");
                *classes.entry("refused: no JSON form").or_default() += 1;
                continue;
            }
            Err(other) => panic!("{other:?}"),
        };
        let again = folder(Some(&written), None, None);
        assert_eq!(
            &again.default.permissions, permissions,
            "{input}\n{written}"
        );
        assert_eq!(again.default.permissions.to_json().unwrap(), written);
        if !clean(&config) {
            *classes.entry("loaded with a rewrite finding").or_default() += 1;
            continue;
        }
        // Against the input, member for member.
        let deduplicated = json::last_wins(&input);
        if deduplicated != input {
            *classes.entry("a member written twice").or_default() += 1;
        }
        let mut expected = read(&deduplicated);
        let root = expected.as_object_mut().unwrap();
        let mut same = true;
        // An empty list written on purpose is written back.
        if root
            .get("allowed_modules")
            .and_then(Value::as_array)
            .is_some_and(facet_value::VArray::is_empty)
        {
            *classes
                .entry("as the input, with an empty allowed_modules")
                .or_default() += 1;
        }
        if root
            .get("module_permissions")
            .and_then(Value::as_object)
            .is_some_and(facet_value::VObject::is_empty)
        {
            root.remove("module_permissions");
            *classes
                .entry("an empty module_permissions is not written")
                .or_default() += 1;
            same = false;
        }
        if same {
            *classes.entry("as the input").or_default() += 1;
        }
        assert_eq!(
            canon(&read(&written)),
            canon(&expected),
            "{input}\n{written}"
        );
    }
    let names: Vec<&str> = classes.keys().copied().collect();
    assert_eq!(
        names,
        [
            "a member written twice",
            "an empty module_permissions is not written",
            "as the input",
            "as the input, with an empty allowed_modules",
            "loaded with a rewrite finding",
            "refused: no JSON form",
        ]
    );
    assert!(classes.values().all(|count| *count >= 40), "{classes:?}");
}

#[test]
fn generated_variables_and_secrets_files_round_trip() {
    let mut rng = Rng(0x5eed_c0f1_6000_0003);
    let mut classes: BTreeMap<&'static str, usize> = BTreeMap::new();
    for _ in 0..3000 {
        let mut variables = Vec::new();
        let mut secrets = Vec::new();
        for _ in 0..rng.below(6) {
            let name = rng.pick(&["a", "b", "", "nested", "é", "a"]).to_owned();
            let value = if rng.chance(12) {
                "[1e999]".to_owned()
            } else {
                any(&mut rng, 2)
            };
            variables.push((name.clone(), value));
            let secret = rng.pick(&[
                "\"one\"",
                "\"\"",
                "\"t \\n w\"",
                "\"x\"",
                "5",
                "null",
                "[\"x\"]",
            ]);
            secrets.push((name, secret.to_owned()));
        }
        for (file, input) in [
            ("variables", object(&variables)),
            ("secrets", object(&secrets)),
        ] {
            let variables = file == "variables";
            let config = if variables {
                folder(None, Some(&input), None)
            } else {
                folder(None, None, Some(&input))
            };
            let pack = &config.default;
            let written = if variables {
                pack.variables_json()
            } else {
                pack.secrets_json()
            }
            .unwrap();
            let again = if variables {
                folder(None, Some(&written), None)
            } else {
                folder(None, None, Some(&written))
            };
            assert!(again.diagnostics().is_empty(), "{input}\n{written}");
            assert_eq!(
                again.default.variables, pack.variables,
                "{input}\n{written}"
            );
            assert_eq!(again.default.secrets, pack.secrets, "{input}\n{written}");
            if !clean(&config) {
                // A number held to the largest double, a secret that is no string.
                *classes.entry("loaded with a rewrite finding").or_default() += 1;
                continue;
            }
            let deduplicated = json::last_wins(&input);
            let class = if deduplicated == input {
                "as the input"
            } else {
                "a member written twice"
            };
            *classes.entry(class).or_default() += 1;
            assert_eq!(
                canon(&read(&written)),
                canon(&read(&deduplicated)),
                "{input}\n{written}"
            );
        }
    }
    let names: Vec<&str> = classes.keys().copied().collect();
    assert_eq!(
        names,
        [
            "a member written twice",
            "as the input",
            "loaded with a rewrite finding"
        ]
    );
    assert!(classes.values().all(|count| *count >= 300), "{classes:?}");
}

#[test]
fn generated_commands_files_round_trip() {
    let mut rng = Rng(0x5eed_c0f1_6000_0004);
    let mut classes: BTreeMap<&'static str, usize> = BTreeMap::new();
    let level = |rng: &mut Rng| {
        let names: Vec<String> = CommandPermissionLevel::ALL
            .iter()
            .map(|level| format!("{:?}", level.name()))
            .collect();
        match rng.below(14) {
            0 => "\"bogus\"".to_owned(),
            1 => "3".to_owned(),
            n => names[n % names.len()].clone(),
        }
    };
    for _ in 0..4000 {
        let mut members: Vec<(String, String)> = Vec::new();
        if rng.chance(2) {
            members.push(("default_permission_level".to_owned(), level(&mut rng)));
        }
        if !rng.chance(3) {
            let commands: Vec<String> = (0..rng.below(4))
                .map(|index| {
                    let name = ["list", "say", "op", "stop"][index];
                    format!("{name:?}: {}", level(&mut rng))
                })
                .collect();
            members.push((
                "permission_levels".to_owned(),
                format!("{{{}}}", commands.join(", ")),
            ));
        }
        if rng.chance(3) {
            members.push((
                rng.pick(&["unknown_member", "extra"]).to_owned(),
                any(&mut rng, 2),
            ));
        }
        if rng.chance(3) {
            let at = rng.below(members.len() + 1);
            let len = members.len();
            members.rotate_left(at.min(len));
        }
        // The file without what is written twice: the first of two members stands.
        let first = object(&members);
        let mut input = first.clone();
        if rng.chance(5) && !members.is_empty() {
            let (key, _) = members[rng.below(members.len())].clone();
            members.push((key, "\"internal\"".to_owned()));
            input = object(&members);
            *classes.entry("a member written twice").or_default() += 1;
        }
        let config = commands(&input);
        let Some(commands) = &config.commands else {
            assert!(config.diagnostics().has_errors(), "{input}");
            *classes.entry("refused at load").or_default() += 1;
            continue;
        };
        assert!(clean(&config), "{input}");
        let written = commands.to_json().unwrap();
        let again = self::commands(&written);
        assert_eq!(
            again.commands.as_ref(),
            Some(commands),
            "{input}\n{written}"
        );
        assert_eq!(again.commands.unwrap().to_json().unwrap(), written);
        let mut expected = read(&first);
        let root = expected.as_object_mut().unwrap();
        if root
            .get("permission_levels")
            .and_then(Value::as_object)
            .is_some_and(facet_value::VObject::is_empty)
        {
            root.remove("permission_levels");
            *classes
                .entry("an empty permission_levels is not written")
                .or_default() += 1;
        } else {
            *classes.entry("as the input").or_default() += 1;
        }
        assert_eq!(
            canon(&read(&written)),
            canon(&expected),
            "{input}\n{written}"
        );
    }
    let names: Vec<&str> = classes.keys().copied().collect();
    assert_eq!(
        names,
        [
            "a member written twice",
            "an empty permission_levels is not written",
            "as the input",
            "refused at load",
        ]
    );
    assert!(classes.values().all(|count| *count >= 200), "{classes:?}");
}
