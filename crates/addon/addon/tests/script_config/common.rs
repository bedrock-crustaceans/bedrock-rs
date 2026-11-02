//! The fixtures of `families/script_config`: a small server root per row (`config/…`,
//! `server.properties`), `expected.toml` with every finding (code, severity, phase, file, path
//! and, where the server has one, its text) and, for the rows that need a stack, `stack.toml`.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, Diagnostics, Phase};
use bedrock_addon::families::script::config::{
    COMMANDS_FILE, CONFIG_DIR, ConfigFolder, PERMISSIONS_FILE, PROPERTIES_FILE, PluginVerdict,
    SECRETS_FILE, ScriptConfig, ScriptConfigFiles, ScriptProperties, ServerProperties,
    VARIABLES_FILE,
};
use bedrock_addon::families::script::{
    CatalogModule, CatalogVersion, ModuleCatalog, ScriptContext, ScriptDiscovery, ScriptFiles,
    ScriptModule, ScriptPlugin, ScriptVersion,
};
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;
use bytes::Bytes;
use uuid::Uuid;

/// The module UUID most fixtures use for their pack.
pub const MODULE: &str = "a0000000-0000-4000-8000-000000000001";
/// The module UUID of the variables fixtures (the probe's).
pub const VARS: &str = "426f63cd-dd57-5b19-8d35-d73e1b7b1cc5";

/// The server root of fixture `name`.
pub fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/script/script_config")
        .join(name)
}

fn read(path: &Path) -> Option<Bytes> {
    fs::read(path).ok().map(Bytes::from)
}

/// The `config/` folder of fixture `name`, read as a driver reads it.
pub fn files(name: &str) -> ScriptConfigFiles {
    let dir = fixture(name).join(CONFIG_DIR);
    let mut files = ScriptConfigFiles {
        commands: read(&dir.join(COMMANDS_FILE)),
        ..ScriptConfigFiles::default()
    };
    let Ok(entries) = fs::read_dir(&dir) else {
        return files;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if !path.is_dir() {
            continue;
        }
        let mut folder = ConfigFolder::new(path.file_name().unwrap().to_str().unwrap());
        folder.permissions = read(&path.join(PERMISSIONS_FILE));
        folder.variables = read(&path.join(VARIABLES_FILE));
        folder.secrets = read(&path.join(SECRETS_FILE));
        folder.stray_commands = path.join(COMMANDS_FILE).exists();
        files.folders.push(folder);
    }
    files
}

/// The parsed `config/` folder of fixture `name`.
pub fn config(name: &str) -> ScriptConfig {
    ScriptConfig::parse(&files(name))
}

/// The `[[table]]`s of a fixture's TOML file, in order: `key = "string"`, `key = true` and
/// `key = false` lines; strings are JSON strings.
fn tables(path: &Path) -> Vec<(String, HashMap<String, String>)> {
    let text = fs::read_to_string(path).unwrap();
    let mut tables: Vec<(String, HashMap<String, String>)> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix("[[").and_then(|l| l.strip_suffix("]]")) {
            tables.push((name.to_owned(), HashMap::new()));
            continue;
        }
        let (key, value) = line.split_once(" = ").expect("key = value");
        let value = if value.starts_with('"') {
            bedrock_addon::json::from_str::<String>(value).unwrap()
        } else {
            value.to_owned()
        };
        tables.last_mut().unwrap().1.insert(key.to_owned(), value);
    }
    tables
}

/// The plugin of a pack called `name` (header `header`) whose script module `module` of
/// `version` depends on `dependencies` (`(module name, optional)`): its format-3 manifest
/// read by the script module's reader and discovered as a stack discovers it, without a
/// permission list, on an engine that serves each named module.
pub fn plugin(
    name: &str,
    version: &str,
    header: Uuid,
    module: Uuid,
    dependencies: &[(String, bool)],
) -> ScriptPlugin {
    let listed: Vec<String> = dependencies
        .iter()
        .map(|(module_name, optional)| {
            format!(
                r#"{{ "module_name": {module_name:?}, "version": "1.0.0", "optional": {optional} }}"#
            )
        })
        .collect();
    let manifest = format!(
        r#"{{
    "format_version": 3,
    "header": {{ "name": {name:?}, "description": "", "uuid": "{header}", "version": "1.0.0",
        "min_engine_version": "1.26.30" }},
    "modules": [{{ "type": "script", "uuid": "{module}", "version": {version:?},
        "entry": "scripts/main.js" }}],
    "dependencies": [{}]
}}"#,
        listed.join(", ")
    );
    let read = ScriptModule::parse(
        &PackPath::from_static("manifest.json"),
        manifest.as_bytes(),
        &LoadContext::detached(),
    )
    .unwrap_or_else(|rejected| panic!("{manifest}\n{rejected:?}"));
    let script = read.value().clone().expect("a script module");
    let mut catalog = ModuleCatalog::new();
    for (module_name, _) in dependencies {
        catalog.insert(
            module_name,
            CatalogModule {
                versions: vec![CatalogVersion::new(ScriptVersion::new(1, 0, 0))],
                ..CatalogModule::default()
            },
        );
    }
    // Every gate but the permission list is open: the list is what these tests are about.
    let cx = ScriptContext {
        beta_apis: true,
        editor_world: true,
        allow_internal: true,
        ..ScriptContext::default()
    };
    let found = ScriptDiscovery::new(&catalog).discover(
        &script,
        &ScriptFiles::from_members(["main.js"]),
        &cx,
    );
    found
        .plugin()
        .cloned()
        .unwrap_or_else(|| panic!("{name}: {:?}", found.diagnostics()))
}

/// The script modules of fixture `name`'s `stack.toml`.
pub fn plugins(name: &str) -> Vec<ScriptPlugin> {
    struct Pack {
        name: String,
        version: String,
        header: Uuid,
        module: Uuid,
        dependencies: Vec<(String, bool)>,
    }
    let mut packs: Vec<Pack> = Vec::new();
    for (table, mut keys) in tables(&fixture(name).join("stack.toml")) {
        match table.as_str() {
            "pack" => packs.push(Pack {
                name: keys.remove("name").unwrap(),
                version: keys.remove("version").unwrap(),
                header: keys["header_uuid"].parse().unwrap(),
                module: keys["script_module_uuid"].parse().unwrap(),
                dependencies: Vec::new(),
            }),
            "dependency" => packs.last_mut().unwrap().dependencies.push((
                keys.remove("module_name").unwrap(),
                keys["optional"] == "true",
            )),
            other => panic!("unknown table {other}"),
        }
    }
    packs
        .iter()
        .map(|pack| {
            plugin(
                &pack.name,
                &pack.version,
                pack.header,
                pack.module,
                &pack.dependencies,
            )
        })
        .collect()
}

/// One finding, in `expected.toml`'s terms.
#[derive(Clone, Debug)]
pub struct Finding {
    pub code: String,
    pub severity: String,
    pub phase: String,
    pub file: String,
    pub path: String,
    /// The code's text; in an expectation, `None` when the finding is ours and has no
    /// text of the server's to pin.
    pub text: Option<String>,
}

impl PartialEq for Finding {
    fn eq(&self, other: &Self) -> bool {
        let text = match (&self.text, &other.text) {
            (Some(a), Some(b)) => a == b,
            _ => true,
        };
        text && (
            &self.code,
            &self.severity,
            &self.phase,
            &self.file,
            &self.path,
        ) == (
            &other.code,
            &other.severity,
            &other.phase,
            &other.file,
            &other.path,
        )
    }
}

fn phase(phase: Phase) -> &'static str {
    match phase {
        Phase::Read | Phase::Parse => "parse",
        Phase::Validate => "validate",
        Phase::Link => "link",
        _ => "other",
    }
}

/// `diagnostics` in `expected.toml`'s terms.
pub fn seen<'a>(diagnostics: impl IntoIterator<Item = &'a Diagnostic>) -> Vec<Finding> {
    diagnostics
        .into_iter()
        .map(|d| Finding {
            code: d.code.name().to_owned(),
            severity: d.severity.to_string(),
            phase: phase(d.phase).to_owned(),
            file: d
                .location
                .path
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            path: d.location.json_path.to_string(),
            text: Some(d.code.to_string()),
        })
        .collect()
}

/// The findings `expected.toml` of fixture `name` lists for `phases`, in order.
pub fn expected(name: &str, phases: &[&str]) -> Vec<Finding> {
    tables(&fixture(name).join("expected.toml"))
        .into_iter()
        .map(|(table, mut keys)| {
            assert_eq!(table, "diagnostic");
            Finding {
                code: keys.remove("code").unwrap(),
                severity: keys.remove("severity").unwrap(),
                phase: keys.remove("phase").unwrap(),
                file: keys.remove("file").unwrap(),
                path: keys.remove("path").unwrap(),
                text: keys.remove("text"),
            }
        })
        .filter(|finding| phases.contains(&finding.phase.as_str()))
        .collect()
}

/// Requires `diagnostics` to be exactly the findings of fixture `name` for `phases`.
pub fn assert_findings(name: &str, phases: &[&str], diagnostics: &Diagnostics) {
    let seen = seen(diagnostics);
    let expected = expected(name, phases);
    assert_eq!(
        seen, expected,
        "{name}: the findings differ from expected.toml"
    );
}

/// Parses the `config/` folder of fixture `name` and requires its findings to be those of
/// `expected.toml` (phase `parse`).
pub fn check_config(name: &str) -> ScriptConfig {
    let config = config(name);
    assert_findings(name, &["parse"], config.diagnostics());
    config
}

/// Runs the stack of fixture `name` against its configuration: the folders no module uses,
/// then the dependency filter per pack. Requires the findings to be those of `expected.toml`
/// (phase `link`) and gives each pack's verdict.
pub fn check_stack(name: &str) -> (ScriptConfig, Vec<PluginVerdict>) {
    let config = config(name);
    let plugins = plugins(name);
    let mut findings = config.unmatched_folders(&plugins);
    let verdicts = config.filter_stack(&plugins);
    // One by one gives the same.
    let single: Vec<PluginVerdict> = plugins.iter().map(|p| config.filter(p)).collect();
    assert_eq!(verdicts, single);
    for verdict in &verdicts {
        findings.extend(verdict.diagnostics.iter().cloned());
    }
    assert_findings(name, &["link"], &findings);
    (config, verdicts)
}

/// Parses the `server.properties` of fixture `name` and reads its script keys; requires the
/// findings of both to be those of `expected.toml` (phases `parse`, then `validate`).
pub fn check_properties(name: &str) -> (ServerProperties, ScriptProperties) {
    let bytes = fs::read(fixture(name).join(PROPERTIES_FILE)).unwrap();
    let properties = ServerProperties::parse(&bytes);
    let mut findings = properties.diagnostics().clone();
    let script = properties.script(&mut findings);
    assert_findings(name, &["parse", "validate"], &findings);
    (properties, script)
}
