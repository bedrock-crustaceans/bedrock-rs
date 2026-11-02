//! The chain is one struct: its range.

use bedrock_addon_core::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{RangeModel, Selection, select_versions};

const V1_21_110: FormatVersion = FormatVersion::new(1, 21, 110);

#[test]
fn the_one_struct_covers_1_21_110_to_the_engine() {
    let select = |version, prerelease| {
        select_versions(
            &[V1_21_110],
            V1_21_110,
            RangeModel::ClosedToEngine,
            None,
            &FormatVersionInput {
                version: Some(version),
                beta_literal: false,
                prerelease,
            },
        )
    };
    assert_eq!(select(V1_21_110, false), Selection::Struct(0));
    assert_eq!(select(ENGINE_VERSION, false), Selection::Struct(0));
    assert_eq!(
        select(FormatVersion::new(1, 21, 109), false),
        Selection::BelowFloor
    );
    assert_eq!(select(V1_21_110, true), Selection::BelowFloor);
    let above = FormatVersion::new(
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch + 1,
    );
    assert_eq!(select(above, false), Selection::Unsupported);
}
