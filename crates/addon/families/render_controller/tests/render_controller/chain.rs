//! The one struct's range: the chain has no step.

use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{RangeModel, Selection, select_versions};

const V1_8_0: FormatVersion = FormatVersion::new(1, 8, 0);

/// The chain is contiguous by construction (one struct); the range model that describes the
/// loader is "from the floor up to a ceiling that is not the engine": 65535.0.0.
#[test]
fn the_one_struct_covers_1_8_0_to_the_schema_maximum() {
    let select = |version, prerelease| {
        select_versions(
            &[V1_8_0],
            V1_8_0,
            RangeModel::ClosedToEngine,
            Some(FormatVersion::new(65535, 0, 0)),
            &FormatVersionInput {
                version: Some(version),
                beta_literal: false,
                prerelease,
            },
        )
    };
    assert_eq!(select(V1_8_0, false), Selection::Struct(0));
    assert_eq!(
        select(FormatVersion::new(1, 10, 0), false),
        Selection::Struct(0)
    );
    assert_eq!(
        select(FormatVersion::new(65535, 0, 0), false),
        Selection::Struct(0)
    );
    assert_eq!(
        select(FormatVersion::new(1, 7, 99), false),
        Selection::BelowFloor
    );
    assert_eq!(select(V1_8_0, true), Selection::BelowFloor);
    assert_eq!(
        select(FormatVersion::new(65535, 0, 1), false),
        Selection::Unsupported
    );
}
