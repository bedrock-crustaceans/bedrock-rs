//! The round-trip level of `.mcstructure`, **Exact**: an unchanged structure is
//! written back byte for byte, vanilla's own files and the accepted fixtures alike; a changed
//! one keeps the file's layout where the values still match it.

use bedrock_addon_core::document::Content;
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_family_structure::Structure;
use nbtx::Value;

use super::common::{bytes, fixture, loads};

/// Vanilla captures: `functional_test_root` and `processors_test` (GDK 1.26.20.04 dev test
/// assets), `tick_block_structure` (same, tags out of `std::map` order and an empty entity list
/// of Compound tags), `sulfur_spring_small_1` and the bastion `hoglin` (GDK retail 1.26.45.01
/// behaviour packs `vanilla_1.26.30` and `vanilla`).
const CAPTURES: [&str; 5] = [
    "capture_functional_test_root",
    "capture_processors_test",
    "capture_tick_block_structure",
    "capture_sulfur_spring_small_1",
    "capture_hoglin",
];

/// The accepted file fixtures.
const ACCEPTED: [&str; 11] = [
    "s01_vanilla_1x1x1",
    "s08_nabu_absent",
    "s12_origin_negative",
    "s18_indices_empty",
    "s19_indices_non_int",
    "s24_palette_empty",
    "s31_old_block_version",
    "s32_unknown_block",
    "s33_extra_keys",
    "n09_negative_indices",
    "n10_index_past_palette",
];

/// Loads `input`, writes it, and requires the same bytes and an equal value on reading them.
#[track_caller]
fn exact(name: &str, input: &[u8]) -> Structure {
    let structure = loads(input).into_value();
    let written = structure.to_bytes().expect("writes");
    assert!(written == input, "{name}: the written file differs");
    assert_eq!(loads(&written).into_value(), structure, "{name}");
    structure
}

#[test]
fn vanilla_captures_are_written_back_byte_for_byte() {
    for name in CAPTURES {
        exact(name, &fixture(name));
    }
}

#[test]
fn accepted_fixtures_are_written_back_byte_for_byte() {
    for name in ACCEPTED {
        exact(name, &fixture(name));
    }
}

#[test]
fn the_captures_hold_what_vanilla_wrote() {
    let jigsaw = exact("processors_test", &fixture("capture_processors_test"));
    assert_eq!(jigsaw.size, [1, 3, 1]);
    assert_eq!(
        jigsaw.structure.block_indices,
        [vec![0, 1, 2], vec![-1, -1, -1]]
    );
    let default = &jigsaw.structure.palette["default"];
    assert_eq!(default.block_palette.len(), 3);
    let entity = default.block_position_data[&0]
        .block_entity_data
        .as_ref()
        .unwrap();
    assert_eq!(
        entity.get(b"id".as_slice()),
        Some(&Value::String("JigsawBlock".into()))
    );
    let hoglin = exact("hoglin", &fixture("capture_hoglin"));
    assert_eq!(hoglin.structure.entities.len(), 1);
    assert_eq!(
        hoglin.structure.entities[0].get(b"identifier".as_slice()),
        Some(&Value::String("minecraft:hoglin".into()))
    );
}

#[test]
fn a_structure_built_in_code_is_written_in_vanillas_layout() {
    // s01 is in the layout `/structure save` writes: tags by name, empty lists of End.
    let loaded = loads(&fixture("s01_vanilla_1x1x1")).into_value();
    let built = Structure {
        layout: Default::default(),
        ..loaded.clone()
    };
    assert_eq!(built.to_bytes().unwrap(), fixture("s01_vanilla_1x1x1"));
}

#[test]
fn a_changed_structure_keeps_the_files_layout_elsewhere() {
    // `tick_block_structure` has `structure_world_origin` before `structure` and an empty
    // entity list of Compound tags; both survive a change of the size and an index.
    let mut structure = loads(&fixture("capture_tick_block_structure")).into_value();
    structure.structure_world_origin = [4, 5, 6];
    structure.structure.block_indices[1][0] = 0;
    let written = structure.to_bytes().unwrap();
    let again = loads(&written).into_value();
    assert_eq!(again, structure);
    let expected = bytes(
        r#"{format_version: 1, size: [1, 1, 1], structure_world_origin: [4, 5, 6], structure: {block_indices: [[0], [0]], entities: [], palette: {default: {block_palette: [{name: "structuretest:tick_block", states: {}, version: 18100737}], block_position_data: {}}}}}"#,
    );
    // Same length: only values changed; the entity list keeps its Compound tag (byte 10).
    assert_eq!(written.len(), expected.len());
    let entities = written
        .windows(8)
        .position(|w| w == b"entities")
        .expect("entities");
    assert_eq!(written[entities + 8], 10, "the empty list keeps its tag");
}

#[test]
fn integer_widths_survive_while_their_value_is_unchanged() {
    // s19 writes its indices as Shorts (read as 0); unchanged, they stay Shorts; set to 1, the
    // layer becomes Ints.
    let input = fixture("s19_indices_non_int");
    let mut structure = exact("s19", &input);
    structure.structure.block_indices[0][0] = 1;
    let again = loads(&structure.to_bytes().unwrap()).into_value();
    assert_eq!(again.structure.block_indices, [vec![1], vec![0]]);
}

#[test]
fn a_root_name_and_trailing_bytes_are_kept() {
    let mut input = nbtx::to_le_bytes(&nbtx::Named::new(
        "named",
        super::common::tree(
            r#"{format_version: 1, size: [1, 1, 1], structure_world_origin: [0, 0, 0], structure: {block_indices: [[], []], entities: [], palette: {}}}"#,
        ),
    ))
    .unwrap();
    input.extend_from_slice(b"\x00junk");
    exact("named", &input);
}

#[test]
fn write_json_writes_the_nbt_file_and_to_json_refuses() {
    let structure = loads(&fixture("s01_vanilla_1x1x1")).into_value();
    let mut out = Vec::new();
    structure
        .write_json(&mut out, WriteStyle::default())
        .unwrap();
    assert_eq!(out, fixture("s01_vanilla_1x1x1"));
    assert!(matches!(
        structure.to_json(),
        Err(EncodeError::NotRepresentable { .. })
    ));
}
