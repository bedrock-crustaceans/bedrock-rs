//! The 1.26.40+ reader of 1.26.60: one slice at exactly 1.26.40 under the strict mask `0xfffc`, read from the
//! game's document input. Whatever that reader refuses (M2) makes the piece invalid: the
//! meta file is rejected with the reader's findings and the game's "Could not parse metafile
//! json file" line.

use bedrock_addon::families::persona::PersonaPieces;

use crate::common::{hat, refused_at};

const FILE: &str = "pieces/hat/hat.meta.json";

/// The members every valid meta file of the tests has.
const REQUIRED: &str = r#""piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_hair""#;

/// A meta file with the required members and `members`.
fn meta_with(members: &str) -> String {
    let comma = if members.is_empty() { "" } else { ", " };
    format!("{{ {REQUIRED}{comma}{members} }}")
}

/// The piece `pieces/hat` with the meta file `text`, loaded from a piece pack.
fn load(text: &str) -> PersonaPieces {
    hat(text, &[("hat.geometry.json", r#"{ "g": {} }"#)])
}

/// The error findings of the one rejected meta file of `pieces` ([`refused_at`]).
fn refused(pieces: &PersonaPieces) -> Vec<String> {
    refused_at(pieces, FILE)
}

/// `refused` for the meta file with the required members and `members`.
fn refused_with(members: &str) -> Vec<String> {
    refused(&load(&meta_with(members)))
}

#[test]
fn a_meta_file_with_the_required_members_loads() {
    let pieces = load(&meta_with(""));
    assert_eq!(pieces.pieces.len(), 1, "{:?}", pieces.rejected);
}

#[test]
fn an_unknown_member_invalidates_the_piece() {
    assert_eq!(
        refused_with(r#""hat_size": 3"#),
        ["error UnrecognizedField hat_size"]
    );
}

#[test]
fn color_swatches_is_no_member_from_1_26_40() {
    // the string is not in the game; M3.
    assert_eq!(
        refused_with(r#""color_swatches": ["primary"]"#),
        ["error UnrecognizedField color_swatches"]
    );
}

#[test]
fn an_unknown_member_of_an_entry_invalidates_the_piece() {
    for (members, path) in [
        (
            r#""texture_sources": [{ "texture": "t", "tint": "m" }]"#,
            "texture_sources/0/tint",
        ),
        (
            r#""geometry_sources": [{ "geometry": "g", "use_face_uv": true }]"#,
            "geometry_sources/0/use_face_uv",
        ),
        (
            r#""animation_sources": [{ "name": "n", "animationFile": "f", "animation_file": "f" }]"#,
            "animation_sources/0/animation_file",
        ),
        (
            r##""tint_color": { "r_color": "#ffffff", "z_color": "#000000" }"##,
            "tint_color/z_color",
        ),
    ] {
        assert_eq!(
            refused_with(members),
            [format!("error UnrecognizedField {path}")],
            "{members}"
        );
    }
}

#[test]
fn a_name_outside_the_bound_enum_invalidates_the_piece() {
    // E1 `invalid enum value` (0x1000): an error in both masks.
    for (members, path) in [
        (r#""piece_sub_type": "persona_hat""#, "piece_sub_type"),
        (r#""piece_sub_type": """#, "piece_sub_type"),
        (r#""body_variant": "huge""#, "body_variant"),
        (
            r#""geometry_sources": [{ "geometry": "g", "arm_size": "thin" }]"#,
            "geometry_sources/0/arm_size",
        ),
        (
            r#""geometry_sources": [{ "geometry": "g", "body_size": "huge" }]"#,
            "geometry_sources/0/body_size",
        ),
        (
            r#""geometry_sources": [{ "geometry": "g", "side": "up" }]"#,
            "geometry_sources/0/side",
        ),
    ] {
        assert_eq!(
            refused_with(members),
            [format!("error InvalidEnumValue {path}")],
            "{members}"
        );
    }
}

#[test]
fn a_piece_type_outside_the_bound_enum_invalidates_the_piece() {
    // The four side types are internal-only from 1.26.40 (the binder's help text, M4), and
    // neither the empty text nor the name the game prints for an unknown type is bound.
    for name in [
        "persona_hat",
        "",
        "persona_unknown",
        "persona_left_leg",
        "persona_right_leg",
        "persona_left_arm",
        "persona_right_arm",
    ] {
        let text = format!(
            r#"{{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "{name}" }}"#
        );
        assert_eq!(
            refused(&load(&text)),
            ["error InvalidEnumValue piece_type"],
            "{name}"
        );
    }
}

#[test]
fn persona_coco_and_the_empty_sizes_load() {
    let text = r#"{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_coco", "body_variant": "", "geometry_sources": [{ "geometry": "g", "arm_size": "", "body_size": "" }] }"#;
    let pieces = load(text);
    assert_eq!(pieces.pieces.len(), 1, "{:?}", pieces.rejected);
}

#[test]
fn a_member_of_another_json_type_invalidates_the_piece() {
    // `0x8`, an error under 0xfffc (Revision 1).
    for (members, path) in [
        (r#""allow_tint_override": "yes""#, "allow_tint_override"),
        (r#""clothing_map": 3"#, "clothing_map"),
        (r#""zone": 3"#, "zone"),
        (r#""zone": ["head", 3]"#, "zone/1"),
        (r#""texture_sources": {}"#, "texture_sources"),
        (r#""texture_sources": [7]"#, "texture_sources/0"),
        (r#""tint_color": 5"#, "tint_color"),
        (
            r#""tint_base_color": { "r_color": 1 }"#,
            "tint_base_color/r_color",
        ),
        (
            r#""texture_sources": [{ "texture": "t", "frames": 2.5 }]"#,
            "texture_sources/0/frames",
        ),
        (
            r#""texture_sources": [{ "texture": "t", "frames": 4294967296 }]"#,
            "texture_sources/0/frames",
        ),
        (
            r#""texture_sources": [{ "texture": "t", "animated": "true" }]"#,
            "texture_sources/0/animated",
        ),
        (
            r#""geometry_sources": [{ "geometry": "g", "arm_size": 1 }]"#,
            "geometry_sources/0/arm_size",
        ),
        // 1.26.40 types `is_face` as a string (`optional<std::string>`): a boolean is no string.
        (
            r#""geometry_sources": [{ "geometry": "g", "is_face": true }]"#,
            "geometry_sources/0/is_face",
        ),
    ] {
        assert_eq!(
            refused_with(members),
            [format!("error InvalidType {path}")],
            "{members}"
        );
    }
}

#[test]
fn a_null_member_invalidates_the_piece() {
    // Rule N2: `null` is present and fails its type with `0x8`.
    for (members, path) in [
        (r#""piece_sub_type": null"#, "piece_sub_type"),
        (r#""zone": null"#, "zone"),
        (r#""tint_color": null"#, "tint_color"),
        (r#""allow_tint_override": null"#, "allow_tint_override"),
        (
            r#""texture_sources": [{ "texture": "t", "tint_map": null }]"#,
            "texture_sources/0/tint_map",
        ),
        (r#""tint_color": { "r_color": null }"#, "tint_color/r_color"),
    ] {
        assert_eq!(
            refused_with(members),
            [format!("error InvalidType {path}")],
            "{members}"
        );
    }
    let text = r#"{ "piece_id": null, "piece_name": "hat", "piece_type": "persona_hair" }"#;
    assert_eq!(refused(&load(text)), ["error InvalidType piece_id"]);
}

#[test]
fn a_missing_required_member_invalidates_the_piece() {
    // D4 `missing required field` (0x80).
    for (text, path) in [
        (
            r#"{ "piece_name": "hat", "piece_type": "persona_hair" }"#,
            "piece_id",
        ),
        (
            r#"{ "piece_id": "x", "piece_type": "persona_hair" }"#,
            "piece_name",
        ),
        (r#"{ "piece_id": "x", "piece_name": "hat" }"#, "piece_type"),
    ] {
        assert_eq!(
            refused(&load(text)),
            [format!("error MissingRequiredField {path}")],
            "{text}"
        );
    }
    for (members, path) in [
        (r#""texture_sources": [{}]"#, "texture_sources/0/texture"),
        (
            r#""geometry_sources": [{ "arm_size": "slim" }]"#,
            "geometry_sources/0/geometry",
        ),
        (
            r#""animation_sources": [{ "name": "n" }]"#,
            "animation_sources/0/animationFile",
        ),
        (
            r#""animation_sources": [{ "animationFile": "f" }]"#,
            "animation_sources/0/name",
        ),
    ] {
        assert_eq!(
            refused_with(members),
            [format!("error MissingRequiredField {path}")],
            "{members}"
        );
    }
}

#[test]
fn an_empty_texture_is_a_texture() {
    // The member is there; the texture builder skips the entry (M17, an information).
    let pieces = load(&meta_with(r#""texture_sources": [{ "texture": "" }]"#));
    assert_eq!(pieces.pieces.len(), 1, "{:?}", pieces.rejected);
}
