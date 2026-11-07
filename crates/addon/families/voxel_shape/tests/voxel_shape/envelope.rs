//! The version window and the envelope: vs02 and the version-window rows.

use bedrock_addon_core::diagnostic::{Kind, Severity};

use super::common::*;

const NO_PARSER_1_21_100: &str = "unsupported version: no parser available for version 1.21.100";

#[test]
fn vs02_below_1_21_110_there_is_no_parser() {
    let text = slab().replace("1.21.110", "1.21.100");
    assert_eq!(
        errors(&rejects(&text)),
        [
            error(
                "core/no_parser_for_version",
                NO_PARSER_1_21_100,
                "format_version"
            ),
            failed()
        ]
    );
}

#[test]
fn a_pre_release_of_the_floor_is_below_it_and_named_as_written() {
    let text = slab().replace("1.21.110", "1.21.110-beta.2");
    let d = rejects(&text);
    let found = with_code(&d, "core/no_parser_for_version");
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(
        found[0].code.to_string(),
        "unsupported version: no parser available for version 1.21.110-beta.2"
    );
}

#[test]
fn an_array_version_below_the_floor_is_named_in_dotted_form() {
    let text = slab().replace(r#""1.21.110""#, "[1, 21, 100]");
    let d = rejects(&text);
    let found = with_code(&d, "core/no_parser_for_version");
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(found[0].code.to_string(), NO_PARSER_1_21_100);
}

#[test]
fn below_the_floor_the_member_count_is_not_checked() {
    // Vanilla counts the root members only for a document at or above the loader's minimum.
    let text = slab()
        .replace("1.21.110", "1.21.100")
        .replacen('{', r#"{"extra": 1, "#, 1);
    let d = rejects(&text);
    assert!(d.iter().all(|f| f.code.kind() != Kind::Syntax), "{d:#?}");
    let found = with_code(&d, "core/no_parser_for_version");
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(found[0].code.to_string(), NO_PARSER_1_21_100);
}

/// The one finding of a document without a version, whatever else it holds.
#[track_caller]
fn no_valid_version(text: &str) {
    let d = rejects(text);
    let found = with_code(&d, "core/no_valid_version");
    assert_eq!(found.len(), 1, "{text}: {d:#?}");
    assert_eq!(found[0].severity, Severity::Error);
    assert_eq!(
        found[0].code.to_string(),
        "no valid version passed by the caller or found in the input"
    );
    let others: Vec<_> = d
        .iter()
        .filter(|f| {
            !matches!(
                f.code.id(),
                "core/no_valid_version" | "voxel_shape/failed_to_load"
            )
        })
        .collect();
    assert!(others.is_empty(), "{text}: {others:#?}");
}

#[test]
fn vs24_a_missing_format_version_is_an_error() {
    no_valid_version(&slab().replace(r#""format_version": "1.21.110", "#, ""));
}

#[test]
fn a_missing_format_version_is_reported_before_anything_else_is_read() {
    // Without a version vanilla keeps the text as it is: no member count, no payload lookup.
    no_valid_version("{}");
    no_valid_version(r#"{"minecraft:voxel_shape": 5, "more": 1, "members": 2}"#);
}

#[test]
fn a_payload_under_another_name_is_missing_deviation() {
    // Vanilla hands the second member to the schema whatever its name (the name check is a
    // development-build assert); ours wants the payload under its key.
    let text = slab().replace("minecraft:voxel_shape", "minecraft:voxel_shapes");
    assert_eq!(
        kinds(&rejects(&text)),
        [
            kind("core/payload_under_other_key", ""),
            kind("voxel_shape/failed_to_load", "")
        ]
    );
}

#[test]
fn a_beta_document_has_no_parser() {
    // The family has no beta struct: with the Beta APIs toggle the literal selects nothing.
    let text = slab().replace("1.21.110", "beta");
    let mut cx = user_pack();
    cx.beta_apis = true;
    assert_eq!(
        errors(&rejects_in(&text, &cx)),
        [
            error(
                "core/no_parser_for_version",
                "unsupported version: no parser available for version beta",
                "format_version"
            ),
            failed()
        ]
    );
    assert_eq!(
        kinds(&rejects(&text)),
        [
            // Without the toggle too: the slice is selected first.
            kind("core/no_parser_for_version", "format_version"),
            kind("voxel_shape/failed_to_load", "")
        ]
    );
}

/// The game's voxel shape loader builds a `MinecraftDocumentInput`:
/// `initAsJson` reads a number as its text, so `1.22` is 1.22.0 and loads,
/// and `1.21` is 1.21.0, below the slice, with the version named as written.
#[test]
fn a_number_format_version_is_read_as_its_text() {
    let text = slab().replace(r#""1.21.110""#, "1.22");
    let loaded = loads(&text);
    assert_eq!(
        loaded.version().written,
        Some(bedrock_addon_core::version::FormatVersion::new(1, 22, 0))
    );
    let text = slab().replace(r#""1.21.110""#, "1.21");
    assert_eq!(
        errors(&rejects(&text)),
        [
            error(
                "core/no_parser_for_version",
                "unsupported version: no parser available for version 1.21.0",
                "format_version"
            ),
            failed()
        ]
    );
}
