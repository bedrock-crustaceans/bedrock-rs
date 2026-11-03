//! Where a Molang field's version comes from: the pack's `min_engine_version`, the
//! document's `format_version`, or a constant, per field.

use semver::{Prerelease, Version};

use super::{EngineVersion, ExperimentMask, MolangVersion, RawVersion};
use crate::version::{FormatVersionInput, MinEngineVersion, SemVersion};

/// The pack's Molang version (vanilla maps the manifest's
/// `min_engine_version` to one): `None`, an absent or invalid version, is
/// `MolangVersion::Invalid` (−1). Format-1 manifests pass their substitute
/// (`MinEngineVersion::format_1`), which maps to 0. Sub-packs use the parent's.
#[doc(hidden)]
pub fn pack_molang_version(min_engine_version: Option<&MinEngineVersion>) -> RawVersion {
    min_engine_version
        .map_or(MolangVersion::Invalid, MinEngineVersion::molang_version)
        .into()
}

/// A document's `format_version` mapped as an engine version (vanilla reads the document's
/// version as a minimum engine version): a pre-release tag sorts below its release, and the
/// `"beta"` literal is the beta sentinel 9999.9999.9999-beta, so the newest version.
#[doc(hidden)]
pub fn format_molang_version(version: &FormatVersionInput) -> RawVersion {
    let Some(numbers) = version.version else {
        // The `"beta"` literal.
        return MolangVersion::LATEST.into();
    };
    let mut engine = Version::new(
        u64::from(numbers.major),
        u64::from(numbers.minor),
        u64::from(numbers.patch),
    );
    if version.prerelease {
        // Only "has a tag" takes part in the mapping's order in molang.
        engine.pre = Prerelease::new("pre").unwrap_or(Prerelease::EMPTY);
    }
    MolangVersion::from(&EngineVersion::Version(engine)).into()
}

/// The engine version from which animation controllers take the pack's version instead of the
/// file's (vanilla `ANIMATION_CONTROLLER_ENGINE_FIX_VERSION`).
const CONTROLLER_ENGINE_FIX: SemVersion = SemVersion::new(1, 20, 50);

/// The inputs a field's version is chosen from, once per document: the pack's version, whether
/// the pack predates the controller fix, the document's version, the version a merge injected,
/// and the world's experiments for the compile pass.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MolangContext {
    /// The pack's version (`LoadContext::molang_version`).
    pub pack: RawVersion,
    /// The pack's `min_engine_version` is below 1.20.50, or absent.
    pub pack_before_controller_fix: bool,
    /// The document's `format_version`, mapped.
    pub document: RawVersion,
    /// The `__molang_version` a merge wrote into the entry (`sounds.json`), if any.
    pub injected: Option<RawVersion>,
    /// The world's experiments as molang's mask (`LoadContext::molang_experiments`).
    pub experiments: ExperimentMask,
}

impl MolangContext {
    /// The context of a document of version `document` in a pack whose manifest gave
    /// `min_engine_version` (`None`: absent or invalid), with the world's `experiments`. The
    /// pack's version is `pack_version` when the caller has it (`LoadContext::molang_version`,
    /// the one source of truth for a loaded pack), else the mapping of `min_engine_version`.
    /// Nothing injected.
    pub fn new(
        min_engine_version: Option<&MinEngineVersion>,
        experiments: ExperimentMask,
        document: &FormatVersionInput,
        pack_version: Option<RawVersion>,
    ) -> Self {
        // An absent version is vanilla's zeroed `SemVersion`, below every version.
        let pack_before_controller_fix =
            min_engine_version.is_none_or(|v| v.version < CONTROLLER_ENGINE_FIX);
        Self {
            pack: pack_version.unwrap_or_else(|| pack_molang_version(min_engine_version)),
            pack_before_controller_fix,
            document: format_molang_version(document),
            injected: None,
            experiments,
        }
    }

    /// A context outside any pack (`LoadContext::detached`): the newest version everywhere.
    pub fn detached() -> Self {
        Self {
            pack: MolangVersion::LATEST.into(),
            pack_before_controller_fix: false,
            document: MolangVersion::LATEST.into(),
            injected: None,
            experiments: ExperimentMask::empty(),
        }
    }
}

/// Where a field's version comes from; one per field spec.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MolangVersionSource {
    /// The pack's `min_engine_version`: the default.
    Pack,
    /// The document's `format_version`: item files, legacy spawn rules and spawn groups `tags`.
    FormatVersion,
    /// Animation controllers: the document's `format_version` while the pack's
    /// `min_engine_version` is below 1.20.50, else the pack's.
    ControllerBackCompat,
    /// A fixed version: block files from 1.26.20 and cereal block descriptor `tags` (Molang
    /// version 13), biome `height_range` (1), persona emote animations (0).
    Constant(MolangVersion),
    /// The pack's version, also for an object form, which pass 2 compiles at the pack's version:
    /// feature and feature rule scatter parameters.
    PackIgnoringObject,
    /// The `__molang_version` a merge injected, else the author's, else 0: `sounds.json` variant
    /// keys.
    Injected,
}

impl MolangVersionSource {
    /// The raw version a string form takes in pass 1; an object form keeps its own.
    pub fn resolve(self, cx: &MolangContext) -> RawVersion {
        match self {
            Self::ControllerBackCompat if cx.pack_before_controller_fix => cx.document,
            Self::Pack | Self::PackIgnoringObject | Self::ControllerBackCompat => cx.pack,
            Self::FormatVersion => cx.document,
            Self::Constant(version) => version.into(),
            Self::Injected => cx.injected.unwrap_or(RawVersion(0)),
        }
    }
}
