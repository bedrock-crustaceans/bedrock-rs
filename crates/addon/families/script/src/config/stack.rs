//! The configuration against the script modules of a stack: the dependency filter of plugin
//! discovery, the folders no module
//! uses, and the keys an older engine does not know.

use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, VanillaText,
};
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::version::FormatVersion;

use super::net::MAX_WEBSOCKET_PAYLOAD_SIZE;
use super::read::is_lookup_key;
use super::{
    CONFIG_DIR, DEFAULT_DIR, PAYLOAD_SIZE_SINCE, PERMISSIONS_FILE, PackScriptConfig,
    SERVER_NET_MODULE, ScriptConfig, ServerNet, WILDCARD_MODULE,
};
use crate::{ScriptContext, ScriptModuleRef, ScriptPlugin};

/// What the dependency filter decided for one script module.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct PluginVerdict {
    /// Whether the script module runs: no required dependency was refused.
    pub runs: bool,
    /// The optional dependencies that were refused and dropped, in order.
    pub dropped: Vec<String>,
    /// One error per refused required dependency, one info per dropped optional one.
    pub diagnostics: Diagnostics,
}

impl ScriptConfig {
    /// The gates `cx` with the permission list this configuration gives the script module
    /// `module_uuid`: the `allowed_modules` of the folder named after the module, else of
    /// `default` ([`for_module`](Self::for_module)).
    #[must_use]
    pub fn context<'a>(
        &'a self,
        mut cx: ScriptContext<'a>,
        module_uuid: Uuid,
    ) -> ScriptContext<'a> {
        cx.allowed_modules = Some(&self.for_module(module_uuid).permissions.allowed_modules);
        cx
    }

    /// Runs the dependency filter of plugin discovery for `plugin`, with the configuration of
    /// its script module ([`for_module`](Self::for_module)).
    ///
    /// Each native module the plugin depends on must be in the folder's `allowed_modules`,
    /// exactly or through `*`. A required one that is not refuses the plugin, with the
    /// server's error; an optional one is dropped and the plugin runs without it.
    ///
    /// `plugin` is what discovery made of a pack's script module
    /// ([`ScriptBundle::plugin`](crate::ScriptBundle::plugin) on a stack, which is
    /// discovered without a permission list). A discovery that was itself given the list
    /// ([`context`](Self::context)) has
    /// already applied it: its plugins pass here, and the refusals are its findings.
    ///
    /// One call reads the folder's whole list; for every script module of a stack use
    /// [`filter_stack`](Self::filter_stack), which reads each folder's list once.
    pub fn filter(&self, plugin: &ScriptPlugin) -> PluginVerdict {
        let config = self.for_module(plugin.module_uuid);
        verdict(config, &allowed_set(config), plugin)
    }

    /// [`filter`](Self::filter) for each of `plugins`, in order. Linear in the plugins'
    /// dependencies and the lists of the folders they use, however many plugins share
    /// `default`.
    pub fn filter_stack<'p>(
        &self,
        plugins: impl IntoIterator<Item = &'p ScriptPlugin>,
    ) -> Vec<PluginVerdict> {
        // The list of each folder in use, by the folder's place in this value.
        let mut lists: HashMap<*const PackScriptConfig, HashSet<&str>> = HashMap::new();
        plugins
            .into_iter()
            .map(|plugin| {
                let config = self.for_module(plugin.module_uuid);
                let allowed = lists
                    .entry(std::ptr::from_ref(config))
                    .or_insert_with(|| allowed_set(config));
                verdict(config, allowed, plugin)
            })
            .collect()
    }
}

/// The `allowed_modules` of `config`, for look-up.
fn allowed_set(config: &PackScriptConfig) -> HashSet<&str> {
    config
        .permissions
        .allowed_modules
        .iter()
        .map(String::as_str)
        .collect()
}

/// The dependency filter for `plugin` under `config`, whose list is `allowed`.
fn verdict(
    config: &PackScriptConfig,
    allowed: &HashSet<&str>,
    plugin: &ScriptPlugin,
) -> PluginVerdict {
    {
        let all = allowed.contains(WILDCARD_MODULE);
        let mut verdict = PluginVerdict {
            runs: true,
            dropped: Vec::new(),
            diagnostics: Diagnostics::default(),
        };
        let location = || Location::new(None, permissions_path(config), JsonPath::new(), None);
        for dependency in &plugin.dependencies {
            let module = dependency.module_name.as_str();
            if all || allowed.contains(module) {
                continue;
            }
            if dependency.optional {
                // The server drops an optional dependency without a log line.
                verdict.dropped.push(module.to_owned());
                verdict.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::DroppedField { text: None },
                    Severity::Info,
                    Phase::Link,
                    location(),
                ));
            } else {
                // A required dependency the folder does not allow rejects the plugin.
                verdict.runs = false;
                let text = Some(VanillaText::PluginModuleNotConfigured {
                    plugin: plugin.name.clone(),
                    module: module.to_owned(),
                });
                verdict.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::ScriptModulePermission { text },
                    Severity::Error,
                    Phase::Link,
                    location(),
                ));
            }
        }
        verdict
    }
}

impl ScriptConfig {
    /// The folders no script module of `plugins` uses, one finding each, in folder order
    /// (20 S8; a finding of the link phase, since it needs the stack).
    ///
    /// `plugins` are the scripts of the world that start: on a stack,
    /// `stack.scripts().iter().filter_map(|bundle| bundle.plugin.as_ref())`. A module whose
    /// script does not start is no plugin, and a folder named after it or its pack reads
    /// here as matching nothing; [`unmatched_folders_of_modules`](Self::unmatched_folders_of_modules)
    /// takes every module of the stack.
    ///
    /// A folder named after a pack's header UUID, or after a script module's UUID in any
    /// spelling but the lower-case hyphenated one, is an information that names the folder the
    /// module does use; any other folder that matches no script module is an information
    /// without it. `default` is never reported. The server has no line for a folder it does
    /// not use (the one config line it logs, `Failed to load config for %s due to error(s): %s`,
    /// is about a failed load), so every finding is the crate's own, an information.
    pub fn unmatched_folders<'p>(
        &self,
        plugins: impl IntoIterator<Item = &'p ScriptPlugin>,
    ) -> Diagnostics {
        self.unmatched_folders_of_modules(plugins.into_iter().map(ScriptModuleRef::from))
    }

    /// [`unmatched_folders`](Self::unmatched_folders) against script modules given by their
    /// identity, so that a module whose script does not start (its main file is missing, a
    /// version it asks for is not served) still answers for its folders: the server keys a
    /// pack's configuration by the module's UUID before it starts anything.
    ///
    /// On a stack, `stack.scripts().iter().filter_map(|bundle| bundle.module)` is every
    /// script module of the world; a [`ScriptModule`](crate::ScriptModule) read from
    /// a manifest gives its identity too.
    pub fn unmatched_folders_of_modules(
        &self,
        modules_of_stack: impl IntoIterator<Item = ScriptModuleRef>,
    ) -> Diagnostics {
        let mut modules = HashSet::new();
        // A header UUID → the script module of that pack.
        let mut headers = HashMap::new();
        for module in modules_of_stack {
            modules.insert(module.module_uuid);
            headers
                .entry(module.pack_uuid)
                .or_insert(module.module_uuid);
        }
        let mut findings = Diagnostics::default();
        for folder in self.packs.keys() {
            if folder == DEFAULT_DIR {
                continue;
            }
            let uuid = Uuid::try_parse(folder).ok();
            let (expected, severity) = match uuid {
                Some(uuid) if modules.contains(&uuid) => {
                    if is_lookup_key(folder) {
                        continue;
                    }
                    (Some(uuid), Severity::Info)
                }
                Some(uuid) => match headers.get(&uuid) {
                    Some(module) => (Some(*module), Severity::Info),
                    None => (None, Severity::Info),
                },
                None => (None, Severity::Info),
            };
            findings.push(Diagnostic::new(
                DiagnosticCode::UnmatchedConfigFolder {
                    folder: folder.clone(),
                    expected,
                },
                severity,
                Phase::Link,
                Location::new(
                    None,
                    PackPath::new(&format!("{CONFIG_DIR}/{folder}")).ok(),
                    JsonPath::new(),
                    None,
                ),
            ));
        }
        findings
    }

    /// The members an engine of version `target` does not know yet, one information each, in
    /// folder order with `default` first: the crate's prediction of an older engine, which no
    /// game version logs, so each is an information.
    ///
    /// A server-net entry with `max_websocket_payload_size` is refused as a whole by an
    /// engine before [`PAYLOAD_SIZE_SINCE`], which then leaves the pack's requests
    /// unrestricted.
    pub fn engine_findings(&self, target: FormatVersion) -> Diagnostics {
        let mut findings = Diagnostics::default();
        if target >= PAYLOAD_SIZE_SINCE {
            return findings;
        }
        for config in std::iter::once(&self.default).chain(self.packs.values()) {
            let ServerNet::Configured(net) = &config.permissions.server_net else {
                continue;
            };
            if net.max_websocket_payload_size.is_none() {
                continue;
            }
            let at = JsonPath::new()
                .child("module_permissions")
                .child(SERVER_NET_MODULE)
                .child(MAX_WEBSOCKET_PAYLOAD_SIZE);
            findings.push(Diagnostic::new(
                DiagnosticCode::FieldSinceEngine {
                    since: PAYLOAD_SIZE_SINCE,
                },
                Severity::Info,
                Phase::Validate,
                Location::new(None, permissions_path(config), at, None),
            ));
        }
        findings
    }
}

/// `config/<folder>/permissions.json` of `config`.
fn permissions_path(config: &PackScriptConfig) -> Option<PackPath> {
    let folder = match &config.source {
        super::ConfigSource::Default => DEFAULT_DIR,
        super::ConfigSource::Folder(folder) => folder,
    };
    PackPath::new(&format!("{CONFIG_DIR}/{folder}/{PERMISSIONS_FILE}")).ok()
}
