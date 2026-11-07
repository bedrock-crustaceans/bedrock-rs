//! The fixtures, the file rows `s01`–`s33`: each file is loaded on its own;
//! a failing rule rejects the file with vanilla's text.

use bedrock_addon_family_structure::{Contents, Layout, Palette, Structure};
use indexmap::IndexMap;
use nbtx::Value;

use super::common::{fails_with, fixture, loads, rejects};

/// A raw block palette entry `{name, states: {}, version}`.
pub fn block(name: &str, version: i32) -> nbtx::Compound {
    nbtx::Compound::from_iter([
        ("name".into(), Value::String(name.into())),
        ("states".into(), Value::Compound(nbtx::Compound::new())),
        ("version".into(), Value::Int(version)),
    ])
}

#[test]
fn s01_a_vanilla_1x1x1_export_loads() {
    let document = loads(&fixture("s01_vanilla_1x1x1"));
    let expected = Structure {
        format_version: 1,
        neighbor_aware_block_upgrade_version: None,
        size: [1, 1, 1],
        structure_world_origin: [12, 64, -7],
        structure: Contents {
            block_indices: [vec![0], vec![-1]],
            entities: Vec::new(),
            palette: IndexMap::from_iter([(
                "default".to_owned(),
                Palette {
                    block_palette: vec![block("minecraft:stone", 18_168_865)],
                    ..Palette::default()
                },
            )]),
            extra: nbtx::Compound::new(),
        },
        extra: nbtx::Compound::new(),
        layout: Layout::default(),
    };
    assert_eq!(*document.value(), expected);
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    // The palette `default` has one entry; layer 1 is `[-1]`.
    assert_eq!(document.structure.palette["default"].block_palette.len(), 1);
    assert_eq!(document.structure.block_indices[1], [-1]);
}

/// Vanilla's required-field text for `field`.
fn required(field: &str) -> String {
    format!("\"{field}\" field, a required field, is missing from the structure.")
}

#[test]
fn s02_a_format_version_of_0_or_none_is_the_required_field_error() {
    for name in ["s02_format_0", "s02_format_missing"] {
        let finding = fails_with(
            &fixture(name),
            "structure/required",
            &required("format_version"),
        );
        assert_eq!(finding.location.json_path.to_string(), "format_version");
    }
}

#[test]
fn s03_a_format_version_short_reads_as_0() {
    fails_with(
        &fixture("s03_format_short"),
        "structure/required",
        &required("format_version"),
    );
}

#[test]
fn s04_a_format_version_of_2_is_larger_than_the_current_version() {
    let finding = fails_with(
        &fixture("s04_format_2"),
        "structure/larger_version",
        "\"format_version\" field has a value larger than the current version.  We expect a format version with a value at most 1 and got 2.",
    );
    assert_eq!(finding.location.json_path.to_string(), "format_version");
}

#[test]
fn s05_a_format_version_of_minus_1_is_smaller_than_the_current_version() {
    fails_with(
        &fixture("s05_format_neg1"),
        "structure/smaller_version",
        "\"format_version\" field has a value smaller than the current version.  We expect a format version with a value at most 1 and got -1.",
    );
}

#[test]
fn s06_a_neighbor_aware_version_outside_0_to_1_is_an_invalid_value() {
    for (name, value) in [("s06_nabu_2", 2), ("s06_nabu_neg1", -1)] {
        let finding = fails_with(
            &fixture(name),
            "structure/invalid_value",
            &format!(
                "\"neighbor_aware_block_upgrade_version\" field has an invalid value of {value}. Valid values are 0 and up to 1."
            ),
        );
        assert_eq!(
            finding.location.json_path.to_string(),
            "neighbor_aware_block_upgrade_version"
        );
    }
}

#[test]
fn s07_a_neighbor_aware_version_string_is_not_an_integer() {
    fails_with(
        &fixture("s07_nabu_string"),
        "structure/not_an_integer",
        "\"neighbor_aware_block_upgrade_version\" field is not an integer.",
    );
}

#[test]
fn s08_an_absent_neighbor_aware_version_reads_as_0() {
    let document = loads(&fixture("s08_nabu_absent"));
    assert_eq!(document.neighbor_aware_block_upgrade_version, None);
    // Absent is version 0 (`None`): walls are re-connected when placed.
    assert_eq!(document.neighbor_aware_block_upgrade_version(), 0);
    let wall = &document.structure.palette["default"].block_palette[0];
    assert_eq!(
        wall.get(b"name".as_slice()),
        Some(&Value::String("minecraft:cobblestone_wall".into()))
    );
}

#[test]
fn s09_a_size_of_2_or_4_ints_expects_3_elements() {
    for name in ["s09_size_2elems", "s09_size_4elems"] {
        let finding = fails_with(
            &fixture(name),
            "structure/three_elements",
            "The \"size\" field expects 3 elements.",
        );
        assert_eq!(finding.location.json_path.to_string(), "size");
    }
}

#[test]
fn s10_a_negative_size_component_fails() {
    fails_with(
        &fixture("s10_size_negative"),
        "structure/negative",
        "The \"size\" field shouldn't have any negative values.",
    );
}

#[test]
fn s11_a_size_int_array_is_not_a_list() {
    fails_with(
        &fixture("s11_size_not_list"),
        "structure/required",
        &required("size"),
    );
}

#[test]
fn s12_a_negative_world_origin_loads() {
    let document = loads(&fixture("s12_origin_negative"));
    assert_eq!(document.structure_world_origin, [-5, -64, 3]);
}

#[test]
fn s13_a_root_without_structure_is_the_required_field_error() {
    let finding = fails_with(
        &fixture("s13_no_structure"),
        "structure/required",
        &required("structure"),
    );
    assert_eq!(finding.location.json_path.to_string(), "structure");
}

#[test]
fn s14_block_indices_must_have_exactly_2_layers() {
    for (name, count) in [("s14_indices_1_layer", 1), ("s14_indices_3_layers", 3)] {
        let finding = fails_with(
            &fixture(name),
            "structure/two_arrays",
            &format!(
                "The \"block_indices\" field should be an array with 2 arrays and instead we have {count} arrays."
            ),
        );
        assert_eq!(
            finding.location.json_path.to_string(),
            "structure/block_indices"
        );
    }
}

#[test]
fn s15_a_layer_that_is_not_a_list_fails() {
    // Corrected fixture: an NBT list holds one tag type, so both layers are
    // IntArrays and the first is reported.
    fails_with(
        &fixture("s15_indices_layer_not_list"),
        "structure/first_array",
        "The \"block_indices\" field's first array is either missing or not a list.",
    );
}

#[test]
fn s16_layers_of_unequal_length_fail() {
    fails_with(
        &fixture("s16_indices_unequal"),
        "structure/same_size",
        "The \"block_indices\" field's arrays need to both be the same size.",
    );
}

#[test]
fn s17_an_index_count_other_than_the_volume_fails() {
    fails_with(
        &fixture("s17_indices_volume"),
        "structure/volume",
        "The \"block_indices\" field should have as many elements as defined by the \"size\" field.",
    );
}

#[test]
fn s18_two_empty_layers_skip_the_volume_check() {
    let document = loads(&fixture("s18_indices_empty"));
    assert_eq!(document.size, [2, 2, 2]);
    assert_eq!(document.structure.block_indices, [vec![], vec![]]);
}

#[test]
fn s19_indices_of_another_type_decode_as_0() {
    let document = loads(&fixture("s19_indices_non_int"));
    assert_eq!(document.structure.block_indices, [vec![0], vec![0]]);
}

#[test]
fn s20_a_structure_without_entities_is_the_required_field_error() {
    let finding = fails_with(
        &fixture("s20_entities_missing"),
        "structure/required",
        &required("entities"),
    );
    assert_eq!(finding.location.json_path.to_string(), "structure/entities");
}

#[test]
fn s21_an_entity_that_is_not_a_compound_fails_the_file() {
    let finding = fails_with(
        &fixture("s21_entities_int"),
        "structure/invalid_entity",
        "At index 0 while reading \"entities\", we found an invalid entity in the list of entities.",
    );
    assert_eq!(
        finding.location.json_path.to_string(),
        "structure/entities/0"
    );
}

#[test]
fn s22_a_structure_without_palette_is_the_required_field_error() {
    let finding = fails_with(
        &fixture("s22_palette_missing"),
        "structure/required",
        &required("palette"),
    );
    assert_eq!(finding.location.json_path.to_string(), "structure/palette");
}

#[test]
fn s23_a_palette_that_is_not_a_compound_fails() {
    let finding = fails_with(
        &fixture("s23_palette_value_int"),
        "structure/compound_elements",
        "Expected a compound tag when parsing elements of \"palette\" field.",
    );
    assert_eq!(
        finding.location.json_path.to_string(),
        "structure/palette/default"
    );
}

#[test]
fn s24_an_empty_palette_compound_loads_without_palettes() {
    let document = loads(&fixture("s24_palette_empty"));
    assert!(document.structure.palette.is_empty());
}

#[test]
fn s25_a_block_palette_entry_that_is_not_a_compound_fails() {
    let finding = fails_with(
        &fixture("s25_block_palette_int_elem"),
        "structure/invalid_block",
        "At index 0 while reading \"block_palette\", we found an invalid block in the palette, we were expecting a compound tag here.",
    );
    assert_eq!(
        finding.location.json_path.to_string(),
        "structure/palette/default/block_palette/0"
    );
}

#[test]
fn s26_a_palette_without_block_position_data_is_the_required_field_error() {
    let finding = fails_with(
        &fixture("s26_bpd_missing"),
        "structure/required",
        &required("block_position_data"),
    );
    assert_eq!(
        finding.location.json_path.to_string(),
        "structure/palette/default/block_position_data"
    );
}

#[test]
fn s27_a_block_position_key_that_is_not_a_number_fails() {
    fails_with(
        &fixture("s27_bpd_key_nonnumeric"),
        "structure/not_a_number",
        "Expected a string formatted as a number when parsing contents of the \"block_position_data\" field, but got \"a\"",
    );
}

#[test]
fn s28_a_block_position_value_that_is_not_a_compound_fails() {
    let finding = fails_with(
        &fixture("s28_bpd_value_int"),
        "structure/compound_value",
        "Expected a compound tag when parsing elements of \"block_position_data\" field at the value for \"0\".",
    );
    assert_eq!(
        finding.location.json_path.to_string(),
        "structure/palette/default/block_position_data/0"
    );
}

#[test]
fn s29_a_block_entity_that_is_not_a_compound_fails() {
    fails_with(
        &fixture("s29_bed_not_compound"),
        "structure/compound_field",
        "Expected a compound tag for the \"block_entity_data\" field.",
    );
}

#[test]
fn s30_tick_queue_data_must_be_a_list_of_compounds() {
    fails_with(
        &fixture("s30_tqd_not_list"),
        "structure/list_field",
        "Expected a list tag for the \"tick_queue_data\" field.",
    );
    let finding = fails_with(
        &fixture("s30_tqd_int_elem"),
        "structure/compound_at",
        "Expected a compound tag at index 0 when parsing elements of \"tick_queue_data\" field.",
    );
    assert_eq!(
        finding.location.json_path.to_string(),
        "structure/palette/default/block_position_data/0/tick_queue_data/0"
    );
}

#[test]
fn s31_an_old_palette_entry_is_kept_raw_with_its_version() {
    // Vanilla upgrades the entry (`minecraft:wool` + `color: red` at 17959425 →
    // `minecraft:red_wool`) only when the structure is placed, through the block NBT upgrader
    // shared with world chunks; loading keeps it as written.
    let document = loads(&fixture("s31_old_block_version"));
    assert!(document.diagnostics().is_empty());
    let entry = &document.structure.palette["default"].block_palette[0];
    assert_eq!(
        entry.get(b"name".as_slice()),
        Some(&Value::String("minecraft:wool".into()))
    );
    assert_eq!(
        entry.get(b"version".as_slice()),
        Some(&Value::Int(17_959_425))
    );
    let states = nbtx::Compound::from_iter([("color".into(), Value::String("red".into()))]);
    assert_eq!(
        entry.get(b"states".as_slice()),
        Some(&Value::Compound(states))
    );
}

#[test]
fn s32_an_unknown_palette_block_is_kept_raw() {
    // The block name is resolved in the link pass (20 W25, `link_rules.rs`); loading keeps the
    // entry, which vanilla places as the unknown-block placeholder.
    let document = loads(&fixture("s32_unknown_block"));
    assert!(document.diagnostics().is_empty());
    let entry = &document.structure.palette["default"].block_palette[0];
    assert_eq!(
        *entry,
        nbtx::Compound::from_iter([("name".into(), Value::String("demo:missing".into()))])
    );
}

#[test]
fn s33_unknown_tags_are_kept_without_a_diagnostic() {
    let document = loads(&fixture("s33_extra_keys"));
    assert!(document.diagnostics().is_empty());
    let one = |name: &str, value: &str| {
        nbtx::Compound::from_iter([(name.into(), Value::String(value.into()))])
    };
    assert_eq!(document.extra, one("zz_tool", "demo exporter"));
    assert_eq!(document.structure.extra, one("author", "demo"));
    assert_eq!(
        document.structure.palette["default"].extra,
        one("note", "palette")
    );
}

/// The info findings of `document`, as `(code name, tag path)`.
fn infos(document: &bedrock_addon_core::document::Document<Structure>) -> Vec<(String, String)> {
    document
        .diagnostics()
        .iter()
        .filter(|f| f.severity == bedrock_addon_core::diagnostic::Severity::Info)
        .map(|f| (f.code.id().to_owned(), f.location.json_path.to_string()))
        .collect()
}

#[test]
fn n09_negative_indices_load_and_below_minus_1_is_noted_deviation() {
    // Placement skips every negative index, so both positions keep the world block.
    // Ours: an index outside -1..palette length is an info; vanilla says nothing.
    let document = loads(&fixture("n09_negative_indices"));
    assert_eq!(document.structure.block_indices[0], [-1, -7]);
    assert_eq!(
        infos(&document),
        [(
            "core/invalid_value".to_owned(),
            "structure/block_indices/0/1".to_owned()
        )]
    );
}

#[test]
fn n10_an_index_past_the_palette_loads_with_an_info_deviation() {
    // Placement puts `minecraft:air` there (bin; ours: an info, vanilla silent.
    let document = loads(&fixture("n10_index_past_palette"));
    assert_eq!(document.structure.block_indices[0], [5]);
    assert_eq!(
        infos(&document),
        [(
            "core/invalid_value".to_owned(),
            "structure/block_indices/0/0".to_owned()
        )]
    );
    assert!(!document.diagnostics().has_errors());
}

#[test]
fn indices_are_checked_against_the_default_palette_only() {
    // s01: index 0 and -1 against a one-entry `default` palette: nothing to note.
    assert!(infos(&loads(&fixture("s01_vanilla_1x1x1"))).is_empty());
    // s24: no palettes at all, so nothing is checked.
    assert!(infos(&loads(&fixture("s24_palette_empty"))).is_empty());
}

#[test]
fn every_finding_of_a_structure_file_carries_its_pack_and_path() {
    use bedrock_addon_core::pack_identity::{PackPath, PackRef};
    let path = PackPath::from_static("structures/demo/test.mcstructure");
    let check = |found: &bedrock_addon_core::diagnostic::Diagnostics| {
        assert!(!found.is_empty());
        for finding in found {
            assert_eq!(
                finding.location.pack,
                Some(PackRef::app_data()),
                "{finding:#?}"
            );
            assert_eq!(finding.location.path.as_ref(), Some(&path), "{finding:#?}");
        }
    };
    // Loaded with an information.
    check(loads(&fixture("n09_negative_indices")).diagnostics());
    // Rejected: no NBT at all.
    check(&rejects(b"not nbt"));
}
