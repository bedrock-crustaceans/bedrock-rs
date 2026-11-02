//! The family notes not pinned beside their fixture row (n05, n06, n08, n09 and n10 are
//! in `stack.rs` and `rows.rs`).

use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_family_structure::Structure;

use super::common::bytes;

/// A valid structure with `neighbor_aware_block_upgrade_version` `value`.
fn with_version(value: i32) -> Vec<u8> {
    bytes(&format!(
        "{{format_version: 1, neighbor_aware_block_upgrade_version: {value}, size: [1, 1, 1], structure_world_origin: [0, 0, 0], structure: {{block_indices: [[], []], entities: [], palette: {{}}}}}}"
    ))
}

#[test]
fn the_neighbor_aware_bound_is_1_for_every_engine_deviation() {
    // 1.26.20 accepts 0..=1 (`WallBlock`); the 1.26.51 headers know up to 4
    // (`ThinFenceAndTripWireBlock`) but not the release that raised the bound, so ours keeps 1
    // for every target ().
    for engine in [FormatVersion::new(1, 26, 20), FormatVersion::new(1, 26, 50)] {
        let mut cx = LoadContext::detached();
        cx.engine_version = engine;
        for value in [0, 1] {
            let path = bedrock_addon_core::pack_identity::PackPath::from_static(
                "structures/demo/x.mcstructure",
            );
            let loaded = <Structure as bedrock_addon_core::document::Content>::parse(
                &path,
                &with_version(value),
                &cx,
            );
            assert!(loaded.is_ok(), "{engine}: {value}");
        }
        for value in [2, 3, 4] {
            let path = bedrock_addon_core::pack_identity::PackPath::from_static(
                "structures/demo/x.mcstructure",
            );
            let rejected = <Structure as bedrock_addon_core::document::Content>::parse(
                &path,
                &with_version(value),
                &cx,
            )
            .expect_err("rejected");
            assert_eq!(
                rejected.diagnostics.iter().next().unwrap().code.to_string(),
                format!(
                    "\"neighbor_aware_block_upgrade_version\" field has an invalid value of {value}. Valid values are 0 and up to 1."
                )
            );
        }
    }
}
