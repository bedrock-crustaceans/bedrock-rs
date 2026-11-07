//! Packs and stacks: one `Pack`, the `PackRepository` of every pack found, the typed
//! `PackStack<S>` with its accessors, the `StackSet` of a world, and the `PackWriter`.

mod link;
pub(crate) mod open;
mod report;
mod repository;
pub mod settings;
pub(crate) mod stack;
mod writer;

pub use bedrock_addon_core::pack_identity::{
    PackIdVersion, PackOrigin, PackRef, PackScope, PackSide, PackType,
};
pub use link::LinkOptions;
#[allow(
    unused_imports,
    reason = "the families whose load holds a validator reference"
)]
pub(crate) use link::{Validator, validator_text};
pub use report::StackReport;
pub use repository::{InvalidPack, PackRepository, PackSelection, SourceReport, WorldPacks};
pub use stack::VANILLA_EXPERIMENTAL_PACKS;
pub use stack::{Behavior, PackStack, Resource, Side, StackSet, WrongPackType};
#[cfg(all(feature = "texture_file", feature = "texture_set"))]
pub use stack::{LayerImage, TextureFile};
pub use writer::{PackWriter, WriteError};

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, PackReport, Phase, Severity,
};
use bedrock_addon_core::load::{Limits, Place};
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::version::MinEngineVersion;
use bytes::Bytes;

use crate::manifest::{CapabilityRegistry, Dependency, Manifest};
use crate::storage::{ContentKeys, StorageError};

/// One pack: its manifest, its files and what went wrong reading them.
///
/// Cheap to clone (an `Arc` inside). A `Pack` is immutable; to change files, copy it into a
/// [`PackWriter`].
#[derive(Clone)]
pub struct Pack {
    inner: Arc<PackInner>,
}

/// The shared part of a [`Pack`]: the manifest, what composition reads of it, the file index
/// with the resident bytes, the report and the location. Built by the pack loader (`open.rs`).
pub(crate) struct PackInner {
    manifest: Arc<Manifest>,
    facts: PackFacts,
    files: PackFiles,
    report: PackReport,
    location: Place,
}

impl PackInner {
    /// A pack from its parts; what composition reads is taken from `manifest` once, here.
    pub(crate) fn new(
        manifest: impl Into<Arc<Manifest>>,
        files: PackFiles,
        report: PackReport,
        location: Place,
    ) -> Self {
        let manifest = manifest.into();
        Self {
            facts: PackFacts::of(&manifest),
            manifest,
            files,
            report,
            location,
        }
    }
}

/// What stack composition and the stack walk read of a manifest: identity, display name, the
/// pack types of the modules, pack dependencies, subpacks and `min_engine_version`.
#[derive(Clone, Debug)]
pub(crate) struct PackFacts {
    /// `(uuid, version)` of the header and the pack type the last module decides.
    pub(crate) id: PackIdVersion,
    /// The header name.
    pub(crate) name: Box<str>,
    /// The header `description` as written, if any.
    pub(crate) description: Option<Box<str>>,
    /// The names of the manifest's `capabilities`, as written.
    pub(crate) capabilities: Vec<Box<str>>,
    /// The pack type of every module (`data`, `script` → behaviour; `resources`,
    /// `client_script` → resources), in manifest order.
    pub(crate) module_types: Vec<PackType>,
    /// Pack dependencies by exact `(uuid, version)`; script-module dependencies
    /// (`@minecraft/server` and the like, by name or by the module's UUID) are not here: they
    /// name no pack.
    pub(crate) dependencies: Vec<PackIdVersion>,
    /// The `subpacks` entries, in manifest order.
    pub(crate) subpacks: Vec<SubpackFacts>,
    /// The header `min_engine_version`; `None` when absent or invalid.
    pub(crate) min_engine_version: Option<MinEngineVersion>,
}

/// Whether `uuid` is one of the two UUIDs a manifest may name a native script module by
/// (`@minecraft/server`, `@minecraft/server-ui`): the deprecated way to depend on the module,
/// which the script host reads and which names no pack.
pub(crate) fn is_native_module_uuid(uuid: uuid::Uuid) -> bool {
    matches!(
        uuid.as_u128(),
        0xb26a_4d4c_afdf_4690_88f8_9318_4631_2678 | 0x2bd5_0a27_ab5f_4f40_a596_3641_627c_635e
    )
}

#[cfg(all(test, feature = "script"))]
mod native_uuids {
    /// Stack composition leaves out exactly the UUIDs discovery reads as native modules.
    #[test]
    fn the_uuids_composition_skips_are_the_ones_discovery_knows() {
        for n in [
            0xb26a_4d4c_afdf_4690_88f8_9318_4631_2678_u128,
            0x2bd5_0a27_ab5f_4f40_a596_3641_627c_635e,
            0xb26a_4d4c_afdf_4690_88f8_9318_4631_2679,
            0,
        ] {
            let uuid = uuid::Uuid::from_u128(n);
            assert_eq!(
                super::is_native_module_uuid(uuid),
                crate::families::script::native_module_of(uuid).is_some(),
                "{uuid}"
            );
        }
    }
}

impl PackFacts {
    /// What composition reads of `manifest`: its identity (the last module decides the pack
    /// type), header name, every module's pack type, the pack dependencies (native-module ones
    /// are the script host's), the subpacks and the header `min_engine_version`.
    pub(crate) fn of(manifest: &Manifest) -> PackFacts {
        PackFacts {
            id: manifest.identity(),
            name: manifest.header.display_name().into(),
            description: manifest.header.description.as_deref().map(Box::from),
            capabilities: manifest
                .capabilities
                .iter()
                .map(|capability| Box::from(capability.name()))
                .collect(),
            module_types: manifest
                .modules
                .iter()
                .map(|module| module.module_type.pack_type())
                .collect(),
            dependencies: manifest
                .dependencies
                .iter()
                .filter_map(|dependency| match dependency {
                    // A dependency on a script module's UUID names no pack, so it is not a pack
                    // dependency.
                    Dependency::Pack { uuid, .. } if is_native_module_uuid(*uuid) => None,
                    Dependency::Pack { uuid, version } => Some(PackIdVersion::new(
                        *uuid,
                        version.clone(),
                        PackType::Invalid,
                    )),
                    Dependency::NativeModule { .. } => None,
                })
                .collect(),
            subpacks: manifest
                .subpacks
                .iter()
                .map(|subpack| SubpackFacts {
                    folder_name: subpack.folder_name.as_str().into(),
                    name: subpack.name.as_str().into(),
                    memory_tier: subpack.memory_tier.number(),
                })
                .collect(),
            min_engine_version: manifest.header.min_engine_version.clone(),
        }
    }

    /// The pack has a module of `pack_type`'s kind (`data` for behaviour, `resources` for
    /// resources); a pack with both is either.
    pub(crate) fn has_module(&self, pack_type: PackType) -> bool {
        self.id.pack_type == pack_type || self.module_types.contains(&pack_type)
    }
}

/// One `subpacks` entry as composition reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SubpackFacts {
    /// `folder_name`: the folder under `subpacks/`.
    pub(crate) folder_name: Box<str>,
    /// `name`.
    pub(crate) name: Box<str>,
    /// The hardware memory tier, 0 to 5 (legacy `memory_tier` values already mapped).
    pub(crate) memory_tier: u8,
}

/// A pack's file index: every path, bytewise, with the bytes of the resident ones.
#[derive(Clone, Debug, Default)]
pub(crate) struct PackFiles {
    entries: BTreeMap<PackPath, FileSlot>,
    /// The archive, brarchive layer and encryption behind the index (`open.rs`).
    store: open::Store,
}

/// One indexed file.
#[derive(Clone, Debug)]
pub(crate) enum FileSlot {
    /// Loaded: the bytes as stored.
    Resident(Bytes),
    /// Indexed only.
    Indexed,
    /// In the pack's resident archive (`open::Archive`).
    #[cfg_attr(not(feature = "zip"), allow(dead_code))]
    Archived,
}

impl PackFiles {
    /// Adds or replaces the file at `path`.
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "the in-crate tests build packs with it")
    )]
    pub(crate) fn insert(&mut self, path: PackPath, slot: FileSlot) {
        self.entries.insert(path, slot);
    }

    /// Every path, bytewise.
    pub(crate) fn paths(&self) -> impl Iterator<Item = &PackPath> + '_ {
        open::Union::new(self.entries.keys(), self.brarchive_paths())
    }
}

/// Resident files; a path given twice keeps its last bytes.
impl FromIterator<(PackPath, Bytes)> for PackFiles {
    fn from_iter<I: IntoIterator<Item = (PackPath, Bytes)>>(iter: I) -> Self {
        Self {
            entries: iter
                .into_iter()
                .map(|(path, bytes)| (path, FileSlot::Resident(bytes)))
                .collect(),
            store: open::Store::default(),
        }
    }
}

impl Pack {
    /// A pack around its shared part.
    pub(crate) fn from_inner(inner: PackInner) -> Pack {
        Pack {
            inner: Arc::new(inner),
        }
    }

    /// What composition reads of the manifest.
    pub(crate) fn facts(&self) -> &PackFacts {
        &self.inner.facts
    }

    /// The bytes of one file: inflated and decrypted, from the resident files, archive or
    /// brarchive layer; `NotResident` when it is indexed only.
    pub(crate) fn resident(&self, path: &PackPath) -> Result<Bytes, StorageError> {
        self.inner.files.read(path)
    }

    /// The parsed manifest, upgraded to the comprehensive (format 3) model.
    pub fn manifest(&self) -> &Manifest {
        &self.inner.manifest
    }

    /// The parsed manifest, shared with whoever else holds it (a stack's entry of the pack).
    pub(crate) fn shared_manifest(&self) -> Arc<Manifest> {
        Arc::clone(&self.inner.manifest)
    }

    /// `(uuid, version)` of the header, the identity a stack refers to.
    pub fn id(&self) -> &PackIdVersion {
        &self.inner.facts.id
    }

    /// Behaviour, resources, skins, world template, …, from the manifest's modules.
    pub fn pack_type(&self) -> PackType {
        self.inner.facts.id.pack_type
    }

    /// Manifest and container findings (vanilla's `PackReport` field set).
    pub fn report(&self) -> &PackReport {
        &self.inner.report
    }

    /// `pack_icon.png`, if resident and decodable as PNG.
    // "Decodable" means the bytes start with the PNG signature; the crate carries no image
    // decoder.
    pub fn icon(&self) -> Option<Bytes> {
        self.resident(&PackPath::from_static(ICON_PATH))
            .ok()
            .filter(|bytes| bytes.starts_with(&bedrock_addon_shared_shapes::PNG_SIGNATURE))
    }

    /// The lints of `pack_icon.png` (the crate's own: the game shows no icon and says nothing):
    /// a pack without the file is `PackIconMissing` (info); an icon whose bytes are resident goes
    /// through [`Pack::check_icon`]. A file that is indexed and not read is there and is not
    /// judged: a host that reads it calls `check_icon` itself. So the two lints of the image
    /// (not a PNG, not square) run only for a pack built with the icon's bytes resident
    /// ([`Pack::from_files`], [`Pack::from_archive`]); for a pack a loader opened from a folder only
    /// `PackIconMissing` can be raised, since no load asks for a byte of an image. Not part
    /// of [`Pack::report`] (the game's field set); a stack lists them among its own findings
    /// ([`PackStack::report`](crate::pack::PackStack::report)).
    // The game reads no pack icon at load and says nothing about it, so these lints are the
    // crate's own.
    #[must_use]
    pub fn icon_findings(&self) -> Diagnostics {
        let path = PackPath::from_static(ICON_PATH);
        if !self.inner.files.entries.contains_key(&path) {
            let mut diagnostics = Diagnostics::default();
            diagnostics.push(icon_finding(
                DiagnosticCode::from(&core_codes::PACK_ICON_MISSING),
                Severity::Info,
            ));
            return diagnostics;
        }
        match self.resident(&path) {
            Ok(bytes) => Pack::check_icon(&bytes),
            Err(_) => Diagnostics::default(),
        }
    }

    /// The lints of the bytes of a `pack_icon.png`: bytes that do not start as a PNG file
    /// with its header are an `InvalidValue` information; an image whose width differs from its
    /// height is `PackIconShape` (info). This is where the two lints run for a pack whose icon
    /// is indexed and not read: the caller reads `pack_icon.png` and passes the bytes; no load
    /// of this crate requests them.
    // The reader is `bedrock_addon_shared_shapes::png_size`, the one the glyph sheet lint uses.
    #[must_use]
    pub fn check_icon(bytes: &[u8]) -> Diagnostics {
        let mut diagnostics = Diagnostics::default();
        match bedrock_addon_shared_shapes::png_size(bytes) {
            None => diagnostics.push(icon_finding(
                // The game reads no icon bytes at load, so bytes that are not a PNG are not a
                // finding of the game's: the crate reports them as an information.
                DiagnosticCode::from(&core_codes::INVALID_VALUE),
                Severity::Info,
            )),
            Some((width, height)) if width != height => {
                diagnostics.push(icon_finding(
                    DiagnosticCode::from(&core_codes::PACK_ICON_SHAPE),
                    Severity::Info,
                ));
            }
            Some(_) => {}
        }
        diagnostics
    }

    /// Where a loader found the pack; `RootId::MEMORY` for `from_files` / `from_archive`.
    pub fn location(&self) -> &Place {
        &self.inner.location
    }

    /// Whether the file's bytes are loaded, not only indexed.
    pub fn is_resident(&self, path: &PackPath) -> bool {
        self.inner.files.is_resident(path)
    }

    /// Every file path in the pack, sorted bytewise.
    pub fn paths(&self) -> impl Iterator<Item = &PackPath> + '_ {
        self.inner.files.paths()
    }
}

/// The pack icon, at the pack root.
const ICON_PATH: &str = "pack_icon.png";

/// A finding at the pack icon.
fn icon_finding(code: DiagnosticCode, severity: Severity) -> Diagnostic {
    Diagnostic::new(
        code,
        severity,
        Phase::Validate,
        Location::new(
            None,
            Some(PackPath::from_static(ICON_PATH)),
            JsonPath::new(),
            None,
        ),
    )
}

/// The identity and the location, never the files.
impl fmt::Debug for Pack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pack")
            .field("id", self.id())
            .field("pack_type", &self.pack_type())
            .field("location", self.location())
            .finish_non_exhaustive()
    }
}

/// How a pack is opened: its origin, the content keys of encrypted packs, the manifest
/// capabilities the host grants and the zip and window limits.
///
/// Built as `OpenOptions { keys, ..OpenOptions::default() }`.
#[derive(Clone)]
pub struct OpenOptions {
    /// Where the pack comes from; `Package` makes it a base-game pack. Default `Unknown`; a
    /// `RepositoryLoader` gives each pack its source's origin.
    pub origin: PackOrigin,
    /// The content keys of encrypted packs, by `ContentIdentity`. A key the map lacks is asked
    /// for with `Output::NeedKey`.
    pub keys: ContentKeys,
    /// The manifest capabilities the host grants (`Manifest::parse`).
    pub capabilities: CapabilityRegistry,
    /// The zip-bomb limits, the archive size limit and the read and job windows.
    pub limits: Limits,
}

/// Origin `Unknown`, no keys, the default capability registry and limits.
impl Default for OpenOptions {
    fn default() -> Self {
        Self {
            origin: PackOrigin::Unknown,
            keys: ContentKeys::default(),
            capabilities: CapabilityRegistry::default(),
            limits: Limits::default(),
        }
    }
}

/// The origin, the key identities (never the keys) and the limits.
impl fmt::Debug for OpenOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenOptions")
            .field("origin", &self.origin)
            .field("keys", &self.keys)
            .field("limits", &self.limits)
            .finish_non_exhaustive()
    }
}

/// Why a [`PackLoader`](crate::load::PackLoader) (or `Pack::from_files` / `Pack::from_archive`)
/// returned no pack.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum OpenError {
    /// The pack root could not be read.
    #[error("{location}: cannot read ({kind})")]
    Unreadable {
        /// The pack root.
        location: Place,
        /// The driver's error kind.
        kind: std::io::ErrorKind,
    },
    /// The container could not be decoded.
    #[error(transparent)]
    Storage(#[from] StorageError),
    /// No manifest at the root or in the single subdirectory holding one.
    #[error("no manifest.json in {location}")]
    NoManifest {
        /// The pack root.
        location: Place,
    },
    /// Several manifests at the same depth.
    #[error("{count} manifests at the same depth in {location}")]
    MultipleManifests {
        /// The pack root.
        location: Place,
        /// How many.
        count: usize,
    },
    /// The manifest was read but is unusable (no valid UUID, no modules, …).
    #[error("invalid manifest in {location}: {}", .report.summary())]
    InvalidManifest {
        /// The pack root.
        location: Place,
        /// The manifest findings.
        report: Box<PackReport>,
    },
    /// A `Manifest` or archive `Index` job panicked.
    #[error("{location}: internal error while opening the pack")]
    Internal {
        /// The pack root.
        location: Place,
    },
}

#[cfg(test)]
pub(crate) mod tests {
    use bedrock_addon_core::load::RootId;
    use bedrock_addon_core::version::SemVersion;
    use uuid::Uuid;

    use super::*;
    use crate::manifest::CapabilityRegistry;

    /// A pack in memory from a manifest text and resident files (`None` bytes: indexed only),
    /// as the pack loader builds one.
    pub(crate) fn pack_from(
        manifest: &str,
        files: &[(&'static str, Option<&'static [u8]>)],
        origin: PackOrigin,
    ) -> Pack {
        let manifest = Manifest::parse(
            &PackPath::from_static("manifest.json"),
            manifest.as_bytes(),
            &CapabilityRegistry::default(),
        )
        .expect("the manifest parses")
        .into_value();
        pack_of(manifest, files, origin)
    }

    /// A pack in memory around a manifest value.
    pub(crate) fn pack_of(
        manifest: Manifest,
        files: &[(&'static str, Option<&'static [u8]>)],
        origin: PackOrigin,
    ) -> Pack {
        let mut index = PackFiles::default();
        for (path, bytes) in files {
            let slot = bytes.map_or(FileSlot::Indexed, |bytes| {
                FileSlot::Resident(Bytes::from_static(bytes))
            });
            index.insert(PackPath::from_static(path), slot);
        }
        let location = Place::root(RootId::MEMORY);
        let report = PackReport::new(location.clone(), manifest.pack_type(), origin);
        Pack::from_inner(PackInner::new(manifest, index, report, location))
    }

    /// Format 2: behaviour and script modules, a pack and a native dependency, two subpacks
    /// with legacy memory tiers.
    const MANIFEST: &str = r#"{
  "format_version": 2,
  "header": {
    "name": "Ruby Pack",
    "description": "",
    "uuid": "00000000-0000-4000-8000-000000000001",
    "version": [1, 2, 3],
    "min_engine_version": [1, 21, 0]
  },
  "modules": [
    { "type": "data", "uuid": "00000000-0000-4000-8000-000000000002", "version": [1, 0, 0] },
    { "type": "script", "language": "javascript", "entry": "scripts/main.js",
      "uuid": "00000000-0000-4000-8000-000000000003", "version": [1, 0, 0] }
  ],
  "dependencies": [
    { "uuid": "00000000-0000-4000-8000-000000000009", "version": [2, 0, 0] },
    { "module_name": "@minecraft/server", "version": "1.11.0" }
  ],
  "subpacks": [
    { "folder_name": "low", "name": "Low", "memory_tier": 0 },
    { "folder_name": "high", "name": "High", "memory_tier": 24 }
  ]
}"#;

    #[test]
    fn facts_are_read_from_the_manifest() {
        let pack = pack_from(MANIFEST, &[], PackOrigin::World);
        let facts = pack.facts();
        let uuid = |n: u128| Uuid::from_u128(0x0000_0000_0000_4000_8000_0000_0000_0000 | n);
        assert_eq!(
            facts.id,
            PackIdVersion::new(uuid(1), SemVersion::new(1, 2, 3), PackType::Behavior)
        );
        assert_eq!(facts.id.pack_type, PackType::Behavior);
        assert_eq!(&*facts.name, "Ruby Pack");
        assert_eq!(facts.module_types, [PackType::Behavior, PackType::Behavior]);
        assert_eq!(
            facts.dependencies,
            [PackIdVersion::new(
                uuid(9),
                SemVersion::new(2, 0, 0),
                PackType::Invalid
            )]
        );
        let subpacks: Vec<(&str, &str, u8)> = facts
            .subpacks
            .iter()
            .map(|s| (&*s.folder_name, &*s.name, s.memory_tier))
            .collect();
        assert_eq!(subpacks, [("low", "Low", 0), ("high", "High", 4)]);
        assert_eq!(
            facts.min_engine_version.as_ref().map(|v| v.version.clone()),
            Some(SemVersion::new(1, 21, 0))
        );
        assert_eq!(*pack.id(), facts.id);
        assert_eq!(pack.pack_type(), PackType::Behavior);
    }

    /// A PNG file's first bytes: the signature and the start of `IHDR`.
    const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR";

    #[test]
    fn the_accessors_read_the_manifest_the_files_and_the_location() {
        let pack = pack_from(
            MANIFEST,
            &[
                ("pack_icon.png", Some(PNG)),
                ("textures/b.png", Some(b"b")),
                ("textures/c.png", None),
            ],
            PackOrigin::World,
        );
        assert_eq!(pack.manifest().header.name, "Ruby Pack");
        assert_eq!(pack.manifest().identity(), *pack.id());
        assert_eq!(*pack.location(), Place::root(RootId::MEMORY));
        assert_eq!(pack.report().pack_origin, PackOrigin::World);
        assert!(pack.is_resident(&PackPath::from_static("textures/b.png")));
        assert!(!pack.is_resident(&PackPath::from_static("textures/c.png")));
        assert!(!pack.is_resident(&PackPath::from_static("textures/d.png")));
        assert_eq!(pack.icon().as_deref(), Some(PNG));
    }

    #[test]
    fn an_icon_is_a_resident_png() {
        let not_png = pack_from(
            MANIFEST,
            &[("pack_icon.png", Some(b"GIF89a"))],
            PackOrigin::World,
        );
        assert!(not_png.icon().is_none());
        let indexed = pack_from(MANIFEST, &[("pack_icon.png", None)], PackOrigin::World);
        assert!(indexed.icon().is_none());
        assert!(pack_from(MANIFEST, &[], PackOrigin::World).icon().is_none());
    }

    #[test]
    fn pack_scope_carries_vanillas_flag_values() {
        let bits: Vec<u8> = [
            PackScope::Global,
            PackScope::AddOn,
            PackScope::Level,
            PackScope::System,
            PackScope::World,
            PackScope::Any,
        ]
        .into_iter()
        .map(PackScope::bits)
        .collect();
        assert_eq!(bits, [1, 2, 4, 8, 6, 7]);
        assert_eq!(PackScope::default(), PackScope::Any);
    }

    #[test]
    fn world_scope_is_add_on_and_level() {
        assert!(PackScope::World.contains(PackScope::AddOn));
        assert!(PackScope::World.contains(PackScope::Level));
        assert!(!PackScope::World.contains(PackScope::Global));
        assert!(PackScope::Any.contains(PackScope::World));
        assert!(!PackScope::Any.contains(PackScope::System));
    }

    #[test]
    fn pack_scope_json_names_are_global_system_world_any() {
        for scope in [
            PackScope::Global,
            PackScope::System,
            PackScope::World,
            PackScope::Any,
        ] {
            let name = scope.json_name().expect("spelled scope");
            assert_eq!(PackScope::from_json_name(name), Some(scope));
        }
        assert_eq!(PackScope::from_json_name("level"), None);
        assert_eq!(PackScope::from_json_name("World"), None);
        assert_eq!(PackScope::from_json_name("System"), None);
        assert_eq!(PackScope::AddOn.json_name(), None);
        assert_eq!(PackScope::Level.json_name(), None);
    }

    fn pack_with(files: &[(&'static str, &'static [u8])]) -> PackFiles {
        files
            .iter()
            .map(|(path, bytes)| (PackPath::from_static(path), Bytes::from_static(bytes)))
            .collect()
    }

    #[test]
    fn files_are_listed_bytewise_and_resident_ones_read() {
        let mut files = pack_with(&[("texts/en_US.lang", b"a=b"), ("Blocks/a.json", b"{}")]);
        files.insert(PackPath::from_static("blocks/b.json"), FileSlot::Indexed);
        let paths: Vec<&str> = files.paths().map(PackPath::as_str).collect();
        assert_eq!(
            paths,
            ["Blocks/a.json", "blocks/b.json", "texts/en_US.lang"]
        );
        assert_eq!(
            files.read(&PackPath::from_static("texts/en_US.lang")).ok(),
            Some(Bytes::from_static(b"a=b"))
        );
        assert!(matches!(
            files.read(&PackPath::from_static("blocks/b.json")),
            Err(StorageError::NotResident)
        ));
        assert!(matches!(
            files.read(&PackPath::from_static("blocks/c.json")),
            Err(StorageError::NotFound)
        ));
    }
}
