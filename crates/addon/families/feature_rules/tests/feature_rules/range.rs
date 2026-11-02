//! The feature-rule chain is one struct: the range of format versions it covers.

use bedrock_addon_core::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{RangeModel, Selection, select_versions};

#[test]
fn the_one_struct_covers_1_12_0_to_the_engine() {
    let select = |version| {
        select_versions(
            &[FormatVersion::new(1, 12, 0)],
            FormatVersion::new(1, 12, 0),
            RangeModel::ClosedToEngine,
            None,
            &FormatVersionInput {
                version: Some(version),
                beta_literal: false,
                prerelease: false,
            },
        )
    };
    assert_eq!(select(FormatVersion::new(1, 12, 0)), Selection::Struct(0));
    assert_eq!(select(ENGINE_VERSION), Selection::Struct(0));
    assert_eq!(select(FormatVersion::new(1, 11, 0)), Selection::BelowFloor);
    assert_eq!(select(FormatVersion::new(1, 27, 0)), Selection::Unsupported);
}
