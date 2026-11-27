//! Struct selection: every row of the `RangeModel` table, the gaps that are
//! `UnsupportedVersion`, below the floor and above the top, and the `"beta"` literal against
//! `-beta.N` pre-releases.

use bedrock_addon_core::version::{
    ENGINE_VERSION, FormatVersion, FormatVersionInput, VersionRange,
};
use bedrock_addon_core::versioning::{RangeModel, Selection, ranges, select_versions};

const V100: FormatVersion = FormatVersion::new(1, 0, 0);
const V110: FormatVersion = FormatVersion::new(1, 1, 0);
const V120: FormatVersion = FormatVersion::new(1, 2, 0);
const CHAIN: [FormatVersion; 3] = [V100, V110, V120];

/// What `format_version` text selects in the three-struct chain under `model`.
fn pick(model: RangeModel, text: &str) -> Selection {
    pick_with(model, None, text)
}

fn pick_with(model: RangeModel, ceiling: Option<FormatVersion>, text: &str) -> Selection {
    let input = FormatVersionInput::parse(text).expect("a version");
    select_versions(&CHAIN, V100, model, ceiling, &input)
}

#[test]
fn closed_to_engine_ranges_run_to_the_engine_version() {
    assert_eq!(
        ranges(&CHAIN, V100, RangeModel::ClosedToEngine, None),
        [
            VersionRange::right_open(V100, V110),
            VersionRange::right_open(V110, V120),
            VersionRange::closed(V120, ENGINE_VERSION),
        ]
    );
    let m = RangeModel::ClosedToEngine;
    assert_eq!(pick(m, "1.0.0"), Selection::Struct(0));
    assert_eq!(pick(m, "1.0.9"), Selection::Struct(0));
    assert_eq!(pick(m, "1.1.0"), Selection::Struct(1));
    assert_eq!(pick(m, "1.2.0"), Selection::Struct(2));
    assert_eq!(pick(m, "1.26.55"), Selection::Struct(2));
    assert_eq!(pick(m, "1.26.60"), Selection::Struct(2));
    assert_eq!(
        pick(m, "1.26.61"),
        Selection::Unsupported,
        "above the engine"
    );
    assert_eq!(pick(m, "0.9.0"), Selection::BelowFloor);
}

#[test]
fn closed_to_latest_takes_only_the_newest_version_itself() {
    assert_eq!(
        ranges(&CHAIN, V100, RangeModel::ClosedToLatest, None),
        [
            VersionRange::right_open(V100, V110),
            VersionRange::right_open(V110, V120),
            VersionRange::exact(V120),
        ]
    );
    let m = RangeModel::ClosedToLatest;
    assert_eq!(pick(m, "1.1.9"), Selection::Struct(1));
    assert_eq!(pick(m, "1.2.0"), Selection::Struct(2));
    assert_eq!(pick(m, "1.2.1"), Selection::Unsupported, "above V_last");
    assert_eq!(pick(m, "1.26.50"), Selection::Unsupported);
}

/// `VersionRange::contains_input` could put `V_last-beta.n` in the inner range
/// `[V_{last-1}, V_last)`; under `ClosedToLatest` no range covers it, so the selection is
/// unsupported.
#[test]
fn closed_to_latest_rejects_a_prerelease_of_the_newest_version() {
    let inner = VersionRange::right_open(V110, V120);
    let input = FormatVersionInput::parse("1.2.0-beta.3").unwrap();
    assert!(
        inner.contains_input(&input),
        "the range type alone would take it"
    );
    assert_eq!(
        pick(RangeModel::ClosedToLatest, "1.2.0-beta.3"),
        Selection::Unsupported
    );
    // One step lower is an ordinary pre-release.
    assert_eq!(
        pick(RangeModel::ClosedToLatest, "1.1.0-beta.3"),
        Selection::Struct(0)
    );
}

#[test]
fn exact_only_takes_each_struct_version_and_nothing_between() {
    assert_eq!(
        ranges(&CHAIN, V100, RangeModel::ExactOnly, None),
        [
            VersionRange::exact(V100),
            VersionRange::exact(V110),
            VersionRange::exact(V120)
        ]
    );
    let m = RangeModel::ExactOnly;
    assert_eq!(pick(m, "1.0.0"), Selection::Struct(0));
    assert_eq!(pick(m, "1.1.0"), Selection::Struct(1));
    assert_eq!(pick(m, "1.2.0"), Selection::Struct(2));
    assert_eq!(
        pick(m, "1.0.5"),
        Selection::Unsupported,
        "a gap between two structs"
    );
    assert_eq!(pick(m, "1.2.1"), Selection::Unsupported, "above V_last");
    assert_eq!(
        pick(m, "1.1.0-beta.1"),
        Selection::Unsupported,
        "no half-open range"
    );
    assert_eq!(pick(m, "0.1.0"), Selection::BelowFloor);
}

#[test]
fn half_open_to_engine_rejects_the_engine_version_itself() {
    assert_eq!(
        ranges(&CHAIN, V100, RangeModel::HalfOpenToEngine, None)[2],
        VersionRange::right_open(V120, ENGINE_VERSION)
    );
    let m = RangeModel::HalfOpenToEngine;
    assert_eq!(pick(m, "1.26.59"), Selection::Struct(2));
    assert_eq!(pick(m, "1.26.60"), Selection::Unsupported);
    assert_eq!(
        pick(m, "1.26.60-beta.1"),
        Selection::Struct(2),
        "below the engine"
    );
}

#[test]
fn an_explicit_ceiling_ends_the_last_range() {
    let ceiling = Some(FormatVersion::new(1, 21, 40));
    assert_eq!(
        pick_with(RangeModel::ClosedToEngine, ceiling, "1.21.40"),
        Selection::Struct(2)
    );
    assert_eq!(
        pick_with(RangeModel::ClosedToEngine, ceiling, "1.21.41"),
        Selection::Unsupported
    );
}

#[test]
fn the_first_range_starts_at_a_floor_below_the_first_struct() {
    let floor = FormatVersion::new(0, 5, 0);
    let input = FormatVersionInput::parse("0.7.0").unwrap();
    assert_eq!(
        select_versions(&CHAIN, floor, RangeModel::ClosedToEngine, None, &input),
        Selection::Struct(0)
    );
    assert_eq!(
        ranges(&CHAIN, floor, RangeModel::ClosedToEngine, None)[0],
        VersionRange::right_open(floor, V110)
    );
}

/// `1.21.60-beta.3` sorts before `1.21.60`, so it selects the struct below.
#[test]
fn a_prerelease_selects_the_struct_below_its_release() {
    for model in [
        RangeModel::ClosedToEngine,
        RangeModel::ClosedToLatest,
        RangeModel::HalfOpenToEngine,
    ] {
        assert_eq!(
            pick(model, "1.1.0-beta.3"),
            Selection::Struct(0),
            "{model:?}"
        );
        assert_eq!(
            pick(model, "1.1.5-beta.3"),
            Selection::Struct(1),
            "{model:?}"
        );
    }
}

/// `FLOOR-beta.n` sorts below the floor in every model.
#[test]
fn a_prerelease_of_the_floor_is_below_it_in_every_model() {
    for model in [
        RangeModel::ClosedToEngine,
        RangeModel::ClosedToLatest,
        RangeModel::ExactOnly,
        RangeModel::HalfOpenToEngine,
    ] {
        assert_eq!(
            pick(model, "1.0.0-beta.1"),
            Selection::BelowFloor,
            "{model:?}"
        );
    }
}

/// The literal `"beta"` is no stable version: it selects no stable struct.
#[test]
fn the_beta_literal_selects_no_stable_struct() {
    let beta = FormatVersionInput::parse("beta").unwrap();
    assert!(beta.beta_literal && beta.version.is_none());
    assert_eq!(
        select_versions(&CHAIN, V100, RangeModel::ClosedToEngine, None, &beta),
        Selection::Unsupported
    );
    let tagged = FormatVersionInput::parse("1.1.0-beta.2").unwrap();
    assert!(!tagged.beta_literal && tagged.prerelease);
}

#[test]
fn ranges_of_an_empty_chain_are_empty() {
    assert!(ranges(&[], V100, RangeModel::ClosedToEngine, None).is_empty());
}
