//! The validation rules that no fixture row of the fixtures carries, the order of the
//! checks, the listed deviations, and the rules across packs. Manifests are written inline.

use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::families::script::{
    Discovery, ScriptContext, ScriptDiscovery, ScriptFiles, ScriptModule, ScriptRuntime,
    ScriptTarget,
};
use bedrock_addon::load::LoadContext;
use bedrock_addon::manifest::Manifest;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::FormatVersion;

use crate::common::{BDS_ALLOWED_MODULES, BDS_ENGINE, bds_catalog, code_name, severity_name};

const SERVER_UUID: &str = "b26a4d4c-afdf-4690-88f8-931846312678";
const SERVER_UI_UUID: &str = "2bd50a27-ab5f-4f40-a596-3641627c635e";

/// A format-2 behaviour pack manifest named `pack` with one script module (UUID ending in
/// `module`, version 3.2.1) and the given module members and dependencies.
fn legacy(pack: &str, module: u8, module_members: &str, dependencies: &str) -> String {
    format!(
        r#"{{
  "format_version": 2,
  "header": {{ "name": "{pack}", "uuid": "11111111-1111-1111-1111-1111111111{module:02x}",
              "version": [1, 0, 0], "min_engine_version": [1, 21, 0] }},
  "modules": [ {{ "type": "script", "uuid": "22222222-2222-2222-2222-2222222222{module:02x}",
                 "version": [3, 2, 1]{module_members} }} ],
  "dependencies": [{dependencies}]
}}"#
    )
}

/// The format-3 form of [`legacy`].
fn comprehensive(pack: &str, module_members: &str, dependencies: &str) -> String {
    format!(
        r#"{{
  "format_version": 3,
  "header": {{ "name": "{pack}", "uuid": "11111111-1111-1111-1111-111111111111",
              "version": "1.0.0", "min_engine_version": "1.26.30" }},
  "modules": [ {{ "type": "script", "uuid": "22222222-2222-2222-2222-222222222222",
                 "version": "3.2.1"{module_members} }} ],
  "dependencies": [{dependencies}]
}}"#
    )
}

const JS: &str = r#", "language": "javascript", "entry": "scripts/main.js""#;

fn engine(version: FormatVersion) -> LoadContext {
    let mut cx = LoadContext::detached();
    cx.engine_version = version;
    cx
}

fn parse(text: &str) -> Result<(Option<ScriptModule>, Diagnostics), Diagnostics> {
    match ScriptModule::parse(
        &PackPath::from_static("manifest.json"),
        text.as_bytes(),
        &engine(BDS_ENGINE),
    ) {
        Ok(document) => {
            let diagnostics = document.diagnostics().clone();
            Ok((document.into_value(), diagnostics))
        }
        Err(rejected) => Err(rejected.diagnostics),
    }
}

fn module(text: &str) -> ScriptModule {
    parse(text)
        .unwrap_or_else(|diagnostics| panic!("refused: {diagnostics:?}"))
        .0
        .expect("a script module")
}

/// `(code, severity, path, text)` of every finding.
fn listed(diagnostics: &Diagnostics) -> Vec<(String, &'static str, String, String)> {
    diagnostics
        .iter()
        .map(|d| {
            (
                code_name(d),
                severity_name(d.severity),
                d.location.json_path.to_string(),
                d.code.to_string(),
            )
        })
        .collect()
}

/// `(code, severity, path)` of every finding.
fn brief(diagnostics: &Diagnostics) -> Vec<(String, &'static str, String)> {
    listed(diagnostics)
        .into_iter()
        .map(|(code, severity, path, _)| (code, severity, path))
        .collect()
}

fn row(code: &str, severity: &'static str, path: &str) -> (String, &'static str, String) {
    (code.to_owned(), severity, path.to_owned())
}

fn main_js() -> ScriptFiles {
    ScriptFiles::from_pack_paths(["manifest.json", "scripts/main.js"])
}

fn bds_allowed() -> Vec<String> {
    BDS_ALLOWED_MODULES
        .iter()
        .map(|m| (*m).to_owned())
        .collect()
}

/// Discovery of one pack on the probe server: BDS permissions, the BDS catalog.
fn discover_with(
    text: &str,
    files: &ScriptFiles,
    change: impl FnOnce(&mut ScriptContext<'_>),
) -> Discovery {
    let allowed = bds_allowed();
    let mut cx = ScriptContext {
        allowed_modules: Some(&allowed),
        ..ScriptContext::default()
    };
    change(&mut cx);
    let catalog = bds_catalog();
    ScriptDiscovery::new(&catalog).discover(&module(text), files, &cx)
}

fn discover(text: &str) -> Discovery {
    discover_with(text, &main_js(), |_| {})
}

// ---- stage 1: the rows without a fixture ----

#[test]
fn m2_a_legacy_dependency_that_is_no_object_refuses_the_pack() {
    let refused = parse(&legacy("p", 1, JS, r#""@minecraft/server""#)).expect_err("refused");
    assert_eq!(
        brief(&refused),
        [row("WrongTypeProperty", "error", "dependencies/0")]
    );
}

#[test]
fn m4_a_legacy_module_name_of_the_wrong_type_refuses_the_pack() {
    let refused = parse(&legacy(
        "p",
        1,
        JS,
        r#"{ "module_name": 7, "version": "2.0.0" }"#,
    ))
    .expect_err("refused");
    assert_eq!(
        brief(&refused),
        [row(
            "WrongTypeProperty",
            "error",
            "dependencies/0/module_name"
        )]
    );
}

#[test]
fn m4_a_legacy_element_without_a_module_name_is_read_as_a_pack_dependency() {
    // The half of M4 that cannot fire: `module_name` is what makes an element a module
    // dependency, so one without it is a pack dependency, and what it lacks is the pack
    // dependency's `uuid`. No finding names `module_name`.
    for (dependency, findings) in [
        (
            r#"{ "version": "2.0.0" }"#,
            vec![row("MissingProperty", "error", "dependencies/0/uuid")],
        ),
        (
            r#"{ "module_name": null, "version": [1, 0, 0] }"#,
            vec![row("MissingProperty", "error", "dependencies/0/uuid")],
        ),
        (
            "{ }",
            vec![
                row("MissingProperty", "error", "dependencies/0/uuid"),
                row("MissingProperty", "error", "dependencies/0/version"),
            ],
        ),
    ] {
        let refused = parse(&legacy("p", 1, JS, dependency)).expect_err("refused");
        assert_eq!(brief(&refused), findings, "{dependency}");
    }
    // With its `uuid` it is a pack dependency, and the script side has no dependency.
    let pack = r#"{ "uuid": "33333333-3333-3333-3333-333333333333", "version": [1, 0, 0] }"#;
    let (module, diagnostics) = parse(&legacy("p", 1, JS, pack)).expect("loads");
    assert_eq!(brief(&diagnostics), []);
    assert_eq!(module.expect("a script module").dependencies.len(), 0);
}

#[test]
fn m5_a_legacy_version_string_that_is_no_version_refuses_the_pack() {
    for version in ["*", "2.0", "2", "abc"] {
        let dependency =
            format!(r#"{{ "module_name": "@minecraft/server", "version": "{version}" }}"#);
        let refused = parse(&legacy("p", 1, JS, &dependency)).expect_err("refused");
        assert_eq!(
            brief(&refused),
            [row(
                "MalformedPropertyVersion",
                "error",
                "dependencies/0/version"
            )],
            "{version}"
        );
    }
}

#[test]
fn m7_a_legacy_dependency_without_a_version_refuses_the_pack() {
    let refused = parse(&legacy(
        "p",
        1,
        JS,
        r#"{ "module_name": "@minecraft/server" }"#,
    ))
    .expect_err("refused");
    assert_eq!(
        brief(&refused),
        [row("MissingProperty", "error", "dependencies/0/version")]
    );
}

#[test]
fn m6_a_legacy_version_of_another_type_refuses_the_pack() {
    for version in ["true", "{}", "2.5"] {
        let dependency =
            format!(r#"{{ "module_name": "@minecraft/server", "version": {version} }}"#);
        let refused = parse(&legacy("p", 1, JS, &dependency)).expect_err("refused");
        assert_eq!(
            brief(&refused),
            [row("WrongTypeProperty", "error", "dependencies/0/version")],
            "{version}"
        );
    }
}

#[test]
fn m12_an_unknown_member_of_a_v3_dependency_refuses_the_pack() {
    let refused = parse(&comprehensive(
        "p",
        JS,
        r#"{ "module_name": "@minecraft/server", "version": "2.0.0", "note": 1 }"#,
    ))
    .expect_err("refused");
    assert_eq!(
        brief(&refused),
        [row("UnrecognizedField", "error", "dependencies/0/note")]
    );
}

#[test]
fn m13_a_v3_dependency_without_its_version_refuses_the_pack() {
    let refused = parse(&comprehensive(
        "p",
        JS,
        r#"{ "module_name": "@minecraft/server" }"#,
    ))
    .expect_err("refused");
    assert_eq!(
        brief(&refused),
        [row(
            "MissingRequiredField",
            "error",
            "dependencies/0/version"
        )]
    );
}

#[test]
fn m13_a_v3_element_without_a_module_name_is_read_as_a_pack_dependency() {
    // The half of M13 that cannot fire: the element without `module_name` goes to the pack
    // branch, whose required member is `uuid`. No finding names `module_name`.
    for (dependency, findings) in [
        (
            r#"{ "version": "2.0.0" }"#,
            vec![row("MissingRequiredField", "error", "dependencies/0/uuid")],
        ),
        (
            "{ }",
            vec![
                row("MissingRequiredField", "error", "dependencies/0/uuid"),
                row("MissingRequiredField", "error", "dependencies/0/version"),
            ],
        ),
    ] {
        let refused = parse(&comprehensive("p", JS, dependency)).expect_err("refused");
        assert_eq!(brief(&refused), findings, "{dependency}");
    }
    let pack = r#"{ "uuid": "33333333-3333-3333-3333-333333333333", "version": "1.0.0" }"#;
    let (module, diagnostics) = parse(&comprehensive("p", JS, pack)).expect("loads");
    assert_eq!(brief(&diagnostics), []);
    assert_eq!(module.expect("a script module").dependencies.len(), 0);
}

#[test]
fn m10_a_v3_version_that_is_a_number_refuses_the_pack() {
    let refused = parse(&comprehensive(
        "p",
        JS,
        r#"{ "module_name": "@minecraft/server", "version": 2 }"#,
    ))
    .expect_err("refused");
    assert_eq!(
        brief(&refused),
        [row("InvalidType", "error", "dependencies/0/version")]
    );
}

#[test]
fn a_legacy_manifest_ignores_unknown_members_of_a_module_and_a_dependency() {
    let text = legacy(
        "p",
        1,
        r#", "language": "javascript", "entry": "scripts/main.js", "main": "x""#,
        r#"{ "module_name": "@minecraft/server", "version": "2.0.0", "note": 1 }"#,
    );
    let (module, diagnostics) = parse(&text).expect("loads");
    assert_eq!(brief(&diagnostics), []);
    assert_eq!(module.expect("a script module").dependencies.len(), 1);
}

#[test]
fn m8_a_legacy_optional_is_ignored_whatever_it_says() {
    for optional in ["true", "false", "\"yes\"", "null"] {
        let dependency = format!(
            r#"{{ "module_name": "@minecraft/server", "version": "2.0.0", "optional": {optional} }}"#
        );
        let (module, diagnostics) = parse(&legacy("p", 1, JS, &dependency)).expect("loads");
        assert_eq!(
            brief(&diagnostics),
            [row("IgnoredMember", "info", "dependencies/0/optional")],
            "{optional}"
        );
        assert!(!module.expect("a script module").dependencies[0].optional);
    }
}

#[test]
fn a_v3_optional_is_read_and_is_no_finding() {
    let text = comprehensive(
        "p",
        JS,
        r#"{ "module_name": "@minecraft/server", "version": "2.0.0", "optional": true },
           { "module_name": "@minecraft/server-ui", "version": "2.0.0", "optional": false },
           { "module_name": "@minecraft/server-admin", "version": "beta" }"#,
    );
    let (module, diagnostics) = parse(&text).expect("loads");
    assert_eq!(brief(&diagnostics), []);
    let optional: Vec<bool> = module
        .expect("a script module")
        .dependencies
        .iter()
        .map(|d| d.optional)
        .collect();
    assert_eq!(optional, [true, false, false]);
}

#[test]
fn m14_only_the_relaxed_form_is_gated_and_only_below_1_26_30() {
    let relaxed = |min_engine: &str, version: &str| {
        comprehensive(
            "p",
            JS,
            &format!(r#"{{ "module_name": "@minecraft/server", "version": "{version}" }}"#),
        )
        .replace("1.26.30", min_engine)
    };
    let at = |text: &str, engine_version: FormatVersion| {
        ScriptModule::parse(
            &PackPath::from_static("manifest.json"),
            text.as_bytes(),
            &engine(engine_version),
        )
        .map(|document| brief(document.diagnostics()))
        .map_err(|rejected| brief(&rejected.diagnostics))
    };
    let warning = row("FieldSinceEngine", "info", "dependencies/0/version");
    let error = row(
        "MalformedPropertyVersion",
        "error",
        "dependencies/0/version",
    );
    // The pack's own engine: 1.26.29 is below, 1.26.30 is not.
    assert_eq!(
        at(&relaxed("1.26.29", "2.0"), BDS_ENGINE),
        Ok(vec![warning.clone()])
    );
    assert_eq!(at(&relaxed("1.26.30", "2.0"), BDS_ENGINE), Ok(vec![]));
    assert_eq!(at(&relaxed("1.26.29", "2"), BDS_ENGINE), Ok(vec![warning]));
    // A full version and a selector are no relaxed form, on any engine.
    for version in [
        "2.0.0",
        "2.0.0-beta",
        "2.0.0+abc",
        "beta",
        "alpha",
        "internal",
    ] {
        let old = FormatVersion::new(1, 26, 20);
        assert_eq!(
            at(&relaxed("1.26.20", version), old),
            Ok(vec![]),
            "{version}"
        );
    }
    // The engine: refused below 1.26.30.
    assert_eq!(
        at(&relaxed("1.26.20", "2.0"), FormatVersion::new(1, 26, 29)),
        Err(vec![error])
    );
    assert_eq!(
        at(&relaxed("1.26.30", "2.0"), FormatVersion::new(1, 26, 30)),
        Ok(vec![])
    );
}

#[test]
fn the_relaxed_form_is_the_legacy_readers_error_on_every_engine() {
    // M5: the legacy reader never takes `"2.0"`, whatever the engine.
    let text = legacy(
        "p",
        1,
        JS,
        r#"{ "module_name": "@minecraft/server", "version": "2.0" }"#,
    );
    let refused = parse(&text).expect_err("refused");
    assert_eq!(
        brief(&refused),
        [row(
            "MalformedPropertyVersion",
            "error",
            "dependencies/0/version"
        )]
    );
}

#[test]
fn of_a_version_written_twice_the_last_decides_the_form() {
    // The last of two members stands, also for what stage 1 reads from the text.
    let text = comprehensive(
        "p",
        JS,
        r#"{ "module_name": "@minecraft/server", "version": "2.0", "version": "2.0.0" }"#,
    );
    let document = ScriptModule::parse(
        &PackPath::from_static("manifest.json"),
        text.as_bytes(),
        &engine(FormatVersion::new(1, 26, 20)),
    )
    .expect("the last version is a full one");
    assert_eq!(brief(document.diagnostics()), []);
    let reversed = text.replace(
        r#""2.0", "version": "2.0.0""#,
        r#""2.0.0", "version": "2.0""#,
    );
    assert!(
        ScriptModule::parse(
            &PackPath::from_static("manifest.json"),
            reversed.as_bytes(),
            &engine(FormatVersion::new(1, 26, 20)),
        )
        .is_err()
    );
}

#[test]
fn a_manifest_with_a_byte_order_mark_reads_alike() {
    let text = legacy(
        "p",
        1,
        JS,
        r#"{ "module_name": "@minecraft/server", "version": "beta", "optional": true }"#,
    );
    let with_bom = format!("\u{feff}{text}");
    assert_eq!(
        parse(&with_bom).expect("loads"),
        parse(&text).expect("loads")
    );
}

#[test]
fn a_manifest_without_a_script_module_has_no_script_side_and_keeps_its_findings() {
    let text = legacy(
        "p",
        1,
        "",
        r#"{ "module_name": "@minecraft/server", "version": "2.0.0", "optional": true }"#,
    )
    .replace(r#""type": "script""#, r#""type": "data""#);
    let (module, diagnostics) = parse(&text).expect("loads");
    assert_eq!(module, None);
    assert_eq!(
        brief(&diagnostics),
        [row("IgnoredMember", "info", "dependencies/0/optional")]
    );
}

#[test]
fn paths_are_the_files_when_the_legacy_reader_drops_an_element() {
    // The first element is dropped (M3); the second is the manifest's first dependency and
    // the file's second.
    let text = legacy(
        "p",
        1,
        JS,
        &format!(
            r#"{{ "module_name": "@minecraft/server", "uuid": "{SERVER_UUID}", "version": "2.0.0" }},
               {{ "uuid": "33333333-3333-3333-3333-333333333333", "version": [1, 0, 0] }},
               {{ "module_name": "@minecraft/server", "version": "beta" }},
               {{ "uuid": "{SERVER_UI_UUID}", "version": [0, 1, 0] }}"#
        ),
    );
    let (parsed, diagnostics) = parse(&text).expect("loads");
    assert_eq!(
        brief(&diagnostics),
        [row("InvalidProperty", "warning", "dependencies/0")]
    );
    let parsed = parsed.expect("a script module");
    assert_eq!(parsed.dependencies[0].path.to_string(), "dependencies/2");
    assert_eq!(
        parsed.uuid_dependencies.len(),
        1,
        "one pack dependency names a native module"
    );
    assert_eq!(
        parsed.uuid_dependencies[0].path.to_string(),
        "dependencies/3"
    );

    // From the manifest alone the places are the file's too: the manifest keeps them.
    let manifest = Manifest::parse(
        &PackPath::from_static("manifest.json"),
        text.as_bytes(),
        &bedrock_addon::manifest::CapabilityRegistry::default(),
    )
    .expect("loads");
    let viewed = ScriptModule::from_manifest(&manifest).expect("a script module");
    assert_eq!(viewed.dependencies[0].path.to_string(), "dependencies/2");
    assert_eq!(
        viewed.uuid_dependencies[0].path.to_string(),
        "dependencies/3"
    );
    // And everything else is the same value.
    assert_eq!(
        viewed.dependencies[0].module_name,
        parsed.dependencies[0].module_name
    );
    assert_eq!(
        viewed.dependencies[0].version,
        parsed.dependencies[0].version
    );
    assert_eq!(
        (viewed.module_uuid, &viewed.module_version),
        (parsed.module_uuid, &parsed.module_version)
    );
    assert_eq!(viewed.plugin_name(), "p - 3.2.1");
}

#[test]
fn a_format_0_manifest_has_its_script_module_in_the_header_and_no_dependencies() {
    let text = r#"{
  "format_version": 0,
  "header": {
    "name": "Old Pack", "pack_id": "11111111-1111-1111-1111-111111111111", "packs_version": [1, 0, 0],
    "modules": [ { "type": "script", "uuid": "22222222-2222-2222-2222-222222222222", "version": [3, 2, 1],
                   "entry": "scripts/main.js" } ]
  },
  "dependencies": [ { "module_name": "@minecraft/server", "version": "beta", "optional": true } ]
}"#;
    let (module, diagnostics) = parse(text).expect("loads");
    let module = module.expect("a script module");
    assert_eq!(module.path.to_string(), "header/modules/0");
    assert!(
        module.dependencies.is_empty(),
        "format 0 reads no dependencies"
    );
    assert!(
        !listed(&diagnostics)
            .iter()
            .any(|(code, ..)| code == "IgnoredMember"),
        "nothing of the unread dependencies is reported: {diagnostics:?}"
    );
}

#[test]
fn a_client_script_module_is_the_client_target() {
    let text = legacy("p", 1, JS, "").replace(r#""type": "script""#, r#""type": "client_script""#);
    assert_eq!(module(&text).target, ScriptTarget::Client);
    assert_eq!(module(&legacy("p", 1, JS, "")).target, ScriptTarget::Server);
}

#[test]
fn target_spellings_both_ways() {
    for (target, name) in [
        (ScriptTarget::Server, "script"),
        (ScriptTarget::Client, "client_script"),
    ] {
        assert_eq!(target.module_type(), name);
        assert_eq!(ScriptTarget::from_module_type(name), Some(target));
    }
    for other in ["Script", "javascript", "data", "client_data", ""] {
        assert_eq!(ScriptTarget::from_module_type(other), None, "{other:?}");
    }
    assert_eq!(ScriptRuntime::JavaScript.name(), "JavaScript");
}

#[test]
fn only_the_two_native_module_uuids_are_native_dependencies() {
    let text = legacy(
        "p",
        1,
        JS,
        &format!(
            r#"{{ "uuid": "{SERVER_UUID}", "version": [0, 1, 0] }},
               {{ "uuid": "{SERVER_UI_UUID}", "version": [1, 2, 3] }},
               {{ "uuid": "6f4b6893-1bb6-42fd-b458-7fa3d0c89616", "version": [0, 1, 0] }}"#
        ),
    );
    let module = module(&text);
    let uuids: Vec<String> = module
        .uuid_dependencies
        .iter()
        .map(|d| d.uuid.to_string())
        .collect();
    // The gametest module's UUID is an ordinary pack dependency.
    assert_eq!(uuids, [SERVER_UUID, SERVER_UI_UUID]);
}

// ---- stage 2: order, several findings, the entry's path rules ----

#[test]
fn every_dependency_is_filtered_after_one_rejected_the_plugin() {
    let found = discover(&legacy(
        "p",
        1,
        JS,
        r#"{ "module_name": "@minecraft/server", "version": "beta" },
           { "module_name": "@minecraft/common", "version": "1.0.0" },
           { "module_name": "@minecraft/server", "version": "internal" },
           { "module_name": "@minecraft/server", "version": "alpha" }"#,
    ));
    assert!(found.plugin().is_none());
    assert_eq!(
        brief(found.diagnostics()),
        [
            row("MissingToggles", "error", "dependencies/0"),
            row("ScriptModulePermission", "error", "dependencies/1"),
            row("ScriptModulePermission", "error", "dependencies/2"),
            row("ScriptAlphaDependency", "info", "dependencies/3"),
        ]
    );
}

#[test]
fn a_gated_dependency_hides_the_missing_main_file_and_the_uuid_warning() {
    // S12 and S5 run only when no dependency rejected the plugin.
    let text = legacy(
        "p",
        1,
        r#", "entry": "scripts/missing.js""#,
        &format!(
            r#"{{ "module_name": "@minecraft/server", "version": "beta" }},
               {{ "uuid": "{SERVER_UUID}", "version": [0, 1, 0] }}"#
        ),
    );
    assert_eq!(
        brief(discover(&text).diagnostics()),
        [row("MissingToggles", "error", "dependencies/0")]
    );
    // Without the gated dependency both appear, the warning first.
    let ungated = text.replace(r#""version": "beta""#, r#""version": "2.0.0""#);
    assert_eq!(
        brief(discover(&ungated).diagnostics()),
        [
            row("DeprecatedMember", "warning", "dependencies/1"),
            row("UnresolvedReference", "error", "modules/0/entry"),
        ]
    );
}

#[test]
fn the_language_is_checked_before_the_entry_and_the_entry_before_the_dependencies() {
    let dependency = r#"{ "module_name": "@minecraft/server", "version": "beta" }"#;
    let both = legacy(
        "p",
        1,
        r#", "language": "typescript", "entry": "scripts/main.ts""#,
        dependency,
    );
    assert_eq!(
        brief(discover(&both).diagnostics()),
        [row("InvalidEnumValue", "error", "modules/0/language")]
    );
    let entry = legacy("p", 1, r#", "entry": "scripts/main.ts""#, dependency);
    assert_eq!(
        brief(discover(&entry).diagnostics()),
        [row("InvalidValue", "error", "modules/0/entry")]
    );
}

#[test]
fn the_entry_loses_only_a_leading_scripts_root() {
    // (entry, files, main or the name vanilla prints as missing) [bds-run Q01–Q08].
    let runs = [
        ("scripts/sub/main.js", "scripts/sub/main.js", "sub/main.js"), // Q03
        ("sub/main.js", "scripts/sub/main.js", "sub/main.js"),         // Q08
        ("scripts\\main.js", "scripts/main.js", "main.js"),            // Q06
        ("scripts/.hidden.js", "scripts/.hidden.js", ".hidden.js"),
        ("scripts/a.b.js", "scripts/a.b.js", "a.b.js"),
    ];
    for (entry, file, main) in runs {
        let entry_json = entry.replace('\\', "\\\\");
        let text = legacy("p", 1, &format!(r#", "entry": "{entry_json}""#), "");
        let found = discover_with(&text, &ScriptFiles::from_pack_paths([file]), |_| {});
        assert_eq!(brief(found.diagnostics()), [], "{entry}");
        assert_eq!(found.plugin().expect("runs").main.as_str(), main, "{entry}");
    }
    let missing = [
        ("./scripts/main.js", "./scripts/main.js"), // Q04
        ("/scripts/main.js", "/scripts/main.js"),   // Q05
        ("other/main.js", "other/main.js"),         // Q07
        ("Scripts/main.js", "Scripts/main.js"),
        ("scripts/Main.js", "Main.js"),
    ];
    for (entry, printed) in missing {
        let text = legacy("p", 1, &format!(r#", "entry": "{entry}""#), "");
        let found = discover(&text);
        assert!(found.plugin().is_none(), "{entry}");
        assert_eq!(
            listed(found.diagnostics()),
            [(
                "UnresolvedReference".to_owned(),
                "error",
                "modules/0/entry".to_owned(),
                format!("Plugin [p - 3.2.1] - does not contain main file [{printed}]."),
            )],
            "{entry}"
        );
    }
}

#[test]
fn an_entry_without_the_js_extension_is_not_supported() {
    for entry in [
        "scripts/main",
        "scripts/main.js.txt",
        "scripts/main.js/",
        "",
        "scripts/js",
    ] {
        let text = legacy("p", 1, &format!(r#", "entry": "{entry}""#), "");
        let found = discover(&text);
        assert_eq!(
            listed(found.diagnostics()),
            [(
                "InvalidValue".to_owned(),
                "error",
                "modules/0/entry".to_owned(),
                format!(
                    "Plugin [p - 3.2.1] - entry [{entry}] not supported for runtime [JavaScript]."
                ),
            )],
            "{entry:?}"
        );
    }
}

#[test]
fn script_files_are_the_js_files_below_scripts() {
    let files = ScriptFiles::from_pack_paths([
        "scripts/main.js",
        "scripts/lib/b.js",
        "scripts/.hidden.js",
        "scripts/a.JS",
        "scripts/a.mjs",
        "scripts/a.ts",
        "scripts/a.js.map",
        "scripts/noext",
        "main.js",
        "Scripts/c.js",
        "manifest.json",
    ]);
    assert_eq!(files.len(), 3);
    for member in ["main.js", "lib/b.js", ".hidden.js"] {
        assert!(files.contains(member), "{member}");
    }
    for other in [
        "a.JS",
        "a.mjs",
        "c.js",
        "scripts/main.js",
        "Main.js",
        "lib/B.js",
    ] {
        assert!(!files.contains(other), "{other}");
    }
    assert!(ScriptFiles::from_pack_paths(["manifest.json"]).is_empty());
    assert!(ScriptFiles::from_members(["x.js"]).contains("x.js"));
}

#[test]
fn s13_an_editor_module_is_dropped_outside_an_editor_world() {
    let text = legacy(
        "p",
        1,
        JS,
        r#"{ "module_name": "@minecraft/server-editor", "version": "beta" },
           { "module_name": "@minecraft/server", "version": "2.0.0" }"#,
    );
    let found = discover(&text);
    assert_eq!(
        listed(found.diagnostics()),
        [(
            "DroppedField".to_owned(),
            "info",
            "dependencies/0".to_owned(),
            "Plugin [p - 3.2.1] - skipping dependency on [@minecraft/server-editor] because this is not an editor world."
                .to_owned(),
        )]
    );
    let plugin = found
        .plugin()
        .expect("the script runs without the editor module");
    let names: Vec<&str> = plugin
        .dependencies
        .iter()
        .map(|d| d.module_name.as_str())
        .collect();
    assert_eq!(names, ["@minecraft/server"]);

    // In an editor world the filter lets it through to the others: here the Beta APIs gate,
    // since no beta of a module the catalog lacks is `0.0.0-beta`.
    let editor = discover_with(&text, &main_js(), |cx| cx.editor_world = true);
    assert_eq!(
        brief(editor.diagnostics()),
        [row("MissingToggles", "error", "dependencies/0")]
    );
}

#[test]
fn s9_an_optional_dependency_a_permission_refuses_is_dropped() {
    let text = comprehensive(
        "p",
        JS,
        r#"{ "module_name": "@minecraft/common", "version": "1.0.0", "optional": true },
           { "module_name": "@minecraft/server", "version": "2.0.0", "optional": true }"#,
    );
    let found = discover(&text);
    assert_eq!(
        brief(found.diagnostics()),
        [row("DroppedField", "info", "dependencies/0")]
    );
    let plugin = found.plugin().expect("runs");
    assert_eq!(plugin.dependencies.len(), 1);
    assert!(plugin.dependencies[0].optional);
}

#[test]
fn s10_the_wildcard_of_a_permission_list_allows_every_module() {
    // script_config M5: `"*"` in `allowed_modules` allows every module [bds-run B `wild`].
    let text = legacy(
        "p",
        1,
        JS,
        r#"{ "module_name": "@minecraft/server-ui", "version": "2.0.0" },
           { "module_name": "@minecraft/server", "version": "2.0.0" }"#,
    );
    let with = |allowed: &[&str]| {
        let allowed: Vec<String> = allowed.iter().map(|name| (*name).to_owned()).collect();
        let cx = ScriptContext {
            allowed_modules: Some(&allowed),
            ..ScriptContext::default()
        };
        let found = ScriptDiscovery::new(&bds_catalog()).discover(&module(&text), &main_js(), &cx);
        (brief(found.diagnostics()), found.plugin().is_some())
    };
    assert_eq!(with(&["*"]), (Vec::new(), true));
    assert_eq!(with(&["@minecraft/server", "*"]), (Vec::new(), true));
    // Only the one-character name: a pattern is a name like any other.
    assert_eq!(
        with(&["@minecraft/*", "@minecraft/server"]),
        (
            vec![row("ScriptModulePermission", "error", "dependencies/0")],
            false
        )
    );
}

#[test]
fn the_package_origin_lifts_the_three_gates_and_not_the_permissions() {
    let text = legacy(
        "p",
        1,
        JS,
        r#"{ "module_name": "@minecraft/server", "version": "beta" },
           { "module_name": "@minecraft/server-gametest", "version": "beta" },
           { "module_name": "@minecraft/common", "version": "1.0.0" }"#,
    );
    let package = discover_with(&text, &main_js(), |cx| cx.is_package_origin = true);
    assert_eq!(
        brief(package.diagnostics()),
        [row("ScriptModulePermission", "error", "dependencies/2")]
    );
    let beta = discover_with(&text, &main_js(), |cx| cx.beta_apis = true);
    assert_eq!(brief(beta.diagnostics()), brief(package.diagnostics()));
    let neither = discover(&text);
    assert_eq!(
        brief(neither.diagnostics()),
        [
            row("MissingToggles", "error", "dependencies/0"),
            row("MissingToggles", "error", "dependencies/1"),
            row("ScriptModulePermission", "error", "dependencies/2"),
        ]
    );
}

#[test]
fn a_beta_selector_of_a_module_without_a_beta_is_gated_by_its_major() {
    // the fixtures unit test: `0.0.0-beta`, then the Beta APIs gate since the major is 0.
    let text = legacy(
        "p",
        1,
        JS,
        r#"{ "module_name": "@minecraft/nonexistent", "version": "alpha" }"#,
    );
    let found = discover_with(&text, &main_js(), |cx| cx.allowed_modules = None);
    assert_eq!(
        listed(found.diagnostics()),
        [(
            "MissingToggles".to_owned(),
            "error",
            "dependencies/0".to_owned(),
            "Plugin [p - 3.2.1] - requesting dependency on beta APIs [@minecraft/nonexistent - 0.0.0-alpha], but the Beta APIs experiment is not enabled."
                .to_owned(),
        )]
    );
}

#[test]
fn s12_the_server_ui_uuid_stands_for_server_ui() {
    // the validation rules S12, the schema: either UUID is the
    // deprecation warning and a dependency on 0.1.0, whatever version is written.
    let text = legacy(
        "p",
        1,
        JS,
        &format!(r#"{{ "uuid": "{SERVER_UI_UUID}", "version": [9, 9, 9] }}"#),
    );
    let found = discover(&text);
    assert_eq!(
        listed(found.diagnostics()),
        [(
            "DeprecatedMember".to_owned(),
            "warning",
            "dependencies/0".to_owned(),
            "Plugin [p - 3.2.1] dependency by UUID is deprecated and only supports version 0.1.0, use module_name instead."
                .to_owned(),
        )]
    );
    let plugin = found.plugin().expect("the script runs");
    let resolved: Vec<String> = plugin
        .dependencies
        .iter()
        .map(|d| format!("{} {} {}", d.module_name, d.requested, d.resolved))
        .collect();
    assert_eq!(resolved, ["@minecraft/server-ui 0.1.0 0.1.0"]);

    // Both UUIDs in one manifest: one warning each, both modules at 0.1.0.
    let both = legacy(
        "p",
        1,
        JS,
        &format!(
            r#"{{ "uuid": "{SERVER_UUID}", "version": [0, 1, 0] }},
               {{ "uuid": "{SERVER_UI_UUID}", "version": [0, 1, 0] }}"#
        ),
    );
    let found = discover(&both);
    assert_eq!(
        brief(found.diagnostics()),
        [
            row("DeprecatedMember", "warning", "dependencies/0"),
            row("DeprecatedMember", "warning", "dependencies/1"),
        ]
    );
    let names: Vec<String> = found
        .plugin()
        .expect("the script runs")
        .dependencies
        .iter()
        .map(|d| format!("{} {}", d.module_name, d.resolved))
        .collect();
    assert_eq!(
        names,
        ["@minecraft/server 0.1.0", "@minecraft/server-ui 0.1.0"]
    );
}

// ---- the listed deviations ----

#[test]
fn a_conflict_beside_an_unshipped_request_reads_alike_with_the_beta_apis_on_or_off() {
    // The pack of fixture sm48, whose probe ran in a world with the experiment on.
    let path = crate::common::fixtures().join("sm48_conflict_and_unshipped/manifest.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let off = discover_with(&text, &main_js(), |cx| cx.beta_apis = false);
    let on = discover_with(&text, &main_js(), |cx| cx.beta_apis = true);
    assert_eq!(
        brief(off.diagnostics()),
        [
            row("ScriptModuleVersionUnavailable", "error", "dependencies/1"),
            row("ScriptModuleVersionConflict", "error", "dependencies/1"),
        ]
    );
    assert_eq!(listed(off.diagnostics()), listed(on.diagnostics()));
    assert!(off.plugin().is_none() && on.plugin().is_none());
}

#[test]
fn unknown_module_is_reported_at_discovery_deviation() {
    // Vanilla accepts the name at discovery and fails at context creation; ours reports it
    // with the dependency, before the main file is looked at.
    let text = legacy(
        "p",
        1,
        r#", "entry": "scripts/missing.js""#,
        r#"{ "module_name": "@minecraft/nonexistent", "version": "1.0.0" }"#,
    );
    let found = discover_with(&text, &main_js(), |cx| cx.allowed_modules = None);
    assert_eq!(
        brief(found.diagnostics()),
        [row("UnresolvedReference", "error", "dependencies/0")]
    );
}

#[test]
fn v3_bad_semver_is_a_malformed_version_deviation() {
    // Vanilla: a Puv validation error printed as the literal `Pack error: '%s'.`.
    let refused = parse(&comprehensive(
        "p",
        JS,
        r#"{ "module_name": "@minecraft/server", "version": "abc" }"#,
    ))
    .expect_err("refused");
    assert_eq!(
        brief(&refused),
        [row(
            "MalformedPropertyVersion",
            "error",
            "dependencies/0/version"
        )]
    );
}

#[test]
fn legacy_optional_is_an_info_deviation() {
    // Vanilla: silently ignored.
    let text = legacy(
        "p",
        1,
        JS,
        r#"{ "module_name": "@minecraft/server", "version": "2.0.0", "optional": true }"#,
    );
    let (_, diagnostics) = parse(&text).expect("loads");
    let all = listed(&diagnostics);
    assert_eq!(all.len(), 1);
    assert_eq!((all[0].0.as_str(), all[0].1), ("IgnoredMember", "info"));
    assert_eq!(
        diagnostics.iter().next().map(|d| d.severity),
        Some(Severity::Info)
    );
}

#[test]
fn optional_dependency_dropped_by_a_gate_is_an_info_deviation() {
    // Vanilla: silent.
    let text = comprehensive(
        "p",
        JS,
        r#"{ "module_name": "@minecraft/server", "version": "beta", "optional": true }"#,
    );
    let found = discover(&text);
    assert_eq!(
        brief(found.diagnostics()),
        [row("DroppedField", "info", "dependencies/0")]
    );
    assert!(found.plugin().expect("runs").dependencies.is_empty());
}

#[test]
fn internal_apis_are_refused_unless_the_build_allows_them_deviation() {
    // Vanilla's 1.26.20 development build allows them; ours follows the published server.
    let text = legacy(
        "p",
        1,
        JS,
        r#"{ "module_name": "@minecraft/server", "version": "internal" }"#,
    );
    assert_eq!(
        brief(discover(&text).diagnostics()),
        [row("ScriptModulePermission", "error", "dependencies/0")]
    );
    let allowed = discover_with(&text, &main_js(), |cx| cx.allow_internal = true);
    assert_eq!(brief(allowed.diagnostics()), []);
    assert_eq!(
        allowed.plugin().expect("runs").dependencies[0]
            .resolved
            .to_string(),
        "2.9.0-internal"
    );
}

#[test]
fn alpha_dependency_is_an_info_deviation() {
    // Vanilla: not gated and no message.
    let text = legacy(
        "p",
        1,
        JS,
        r#"{ "module_name": "@minecraft/server", "version": "3.0.0-alpha" }"#,
    );
    let found = discover(&text);
    assert_eq!(
        brief(found.diagnostics()),
        [row("ScriptAlphaDependency", "info", "dependencies/0")]
    );
    assert!(found.plugin().is_some());
    // A tag that only starts with `alpha` is another tag.
    let other = legacy(
        "p",
        1,
        JS,
        r#"{ "module_name": "@minecraft/server", "version": "3.0.0-alpha.1" }"#,
    );
    assert!(
        !listed(discover(&other).diagnostics())
            .iter()
            .any(|(code, ..)| code == "ScriptAlphaDependency")
    );
}

// ---- across packs: S2 ----

fn stack(packs: &[String]) -> Vec<Discovery> {
    let allowed = bds_allowed();
    let cx = ScriptContext {
        allowed_modules: Some(&allowed),
        ..ScriptContext::default()
    };
    let catalog = bds_catalog();
    let mut discovery = ScriptDiscovery::new(&catalog);
    packs
        .iter()
        .map(|text| discovery.discover(&module(text), &main_js(), &cx))
        .collect()
}

/// A pack whose script module has the UUID and version of every other pack made here.
fn twin(pack: &str, module_members: &str, dependencies: &str) -> String {
    legacy(pack, 1, module_members, dependencies).replace(
        "11111111-1111-1111-1111-111111111101",
        &format!("{:0>8}-1111-1111-1111-111111111101", pack.len()),
    )
}

#[test]
fn s2_a_module_an_earlier_pack_brought_is_skipped() {
    let server = r#"{ "module_name": "@minecraft/server", "version": "2.0.0" }"#;
    let found = stack(&[twin("top", JS, server), twin("lower", JS, server)]);
    assert_eq!(brief(found[0].diagnostics()), []);
    assert!(found[0].plugin().is_some());
    assert!(found[1].plugin().is_none());
    assert_eq!(
        listed(found[1].diagnostics()),
        [(
            "DuplicateIdentifier".to_owned(),
            "warning",
            "modules/0".to_owned(),
            "Plugin [lower - 3.2.1] - module uuid [22222222-2222-2222-2222-222222222201] with version [3.2.1] already exists, skipping."
                .to_owned(),
        )]
    );
}

#[test]
fn s2_three_packs_with_one_module_run_the_first_only() {
    let found = stack(&[twin("a", JS, ""), twin("bb", JS, ""), twin("ccc", JS, "")]);
    let runs: Vec<bool> = found.iter().map(|d| d.plugin().is_some()).collect();
    assert_eq!(runs, [true, false, false]);
    for later in &found[1..] {
        assert_eq!(
            brief(later.diagnostics()),
            [row("DuplicateIdentifier", "warning", "modules/0")]
        );
    }
}

#[test]
fn s2_another_version_or_another_uuid_is_another_plugin() {
    let other_version = twin("bb", JS, "").replace("[3, 2, 1]", "[3, 2, 2]");
    let other_uuid = legacy("ccc", 2, JS, "");
    let found = stack(&[twin("a", JS, ""), other_version, other_uuid]);
    let runs: Vec<bool> = found.iter().map(|d| d.plugin().is_some()).collect();
    assert_eq!(runs, [true, true, true]);
}

#[test]
fn s2_a_plugin_whose_context_failed_still_exists() {
    // The top pack's plugin is discovered and fails at context creation (R1): the plugin
    // exists, so the lower pack's equal module is skipped, in two and in three packs.
    let unshipped = r#"{ "module_name": "@minecraft/server", "version": "9.0.0" }"#;
    let found = stack(&[
        twin("a", JS, unshipped),
        twin("bb", JS, ""),
        twin("ccc", JS, ""),
    ]);
    assert_eq!(
        brief(found[0].diagnostics()),
        [row(
            "ScriptModuleVersionUnavailable",
            "error",
            "dependencies/0"
        )]
    );
    for later in &found[1..] {
        assert!(later.plugin().is_none());
        assert_eq!(
            brief(later.diagnostics()),
            [row("DuplicateIdentifier", "warning", "modules/0")]
        );
    }
}

#[test]
fn s2_a_module_discovery_rejected_is_no_plugin_and_the_next_pack_runs() {
    // Vanilla's rule is "a plugin with this UUID and version already exists": a module that
    // discovery rejected made no plugin, so the next pack's equal module is discovered, and
    // the third is then skipped.
    let gated = r#"{ "module_name": "@minecraft/server", "version": "beta" }"#;
    let found = stack(&[
        twin("a", JS, gated),
        twin("bb", JS, ""),
        twin("ccc", JS, ""),
    ]);
    assert_eq!(
        brief(found[0].diagnostics()),
        [row("MissingToggles", "error", "dependencies/0")]
    );
    assert!(found[0].plugin().is_none());
    assert!(found[1].plugin().is_some());
    assert_eq!(brief(found[1].diagnostics()), []);
    assert!(found[2].plugin().is_none());
    assert_eq!(
        brief(found[2].diagnostics()),
        [row("DuplicateIdentifier", "warning", "modules/0")]
    );
}

#[test]
fn s2_a_module_whose_main_file_is_missing_still_exists() {
    // Vanilla builds the plugin before `ValidatePlugin` looks for the main file, and the
    // folder names no fall-back for S5: the module that won its UUID and version keeps it,
    // so the lower pack's equal module is skipped, in two packs and in three.
    let missing = r#", "entry": "scripts/missing.js""#;
    for lower in [
        vec![twin("bb", JS, "")],
        vec![twin("bb", JS, ""), twin("ccc", JS, "")],
    ] {
        let mut packs = vec![twin("a", missing, "")];
        packs.extend(lower);
        let found = stack(&packs);
        assert_eq!(
            listed(found[0].diagnostics()),
            [(
                "UnresolvedReference".to_owned(),
                "error",
                "modules/0/entry".to_owned(),
                "Plugin [a - 3.2.1] - does not contain main file [missing.js].".to_owned(),
            )]
        );
        assert!(found[0].plugin().is_none());
        for later in &found[1..] {
            assert!(later.plugin().is_none(), "no pack below runs the module");
            assert_eq!(
                brief(later.diagnostics()),
                [row("DuplicateIdentifier", "warning", "modules/0")]
            );
        }
    }
}

#[test]
fn s2_a_script_and_a_client_script_twin_do_not_shadow_each_other() {
    // Vanilla discovers the plugins of one resource type at a time (the server's manager
    // asks for `script`, the client's for `client_script`), so "already exists" is per
    // target: one discovery that is handed both keeps them apart.
    let client = |pack: &str| {
        twin(pack, JS, "").replace(r#""type": "script""#, r#""type": "client_script""#)
    };
    for packs in [
        [twin("a", JS, ""), client("bb")],
        [client("a"), twin("bb", JS, "")],
    ] {
        let found = stack(&packs);
        for one in &found {
            assert_eq!(brief(one.diagnostics()), []);
            assert!(one.plugin().is_some(), "both targets run their module");
        }
    }
    // Within one target the later module is skipped, on either side.
    let found = stack(&[
        twin("a", JS, ""),
        client("bb"),
        twin("ccc", JS, ""),
        client("dddd"),
    ]);
    let runs: Vec<bool> = found.iter().map(|d| d.plugin().is_some()).collect();
    assert_eq!(runs, [true, true, false, false]);
    for later in &found[2..] {
        assert_eq!(
            brief(later.diagnostics()),
            [row("DuplicateIdentifier", "warning", "modules/0")]
        );
    }
}

#[test]
fn s1_the_later_modules_are_skipped_also_when_the_first_is_disabled() {
    let text = legacy("p", 1, r#", "language": "typescript", "entry": "scripts/main.js""#, "").replace(
        r#""modules": [ "#,
        r#""modules": [ { "type": "script", "uuid": "44444444-4444-4444-4444-444444444444", "version": [9, 9, 9],
                          "entry": "scripts/main.js" }, "#,
    );
    // The first module is now the inserted one, which runs; the second (typescript) is skipped
    // and its language is never looked at.
    let found = discover(&text);
    assert_eq!(
        listed(found.diagnostics()),
        [(
            "MultipleScriptModules".to_owned(),
            "warning",
            "modules/1".to_owned(),
            "Plugin [p - 3.2.1] - skipped, only one script module allowed per pack.".to_owned(),
        )]
    );
    assert_eq!(
        found.plugin().expect("runs").module_version.to_string(),
        "9.9.9"
    );

    // The other way round: the first is disabled by its language, the second still skipped.
    let swapped = legacy("p", 1, r#", "language": "typescript", "entry": "scripts/main.js""#, "").replace(
        r#" } ],
  "dependencies""#,
        r#" }, { "type": "script", "uuid": "44444444-4444-4444-4444-444444444444", "version": [9, 9, 9],
                 "entry": "scripts/main.js" } ],
  "dependencies""#,
    );
    let found = discover(&swapped);
    assert!(found.plugin().is_none());
    assert_eq!(
        brief(found.diagnostics()),
        [
            row("InvalidEnumValue", "error", "modules/0/language"),
            row("MultipleScriptModules", "warning", "modules/1"),
        ]
    );
}

// ---- the plugin ----

#[test]
fn the_plugin_carries_what_the_host_needs() {
    let text = legacy(
        "p",
        1,
        JS,
        r#"{ "module_name": "@minecraft/server", "version": "2.0.0" }"#,
    )
    .replace(
        r#""dependencies""#,
        r#""capabilities": ["script_eval"], "dependencies""#,
    );
    let found = discover(&text);
    let plugin = found.plugin().expect("runs");
    assert_eq!(plugin.pack_name, "p");
    assert_eq!(plugin.runtime, ScriptRuntime::JavaScript);
    assert_eq!(plugin.main.as_str(), "main.js");
    assert_eq!(plugin.entry, "scripts/main.js");
    assert_eq!(plugin.name, "p - 3.2.1");
    assert!(plugin.script_eval);
    assert_eq!(
        plugin
            .min_engine_version
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("1.21.0")
    );
    let dependency = &plugin.dependencies[0];
    assert_eq!(
        dependency
            .module_uuid
            .map(|uuid| uuid.to_string())
            .as_deref(),
        Some(SERVER_UUID)
    );
    assert_eq!(dependency.requested.to_string(), "2.0.0");
    assert_eq!(dependency.resolved.to_string(), "2.8.0");
    assert!(!dependency.optional);
}

#[test]
fn a_context_for_a_pack_takes_its_toggles_from_the_load_context() {
    let mut cx = LoadContext::detached();
    assert_eq!(ScriptContext::for_pack(&cx), ScriptContext::default());
    cx.beta_apis = true;
    cx.is_base_game_pack = true;
    let script = ScriptContext::for_pack(&cx);
    assert!(script.beta_apis && script.is_package_origin);
    assert!(!script.editor_world && !script.allow_internal);
    assert_eq!(script.allowed_modules, None);
}

// ---- determinism ----

#[test]
fn the_same_input_prints_the_same_debug_text() {
    // The script files in whatever order the pack lists them.
    let forward = ScriptFiles::from_pack_paths(["scripts/a.js", "scripts/b.js", "scripts/c/d.js"]);
    let backward = ScriptFiles::from_pack_paths([
        "scripts/c/d.js",
        "scripts/b.js",
        "scripts/a.js",
        "scripts/a.js",
    ]);
    assert_eq!(forward, backward);
    assert_eq!(format!("{forward:?}"), format!("{backward:?}"));
    assert_eq!(backward.len(), 3);

    // A catalog and a discovery, built twice.
    let print = || {
        let catalog = bds_catalog();
        let text = legacy(
            "p",
            1,
            JS,
            r#"{ "module_name": "@minecraft/server", "version": "1.5.0" },
               { "module_name": "@minecraft/server", "version": "9.0.0" },
               { "module_name": "@minecraft/server-ui", "version": "2.0.0" }"#,
        );
        let module = module(&text);
        let mut discovery = ScriptDiscovery::new(&catalog);
        let found = discovery.discover(&module, &main_js(), &ScriptContext::default());
        format!("{catalog:?}\n{module:?}\n{found:?}\n{discovery:?}")
    };
    let first = print();
    assert!(first.contains("ScriptModuleVersionConflict"));
    for _ in 0..8 {
        assert_eq!(print(), first);
    }
    let shipped = format!(
        "{:?}",
        bedrock_addon::families::script::ModuleCatalog::shipped()
    );
    assert_eq!(
        format!(
            "{:?}",
            bedrock_addon::families::script::ModuleCatalog::shipped()
        ),
        shipped
    );
}

// ---- "Not diagnosed" ----

#[test]
fn a_script_module_beside_a_data_module_is_found_at_its_index() {
    // Q38: a `data` and a `script` module in one pack are normal.
    let text = legacy(
        "p",
        1,
        r#", "description": "never read by the script system", "entry": "scripts/main.js""#,
        r#"{ "module_name": "@minecraft/server", "version": "2.0.0" },
           { "module_name": "@minecraft/server", "version": "2.0.0" }"#,
    )
    .replace(
        r#""modules": [ "#,
        r#""modules": [ { "type": "data", "uuid": "55555555-5555-5555-5555-555555555555", "version": [1, 0, 0] }, "#,
    );
    let (parsed, diagnostics) = parse(&text).expect("loads");
    assert_eq!(brief(&diagnostics), []);
    let parsed = parsed.expect("a script module");
    assert_eq!(parsed.path.to_string(), "modules/1");
    assert!(parsed.skipped.is_empty());
    assert_eq!(parsed.language, None);
    // The same dependency twice is kept twice and is no finding (Q19).
    assert_eq!(parsed.dependencies.len(), 2);
    let found = discover(&text);
    assert_eq!(brief(found.diagnostics()), []);
    assert_eq!(found.plugin().expect("runs").dependencies.len(), 2);
}

#[test]
fn script_eval_without_a_script_and_a_script_without_it_are_no_finding() {
    let with = legacy("p", 1, JS, "").replace(
        r#""dependencies""#,
        r#""capabilities": ["script_eval"], "dependencies""#,
    );
    assert!(module(&with).script_eval);
    assert_eq!(brief(discover(&with).diagnostics()), []);
    let without = legacy("p", 1, JS, "");
    assert!(!module(&without).script_eval);
    assert_eq!(brief(discover(&without).diagnostics()), []);
    assert!(!discover(&without).plugin().expect("runs").script_eval);
}

#[test]
fn s10_the_permission_list_of_a_discovery_is_the_configuration_s_for_the_module() {
    use bedrock_addon::families::script::config::{ConfigFolder, ScriptConfig, ScriptConfigFiles};
    // The script module's own folder when the server has one, else `default`: the effective
    // list of `ScriptConfig::for_module`, handed to the discovery as its gate.
    let text = legacy(
        "p",
        1,
        JS,
        r#"{ "module_name": "@minecraft/server-ui", "version": "2.0.0" },
           { "module_name": "@minecraft/server", "version": "2.0.0" }"#,
    );
    let script = module(&text);
    let folder = |name: &str, permissions: &str| {
        let mut folder = ConfigFolder::new(name);
        folder.permissions = Some(bytes::Bytes::copy_from_slice(permissions.as_bytes()));
        folder
    };
    let run = |folders: Vec<ConfigFolder>| {
        let config = ScriptConfig::parse(&ScriptConfigFiles {
            folders,
            ..ScriptConfigFiles::default()
        });
        let cx = ScriptContext::default().with_config(&config, script.module_uuid);
        let found = ScriptDiscovery::new(&bds_catalog()).discover(&script, &main_js(), &cx);
        (brief(found.diagnostics()), found.plugin().is_some())
    };
    let server_only = r#"{"allowed_modules": ["@minecraft/server"]}"#;
    let both = r#"{"allowed_modules": ["@minecraft/server", "@minecraft/server-ui"]}"#;
    let refused = (
        vec![row("ScriptModulePermission", "error", "dependencies/0")],
        false,
    );
    assert_eq!(run(vec![folder("default", server_only)]), refused);
    assert_eq!(run(vec![folder("default", both)]), (Vec::new(), true));
    // The module's own folder stands in for `default`, either way round.
    let own = script.module_uuid.to_string();
    assert_eq!(
        run(vec![folder("default", server_only), folder(&own, both)]),
        (Vec::new(), true)
    );
    assert_eq!(
        run(vec![folder("default", both), folder(&own, server_only)]),
        refused
    );
    // No configuration at all is an empty `default`: every module is refused (D6).
    assert!(!run(Vec::new()).1);
    // No list (a world no dedicated server hosts) refuses nothing.
    let found = ScriptDiscovery::new(&bds_catalog()).discover(
        &script,
        &main_js(),
        &ScriptContext::default(),
    );
    assert!(found.plugin().is_some());
}

#[test]
fn a_syntax_fault_in_the_copy_of_a_dependency_member_that_loses_refuses_the_manifest() {
    // Of a member written twice the last stands, and the text is still read whole: an
    // earlier copy that is no JSON refuses the manifest, and the script side reads no form
    // from a text the manifest reader did not take.
    let faulty = legacy(
        "p",
        1,
        JS,
        r#"{ "module_name": "@minecraft/server", "version": tru, "version": "2.0.0" }"#,
    );
    let rejected = parse(&faulty).expect_err("the manifest is refused");
    assert_eq!(
        rejected.first_error().map(|f| f.code.name()),
        Some("MalformedInput")
    );
    // The same manifest with a well-formed copy that loses reads the last.
    let sound = legacy(
        "p",
        1,
        JS,
        r#"{ "module_name": "@minecraft/server", "version": [9, 9], "version": "2.0.0" }"#,
    );
    let (script, _) = parse(&sound).expect("the manifest loads");
    assert_eq!(script.expect("a script module").dependencies.len(), 1);
}

#[test]
fn a_manifest_whose_dependencies_were_edited_after_reading_ignores_the_stale_places() {
    use bedrock_addon::manifest::Dependency;
    // The reader drops the first element, so the places are the file's (`dependencies/1`, ...)
    // and no longer the positions the manifest holds its dependencies at.
    let text = legacy(
        "p",
        1,
        JS,
        &format!(
            r#"{{ "module_name": "@minecraft/server", "uuid": "{SERVER_UUID}", "version": "2.0.0" }},
               {{ "module_name": "@minecraft/server", "version": "beta" }},
               {{ "uuid": "{SERVER_UI_UUID}", "version": [0, 1, 0] }}"#
        ),
    );
    let mut manifest = Manifest::parse(
        &PackPath::from_static("manifest.json"),
        text.as_bytes(),
        &bedrock_addon::manifest::CapabilityRegistry::default(),
    )
    .expect("loads")
    .into_value();
    let read = ScriptModule::from_manifest(&manifest).expect("a script module");
    assert_eq!(read.dependencies[0].path.to_string(), "dependencies/1");
    // Edited by hand: a dependency is pushed, so the places no longer match the list.
    let pushed = manifest.dependencies[0].clone();
    manifest.dependencies.push(pushed);
    let edited = ScriptModule::from_manifest(&manifest).expect("a script module");
    let held: Vec<String> = manifest
        .dependencies
        .iter()
        .enumerate()
        .filter(|(_, dependency)| matches!(dependency, Dependency::NativeModule { .. }))
        .map(|(index, _)| format!("dependencies/{index}"))
        .collect();
    let paths: Vec<String> = edited
        .dependencies
        .iter()
        .map(|dependency| dependency.path.to_string())
        .collect();
    assert_eq!(
        paths, held,
        "a stale place must give way to the position the list holds"
    );
}
