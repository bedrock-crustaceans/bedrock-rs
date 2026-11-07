//! The validation rules, the load rules no fixture row covers: each failing
//! rule rejects the file with vanilla's text (and its code, the validation rules).

use super::common::{bytes, fails_with, loads};

/// A valid 1x1x1 structure with `root` as its root members after `format_version` and `size`,
/// `body` as the members of `structure`.
fn doc(root: &str, body: &str) -> Vec<u8> {
    bytes(&format!(
        "{{format_version: 1, size: [1, 1, 1], {root} structure: {{{body}}}}}"
    ))
}

/// A valid `structure` compound.
const BODY: &str = r#"block_indices: [[0], [-1]], entities: [], palette: {default: {block_palette: [{name: "minecraft:stone"}], block_position_data: {}}}"#;

/// Vanilla's required-field text for `field`.
fn required(field: &str) -> String {
    format!("\"{field}\" field, a required field, is missing from the structure.")
}

#[test]
fn rule_7_a_missing_world_origin_is_required() {
    for origin in [
        "",
        "structure_world_origin: 3,",
        "structure_world_origin: [I; 0, 0, 0],",
    ] {
        let finding = fails_with(
            &doc(origin, BODY),
            "structure/required",
            &required("structure_world_origin"),
        );
        assert_eq!(
            finding.location.json_path.to_string(),
            "structure_world_origin"
        );
    }
}

#[test]
fn rule_7_a_world_origin_of_2_ints_expects_3_elements() {
    fails_with(
        &doc("structure_world_origin: [1, 2],", BODY),
        "structure/three_elements",
        "The \"structure_world_origin\" field expects 3 elements.",
    );
}

#[test]
fn rule_6_a_missing_size_is_required() {
    fails_with(
        &bytes(&format!(
            "{{format_version: 1, structure_world_origin: [0, 0, 0], structure: {{{BODY}}}}}"
        )),
        "structure/required",
        &required("size"),
    );
}

#[test]
fn rule_6_size_elements_of_another_type_read_as_0() {
    // `ListTag::getInt` reads a Short as 0: a 0x0x0 box.
    let document = loads(&bytes(
        "{format_version: 1, size: [1s, 1s, 1s], structure_world_origin: [0, 0, 0], structure: {block_indices: [[], []], entities: [], palette: {}}}",
    ));
    assert_eq!(document.size, [0, 0, 0]);
}

#[test]
fn rule_8_a_structure_that_is_not_a_compound_is_required() {
    fails_with(
        &bytes(
            "{format_version: 1, size: [1, 1, 1], structure_world_origin: [0, 0, 0], structure: [1]}",
        ),
        "structure/required",
        &required("structure"),
    );
}

#[test]
fn rule_9_block_indices_missing_or_not_a_list_is_required() {
    let rest = r"entities: [], palette: {}";
    for indices in ["", "block_indices: 0,", "block_indices: [I; 0, -1],"] {
        let finding = fails_with(
            &doc(
                "structure_world_origin: [0, 0, 0],",
                &format!("{indices} {rest}"),
            ),
            "structure/required",
            &required("block_indices"),
        );
        assert_eq!(
            finding.location.json_path.to_string(),
            "structure/block_indices"
        );
    }
}

#[test]
fn rule_10_the_volume_is_an_i32_product_and_wraps() {
    // 65537 × 65537 × 1 = 2^32 + 2^17 + 1, which wraps to 131 073 as an i32: layers of that
    // length load, and the true volume would not.
    let layer = vec!["0"; 131_073].join(",");
    let document = loads(&bytes(&format!(
        "{{format_version: 1, size: [65537, 65537, 1], structure_world_origin: [0, 0, 0], structure: {{block_indices: [[{layer}], [{layer}]], entities: [], palette: {{}}}}}}"
    )));
    assert_eq!(document.structure.block_indices[0].len(), 131_073);
    // 65536 × 65536 × 1 wraps to 0, which a one-index layer does not match: the check is not
    // skipped.
    fails_with(
        &bytes(
            "{format_version: 1, size: [65536, 65536, 1], structure_world_origin: [0, 0, 0], structure: {block_indices: [[0], [-1]], entities: [], palette: {}}}",
        ),
        "structure/volume",
        "The \"block_indices\" field should have as many elements as defined by the \"size\" field.",
    );
}

/// A 1x1x1 structure whose `default` palette has the members `palette` and whose entities are
/// `entities`.
fn with_palette(palette: &str, entities: &str) -> Vec<u8> {
    doc(
        "structure_world_origin: [0, 0, 0],",
        &format!(
            "block_indices: [[0], [-1]], entities: {entities}, palette: {{default: {{{palette}}}}}"
        ),
    )
}

#[test]
fn rule_11_a_palette_that_is_not_a_compound_is_required() {
    fails_with(
        &doc(
            "structure_world_origin: [0, 0, 0],",
            "block_indices: [[0], [-1]], entities: [], palette: [1]",
        ),
        "structure/required",
        &required("palette"),
    );
}

#[test]
fn rule_12_a_palette_without_block_palette_is_the_required_field_error() {
    for block_palette in ["", "block_palette: 1,"] {
        let finding = fails_with(
            &with_palette(&format!("{block_palette} block_position_data: {{}}"), "[]"),
            "structure/required",
            &required("block_palette"),
        );
        assert_eq!(
            finding.location.json_path.to_string(),
            "structure/palette/default/block_palette"
        );
    }
}

#[test]
fn rule_12_block_position_keys_are_not_checked_against_the_volume() {
    let document = loads(&with_palette(
        r#"block_palette: [], block_position_data: {"-5": {}, "900": {tick_queue_data: [{other: 1b}]}}"#,
        "[]",
    ));
    let positions = &document.structure.palette["default"].block_position_data;
    assert_eq!(positions.keys().copied().collect::<Vec<_>>(), [-5, 900]);
    // `tick_delay` absent reads as 0; other keys of a tick are kept.
    let tick = &positions[&900].tick_queue_data.as_ref().unwrap()[0];
    assert_eq!(tick.tick_delay, None);
    assert_eq!(tick.tick_delay(), 0);
    assert_eq!(tick.extra.len(), 1);
}

#[test]
fn rule_12_a_tick_delay_of_another_type_reads_as_0() {
    let document = loads(&with_palette(
        r#"block_palette: [], block_position_data: {"0": {tick_queue_data: [{tick_delay: 5s}, {tick_delay: 7}]}}"#,
        "[]",
    ));
    let ticks = document.structure.palette["default"].block_position_data[&0]
        .tick_queue_data
        .clone()
        .unwrap();
    assert_eq!(ticks[0].tick_delay(), 0);
    assert_eq!(ticks[1].tick_delay(), 7);
}

#[test]
fn rule_11_palettes_are_read_before_entities() {
    fails_with(
        &with_palette("block_position_data: {}", "[4]"),
        "structure/required",
        &required("block_palette"),
    );
}

#[test]
fn rule_13_an_empty_entity_list_loads() {
    // An empty list of Compound tags is in the `tick_block_structure` capture (round_trip.rs).
    let document = loads(&with_palette(
        "block_palette: [], block_position_data: {}",
        "[]",
    ));
    assert!(document.structure.entities.is_empty());
}

#[test]
fn rule_1_a_file_that_is_not_nbt_reads_as_an_empty_compound() {
    // `NbtIo::readOrGetEmpty`: a corrupt file is an empty root compound, which fails with the
    // `format_version` text.
    let truncated = &bytes("{format_version: 1, size: [1, 1, 1]}")[..12];
    let not_a_compound = bytes("[1, 2, 3]");
    for input in [
        &b""[..],
        &b"{\"format_version\": 1}"[..],
        truncated,
        &not_a_compound,
    ] {
        let finding = fails_with(input, "structure/not_nbt", &required("format_version"));
        assert_eq!(finding.location.json_path.to_string(), "");
    }
}

#[test]
fn bytes_after_the_root_compound_are_ignored() {
    let mut input = bytes(
        r"{format_version: 1, size: [1, 1, 1], structure_world_origin: [0, 0, 0], structure: {block_indices: [[0], [-1]], entities: [], palette: {}}}",
    );
    input.extend_from_slice(b"trailing");
    assert_eq!(loads(&input).size, [1, 1, 1]);
}

#[test]
fn a_container_tag_of_another_type_is_missing_as_vanilla_reads_it() {
    // `getList` / `getCompound` give vanilla nothing for a tag of another type, and its
    // text says "missing"; our code is `MissingRequiredField` for the absent and the mistyped
    // tag alike, at each of the seven reads.
    let origin = "structure_world_origin: [0, 0, 0],";
    let palette = "block_palette: [], block_position_data: {}";
    let cases: [(&str, Vec<u8>, &str); 7] = [
        (
            "size",
            bytes(&format!(
                "{{format_version: 1, size: \"1x1x1\", {origin} structure: {{{BODY}}}}}"
            )),
            "size",
        ),
        (
            "structure_world_origin",
            doc("structure_world_origin: {},", BODY),
            "structure_world_origin",
        ),
        (
            "structure",
            bytes(&format!(
                "{{format_version: 1, size: [1, 1, 1], {origin} structure: 1b}}"
            )),
            "structure",
        ),
        (
            "block_indices",
            doc(origin, "block_indices: {}, entities: [], palette: {}"),
            "structure/block_indices",
        ),
        (
            "palette",
            doc(
                origin,
                "block_indices: [[0], [-1]], entities: [], palette: \"default\"",
            ),
            "structure/palette",
        ),
        (
            "block_position_data",
            with_palette("block_palette: [], block_position_data: [1]", "[]"),
            "structure/palette/default/block_position_data",
        ),
        (
            "entities",
            with_palette(palette, "{}"),
            "structure/entities",
        ),
    ];
    for (field, input, at) in cases {
        let finding = fails_with(&input, "structure/required", &required(field));
        assert_eq!(finding.location.json_path.to_string(), at);
    }
    // `block_palette` of another type: rule_12 above.
}
