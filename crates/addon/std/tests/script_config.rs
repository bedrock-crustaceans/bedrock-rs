//! `read_script_config`, `read_script_config_files` and `read_server_properties`: a server's
//! `config/` folder and `server.properties` read from disk, and the smoke test over a dedicated
//! server install of the game.
//!
//! Run the smoke test with
//! `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test script_config`; without
//! the variable it skips with a message.

#![cfg(feature = "script")]

#[macro_use]
mod common;

use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::families::script::{
    CatalogEntry, CatalogModule, ModuleCatalog, ScriptContext, ScriptDiscovery, ScriptFiles,
    ScriptModule, ScriptPlugin, ScriptVersion,
};
use bedrock_addon::families::server_config::{
    CommandPermissionLevel, ConfigSource, ScriptConfig, ScriptProperties, ServerNet,
};
use bedrock_addon_std::{
    ReadError, read_script_config, read_script_config_files, read_server_properties,
};

use common::game_files;

const MODULE: &str = "a0000000-0000-4000-8000-000000000001";
const VARS: &str = "426f63cd-dd57-5b19-8d35-d73e1b7b1cc5";

/// The server root of a fixture of the family, in the `bedrock_addon` crate.
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../addon/tests/fixtures/script/script_config")
        .join(name)
}

/// The plugin of a pack `probe` whose script module depends on `module`: its manifest read
/// by the script module's reader and discovered without a permission list, on an engine
/// that serves the module.
fn plugin(module: &str) -> ScriptPlugin {
    let manifest = format!(
        r#"{{
    "format_version": 2,
    "header": {{ "name": "probe", "description": "",
        "uuid": "b0000000-0000-4000-8000-000000000001", "version": [1, 0, 0],
        "min_engine_version": [1, 21, 0] }},
    "modules": [{{ "type": "script", "uuid": "{MODULE}", "version": [1, 0, 0],
        "entry": "scripts/main.js" }}],
    "dependencies": [{{ "module_name": "{module}", "version": "1.0.0" }}]
}}"#
    );
    let read = ScriptModule::parse(
        &bedrock_addon::storage::PackPath::from_static("manifest.json"),
        manifest.as_bytes(),
        &bedrock_addon::load::LoadContext::detached(),
    )
    .expect("a manifest");
    let script = read.value().clone().expect("a script module");
    let mut catalog = ModuleCatalog::new();
    catalog.insert(
        module,
        CatalogModule {
            versions: vec![CatalogEntry::new(ScriptVersion::new(1, 0, 0))],
            ..CatalogModule::default()
        },
    );
    ScriptDiscovery::new(&catalog)
        .discover(
            &script,
            &ScriptFiles::from_members(["main.js"]),
            &ScriptContext::default(),
        )
        .plugin()
        .cloned()
        .expect("the plugin")
}

/// (code, severity, file) of each finding.
fn findings(diagnostics: &Diagnostics) -> Vec<(&'static str, Severity, String)> {
    diagnostics
        .iter()
        .map(|d| {
            let file = d.location.path.as_ref().map(ToString::to_string);
            (d.code.id(), d.severity, file.unwrap_or_default())
        })
        .collect()
}

#[test]
fn a_config_folder_is_read_folder_by_folder() {
    let dir = fixture("sc05_no_merge_variables").join("config");
    let config = read_script_config(&dir).unwrap();
    assert!(config.diagnostics().is_empty());
    assert_eq!(config.default.source, ConfigSource::Default);
    assert_eq!(
        config.default.variables.keys().collect::<Vec<_>>(),
        ["default_only"]
    );
    assert_eq!(config.packs.keys().collect::<Vec<_>>(), [VARS]);
    let pack = &config.packs[VARS];
    assert_eq!(pack.source, ConfigSource::Folder(VARS.to_owned()));
    assert_eq!(pack.variables.keys().collect::<Vec<_>>(), ["own"]);
    assert_eq!(pack.secrets["s"].expose(), "x");
    assert_eq!(
        pack.permissions.allowed_modules,
        ["@minecraft/server", "@minecraft/server-admin"]
    );
    // The same value the pure parser gives for the files as read.
    let files = read_script_config_files(&dir).unwrap();
    assert_eq!(files.folders.len(), 2);
    assert!(files.unreadable.is_empty());
    assert_eq!(ScriptConfig::parse(&files), config);
}

#[test]
fn sc08_a_missing_folder_is_an_empty_configuration() {
    let config = read_script_config(fixture("sc08_no_config").join("config")).unwrap();
    assert_eq!(config, ScriptConfig::default());
    assert!(config.diagnostics().is_empty());
}

#[test]
fn a_config_path_that_is_a_file_is_an_io_error() {
    let path = fixture("sc10_stray_file").join("config/afile.json");
    match read_script_config(&path) {
        Err(ReadError::Io(error)) => assert_eq!(error.path, path),
        other => panic!("{other:?}"),
    }
}

#[test]
fn sc10_files_beside_the_folders_are_not_read() {
    let files = read_script_config_files(fixture("sc10_stray_file").join("config")).unwrap();
    assert_eq!(files.folders.len(), 1);
    assert_eq!(&*files.folders[0].name, "default");
    assert!(files.commands.is_none());
    let config = ScriptConfig::parse(&files);
    assert!(config.diagnostics().is_empty());
    assert!(config.packs.is_empty());
}

#[test]
fn commands_json_is_read_at_the_root_only() {
    let config = read_script_config(fixture("sc33_commands_json_valid").join("config")).unwrap();
    let commands = config.commands.as_ref().unwrap();
    assert_eq!(
        commands.permission_levels["list"],
        CommandPermissionLevel::Internal
    );
    // sc36: the one in `default/` is seen and not read.
    let dir = fixture("sc36_commands_json_in_default").join("config");
    let files = read_script_config_files(&dir).unwrap();
    assert!(files.commands.is_none());
    assert!(files.folders[0].stray_commands);
    let config = read_script_config(&dir).unwrap();
    assert_eq!(config.commands, None);
    assert_eq!(
        findings(config.diagnostics()),
        [(
            "core/ignored_file",
            Severity::Info,
            "config/default/commands.json".to_owned()
        )]
    );
}

#[test]
fn every_folder_is_read_whatever_its_name() {
    let config = read_script_config(fixture("sc09_non_uuid_folder").join("config")).unwrap();
    assert_eq!(config.packs.keys().collect::<Vec<_>>(), ["not-a-uuid"]);
    assert_eq!(
        findings(config.diagnostics()),
        [(
            "server_config/config_parse_failed_malformed_input",
            Severity::Error,
            "config/not-a-uuid/permissions.json".to_owned()
        )]
    );
    // An upper-case folder keeps its spelling.
    let config = read_script_config(fixture("sc06_upper_case_folder").join("config")).unwrap();
    assert_eq!(
        config.packs.keys().collect::<Vec<_>>(),
        [&MODULE.to_uppercase()]
    );
}

/// A copy of fixture `name`'s `config/` in a temporary directory.
fn copy_of(name: &str) -> tempfile::TempDir {
    let from = fixture(name).join("config");
    let to = tempfile::tempdir().unwrap();
    for folder in fs::read_dir(&from).unwrap() {
        let folder = folder.unwrap().path();
        let target = to.path().join(folder.file_name().unwrap());
        fs::create_dir(&target).unwrap();
        for file in fs::read_dir(&folder).unwrap() {
            let file = file.unwrap().path();
            fs::copy(&file, target.join(file.file_name().unwrap())).unwrap();
        }
    }
    to
}

#[test]
fn a_file_that_cannot_be_read_is_a_finding_and_the_folder_is_used() {
    // A directory where the file should be cannot be read as a file on any platform.
    let config_dir = copy_of("sc25_unreadable");
    fs::create_dir(config_dir.path().join(MODULE).join("variables.json")).unwrap();
    fs::create_dir(config_dir.path().join("commands.json")).unwrap();
    let files = read_script_config_files(config_dir.path()).unwrap();
    assert_eq!(files.unreadable.len(), 2);
    let config = read_script_config(config_dir.path()).unwrap();
    let pack = &config.packs[MODULE];
    assert_eq!(pack.permissions.allowed_modules, ["@minecraft/server"]);
    assert!(pack.variables.is_empty());
    assert_eq!(config.commands, None);
    assert_eq!(
        findings(config.diagnostics()),
        [
            (
                "server_config/config_open_failed",
                Severity::Error,
                format!("config/{MODULE}/variables.json")
            ),
            (
                "server_config/config_open_failed",
                Severity::Error,
                "config/commands.json".to_owned()
            )
        ]
    );
    assert_eq!(
        config.diagnostics().first_error().unwrap().code.to_string(),
        format!("Failed to open file 'config/{MODULE}/variables.json'")
    );
}

#[cfg(unix)]
#[test]
fn sc25_unreadable_a_file_of_mode_000_is_a_read_error() {
    use std::os::unix::fs::PermissionsExt;
    let config_dir = copy_of("sc25_unreadable");
    let path = config_dir.path().join(MODULE).join("variables.json");
    fs::write(&path, "{\"v\": 1}").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read(&path).is_ok() {
        eprintln!("skipped: this user reads a file of mode 000 (root)");
        return;
    }
    let config = read_script_config(config_dir.path()).unwrap();
    assert!(config.packs[MODULE].variables.is_empty());
    let finding = config.diagnostics().first_error().unwrap();
    assert_eq!(finding.code.id(), "server_config/config_open_failed");
    assert_eq!(
        finding.detail.as_deref(),
        Some(&bedrock_addon::diagnostic::DiagnosticDetail::Io(
            std::io::ErrorKind::PermissionDenied
        ))
    );
    assert_eq!(
        finding.code.to_string(),
        format!("Failed to open file 'config/{MODULE}/variables.json'")
    );
    assert_eq!(config.diagnostics().len(), 1);
}

#[test]
fn server_properties_are_read_and_parsed() {
    let path = fixture("sp12_enable_script").join("server.properties");
    let properties = read_server_properties(&path).unwrap();
    assert_eq!(properties.get("enable-script"), Some("false"));
    let mut findings = Diagnostics::default();
    assert!(!properties.script(&mut findings).enable_script);
    assert!(findings.is_empty());
}

#[test]
fn a_missing_server_properties_holds_no_line() {
    let dir = tempfile::tempdir().unwrap();
    let properties = read_server_properties(dir.path().join("server.properties")).unwrap();
    assert!(properties.entries.is_empty());
    let mut findings = Diagnostics::default();
    assert_eq!(
        properties.script(&mut findings),
        ScriptProperties::default()
    );
    // A path that cannot be read as a file is an error.
    match read_server_properties(dir.path()) {
        Err(ReadError::Io(error)) => assert_eq!(error.path, dir.path()),
        other => panic!("{other:?}"),
    }
}

/// The server install: its `config/` folder and its `server.properties` read with zero
/// errors, and in fact without a finding.
#[test]
fn smoke_the_server_install_reads_without_a_finding() {
    let Some(server) = game_files::source("bds-1.26.36.1") else {
        game_files::skipped("bds-1.26.36.1");
        return;
    };
    let files = read_script_config_files(server.join("config")).unwrap();
    let config = read_script_config(server.join("config")).unwrap();
    let properties = read_server_properties(server.join("server.properties")).unwrap();
    let mut property_findings = properties.diagnostics().clone();
    let script = properties.script(&mut property_findings);
    println!(
        "smoke script_config: server install config/: {} folder(s), {} file(s), commands.json {}, {} allowed module(s), {} finding(s); server.properties: {} applied line(s), {} finding(s)",
        files.folders.len(),
        files
            .folders
            .iter()
            .map(|f| usize::from(f.permissions.is_some())
                + usize::from(f.variables.is_some())
                + usize::from(f.secrets.is_some()))
            .sum::<usize>(),
        if files.commands.is_some() {
            "present"
        } else {
            "absent"
        },
        config.default.permissions.allowed_modules.len(),
        config.diagnostics().len(),
        properties.entries.len(),
        property_findings.len(),
    );
    // Exact counts: one folder (`default`) with one file, six allowed modules, 40 lines.
    assert_eq!(files.folders.len(), 1);
    assert!(files.unreadable.is_empty());
    assert!(files.commands.is_none());
    assert!(
        config.diagnostics().is_empty(),
        "{:?}",
        config.diagnostics()
    );
    assert!(config.packs.is_empty());
    assert_eq!(config.commands, None);
    assert_eq!(
        config.default.permissions.allowed_modules,
        [
            "@minecraft/server-gametest",
            "@minecraft/server",
            "@minecraft/server-ui",
            "@minecraft/server-admin",
            "@minecraft/server-editor",
            "@minecraft/debug-utilities"
        ]
    );
    assert_eq!(
        config.default.permissions.server_net,
        ServerNet::Unrestricted
    );
    assert!(config.default.variables.is_empty());
    assert!(config.default.secrets.is_empty());
    assert!(property_findings.is_empty(), "{property_findings:?}");
    assert_eq!(properties.entries.len(), 40);
    assert_eq!(script, ScriptProperties::default());
    // What the shipped configuration means for a pack: `@minecraft/server` yes, server-net no.
    assert!(config.filter(&plugin("@minecraft/server")).runs);
    assert!(!config.filter(&plugin("@minecraft/server-net")).runs);
    assert!(
        config
            .unmatched_folders(&[plugin("@minecraft/server")])
            .is_empty()
    );
}
