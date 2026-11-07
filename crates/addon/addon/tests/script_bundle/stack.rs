//! `scripts()` on both stacks: which packs have a bundle, the side that decides the module
//! type, base-game packs, subpacks, the cache, and the same bundles through a `StackLoader`.

use bedrock_addon::diagnostic::{Code, Severity};
use bedrock_addon::families::script::{Resolution, ScriptPath};
use bedrock_addon::load::{
    Collection, CollectionSet, LoadOptions, RepositoryLoader, Residency, RootId, Source,
    StackLoader, StackOptions,
};
use bedrock_addon::pack::{
    Behavior, OpenOptions, PackOrigin, PackSide, PackStack, Resource, StackSet, WorldPacks,
};

use super::common::{pack_of, script_manifest, script_stack};
use super::responder::{Order, Responder};

/// A manifest named `name` with the modules `modules` (`(type, entry)`), its UUIDs from `n`.
fn manifest(name: &str, n: u128, modules: &[(&str, Option<&str>)], rest: &str) -> Vec<u8> {
    let modules: Vec<String> = modules
        .iter()
        .enumerate()
        .map(|(index, (module_type, entry))| {
            let entry = entry.map_or(String::new(), |entry| format!(r#""entry": "{entry}", "#));
            format!(
                r#"{{ "type": "{module_type}", {entry}"uuid": "{}", "version": [1, 0, 0] }}"#,
                uuid::Uuid::from_u128(n + 1_000_000 + index as u128)
            )
        })
        .collect();
    format!(
        r#"{{
    "format_version": 2,
    "header": {{ "name": "{name}", "description": "", "uuid": "{}", "version": [1, 0, 0],
        "min_engine_version": [1, 21, 0] }},
    "modules": [{}]{rest}
}}"#,
        uuid::Uuid::from_u128(n),
        modules.join(", "),
    )
    .into_bytes()
}

fn files(manifest: Vec<u8>, files: &[(&str, &[u8])]) -> Vec<(String, Vec<u8>)> {
    let mut all = vec![("manifest.json".to_owned(), manifest)];
    all.extend(files.iter().map(|(p, b)| ((*p).to_owned(), b.to_vec())));
    all
}

#[test]
fn a_pack_without_a_module_and_without_script_files_has_no_bundle() {
    let plain = pack_of(
        files(
            manifest("plain", 1, &[("data", None)], ""),
            &[
                ("functions/a.mcfunction", b"say hi\n"),
                ("src/tool.js", b""),
            ],
        ),
        PackOrigin::User,
    );
    let with_folder = pack_of(
        files(
            manifest("folder", 2, &[("data", None)], ""),
            &[("scripts/notes.txt", b"n")],
        ),
        PackOrigin::User,
    );
    let with_module = pack_of(
        files(
            manifest("module", 3, &[("script", Some("scripts/main.js"))], ""),
            &[],
        ),
        PackOrigin::User,
    );
    let stack =
        PackStack::<Behavior>::of([plain, with_folder, with_module], LoadOptions::default())
            .expect("behaviour packs");
    let bundles = stack.scripts();
    let names: Vec<&str> = bundles.iter().map(|b| b.pack.name()).collect();
    // A file below `scripts/` or a script module makes a bundle; a `.js` file elsewhere in a
    // pack without a module does not.
    assert_eq!(names, ["folder", "module"]);
    assert_eq!(bundles[0].files.len(), 0);
    assert_eq!(bundles[0].others.len(), 1);
    // No script file, no module: nothing to say.
    assert_eq!(bundles[0].findings().count(), 0);
    // A module whose main file is not there.
    assert_eq!(bundles[1].findings().count(), 1);
    assert_eq!(bundles[1].side, PackSide::Behavior);
}

#[test]
fn l20_a_pack_of_the_game_without_a_script_module_is_not_told_that_its_scripts_never_load() {
    // The layout of the game's library packs (`server_library`, `server_ui_library`,
    // `server_editor_library`): a `data` module and the wrapper files below `scripts/`.
    let told_of = |origin: PackOrigin| {
        let library = pack_of(
            files(
                manifest("library", 7, &[("data", None)], ""),
                &[
                    ("scripts/server-1.0.0.js", b"export {};"),
                    ("scripts/server-beta.js", b"export {};"),
                ],
            ),
            origin,
        );
        let stack =
            PackStack::<Behavior>::of([library], LoadOptions::default()).expect("a behaviour pack");
        let bundles = stack.scripts();
        assert_eq!(bundles.len(), 1, "{origin:?}");
        let bundle = &bundles[0];
        assert!(
            bundle.plugin.is_none() && bundle.graph.is_none(),
            "{origin:?}"
        );
        let codes: Vec<(&str, Severity)> = bundle
            .findings()
            .map(|finding| (finding.code.name(), finding.severity))
            .collect();
        (bundle.files.len(), codes)
    };
    // The engine reads the scripts of its own library packs itself, by the pack's folder name:
    // the bundle lists them, and nothing says they can never be loaded.
    assert_eq!(told_of(PackOrigin::Package), (2, vec![]));
    // In a pack that is not the game's nothing ever reads them (L20): one note for the pack.
    // `Dev` and the other origins are no base-game pack either (vanilla's `isBaseGamePack`
    // is the app's own package alone).
    for origin in [
        PackOrigin::User,
        PackOrigin::World,
        PackOrigin::Dev,
        PackOrigin::Unknown,
        PackOrigin::Treatment,
        PackOrigin::PremiumCache,
    ] {
        assert_eq!(
            told_of(origin),
            (2, vec![("ScriptFileNotLoadable", Severity::Info)]),
            "{origin:?}"
        );
    }
}

#[test]
fn the_accessor_loads_once() {
    let stack = script_stack(&[("scripts/main.js", b"")]);
    assert!(std::ptr::eq(stack.scripts(), stack.scripts()));
}

#[test]
fn a_resource_pack_s_bundle_is_run_by_its_client_script_module() {
    let scripts: [(&str, &[u8]); 2] = [
        ("scripts/client.js", b"import \"./ui.js\";\n"),
        ("scripts/ui.js", b""),
    ];
    let client = pack_of(
        files(
            manifest(
                "client",
                5,
                &[
                    ("resources", None),
                    ("client_script", Some("scripts/client.js")),
                ],
                "",
            ),
            &scripts,
        ),
        PackOrigin::User,
    );
    // A resource pack without the module: its scripts run nowhere.
    let plain = pack_of(
        files(manifest("plain", 6, &[("resources", None)], ""), &scripts),
        PackOrigin::User,
    );
    let stack =
        PackStack::<Resource>::of([client, plain], LoadOptions::default()).expect("resource packs");
    let bundles = stack.scripts();
    assert_eq!(bundles.len(), 2);
    assert_eq!(bundles[0].side, PackSide::Resource);
    let entry = bundles[0].entry.as_ref().expect("the client entry");
    assert_eq!(entry.main.as_str(), "client.js");
    assert_eq!(bundles[0].graph.as_ref().unwrap().reachable_static.len(), 2);
    assert_eq!(bundles[0].findings().count(), 0);
    assert_eq!(bundles[1].entry, None);
    assert_eq!(bundles[1].side, PackSide::Resource);
    let codes: Vec<&str> = bundles[1].findings().map(|f| f.code.name()).collect();
    assert_eq!(codes, ["ScriptFileNotLoadable"]);
}

#[test]
fn a_client_script_module_is_checked_as_one_that_runs_whatever_the_stack_enables() {
    // Whether a client starts a `client_script` module is a setting of that client (an
    // option, the Beta APIs experiment or a client flag), which no file of the pack holds:
    // the bundle says what happens when it runs, with the Beta APIs on or off.
    let findings = |beta_apis: bool| -> Vec<(&'static str, Severity)> {
        let client = pack_of(
            files(
                manifest(
                    "client",
                    5,
                    &[
                        ("resources", None),
                        ("client_script", Some("scripts/client.js")),
                    ],
                    "",
                ),
                &[("scripts/client.js", b"import \"./gone.js\";\n")],
            ),
            PackOrigin::User,
        );
        let options = LoadOptions {
            beta_apis,
            ..LoadOptions::default()
        };
        let stack = PackStack::<Resource>::of([client], options).expect("a resource pack");
        let bundle = &stack.scripts()[0];
        assert!(bundle.graph.is_some());
        bundle
            .findings()
            .map(|f| (f.code.name(), f.severity))
            .collect()
    };
    let wanted = [("ScriptImportNotFound", Severity::Error)];
    assert_eq!(findings(false), wanted);
    assert_eq!(findings(true), wanted);
}

#[test]
fn l15_a_js_file_outside_scripts_is_a_note_in_a_pack_whose_module_has_another_language() {
    // L5 drops the plugin before it looks at the files; a `.js` file outside `scripts/` is
    // no less unloadable for that (L15).
    let lua = String::from_utf8(manifest(
        "lua",
        8,
        &[("script", Some("scripts/main.js"))],
        "",
    ))
    .expect("a manifest")
    .replace(
        r#""type": "script", "#,
        r#""type": "script", "language": "lua", "#,
    );
    let pack = pack_of(
        files(
            lua.into_bytes(),
            &[
                ("scripts/main.js", b""),
                ("src/a.js", b""),
                ("b.js", b""),
                ("src/c.JS", b""),
            ],
        ),
        PackOrigin::User,
    );
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default()).expect("a pack");
    let bundle = &stack.scripts()[0];
    let found: Vec<(&str, Severity, Option<&str>)> = bundle
        .findings()
        .map(|f| {
            (
                f.code.name(),
                f.severity,
                f.location
                    .path
                    .as_ref()
                    .map(bedrock_addon::storage::PackPath::as_str),
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            ("InvalidEnumValue", Severity::Error, Some("manifest.json")),
            ("ScriptFileNotLoadable", Severity::Info, Some("b.js")),
            ("ScriptFileNotLoadable", Severity::Info, Some("src/a.js")),
        ]
    );
    assert_eq!(bundle.entry, None);
    assert_eq!(bundle.graph, None);
}

/// The native imports of a pack: an undeclared module, a bindings module, and one a declared
/// module brings.
const NATIVES: &[u8] = b"import \"@minecraft/server-editor-private-bindings\";\n\
    import \"@minecraft/server-bindings\";\nimport \"@minecraft/common\";\n";

#[test]
fn a_base_game_pack_imports_the_bindings_a_world_s_pack_may_not() {
    let pack = |origin| {
        let all = files(
            script_manifest("natives", 9, "scripts/main.js"),
            &[("scripts/main.js", NATIVES)],
        );
        let stack = PackStack::<Behavior>::of([pack_of(all, origin)], LoadOptions::default())
            .expect("a behaviour pack");
        let codes: Vec<&'static str> = stack.scripts()[0]
            .findings()
            .map(|finding| finding.code.name())
            .collect();
        codes
    };
    assert_eq!(pack(PackOrigin::Package), Vec::<&str>::new());
    assert_eq!(
        pack(PackOrigin::User),
        ["ScriptModuleUnrecognized", "ScriptModuleRestricted"]
    );
}

/// A world with one behaviour pack at root 1 that has a subpack `sp`, selected by the world,
/// with `extra` files (path below root 1, bytes) in the pack too.
fn world_with_subpack_and(extra: &[(&str, &[u8])]) -> (Responder, WorldPacks) {
    let subpacks = r#",
    "dependencies": [{ "module_name": "@minecraft/server", "version": "2.0.0" }],
    "subpacks": [{ "folder_name": "sp", "name": "sp", "memory_tier": 0 }]"#;
    let mut responder = Responder::new(Order::Issue);
    responder
        .file(
            1,
            "bp/manifest.json",
            manifest(
                "subpacked",
                1,
                &[("data", None), ("script", Some("scripts/main.js"))],
                subpacks,
            ),
        )
        .file(
            1,
            "bp/scripts/main.js",
            &b"import \"./lib.js\";\nimport \"./only_sub.js\";\n"[..],
        )
        .file(1, "bp/scripts/lib.js", &b"import \"./gone.js\";\n"[..])
        .file(1, "bp/subpacks/sp/scripts/lib.js", &b"export {};\n"[..])
        .file(
            1,
            "bp/subpacks/sp/scripts/only_sub.js",
            &b"export {};\n"[..],
        )
        .file(
            1,
            "bp/subpacks/other/scripts/never.js",
            &b"export {};\n"[..],
        );
    for (path, bytes) in extra {
        responder.file(1, path, bytes.to_vec());
    }
    let entry = format!(
        r#"[{{"pack_id": "{}", "version": [1, 0, 0], "subpack": "sp"}}]"#,
        uuid::Uuid::from_u128(1)
    );
    let world = WorldPacks::parse(Some(entry.as_bytes()), None).unwrap();
    (responder, world)
}

/// The stacks of the world, the collections `collections` loaded by a `StackLoader` in
/// `order`, every file of the packs kept resident.
fn loaded(order: Order, collections: CollectionSet) -> StackSet {
    loaded_with(order, collections, &[])
}

/// [`loaded`] with `extra` files in the pack.
fn loaded_with(order: Order, collections: CollectionSet, extra: &[(&str, &[u8])]) -> StackSet {
    let (mut responder, world) = world_with_subpack_and(extra);
    let mut repository = RepositoryLoader::new(
        vec![Source::Directory {
            root: RootId(1),
            origin: PackOrigin::World,
        }],
        OpenOptions::default(),
    );
    let repository = responder.drive(&mut repository);
    let set = repository.stacks(&world, LoadOptions::default());
    let mut responder = responder.reordered(order);
    let options = StackOptions {
        collections,
        behavior: Residency::All,
        resource: Residency::All,
        ..StackOptions::default()
    };
    let mut loader = StackLoader::new(set, options);
    responder.drive(&mut loader)
}

#[test]
fn a_subpack_s_script_stands_in_for_the_pack_s_file_of_its_name() {
    let set = loaded(Order::Issue, CollectionSet::only([Collection::Scripts]));
    let bundles = set.behavior.scripts();
    assert_eq!(bundles.len(), 1);
    let bundle = &bundles[0];
    let names: Vec<&str> = bundle.files.keys().map(ScriptPath::as_str).collect();
    // The names are the keys: no subpack folder, and no file of a subpack that is not active.
    assert_eq!(names, ["lib.js", "main.js", "only_sub.js"]);
    let lib = &bundle.files[&ScriptPath::new("lib.js")];
    assert_eq!(lib.path.as_str(), "subpacks/sp/scripts/lib.js");
    // The subpack's `lib.js` imports nothing: the pack's own, which imports a missing file,
    // is not read.
    assert!(lib.imports().is_empty());
    // The one finding: the script of a subpack folder the manifest does not declare is a
    // `.js` file outside `scripts/` (L15).
    let found: Vec<(&str, Option<&str>)> = bundle
        .findings()
        .map(|f| {
            (
                f.code.name(),
                f.location
                    .path
                    .as_ref()
                    .map(bedrock_addon::storage::PackPath::as_str),
            )
        })
        .collect();
    assert_eq!(
        found,
        [(
            "ScriptFileNotLoadable",
            Some("subpacks/other/scripts/never.js")
        )]
    );
    assert_eq!(
        bundle.main().unwrap().imports()[1].resolution,
        Resolution::File(ScriptPath::new("only_sub.js"))
    );
}

#[test]
fn a_stack_loader_gives_the_bundles_of_the_lazy_accessor_in_every_order() {
    let lazy = loaded(Order::Issue, CollectionSet::only([]));
    let reference = format!("{:?}", lazy.behavior.scripts());
    assert!(reference.contains("only_sub.js"));
    for order in Order::ALL {
        let set = loaded(order, CollectionSet::only([Collection::Scripts]));
        assert_eq!(
            format!("{:?}", set.behavior.scripts()),
            reference,
            "{order:?}"
        );
        assert_eq!(set.resource.scripts().len(), 0);
    }
}

/// A `manifest.json` of the active subpack stands in for the pack's by its key, as every
/// subpack file does, and stage 1 reads the file the walk handed over: the subpack's bytes,
/// placed at the subpack's path. (Whether the game reads a subpack's manifest at all is an
/// open question; this pins what the loader did before it read the opened manifest.)
#[test]
fn a_subpack_s_manifest_stands_in_for_the_pack_s_and_stage_1_reads_its_bytes() {
    let sub_manifest = manifest(
        "subpacked",
        1,
        &[("data", None), ("script", Some("scripts/main.js"))],
        r#",
    "dependencies": [{ "module_name": "@minecraft/server", "version": "2.0.0", "optional": true }]"#,
    );
    let set = loaded_with(
        Order::Issue,
        CollectionSet::only([Collection::Scripts]),
        &[("bp/subpacks/sp/manifest.json", &sub_manifest)],
    );
    let bundles = set.behavior.scripts();
    assert_eq!(bundles.len(), 1);
    let found: Vec<(&str, Option<&str>, String)> = bundles[0]
        .findings()
        .filter(|f| f.code.name() == "IgnoredMember")
        .map(|f| {
            (
                f.code.name(),
                f.location
                    .path
                    .as_ref()
                    .map(bedrock_addon::storage::PackPath::as_str),
                f.location.json_path.to_string(),
            )
        })
        .collect();
    assert_eq!(
        found,
        [(
            "IgnoredMember",
            Some("subpacks/sp/manifest.json"),
            "dependencies/0/optional".to_owned()
        )]
    );
}

#[test]
fn the_stack_s_report_lists_a_bundle_s_findings() {
    let stack = script_stack(&[("scripts/main.js", b"import \"./gone.js\";\n")]);
    assert_eq!(stack.scripts().len(), 1);
    let report = stack.report();
    let found: Vec<&Code> = report
        .iter()
        .map(|finding| &finding.code)
        .filter(|code| code.is(&bedrock_addon::diagnostic::codes::SCRIPT_IMPORT_NOT_FOUND))
        .collect();
    assert_eq!(found.len(), 1);
    assert!(report.has_errors());
}

// ---- the join with the script module: one module per pack, read once, each rule once ----

/// A format-2 manifest of a behaviour pack `name` (its UUIDs from `n`) with the `modules`
/// and `dependencies` written as JSON.
fn manifest_of(name: &str, n: u128, modules: &str, dependencies: &str) -> Vec<u8> {
    format!(
        r#"{{
    "format_version": 2,
    "header": {{ "name": "{name}", "description": "", "uuid": "{}", "version": [1, 0, 0],
        "min_engine_version": [1, 21, 0] }},
    "modules": [{modules}],
    "dependencies": [{dependencies}]
}}"#,
        uuid::Uuid::from_u128(n),
    )
    .into_bytes()
}

/// One script module as JSON: its UUID from `n`, `rest` after the version (`"entry": …`).
fn script_module(n: u128, rest: &str) -> String {
    format!(
        r#"{{ "type": "script", "uuid": "{}", "version": [1, 0, 0]{rest} }}"#,
        uuid::Uuid::from_u128(n)
    )
}

const MAIN_JS: &str = r#", "entry": "scripts/main.js""#;
const SERVER_2: &str = r#"{ "module_name": "@minecraft/server", "version": "2.0.0" }"#;

/// One pack of a test stack: its manifest and its other files.
type PackFiles<'a> = (Vec<u8>, Vec<(&'a str, &'a [u8])>);

/// The behaviour stack of `packs` (each a manifest and its files), the first on top.
fn stack_with(packs: Vec<PackFiles<'_>>, options: LoadOptions) -> PackStack<Behavior> {
    let packs = packs
        .into_iter()
        .map(|(manifest, others)| pack_of(files(manifest, &others), PackOrigin::User));
    PackStack::<Behavior>::of(packs, options).expect("behaviour packs")
}

/// One finding as the join tests compare it: code, severity, file, JSON path.
type Told = (&'static str, Severity, String, String);

fn told(bundle: &bedrock_addon::families::script::ScriptBundle) -> Vec<Told> {
    bundle
        .findings()
        .map(|f| {
            assert_eq!(f.location.pack.as_ref(), Some(&bundle.pack), "{f:?}");
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

fn finding(code: &'static str, severity: Severity, path: &str) -> Told {
    (code, severity, "manifest.json".to_owned(), path.to_owned())
}

#[test]
fn m8_a_legacy_optional_is_told_through_scripts() {
    let dependencies =
        r#"{ "module_name": "@minecraft/server", "version": "2.0.0", "optional": true }"#;
    let stack = stack_with(
        vec![(
            manifest_of("opt", 20, &script_module(21, MAIN_JS), dependencies),
            vec![("scripts/main.js", b"")],
        )],
        LoadOptions::default(),
    );
    let bundle = &stack.scripts()[0];
    assert_eq!(
        told(bundle),
        [finding(
            "IgnoredMember",
            Severity::Info,
            "dependencies/0/optional"
        )]
    );
    assert!(bundle.graph.is_some());
}

#[test]
fn m14_the_relaxed_form_in_a_pack_for_an_older_engine_is_told_through_scripts() {
    let manifest = |min_engine: &str| {
        format!(
            r#"{{
    "format_version": 3,
    "header": {{ "name": "relaxed", "description": "", "uuid": "{}", "version": "1.0.0",
        "min_engine_version": "{min_engine}" }},
    "modules": [{{ "type": "script", "uuid": "{}", "version": "1.0.0",
        "entry": "scripts/main.js" }}],
    "dependencies": [{{ "module_name": "@minecraft/server", "version": "2.0" }}]
}}"#,
            uuid::Uuid::from_u128(30),
            uuid::Uuid::from_u128(31),
        )
        .into_bytes()
    };
    let load = |min_engine: &str| {
        let stack = stack_with(
            vec![(manifest(min_engine), vec![("scripts/main.js", b"")])],
            LoadOptions::default(),
        );
        let bundle = &stack.scripts()[0];
        assert!(bundle.graph.is_some(), "{min_engine}");
        told(bundle)
    };
    assert_eq!(
        load("1.26.29"),
        [finding(
            "FieldSinceEngine",
            Severity::Info,
            "dependencies/0/version"
        )]
    );
    assert_eq!(load("1.26.30"), []);
}

#[test]
fn s1_a_later_script_module_is_told_once_at_the_module() {
    let modules = format!(
        "{}, {}",
        script_module(41, MAIN_JS),
        script_module(42, r#", "entry": "scripts/second.js""#)
    );
    let stack = stack_with(
        vec![(
            manifest_of("two", 40, &modules, SERVER_2),
            vec![("scripts/main.js", b""), ("scripts/second.js", b"")],
        )],
        LoadOptions::default(),
    );
    let bundle = &stack.scripts()[0];
    assert_eq!(
        told(bundle),
        [finding(
            "MultipleScriptModules",
            Severity::Warning,
            "modules/1"
        )]
    );
    assert!(bundle.graph.is_some());
}

#[test]
fn s3_another_language_is_told_once_at_the_member() {
    let module = script_module(51, r#", "language": "lua", "entry": "scripts/main.js""#);
    let stack = stack_with(
        vec![(
            manifest_of("lua", 50, &module, SERVER_2),
            vec![("scripts/main.js", b"")],
        )],
        LoadOptions::default(),
    );
    let bundle = &stack.scripts()[0];
    assert_eq!(
        told(bundle),
        [finding(
            "InvalidEnumValue",
            Severity::Error,
            "modules/0/language"
        )]
    );
    assert_eq!((bundle.entry.as_ref(), bundle.graph.as_ref()), (None, None));
}

#[test]
fn s4_an_entry_that_is_absent_or_no_js_file_is_told_once() {
    let load = |rest: &str| {
        let stack = stack_with(
            vec![(
                manifest_of("entry", 60, &script_module(61, rest), SERVER_2),
                vec![("scripts/main.js", b""), ("scripts/main.ts", b"")],
            )],
            LoadOptions::default(),
        );
        let bundle = &stack.scripts()[0];
        assert_eq!(bundle.graph, None);
        told(bundle)
    };
    assert_eq!(
        load(""),
        [finding(
            "MissingRequiredField",
            Severity::Error,
            "modules/0"
        )]
    );
    assert_eq!(
        load(r#", "entry": "scripts/main.ts""#),
        [finding("InvalidValue", Severity::Error, "modules/0/entry")]
    );
}

#[test]
fn s5_a_missing_main_file_is_told_once_with_the_file_that_differs_in_case() {
    use bedrock_addon::diagnostic::{DiagnosticDetail, ReferenceHint};
    let load = |entry: &str| {
        let rest = format!(r#", "entry": "{entry}""#);
        let stack = stack_with(
            vec![(
                manifest_of("main", 70, &script_module(71, &rest), SERVER_2),
                vec![("scripts/main.js", b"")],
            )],
            LoadOptions::default(),
        );
        let bundle = &stack.scripts()[0];
        assert_eq!(
            told(bundle),
            [finding(
                "UnresolvedReference",
                Severity::Error,
                "modules/0/entry"
            )]
        );
        assert_eq!(bundle.graph, None);
        let found = bundle.findings().next().expect("the finding");
        assert_eq!(
            found.code.to_string(),
            format!(
                "Plugin [main - 1.0.0] - does not contain main file [{}].",
                entry.trim_start_matches("scripts/")
            )
        );
        found.detail.as_deref().cloned()
    };
    assert_eq!(
        load("scripts/Main.js"),
        Some(DiagnosticDetail::Reference {
            kind: "script file",
            id: "Main.js".to_owned(),
            hint: Some(ReferenceHint::CaseMismatch("main.js".to_owned())),
        })
    );
    assert_eq!(
        load("scripts/gone.js"),
        Some(DiagnosticDetail::Reference {
            kind: "script file",
            id: "gone.js".to_owned(),
            hint: None,
        })
    );
}

#[test]
fn s2_one_discovery_serves_the_stack_and_skips_a_module_a_higher_pack_brought() {
    let pack = |name: &str, n: u128| {
        (
            manifest_of(name, n, &script_module(81, MAIN_JS), SERVER_2),
            vec![("scripts/main.js", &b""[..])],
        )
    };
    let stack = stack_with(
        vec![pack("upper", 80), pack("lower", 82), pack("lowest", 83)],
        LoadOptions::default(),
    );
    let bundles = stack.scripts();
    assert_eq!(told(&bundles[0]), []);
    assert!(bundles[0].graph.is_some());
    for lower in &bundles[1..] {
        assert_eq!(
            told(lower),
            [finding(
                "DuplicateIdentifier",
                Severity::Warning,
                "modules/0"
            )]
        );
        assert_eq!(lower.graph, None);
    }
}

#[test]
fn the_versions_a_pack_asks_for_are_resolved_through_scripts() {
    let load = |dependencies: &str, beta_apis: bool| {
        let stack = stack_with(
            vec![(
                manifest_of("deps", 90, &script_module(91, MAIN_JS), dependencies),
                vec![("scripts/main.js", b"")],
            )],
            LoadOptions {
                beta_apis,
                ..LoadOptions::default()
            },
        );
        let bundle = &stack.scripts()[0];
        (told(bundle), bundle.graph.is_some())
    };
    // S3: a version no engine serves (script_module R1).
    assert_eq!(
        load(
            r#"{ "module_name": "@minecraft/server", "version": "9.9.9" }"#,
            false
        ),
        (
            vec![finding(
                "ScriptModuleVersionUnavailable",
                Severity::Error,
                "dependencies/0"
            )],
            false
        )
    );
    // Two majors of one module (R2).
    let conflict = load(
        r#"{ "module_name": "@minecraft/server", "version": "1.5.0" },
           { "module_name": "@minecraft/server", "version": "2.0.0" }"#,
        false,
    );
    assert_eq!(
        conflict,
        (
            vec![finding(
                "ScriptModuleVersionConflict",
                Severity::Error,
                "dependencies/1"
            )],
            false
        )
    );
    // The Beta APIs gate reads the stack's toggle (S6).
    let beta = r#"{ "module_name": "@minecraft/server", "version": "beta" }"#;
    assert_eq!(
        load(beta, false),
        (
            vec![finding("MissingToggles", Severity::Error, "dependencies/0")],
            false
        )
    );
    assert_eq!(load(beta, true), (Vec::new(), true));
    // S2 of 20: a module no engine has (S11).
    assert_eq!(
        load(
            r#"{ "module_name": "@minecraft/nope", "version": "1.0.0" }"#,
            false
        ),
        (
            vec![finding(
                "UnresolvedReference",
                Severity::Error,
                "dependencies/0"
            )],
            false
        )
    );
}

#[test]
fn a_dependency_discovery_dropped_is_no_module_of_the_script() {
    // A format-3 optional dependency on a beta version, the Beta APIs off: dropped, and the
    // script runs without it; its import of the module then fails at run time.
    let manifest = format!(
        r#"{{
    "format_version": 3,
    "header": {{ "name": "dropped", "description": "", "uuid": "{}", "version": "1.0.0",
        "min_engine_version": "1.26.30" }},
    "modules": [{{ "type": "script", "uuid": "{}", "version": "1.0.0",
        "entry": "scripts/main.js" }}],
    "dependencies": [
        {{ "module_name": "@minecraft/server", "version": "2.0.0" }},
        {{ "module_name": "@minecraft/server-ui", "version": "beta", "optional": true }}
    ]
}}"#,
        uuid::Uuid::from_u128(100),
        uuid::Uuid::from_u128(101),
    )
    .into_bytes();
    let script: &[u8] = b"import \"@minecraft/server\";\nimport(\"@minecraft/server-ui\");\n";
    let stack = stack_with(
        vec![(manifest, vec![("scripts/main.js", script)])],
        LoadOptions::default(),
    );
    let bundle = &stack.scripts()[0];
    let found: Vec<(&str, Severity, String)> = told(bundle)
        .into_iter()
        .map(|(code, severity, file, _)| (code, severity, file))
        .collect();
    assert_eq!(
        found,
        [
            ("DroppedField", Severity::Info, "manifest.json".to_owned()),
            (
                "ScriptModuleUnrecognized",
                Severity::Info,
                "scripts/main.js".to_owned()
            ),
        ]
    );
}

/// A world of three behaviour packs under root 1: `run` with a script module that starts,
/// `lua` whose module vanilla drops for its language, and `lib` with a data module alone.
fn world_of_three() -> (Responder, WorldPacks) {
    let mut responder = Responder::new(Order::Issue);
    responder
        .file(
            1,
            "run/manifest.json",
            manifest_of("run", 110, &script_module(111, MAIN_JS), SERVER_2),
        )
        .file(1, "run/scripts/main.js", &b"import \"./used.js\";\n"[..])
        .file(1, "run/scripts/used.js", &b"export {};\n"[..])
        .file(1, "run/scripts/unused.js", &b"export {};\n"[..])
        .file(
            1,
            "lua/manifest.json",
            manifest_of(
                "lua",
                120,
                &script_module(121, r#", "language": "lua", "entry": "scripts/main.js""#),
                SERVER_2,
            ),
        )
        .file(1, "lua/scripts/main.js", &b"import \"./gone.js\";\n"[..])
        .file(1, "lua/tool.js", &b""[..])
        .file(
            1,
            "lib/manifest.json",
            manifest("lib", 130, &[("data", None)], ""),
        )
        .file(1, "lib/scripts/a.js", &b"import \"./gone.js\";\n"[..])
        .file(1, "lib/scripts/b.js", &b"\xff"[..]);
    let entries: Vec<String> = [110u128, 120, 130]
        .iter()
        .map(|n| {
            format!(
                r#"{{"pack_id": "{}", "version": [1, 0, 0]}}"#,
                uuid::Uuid::from_u128(*n)
            )
        })
        .collect();
    let world = format!("[{}]", entries.join(", "));
    (
        responder,
        WorldPacks::parse(Some(world.as_bytes()), None).unwrap(),
    )
}

#[test]
fn a_load_asks_for_the_scripts_of_a_pack_whose_module_runs_and_of_no_other() {
    let (mut responder, world) = world_of_three();
    let mut repository = RepositoryLoader::new(
        vec![Source::Directory {
            root: RootId(1),
            origin: PackOrigin::World,
        }],
        OpenOptions::default(),
    );
    let repository = responder.drive(&mut repository);
    let set = repository.stacks(&world, LoadOptions::default());
    let opened = responder;
    let mut responder = opened.reordered(Order::Issue);
    let mut loader = StackLoader::new(set, StackOptions::default());
    let set = responder.drive(&mut loader);
    let mut scripts_read: Vec<String> = [&opened, &responder]
        .iter()
        .flat_map(|responder| &responder.log.requests)
        .filter_map(|request| match request {
            bedrock_addon::load::Request::ReadFile { location, .. } => Some(location.to_string()),
            bedrock_addon::load::Request::ListDir { .. } => None,
        })
        .filter(|location| {
            std::path::Path::new(location)
                .extension()
                .is_some_and(|extension| extension == "js")
        })
        .collect();
    scripts_read.sort();
    assert_eq!(scripts_read.len(), 3, "{scripts_read:?}");
    for (read, wanted) in scripts_read.iter().zip([
        "run/scripts/main.js",
        "run/scripts/unused.js",
        "run/scripts/used.js",
    ]) {
        assert!(read.ends_with(wanted), "{read} is not {wanted}");
    }
    // What needs no byte is still told: the members by name, L15 and L20.
    let bundles = set.behavior.scripts();
    let names = |index: usize| -> Vec<&str> {
        bundles[index]
            .files
            .keys()
            .map(ScriptPath::as_str)
            .collect()
    };
    assert_eq!(names(0), ["main.js", "unused.js", "used.js"]);
    assert_eq!(names(1), ["main.js"]);
    assert_eq!(names(2), ["a.js", "b.js"]);
    // A file that was not read says so: it has no content, where a read file has its size.
    let read = |index: usize| -> Vec<Option<u64>> {
        bundles[index]
            .files
            .values()
            .map(|file| file.content.as_ref().map(|content| content.size))
            .collect()
    };
    assert_eq!(read(0), [Some(20), Some(11), Some(11)]);
    assert_eq!(read(1), [None]);
    assert_eq!(read(2), [None, None]);
    assert_eq!(
        bundles[1].imports().count() + bundles[2].imports().count(),
        0
    );
    let codes = |index: usize| -> Vec<(&str, Option<&str>)> {
        bundles[index]
            .findings()
            .map(|f| {
                (
                    f.code.name(),
                    f.location
                        .path
                        .as_ref()
                        .map(bedrock_addon::storage::PackPath::as_str),
                )
            })
            .collect()
    };
    assert_eq!(codes(0), []);
    assert_eq!(
        codes(1),
        [
            ("InvalidEnumValue", Some("manifest.json")),
            ("ScriptFileNotLoadable", Some("tool.js")),
        ]
    );
    assert_eq!(codes(2), [("ScriptFileNotLoadable", None)]);
}

/// The script modules of a stack that a server configuration is held against: the way from
/// a stack to `ScriptConfig::unmatched_folders`.
#[cfg(feature = "server_config")]
fn config_folders_told(
    stack: &PackStack<Behavior>,
    folders: &[&str],
) -> Vec<(String, Severity, Option<uuid::Uuid>)> {
    use bedrock_addon::families::server_config::{ConfigFolder, ScriptConfig, ScriptConfigFiles};
    let config = ScriptConfig::parse(&ScriptConfigFiles {
        folders: folders
            .iter()
            .map(|name| {
                let mut folder = ConfigFolder::new(*name);
                folder.permissions = Some(bytes::Bytes::from_static(
                    br#"{"allowed_modules": ["@minecraft/server"]}"#,
                ));
                folder
            })
            .collect(),
        ..ScriptConfigFiles::default()
    });
    config
        .unmatched_folders_of_modules(stack.scripts().iter().filter_map(|bundle| bundle.module))
        .iter()
        .map(|finding| {
            let folder = finding
                .code
                .args_of(&bedrock_addon::diagnostic::codes::UNMATCHED_CONFIG_FOLDER)
                .or_else(|| {
                    finding.code.args_of(
                        &bedrock_addon::diagnostic::codes::UNMATCHED_CONFIG_FOLDER_EXPECTED,
                    )
                })
                .unwrap_or_else(|| panic!("{:?}", finding.code))[0]
                .clone();
            let expected = match finding.detail.as_deref() {
                Some(bedrock_addon::diagnostic::DiagnosticDetail::Uuid(uuid)) => Some(*uuid),
                _ => None,
            };
            (folder, finding.severity, expected)
        })
        .collect()
}

// K124: a pack whose script does not start (its main file is missing) still has its script
// module, and the server still keys the pack's configuration by that module's UUID. A folder
// named after the pack's header UUID is the mistake D3 warns of, plugin or no plugin, and the
// folder named after the module is the module's.
#[test]
#[cfg(feature = "server_config")]
fn a_config_folder_is_held_against_a_script_module_whose_plugin_does_not_start() {
    let header = uuid::Uuid::from_u128(7);
    let module = uuid::Uuid::from_u128(1_000_007);
    let folders = [
        header.to_string(),
        module.to_string(),
        module.to_string().to_uppercase(),
        "notes".to_owned(),
    ];
    let folders: Vec<&str> = folders.iter().map(String::as_str).collect();
    let mut wanted = vec![
        // D3: the header UUID; the folder the module does use is named (an information of
        // ours, like every unmatched folder).
        (folders[0].to_owned(), Severity::Info, Some(module)),
        // D4: the module's UUID in another spelling.
        (folders[2].to_owned(), Severity::Info, Some(module)),
        // D5: nothing of the stack.
        ("notes".to_owned(), Severity::Info, None),
    ];
    wanted.sort_by(|a, b| a.0.cmp(&b.0));
    // The script starts.
    let started = script_stack(&[("scripts/main.js", b"")]);
    assert!(started.scripts()[0].plugin.is_some());
    assert_eq!(config_folders_told(&started, &folders), wanted);
    // The main file is missing: no plugin, the same module, the same findings.
    let stopped = script_stack(&[("scripts/other.js", b"")]);
    assert!(stopped.scripts()[0].plugin.is_none());
    assert_eq!(config_folders_told(&stopped, &folders), wanted);
    // The module of a bundle is the manifest's, started or not; a plugin gives the same.
    let of = |stack: &PackStack<Behavior>| stack.scripts()[0].module;
    let expected = bedrock_addon::families::script::ScriptModuleRef {
        pack_uuid: header,
        module_uuid: module,
    };
    assert_eq!(of(&started), Some(expected));
    assert_eq!(of(&stopped), Some(expected));
    assert_eq!(
        started.scripts()[0]
            .plugin
            .as_ref()
            .map(bedrock_addon::families::script::ScriptModuleRef::from),
        Some(expected)
    );
    // A pack with script files and no script module has a bundle and no module.
    let plain = PackStack::<Behavior>::of(
        [pack_of(
            vec![
                (
                    "manifest.json".to_owned(),
                    manifest("plain", 9, &[("data", None)], ""),
                ),
                ("scripts/a.js".to_owned(), Vec::new()),
            ],
            PackOrigin::User,
        )],
        LoadOptions::default(),
    )
    .expect("a behaviour pack");
    assert_eq!(plain.scripts().len(), 1);
    assert_eq!(plain.scripts()[0].module, None);
}

// ---- stage 1 reads the manifest the pack was opened with ----

/// The bundle findings of one behaviour pack whose manifest is `manifest`, opened with the
/// capabilities `capabilities` grants, and whether its plugin starts.
fn told_of(
    manifest: Vec<u8>,
    capabilities: bedrock_addon::manifest::CapabilityRegistry,
) -> (Vec<Told>, bool) {
    let files = files(manifest, &[("scripts/main.js", b"")])
        .into_iter()
        .map(|(path, bytes)| {
            (
                bedrock_addon::storage::PackPath::new(&path).expect("a pack path"),
                bytes,
            )
        });
    let options = OpenOptions {
        origin: PackOrigin::User,
        capabilities,
        ..OpenOptions::default()
    };
    let pack = bedrock_addon::pack::Pack::from_files(files, &options)
        .unwrap_or_else(|error| panic!("{error}"));
    let stack =
        PackStack::<Behavior>::of([pack], LoadOptions::default()).expect("a behaviour pack");
    let bundles = stack.scripts();
    assert_eq!(bundles.len(), 1);
    (told(&bundles[0]), bundles[0].graph.is_some())
}

#[test]
fn stage_1_places_a_legacy_finding_in_the_file_past_a_dropped_element() {
    let dependencies = r#"{ "module_name": "@minecraft/server", "uuid": "b26a4d4c-afdf-4690-88f8-931846312678", "version": "2.0.0" },
        { "module_name": "@minecraft/server", "version": "2.0.0", "optional": null },
        { "module_name": "@demo/unknown", "version": "1.0.0" }"#;
    let told = told_of(
        manifest_of("dropped", 40, &script_module(41, MAIN_JS), dependencies),
        bedrock_addon::manifest::CapabilityRegistry::default(),
    );
    // The dropped first element still counts: the `optional` and the unknown module are
    // told at the file's second and third elements.
    assert_eq!(
        told,
        (
            vec![
                finding("IgnoredMember", Severity::Info, "dependencies/1/optional"),
                finding("UnresolvedReference", Severity::Error, "dependencies/2"),
            ],
            false
        )
    );
}

#[test]
fn stage_1_places_a_relaxed_version_in_the_file_past_a_pack_dependency() {
    let manifest = format!(
        r#"{{
    "format_version": 3,
    "header": {{ "name": "relaxed", "description": "", "uuid": "{}", "version": "1.0.0",
        "min_engine_version": "1.26.0" }},
    "modules": [{{ "type": "script", "uuid": "{}", "version": "1.0.0",
        "entry": "scripts/main.js" }}],
    "capabilities": ["script_eval"],
    "dependencies": [
        {{ "uuid": "33333333-3333-3333-3333-333333333333", "version": "1.0.0" }},
        {{ "module_name": "@minecraft/server", "version": "2.0" }}
    ]
}}"#,
        uuid::Uuid::from_u128(50),
        uuid::Uuid::from_u128(51),
    )
    .into_bytes();
    // A host that grants no capability: the pack's own findings say so when it is opened,
    // and stage 1 adds nothing about it.
    let told = told_of(
        manifest,
        bedrock_addon::manifest::CapabilityRegistry::empty(),
    );
    assert_eq!(
        told,
        (
            vec![finding(
                "FieldSinceEngine",
                Severity::Info,
                "dependencies/1/version"
            )],
            true
        )
    );
}
