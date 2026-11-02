//! The chains of the roots and the compositions: one struct each, and the range of the
//! accepted `format_version`.

use bedrock_addon_core::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    Family, RangeModel, Selection, VersionChain, check_chain, select, select_versions,
};
use bedrock_addon_family_ddui::FORMAT_VERSION;
use bedrock_addon_family_ddui::history::{CompositionFamily, RootFamily};

#[test]
fn the_chains_hold_the_one_struct() {
    assert_eq!(
        <<RootFamily as Family>::Any as VersionChain<RootFamily>>::VERSIONS,
        [FORMAT_VERSION]
    );
    assert_eq!(
        <<CompositionFamily as Family>::Any as VersionChain<CompositionFamily>>::VERSIONS,
        [FORMAT_VERSION]
    );
}

#[test]
fn the_chains_are_contiguous_closed_at_their_one_struct_and_without_beta() {
    assert_eq!(check_chain::<RootFamily>(), Ok(()));
    assert_eq!(check_chain::<CompositionFamily>(), Ok(()));
    assert_eq!(FORMAT_VERSION, FormatVersion::new(1, 21, 130));
    assert_eq!(RootFamily::FLOOR, FORMAT_VERSION);
    assert_eq!(CompositionFamily::FLOOR, FORMAT_VERSION);
    assert_eq!(RootFamily::RANGE_MODEL, RangeModel::ClosedToLatest);
    assert_eq!(CompositionFamily::RANGE_MODEL, RangeModel::ClosedToLatest);
    assert_eq!(RootFamily::CEILING, None);
    assert_eq!(CompositionFamily::CEILING, None);
    let beta = [
        <<RootFamily as Family>::Any as VersionChain<RootFamily>>::HAS_BETA,
        <<CompositionFamily as Family>::Any as VersionChain<CompositionFamily>>::HAS_BETA,
    ];
    assert_eq!(beta, [false, false]);
    assert_eq!(RootFamily::PAYLOAD_KEY, "minecraft:ui-root");
    assert_eq!(CompositionFamily::PAYLOAD_KEY, "minecraft:ui-composition");
    assert_eq!(RootFamily::DIRECTORIES, ["ddui/root"]);
    assert_eq!(CompositionFamily::DIRECTORIES, ["ddui/composition"]);
}

/// Exactly 1.21.130: the ceiling is the struct's version, not the engine's.
#[test]
fn the_one_struct_covers_1_21_130_exactly() {
    let version = |version, prerelease| FormatVersionInput {
        version: Some(version),
        beta_literal: false,
        prerelease,
    };
    for select in [
        |w: &FormatVersionInput| select::<RootFamily>(w),
        |w: &FormatVersionInput| select::<CompositionFamily>(w),
    ] {
        assert_eq!(
            select(&version(FORMAT_VERSION, false)),
            Selection::Struct(0)
        );
        assert_eq!(
            select(&version(FormatVersion::new(1, 21, 129), false)),
            Selection::BelowFloor
        );
        // A pre-release of the version sorts below it.
        assert_eq!(
            select(&version(FORMAT_VERSION, true)),
            Selection::BelowFloor
        );
        assert_eq!(
            select(&version(FormatVersion::new(1, 21, 131), false)),
            Selection::Unsupported
        );
        assert_eq!(
            select(&version(ENGINE_VERSION, false)),
            Selection::Unsupported
        );
    }
    // The model, not the constants, closes the range: the same chain under the default model
    // would reach the engine.
    let open = select_versions(
        &[FORMAT_VERSION],
        FORMAT_VERSION,
        RangeModel::ClosedToEngine,
        None,
        &version(ENGINE_VERSION, false),
    );
    assert_eq!(open, Selection::Struct(0));
}
