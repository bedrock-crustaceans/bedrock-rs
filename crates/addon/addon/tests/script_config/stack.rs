//! The configuration against the script modules of a stack: folder lookup, the dependency
//! filter and the folders nothing uses (fixtures sc01–sc18, sc23–sc24, sc44, sc49; validation
//! D1–D6, M1, N8; link rules S8, S9).

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::script::ScriptPlugin;
use bedrock_addon::families::server_config::{
    ConfigFolder, ConfigSource, PackPermissions, ScriptConfig, ScriptConfigFiles,
};
use bedrock_addon::version::FormatVersion;
use bytes::Bytes;
use uuid::Uuid;

use crate::common::{MODULE, VARS, check_config, check_stack, config, seen};

fn uuid(text: &str) -> Uuid {
    text.parse().unwrap()
}

fn plugin(name: &str, module: &str, dependencies: &[(&str, bool)]) -> ScriptPlugin {
    let dependencies: Vec<(String, bool)> = dependencies
        .iter()
        .map(|(module, optional)| ((*module).to_owned(), *optional))
        .collect();
    crate::common::plugin(
        name,
        "1.0.0",
        uuid("b0000000-0000-4000-8000-0000000000ff"),
        uuid(module),
        &dependencies,
    )
}

/// A `config/` whose folders hold the given `permissions.json` texts.
fn folders(entries: &[(&str, &str)]) -> ScriptConfig {
    ScriptConfig::parse(&ScriptConfigFiles {
        folders: entries
            .iter()
            .map(|(name, text)| {
                let mut folder = ConfigFolder::new(*name);
                folder.permissions = Some(Bytes::copy_from_slice(text.as_bytes()));
                folder
            })
            .collect(),
        ..ScriptConfigFiles::default()
    })
}

const SERVER_ONLY: &str = r#"{"allowed_modules": ["@minecraft/server"]}"#;
const UI_ONLY: &str = r#"{"allowed_modules": ["@minecraft/server-ui"]}"#;

// ---- lookup and precedence ----

#[test]
fn sc01_default_only_refuses_the_unlisted_module() {
    let (config, verdicts) = check_stack("sc01_default_only");
    assert!(!verdicts[0].runs);
    assert!(verdicts[0].dropped.is_empty());
    let finding = verdicts[0].diagnostics.first_error().unwrap();
    assert_eq!(finding.code.name(), "ScriptModulePermission");
    assert_eq!(
        config.for_module(uuid(MODULE)).source,
        ConfigSource::Default
    );
    check_config("sc01_default_only");
}

#[test]
fn sc02_module_folder_is_used_instead_of_default() {
    let (config, verdicts) = check_stack("sc02_module_folder");
    assert!(verdicts[0].runs);
    assert!(verdicts[0].diagnostics.is_empty());
    let used = config.for_module(uuid(MODULE));
    assert_eq!(used.source, ConfigSource::Folder(MODULE.to_owned()));
    assert_eq!(
        used.permissions.allowed_modules,
        ["@minecraft/server", "@minecraft/server-net"]
    );
    check_config("sc02_module_folder");
}

#[test]
fn sc03_header_folder_is_never_used() {
    let (config, verdicts) = check_stack("sc03_header_folder");
    assert!(!verdicts[0].runs);
    assert_eq!(
        config.for_module(uuid(MODULE)).source,
        ConfigSource::Default
    );
    // Not even for a lookup by the header UUID's own value: only script modules are looked up.
    check_config("sc03_header_folder");
}

#[test]
fn sc04_folder_without_permissions_allows_nothing() {
    let (config, verdicts) = check_stack("sc04_folder_without_permissions");
    assert!(!verdicts[0].runs);
    let used = config.for_module(uuid(MODULE));
    assert!(used.permissions.allowed_modules.is_empty());
    assert_eq!(used.variables.len(), 1);
    check_config("sc04_folder_without_permissions");
}

#[test]
fn sc05_no_merge_variables() {
    let (config, verdicts) = check_stack("sc05_no_merge_variables");
    assert!(verdicts.iter().all(|verdict| verdict.runs));
    let own = config.for_module(uuid(VARS));
    assert_eq!(own.variables.keys().collect::<Vec<_>>(), ["own"]);
    assert_eq!(own.secrets.keys().collect::<Vec<_>>(), ["s"]);
    let other = config.for_module(uuid("a0000000-0000-4000-8000-000000000002"));
    assert_eq!(other.variables.keys().collect::<Vec<_>>(), ["default_only"]);
    assert!(other.secrets.is_empty());
    check_config("sc05_no_merge_variables");
}

#[test]
fn sc06_upper_case_folder_is_never_used() {
    let (config, verdicts) = check_stack("sc06_upper_case_folder");
    assert!(!verdicts[0].runs);
    assert_eq!(
        config.for_module(uuid(MODULE)).source,
        ConfigSource::Default
    );
    assert!(config.packs.contains_key(&MODULE.to_uppercase()));
    check_config("sc06_upper_case_folder");
}

#[test]
fn sc07_no_default_refuses_every_dependency() {
    let (config, verdicts) = check_stack("sc07_no_default");
    assert!(!verdicts[0].runs);
    assert_eq!(config, ScriptConfig::default());
    check_config("sc07_no_default");
}

#[test]
fn sc08_no_config_refuses_every_dependency() {
    let (config, verdicts) = check_stack("sc08_no_config");
    assert!(!verdicts[0].runs);
    assert_eq!(config, ScriptConfig::default());
}

#[test]
fn d6_a_plugin_without_dependencies_runs_without_any_configuration() {
    let verdict = ScriptConfig::default().filter(&plugin("bare", MODULE, &[]));
    assert!(verdict.runs);
    assert!(verdict.diagnostics.is_empty());
}

// K124: the filter's input can be built by hand, without a manifest and a discovery: the
// plugin a host already knows (its pack, its module, what it depends on).
#[test]
fn a_plugin_built_by_hand_is_filtered_as_the_discovered_one() {
    use bedrock_addon::families::script::{
        ResolvedDependency, ScriptPath, ScriptRuntime, ScriptVersion,
    };
    let header = uuid("b0000000-0000-4000-8000-0000000000ff");
    let version = ScriptVersion::new(1, 0, 0);
    let mut built = ScriptPlugin::new("p", header, uuid(MODULE), version.clone());
    assert_eq!(built.name, "p - 1.0.0");
    assert_eq!(
        (built.pack_name.as_str(), built.pack_uuid, built.module_uuid),
        ("p", header, uuid(MODULE))
    );
    assert_eq!(built.module_version, version);
    assert_eq!(built.runtime, ScriptRuntime::JavaScript);
    assert_eq!(built.entry, "scripts/main.js");
    assert_eq!(built.main, ScriptPath::new("main.js"));
    assert!(built.dependencies.is_empty());
    assert!(!built.script_eval && !built.is_package_origin);
    assert_eq!(built.min_engine_version, None);
    // Without a dependency it runs under any configuration, as a discovered one does.
    assert!(ScriptConfig::default().filter(&built).runs);
    // With the dependencies a discovery would have kept, the verdicts are the discovered
    // plugin's: one required module the folder does not allow, one optional.
    let required = ResolvedDependency::new("@minecraft/server-net", ScriptVersion::new(1, 0, 0));
    assert_eq!(
        (
            required.module_name.as_str(),
            required.optional,
            required.module_uuid
        ),
        ("@minecraft/server-net", false, None)
    );
    assert_eq!(required.requested, required.resolved);
    let mut optional = ResolvedDependency::new("@minecraft/server-ui", ScriptVersion::new(1, 0, 0));
    optional.optional = true;
    built.dependencies = vec![
        ResolvedDependency::new("@minecraft/server", ScriptVersion::new(1, 0, 0)),
        required,
        optional,
    ];
    let discovered = plugin(
        "p",
        MODULE,
        &[
            ("@minecraft/server", false),
            ("@minecraft/server-net", false),
            ("@minecraft/server-ui", true),
        ],
    );
    assert_eq!(built.name, discovered.name);
    let config = folders(&[("default", SERVER_ONLY)]);
    let verdict = config.filter(&built);
    assert!(!verdict.runs);
    assert_eq!(verdict.dropped, ["@minecraft/server-ui"]);
    assert_eq!(verdict, config.filter(&discovered));
    assert_eq!(config.filter_stack([&built]), [verdict]);
    // The hand-built plugin answers for its folders too.
    let own = folders(&[("default", SERVER_ONLY), (MODULE, SERVER_ONLY)]);
    assert!(own.unmatched_folders([&built]).is_empty());
}

#[test]
fn sc09_non_uuid_folder_is_loaded_and_never_used() {
    let (config, verdicts) = check_stack("sc09_non_uuid_folder");
    assert!(verdicts[0].runs);
    assert!(config.packs.contains_key("not-a-uuid"));
    check_config("sc09_non_uuid_folder");
}

#[test]
fn sc10_stray_file_is_ignored() {
    let (config, verdicts) = check_stack("sc10_stray_file");
    assert!(verdicts[0].runs);
    assert!(config.packs.is_empty());
    check_config("sc10_stray_file");
}

#[test]
fn a_folder_whose_permissions_are_refused_never_falls_back_to_default() {
    // The folder stands for the module, whatever its files are worth.
    let config = folders(&[("default", SERVER_ONLY), (MODULE, "{ broken")]);
    let verdict = config.filter(&plugin("p", MODULE, &[("@minecraft/server", false)]));
    assert!(!verdict.runs);
    // With a third folder for another module, each module still has only its own.
    let other = "a0000000-0000-4000-8000-000000000002";
    let config = folders(&[
        ("default", SERVER_ONLY),
        (MODULE, "{ broken"),
        (other, UI_ONLY),
    ]);
    assert!(
        !config
            .filter(&plugin("p", MODULE, &[("@minecraft/server", false)]))
            .runs
    );
    assert!(
        !config
            .filter(&plugin("q", other, &[("@minecraft/server", false)]))
            .runs
    );
    assert!(
        config
            .filter(&plugin("q", other, &[("@minecraft/server-ui", false)]))
            .runs
    );
    let third = "a0000000-0000-4000-8000-000000000003";
    assert!(
        config
            .filter(&plugin("r", third, &[("@minecraft/server", false)]))
            .runs
    );
}

#[test]
fn m1_a_pack_folder_that_lacks_what_default_has_is_warned_deviation() {
    let shadows = |config: &ScriptConfig| -> Vec<String> {
        seen(config.diagnostics())
            .into_iter()
            .filter(|f| f.code == "PackConfigShadowsDefault")
            .inspect(|f| assert_eq!(f.severity, "info"))
            .map(|f| f.file)
            .collect()
    };
    let folder =
        |name: &str, permissions: Option<&str>, variables: Option<&str>, secrets: Option<&str>| {
            let mut folder = ConfigFolder::new(name);
            let bytes = |text: &str| Bytes::copy_from_slice(text.as_bytes());
            folder.permissions = permissions.map(bytes);
            folder.variables = variables.map(bytes);
            folder.secrets = secrets.map(bytes);
            folder
        };
    let full = folder(
        "default",
        Some(SERVER_ONLY),
        Some(r#"{"v": 1}"#),
        Some(r#"{"s": "x"}"#),
    );
    let parse = |folders: Vec<ConfigFolder>| {
        ScriptConfig::parse(&ScriptConfigFiles {
            folders,
            ..ScriptConfigFiles::default()
        })
    };
    // Each file the folder lacks and `default` has.
    let config = parse(vec![full.clone(), folder(MODULE, None, None, None)]);
    assert_eq!(
        shadows(&config),
        ["permissions.json", "variables.json", "secrets.json"]
            .map(|file| format!("config/{MODULE}/{file}"))
    );
    // A `permissions.json` without `allowed_modules` lacks the list too.
    let config = parse(vec![
        full.clone(),
        folder(MODULE, Some("{}"), Some("{}"), Some("{}")),
    ]);
    assert_eq!(
        shadows(&config),
        [format!("config/{MODULE}/permissions.json")]
    );
    // An empty list was written on purpose.
    let config = parse(vec![
        full.clone(),
        folder(
            MODULE,
            Some(r#"{"allowed_modules": []}"#),
            Some("{}"),
            Some("{}"),
        ),
    ]);
    assert!(shadows(&config).is_empty());
    // Nothing to hide: `default` has nothing.
    let config = parse(vec![folder(MODULE, None, None, None)]);
    assert!(shadows(&config).is_empty());
    let config = parse(vec![
        folder("default", Some("{}"), Some("{}"), Some("{}")),
        folder(MODULE, None, None, None),
    ]);
    assert!(shadows(&config).is_empty());
    // A folder no module can ever use hides nothing.
    let config = parse(vec![
        full.clone(),
        folder("not-a-uuid", None, None, None),
        folder(&MODULE.to_uppercase(), None, None, None),
    ]);
    assert!(shadows(&config).is_empty());
    // A file that was refused has its own error.
    let config = parse(vec![
        full,
        folder(MODULE, Some("{ broken"), Some("{}"), Some("{}")),
    ]);
    assert!(shadows(&config).is_empty());
}

// ---- the dependency filter ----

#[test]
fn sc11_wildcard_allows_every_dependency() {
    let (config, verdicts) = check_stack("sc11_wildcard");
    assert!(verdicts[0].runs);
    assert!(
        config
            .default
            .permissions
            .is_module_allowed("@anything/at-all")
    );
}

#[test]
fn the_module_match_is_exact() {
    let permissions = PackPermissions {
        allowed_modules: vec!["@minecraft/server".to_owned()],
        ..PackPermissions::default()
    };
    assert!(permissions.is_module_allowed("@minecraft/server"));
    for other in [
        "@minecraft/server-net",
        "@Minecraft/Server",
        "@minecraft/server@2.0.0",
        "@minecraft/serve",
        "",
        "*",
    ] {
        assert!(!permissions.is_module_allowed(other), "{other}");
    }
    assert!(!PackPermissions::default().is_module_allowed("@minecraft/server"));
}

#[test]
fn sc12_version_suffix_matches_nothing() {
    let (_, verdicts) = check_stack("sc12_version_suffix");
    assert!(!verdicts[0].runs);
    assert_eq!(verdicts[0].diagnostics.len(), 1);
}

#[test]
fn sc13_case_matches_nothing() {
    let (_, verdicts) = check_stack("sc13_case");
    assert!(!verdicts[0].runs);
}

#[test]
fn sc14_string_not_array_refuses_the_plugin() {
    let (_, verdicts) = check_stack("sc14_string_not_array");
    assert!(!verdicts[0].runs);
}

#[test]
fn sc15_mixed_elements_allow_both_strings() {
    let (_, verdicts) = check_stack("sc15_mixed_elements");
    assert!(verdicts[0].runs);
}

#[test]
fn sc16_optional_dependency_is_dropped_and_the_plugin_runs() {
    let (_, verdicts) = check_stack("sc16_optional_dependency");
    assert!(verdicts[0].runs);
    assert_eq!(verdicts[0].dropped, ["@minecraft/server-net"]);
    assert_eq!(verdicts[0].diagnostics.max_severity(), Some(Severity::Info));
}

#[test]
fn d1_d2_each_refused_dependency_has_its_own_finding() {
    let config = folders(&[("default", SERVER_ONLY)]);
    let verdict = config.filter(&plugin(
        "many",
        MODULE,
        &[
            ("@minecraft/server-ui", true),
            ("@minecraft/server", false),
            ("@minecraft/server-net", false),
            ("@minecraft/server-admin", true),
            ("@minecraft/common", false),
        ],
    ));
    assert!(!verdict.runs);
    assert_eq!(
        verdict.dropped,
        ["@minecraft/server-ui", "@minecraft/server-admin"]
    );
    let texts: Vec<(String, String, String)> = seen(&verdict.diagnostics)
        .into_iter()
        .map(|f| (f.code, f.severity, f.text.unwrap()))
        .collect();
    let refused = |module: &str| {
        (
            "ScriptModulePermission".to_owned(),
            "error".to_owned(),
            format!(
                "Plugin [many - 1.0.0] - requesting dependency on module [{module}] but it is not configured to use it."
            ),
        )
    };
    let dropped = (
        "DroppedField".to_owned(),
        "info".to_owned(),
        "field dropped".to_owned(),
    );
    assert_eq!(
        texts,
        [
            dropped.clone(),
            refused("@minecraft/server-net"),
            dropped,
            refused("@minecraft/common")
        ]
    );
}

#[test]
fn the_json_reader_rows_refuse_the_plugin() {
    // sc17, sc18, sc23, sc24: a `permissions.json` that gives no list refuses every
    // required dependency, with the folder's own file named.
    for name in [
        "sc17_syntax_error_permissions",
        "sc18_empty_file",
        "sc23_root_array",
        "sc24_null_root",
    ] {
        let (_, verdicts) = check_stack(name);
        assert!(!verdicts[0].runs, "{name}");
    }
    for name in [
        "sc19_comments",
        "sc20_trailing_comma",
        "sc21_bom",
        "sc22_trailing_garbage",
        "sc26_extra_member",
    ] {
        let (_, verdicts) = check_stack(name);
        assert!(verdicts[0].runs, "{name}");
    }
}

#[test]
fn sc49_unlisted_modules_are_refused_by_the_shipped_default() {
    let (_, verdicts) = check_stack("sc49_unlisted_modules");
    assert_eq!(verdicts.len(), 3);
    assert!(verdicts.iter().all(|verdict| !verdict.runs));
}

#[test]
fn a_world_a_client_hosts_allows_every_module() {
    let config = ScriptConfig::client_hosted();
    assert_eq!(config.default.permissions.allowed_modules, ["*"]);
    assert!(config.packs.is_empty());
    let verdict = config.filter(&plugin("p", MODULE, &[("@minecraft/server-net", false)]));
    assert!(verdict.runs);
}

// ---- the folders nothing uses ----

#[test]
fn d3_d4_d5_each_kind_of_unmatched_folder() {
    let header = "b0000000-0000-4000-8000-0000000000ff";
    let simple = MODULE.replace('-', "");
    let braced = format!("{{{MODULE}}}");
    let stranger = "c0000000-0000-4000-8000-000000000009";
    let config = folders(&[
        ("default", SERVER_ONLY),
        (MODULE, SERVER_ONLY),
        (header, SERVER_ONLY),
        (&header.to_uppercase(), SERVER_ONLY),
        (&simple, SERVER_ONLY),
        (&braced, SERVER_ONLY),
        (stranger, SERVER_ONLY),
        ("notes", SERVER_ONLY),
    ]);
    let plugins = [plugin("p", MODULE, &[])];
    let found: Vec<(String, String, Option<Uuid>)> = config
        .unmatched_folders(&plugins)
        .iter()
        .map(|d| {
            let folder = d
                .code
                .args_of(&bedrock_addon::diagnostic::codes::UNMATCHED_CONFIG_FOLDER)
                .or_else(|| {
                    d.code.args_of(
                        &bedrock_addon::diagnostic::codes::UNMATCHED_CONFIG_FOLDER_EXPECTED,
                    )
                })
                .unwrap_or_else(|| panic!("{:?}", d.code))[0]
                .clone();
            let expected = match d.detail.as_deref() {
                Some(bedrock_addon::diagnostic::DiagnosticDetail::Uuid(uuid)) => Some(*uuid),
                _ => None,
            };
            (folder, d.severity.to_string(), expected)
        })
        .collect();
    let module = Some(uuid(MODULE));
    // Every unmatched folder is an information of ours: the server has no
    // line for a folder it does not use; the module's folder is named where it is known.
    let near = |folder: &str| (folder.to_owned(), "info".to_owned(), module);
    let info = |folder: &str| (folder.to_owned(), "info".to_owned(), None);
    // In folder order (sorted by name); `default` and the module's own folder are used.
    let mut expected = vec![
        near(&header.to_uppercase()),
        near(&simple),
        near(header),
        info(stranger),
        info("notes"),
        near(&braced),
    ];
    expected.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(found, expected);
    // Without a stack every folder but `default` is unmatched, as an info.
    let alone = config.unmatched_folders(&[]);
    assert_eq!(alone.len(), 7);
    assert_eq!(alone.max_severity(), Some(Severity::Info));
}

// ---- engine gates ----

#[test]
fn sc44_net_payload_size_is_newer_than_an_older_engine() {
    let config = config("sc44_net_payload_size");
    let since = FormatVersion::new(1, 26, 36);
    for older in [FormatVersion::new(1, 26, 20), FormatVersion::new(1, 26, 35)] {
        let findings = config.engine_findings(older);
        let found = seen(&findings);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].code, "FieldSinceEngine");
        // The crate's prediction of an older engine: an information.
        assert_eq!(found[0].severity, "info");
        assert_eq!(found[0].file, "config/default/permissions.json");
        assert_eq!(
            found[0].path,
            "module_permissions/@minecraft~1server-net/max_websocket_payload_size"
        );
        assert_eq!(
            findings
                .iter()
                .next()
                .unwrap()
                .code
                .args_of(&bedrock_addon::diagnostic::codes::FIELD_SINCE_ENGINE),
            Some(&[since.to_string()][..])
        );
    }
    for known in [since, FormatVersion::new(1, 26, 50)] {
        assert!(config.engine_findings(known).is_empty());
    }
    // An entry without the member is known to every engine.
    let old = FormatVersion::new(1, 26, 20);
    assert!(
        crate::common::config("sc37_net_valid")
            .engine_findings(old)
            .is_empty()
    );
}
