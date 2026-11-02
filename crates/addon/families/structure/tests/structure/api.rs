//! The inherent helpers: construction, positions and validation.

use bedrock_addon_core::load::LoadContext;
use bedrock_addon_family_structure::{Contents, Palette, Structure};
use indexmap::IndexMap;

use super::common::{fixture, loads};
use super::rows::block;

#[test]
fn new_is_a_valid_box_that_keeps_every_world_block() {
    let structure = Structure::new([2, 3, 4]);
    assert_eq!(structure.format_version, Structure::FORMAT_VERSION);
    assert_eq!(structure.neighbor_aware_block_upgrade_version, None);
    assert_eq!(structure.size, [2, 3, 4]);
    assert_eq!(structure.structure_world_origin, [0, 0, 0]);
    assert_eq!(
        structure.structure.block_indices,
        [vec![-1; 24], vec![-1; 24]]
    );
    assert!(structure.structure.palette.is_empty());
    assert!(structure.validate(&LoadContext::detached()).is_empty());
    // Written and read back, it is the same structure.
    let again = loads(&structure.to_bytes().unwrap()).into_value();
    assert_eq!(again, structure);
}

#[test]
fn a_struct_literal_builds_a_structure_from_outside_the_crate() {
    let structure = Structure {
        size: [1, 1, 1],
        structure: Contents {
            block_indices: [vec![0], vec![-1]],
            palette: IndexMap::from_iter([(
                Structure::DEFAULT_PALETTE.to_owned(),
                Palette {
                    block_palette: vec![block("minecraft:stone", 18_168_865)],
                    ..Palette::default()
                },
            )]),
            ..Contents::default()
        },
        structure_world_origin: [12, 64, -7],
        ..Structure::new([0, 0, 0])
    };
    // s01 is this structure in vanilla's layout.
    assert_eq!(structure.to_bytes().unwrap(), fixture("s01_vanilla_1x1x1"));
}

#[test]
fn validate_runs_the_load_rules() {
    let mut structure = Structure::new([1, 1, 1]);
    structure.size = [1, -1, 1];
    let findings = structure.validate(&LoadContext::detached());
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings.iter().next().unwrap().code.to_string(),
        "The \"size\" field shouldn't have any negative values."
    );
}

#[test]
fn index_runs_z_fastest_then_y_then_x() {
    let structure = Structure::new([2, 3, 4]);
    assert_eq!(structure.index([0, 0, 0]), Some(0));
    assert_eq!(structure.index([0, 0, 1]), Some(1));
    assert_eq!(structure.index([0, 1, 0]), Some(4));
    assert_eq!(structure.index([1, 0, 0]), Some(12));
    assert_eq!(structure.index([1, 2, 3]), Some(23));
    assert_eq!(structure.index([2, 0, 0]), None);
    assert_eq!(structure.index([0, -1, 0]), None);
    // The jigsaw capture is 1x3x1: its three indices run up the y axis.
    let jigsaw = loads(&fixture("capture_processors_test"));
    assert_eq!(jigsaw.index([0, 2, 0]), Some(2));
}
