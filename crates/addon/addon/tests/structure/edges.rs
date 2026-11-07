//! Both ends of every range and both halves of every compound condition of the structure
//! rules, the writer and the lookups, beyond what the fixture rows pin.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::Document;
use bedrock_addon::families::structure::{Palette, Structure};
use bedrock_addon::json::EncodeError;
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::pack::{Behavior, PackStack};
use bedrock_addon::storage::PackPath;
use nbtx::{Value, ValueList};

use super::common::{bytes, fails_with, fixture, loads, tree};
use super::pack_common::bp;

/// A 1x1x1 structure with the root members `root` (after `format_version`), the `structure`
/// members `body`.
fn doc(root: &str, body: &str) -> String {
    format!("{{format_version: 1, {root}, structure: {{{body}}}}}")
}

const ROOT: &str = "size: [1, 1, 1], structure_world_origin: [0, 0, 0]";
const BODY: &str = "block_indices: [[0], [-1]], entities: [], palette: {}";

/// A 1x1x1 structure whose `default` palette has the block position data `positions`.
fn positions(positions: &str) -> Vec<u8> {
    bytes(&doc(
        ROOT,
        &format!(
            "block_indices: [[0], [-1]], entities: [], palette: {{default: {{block_palette: [{{name: \"minecraft:stone\"}}], block_position_data: {{{positions}}}}}}}"
        ),
    ))
}

/// The info findings of `document`, as tag paths.
fn notes(document: &Document<Structure>) -> Vec<String> {
    document
        .diagnostics()
        .iter()
        .filter(|f| f.severity == Severity::Info)
        .map(|f| f.location.json_path.to_string())
        .collect()
}

// ---- load rules ---------------------------------------------------------------------------

#[test]
fn a_negative_size_fails_in_every_component_and_zero_does_not() {
    for size in [
        "[-1, 1, 1]",
        "[1, -1, 1]",
        "[1, 1, -1]",
        "[-2147483648, 0, 0]",
    ] {
        fails_with(
            &bytes(&doc(
                &format!("size: {size}, structure_world_origin: [0, 0, 0]"),
                "block_indices: [[], []], entities: [], palette: {}",
            )),
            "structure/negative",
            "The \"size\" field shouldn't have any negative values.",
        );
    }
    let empty = loads(&bytes(&doc(
        "size: [0, 0, 0], structure_world_origin: [0, 0, 0]",
        "block_indices: [[], []], entities: [], palette: {}",
    )));
    assert_eq!(empty.size, [0, 0, 0]);
}

#[test]
fn a_world_origin_of_4_ints_expects_3_elements() {
    fails_with(
        &bytes(&doc(
            "size: [1, 1, 1], structure_world_origin: [1, 2, 3, 4]",
            BODY,
        )),
        "structure/three_elements",
        "The \"structure_world_origin\" field expects 3 elements.",
    );
}

#[test]
fn format_versions_far_from_1_are_larger_or_smaller() {
    fails_with(
        &bytes(&format!(
            "{{format_version: 2147483647, {ROOT}, structure: {{{BODY}}}}}"
        )),
        "structure/larger_version",
        "\"format_version\" field has a value larger than the current version.  We expect a format version with a value at most 1 and got 2147483647.",
    );
    fails_with(
        &bytes(&format!(
            "{{format_version: -2147483648, {ROOT}, structure: {{{BODY}}}}}"
        )),
        "structure/smaller_version",
        "\"format_version\" field has a value smaller than the current version.  We expect a format version with a value at most 1 and got -2147483648.",
    );
}

#[test]
fn block_indices_with_no_layers_reports_0_arrays() {
    fails_with(
        &bytes(&doc(ROOT, "block_indices: [], entities: [], palette: {}")),
        "structure/two_arrays",
        "The \"block_indices\" field should be an array with 2 arrays and instead we have 0 arrays.",
    );
}

#[test]
fn the_shorter_layer_may_be_either_one() {
    for layers in ["[[0], []]", "[[], [-1]]", "[[0, 0], [-1]]"] {
        fails_with(
            &bytes(&doc(
                ROOT,
                &format!("block_indices: {layers}, entities: [], palette: {{}}"),
            )),
            "structure/same_size",
            "The \"block_indices\" field's arrays need to both be the same size.",
        );
    }
}

#[test]
fn an_index_count_above_the_volume_fails_too() {
    fails_with(
        &bytes(&doc(
            ROOT,
            "block_indices: [[0, 0], [-1, -1]], entities: [], palette: {}",
        )),
        "structure/volume",
        "The \"block_indices\" field should have as many elements as defined by the \"size\" field.",
    );
}

#[test]
fn an_empty_list_of_any_tag_holds_no_compounds() {
    // SNBT cannot spell an empty list of Int tags; the tree is built by hand.
    let Value::Compound(mut root) = tree(&doc(
        ROOT,
        "block_indices: [[0], [-1]], entities: [], palette: {default: {block_palette: [], block_position_data: {}}}",
    )) else {
        panic!("a compound");
    };
    let Some(Value::Compound(body)) = root.get_mut(b"structure".as_slice()) else {
        panic!("structure");
    };
    body.insert("entities".into(), Value::List(ValueList::Int(Vec::new())));
    let Some(Value::Compound(palettes)) = body.get_mut(b"palette".as_slice()) else {
        panic!("palette");
    };
    let Some(Value::Compound(default)) = palettes.get_mut(b"default".as_slice()) else {
        panic!("default");
    };
    default.insert(
        "block_palette".into(),
        Value::List(ValueList::String(Vec::new())),
    );
    let input = nbtx::to_le_bytes(&Value::Compound(root)).unwrap();
    let document = loads(&input);
    assert!(document.structure.entities.is_empty());
    assert!(
        document.structure.palette["default"]
            .block_palette
            .is_empty()
    );
    // And the odd tags are written back.
    assert_eq!(document.to_bytes().unwrap(), input);
}

#[test]
fn block_position_keys_are_whole_decimal_i32s() {
    // The key as Rust reads an `i32` (a sign allowed, nothing else around the digits).
    for (key, index) in [
        ("\"0\"", 0),
        ("\"-1\"", -1),
        ("\"+5\"", 5),
        ("\"007\"", 7),
        ("\"2147483647\"", i32::MAX),
        ("\"-2147483648\"", i32::MIN),
    ] {
        let document = loads(&positions(&format!("{key}: {{}}")));
        let keys: Vec<i32> = document.structure.palette["default"]
            .block_position_data
            .keys()
            .copied()
            .collect();
        assert_eq!(keys, [index], "{key}");
    }
    for key in [
        "\"\"",
        "\"1.5\"",
        "\" 1\"",
        "\"1 \"",
        "\"0x1\"",
        "\"2147483648\"",
        "\"-2147483649\"",
        "\"1a\"",
    ] {
        let text = key.trim_matches('"');
        fails_with(
            &positions(&format!("{key}: {{}}")),
            "structure/not_a_number",
            &format!(
                "Expected a string formatted as a number when parsing contents of the \"block_position_data\" field, but got \"{text}\""
            ),
        );
    }
}

#[test]
fn two_spellings_of_one_block_position_keep_the_later_in_name_order() {
    // Vanilla walks its sorted map: "07" before "7"; the later entry replaces the earlier.
    let document = loads(&positions(
        r#""7": {tick_queue_data: [{tick_delay: 2}]}, "07": {tick_queue_data: [{tick_delay: 1}]}"#,
    ));
    let data = &document.structure.palette["default"].block_position_data;
    assert_eq!(data.len(), 1);
    assert_eq!(
        data[&7].tick_queue_data.as_ref().unwrap()[0].tick_delay(),
        2
    );
}

#[test]
fn the_first_bad_palette_by_name_is_the_one_reported_and_file_order_is_kept() {
    // `b` is not a compound, `a` lacks `block_palette`: vanilla's sorted map reaches `a` first.
    let finding = fails_with(
        &bytes(&doc(
            ROOT,
            "block_indices: [[0], [-1]], entities: [], palette: {b: 1, a: {block_position_data: {}}}",
        )),
        "structure/required",
        "\"block_palette\" field, a required field, is missing from the structure.",
    );
    assert_eq!(
        finding.location.json_path.to_string(),
        "structure/palette/a/block_palette"
    );
    let document = loads(&bytes(&doc(
        ROOT,
        "block_indices: [[0], [-1]], entities: [], palette: {zeta: {block_palette: [], block_position_data: {}}, alpha: {block_palette: [], block_position_data: {}}}",
    )));
    let names: Vec<&str> = document
        .structure
        .palette
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(names, ["zeta", "alpha"]);
}

#[test]
fn a_block_entity_is_read_before_the_tick_queue() {
    fails_with(
        &positions(r#""0": {tick_queue_data: 1, block_entity_data: 1}"#),
        "structure/compound_field",
        "Expected a compound tag for the \"block_entity_data\" field.",
    );
    let document = loads(&positions(
        r#""0": {block_entity_data: {id: "Chest"}, tick_queue_data: [], other: 1b}"#,
    ));
    let data = &document.structure.palette["default"].block_position_data[&0];
    assert_eq!(data.block_entity_data.as_ref().unwrap().len(), 1);
    assert_eq!(data.tick_queue_data.as_deref(), Some(&[][..]));
    assert_eq!(data.extra.len(), 1);
}

// ---- our index notes ----------------------------------------------------------------------

/// A 1x1x2 structure whose layers are `layers` and whose `default` palette has two entries.
fn indexed(layers: &str) -> Document<Structure> {
    loads(&bytes(&doc(
        "size: [1, 1, 2], structure_world_origin: [0, 0, 0]",
        &format!(
            "block_indices: {layers}, entities: [], palette: {{default: {{block_palette: [{{name: \"minecraft:stone\"}}, {{name: \"minecraft:dirt\"}}], block_position_data: {{}}}}}}"
        ),
    )))
}

#[test]
fn indices_from_minus_1_to_the_last_palette_entry_are_not_noted() {
    assert!(notes(&indexed("[[-1, 0], [1, -1]]")).is_empty());
}

#[test]
fn the_first_index_outside_the_palette_is_noted_per_layer() {
    // The palette length itself is outside; -2 is outside.
    assert_eq!(
        notes(&indexed("[[0, 2], [-1, -1]]")),
        ["structure/block_indices/0/1"]
    );
    assert_eq!(
        notes(&indexed("[[-2, 0], [-1, -1]]")),
        ["structure/block_indices/0/0"]
    );
    // Layer 1 is checked too, and each layer notes only its first.
    assert_eq!(
        notes(&indexed("[[9, 9], [-1, 7]]")),
        ["structure/block_indices/0/0", "structure/block_indices/1/1"]
    );
}

#[test]
fn only_the_default_palette_is_measured() {
    let document = loads(&bytes(&doc(
        ROOT,
        "block_indices: [[3], [-1]], entities: [], palette: {other: {block_palette: [], block_position_data: {}}}",
    )));
    assert!(notes(&document).is_empty());
}

// ---- the writer ---------------------------------------------------------------------------

#[test]
fn a_list_changed_in_one_place_falls_back_to_ints() {
    // Layers written as Shorts (read as 0): one index set, so the layer cannot stay Shorts.
    let mut structure = loads(&bytes(&doc(
        "size: [1, 1, 2], structure_world_origin: [0, 0, 0]",
        "block_indices: [[3s, 4s], [5s, 6s]], entities: [], palette: {}",
    )))
    .into_value();
    structure.structure.block_indices[0][1] = 9;
    let written = structure.to_bytes().unwrap();
    let again = loads(&written).into_value();
    assert_eq!(again.structure.block_indices, [vec![0, 9], vec![0, 0]]);
    // The untouched layer is still the file's Shorts.
    let Value::Compound(root) = nbtx::from_le_bytes::<Value>(&mut written.as_slice()).unwrap()
    else {
        panic!("a compound");
    };
    let Some(Value::Compound(body)) = root.get(b"structure".as_slice()) else {
        panic!("structure");
    };
    assert_eq!(
        body.get(b"block_indices".as_slice()),
        Some(&Value::List(ValueList::List(vec![
            ValueList::Int(vec![0, 9]),
            ValueList::Short(vec![5, 6]),
        ])))
    );
}

#[test]
fn a_list_of_another_length_is_written_new() {
    let mut structure = loads(&fixture("s19_indices_non_int")).into_value();
    structure.size = [1, 1, 2];
    structure.structure.block_indices = [vec![0, 0], vec![-1, -1]];
    let again = loads(&structure.to_bytes().unwrap()).into_value();
    assert_eq!(again, structure);
}

#[test]
fn tags_added_and_removed_in_a_kept_layout_are_written_so() {
    // `tick_block_structure` keeps its own tag order (`structure` last).
    let mut structure = loads(&fixture("capture_tick_block_structure")).into_value();
    structure.neighbor_aware_block_upgrade_version = Some(1);
    structure
        .structure
        .palette
        .insert("extra".to_owned(), Palette::default());
    let written = structure.to_bytes().unwrap();
    let again = loads(&written).into_value();
    assert_eq!(again, structure);
    let Value::Compound(root) = nbtx::from_le_bytes::<Value>(&mut written.as_slice()).unwrap()
    else {
        panic!("a compound");
    };
    let names: Vec<String> = root
        .keys()
        .map(|name| String::from_utf8_lossy(name).into_owned())
        .collect();
    // The file's tags in the file's order, then the new one.
    assert_eq!(
        names,
        [
            "format_version",
            "size",
            "structure_world_origin",
            "structure",
            "neighbor_aware_block_upgrade_version"
        ]
    );
    // Removing them again gives the file back.
    let mut restored = again;
    restored.neighbor_aware_block_upgrade_version = None;
    restored.structure.palette.shift_remove("extra");
    assert_eq!(
        restored.to_bytes().unwrap(),
        fixture("capture_tick_block_structure")
    );
}

#[test]
fn a_structure_built_in_code_is_written_tag_by_name_with_empty_lists_of_end() {
    use bedrock_addon::families::structure::{BlockPositionData, TickQueueEntry};
    // Empty layers, entities and block palette are lists of End tags.
    assert_eq!(
        Structure::new([0, 0, 0]).to_bytes().unwrap(),
        bytes(
            "{format_version: 1, size: [0, 0, 0], structure: {block_indices: [[], []], entities: [], palette: {}}, structure_world_origin: [0, 0, 0]}"
        )
    );
    // Unknown tags sort among the known ones at every level, a tick's included.
    let extra = |name: &str| nbtx::Compound::from_iter([(name.into(), Value::Byte(1))]);
    let mut structure = Structure::new([0, 0, 0]);
    structure.neighbor_aware_block_upgrade_version = Some(1);
    structure.extra = extra("a_root");
    structure.structure.extra = extra("zz_body");
    structure.structure.palette.insert(
        "default".to_owned(),
        Palette {
            block_position_data: [
                (
                    10,
                    BlockPositionData {
                        tick_queue_data: Some(vec![TickQueueEntry {
                            tick_delay: Some(3),
                            extra: extra("zeta"),
                        }]),
                        extra: extra("a_position"),
                        ..BlockPositionData::default()
                    },
                ),
                (2, BlockPositionData::default()),
            ]
            .into(),
            extra: extra("a_palette"),
            ..Palette::default()
        },
    );
    structure
        .structure
        .palette
        .insert("alpha".to_owned(), Palette::default());
    assert_eq!(
        structure.to_bytes().unwrap(),
        bytes(
            r#"{a_root: 1b, format_version: 1, neighbor_aware_block_upgrade_version: 1, size: [0, 0, 0], structure: {block_indices: [[], []], entities: [], palette: {alpha: {block_palette: [], block_position_data: {}}, default: {a_palette: 1b, block_palette: [], block_position_data: {"10": {a_position: 1b, tick_queue_data: [{tick_delay: 3, zeta: 1b}]}, "2": {}}}}, zz_body: 1b}, structure_world_origin: [0, 0, 0]}"#
        )
    );
}

#[test]
fn what_nbt_cannot_hold_is_not_representable() {
    let mut structure = Structure::new([0, 0, 0]);
    structure
        .structure
        .palette
        .insert("p".repeat(40_000), Palette::default());
    assert!(matches!(
        structure.to_bytes(),
        Err(EncodeError::NotRepresentable { .. })
    ));
    let findings = structure.validate(&LoadContext::detached());
    assert_eq!(findings.len(), 1);
    let finding = findings.iter().next().unwrap();
    assert_eq!(finding.code.id(), "core/syntax");
    assert_eq!(finding.severity, Severity::Error);
}

// ---- construction and positions -----------------------------------------------------------

#[test]
fn new_with_a_component_below_1_has_empty_layers() {
    for size in [[2, -1, 2], [0, 4, 4], [3, 3, 0]] {
        let structure = Structure::new(size);
        assert_eq!(
            structure.structure.block_indices,
            [vec![], vec![]],
            "{size:?}"
        );
    }
    assert_eq!(
        Structure::new([1, 1, 1]).structure.block_indices,
        [vec![-1], vec![-1]]
    );
}

#[test]
fn new_fills_the_layers_up_to_the_largest_box_structure_save_takes() {
    // 64 × 384 × 64 positions (the `/structure save` limit in the overworld) are filled; one
    // more block on any axis, and any size whose volume overflows, gets empty layers, which
    // vanilla accepts for any size. No panic, no allocation of the volume.
    let full = Structure::new([64, 384, 64]);
    assert_eq!(full.structure.block_indices[0].len(), 64 * 384 * 64);
    assert_eq!(full.structure.block_indices[1].len(), 64 * 384 * 64);
    assert!(full.structure.block_indices[0].iter().all(|i| *i == -1));
    for size in [
        [65, 384, 64],
        [64, 385, 64],
        [64, 384, 65],
        [1, 1, 64 * 384 * 64 + 1],
        [i32::MAX, 1, 1],
        [i32::MAX; 3],
        [65_536, 65_536, 1],
    ] {
        let structure = Structure::new(size);
        assert_eq!(structure.size, size);
        let lengths = structure.structure.block_indices.each_ref().map(Vec::len);
        assert_eq!(lengths, [0, 0], "{size:?}");
        assert!(
            structure.validate(&LoadContext::detached()).is_empty(),
            "{size:?}"
        );
    }
    // The limit is on the volume, not on an axis.
    assert_eq!(
        Structure::new([1, 1, 64 * 384 * 64])
            .structure
            .block_indices[0]
            .len(),
        64 * 384 * 64
    );
}

#[test]
fn index_is_bounded_on_every_axis() {
    let structure = Structure::new([2, 3, 4]);
    for inside in [[1, 0, 0], [0, 2, 0], [0, 0, 3]] {
        assert!(structure.index(inside).is_some(), "{inside:?}");
    }
    for outside in [
        [2, 0, 0],
        [0, 3, 0],
        [0, 0, 4],
        [-1, 0, 0],
        [0, -1, 0],
        [0, 0, -1],
    ] {
        assert_eq!(structure.index(outside), None, "{outside:?}");
    }
    // A box with a negative size has no positions.
    let mut broken = Structure::new([1, 1, 1]);
    broken.size = [1, -1, 1];
    assert_eq!(broken.index([0, 0, 0]), None);
}

#[test]
fn index_of_a_position_past_what_an_index_can_count_is_none() {
    // The largest box: its far corner's index does not fit a `usize`; no panic.
    let huge = Structure::new([i32::MAX; 3]);
    assert_eq!(huge.index([i32::MAX - 1; 3]), None);
    // Its near positions still have one.
    assert_eq!(huge.index([0, 0, 5]), Some(5));
    assert_eq!(huge.index([0, 1, 0]), Some(2_147_483_647));
}

// ---- listing and lookups ------------------------------------------------------------------

fn id(text: &str) -> bedrock_addon::id::StructureId {
    bedrock_addon::id::StructureId::parse(text)
}

/// A valid structure whose world origin tells the copies apart.
fn marked(x: i32) -> &'static [u8] {
    Box::leak(
        bytes(&doc(
            &format!("size: [1, 1, 1], structure_world_origin: [{x}, 0, 0]"),
            BODY,
        ))
        .into_boxed_slice(),
    )
}

/// A file that fails to load.
fn broken() -> &'static [u8] {
    Box::leak(fixture("s04_format_2").into_boxed_slice())
}

/// The world origin's x of the structure `find` gives for `text`.
fn found(stack: &PackStack<Behavior>, text: &str) -> Option<i32> {
    Structure::find(stack.structures(), &id(text))
        .expect("namespaced")
        .map(|d| d.structure_world_origin[0])
}

fn stack_of(packs: impl IntoIterator<Item = bedrock_addon::pack::Pack>) -> PackStack<Behavior> {
    PackStack::<Behavior>::of(packs, LoadOptions::default()).expect("a behaviour stack")
}

#[test]
fn only_files_under_structures_with_the_exact_extension_have_an_identifier() {
    let id_of = |path: &'static str| {
        Structure::id_of(&PackPath::from_static(path)).map(|id| id.as_str().to_owned())
    };
    assert_eq!(
        id_of("structures/a.mcstructure").as_deref(),
        Some("mystructure:a")
    );
    assert_eq!(id_of("structures/a/b.mcstructure").as_deref(), Some("a:b"));
    assert_eq!(
        id_of("structures/a/b/c.d.mcstructure").as_deref(),
        Some("a:b/c.d")
    );
    for path in [
        "other/a.mcstructure",
        "structuresx/a.mcstructure",
        "a.mcstructure",
        "structures/a.nbt",
        "structures/a.mcstructure.bak",
        "structures/a.Mcstructure",
    ] {
        assert_eq!(id_of(path), None, "{path}");
    }
}

#[test]
fn an_empty_namespace_is_no_namespace() {
    let stack = stack_of([bp(1).file("structures/house.mcstructure", marked(1)).open()]);
    let finding = Structure::find(stack.structures(), &id(":house")).unwrap_err();
    assert_eq!(
        finding.code.to_string(),
        "Structure ':house' must be prefixed by a namespace"
    );
    assert!(Structure::path_of(&id(":house")).is_none());
}

#[test]
fn a_higher_packs_mystructure_copy_beats_a_lower_packs_root_copy() {
    let top = bp(1)
        .file("structures/mystructure/house.mcstructure", marked(1))
        .open();
    let bottom = bp(2).file("structures/house.mcstructure", marked(2)).open();
    assert_eq!(
        found(&stack_of([top, bottom]), "mystructure:house"),
        Some(1)
    );
}

#[test]
fn another_packs_failed_primary_copy_does_not_hide_a_root_copy() {
    let top = bp(1).file("structures/house.mcstructure", marked(1)).open();
    let bottom = bp(2)
        .file("structures/mystructure/house.mcstructure", broken())
        .open();
    let stack = stack_of([top, bottom]);
    assert_eq!(found(&stack, "mystructure:house"), Some(1));
    // No pack's root copy is hidden behind its own failed primary copy here, so no root copy
    // stands in at the primary path: the collection holds each file once.
    let primary = PackPath::from_static("structures/mystructure/house.mcstructure");
    assert!(stack.structures().get(&primary).is_none());
    assert_eq!(stack.structures().iter().count(), 1);
}

#[test]
fn a_stand_in_is_listed_at_the_primary_path_only_where_a_root_copy_is_hidden() {
    let top = bp(1)
        .file("structures/house.mcstructure", marked(1))
        .file("structures/mystructure/house.mcstructure", broken())
        .file("structures/barn.mcstructure", marked(5))
        .open();
    // `zoo` sorts after `house`: only the root copy of the hidden name stands in.
    let bottom = bp(2)
        .file("structures/house.mcstructure", marked(2))
        .file("structures/barn.mcstructure", marked(6))
        .file("structures/zoo.mcstructure", marked(9))
        .open();
    let stack = stack_of([top, bottom]);
    let structures = stack.structures();
    let mut entries: Vec<(String, i32, String)> = structures
        .iter()
        .map(|(path, d)| {
            (
                path.to_string(),
                d.structure_world_origin[0],
                d.origin().pack.name().to_owned(),
            )
        })
        .collect();
    entries.sort();
    assert_eq!(
        entries,
        [
            (
                "structures/barn.mcstructure".to_owned(),
                5,
                "bp 1".to_owned()
            ),
            (
                "structures/house.mcstructure".to_owned(),
                1,
                "bp 1".to_owned()
            ),
            (
                "structures/mystructure/house.mcstructure".to_owned(),
                2,
                "bp 2".to_owned()
            ),
            (
                "structures/zoo.mcstructure".to_owned(),
                9,
                "bp 2".to_owned()
            ),
        ]
    );
    assert_eq!(found(&stack, "mystructure:barn"), Some(5));
}

#[test]
fn a_root_copy_stands_in_once_however_many_packs_fail_their_primary_copy() {
    let blocked = |n: u128, x: i32| {
        bp(n)
            .file("structures/house.mcstructure", marked(x))
            .file("structures/mystructure/house.mcstructure", broken())
            .open()
    };
    let bottom = bp(3).file("structures/house.mcstructure", marked(3)).open();
    let options = LoadOptions {
        keep_shadowed: true,
        ..LoadOptions::default()
    };
    let stack = PackStack::<Behavior>::of([blocked(1, 1), blocked(2, 2), bottom], options)
        .expect("a behaviour stack");
    let structures = stack.structures();
    let primary = PackPath::from_static("structures/mystructure/house.mcstructure");
    let copies: Vec<i32> = structures
        .layers(&primary)
        .map(|d| d.structure_world_origin[0])
        .collect();
    assert_eq!(copies, [3]);
    assert_eq!(found(&stack, "mystructure:house"), Some(3));
}

#[test]
fn a_loaded_primary_copy_beside_a_root_copy_hides_nothing() {
    // No primary copy failed: the lower pack's root copy does not stand in at the primary path.
    let top = bp(1)
        .file("structures/house.mcstructure", marked(1))
        .file("structures/mystructure/house.mcstructure", marked(2))
        .open();
    let bottom = bp(2).file("structures/house.mcstructure", marked(3)).open();
    let options = LoadOptions {
        keep_shadowed: true,
        ..LoadOptions::default()
    };
    let stack = PackStack::<Behavior>::of([top, bottom], options).expect("a behaviour stack");
    let primary = PackPath::from_static("structures/mystructure/house.mcstructure");
    let copies: Vec<i32> = stack
        .structures()
        .layers(&primary)
        .map(|d| d.structure_world_origin[0])
        .collect();
    assert_eq!(copies, [2]);
    assert_eq!(found(&stack, "mystructure:house"), Some(2));
}

#[test]
fn only_the_mystructure_folder_has_a_root_path() {
    // `mystructure2` merely starts like the default namespace: its failed file hides nothing,
    // and no file of the folder `2` stands in for it.
    let top = bp(1)
        .file("structures/mystructure2/x.mcstructure", broken())
        .file("structures/2/x.mcstructure", marked(1))
        .open();
    let bottom = bp(2).file("structures/2/x.mcstructure", marked(2)).open();
    let stack = stack_of([top, bottom]);
    assert_eq!(found(&stack, "mystructure2:x"), None);
    assert_eq!(found(&stack, "2:x"), Some(1));
    assert_eq!(stack.structures().iter().count(), 1);
}

#[test]
fn another_namespace_never_gets_a_root_copy_in_its_place() {
    // the family notes: only `mystructure` reads the root-level path. A failed `structures/demo/x`
    // beside a loaded root `x` hides nothing of `demo:x`, and the lower pack's root `x` does
    // not stand in for it.
    let top = bp(1)
        .file("structures/demo/x.mcstructure", broken())
        .file("structures/x.mcstructure", marked(1))
        .open();
    let bottom = bp(2).file("structures/x.mcstructure", marked(2)).open();
    let stack = stack_of([top, bottom]);
    assert_eq!(found(&stack, "demo:x"), None);
    let primary = PackPath::from_static("structures/demo/x.mcstructure");
    assert!(stack.structures().get(&primary).is_none());
    assert_eq!(stack.structures().iter().count(), 1);
    assert_eq!(found(&stack, "mystructure:x"), Some(1));
}

#[test]
fn another_failed_file_of_the_pack_does_not_hide_its_root_copy() {
    let pack = bp(1)
        .file("structures/house.mcstructure", marked(1))
        .file("structures/mystructure/barn.mcstructure", broken())
        .open();
    assert_eq!(found(&stack_of([pack]), "mystructure:house"), Some(1));
}

#[test]
fn a_pack_whose_only_copies_fail_leaves_the_structure_unfound() {
    let pack = bp(1)
        .file("structures/house.mcstructure", marked(1))
        .file("structures/mystructure/house.mcstructure", broken())
        .open();
    let stack = stack_of([pack]);
    assert_eq!(found(&stack, "mystructure:house"), None);
    // The jigsaw lookup has no `mystructure` rule: the root file is its file.
    let template = Structure::find_template(stack.structures(), &id("x:house")).expect("found");
    assert_eq!(template.structure_world_origin[0], 1);
}

#[test]
fn a_lower_packs_root_copy_is_found_below_a_pack_whose_primary_copy_failed() {
    // the family notes: on failure `StructureManager::load` "continues with the next pack in the stack
    // that has the file"; the next pack has no `mystructure/house`, so its root copy is read.
    // The top pack's own root copy is not (it has the primary path).
    let noted: &'static [u8] = Box::leak(
        bytes(&doc(
            "size: [1, 1, 1], structure_world_origin: [2, 0, 0]",
            "block_indices: [[5], [-1]], entities: [], palette: {default: {block_palette: [], block_position_data: {}}}",
        ))
        .into_boxed_slice(),
    );
    let top = bp(1)
        .file("structures/house.mcstructure", marked(1))
        .file("structures/mystructure/house.mcstructure", broken())
        .open();
    let middle = bp(2).file("structures/house.mcstructure", broken()).open();
    let bottom = bp(3).file("structures/house.mcstructure", noted).open();
    let stack = stack_of([top, middle, bottom]);
    assert_eq!(found(&stack, "mystructure:house"), Some(2));
    let structures = stack.structures();
    let document = Structure::find(structures, &id("mystructure:house"))
        .unwrap()
        .unwrap();
    assert_eq!(document.origin().pack.name(), "bp 3");
    assert_eq!(
        document.origin().path.as_str(),
        "structures/house.mcstructure"
    );
    // The jigsaw lookup still reads the top pack's root file.
    let template = Structure::find_template(structures, &id("x:house")).expect("found");
    assert_eq!(template.structure_world_origin[0], 1);
    // Each file is reported once: two failed copies, one index note.
    assert_eq!(structures.rejected().len(), 2);
    let severities: Vec<Severity> = structures.diagnostics().map(|f| f.severity).collect();
    assert_eq!(
        severities.iter().filter(|s| **s == Severity::Error).count(),
        2
    );
    assert_eq!(
        severities.iter().filter(|s| **s == Severity::Info).count(),
        1
    );
}

#[test]
fn a_lower_packs_primary_copy_still_beats_its_own_root_copy_below_a_failed_pack() {
    let top = bp(1)
        .file("structures/house.mcstructure", marked(1))
        .file("structures/mystructure/house.mcstructure", broken())
        .open();
    let bottom = bp(2)
        .file("structures/house.mcstructure", marked(2))
        .file("structures/mystructure/house.mcstructure", marked(3))
        .open();
    assert_eq!(
        found(&stack_of([top, bottom]), "mystructure:house"),
        Some(3)
    );
    // And a lower pack whose primary copy fails too passes its turn as well.
    let top = bp(1)
        .file("structures/house.mcstructure", marked(1))
        .file("structures/mystructure/house.mcstructure", broken())
        .open();
    let middle = bp(2)
        .file("structures/house.mcstructure", marked(2))
        .file("structures/mystructure/house.mcstructure", broken())
        .open();
    let bottom = bp(3).file("structures/house.mcstructure", marked(4)).open();
    assert_eq!(
        found(&stack_of([top, middle, bottom]), "mystructure:house"),
        Some(4)
    );
}

#[test]
fn files_of_other_extensions_under_structures_are_neither_read_nor_reported() {
    let pack = bp(1)
        .file("structures/notes.json", b"{}")
        .file("structures/ruins/tower.NBT", b"x")
        .file("structures/ruins/tower.nbt.bak", b"x")
        .open();
    let stack = stack_of([pack]);
    assert_eq!(stack.structures().iter().count(), 0);
    assert_eq!(stack.structures().diagnostics().count(), 0);
}
