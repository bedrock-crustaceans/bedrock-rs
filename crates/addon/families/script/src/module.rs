//! `ScriptModule`: the script side of a pack manifest, plugin discovery and the resolution of
//! native-module versions against a `ModuleCatalog`.
//!
//! Vanilla treats a pack's script in four stages, and only the first can refuse the pack:
//!
//! 1. the manifest is read ([`ScriptModule::parse`], [`ScriptModule::from_manifest`]);
//! 2. the plugin is discovered: one script module per pack, its language and entry, and each
//!    native-module dependency against the Beta APIs, internal, editor and permission gates;
//! 3. a context is created: every requested version is resolved against the engine's catalog;
//! 4. the script runs (not this crate's concern).
//!
//! Stages 2 and 3 are [`ScriptDiscovery::discover`]. A finding there disables the script and
//! leaves the pack's data in the stack.

mod catalog;
mod discover;
mod resolve;
#[cfg(test)]
mod tests;
mod version;

pub use catalog::{CatalogModule, CatalogVersion, ModuleCatalog};
pub use discover::{
    Discovery, ResolvedDependency, ScriptContext, ScriptDiscovery, ScriptFiles, ScriptModuleRef,
    ScriptPlugin, ScriptRuntime,
};
pub use version::ScriptVersion;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode as Code, Diagnostics, JsonPath, Location, Phase, Severity,
    VanillaText,
};
use bedrock_addon_core::document::{Document, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::version::{FormatVersion, SemVersion};
use uuid::Uuid;

use bedrock_addon_core::manifest::{
    Capability, CapabilityRegistry, Dependency, DependencyTexts, Manifest, ManifestFormat,
    NativeModuleVersion, dependency_texts,
};

/// The engine version from which a format-3 manifest may write a native-module dependency's
/// `version` in the relaxed form (`"2.0"`).
// Before 1.26.30 the engine reads the `version` as a strict semantic version only.
pub(crate) const RELAXED_VERSION_SINCE: FormatVersion = FormatVersion::new(1, 26, 30);

/// Which script host a script module is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ScriptTarget {
    /// A module of type `script`: run by the server, in a behaviour pack.
    Server,
    /// A module of type `client_script`: run by the client, in a resource pack.
    Client,
}

impl ScriptTarget {
    /// The manifest module `type`: `script` or `client_script`.
    pub fn module_type(self) -> &'static str {
        match self {
            Self::Server => "script",
            Self::Client => "client_script",
        }
    }

    /// The target of the module `type` spelled `name`, compared byte for byte.
    pub fn from_module_type(name: &str) -> Option<Self> {
        match name {
            "script" => Some(Self::Server),
            "client_script" => Some(Self::Client),
            _ => None,
        }
    }
}

/// One native-module dependency of a manifest: `{ "module_name": …, "version": … }`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct NativeDependency {
    /// JSON `module_name`, as written; module names are compared byte for byte.
    pub module_name: String,
    /// JSON `version`: a version, or the selector `alpha`, `beta` or `internal`.
    pub version: NativeModuleVersion,
    /// JSON `optional` (format 3; a legacy manifest's is not read): a gate then drops the
    /// dependency instead of disabling the script, and the script must import the module
    /// with `import()`.
    pub optional: bool,
    /// Where the dependency is: `dependencies/<index>`.
    pub path: JsonPath,
}

/// A pack dependency whose UUID is a native module's: the deprecated way to depend on
/// `@minecraft/server` or `@minecraft/server-ui`, which stands for version 0.1.0.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct UuidDependency {
    /// JSON `uuid`: the native module's UUID.
    pub uuid: Uuid,
    /// JSON `version`, as written; vanilla ignores it.
    pub version: SemVersion,
    /// Where the dependency is: `dependencies/<index>`.
    pub path: JsonPath,
}

/// A second or later script module of a manifest: vanilla skips it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SkippedModule {
    /// JSON `uuid`.
    pub uuid: Uuid,
    /// JSON `version`.
    pub version: SemVersion,
    /// Where the module is: `modules/<index>`.
    pub path: JsonPath,
}

/// The script side of one pack manifest: its `script` or `client_script` module, the
/// native modules the pack depends on, and the `script_eval` capability.
///
/// Read with [`ScriptModule::parse`] from the manifest's bytes, or with
/// [`ScriptModule::from_manifest`] from a manifest already read. Whether the script may run,
/// and with which module versions, is [`ScriptDiscovery::discover`].
#[doc(alias = "script_module")]
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ScriptModule {
    /// The pack's name as reports show it (the manifest header's `name`).
    pub pack_name: String,
    /// The manifest header's `uuid`: the pack's identity. No server configuration is ever
    /// looked up by it ([`module_uuid`](Self::module_uuid) is the key).
    pub pack_uuid: Uuid,
    /// The manifest file the module was read from.
    pub file: PackPath,
    /// JSON `type`: `script` or `client_script`.
    pub target: ScriptTarget,
    /// JSON `uuid`: the script's identity, and the key of its server configuration.
    pub module_uuid: Uuid,
    /// JSON `version`.
    pub module_version: SemVersion,
    /// JSON `language`, as written. Absent or empty is JavaScript.
    pub language: Option<String>,
    /// JSON `entry`, as written: the main file.
    pub entry: Option<String>,
    /// Where the module is: `modules/<index>`.
    pub path: JsonPath,
    /// The manifest's later script modules, which vanilla skips.
    pub skipped: Vec<SkippedModule>,
    /// The manifest's native-module dependencies, in manifest order; a module named twice
    /// is kept twice.
    pub dependencies: Vec<NativeDependency>,
    /// The manifest's pack dependencies on the UUID of a native module.
    pub uuid_dependencies: Vec<UuidDependency>,
    /// Whether the manifest's `capabilities` hold `script_eval`.
    pub script_eval: bool,
    /// The manifest header's `min_engine_version`.
    pub min_engine_version: Option<SemVersion>,
}

impl ScriptModule {
    /// The script side of `manifest`; `None` when it has no `script` or `client_script`
    /// module.
    ///
    /// Paths count the manifest's modules and dependencies as [`Manifest`] holds them.
    pub fn from_manifest(manifest: &Manifest) -> Option<ScriptModule> {
        Self::build(manifest, PackPath::from_static("manifest.json"), None)
    }

    /// Reads the script side of the manifest at `path` (vanilla's stage 1), with the
    /// capabilities the crate grants by default ([`CapabilityRegistry::default`]).
    ///
    /// The manifest is read as [`Manifest::parse`] reads it, and refused when that refuses
    /// it. On top of that, a format-3 dependency `version` in the relaxed form (`"2.0"`) is
    /// refused when `cx.engine_version` is below 1.26.30, and is a warning when the pack's
    /// own `min_engine_version` is; a legacy `optional` is reported as ignored. The value is
    /// `None` for a manifest without a script module. Paths are those of the file.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`] when the manifest is refused ([`Manifest::parse`]), or when the script
    /// side of it carries an error, such as a relaxed dependency `version` below 1.26.30.
    pub fn parse(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<Option<ScriptModule>>, Rejected> {
        Self::parse_with(path, bytes, cx, &CapabilityRegistry::default())
    }

    /// [`parse`](Self::parse) with the capabilities the host grants: `registry` is what
    /// [`Manifest::parse`] checks the manifest's `capabilities` against.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`] when the manifest is refused ([`Manifest::parse`]), or when the script
    /// side of it carries an error, such as a relaxed dependency `version` below 1.26.30.
    pub fn parse_with(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
        registry: &CapabilityRegistry,
    ) -> Result<Document<Option<ScriptModule>>, Rejected> {
        let manifest = Manifest::parse(path, bytes, registry)?;
        let Stage1 {
            module,
            added,
            refused,
        } = Self::stage1(&manifest, path, bytes, cx);
        let mut diagnostics = manifest.diagnostics().clone();
        diagnostics.extend(added);
        if refused {
            return Err(Rejected::new(manifest.origin().clone(), diagnostics));
        }
        Ok(Document::new(
            module,
            manifest.origin().clone(),
            manifest.version().clone(),
            diagnostics,
        ))
    }

    /// What a stack reads of the manifest at `path` of one of its packs: the script side,
    /// and the findings stage 1 adds to those the manifest reader told when the pack was
    /// opened. A manifest either of them refuses has no module.
    pub(crate) fn read(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
        registry: &CapabilityRegistry,
    ) -> Stage1 {
        match Manifest::parse(path, bytes, registry) {
            Ok(manifest) => Self::stage1(&manifest, path, bytes, cx),
            // The pack's opening told why.
            Err(_) => Stage1 {
                module: None,
                added: Diagnostics::default(),
                refused: true,
            },
        }
    }

    /// Stage 1 on a manifest already read from `bytes`: the script side, and the findings
    /// stage 1 adds to the manifest reader's own.
    pub(crate) fn stage1(
        manifest: &Manifest,
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Stage1 {
        // Format 0 reads no dependencies.
        let texts = match manifest.original_format {
            ManifestFormat::V0 => DependencyTexts::default(),
            format => dependency_texts(bytes, format != ManifestFormat::V3),
        };
        let legacy = manifest.original_format != ManifestFormat::V3;
        let mut added = Diagnostics::default();
        let mut refused = false;
        let mut finding = |code, severity, index: usize, key: &str| {
            let mut at = JsonPath::new();
            at.push("dependencies");
            at.push_index(index);
            at.push(key);
            added.push(Diagnostic::new(
                code,
                severity,
                Phase::Parse,
                Location::new(None, Some(path.clone()), at, None),
            ));
        };
        let pack_engine = manifest
            .header
            .min_engine_version
            .as_ref()
            .map(|min| min.version.format_version());
        // The native-module dependencies the reader kept, in file order: the names of
        // `texts.native`.
        let mut names = manifest
            .dependencies
            .iter()
            .filter_map(|dependency| match dependency {
                Dependency::NativeModule { module_name, .. } => Some(module_name.as_str()),
                Dependency::Pack { .. } => None,
            });
        for native in &texts.native {
            let name = names.next();
            if legacy && native.has_optional {
                // The legacy manifest parser has no `optional` key, so it ignores the member.
                finding(
                    Code::IgnoredMember { text: None },
                    Severity::Info,
                    native.index,
                    "optional",
                );
            }
            if !legacy && native.relaxed {
                if cx.engine_version < RELAXED_VERSION_SINCE {
                    // The older reader's `SemVersion` cannot read the string, and the
                    // validator prints `dependency {} invalid version: {}` with the invalid
                    // version it is left with (the same line it prints for `"abc"`).
                    refused = true;
                    finding(
                        Code::MalformedPropertyVersion {
                            text: name.map(|module| VanillaText::DependencyInvalidVersion {
                                module: module.to_owned(),
                                version: "0.0.0".to_owned(),
                            }),
                        },
                        Severity::Error,
                        native.index,
                        "version",
                    );
                } else if pack_engine.is_some_and(|engine| engine < RELAXED_VERSION_SINCE) {
                    // The running engine reads the form and the game says nothing; the pack
                    // claims an older one: the game writes no line for this, so it is the
                    // crate's own lint, an information.
                    finding(
                        Code::FieldSinceEngine {
                            since: RELAXED_VERSION_SINCE,
                        },
                        Severity::Info,
                        native.index,
                        "version",
                    );
                }
            }
        }
        let module = if refused {
            None
        } else {
            Self::build(manifest, path.clone(), Some(&texts))
        };
        Stage1 {
            module,
            added,
            refused,
        }
    }

    /// The script side of `manifest`, read from `file`; `texts` gives the dependencies' places
    /// in the file.
    fn build(
        manifest: &Manifest,
        file: PackPath,
        texts: Option<&DependencyTexts>,
    ) -> Option<ScriptModule> {
        let module_path = |index: usize| {
            let mut at = JsonPath::new();
            if manifest.original_format == ManifestFormat::V0 {
                // Format 0 has its modules inside the header.
                at.push("header");
            }
            at.push("modules");
            at.push_index(index);
            at
        };
        let dependency_path = |index: usize| {
            let mut at = JsonPath::new();
            at.push("dependencies");
            at.push_index(index);
            at
        };
        let mut scripts = manifest
            .modules
            .iter()
            .enumerate()
            .filter_map(|(i, module)| {
                ScriptTarget::from_module_type(module.module_type.name())
                    .map(|target| (i, module, target))
            });
        let (index, first, target) = scripts.next()?;
        let skipped = scripts
            .map(|(i, module, _)| SkippedModule {
                uuid: module.uuid,
                version: module.version.clone(),
                path: module_path(i),
            })
            .collect();
        let mut dependencies = Vec::new();
        let mut uuid_dependencies = Vec::new();
        let (mut natives, mut packs) = (0, 0);
        for (i, dependency) in manifest.dependencies.iter().enumerate() {
            match dependency {
                Dependency::NativeModule {
                    module_name,
                    version,
                    optional,
                } => {
                    let place = texts
                        .and_then(|texts| texts.native.get(natives))
                        .map_or(i, |native| native.index);
                    natives += 1;
                    dependencies.push(NativeDependency {
                        module_name: module_name.clone(),
                        version: version.clone(),
                        optional: optional.unwrap_or(false),
                        path: dependency_path(place),
                    });
                }
                Dependency::Pack { uuid, version } => {
                    let place = texts
                        .and_then(|texts| texts.packs.get(packs))
                        .map_or(i, |place| *place);
                    packs += 1;
                    if native_module_of(*uuid).is_some() {
                        uuid_dependencies.push(UuidDependency {
                            uuid: *uuid,
                            version: version.clone(),
                            path: dependency_path(place),
                        });
                    }
                }
            }
        }
        Some(ScriptModule {
            pack_name: manifest.header.display_name().to_owned(),
            pack_uuid: manifest.header.uuid,
            file,
            target,
            module_uuid: first.uuid,
            module_version: first.version.clone(),
            language: first.language.clone(),
            entry: first.entry.clone(),
            path: module_path(index),
            skipped,
            dependencies,
            uuid_dependencies,
            script_eval: manifest.capabilities.contains(&Capability::ScriptEval),
            min_engine_version: manifest
                .header
                .min_engine_version
                .as_ref()
                .map(|min| min.version.clone()),
        })
    }

    /// The name vanilla prints for the plugin: `<pack name> - <module version>`.
    pub fn plugin_name(&self) -> String {
        format!(
            "{} - {}",
            self.pack_name,
            ScriptVersion::from(&self.module_version)
        )
    }
}

/// Whether `manifest` has a script module the script host builds a plugin from and reads
/// script files for: its first `script` or `client_script` module, when its language is
/// JavaScript and its entry a `.js` file. Vanilla drops any other before it looks at a file
/// (an unsupported language or entry), and a pack without one has no plugin at all, so a stack reads the script files
/// of no other pack.
pub fn starts_a_plugin(manifest: &Manifest) -> bool {
    manifest
        .modules
        .iter()
        .find(|module| ScriptTarget::from_module_type(module.module_type.name()).is_some())
        .is_some_and(|module| {
            ScriptRuntime::from_language(module.language.as_deref()).is_some_and(|runtime| {
                module
                    .entry
                    .as_deref()
                    .is_some_and(|entry| discover::extension_with_dot(entry) == runtime.extension())
            })
        })
}

/// What stage 1 makes of a manifest the manifest reader accepted.
pub(crate) struct Stage1 {
    /// The script side; `None` for a manifest without a script module, and for one stage 1
    /// refuses.
    pub(crate) module: Option<ScriptModule>,
    /// The findings stage 1 adds to the manifest reader's.
    pub(crate) added: Diagnostics,
    /// Stage 1 refuses the manifest.
    pub(crate) refused: bool,
}

/// The native module a pack dependency's UUID stands for: vanilla knows two.
pub fn native_module_of(uuid: Uuid) -> Option<&'static str> {
    match uuid.as_u128() {
        0xb26a_4d4c_afdf_4690_88f8_9318_4631_2678 => Some("@minecraft/server"),
        0x2bd5_0a27_ab5f_4f40_a596_3641_627c_635e => Some("@minecraft/server-ui"),
        _ => None,
    }
}
