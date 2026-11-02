//! The rows about one file that loads, and the version
//! window: vs01, vs04, vs07, vs12, vs16, vs18, vs24b, vs25.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::version::{ENGINE_VERSION, FormatVersion};
use bedrock_addon_family_voxel_shape::ShapeBox;

use super::common::*;

const V1_21_110: FormatVersion = FormatVersion::new(1, 21, 110);

#[test]
fn vs01_the_vanilla_anvil_loads_from_a_base_game_pack() {
    // The vanilla 1.26.50 `behavior_pack/shapes/anvil.json`, unchanged.
    let text = fixture("vs01_anvil_base_game/shapes/anvil.json");
    let anvil = loads_in(&text, &base_game());
    assert_eq!(anvil.description.identifier.as_str(), "minecraft:anvil");
    assert_eq!(anvil.shape.boxes.len(), 7);
    assert_eq!(
        anvil.shape.boxes[0],
        ShapeBox::new([2.0, 0.0, 2.0], [14.0, 4.0, 14.0])
    );
    assert_eq!(
        anvil.shape.boxes[6],
        ShapeBox::new([10.0, 10.0, 4.0], [13.0, 16.0, 12.0])
    );
    assert_eq!(anvil.source_format_version, V1_21_110);
}

#[test]
fn vs04_the_docs_slab_loads_from_a_user_pack() {
    let slab = loads(&slab());
    assert_eq!(slab.description.identifier.as_str(), "mypack:slab_bottom");
    assert_eq!(
        slab.shape.boxes,
        [ShapeBox::new([0.0, 0.0, 0.0], [16.0, 8.0, 16.0])]
    );
    assert_eq!(slab.source_format_version, V1_21_110);
    let version = slab.version();
    assert_eq!(version.written, Some(V1_21_110));
    assert_eq!(version.parsed_as, V1_21_110);
    assert!(!version.upgraded && !version.beta);
    assert_eq!(slab.origin().path.as_str(), "shapes/test.json");
}

#[test]
fn vs07_a_bare_name_loads_from_a_user_pack() {
    let half = loads(&shape("half_block", &[("[0, 0, 0]", "[16, 8, 16]")]));
    assert_eq!(half.description.identifier.as_str(), "half_block");
}

#[test]
fn vs12_a_box_at_the_exact_bounds_loads() {
    // min.x = -14 is the lowest coordinate; the shape then spans exactly 30 pixels on x.
    let at_bounds = loads(&shape(
        "test:at_error_bounds",
        &[("[-14, 0, 0]", "[16, 16, 16]")],
    ));
    assert_eq!(at_bounds.shape.boxes[0].min, [-14.0, 0.0, 0.0]);
}

#[test]
fn vs16_an_overlap_of_less_than_a_pixel_is_enough() {
    let minimal = loads(&shape(
        "test:minimal_overlap",
        &[("[15.5, 0, 0]", "[20, 16, 16]")],
    ));
    assert_eq!(minimal.shape.boxes[0].min, [15.5, 0.0, 0.0]);
}

#[test]
fn vs18_an_extent_of_exactly_30_loads() {
    let widest = loads(&shape(
        "test:maximum_extent",
        &[("[0, 0, 0]", "[30, 30, 30]")],
    ));
    assert_eq!(widest.shape.boxes[0].max, [30.0, 30.0, 30.0]);
}

#[test]
fn vs24b_a_format_version_that_is_no_version_is_malformed() {
    // A number is a version on the document input (`1.21` is 1.21.0: `envelope.rs`).
    let unreadable = [
        r#""abc""#,
        "true",
        "null",
        r#"{"major": 1}"#,
        "[1, 21]",
        "[1, 21, 110, 0]",
        "[1, -21, 110]",
        r#"["1", 21, 110]"#,
    ];
    for version in unreadable {
        let text = slab().replace(r#""1.21.110""#, version);
        assert_eq!(
            errors(&rejects(&text)),
            [
                error(
                    "MalformedFormatVersion",
                    "Malformed format version",
                    "format_version"
                ),
                failed()
            ],
            "{version}"
        );
    }
}

#[test]
fn vs24b_an_array_of_three_integers_is_a_version() {
    let text = slab().replace(r#""1.21.110""#, "[1, 21, 110]");
    let slab = loads(&text);
    assert_eq!(slab.version().written, Some(V1_21_110));
    assert_eq!(slab.source_format_version, V1_21_110);
}

#[test]
fn vs24b_a_third_root_member_is_an_envelope_error() {
    let text = slab().replacen('{', r#"{"extra": 1, "#, 1);
    assert_eq!(
        errors(&rejects(&text)),
        [
            error(
                "MalformedInput",
                "A Minecraft Document must have exactly two members: \"format_version\" and a payload member",
                ""
            ),
            failed()
        ]
    );
}

#[test]
fn vs25_a_version_above_the_engine_has_no_parser() {
    let above = FormatVersion::new(
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch + 10,
    );
    let text = slab().replace("1.21.110", &above.to_string());
    assert_eq!(
        errors(&rejects(&text)),
        [
            error(
                "UnsupportedVersion",
                &format!("unsupported version: no parser available for version {above}"),
                "format_version"
            ),
            failed()
        ]
    );
}

#[test]
fn the_engine_version_is_the_last_one_read() {
    let text = slab().replace("1.21.110", &ENGINE_VERSION.to_string());
    let slab = loads(&text);
    assert_eq!(slab.source_format_version, ENGINE_VERSION);
    assert_eq!(slab.version().parsed_as, V1_21_110);
    assert!(!slab.version().upgraded);
}

#[test]
fn a_pre_release_above_the_floor_loads_with_an_info_and_no_closing_line() {
    // Mutation pass: a parse finding below error level must not end the file with
    // "Failed to load".
    let text = slab().replace("1.21.110", "1.21.120-beta.3");
    let shape = load(&text, &user_pack()).unwrap();
    let found: Vec<_> = shape
        .diagnostics()
        .iter()
        .map(|f| (f.code.name(), f.severity))
        .collect();
    assert_eq!(found, [("PrereleaseFormatVersion", Severity::Info)]);
}
