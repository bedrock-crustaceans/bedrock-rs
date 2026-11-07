//! The fixture runner: one folder per case under `tests/fixtures/script/script_module/`, with
//! the pack's `manifest.json` and files, and `expected.toml` saying what vanilla made of the
//! probe pack the case mirrors.
//!
//! `expected.toml` is read by the small reader below (the crate has no TOML dependency):
//! `key = "text" | true | false | ["text", …]` lines, `[[table]]` headers and `#` comments.

use std::fs;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon::families::script::{
    CatalogEntry, CatalogModule, Discovery, ModuleCatalog, ScriptContext, ScriptDiscovery,
    ScriptFiles, ScriptModule, ScriptPlugin, ScriptVersion,
};
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::FormatVersion;
use uuid::Uuid;

/// `allowed_modules` of BDS 1.26.36.1's `config/default/permissions.json`, the list every
/// probe ran under.
pub const BDS_ALLOWED_MODULES: [&str; 6] = [
    "@minecraft/server-gametest",
    "@minecraft/server",
    "@minecraft/server-ui",
    "@minecraft/server-admin",
    "@minecraft/server-editor",
    "@minecraft/debug-utilities",
];

/// The engine the probes ran on.
pub const BDS_ENGINE: FormatVersion = FormatVersion::new(1, 26, 36);

pub fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/script/script_module")
}

// ---- the TOML subset ----

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Text(String),
    Flag(bool),
    List(Vec<String>),
}

/// One table: its header (`""` for the root, `diagnostic`, `module`, `module.version`) and
/// its keys in file order.
#[derive(Clone, Debug, Default)]
pub struct Table {
    pub header: String,
    pub keys: Vec<(String, Value)>,
}

impl Table {
    pub fn text(&self, key: &str) -> Option<&str> {
        self.keys
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| match v {
                Value::Text(text) => text.as_str(),
                other => panic!("{key} is {other:?}, not text"),
            })
    }

    pub fn flag(&self, key: &str) -> Option<bool> {
        self.keys
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| match v {
                Value::Flag(flag) => *flag,
                other => panic!("{key} is {other:?}, not a flag"),
            })
    }

    pub fn list(&self, key: &str) -> Option<&[String]> {
        self.keys
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| match v {
                Value::List(list) => list.as_slice(),
                other => panic!("{key} is {other:?}, not a list"),
            })
    }
}

/// One quoted string at the start of `text`; the string and what follows it.
fn quoted(text: &str) -> (String, &str) {
    let mut out = String::new();
    let mut chars = text
        .strip_prefix('"')
        .expect("a quoted string")
        .char_indices();
    while let Some((at, c)) = chars.next() {
        match c {
            '"' => return (out, &text[at + 2..]),
            '\\' => match chars.next().expect("an escape").1 {
                'n' => out.push('\n'),
                other => out.push(other),
            },
            other => out.push(other),
        }
    }
    panic!("unterminated string in {text:?}");
}

pub fn read_toml(path: &Path) -> Vec<Table> {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut tables = vec![Table::default()];
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(header) = line.strip_prefix("[[").and_then(|l| l.strip_suffix("]]")) {
            tables.push(Table {
                header: header.to_owned(),
                keys: Vec::new(),
            });
            continue;
        }
        let (key, value) = line
            .split_once(" = ")
            .unwrap_or_else(|| panic!("{}: not `key = value`: {line}", path.display()));
        let value = match value {
            "true" => Value::Flag(true),
            "false" => Value::Flag(false),
            list if list.starts_with('[') => {
                let mut items = Vec::new();
                let mut rest = list[1..].trim_start();
                while !rest.starts_with(']') {
                    let (item, after) = quoted(rest);
                    items.push(item);
                    rest = after.trim_start().trim_start_matches(',').trim_start();
                }
                Value::List(items)
            }
            text => Value::Text(quoted(text).0),
        };
        tables
            .last_mut()
            .expect("the root table")
            .keys
            .push((key.to_owned(), value));
    }
    tables
}

// ---- the catalog ----

fn version(text: &str) -> ScriptVersion {
    ScriptVersion::parse(text).unwrap_or_else(|| panic!("{text} is no version"))
}

/// `<module> <version>` as its two parts.
pub fn module_and_version(text: &str) -> (String, ScriptVersion) {
    let (module, at) = text.split_once(' ').expect("`<module> <version>`");
    (module.to_owned(), version(at))
}

/// The catalog the probes ran against: `catalog_bds_1_26_36.toml`.
pub fn bds_catalog() -> ModuleCatalog {
    let mut catalog = ModuleCatalog::new();
    let mut module: Option<(String, CatalogModule)> = None;
    for table in read_toml(&fixtures().join("catalog_bds_1_26_36.toml")) {
        match table.header.as_str() {
            "" => {}
            "module" => {
                if let Some((name, done)) = module.take() {
                    catalog.insert(&name, done);
                }
                module = Some((
                    table.text("name").expect("name").to_owned(),
                    CatalogModule {
                        uuid: Some(
                            Uuid::parse_str(table.text("uuid").expect("uuid")).expect("a UUID"),
                        ),
                        bindings: table.text("bindings").map(str::to_owned),
                        versions: Vec::new(),
                    },
                ));
            }
            "module.version" => {
                let mut entry = CatalogEntry::new(version(table.text("version").expect("version")));
                entry.hidden = table.flag("hidden").unwrap_or(false);
                entry.requires = table
                    .list("requires")
                    .unwrap_or_default()
                    .iter()
                    .map(|text| module_and_version(text))
                    .collect();
                module.as_mut().expect("a module").1.versions.push(entry);
            }
            other => panic!("unknown table [[{other}]]"),
        }
    }
    if let Some((name, done)) = module {
        catalog.insert(&name, done);
    }
    catalog
}

// ---- one case ----

/// A finding as a case states it: code, severity, JSON path, and the text when the case
/// gives one.
#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    pub code: String,
    pub severity: String,
    pub path: String,
    pub text: Option<String>,
}

/// The variant name of a finding's code, as the crate gives it (`Code::name`): the
/// form an `expected.toml` states a code in.
pub fn code_name(diagnostic: &Diagnostic) -> String {
    diagnostic.code.id().to_owned()
}

pub fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Debug => "debug",
        Severity::Info => "info",
        Severity::Warning => "warning",
        Severity::Error => "error",
    }
}

fn findings(diagnostics: &Diagnostics, with_text: &[bool]) -> Vec<Finding> {
    diagnostics
        .iter()
        .enumerate()
        .map(|(i, d)| Finding {
            code: code_name(d),
            severity: severity_name(d.severity).to_owned(),
            path: d.location.json_path.to_string(),
            text: with_text
                .get(i)
                .copied()
                .unwrap_or(true)
                .then(|| d.code.to_string()),
        })
        .collect()
}

/// Every file of a case's folder as a pack path, sorted; `expected.toml` is not the pack's.
pub fn pack_paths(folder: &Path) -> Vec<String> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
        for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
            let path = entry.expect("a directory entry").path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let relative = path.strip_prefix(root).expect("below the root");
                out.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut paths = Vec::new();
    walk(folder, folder, &mut paths);
    paths.retain(|path| path != "expected.toml");
    paths.sort();
    paths
}

/// What a case's pack came to.
pub struct Outcome {
    /// The script side, when the manifest was accepted and has one.
    pub module: Option<ScriptModule>,
    /// Discovery, when it ran.
    pub discovery: Option<Discovery>,
    /// Every finding, stage 1 first.
    pub diagnostics: Diagnostics,
    /// The manifest was refused.
    pub rejected: bool,
}

/// Runs the case `name` with the context its `expected.toml` states.
pub fn run(name: &str) -> (Outcome, Vec<Table>) {
    let folder = fixtures().join(name);
    let expected = read_toml(&folder.join("expected.toml"));
    let root = &expected[0];
    let bytes = fs::read(folder.join("manifest.json"))
        .unwrap_or_else(|e| panic!("{name}/manifest.json: {e}"));
    let mut cx = LoadContext::detached();
    cx.engine_version = root.text("engine").map_or(BDS_ENGINE, |text| {
        bedrock_addon::version::SemVersion::parse(text)
            .expect("an engine version")
            .format_version()
    });
    let allowed: Vec<String> = BDS_ALLOWED_MODULES
        .iter()
        .map(|m| (*m).to_owned())
        .collect();
    let script_cx = ScriptContext {
        beta_apis: root.flag("beta_apis").unwrap_or(false),
        editor_world: root.flag("editor_world").unwrap_or(false),
        allow_internal: root.flag("allow_internal").unwrap_or(false),
        is_package_origin: root.flag("package_origin").unwrap_or(false),
        allowed_modules: match root.text("permissions") {
            None | Some("bds") => Some(&allowed),
            Some("none") => None,
            Some(other) => panic!("{name}: permissions = {other:?}"),
        },
    };
    let outcome = match ScriptModule::parse(&PackPath::from_static("manifest.json"), &bytes, &cx) {
        Err(rejected) => Outcome {
            module: None,
            discovery: None,
            diagnostics: rejected.diagnostics.clone(),
            rejected: true,
        },
        Ok(document) => {
            let mut diagnostics = document.diagnostics().clone();
            let module = document.into_value();
            let discovery = module.as_ref().map(|module| {
                let paths = pack_paths(&folder);
                let files = ScriptFiles::from_pack_paths(paths.iter().map(String::as_str));
                let catalog = bds_catalog();
                ScriptDiscovery::new(&catalog).discover(module, &files, &script_cx)
            });
            if let Some(discovery) = &discovery {
                diagnostics.extend(discovery.diagnostics().iter().cloned());
            }
            Outcome {
                module,
                discovery,
                diagnostics,
                rejected: false,
            }
        }
    };
    (outcome, expected)
}

/// The findings an `expected.toml` states, one per table after the root.
fn stated_findings(name: &str, expected: &[Table]) -> Vec<Finding> {
    expected[1..]
        .iter()
        .map(|table| {
            assert_eq!(
                table.header, "diagnostic",
                "{name}: table [[{}]]",
                table.header
            );
            Finding {
                code: table.text("code").expect("code").to_owned(),
                severity: table.text("severity").expect("severity").to_owned(),
                path: table.text("path").expect("path").to_owned(),
                text: table.text("text").map(str::to_owned),
            }
        })
        .collect()
}

/// Requires what the `scope` of an `expected.toml` says about the outcome.
fn check_scope(name: &str, scope: &str, outcome: &Outcome, plugin: Option<&ScriptPlugin>) {
    match scope {
        "pack" => {
            assert!(outcome.rejected, "{name}: the manifest must be refused");
            assert!(
                outcome.diagnostics.has_errors(),
                "{name}: a refusal has an error"
            );
        }
        "module" => {
            assert!(!outcome.rejected, "{name}: the pack must load");
            assert!(
                outcome.module.is_some(),
                "{name}: the manifest has a script module"
            );
            assert!(plugin.is_none(), "{name}: the script must be disabled");
            assert!(
                outcome.diagnostics.has_errors(),
                "{name}: a disabled script has an error"
            );
        }
        "dep" | "none" => {
            assert!(!outcome.rejected, "{name}: the pack must load");
            assert!(plugin.is_some(), "{name}: the script must run");
            assert!(!outcome.diagnostics.has_errors(), "{name}: no error");
        }
        other => panic!("{name}: scope = {other:?}"),
    }
}

/// Runs the case `name` and requires what its `expected.toml` states: the scope, every
/// finding in order (code, severity, path, and the text where one is given), and the
/// plugin's main file, capability and resolved dependencies.
pub fn check(name: &str) {
    let (outcome, expected) = run(name);
    let root = &expected[0];
    let stated = stated_findings(name, &expected);
    let with_text: Vec<bool> = stated.iter().map(|f| f.text.is_some()).collect();
    assert_eq!(
        findings(&outcome.diagnostics, &with_text),
        stated,
        "{name}: findings"
    );

    let scope = root.text("scope").expect("scope");
    let plugin = outcome.discovery.as_ref().and_then(Discovery::plugin);
    check_scope(name, scope, &outcome, plugin);

    let Some(plugin) = plugin else {
        for key in ["resolved", "requested", "optional", "main", "script_eval"] {
            assert!(
                !root.keys.iter().any(|(k, _)| k == key),
                "{name}: `{key}` stated for a script that does not run"
            );
        }
        return;
    };
    let module = outcome.module.as_ref().expect("a module");
    assert_eq!(plugin.pack_name, module.pack_name, "{name}: pack name");
    assert_eq!(
        plugin.module_uuid, module.module_uuid,
        "{name}: module UUID"
    );
    assert_eq!(
        plugin.module_version.to_string(),
        "3.2.1",
        "{name}: module version"
    );
    if let Some(resolved) = root.list("resolved") {
        let actual: Vec<String> = plugin
            .dependencies
            .iter()
            .map(|d| format!("{} {}", d.module_name, d.resolved))
            .collect();
        assert_eq!(actual, resolved, "{name}: resolved");
        let catalog = bds_catalog();
        for dependency in &plugin.dependencies {
            assert_eq!(
                Some(dependency.module_uuid),
                catalog.module(&dependency.module_name).map(|m| m.uuid),
                "{name}: the UUID of {}",
                dependency.module_name
            );
        }
    }
    if let Some(requested) = root.list("requested") {
        let actual: Vec<String> = plugin
            .dependencies
            .iter()
            .map(|d| format!("{} {}", d.module_name, d.requested))
            .collect();
        assert_eq!(actual, requested, "{name}: requested");
    }
    let optional: Vec<&str> = plugin
        .dependencies
        .iter()
        .filter(|d| d.optional)
        .map(|d| d.module_name.as_str())
        .collect();
    assert_eq!(
        optional,
        root.list("optional").unwrap_or_default(),
        "{name}: optional dependencies"
    );
    if let Some(main) = root.text("main") {
        assert_eq!(plugin.main.as_str(), main, "{name}: main");
    }
    if let Some(eval) = root.flag("script_eval") {
        assert_eq!(plugin.script_eval, eval, "{name}: script_eval");
    }
}
