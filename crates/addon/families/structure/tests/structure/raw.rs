//! Raw compounds (entities, palette entries, block entity data) and unknown tags are opaque to
//! the writer: what the value holds is what is written, also in a structure that keeps its
//! file's layout. Only the format's own `getInt` positions keep a
//! file tag that reads as the value.

use bedrock_addon_family_structure::Structure;
use nbtx::Value;

use super::common::{bytes, fixture, loads};

/// A file in a layout the writer does not produce itself (`structure` last), so its tree is
/// kept: an entity, a palette entry with states, a block entity, a tick with an unknown tag and
/// an unknown tag at every level.
const KEPT: &str = r#"{
    format_version: 1,
    size: [1, 1, 1],
    structure_world_origin: [0, 0, 0],
    zz: "tool",
    structure: {
        block_indices: [[0], [-1]],
        entities: [{Health: 20s, identifier: "minecraft:pig"}],
        palette: {
            default: {
                block_palette: [{name: "minecraft:trapdoor", states: {open_bit: 1b}, version: 18168865}],
                block_position_data: {
                    "0": {
                        block_entity_data: {id: "Chest", Items: 3s},
                        tick_queue_data: [{tick_delay: 4, note: "t"}],
                        tag: 7s
                    }
                },
                note: "palette"
            }
        },
        author: "demo"
    }
}"#;

/// The structure of [`KEPT`].
fn kept() -> Structure {
    loads(&bytes(KEPT)).into_value()
}

/// Writes `structure` and reads it back; the result must be `structure`.
#[track_caller]
fn round_trips(structure: &Structure) {
    let again = loads(&structure.to_bytes().expect("writes")).into_value();
    assert_eq!(again, *structure);
}

#[test]
fn the_fixture_itself_is_written_back_byte_for_byte() {
    assert_eq!(kept().to_bytes().unwrap(), bytes(KEPT));
}

#[test]
fn an_entity_tag_set_to_int_0_is_written_as_int_0() {
    let mut structure = kept();
    structure.structure.entities[0].insert("Health".into(), Value::Int(0));
    round_trips(&structure);
}

#[test]
fn a_palette_entry_state_set_to_int_0_is_written_as_int_0() {
    let mut structure = kept();
    let entry = &mut structure.structure.palette["default"].block_palette[0];
    let Some(Value::Compound(states)) = entry.get_mut(b"states".as_slice()) else {
        panic!("states");
    };
    states.insert("open_bit".into(), Value::Int(0));
    round_trips(&structure);
}

#[test]
fn a_block_entity_tag_set_to_int_0_is_written_as_int_0() {
    let mut structure = kept();
    let data = structure.structure.palette["default"]
        .block_position_data
        .get_mut(&0)
        .unwrap();
    data.block_entity_data
        .as_mut()
        .unwrap()
        .insert("Items".into(), Value::Int(0));
    round_trips(&structure);
}

#[test]
fn an_unknown_tag_set_to_int_0_is_written_as_int_0_at_every_level() {
    let mut structure = kept();
    structure.extra.insert("zz".into(), Value::Int(0));
    round_trips(&structure);
    let mut structure = kept();
    structure
        .structure
        .extra
        .insert("author".into(), Value::Int(0));
    round_trips(&structure);
    let mut structure = kept();
    structure.structure.palette["default"]
        .extra
        .insert("note".into(), Value::Int(0));
    round_trips(&structure);
    let mut structure = kept();
    let data = structure.structure.palette["default"]
        .block_position_data
        .get_mut(&0)
        .unwrap();
    data.extra.insert("tag".into(), Value::Int(0));
    data.tick_queue_data.as_mut().unwrap()[0]
        .extra
        .insert("note".into(), Value::Int(0));
    round_trips(&structure);
}

#[test]
fn a_tag_order_changed_inside_a_raw_compound_is_written_as_held() {
    let mut structure = kept();
    structure.structure.entities[0].swap_indices(0, 1);
    let written = structure.to_bytes().unwrap();
    let again = loads(&written).into_value();
    let names: Vec<String> = again.structure.entities[0]
        .keys()
        .map(|name| String::from_utf8_lossy(name).into_owned())
        .collect();
    assert_eq!(names, ["identifier", "Health"]);
    // The same for a palette entry and a block entity.
    let mut structure = kept();
    let palette = &mut structure.structure.palette["default"];
    palette.block_palette[0].swap_indices(0, 2);
    palette
        .block_position_data
        .get_mut(&0)
        .unwrap()
        .block_entity_data
        .as_mut()
        .unwrap()
        .swap_indices(0, 1);
    let again = loads(&structure.to_bytes().unwrap()).into_value();
    let palette = &again.structure.palette["default"];
    let names = |compound: &nbtx::Compound| -> Vec<String> {
        compound
            .keys()
            .map(|name| String::from_utf8_lossy(name).into_owned())
            .collect()
    };
    assert_eq!(
        names(&palette.block_palette[0]),
        ["version", "states", "name"]
    );
    assert_eq!(
        names(
            palette.block_position_data[&0]
                .block_entity_data
                .as_ref()
                .unwrap()
        ),
        ["Items", "id"]
    );
}

#[test]
fn any_edit_inside_a_raw_compound_of_a_capture_survives_the_round_trip() {
    // `tick_block_structure` keeps its tree (tags out of order). Every tag of its palette
    // entry, replaced by each of a spread of values, must come back as set.
    let original = loads(&fixture("capture_tick_block_structure")).into_value();
    let names: Vec<Vec<u8>> = original.structure.palette["default"].block_palette[0]
        .keys()
        .map(|name| name.to_vec())
        .collect();
    assert_eq!(names.len(), 3, "name, states, version");
    let replacements = [
        Value::Int(0),
        Value::Int(7),
        Value::Byte(0),
        Value::Short(0),
        Value::Long(0),
        Value::String("x".into()),
        Value::Compound(nbtx::Compound::new()),
        Value::IntArray(vec![0]),
    ];
    for name in &names {
        for replacement in &replacements {
            let mut structure = original.clone();
            structure.structure.palette["default"].block_palette[0]
                .insert(name.as_slice().into(), replacement.clone());
            let again = loads(&structure.to_bytes().expect("writes")).into_value();
            assert_eq!(
                again,
                structure,
                "{} := {replacement:?}",
                String::from_utf8_lossy(name)
            );
            // And the value as held is the value in the file, tag included.
            assert_eq!(
                again.structure.palette["default"].block_palette[0]
                    .get(name.as_slice())
                    .map(Value::discriminant),
                Some(replacement.discriminant())
            );
        }
    }
    // A new tag and a removed tag in the raw compound.
    let mut structure = original.clone();
    let entry = &mut structure.structure.palette["default"].block_palette[0];
    entry.insert("added".into(), Value::Int(0));
    entry.shift_remove(b"states".as_slice());
    round_trips(&structure);
}

#[test]
fn the_formats_own_integers_keep_a_file_tag_that_reads_as_their_value() {
    // The other side of the rule: `size`, `structure_world_origin`, the index layers and
    // `tick_delay` are read with `getInt`, so a tag of another type is kept while the value is
    // still the 0 it read as, and replaced once the value is set.
    let input = bytes(
        r#"{format_version: 1, size: [0s, 0s, 0s], structure_world_origin: [1b, 2b, 3b], structure: {block_indices: [[], []], entities: [], palette: {default: {block_palette: [], block_position_data: {"0": {tick_queue_data: [{tick_delay: "soon"}]}}}}}}"#,
    );
    let structure = loads(&input).into_value();
    assert_eq!(structure.structure_world_origin, [0, 0, 0]);
    assert_eq!(structure.to_bytes().unwrap(), input);
    let mut changed = structure.clone();
    changed.structure_world_origin = [0, 5, 0];
    changed.structure.palette["default"]
        .block_position_data
        .get_mut(&0)
        .unwrap()
        .tick_queue_data
        .as_mut()
        .unwrap()[0]
        .tick_delay = Some(9);
    round_trips(&changed);
}

#[test]
fn a_tag_removed_early_in_a_kept_layout_leaves_the_later_tags_in_file_order() {
    let mut structure = loads(&bytes(
        r#"{zz: 1b, structure_world_origin: [0, 0, 0], size: [0, 0, 0], format_version: 1, structure: {block_indices: [[], []], entities: [], palette: {}}}"#,
    ))
    .into_value();
    structure.extra.clear();
    let written = structure.to_bytes().unwrap();
    assert_eq!(
        written,
        bytes(
            r#"{structure_world_origin: [0, 0, 0], size: [0, 0, 0], format_version: 1, structure: {block_indices: [[], []], entities: [], palette: {}}}"#
        )
    );
}
