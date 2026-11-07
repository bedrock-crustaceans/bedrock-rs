//! Plugin discovery (vanilla's stage 2) and the hand-over to version resolution (stage 3).

use crate::codes;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Write as _;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode as Code, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::version::SemVersion;
use indexmap::IndexSet;
use uuid::Uuid;

use super::resolve::{Entry, Request, VersionResolution, newest_serving, requested, resolve};
use super::{ModuleCatalog, ScriptModule, ScriptTarget, ScriptVersion, native_module_of};
use crate::{NativeModuleAccess, ScriptHost, ScriptPath};

/// What decides the gates of plugin discovery: the world's toggles, the build, the pack's
/// origin and the server's permission list.
// Four independent inputs of vanilla's filters, each a yes or no.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScriptContext<'a> {
    /// The Beta APIs experiment: lifts the gate on `beta` versions, on major 0 and on
    /// `@minecraft/server-gametest`.
    pub beta_apis: bool,
    /// The world is an editor world: the editor modules may be depended on.
    pub editor_world: bool,
    /// The build lets every pack use `internal` versions (vanilla's development builds do;
    /// the published server does not).
    pub allow_internal: bool,
    /// The pack is bundled with the game: the Beta APIs, internal and gametest gates do not
    /// apply to it.
    pub is_package_origin: bool,
    /// The modules the server's configuration lets this script use (`allowed_modules` of
    /// its `permissions.json`): exact names, or `*` for every module; `None` where no such
    /// list applies (a world no dedicated server hosts).
    pub allowed_modules: Option<&'a [String]>,
}

impl ScriptContext<'static> {
    /// The gates of a pack loaded with `cx`: its Beta APIs toggle and whether it is a
    /// base-game pack. Not an editor world, no internal versions, no permission list.
    pub fn for_pack(cx: &LoadContext) -> Self {
        Self {
            beta_apis: cx.beta_apis,
            is_package_origin: cx.is_base_game_pack,
            ..Self::default()
        }
    }
}

/// The script files of one pack: every file below `scripts/` whose extension is exactly
/// `.js`, by its path below `scripts/` (`lib/b.js`). Names are compared byte for byte.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScriptFiles {
    /// Sorted bytewise, each once.
    members: Vec<Box<str>>,
}

impl<'a> FromIterator<&'a str> for ScriptFiles {
    /// The script files named by their paths below `scripts/`, taken as given.
    fn from_iter<I: IntoIterator<Item = &'a str>>(members: I) -> Self {
        let mut members: Vec<Box<str>> = members.into_iter().map(Box::from).collect();
        members.sort_unstable();
        members.dedup();
        Self { members }
    }
}

impl ScriptFiles {
    /// The script files among a pack's file paths (`scripts/main.js`, `manifest.json`, …);
    /// every other path is ignored.
    pub fn from_pack_paths<'a>(paths: impl IntoIterator<Item = &'a str>) -> Self {
        Self::from_members(
            paths
                .into_iter()
                .filter_map(|path| path.strip_prefix("scripts/"))
                .filter(|member| extension_with_dot(member) == ".js"),
        )
    }

    /// The script files named by their paths below `scripts/`, taken as given.
    pub fn from_members<'a>(members: impl IntoIterator<Item = &'a str>) -> Self {
        members.into_iter().collect()
    }

    /// Whether `member` (a path below `scripts/`) is a script file.
    pub fn contains(&self, member: &str) -> bool {
        self.members
            .binary_search_by(|have| (**have).cmp(member))
            .is_ok()
    }

    /// How many script files.
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// Whether there is no script file.
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}

/// The extension of a path's last component with its dot (`.js`), or `""` without one.
pub(super) fn extension_with_dot(path: &str) -> &str {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    name.rfind('.').map_or("", |dot| &name[dot..])
}

/// The runtime a script module's `language` names. Vanilla has one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ScriptRuntime {
    /// JavaScript: `language` absent, empty, or `javascript` in any ASCII case.
    JavaScript,
}

impl ScriptRuntime {
    /// The runtime of a module's `language`; `None` for a language vanilla has no runtime
    /// for (`typescript`, `js`, `javascript ` with a space).
    pub fn from_language(language: Option<&str>) -> Option<Self> {
        // The game compares the name with `javascript` after ASCII lower-casing, and an
        // empty name gets the default runtime.
        match language {
            None | Some("") => Some(Self::JavaScript),
            Some(name) if name.eq_ignore_ascii_case("javascript") => Some(Self::JavaScript),
            Some(_) => None,
        }
    }

    /// The runtime's name as vanilla prints it: `JavaScript`.
    pub fn name(self) -> &'static str {
        match self {
            Self::JavaScript => "JavaScript",
        }
    }

    /// The extension of the runtime's files, with its dot: `.js`.
    pub fn extension(self) -> &'static str {
        match self {
            Self::JavaScript => ".js",
        }
    }
}

/// One native module a discovered plugin uses, with the version it asked for and the
/// version it gets.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ResolvedDependency {
    /// The module's name.
    pub module_name: String,
    /// The module's UUID; `None` when the catalog does not know it.
    pub module_uuid: Option<Uuid>,
    /// The version asked for; a selector is the catalog's newest version of that channel.
    pub requested: ScriptVersion,
    /// The version the engine serves.
    pub resolved: ScriptVersion,
    /// The dependency is optional: the script must import the module with `import()`.
    pub optional: bool,
}

/// A script that may run: what the script host needs to start it.
///
/// It is what discovery made of a pack's [`ScriptModule`], and it answers for that module
/// wherever one is asked about: it is the [`ScriptHost`] a pack's [`ScriptBundle`] is scanned
/// against, and the plugin a server's configuration is filtered for
/// (`ScriptConfig::filter` of the `server_config` family).
///
/// [`ScriptBundle`]: crate::ScriptBundle
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ScriptPlugin {
    /// The pack's name.
    pub pack_name: String,
    /// The pack header's UUID. No server configuration is ever looked up by it.
    pub pack_uuid: Uuid,
    /// The script module's UUID: the name of the pack's server configuration folder.
    pub module_uuid: Uuid,
    /// The script module's version.
    pub module_version: ScriptVersion,
    /// The name vanilla prints the plugin by: `<pack name> - <module version>`.
    pub name: String,
    /// The runtime.
    pub runtime: ScriptRuntime,
    /// The module's `entry`, as the manifest writes it (`scripts/main.js`).
    pub entry: String,
    /// The main file, by its path below `scripts/`.
    pub main: ScriptPath,
    /// The native modules the script may import, in the order the manifest names them.
    pub dependencies: Vec<ResolvedDependency>,
    /// The pack has the `script_eval` capability: `eval` and `Function` are defined.
    pub script_eval: bool,
    /// The pack is bundled with the game ([`ScriptContext::is_package_origin`]): its files
    /// import whatever bindings the engine gives them.
    pub is_package_origin: bool,
    /// The pack's `min_engine_version`.
    pub min_engine_version: Option<SemVersion>,
}

impl ScriptPlugin {
    /// A plugin written by hand: the pack `pack_name` with the header `pack_uuid`, whose
    /// script module `module_uuid` has `module_version`. It runs JavaScript from
    /// `scripts/main.js`, depends on no native module, has no `script_eval` and is a pack of
    /// a world; its name is `<pack name> - <module version>`. Every field is public: set
    /// what differs (the [`dependencies`](Self::dependencies) a configuration is to be
    /// filtered for by `server_config`'s `ScriptConfig::filter`, say).
    ///
    /// A stack's plugins come from discovery
    /// ([`ScriptBundle::plugin`](crate::ScriptBundle::plugin)); this is for a caller
    /// that has no manifest to discover from.
    pub fn new(
        pack_name: impl Into<String>,
        pack_uuid: Uuid,
        module_uuid: Uuid,
        module_version: ScriptVersion,
    ) -> Self {
        let pack_name = pack_name.into();
        Self {
            name: format!("{pack_name} - {module_version}"),
            pack_name,
            pack_uuid,
            module_uuid,
            module_version,
            runtime: ScriptRuntime::JavaScript,
            entry: "scripts/main.js".to_owned(),
            main: ScriptPath::new("main.js"),
            dependencies: Vec::new(),
            script_eval: false,
            is_package_origin: false,
            min_engine_version: None,
        }
    }
}

impl ResolvedDependency {
    /// A required dependency on the native module `module_name`, asked for and served at
    /// `version`, with no UUID of a catalog. The fields are public: set what differs.
    pub fn new(module_name: impl Into<String>, version: ScriptVersion) -> Self {
        Self {
            module_name: module_name.into(),
            module_uuid: None,
            requested: version.clone(),
            resolved: version,
            optional: false,
        }
    }
}

/// The identity of a script module, as a server configuration is held against it: the
/// header UUID of its pack and its own UUID. A module has one whether or not its script
/// starts; a [`ScriptPlugin`] and a [`ScriptModule`] both give theirs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScriptModuleRef {
    /// The pack header's UUID. No server configuration is ever looked up by it.
    pub pack_uuid: Uuid,
    /// The script module's UUID: the name of the pack's server configuration folder.
    pub module_uuid: Uuid,
}

impl From<&ScriptPlugin> for ScriptModuleRef {
    fn from(plugin: &ScriptPlugin) -> Self {
        Self {
            pack_uuid: plugin.pack_uuid,
            module_uuid: plugin.module_uuid,
        }
    }
}

impl From<&ScriptModule> for ScriptModuleRef {
    fn from(module: &ScriptModule) -> Self {
        Self {
            pack_uuid: module.pack_uuid,
            module_uuid: module.module_uuid,
        }
    }
}

/// The native modules no pack file may import.
// The game refuses an import of these names from a pack's files; the crate lists exactly
// the names it refuses.
const RESTRICTED: [&str; 2] = [
    "@minecraft/server-bindings",
    "@minecraft/server-ui-private-bindings",
];

/// The native modules a module of the context brings with it: `(module, brought)`.
// `@minecraft/common` is a dependency of the server wrapper;
// `@minecraft/server-ui-bindings` is importable with `server-ui` declared.
const BROUGHT: [(&str, &str); 2] = [
    ("@minecraft/server", "@minecraft/common"),
    ("@minecraft/server-ui", "@minecraft/server-ui-bindings"),
];

/// The plugin answers for its script module: the modules of its context are the
/// dependencies discovery kept ([`dependencies`](ScriptPlugin::dependencies)), so one a gate
/// dropped is unknown to the script.
impl ScriptHost for ScriptPlugin {
    fn plugin(&self) -> &str {
        &self.name
    }

    fn entry(&self) -> Option<&str> {
        Some(&self.entry)
    }

    fn native(&self, name: &str) -> NativeModuleAccess {
        let has = |module: &str| self.dependencies.iter().any(|d| d.module_name == module);
        if has(name) {
            return NativeModuleAccess::Available;
        }
        // The engine's own packs reach the bindings a pack of a world may not (the editor
        // pack's `Main.js` imports `@minecraft/server-editor-private-bindings`).
        if self.is_package_origin && name.starts_with('@') {
            return NativeModuleAccess::Available;
        }
        if RESTRICTED.contains(&name) {
            return NativeModuleAccess::Restricted;
        }
        let brought = BROUGHT
            .iter()
            .any(|(module, brings)| *brings == name && has(module));
        if brought {
            NativeModuleAccess::Available
        } else {
            NativeModuleAccess::Unknown
        }
    }

    fn script_eval(&self) -> bool {
        self.script_eval
    }
}

/// What discovery made of one pack's script.
#[derive(Clone, Debug, PartialEq)]
pub struct Discovery {
    plugin: Option<ScriptPlugin>,
    diagnostics: Diagnostics,
}

impl Discovery {
    /// The script to run; `None` when a finding disabled it. The pack's data loads either
    /// way.
    pub fn plugin(&self) -> Option<&ScriptPlugin> {
        self.plugin.as_ref()
    }

    /// The findings, in the order vanilla logs them.
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    pub(super) fn new(plugin: Option<ScriptPlugin>, diagnostics: Diagnostics) -> Self {
        Self {
            plugin,
            diagnostics,
        }
    }
}

/// Plugin discovery over the packs of one world, against one catalog.
///
/// Call [`discover`](Self::discover) for each pack with a script, top of the stack first: a
/// script module whose UUID and version an earlier pack already brought for the same
/// [`ScriptTarget`] is skipped. A `script` and a `client_script` module are discovered
/// apart, as the server's and the client's script hosts do.
#[derive(Clone, Debug)]
pub struct ScriptDiscovery<'a> {
    catalog: &'a ModuleCatalog,
    /// The target, UUID and version of every module a plugin was built for, in that order.
    seen: IndexSet<(ScriptTarget, Uuid, ScriptVersion)>,
}

impl<'a> ScriptDiscovery<'a> {
    /// A discovery against `catalog`, with no plugin yet.
    pub fn new(catalog: &'a ModuleCatalog) -> Self {
        Self {
            catalog,
            seen: IndexSet::new(),
        }
    }

    /// Discovers the plugin of `module` (vanilla's stages 2 and 3): its language, entry and
    /// main file among `files`, each dependency against the gates of `cx`, and the versions
    /// the catalog serves for what remains.
    pub fn discover(
        &mut self,
        module: &ScriptModule,
        files: &ScriptFiles,
        cx: &ScriptContext<'_>,
    ) -> Discovery {
        let mut run = Run {
            catalog: self.catalog,
            module,
            cx,
            plugin_name: module.plugin_name(),
            diagnostics: Diagnostics::default(),
            supported: HashMap::new(),
        };
        // Discovery runs for one resource type at a time, so the key carries the target.
        let key = (
            module.target,
            module.module_uuid,
            ScriptVersion::from(&module.module_version),
        );
        let checked = if self.seen.contains(&key) {
            // The module's UUID and version were already brought by an earlier pack. The game
            // prints this as a `WARN [Scripting]` line, so the finding is a warning.
            run.finding(
                Code::from(codes::PLUGIN_MODULE_EXISTS.with([
                    run.plugin_name.clone(),
                    module.module_uuid.to_string(),
                    key.2.to_string(),
                ])),
                Severity::Warning,
                &module.path,
            );
            None
        } else {
            run.check(files)
        };
        // The game builds a plugin from the first script module only and meets the later
        // ones afterwards; it prints a `WARN [Scripting]` line for each, so the finding is a
        // warning.
        for skipped in &module.skipped {
            run.finding(
                Code::from(core_codes::MULTIPLE_SCRIPT_MODULES.with([format!(
                    "{} - {}",
                    module.pack_name,
                    ScriptVersion::from(&skipped.version)
                )])),
                Severity::Warning,
                &skipped.path,
            );
        }
        let plugin = checked.and_then(|checked| {
            // Vanilla has built the plugin by now: its UUID and version are taken, also
            // when its main file is missing or its context fails.
            self.seen.insert(key);
            checked
                .main_found
                .then(|| run.create_context(checked))
                .flatten()
        });
        Discovery::new(plugin, run.diagnostics)
    }
}

/// The module vanilla's gametest filter knows by its UUID: gated by Beta APIs at every
/// version.
const GAMETEST_MODULE: &str = "@minecraft/server-gametest";

/// The element of a permission list that allows every module.
const WILDCARD: &str = "*";

/// The prefix of the editor modules' names.
const EDITOR_MODULES: &str = "@minecraft/server-editor";

/// A dependency that passed discovery: the module, what it asks for, where it is.
struct Kept<'a> {
    module_name: &'a str,
    requested: ScriptVersion,
    optional: bool,
    path: &'a JsonPath,
}

/// A script module vanilla builds a plugin for: its language, entry and dependencies passed.
struct Checked<'a> {
    runtime: ScriptRuntime,
    /// The entry as written.
    entry: &'a str,
    main: String,
    /// The main file is one of the pack's script files; without it the plugin exists
    /// and does not run.
    main_found: bool,
    kept: Vec<Kept<'a>>,
}

/// One pack's discovery.
struct Run<'a> {
    catalog: &'a ModuleCatalog,
    module: &'a ScriptModule,
    cx: &'a ScriptContext<'a>,
    /// `<pack name> - <module version>`.
    plugin_name: String,
    diagnostics: Diagnostics,
    /// The versions the catalog serves per module, read once per module.
    supported: HashMap<&'a str, Vec<ScriptVersion>>,
}

impl<'a> Run<'a> {
    fn finding(&mut self, code: Code, severity: Severity, at: &JsonPath) {
        self.diagnostics.push(Diagnostic::new(
            code,
            severity,
            Phase::Validate,
            Location::new(None, Some(self.module.file.clone()), at.clone(), None),
        ));
    }

    /// The versions the catalog serves of `module`; none for a module it does not know.
    fn supported(&mut self, module: &'a str) -> &[ScriptVersion] {
        let catalog = self.catalog;
        self.supported.entry(module).or_insert_with(|| {
            catalog.module(module).map_or_else(Vec::new, |module| {
                module.versions.iter().map(|v| v.version.clone()).collect()
            })
        })
    }

    /// Stage 2: the module's language and entry, each dependency against the gates, the
    /// by-UUID dependencies and the main file. `None` when a finding ends the module before
    /// vanilla builds its plugin; a missing main file is reported and leaves the plugin.
    fn check(&mut self, files: &ScriptFiles) -> Option<Checked<'a>> {
        let module = self.module;
        let plugin = self.plugin_name.clone();
        // The game prints the runtime name after sanitising it, and an unsupported language
        // sanitises to an empty name.
        let Some(runtime) = ScriptRuntime::from_language(module.language.as_deref()) else {
            self.finding(
                Code::from(codes::PLUGIN_UNSUPPORTED_RUNTIME.with([plugin.clone(), String::new()])),
                Severity::Error,
                &module.path.child("language"),
            );
            return None;
        };
        // The entry must end in the extension the runtime's files have.
        let entry = module.entry.as_deref();
        if entry.is_none_or(|entry| extension_with_dot(entry) != runtime.extension()) {
            let args = [
                plugin,
                entry.unwrap_or_default().to_owned(),
                runtime.name().to_owned(),
            ];
            match entry {
                None => self.finding(
                    Code::from(codes::PLUGIN_ENTRY_NOT_SUPPORTED_MISSING_REQUIRED_FIELD.with(args)),
                    Severity::Error,
                    &module.path,
                ),
                Some(_) => self.finding(
                    Code::from(codes::PLUGIN_ENTRY_NOT_SUPPORTED_INVALID_VALUE.with(args)),
                    Severity::Error,
                    &module.path.child("entry"),
                ),
            }
            return None;
        }
        let written = entry.unwrap_or_default();

        let mut kept = self.filter_dependencies()?;

        // A dependency by UUID stands for version 0.1.0. It is added after the dependency
        // loop, so no gate sees that version. The game prints a `WARN [Scripting]` line for
        // it, so the finding is a warning.
        for dependency in &module.uuid_dependencies {
            let Some(name) = native_module_of(dependency.uuid) else {
                continue;
            };
            self.finding(
                Code::from(codes::PLUGIN_DEPENDENCY_BY_UUID.with([self.plugin_name.clone()])),
                Severity::Warning,
                &dependency.path,
            );
            kept.push(Kept {
                module_name: name,
                requested: ScriptVersion::new(0, 1, 0),
                optional: false,
                path: &dependency.path,
            });
        }

        // The entry without the `scripts/` root must be a script file; the game takes a
        // backslash in the entry for a path separator.
        let entry = written.replace('\\', "/");
        let main = entry.strip_prefix("scripts/").unwrap_or(&entry).to_owned();
        let main_found = files.contains(&main);
        if !main_found {
            self.finding(
                Code::from(
                    codes::PLUGIN_MISSING_MAIN_FILE.with([self.plugin_name.clone(), main.clone()]),
                ),
                Severity::Error,
                &module.path.child("entry"),
            );
        }
        Some(Checked {
            runtime,
            entry: written,
            main,
            main_found,
            kept,
        })
    }

    /// The dependencies that pass the gates. Every dependency is filtered, also after one
    /// rejected the plugin, because the game's loop goes on after a rejection; `None` when
    /// one did.
    fn filter_dependencies(&mut self) -> Option<Vec<Kept<'a>>> {
        let allowed: Option<HashSet<&str>> = self
            .cx
            .allowed_modules
            .map(|allowed| allowed.iter().map(String::as_str).collect());
        let mut kept = Vec::new();
        let mut rejected = false;
        for dependency in &self.module.dependencies {
            let name = dependency.module_name.as_str();
            let version = requested(&dependency.version, self.supported(name));
            match self.gate(name, &version, allowed.as_ref()) {
                Gate::Allow => {}
                Gate::Drop(code) => {
                    self.finding(code, Severity::Info, &dependency.path);
                    continue;
                }
                Gate::Reject(code) => {
                    if dependency.optional {
                        // The game drops a rejected optional dependency without a log
                        // line.
                        self.finding(
                            Code::from(&core_codes::DROPPED_FIELD),
                            Severity::Info,
                            &dependency.path,
                        );
                    } else {
                        self.finding(code, Severity::Error, &dependency.path);
                        rejected = true;
                    }
                    continue;
                }
            }
            if self.catalog.module(name).is_none() {
                // The game accepts the name here and fails later, when it creates the
                // context; the crate reports it with the dependency.
                self.finding(
                    Code::from(codes::PLUGIN_UNKNOWN_MODULE.with([
                        self.module.pack_name.clone(),
                        self.plugin_name.clone(),
                        name.to_string(),
                    ])),
                    Severity::Error,
                    &dependency.path,
                );
                rejected = true;
                continue;
            }
            if version.pre == "alpha" {
                // The game lets an `alpha` selector through every gate.
                self.finding(
                    Code::from(&core_codes::SCRIPT_ALPHA_DEPENDENCY),
                    Severity::Info,
                    &dependency.path,
                );
            }
            kept.push(Kept {
                module_name: name,
                requested: version,
                optional: dependency.optional,
                path: &dependency.path,
            });
        }
        (!rejected).then_some(kept)
    }

    /// The filter chain for one dependency asking for `version` of the module `name`: the
    /// editor filter, the gametest filter, the release filter (beta, then internal) and the
    /// permissions filter; the first that objects decides.
    fn gate(&self, name: &str, version: &ScriptVersion, allowed: Option<&HashSet<&str>>) -> Gate {
        let cx = self.cx;
        let plugin = self.plugin_name.clone();
        let dependency = format!("{name} - {version}");
        if name.starts_with(EDITOR_MODULES) && !cx.editor_world {
            // The editor modules are dropped from a world that is no editor world.
            return Gate::Drop(Code::from(
                codes::PLUGIN_NOT_EDITOR_WORLD.with([plugin.clone(), name.to_string()]),
            ));
        }
        // A pack bundled with the game skips the next three gates.
        let gated = !cx.is_package_origin;
        if gated && !cx.beta_apis && name == GAMETEST_MODULE {
            // `server-gametest` needs the Beta APIs toggle at every version.
            return Gate::Reject(Code::from(
                codes::PLUGIN_BETA_APIS_MODULE.with([plugin.clone(), dependency.clone()]),
            ));
        }
        if gated && !cx.beta_apis && version.is_beta() {
            // A beta version needs the Beta APIs toggle.
            return Gate::Reject(Code::from(
                codes::PLUGIN_BETA_APIS.with([plugin.clone(), dependency.clone()]),
            ));
        }
        if gated && !cx.allow_internal && version.is_internal() {
            // An internal version is for development builds.
            return Gate::Reject(Code::from(
                codes::PLUGIN_INTERNAL_APIS.with([plugin.clone(), dependency.clone()]),
            ));
        }
        // The permission list allows the name itself, or every module with its `*`.
        if allowed.is_some_and(|allowed| !allowed.contains(name) && !allowed.contains(WILDCARD)) {
            // The module is not in the server's `allowed_modules`.
            return Gate::Reject(Code::from(
                codes::PLUGIN_MODULE_NOT_CONFIGURED.with([plugin.clone(), name.to_string()]),
            ));
        }
        Gate::Allow
    }

    /// Stage 3: every request, the plugin's own and those of the modules it brings, resolved
    /// module by module. `None` when one module has no version for its requests.
    fn create_context(&mut self, checked: Checked<'a>) -> Option<ScriptPlugin> {
        let module = self.module;
        let catalog = self.catalog;
        let Requests {
            written,
            order,
            by_module: requests,
            mut failed,
        } = self.requests(&checked.kept);
        let mut served: HashMap<&str, ScriptVersion> = HashMap::new();
        for name in order {
            let resolution = resolve(&requests[name], self.supported(name));
            let at = |entry: &Entry| -> JsonPath {
                written
                    .get(&(name, &entry.version))
                    .filter(|_| entry.explicit)
                    .copied()
                    .unwrap_or(&module.path)
                    .clone()
            };
            match resolution {
                VersionResolution::Resolved {
                    version,
                    by,
                    promoted,
                } => {
                    for entry in promoted {
                        // A request is promoted to the version the context serves.
                        self.finding(
                            Code::from(core_codes::SCRIPT_MODULE_PROMOTED.with([
                                module.pack_name.clone(),
                                name.to_owned(),
                                entry.version.to_string(),
                                version.to_string(),
                                by.clone(),
                            ])),
                            Severity::Info,
                            &at(&entry),
                        );
                    }
                    served.insert(name, version);
                }
                VersionResolution::Failed { conflicts, missing } => {
                    failed = true;
                    for entry in missing {
                        // No version serves the request.
                        let code = self.unavailable(name, &entry);
                        self.finding(code, Severity::Error, &at(&entry));
                    }
                    for (earlier, later) in conflicts {
                        // Two requests need versions that cannot both be served.
                        let place = if later.explicit {
                            at(&later)
                        } else {
                            at(&earlier)
                        };
                        // One line per request, a comma between lines and none after the last.
                        let requests = format!(
                            "\n [{}] requested by [{}],\n [{}] requested by [{}]",
                            earlier.version, earlier.by, later.version, later.by
                        );
                        self.finding(
                            Code::from(core_codes::SCRIPT_MODULE_VERSION_CONFLICT.with([
                                module.pack_name.clone(),
                                name.to_owned(),
                                requests,
                            ])),
                            Severity::Error,
                            &place,
                        );
                    }
                }
            }
        }
        if failed {
            return None;
        }
        let dependencies = checked
            .kept
            .iter()
            .filter_map(|kept| {
                Some(ResolvedDependency {
                    module_name: kept.module_name.to_owned(),
                    module_uuid: catalog.module(kept.module_name)?.uuid,
                    requested: kept.requested.clone(),
                    resolved: served.get(kept.module_name)?.clone(),
                    optional: kept.optional,
                })
            })
            .collect();
        Some(ScriptPlugin {
            pack_name: module.pack_name.clone(),
            pack_uuid: module.pack_uuid,
            module_uuid: module.module_uuid,
            module_version: ScriptVersion::from(&module.module_version),
            name: self.plugin_name.clone(),
            runtime: checked.runtime,
            entry: checked.entry.to_owned(),
            main: ScriptPath::new(checked.main),
            dependencies,
            script_eval: module.script_eval,
            is_package_origin: self.cx.is_package_origin,
            min_engine_version: module.min_engine_version.clone(),
        })
    }

    /// Every request for a version of a module: the plugin's own, and what the versions
    /// serving them require in turn. Each (module, version) is expanded once, so a chain or a
    /// cycle of requirements is walked once.
    fn requests<'k>(&mut self, kept: &'k [Kept<'a>]) -> Requests<'a, 'k> {
        let module = self.module;
        let catalog = self.catalog;
        // Where the plugin's own request for a version of a module is written.
        let mut written: HashMap<(&str, &ScriptVersion), &JsonPath> = HashMap::new();
        let mut queue: VecDeque<(&str, ScriptVersion, String, bool)> = VecDeque::new();
        for kept in kept {
            written
                .entry((kept.module_name, &kept.requested))
                .or_insert(kept.path);
            queue.push_back((
                kept.module_name,
                kept.requested.clone(),
                self.plugin_name.clone(),
                true,
            ));
        }
        // The requests per module, in the order the modules are first asked for.
        let mut order: Vec<&str> = Vec::new();
        let mut requests: HashMap<&str, Vec<Request>> = HashMap::new();
        let mut expanded: HashSet<(&str, ScriptVersion)> = HashSet::new();
        let mut failed = false;
        while let Some((name, version, by, explicit)) = queue.pop_front() {
            let Some(entry) = catalog.module(name) else {
                // The catalog has no such module: the request fails with the module's name.
                self.finding(
                    Code::from(codes::PLUGIN_UNKNOWN_MODULE.with([
                        module.pack_name.clone(),
                        by.clone(),
                        name.to_string(),
                    ])),
                    Severity::Error,
                    written
                        .get(&(name, &version))
                        .copied()
                        .unwrap_or(&module.path),
                );
                failed = true;
                continue;
            };
            // What the version serving this request requires in turn, and what the requested
            // version itself requires when the catalog has it.
            let newest = newest_serving(&version, self.supported(name)).cloned();
            for served in [newest.as_ref(), Some(&version)].into_iter().flatten() {
                let Some(found) = entry.version(served) else {
                    continue;
                };
                if !expanded.insert((name, served.clone())) {
                    continue;
                }
                for (required, at) in &found.requires {
                    queue.push_back((
                        required.as_str(),
                        at.clone(),
                        format!("{} - {served}", entry.requester(name)),
                        false,
                    ));
                }
            }
            requests
                .entry(name)
                .or_insert_with(|| {
                    order.push(name);
                    Vec::new()
                })
                .push(Request {
                    version,
                    by,
                    explicit,
                });
        }
        Requests {
            written,
            order,
            by_module: requests,
            failed,
        }
    }

    /// The finding for a request of `name` that no version serves: the versions the catalog lists,
    /// and its version of the request's pre-release tag when it has one.
    fn unavailable(&self, name: &str, entry: &Entry) -> Code {
        let versions = self
            .catalog
            .module(name)
            .map_or(&[][..], |module| module.versions.as_slice());
        let current = versions
            .iter()
            .filter(|v| !entry.version.pre.is_empty() && v.version.pre == entry.version.pre)
            .map(|v| &v.version)
            .max()
            .map_or_else(String::new, |version| {
                format!("\nCurrent [{}] version is [{version}].", entry.version.pre)
            });
        let available = versions
            .iter()
            .filter(|v| !v.hidden)
            .fold(String::new(), |mut out, v| {
                let _ = write!(out, "\n{}", v.version);
                out
            });
        Code::from(core_codes::SCRIPT_MODULE_VERSION_UNAVAILABLE.with([
            self.module.pack_name.clone(),
            entry.by.clone(),
            entry.version.to_string(),
            name.to_owned(),
            current,
            available,
        ]))
    }
}

/// The requests of one plugin's context.
struct Requests<'a, 'k> {
    /// Where the plugin's own request for a version of a module is written.
    written: HashMap<(&'a str, &'k ScriptVersion), &'a JsonPath>,
    /// The modules in the order they are first asked for.
    order: Vec<&'a str>,
    /// The requests per module.
    by_module: HashMap<&'a str, Vec<Request>>,
    /// A request names a module the catalog does not have.
    failed: bool,
}

/// What the filter chain makes of one dependency.
enum Gate {
    /// The dependency stays.
    Allow,
    /// The dependency is dropped with this finding and the plugin goes on.
    Drop(Code),
    /// The plugin is rejected with this finding, or the dependency dropped when it is
    /// optional.
    Reject(Code),
}
