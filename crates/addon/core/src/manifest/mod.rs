//! The pack manifest: legacy (0/1/2) and comprehensive (3) formats parsed into one `Manifest`,
//! the capability registry, and the script-module parts surfaced as `script::ScriptModule`.

// Submodule declarations and root re-exports.
mod capability;
mod comprehensive;
mod ids;
mod legacy;
mod md5;
mod memory_tier;
mod module_type;
mod parse;
mod read;
mod relaxed_version;
mod script_module;
mod setting;
mod write;

pub use capability::{
    Capability, CapabilityCx, CapabilityRegistry, CapabilityValidation, CapabilityValidator,
    TrustLevel,
};
pub use memory_tier::MemoryTier;
pub use module_type::ModuleType;
pub use relaxed_version::RelaxedSemVersion;
#[doc(hidden)]
pub use script_module::{DependencyTexts, NativeDependencyText, dependency_texts};
pub use setting::{DropdownOption, Setting};

use crate::__private::Sealed;
use crate::document::{Content, Document, Extra, Rejected};
use crate::json::{EncodeError, WriteStyle};
use crate::load::LoadContext;
use crate::pack_identity::{PackIdVersion, PackPath, PackSide, PackType};
use crate::version::{BaseGameVersion, ENGINE_VERSION, MinEngineVersion, SemVersion};
use indexmap::IndexMap;
use uuid::Uuid;

use crate::pack_identity::PackScope;

/// A pack manifest (`manifest.json`), upgraded to the comprehensive (format 3) model.
///
/// Legacy manifests (`format_version` 0, 1, 2) and comprehensive ones (3) read into the same
/// value. It is written back in the format it was read in ([`Manifest::format`]); a new
/// manifest is format 2.
#[doc(alias = "manifest.json")]
#[derive(Clone, Debug, PartialEq)]
pub struct Manifest {
    /// The format the manifest is written in: the format read, except that format 0 reads as 1.
    pub format: ManifestFormat,
    /// The format the file was written in, 0 included.
    pub original_format: ManifestFormat,
    /// JSON `header`.
    pub header: Header,
    /// JSON `modules`. The last module decides the pack's type.
    pub modules: Vec<Module>,
    /// JSON `dependencies`.
    pub dependencies: Vec<Dependency>,
    /// JSON `capabilities`.
    pub capabilities: Vec<Capability>,
    /// JSON `metadata`.
    pub metadata: Option<Metadata>,
    /// JSON `subpacks`.
    pub subpacks: Vec<Subpack>,
    /// JSON `settings`: the pack settings' definitions.
    pub settings: Vec<Setting>,
    /// JSON `has_education_metadata` (format 3).
    pub has_education_metadata: Option<bool>,
    /// Unknown root members of a legacy manifest, kept for write-back.
    pub extra: Extra,
}

/// A manifest's `format_version` (vanilla `PackManifestFormat`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ManifestFormat {
    /// 0: the deprecated `pack_id` / `packs_version` layout, modules inside the header.
    V0,
    /// 1: legacy, `uuid` / `version`.
    V1,
    /// 2: legacy, `uuid` / `version`; the format of new manifests.
    V2,
    /// 3: the comprehensive format (strict members, string versions, settings).
    V3,
}

impl ManifestFormat {
    /// The `format_version` number.
    pub fn number(self) -> u8 {
        match self {
            Self::V0 => 0,
            Self::V1 => 1,
            Self::V2 => 2,
            Self::V3 => 3,
        }
    }
}

/// The manifest `header`.
#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    /// JSON `name`. An empty name reads as the UUID in format 0 and 1; from format 2 it stays
    /// empty, and reports show the pack as `manifestvalidation.noname`.
    pub name: String,
    /// JSON `description`.
    pub description: Option<String>,
    /// JSON `uuid` (format 0: `pack_id`): with `version`, the pack's identity.
    pub uuid: Uuid,
    /// JSON `version` (format 0: `packs_version`).
    pub version: SemVersion,
    /// JSON `min_engine_version`, as the pack uses it: a format-0/1 behaviour or resource pack
    /// without one gets 1.2.5, and one of 1.13.0 or above is capped to 1.12.0. A format-0/1
    /// manifest is written back with the value it read (kept in `extra` when it differs), or
    /// without the member when it had none, while this value is unchanged.
    pub min_engine_version: Option<MinEngineVersion>,
    /// JSON `base_game_version` (world templates).
    pub base_game_version: Option<BaseGameVersion>,
    /// JSON `pack_scope`: `global`, `world` or `any`. `None` when absent (the pack may then be
    /// used anywhere, as with `any`) or when the text names no scope; such text is kept in
    /// `extra` and written back.
    pub pack_scope: Option<PackScope>,
    /// JSON `platform_locked`.
    pub platform_locked: Option<bool>,
    /// JSON `allow_random_seed` (world templates).
    pub allow_random_seed: Option<bool>,
    /// JSON `lock_template_options` (world templates).
    pub lock_template_options: Option<bool>,
    /// JSON `pack_optimization_version`, kept as spelled.
    pub pack_optimization_version: Option<RelaxedSemVersion>,
    /// Unknown header members of a legacy manifest, a `pack_scope` that names no scope (any
    /// format), and a format-0/1 `min_engine_version` that is capped or the default as read,
    /// kept for write-back.
    pub extra: Extra,
}

impl Header {
    /// The name reports show: the header name, or `manifestvalidation.noname` when it is empty.
    #[doc(hidden)]
    pub fn display_name(&self) -> &str {
        if self.name.is_empty() {
            legacy::NO_NAME
        } else {
            &self.name
        }
    }
}

/// One `modules[]` entry.
#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    /// JSON `type`.
    pub module_type: ModuleType,
    /// JSON `uuid`.
    pub uuid: Uuid,
    /// JSON `version`.
    pub version: SemVersion,
    /// JSON `description`.
    pub description: Option<String>,
    /// JSON `entry`: a script module's main file.
    pub entry: Option<String>,
    /// JSON `language`: a script module's language (absent or empty is JavaScript).
    pub language: Option<String>,
    /// Unknown module members of a legacy manifest, kept for write-back.
    pub extra: Extra,
}

impl Module {
    /// A module of `module_type` with `uuid` and `version` and nothing else.
    pub fn new(module_type: ModuleType, uuid: Uuid, version: SemVersion) -> Module {
        Self {
            module_type,
            uuid,
            version,
            description: None,
            entry: None,
            language: None,
            extra: Extra::default(),
        }
    }
}

/// One `dependencies[]` entry: a pack by UUID and version, or a native module by name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Dependency {
    /// A pack: `{ "uuid": …, "version": … }`.
    Pack {
        /// JSON `uuid`.
        uuid: Uuid,
        /// JSON `version`.
        version: SemVersion,
    },
    /// A native script module: `{ "module_name": …, "version": … }`.
    NativeModule {
        /// JSON `module_name` (`@minecraft/server`).
        module_name: String,
        /// JSON `version`: a version or a selector.
        version: NativeModuleVersion,
        /// JSON `optional` (format 3 only).
        optional: Option<bool>,
    },
}

/// A native-module dependency's `version`: a version, or a selector for the newest of a
/// channel.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum NativeModuleVersion {
    /// A version (`"2.0.0"`, `"2.1.0-beta"`).
    Version(SemVersion),
    /// `"alpha"`: the newest alpha.
    LatestAlpha,
    /// `"beta"`: the newest beta.
    LatestBeta,
    /// `"internal"`: the newest internal version.
    LatestInternal,
}

/// One `subpacks[]` entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Subpack {
    /// JSON `name`.
    pub name: String,
    /// JSON `folder_name`: the folder under `subpacks/`.
    pub folder_name: String,
    /// The memory tier: JSON `memory_performance_tier` (format 3), or the tier of the legacy
    /// `memory_tier` (≥ 32 → 5, ≥ 24 → 4, ≥ 18 → 3, ≥ 12 → 2, 11 → 1, else 0).
    pub memory_tier: MemoryTier,
}

/// The manifest `metadata`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Metadata {
    /// JSON `authors`.
    pub authors: Option<Vec<String>>,
    /// JSON `license`.
    pub license: Option<String>,
    /// JSON `url`.
    pub url: Option<String>,
    /// JSON `product_type`: `""` or `addon`.
    pub product_type: Option<String>,
    /// JSON `generated_with`: tool name → versions, each kept as spelled (`"2.0"`).
    pub generated_with: Option<IndexMap<String, Vec<RelaxedSemVersion>>>,
}

impl Manifest {
    /// Parses a manifest: format detection, upgrade to the comprehensive model, validation.
    ///
    /// A `format_version` that reads as a version of 3.0.0 or above (`3`, `"3.0.0"`,
    /// `[3, 0, 0]`; a number is read as its text) selects the comprehensive reader; anything
    /// else the legacy one. A manifest is written with an integer `format_version`.
    /// Capabilities are checked against `capabilities`. Fails when any finding is an error, as
    /// vanilla refuses a pack whose report has errors; the warnings of a manifest that loads
    /// are in its document's diagnostics.
    pub fn parse(
        path: &PackPath,
        bytes: &[u8],
        capabilities: &CapabilityRegistry,
    ) -> Result<Document<Manifest>, Rejected> {
        // A number no double holds is read as the largest finite one, with a warning.
        crate::document::FiniteText::load(bytes, |bytes| parse::parse(path, bytes, capabilities))
    }

    /// A format-2 manifest with fresh random UUIDs for the header and one module of
    /// `pack_type`, version 1.0.0 and `min_engine_version` = `ENGINE_VERSION`.
    pub fn new(name: &str, pack_type: PackType) -> Manifest {
        Self::with_ids(name, pack_type, Uuid::new_v4(), Uuid::new_v4())
    }

    /// As [`Manifest::new`] with caller-chosen UUIDs: deterministic output for tests and generators.
    pub fn with_ids(name: &str, pack_type: PackType, header: Uuid, module: Uuid) -> Manifest {
        let version = SemVersion::new(1, 0, 0);
        Self {
            format: ManifestFormat::V2,
            original_format: ManifestFormat::V2,
            header: Header {
                name: name.to_owned(),
                description: None,
                uuid: header,
                version: version.clone(),
                min_engine_version: Some(MinEngineVersion {
                    version: SemVersion::from(ENGINE_VERSION),
                }),
                base_game_version: None,
                pack_scope: None,
                platform_locked: None,
                allow_random_seed: None,
                lock_template_options: None,
                pack_optimization_version: None,
                extra: Extra::default(),
            },
            modules: vec![Module::new(
                ModuleType::for_pack_type(pack_type),
                module,
                version,
            )],
            dependencies: Vec::new(),
            capabilities: Vec::new(),
            metadata: None,
            subpacks: Vec::new(),
            settings: Vec::new(),
            has_education_metadata: None,
            extra: Extra::default(),
        }
    }

    /// The pack type: the last module's, or [`PackType::Invalid`] without modules.
    pub fn pack_type(&self) -> PackType {
        legacy::pack_type(&self.modules)
    }

    /// The pack's identity: the header's `uuid` and `version`, with the pack type.
    pub fn identity(&self) -> PackIdVersion {
        PackIdVersion::new(
            self.header.uuid,
            self.header.version.clone(),
            self.pack_type(),
        )
    }

    /// Writes the manifest in [`Manifest::format`] to `out`. `NotRepresentable` when a legacy
    /// format cannot hold the data (a setting other than a label, a dependency's `optional`, a
    /// pre-release `min_engine_version`) or a slider number is not finite. On failure nothing
    /// is appended.
    pub fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        write::write(self, out, style)
    }

    /// [`Manifest::write_json`] into a string, pretty-printed with two spaces.
    pub fn to_json(&self) -> Result<String, EncodeError> {
        let mut out = Vec::new();
        self.write_json(&mut out, WriteStyle::default())?;
        // The writer emits UTF-8 only.
        Ok(String::from_utf8_lossy(&out).into_owned())
    }
}

impl Sealed for Manifest {}

/// `manifest.json`, read on both sides. [`Content::parse`] checks capabilities against
/// [`CapabilityRegistry::default`]; [`Manifest::parse`] takes another registry.
impl Content for Manifest {
    const NAME: &'static str = "manifest";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior, PackSide::Resource];

    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        Manifest::parse(path, bytes, &CapabilityRegistry::default())
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        Manifest::write_json(self, out, style)
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        Manifest::to_json(self)
    }
}
