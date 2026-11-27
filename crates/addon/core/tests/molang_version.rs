//! Molang versions: the pack's version from its
//! `min_engine_version` over every range of the game's Molang version table and the pack kinds, the
//! document's from its `format_version`, and one test per `MolangVersionSource`.

use bedrock_addon_core::molang::{
    self, ExperimentMask, MolangContext, MolangVersion, MolangVersionSource, RawVersion,
};
use bedrock_addon_core::version::{FormatVersionInput, MinEngineVersion, SemVersion};

fn engine(text: &str) -> MinEngineVersion {
    MinEngineVersion {
        version: SemVersion::parse(text).expect("a version"),
    }
}

fn pack(text: &str) -> RawVersion {
    molang::pack_molang_version(Some(&engine(text)))
}

fn document(text: &str) -> RawVersion {
    molang::format_molang_version(&FormatVersionInput::parse(text).expect("a format version"))
}

/// The first version of every row and the last version before it.
const RANGES: [(&str, &str, i16); 13] = [
    ("0.0.0", "1.16.220", 0),
    ("1.17.0", "1.17.29", 1),
    ("1.17.30", "1.17.39", 2),
    ("1.17.40", "1.18.9", 4),
    ("1.18.10", "1.18.19", 5),
    ("1.18.20", "1.19.59", 6),
    ("1.19.60", "1.19.99", 7),
    ("1.20.0", "1.20.9", 8),
    ("1.20.10", "1.20.39", 9),
    ("1.20.40", "1.20.49", 10),
    ("1.20.50", "1.20.69", 11),
    ("1.20.70", "1.21.99", 12),
    ("1.21.100", "1.26.50", 13),
];

#[test]
fn every_range_of_the_version_mapping() {
    for (first, last, version) in RANGES {
        assert_eq!(pack(first), RawVersion(version), "{first}");
        assert_eq!(pack(last), RawVersion(version), "{last}");
    }
}

#[test]
fn no_engine_version_maps_to_three() {
    assert_eq!(pack("1.17.39"), RawVersion(2));
    assert_eq!(pack("1.17.40"), RawVersion(4));
    for (first, last, _) in RANGES {
        assert_ne!(pack(first), RawVersion(3), "{first}");
        assert_ne!(pack(last), RawVersion(3), "{last}");
    }
}

#[test]
fn a_pre_release_maps_below_its_release() {
    assert_eq!(pack("1.21.100-beta"), RawVersion(12));
    assert_eq!(pack("1.21.100-beta.3"), RawVersion(12));
    assert_eq!(pack("1.17.0-preview"), RawVersion(0));
    assert_eq!(pack("1.21.101-beta"), RawVersion(13));
}

#[test]
fn the_any_version_maps_to_the_newest() {
    assert_eq!(pack("*"), RawVersion::from(MolangVersion::LATEST));
}

/// The pack kinds.
#[test]
fn the_pack_kinds_of_the_manifest() {
    // BP / RP, format 2 or 3: the mapping of `min_engine_version`.
    assert_eq!(pack("1.20.50"), RawVersion(11));
    // BP / RP, format 1, field missing: 1.2.5.
    assert_eq!(
        molang::pack_molang_version(Some(&MinEngineVersion::format_1(None))),
        RawVersion(0)
    );
    // BP / RP, format 1, 1.13.0 or above: capped to 1.12.0.
    assert_eq!(
        molang::pack_molang_version(Some(&MinEngineVersion::format_1(SemVersion::parse(
            "1.20.0"
        )))),
        RawVersion(0)
    );
    // Format 2 or above without the field, and every other pack type without it: invalid.
    assert_eq!(
        molang::pack_molang_version(None),
        RawVersion::from(MolangVersion::Invalid)
    );
}

#[test]
fn a_documents_format_version_maps_like_an_engine_version() {
    assert_eq!(document("1.20.40"), RawVersion(10));
    assert_eq!(document("1.20.50"), RawVersion(11));
    assert_eq!(document("1.10.0"), RawVersion(0));
    assert_eq!(document("1.21.100-beta.1"), RawVersion(12));
    // The `"beta"` literal is the beta sentinel 9999.9999.9999-beta.
    assert_eq!(document("beta"), RawVersion(13));
}

fn context(pack_version: &str, format_version: &str) -> MolangContext {
    MolangContext::new(
        Some(&engine(pack_version)),
        ExperimentMask::empty(),
        &FormatVersionInput::parse(format_version).expect("a format version"),
        None,
    )
}

#[test]
fn source_pack_is_the_packs_version() {
    let cx = context("1.18.20", "1.21.0");
    assert_eq!(MolangVersionSource::Pack.resolve(&cx), RawVersion(6));
}

/// Item files: the file's `format_version`, not the pack's (vanilla's `setMinEngineVersion`
/// window).
#[test]
fn source_format_version_is_the_documents_version() {
    let cx = context("1.21.100", "1.20.40");
    assert_eq!(
        MolangVersionSource::FormatVersion.resolve(&cx),
        RawVersion(10)
    );
    let cx = context("1.16.0", "1.21.0");
    assert_eq!(
        MolangVersionSource::FormatVersion.resolve(&cx),
        RawVersion(12)
    );
}

/// Animation controllers: the file's `format_version` while the pack is below 1.20.50, the pack's
/// from 1.20.50.
#[test]
fn source_controller_back_compat_switches_at_1_20_50() {
    let source = MolangVersionSource::ControllerBackCompat;
    assert_eq!(source.resolve(&context("1.20.49", "1.10.0")), RawVersion(0));
    assert_eq!(
        source.resolve(&context("1.20.40", "1.19.60")),
        RawVersion(7)
    );
    assert_eq!(
        source.resolve(&context("1.20.50", "1.10.0")),
        RawVersion(11)
    );
    assert_eq!(
        source.resolve(&context("1.21.100", "1.10.0")),
        RawVersion(13)
    );
    // An invalid pack version is below 1.20.50.
    let cx = MolangContext::new(
        None,
        ExperimentMask::empty(),
        &FormatVersionInput::parse("1.19.60").unwrap(),
        None,
    );
    assert_eq!(source.resolve(&cx), RawVersion(7));
}

/// Block files from 1.26.20 (13), biome `height_range` (1), persona emotes (0).
#[test]
fn source_constant_ignores_pack_and_document() {
    let cx = context("1.16.0", "1.10.0");
    for version in [MolangVersion::V13, MolangVersion::V1, MolangVersion::V0] {
        assert_eq!(
            MolangVersionSource::Constant(version).resolve(&cx),
            RawVersion::from(version)
        );
    }
}

/// Scatter parameters: the pack's version (pass 2 also compiles an object form at it).
#[test]
fn source_pack_ignoring_object_is_the_packs_version() {
    let cx = context("1.19.60", "1.21.0");
    assert_eq!(
        MolangVersionSource::PackIgnoringObject.resolve(&cx),
        RawVersion(7)
    );
}

/// `sounds.json` variant keys: the `__molang_version` the merge injected, else 0 (the
/// interactive sounds, which the injection never reaches).
#[test]
fn source_injected_is_the_injected_version_else_zero() {
    let cx = context("1.21.100", "1.21.0");
    assert_eq!(MolangVersionSource::Injected.resolve(&cx), RawVersion(0));
    let injected = MolangContext {
        injected: Some(RawVersion(11)),
        ..cx
    };
    assert_eq!(
        MolangVersionSource::Injected.resolve(&injected),
        RawVersion(11)
    );
}

/// The load context's version is the pack's version when the caller has it; the experiments are
/// copied.
#[test]
fn the_load_contexts_version_is_the_packs_version() {
    let experiments = ExperimentMask::empty().with(molang::Experiment::new(3).unwrap());
    let cx = MolangContext::new(
        Some(&engine("1.16.0")),
        experiments,
        &FormatVersionInput::parse("1.21.0").unwrap(),
        Some(RawVersion(9)),
    );
    assert_eq!(MolangVersionSource::Pack.resolve(&cx), RawVersion(9));
    assert_eq!(cx.experiments, experiments);
    // The controller rule still reads the manifest's version.
    assert_eq!(
        MolangVersionSource::ControllerBackCompat.resolve(&cx),
        RawVersion(12)
    );
}

#[test]
fn a_detached_context_is_the_newest_version() {
    let cx = MolangContext::detached();
    for source in [
        MolangVersionSource::Pack,
        MolangVersionSource::FormatVersion,
        MolangVersionSource::ControllerBackCompat,
        MolangVersionSource::PackIgnoringObject,
    ] {
        assert_eq!(source.resolve(&cx), RawVersion::from(MolangVersion::LATEST));
    }
}

/// The engine version a `min_engine_version` hands molang: the any-version for `*`, the three
/// numbers with the written tag.
#[test]
fn a_min_engine_version_is_the_engine_version_with_its_tag() {
    use bedrock_addon_core::molang::EngineVersion;
    let text = |written: &str| match engine(written).engine_version() {
        EngineVersion::Version(version) => version.to_string(),
        EngineVersion::Any => "*".to_owned(),
    };
    assert_eq!(text("1.21.100"), "1.21.100");
    assert_eq!(text("1.21.100-beta.3"), "1.21.100-beta.3");
    assert!(matches!(engine("*").engine_version(), EngineVersion::Any));
}
