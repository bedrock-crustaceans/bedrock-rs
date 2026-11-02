//! The chain's ranges: one copy step between the two structs of 1.26.60.

use bedrock_addon_core::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{RangeModel, Selection, select_versions};

const V1_21_60: FormatVersion = FormatVersion::new(1, 21, 60);
const V1_26_50: FormatVersion = FormatVersion::new(1, 26, 50);

#[test]
fn the_two_structs_cover_1_21_60_to_1_26_50_and_1_26_50_to_the_engine() {
    let select = |version, prerelease| {
        select_versions(
            &[V1_21_60, V1_26_50],
            V1_21_60,
            RangeModel::ClosedToEngine,
            None,
            &FormatVersionInput {
                version: Some(version),
                beta_literal: false,
                prerelease,
            },
        )
    };
    assert_eq!(select(V1_21_60, false), Selection::Struct(0));
    assert_eq!(
        select(FormatVersion::new(1, 26, 49), false),
        Selection::Struct(0)
    );
    // A pre-release of 1.26.50 sorts below it.
    assert_eq!(select(V1_26_50, true), Selection::Struct(0));
    assert_eq!(select(V1_26_50, false), Selection::Struct(1));
    assert_eq!(select(ENGINE_VERSION, false), Selection::Struct(1));
    assert_eq!(
        select(FormatVersion::new(1, 21, 59), false),
        Selection::BelowFloor
    );
    assert_eq!(select(V1_21_60, true), Selection::BelowFloor);
    let above = FormatVersion::new(
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch + 1,
    );
    assert_eq!(select(above, false), Selection::Unsupported);
}
