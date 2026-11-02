//! The envelope and version gates, the value forms and the schema, the
//! post-validation.

use bedrock_addon::diagnostic::Severity::{Error, Info};
use bedrock_addon::families::texture_set::{LayerKind, LayerValue, Rgb, Rgba, TextureName};
use bedrock_addon::version::FormatVersion;

use crate::common::finding;
use crate::{
    STONE, STONE_SET, error, file, findings, post_failed, rejected, schema_failed, set, stack,
    warning,
};

const A: &str = "textures/blocks/a";
const B: &str = "textures/blocks/b";
const PAYLOAD: &str = "minecraft:texture_set";

fn rgba(r: u8, g: u8, b: u8, a: u8) -> Rgba {
    Rgba {
        r: r.into(),
        g: g.into(),
        b: b.into(),
        a: a.into(),
    }
}

fn rgb(r: u8, g: u8, b: u8) -> Rgb {
    Rgb {
        r: r.into(),
        g: g.into(),
        b: b.into(),
    }
}

/// The path of layer `key` in the document.
fn layer(key: &str) -> String {
    format!("{PAYLOAD}/{key}")
}

/// V8a: the value fits none of the member's schema children.
fn unknown_option(at: &str, allowed: &str) -> super::Rendered {
    error(
        "InvalidType",
        at,
        &format!("unknown child schema option type.  Allowed types:{allowed}"),
    )
}

const COLOUR_TYPES: &str = " 'color' 'string'";
/// The client logs nothing of its own for an unknown member (V6); the text is ours.
const UNKNOWN: &str = "unrecognized field";
/// R4 is ours.
const CLAMPED: &str = "value clamped into its range";

// ---- 1. envelope and version gates ---------------------------------------------------------

#[test]
fn ts01_a_1_16_100_file_with_mer_loads_and_is_upgraded() {
    let stack = stack("ts01_1_16_100_mer");
    let document = set(&stack, STONE);
    assert_eq!(document.color, Some(LayerValue::Array(rgba(1, 2, 3, 4))));
    assert_eq!(
        document.metalness_emissive_roughness,
        Some(LayerValue::Array(rgb(5, 6, 7)))
    );
    // A three-channel layer has alpha 1.0.
    assert_eq!(document.uniform(LayerKind::Mer), Some(rgba(5, 6, 7, 255)));
    assert_eq!(document.metalness_emissive_roughness_subsurface, None);
    let version = document.version();
    assert_eq!(version.written, Some(FormatVersion::new(1, 16, 100)));
    assert_eq!(version.parsed_as, FormatVersion::new(1, 16, 100));
    assert!(version.upgraded);
    assert_eq!(findings(&stack), []);
}

#[test]
fn ts02_mers_in_a_1_16_100_file_is_an_unknown_member() {
    let stack = stack("ts02_1_16_100_mers_rejected");
    assert_eq!(
        rejected(&stack, STONE),
        [
            error(
                "UnrecognizedField",
                &layer("metalness_emissive_roughness_subsurface"),
                UNKNOWN
            ),
            schema_failed()
        ]
    );
}

#[test]
fn ts03_mer_is_still_a_layer_at_1_21_30() {
    let stack = stack("ts03_1_21_30_mer");
    let document = set(&stack, STONE);
    assert_eq!(
        document.metalness_emissive_roughness,
        Some(LayerValue::Array(rgb(5, 6, 7)))
    );
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 21, 30));
    assert!(!document.version().upgraded);
    assert_eq!(findings(&stack), []);
}

#[test]
fn ts04_mers_loads_at_1_21_30() {
    let stack = stack("ts04_1_21_30_mers");
    let document = set(&stack, STONE);
    assert_eq!(
        document.metalness_emissive_roughness_subsurface,
        Some(LayerValue::Array(rgba(5, 6, 7, 8)))
    );
    assert_eq!(document.uniform(LayerKind::Mers), Some(rgba(5, 6, 7, 8)));
    assert_eq!(findings(&stack), []);
}

#[test]
fn ts05_mer_and_mers_together_fail_the_post_validation() {
    let stack = stack("ts05_1_21_30_mer_and_mers");
    assert_eq!(
        rejected(&stack, STONE),
        [
            error(
                "InvalidValue",
                PAYLOAD,
                "TextureSet contains both `metalness_emissive_roughness` AND `metalness_emissive_roughness_subsurface` layers - this is not supported, and one should be removed"
            ),
            post_failed()
        ]
    );
}

/// V3 with the client's text for the file.
fn invalid_version() -> super::Rendered {
    warning(
        "MalformedFormatVersion",
        "format_version",
        &format!("Invalid 'format_version' in JSON file ({STONE_SET})"),
    )
}

#[test]
fn ts06_a_version_that_is_no_version_is_invalid() {
    let stack = stack("ts06_invalid_version");
    assert_eq!(rejected(&stack, STONE), [invalid_version()]);
}

#[test]
fn ts07_a_missing_version_is_invalid() {
    let stack = stack("ts07_missing_version");
    assert_eq!(rejected(&stack, STONE), [invalid_version()]);
}

#[test]
fn ts08_the_array_form_selects_the_schema_and_fails_it() {
    let stack = stack("ts08_version_array");
    assert_eq!(
        rejected(&stack, STONE),
        [
            unknown_option("format_version", " 'version'"),
            schema_failed()
        ]
    );
}

/// V4 for the file of texture `texture` and the version as the client prints it.
fn unsupported(texture: &str, version: &str) -> super::Rendered {
    warning(
        "UnsupportedVersion",
        "format_version",
        &format!(
            "Unsupported version value in \"format_version\" field: {version} ({texture}.texture_set.json)"
        ),
    )
}

#[test]
fn ts09_a_version_between_the_two_is_unsupported() {
    let stack = stack("ts09_between_versions");
    assert_eq!(rejected(&stack, A), [unsupported(A, "1.16.200")]);
    assert_eq!(rejected(&stack, B), [unsupported(B, "1.21.0")]);
}

#[test]
fn ts10_a_version_above_1_21_30_is_unsupported_whatever_the_engine() {
    let stack = stack("ts10_above_max");
    assert_eq!(rejected(&stack, A), [unsupported(A, "1.21.40")]);
    assert_eq!(rejected(&stack, B), [unsupported(B, "1.26.0")]);
}

#[test]
fn ts11_a_version_below_the_floor_is_unsupported() {
    let stack = stack("ts11_below_floor");
    assert_eq!(rejected(&stack, STONE), [unsupported(STONE, "1.16.0")]);
}

#[test]
fn ts12_a_prerelease_tag_and_beta_are_unsupported() {
    let stack = stack("ts12_prerelease");
    assert_eq!(rejected(&stack, A), [unsupported(A, "1.21.30-beta.1")]);
    // `"beta"` is a version to the client's parser, and no schema's.
    assert_eq!(rejected(&stack, B), [unsupported(B, "9999.9999.9999-beta")]);
}

#[test]
fn ts13_a_file_that_is_no_json_fails_to_parse() {
    let stack = stack("ts13_parse_error");
    let unclosed = rejected(&stack, A);
    assert_eq!(unclosed.len(), 1, "{unclosed:?}");
    assert_eq!(
        unclosed[0],
        warning(
            "MalformedInput",
            "",
            &format!("Failed to parse texture set JSON file ({A}.texture_set.json)")
        )
    );
}

// The row lists "trailing garbage" as a parse failure; the client's reader stops at
// the end of the root value (the same `Json::Reader` as the atlases).
#[test]
fn ts13_content_after_the_root_value_is_ignored_with_an_information() {
    let stack = stack("ts13_parse_error");
    let document = set(&stack, B);
    assert!(document.color.is_some());
    let found = super::common::rendered(document.diagnostics().iter());
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!((found[0].0, found[0].1), ("MalformedInput", Info));
}

#[test]
fn ts14_comments_are_allowed() {
    let stack = stack("ts14_comments");
    assert_eq!(
        set(&stack, STONE).color,
        Some(LayerValue::Array(rgba(1, 2, 3, 4)))
    );
    assert_eq!(findings(&stack), []);
}

// `vanilla = "debug assert only"`: the client asserts and shows the missing texture.
#[test]
fn ts15_an_empty_file_is_the_assert_s_information_and_the_image_beside_it_is_no_fallback_deviation()
{
    // The `handleAssert` of `loadOrCreateTextureSetDefinition`: an information with
    // its text; the set is not loaded.
    let stack = stack("ts15_empty_file");
    assert_eq!(
        rejected(&stack, STONE),
        [(
            "EmptyDocument",
            Info,
            String::new(),
            "load function reported OK, but storage is empty - is the file empty? Something went wrong.".to_owned()
        )]
    );
    // `missing` asserted that the png is not used.
    let file = file(&stack, STONE);
    assert_eq!(file.stack_index, 0);
}

#[test]
fn ts16_another_root_member_is_an_unknown_member() {
    let stack = stack("ts16_extra_root_member");
    assert_eq!(
        rejected(&stack, STONE),
        [
            error("UnrecognizedField", "description", UNKNOWN),
            schema_failed()
        ]
    );
}

#[test]
fn ts17_build_metadata_takes_no_part_in_the_match() {
    let stack = stack("ts17_build_metadata");
    let document = set(&stack, STONE);
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 21, 30));
    assert_eq!(findings(&stack), []);
}

// ---- 2. value forms and schema ---------------------------------------------------------------

#[test]
fn ts20_hex_colours_are_alpha_first() {
    let stack = stack("ts20_color_hex8");
    for texture in [A, B] {
        let document = set(&stack, texture);
        assert_eq!(
            document.color,
            Some(LayerValue::Hex(rgba(0xBB, 0xCC, 0xDD, 0xAA))),
            "{texture}"
        );
        assert_eq!(
            document.metalness_emissive_roughness,
            Some(LayerValue::Hex(rgb(0xAA, 0xBB, 0xCC)))
        );
    }
    assert_eq!(findings(&stack), []);
}

#[test]
fn ts21_hex_digits_are_read_in_any_case() {
    let stack = stack("ts21_hex_case");
    assert_eq!(set(&stack, A).value(), set(&stack, B).value());
    assert_eq!(
        set(&stack, A).color,
        Some(LayerValue::Hex(rgba(0xBB, 0xCC, 0xDD, 0xAA)))
    );
}

const HEX_8: &str = "Color Value was not in the correct format. Please use a hex value with 8 characters. EX. #FFFFFFFF";
const HEX_6: &str = "Color Value was not in the correct format as Alpha is ignored for these colors. Please use a hex value with 6 characters. EX. #FFFFFF";

#[test]
fn ts22_six_digits_on_color_are_rejected() {
    let stack = stack("ts22_color_hex6_rejected");
    assert_eq!(
        rejected(&stack, STONE),
        [
            error("InvalidValue", &layer("color"), HEX_8),
            schema_failed()
        ]
    );
}

#[test]
fn ts23_eight_digits_on_mer_are_rejected() {
    let stack = stack("ts23_mer_hex8_rejected");
    assert_eq!(
        rejected(&stack, STONE),
        [
            error(
                "InvalidValue",
                &layer("metalness_emissive_roughness"),
                HEX_6
            ),
            schema_failed()
        ]
    );
}

#[test]
fn ts24_the_first_byte_of_a_mers_hex_is_the_fourth_channel() {
    let stack = stack("ts24_mers_hex8");
    let document = set(&stack, STONE);
    assert_eq!(
        document.color,
        Some(LayerValue::Hex(rgba(0x22, 0x33, 0x44, 0x11)))
    );
    assert_eq!(
        document.metalness_emissive_roughness_subsurface,
        Some(LayerValue::Hex(rgba(0xBB, 0xCC, 0xDD, 0xAA)))
    );
}

#[test]
fn ts25_three_elements_on_color_are_rejected() {
    let stack = stack("ts25_color_array3_rejected");
    assert_eq!(
        rejected(&stack, STONE),
        [
            unknown_option(&layer("color"), COLOUR_TYPES),
            schema_failed()
        ]
    );
}

#[test]
fn ts26_four_elements_on_mer_are_rejected() {
    let stack = stack("ts26_mer_array4_rejected");
    assert_eq!(
        rejected(&stack, STONE),
        [
            unknown_option(&layer("metalness_emissive_roughness"), COLOUR_TYPES),
            schema_failed()
        ]
    );
}

// `vanilla = "Error-level log, file still loads"` (V8e); R4 is ours.
#[test]
fn ts27_a_real_is_255_and_a_value_outside_the_range_is_clamped_deviation() {
    let stack = stack("ts27_array_floats_and_range");
    let document = set(&stack, STONE);
    // V8e: `parseValue<mce::Color>` logs it at the content log's error level and the set loads:
    // a kept error.
    assert!(
        document
            .diagnostics()
            .iter()
            .filter(|finding| finding.code.name() == "InvalidValue")
            .all(|finding| finding.kept && finding.severity == Error)
    );
    assert_eq!(
        document.color,
        Some(LayerValue::Array(rgba(255, 255, 0, 255)))
    );
    assert_eq!(
        findings(&stack),
        [
            finding(
                "InvalidValue",
                Error,
                &format!("{}/0", layer("color")),
                "Expected Integral Number."
            ),
            finding(
                "ClampedValue",
                Info,
                &format!("{}/1", layer("color")),
                CLAMPED
            ),
            finding(
                "ClampedValue",
                Info,
                &format!("{}/2", layer("color")),
                CLAMPED
            ),
        ]
    );
}

#[test]
fn r4_both_ends_of_the_range_are_inside_it() {
    let text = br#"{ "format_version": "1.21.30", "minecraft:texture_set": { "color": [255, 256, 0, -1] } }"#;
    let files: &[(&str, &[u8])] = &[(STONE_SET, text)];
    let stack = super::common::stack_of_files(&[files]);
    assert_eq!(
        set(&stack, STONE).color,
        Some(LayerValue::Array(rgba(255, 255, 0, 0)))
    );
    assert_eq!(
        findings(&stack),
        [
            finding(
                "ClampedValue",
                Info,
                &format!("{}/1", layer("color")),
                CLAMPED
            ),
            finding(
                "ClampedValue",
                Info,
                &format!("{}/3", layer("color")),
                CLAMPED
            ),
        ]
    );
}

#[test]
fn ts28_an_element_that_is_no_number_is_rejected() {
    let stack = stack("ts28_array_non_number");
    assert_eq!(
        rejected(&stack, STONE),
        [
            error(
                "InvalidType",
                &format!("{}/0", layer("color")),
                "wrong JSON type"
            ),
            schema_failed()
        ]
    );
}

#[test]
fn ts29_a_hex_string_on_normal_is_rejected() {
    let stack = stack("ts29_normal_hex_rejected");
    assert_eq!(
        rejected(&stack, STONE),
        [
            unknown_option(&layer("normal"), " 'string'"),
            schema_failed()
        ]
    );
}

#[test]
fn ts30_a_uniform_heightmap_is_rejected() {
    let stack = stack("ts30_heightmap_uniform_rejected");
    for texture in [A, B] {
        assert_eq!(
            rejected(&stack, texture),
            [
                unknown_option(&layer("heightmap"), " 'string'"),
                schema_failed()
            ],
            "{texture}"
        );
    }
}

// `vanilla = "loads with colour (0,0,0,0) and logs Expected [ r, g, b ] or [r, g, b, a] or
// \"#RRGGBB\"."`: ours rejects the object form.
#[test]
fn ts31_an_object_colour_is_rejected_deviation() {
    let stack = stack("ts31_object_color_rejected");
    assert_eq!(
        rejected(&stack, STONE),
        [
            error(
                "InvalidType",
                &layer("color"),
                "Expected [ r, g, b ] or [r, g, b, a] or \"#RRGGBB\"."
            ),
            schema_failed()
        ]
    );
}

#[test]
fn ts32_the_single_channel_keys_are_unknown_members() {
    let stack = stack("ts32_single_channel_keys");
    assert_eq!(
        rejected(&stack, STONE),
        [
            error("UnrecognizedField", &layer("metalness"), UNKNOWN),
            error("UnrecognizedField", &layer("emissive"), UNKNOWN),
            error("UnrecognizedField", &layer("roughness"), UNKNOWN),
            schema_failed()
        ]
    );
}

#[test]
fn ts33_a_null_layer_fails_the_schema() {
    let stack = stack("ts33_null_layer");
    assert_eq!(
        rejected(&stack, STONE),
        [
            unknown_option(&layer("normal"), " 'string'"),
            schema_failed()
        ]
    );
}

// ---- 3. post-validation ----------------------------------------------------------------------

#[test]
fn ts40_a_set_without_color_fails_before_the_other_rules() {
    let stack = stack("ts40_no_color");
    // The file also has `normal` and `heightmap` together (V10): V9b is checked first.
    assert_eq!(
        rejected(&stack, STONE),
        [
            error(
                "InvalidValue",
                PAYLOAD,
                "There must at least be a color layer in any TextureSet"
            ),
            post_failed()
        ]
    );
}

#[test]
fn ts41_heightmap_and_normal_together_fail() {
    let stack = stack("ts41_heightmap_and_normal");
    assert_eq!(
        rejected(&stack, STONE),
        [
            error(
                "InvalidValue",
                PAYLOAD,
                "TextureSet contains both `heightmap` AND `normal` layers - this is not currently supported, and one should be removed"
            ),
            post_failed()
        ]
    );
}

// The file is `resource_pack/textures/flame_atlas.texture_set.json`,
// copied unchanged: the one vanilla set with a uniform layer. The census of all 3130 files is
// the smoke test (crates/addon/std/tests/smoke_texture_set.rs).
#[test]
fn ts42_the_vanilla_file_with_a_uniform_layer_loads() {
    let stack = stack("ts42_vanilla_census");
    let document = set(&stack, "textures/flame_atlas");
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 21, 30));
    assert_eq!(
        document.color,
        Some(LayerValue::Texture(TextureName("flame_atlas".to_owned())))
    );
    assert_eq!(
        document.uniform(LayerKind::Mers),
        Some(rgba(0, 255, 255, 0))
    );
    assert_eq!(findings(&stack), []);
}
