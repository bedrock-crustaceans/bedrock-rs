//! 20 S1 to S10: the script module of a manifest, its files, and the server's configuration.

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticDetail, Phase, ReferenceHint, Severity};
use bedrock_addon::families::script::config::{ConfigFolder, ScriptConfig, ScriptConfigFiles};
use bedrock_addon::families::script::{ScriptBundle, ScriptPlugin};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{LinkOptions, OpenOptions, Pack, PackOrigin, StackSet};
use bedrock_addon::storage::PackPath;
use uuid::Uuid;

use super::super::common::stacks_of;

/// The header UUID of the behaviour pack `n`, and of its script module.
fn header(n: u128) -> Uuid {
    Uuid::from_u128(n)
}

fn module(n: u128) -> Uuid {
    Uuid::from_u128(n + 1_000_000)
}

/// A behaviour pack `n` whose format-2 manifest has one script module with the members
/// `module_members` (JSON, after the version) and the `dependencies` (JSON), and the `files`.
fn script_pack(n: u128, module_members: &str, dependencies: &str, files: &[(&str, &str)]) -> Pack {
    let manifest = format!(
        r#"{{
    "format_version": 2,
    "header": {{ "name": "bp {n}", "description": "", "uuid": "{}", "version": [1, 0, 0],
        "min_engine_version": [1, 21, 0] }},
    "modules": [{{ "type": "script", "uuid": "{}", "version": [1, 0, 0]{module_members} }}],
    "dependencies": [{dependencies}]
}}"#,
        header(n),
        module(n),
    );
    pack_of(&manifest, files, PackOrigin::User)
}

fn pack_of(manifest: &str, files: &[(&str, &str)], origin: PackOrigin) -> Pack {
    let mut all = vec![(
        PackPath::from_static("manifest.json"),
        manifest.as_bytes().to_vec(),
    )];
    for (path, text) in files {
        all.push((
            PackPath::new(path).expect("a path"),
            text.as_bytes().to_vec(),
        ));
    }
    let options = OpenOptions {
        origin,
        ..OpenOptions::default()
    };
    Pack::from_files(all, &options).expect("a pack")
}

const MAIN: &str = r#", "entry": "scripts/main.js""#;
const SERVER: &str = r#"{ "module_name": "@minecraft/server", "version": "2.0.0" }"#;

/// The world of the behaviour packs `packs`, the first on top.
fn world(packs: Vec<Pack>) -> StackSet {
    stacks_of(packs, Vec::new(), LoadOptions::default())
}

/// One finding as the rows compare it: code, severity, file, JSON path.
type Told = (&'static str, Severity, String, String);

fn told_of<'a>(findings: impl IntoIterator<Item = &'a Diagnostic>) -> Vec<Told> {
    findings
        .into_iter()
        .map(|f| {
            (
                f.code.name(),
                f.severity,
                f.location
                    .path
                    .as_ref()
                    .map_or(String::new(), |path| path.as_str().to_owned()),
                f.location.json_path.to_string(),
            )
        })
        .collect()
}

fn row(code: &'static str, severity: Severity, file: &str, path: &str) -> Told {
    (code, severity, file.to_owned(), path.to_owned())
}

/// The findings of the top pack's bundle; and the requirement every load row shares: the
/// stack's report holds each of them exactly once, and the link pass adds nothing.
fn loaded(set: &StackSet) -> Vec<Told> {
    let bundle: &ScriptBundle = &set.behavior.scripts()[0];
    let found = told_of(bundle.findings());
    let report = set.load_all();
    for finding in bundle.findings() {
        let copies = report.iter().filter(|other| *other == finding).count();
        assert_eq!(copies, 1, "{finding}");
        assert_ne!(finding.phase, Phase::Link, "{finding}");
    }
    assert_eq!(
        about_scripts(set, &LinkOptions::default()),
        [],
        "the link pass repeats no load rule"
    );
    assert_eq!(
        about_scripts(
            set,
            &LinkOptions {
                lints: false,
                ..Default::default()
            }
        ),
        []
    );
    found
}

/// What a link pass tells about a manifest or a script file: nothing, since every rule of
/// scripting is a rule of the load.
fn about_scripts(set: &StackSet, options: &LinkOptions) -> Vec<Told> {
    let linked = set.link_with(options);
    told_of(linked.iter().filter(|finding| {
        finding
            .location
            .path
            .as_ref()
            .is_some_and(|path| path.as_str() == "manifest.json" || path.as_str().ends_with(".js"))
    }))
}

#[test]
fn s1_an_entry_that_names_no_file_or_is_absent_is_an_error_of_the_load() {
    // script_module S5: the entry, its `scripts/` root removed, is no script file.
    let missing = world(vec![script_pack(
        1,
        r#", "entry": "scripts/gone.js""#,
        SERVER,
        &[("scripts/main.js", "")],
    )]);
    assert_eq!(
        loaded(&missing),
        [row(
            "UnresolvedReference",
            Severity::Error,
            "manifest.json",
            "modules/0/entry"
        )]
    );
    // S4: no entry at all.
    let absent = world(vec![script_pack(1, "", SERVER, &[("scripts/main.js", "")])]);
    assert_eq!(
        loaded(&absent),
        [row(
            "MissingRequiredField",
            Severity::Error,
            "manifest.json",
            "modules/0"
        )]
    );
    // The prefix is what is stripped, and a file that is there is no finding.
    let found = world(vec![script_pack(
        1,
        MAIN,
        SERVER,
        &[("scripts/main.js", "")],
    )]);
    assert_eq!(loaded(&found), []);
    assert!(found.behavior.scripts()[0].plugin.is_some());
}

#[test]
fn s2_a_module_name_is_matched_byte_for_byte_against_the_catalog() {
    // S11: a module no engine has; names are case-sensitive [bds-run P23].
    for name in ["@minecraft/nope", "@Minecraft/Server"] {
        let dependency = format!(r#"{{ "module_name": "{name}", "version": "2.0.0" }}"#);
        let set = world(vec![script_pack(
            1,
            MAIN,
            &dependency,
            &[("scripts/main.js", "")],
        )]);
        assert_eq!(
            loaded(&set),
            [row(
                "UnresolvedReference",
                Severity::Error,
                "manifest.json",
                "dependencies/0"
            )],
            "{name}"
        );
        assert_eq!(set.behavior.scripts()[0].plugin, None);
    }
    // S10 is the same reference against the server's list: a stack has no list, so the
    // configuration tells it, once (S9 below).
}

#[test]
fn s3_the_versions_a_pack_asks_for_are_resolved_once_per_stack() {
    let load = |dependencies: &str| {
        let set = world(vec![script_pack(
            1,
            MAIN,
            dependencies,
            &[("scripts/main.js", "")],
        )]);
        loaded(&set)
    };
    assert_eq!(
        load(r#"{ "module_name": "@minecraft/server", "version": "9.9.9" }"#),
        [row(
            "ScriptModuleVersionUnavailable",
            Severity::Error,
            "manifest.json",
            "dependencies/0"
        )]
    );
    assert_eq!(
        load(
            r#"{ "module_name": "@minecraft/server", "version": "1.5.0" },
               { "module_name": "@minecraft/server", "version": "2.0.0" }"#
        ),
        [row(
            "ScriptModuleVersionConflict",
            Severity::Error,
            "manifest.json",
            "dependencies/1"
        )]
    );
    // A stable request that the beta version another request names serves is promoted: a
    // lint of the load (R3; the Beta APIs let the beta request through).
    let promoted = stacks_of(
        vec![script_pack(
            1,
            MAIN,
            r#"{ "module_name": "@minecraft/server", "version": "2.0.0" },
               { "module_name": "@minecraft/server", "version": "beta" }"#,
            &[("scripts/main.js", "")],
        )],
        Vec::new(),
        LoadOptions {
            beta_apis: true,
            ..LoadOptions::default()
        },
    );
    assert_eq!(
        loaded(&promoted),
        [row(
            "ScriptModulePromoted",
            Severity::Info,
            "manifest.json",
            "dependencies/0"
        )]
    );
    assert!(promoted.behavior.scripts()[0].plugin.is_some());
    // Two packs with a faulty request each: each is told its own, once.
    let two = world(vec![
        script_pack(
            1,
            MAIN,
            r#"{ "module_name": "@minecraft/server", "version": "9.9.9" }"#,
            &[("scripts/main.js", "")],
        ),
        script_pack(
            2,
            MAIN,
            r#"{ "module_name": "@minecraft/server", "version": "8.8.8" }"#,
            &[("scripts/main.js", "")],
        ),
    ]);
    let report = two.load_all();
    let unavailable = report
        .iter()
        .filter(|f| f.code.name() == "ScriptModuleVersionUnavailable")
        .count();
    assert_eq!(unavailable, 2);
}

#[test]
fn s4_a_dependency_by_uuid_is_deprecated_and_stands_for_0_1_0() {
    // The UUID names a native module, not a pack: no pack of the world has it, and the pack
    // that depends on it is on the stack all the same.
    let set = world(vec![script_pack(
        1,
        MAIN,
        r#"{ "uuid": "b26a4d4c-afdf-4690-88f8-931846312678", "version": [0, 1, 0] }"#,
        &[("scripts/main.js", "")],
    )]);
    assert_eq!(set.behavior.packs().len(), 1);
    assert!(
        set.selection()
            .iter()
            .all(|finding| finding.code.name() != "MissingDependency")
    );
    assert_eq!(
        loaded(&set),
        [row(
            "DeprecatedMember",
            Severity::Warning,
            "manifest.json",
            "dependencies/0"
        )]
    );
    let plugin = set.behavior.scripts()[0].plugin.as_ref().expect("it runs");
    let resolved: Vec<(&str, String)> = plugin
        .dependencies
        .iter()
        .map(|d| (d.module_name.as_str(), d.resolved.to_string()))
        .collect();
    assert_eq!(resolved, [("@minecraft/server", "0.1.0".to_owned())]);
}

#[test]
fn s5_the_missing_main_file_is_one_error_with_the_file_that_differs_in_case() {
    let set = world(vec![script_pack(
        1,
        r#", "entry": "scripts/Main.js""#,
        SERVER,
        &[("scripts/main.js", "")],
    )]);
    assert_eq!(
        loaded(&set),
        [row(
            "UnresolvedReference",
            Severity::Error,
            "manifest.json",
            "modules/0/entry"
        )]
    );
    let bundle = &set.behavior.scripts()[0];
    let finding = bundle.findings().next().expect("the finding");
    assert_eq!(
        finding.code.to_string(),
        "Plugin [bp 1 - 1.0.0] - does not contain main file [Main.js]."
    );
    assert_eq!(
        finding.detail.as_deref(),
        Some(&DiagnosticDetail::Reference {
            kind: "script file",
            id: "Main.js".to_owned(),
            hint: Some(ReferenceHint::CaseMismatch("main.js".to_owned())),
        })
    );
}

#[test]
fn s6_an_import_of_a_file_the_bundle_does_not_have_is_told_by_how_it_is_imported() {
    let script = "import \"./gone.js\";\nimport(\"./late.js\");\nimport \"./\";\n";
    let set = world(vec![script_pack(
        1,
        MAIN,
        SERVER,
        &[("scripts/main.js", script)],
    )]);
    assert_eq!(
        loaded(&set),
        [
            // L8: a static import fails the plugin.
            row(
                "ScriptImportNotFound",
                Severity::Error,
                "scripts/main.js",
                ""
            ),
            // L9: a literal `import()` rejects one promise.
            row(
                "ScriptImportNotFound",
                Severity::Info,
                "scripts/main.js",
                ""
            ),
            // L21: vanilla's normaliser throws and the server terminates.
            row(
                "ScriptImportCrashesEngine",
                Severity::Error,
                "scripts/main.js",
                ""
            ),
        ]
    );
    // `.js` is appended and names are case-sensitive: `./Lib` is `Lib.js`, not `lib.js`.
    let cased = world(vec![script_pack(
        1,
        MAIN,
        SERVER,
        &[
            ("scripts/main.js", "import \"./lib\";\nimport \"./Lib\";\n"),
            ("scripts/lib.js", ""),
        ],
    )]);
    assert_eq!(
        loaded(&cased),
        [row(
            "ScriptImportNotFound",
            Severity::Error,
            "scripts/main.js",
            ""
        )]
    );
}

#[test]
fn s7_an_at_name_is_checked_against_the_modules_the_plugin_got() {
    let script = "import \"@minecraft/server\";\nimport \"@minecraft/server-ui\";\n\
        import(\"@minecraft/server-admin\");\nimport \"@minecraft/server-bindings\";\n";
    let set = world(vec![script_pack(
        1,
        MAIN,
        SERVER,
        &[("scripts/main.js", script)],
    )]);
    assert_eq!(
        loaded(&set),
        [
            // L10: static, then behind an `import()`.
            row(
                "ScriptModuleUnrecognized",
                Severity::Error,
                "scripts/main.js",
                ""
            ),
            row(
                "ScriptModuleUnrecognized",
                Severity::Info,
                "scripts/main.js",
                ""
            ),
            // L11.
            row(
                "ScriptModuleRestricted",
                Severity::Error,
                "scripts/main.js",
                ""
            ),
        ]
    );
}

/// The plugins of the world's behaviour stack: the way from a stack to the scripts that run.
fn plugins(set: &StackSet) -> Vec<&ScriptPlugin> {
    set.behavior
        .scripts()
        .iter()
        .filter_map(|bundle| bundle.plugin.as_ref())
        .collect()
}

/// A server configuration whose folders `folders` each hold a `permissions.json`.
fn config(folders: &[(&str, &str)]) -> ScriptConfig {
    ScriptConfig::parse(&ScriptConfigFiles {
        folders: folders
            .iter()
            .map(|(name, permissions)| {
                let mut folder = ConfigFolder::new(*name);
                folder.permissions = Some(bytes::Bytes::copy_from_slice(permissions.as_bytes()));
                folder
            })
            .collect(),
        ..ScriptConfigFiles::default()
    })
}

const ALLOW_SERVER: &str = r#"{"allowed_modules": ["@minecraft/server"]}"#;

#[test]
fn s8_a_config_folder_no_script_module_of_the_stack_uses_is_told_from_the_stack_s_plugins() {
    let set = world(vec![
        script_pack(1, MAIN, SERVER, &[("scripts/main.js", "")]),
        script_pack(2, MAIN, SERVER, &[("scripts/main.js", "")]),
        // A pack without a script module is no plugin: its folder matches nothing.
        super::super::common::behavior_pack(3, &[]),
    ]);
    let plugins = plugins(&set);
    assert_eq!(plugins.len(), 2);
    let own = module(1).to_string();
    let of_header = header(2).to_string();
    let upper = module(2).to_string().to_uppercase();
    let data_only = header(3).to_string();
    let config = config(&[
        ("default", ALLOW_SERVER),
        (&own, ALLOW_SERVER),
        (&of_header, ALLOW_SERVER),
        (&upper, ALLOW_SERVER),
        (&data_only, ALLOW_SERVER),
        ("notes", ALLOW_SERVER),
    ]);
    let found = config.unmatched_folders(plugins.iter().copied());
    let told: Vec<(&str, Severity, Phase, String, Option<Uuid>)> = found
        .iter()
        .map(|f| {
            let bedrock_addon::diagnostic::DiagnosticCode::UnmatchedConfigFolder {
                folder,
                expected,
            } = &f.code
            else {
                panic!("{f}");
            };
            assert_eq!(
                f.location
                    .path
                    .as_ref()
                    .map(|path| path.as_str().to_owned()),
                Some(format!("config/{folder}"))
            );
            (
                f.code.name(),
                f.severity,
                f.phase,
                folder.clone(),
                *expected,
            )
        })
        .collect();
    let mut wanted = vec![
        // D3: the pack's header UUID; the folder its module does use is named. Every unmatched
        // folder is an information of ours.
        (
            "UnmatchedConfigFolder",
            Severity::Info,
            Phase::Link,
            of_header.clone(),
            Some(module(2)),
        ),
        // D4: the module's UUID in another spelling.
        (
            "UnmatchedConfigFolder",
            Severity::Info,
            Phase::Link,
            upper.clone(),
            Some(module(2)),
        ),
        // D5: nothing of the stack.
        (
            "UnmatchedConfigFolder",
            Severity::Info,
            Phase::Link,
            data_only.clone(),
            None,
        ),
        (
            "UnmatchedConfigFolder",
            Severity::Info,
            Phase::Link,
            "notes".to_owned(),
            None,
        ),
    ];
    // Folders are told in name order.
    wanted.sort_by(|a, b| a.3.cmp(&b.3));
    assert_eq!(told, wanted);
    // `default` and the folder a module uses are no finding; nor is anything of this a
    // finding of `link()`, which holds no configuration.
    assert!(
        set.link()
            .iter()
            .all(|finding| finding.code.name() != "UnmatchedConfigFolder")
    );
}

#[test]
fn s9_a_module_the_configuration_does_not_allow_is_refused_once_by_the_configuration() {
    let set = world(vec![script_pack(
        1,
        MAIN,
        r#"{ "module_name": "@minecraft/server", "version": "2.0.0" },
           { "module_name": "@minecraft/server-ui", "version": "2.0.0" }"#,
        &[("scripts/main.js", "")],
    )]);
    // The stack is discovered without a list: nothing is refused there.
    assert_eq!(loaded(&set), []);
    let plugins = plugins(&set);
    let config = config(&[("default", ALLOW_SERVER)]);
    let verdicts = config.filter_stack(plugins.iter().copied());
    assert_eq!(verdicts.len(), 1);
    assert!(!verdicts[0].runs);
    let found: Vec<(&str, Severity, String)> = verdicts[0]
        .diagnostics
        .iter()
        .map(|f| (f.code.name(), f.severity, f.code.to_string()))
        .collect();
    assert_eq!(
        found,
        [(
            "ScriptModulePermission",
            Severity::Error,
            "Plugin [bp 1 - 1.0.0] - requesting dependency on module [@minecraft/server-ui] \
             but it is not configured to use it."
                .to_owned()
        )]
    );
    // Names are compared byte for byte, and a near miss in the list is the list's own
    // information when it is read (M4; the reader's notes are ours).
    let near = self::config(&[(
        "default",
        r#"{"allowed_modules": ["@minecraft/server", "@Minecraft/Server-UI"]}"#,
    )]);
    assert!(!near.filter(plugins[0]).runs);
    let read: Vec<(&str, Severity, String)> = near
        .diagnostics()
        .iter()
        .map(|f| (f.code.name(), f.severity, f.location.json_path.to_string()))
        .collect();
    assert_eq!(
        read,
        [(
            "InvalidValue",
            Severity::Info,
            "allowed_modules/1".to_owned()
        )]
    );
}

#[test]
fn s10_a_permission_level_for_no_command_is_a_note_of_the_file_s_own_reading() {
    let config = ScriptConfig::parse(&ScriptConfigFiles {
        commands: Some(bytes::Bytes::from_static(
            br#"{"permission_levels": {"give": "admin", "nosuchcommand": "any"}}"#,
        )),
        ..ScriptConfigFiles::default()
    });
    let found: Vec<Told> = told_of(config.diagnostics());
    assert_eq!(
        found,
        [row(
            "UnresolvedReference",
            Severity::Info,
            "config/commands.json",
            "permission_levels/nosuchcommand"
        )]
    );
    let finding = config.diagnostics().iter().next().expect("the note");
    assert_ne!(finding.phase, Phase::Link);
    assert_eq!(
        finding.detail.as_deref(),
        Some(&DiagnosticDetail::Reference {
            kind: "command",
            id: "nosuchcommand".to_owned(),
            hint: None,
        })
    );
}

#[test]
fn s4_a_dependency_on_a_uuid_that_is_no_native_module_is_a_pack_dependency() {
    // Either of the two UUIDs vanilla knows stands for a native module; any other UUID is a
    // pack's, and a pack no one has hides the pack that depends on it.
    let hidden = |uuid: &str| {
        let dependency = format!(r#"{{ "uuid": "{uuid}", "version": [0, 1, 0] }}"#);
        let set = world(vec![script_pack(
            1,
            MAIN,
            &dependency,
            &[("scripts/main.js", "")],
        )]);
        let missing = set
            .selection()
            .iter()
            .filter(|finding| finding.code.name() == "MissingDependency")
            .count();
        (set.behavior.packs().len(), missing)
    };
    assert_eq!(hidden("2bd50a27-ab5f-4f40-a596-3641627c635e"), (1, 0));
    assert_eq!(hidden("b26a4d4c-afdf-4690-88f8-931846312679"), (0, 1));
    assert_eq!(hidden("00000000-0000-4000-8000-0000000000aa"), (0, 1));
}
