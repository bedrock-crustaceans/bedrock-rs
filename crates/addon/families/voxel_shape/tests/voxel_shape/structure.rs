//! What vanilla's schema loader rejects ("Puv failure → file skipped"): a version outside the window, a missing required member, a
//! wrong type, a constraint violation and an unknown member, each followed by "Failed to load
//! voxel shape file <path>".

use bedrock_addon_core::document::Content;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::version::{ENGINE_VERSION, FormatVersion};
use bedrock_addon_family_voxel_shape::VoxelShape;

use super::common::*;

const PAYLOAD: &str = "minecraft:voxel_shape";

#[test]
fn a_file_the_schema_loader_rejects_ends_with_the_failed_to_load_line() {
    let above = FormatVersion::new(
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch + 10,
    );
    let cases = [
        (
            slab().replace("1.21.110", "1.21.100"),
            error(
                "UnsupportedVersion",
                "unsupported version: no parser available for version 1.21.100",
                "format_version",
            ),
        ),
        (
            slab().replace("1.21.110", &above.to_string()),
            error(
                "UnsupportedVersion",
                &format!("unsupported version: no parser available for version {above}"),
                "format_version",
            ),
        ),
        (
            slab().replace(r#""format_version": "1.21.110", "#, ""),
            error(
                "MissingFormatVersion",
                "no valid version passed by the caller or found in the input",
                "",
            ),
        ),
        (
            slab().replace(r#""1.21.110""#, r#""abc""#),
            error(
                "MalformedFormatVersion",
                "Malformed format version",
                "format_version",
            ),
        ),
        (
            slab().replacen('{', r#"{"extra": 1, "#, 1),
            error(
                "MalformedInput",
                "A Minecraft Document must have exactly two members: \"format_version\" and a payload member",
                "",
            ),
        ),
    ];
    for (text, expected) in cases {
        assert_eq!(errors(&rejects(&text)), [expected, failed()], "{text}");
    }
}

#[test]
fn the_failed_to_load_line_names_the_file_s_path_in_the_pack() {
    let rejected = VoxelShape::parse(
        &PackPath::from_static("shapes/sub/too_many_boxes.json"),
        slab().replace("1.21.110", "1.21.100").as_bytes(),
        &user_pack(),
    )
    .unwrap_err();
    let closing = with_code(&rejected.diagnostics, "DocumentRejected");
    assert_eq!(closing.len(), 1);
    assert_eq!(
        closing[0].code.to_string(),
        "Failed to load voxel shape file shapes/sub/too_many_boxes.json"
    );
    assert_eq!(
        closing[0].location.path.as_ref().map(PackPath::as_str),
        Some("shapes/sub/too_many_boxes.json")
    );
}

#[test]
fn every_member_is_required() {
    let slab = slab();
    let cases = [
        (
            r#""description": {"identifier": "mypack:slab_bottom"}, "#,
            "",
            "description",
        ),
        (
            r#""identifier": "mypack:slab_bottom""#,
            "",
            "description/identifier",
        ),
        (
            r#", "shape": {"boxes": [{"min": [0, 0, 0], "max": [16, 8, 16]}]}"#,
            "",
            "shape",
        ),
        (
            r#""boxes": [{"min": [0, 0, 0], "max": [16, 8, 16]}]"#,
            "",
            "shape/boxes",
        ),
        (r#""min": [0, 0, 0], "#, "", "shape/boxes/0/min"),
        (r#", "max": [16, 8, 16]"#, "", "shape/boxes/0/max"),
    ];
    for (member, without, path) in cases {
        assert!(slab.contains(member), "{member}");
        let text = slab.replace(member, without);
        assert_eq!(
            kinds(&rejects(&text)),
            [
                kind("MissingRequiredField", &format!("{PAYLOAD}/{path}")),
                kind("DocumentRejected", ""),
            ],
            "{text}"
        );
    }
}

#[test]
fn a_member_of_the_wrong_type_fails_the_file() {
    let slab = slab();
    let cases = [
        (
            r#"{"identifier": "mypack:slab_bottom"}"#,
            "[]",
            "description",
        ),
        (r#""mypack:slab_bottom""#, "5", "description/identifier"),
        (r#""mypack:slab_bottom""#, "null", "description/identifier"),
        (
            r#"{"boxes": [{"min": [0, 0, 0], "max": [16, 8, 16]}]}"#,
            "[]",
            "shape",
        ),
        (
            r#"[{"min": [0, 0, 0], "max": [16, 8, 16]}]"#,
            "{}",
            "shape/boxes",
        ),
        (
            r#"{"min": [0, 0, 0], "max": [16, 8, 16]}"#,
            "7",
            "shape/boxes/0",
        ),
    ];
    for (member, instead, path) in cases {
        assert!(slab.contains(member), "{member}");
        let text = slab.replace(member, instead);
        assert_eq!(
            kinds(&rejects(&text)),
            [
                kind("InvalidType", &format!("{PAYLOAD}/{path}")),
                kind("DocumentRejected", ""),
            ],
            "{text}"
        );
    }
}

#[test]
fn a_payload_that_is_no_object_fails_the_file() {
    // The typed reader names no member for the payload itself, so the code is the reader's.
    let text = r#"{"format_version": "1.21.110", "minecraft:voxel_shape": 5}"#;
    assert_eq!(
        kinds(&rejects(text)),
        [
            kind("MalformedInput", PAYLOAD),
            kind("DocumentRejected", "")
        ]
    );
}

#[test]
fn an_unreadable_file_fails_with_the_failed_to_load_line() {
    // Not JSON, at a version the envelope is checked for.
    let d = rejects(r#"{"format_version": "1.21.110", "minecraft:voxel_shape": "#);
    assert_eq!(
        kinds(&d),
        [kind("MalformedInput", ""), kind("DocumentRejected", "")]
    );
    assert!(
        d.iter()
            .next()
            .unwrap()
            .code
            .to_string()
            .starts_with("Failed to parse the Minecraft Document JSON: ")
    );
    // An empty file.
    assert_eq!(
        kinds(&rejects("")),
        [kind("EmptyDocument", ""), kind("DocumentRejected", "")]
    );
    // Not UTF-8.
    let rejected = VoxelShape::parse(
        &PackPath::from_static("shapes/test.json"),
        b"\xff\xfe{}",
        &user_pack(),
    )
    .unwrap_err();
    assert_eq!(
        kinds(&rejected.diagnostics),
        [kind("MalformedInput", ""), kind("DocumentRejected", "")]
    );
}

#[test]
fn a_loaded_file_has_no_failed_to_load_line() {
    let slab = loads(&slab());
    assert!(slab.diagnostics().is_empty());
}

/// Vanilla's text of a member no schema type binds.
const NOT_IN_SCHEMA: &str = "this member was found in the input, but is not present in the Schema";

/// The finding of the unknown member at `path` of the payload.
fn unknown(path: &str) -> (&'static str, String, String) {
    error(
        "UnrecognizedField",
        NOT_IN_SCHEMA,
        &format!("{PAYLOAD}/{path}"),
    )
}

#[test]
fn vs03_an_unknown_member_fails_the_file() {
    let text = slab().replace(r#""shape": {"#, r#""shape": {"origin": [0, 0, 0], "#);
    assert_eq!(errors(&rejects(&text)), [unknown("shape/origin"), failed()]);
}

#[test]
fn an_unknown_member_fails_the_file_at_every_level() {
    let slab = slab();
    let in_payload = slab.replace(r#""description": {"#, r#""extra": 1, "description": {"#);
    let in_description = slab.replace(r#""identifier":"#, r#""name": "x", "identifier":"#);
    let in_box = slab.replace(
        r#""min": [0, 0, 0]"#,
        r#""min": [0, 0, 0], "mid": [8, 4, 8]"#,
    );
    assert_eq!(errors(&rejects(&in_payload)), [unknown("extra"), failed()]);
    assert_eq!(
        errors(&rejects(&in_description)),
        [unknown("description/name"), failed()]
    );
    assert_eq!(
        errors(&rejects(&in_box)),
        [unknown("shape/boxes/0/mid"), failed()]
    );
    // Every one of them is reported, in document order.
    let everywhere = slab
        .replace(r#""description": {"#, r#""extra": 1, "description": {"#)
        .replace(r#""identifier":"#, r#""name": "x", "identifier":"#)
        .replace(r#""shape": {"#, r#""shape": {"origin": [0, 0, 0], "#)
        .replace(
            r#""min": [0, 0, 0]"#,
            r#""min": [0, 0, 0], "mid": [8, 4, 8]"#,
        );
    assert_eq!(
        errors(&rejects(&everywhere)),
        [
            unknown("extra"),
            unknown("description/name"),
            unknown("shape/origin"),
            unknown("shape/boxes/0/mid"),
            failed()
        ]
    );
}
