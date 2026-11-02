//! Rows cam03 and cam03b: what a preset, a list and a spline file do with a missing or
//! unreadable `format_version` and with a third root member.

use super::common::*;

const NO_VERSION: &str = "no valid version passed by the caller or found in the input";
const MALFORMED: &str = "Malformed format version";
const TWO_MEMBERS: &str =
    "A Minecraft Document must have exactly two members: \"format_version\" and a payload member";

const PRESET: &str =
    r#""minecraft:camera_preset": {"identifier": "ex:a", "inherit_from": "minecraft:free"}"#;

#[test]
fn cam03_a_preset_without_format_version_is_not_loaded() {
    let d = rejects(preset(&format!("{{{PRESET}}}")));
    assert_eq!(
        errors(&d),
        [
            finding("MissingFormatVersion", "", NO_VERSION),
            failed_presets()
        ]
    );
}

#[test]
fn cam03b_an_unreadable_preset_version_is_malformed() {
    for version in [
        r#""abc""#,
        "true",
        "null",
        "{}",
        "[1, 21]",
        r#"[1, 21, "0"]"#,
    ] {
        let d = rejects(preset(&format!(
            r#"{{"format_version": {version}, {PRESET}}}"#
        )));
        assert_eq!(
            errors(&d),
            [
                finding("MalformedFormatVersion", "format_version", MALFORMED),
                failed_presets()
            ],
            "{version}"
        );
    }
}

#[test]
fn cam03b_a_three_integer_preset_version_routes_then_fails_the_string_bind() {
    let d = rejects(preset(&format!(
        r#"{{"format_version": [1, 21, 0], {PRESET}}}"#
    )));
    let found = errors(&d);
    assert_eq!(found.len(), 2, "{found:?}");
    assert_eq!(
        (found[0].0, found[0].1.as_str()),
        ("InvalidType", "format_version")
    );
    assert_eq!(found[1], failed_presets());
    // Routed first: the same array below the floor is unsupported.
    let d = rejects(preset(&format!(
        r#"{{"format_version": [1, 0, 0], {PRESET}}}"#
    )));
    assert_eq!(errors(&d)[0].0, "UnsupportedVersion");
}

#[test]
fn cam03b_a_third_root_member_of_a_preset_is_an_unknown_member() {
    let d = rejects(preset(&format!(
        r#"{{"format_version": "1.21.60", {PRESET}, "extra": 1}}"#
    )));
    assert_eq!(
        errors(&d),
        [finding(
            "UnrecognizedField",
            "extra",
            "Unrecognized field on camera preset!"
        )]
    );
}

#[test]
fn cam03b_a_preset_without_its_payload_misses_a_required_member() {
    let d = rejects(preset(r#"{"format_version": "1.21.60"}"#));
    assert_eq!(
        errors(&d),
        [
            finding(
                "MissingRequiredField",
                "minecraft:camera_preset",
                "missing required field"
            ),
            failed_presets()
        ]
    );
}

#[test]
fn cam03b_a_third_root_member_of_a_spline_file_fails_the_envelope() {
    let text = splines_json(&spline_json("ex:s", ""));
    loads(spline(&text));
    let extra = text.replacen('{', r#"{"extra": 1, "#, 1);
    let d = rejects(spline(&extra));
    assert_eq!(
        errors(&d),
        [
            finding("MalformedInput", "", TWO_MEMBERS),
            finding(
                "DocumentRejected",
                "",
                "Custom camera splines not loaded correctly."
            )
        ]
    );
}

#[test]
fn cam03b_a_spline_file_without_or_with_an_unreadable_version_is_not_loaded() {
    let payload = format!(
        r#""minecraft:camera_custom_splines": {{"splines": [{}]}}"#,
        spline_json("ex:s", "")
    );
    let d = rejects(spline(&format!("{{{payload}}}")));
    assert_eq!(
        errors(&d)[0],
        finding("MissingFormatVersion", "", NO_VERSION)
    );
    let d = rejects(spline(&format!(
        r#"{{"format_version": "abc", {payload}}}"#
    )));
    assert_eq!(
        errors(&d)[0],
        finding("MalformedFormatVersion", "format_version", MALFORMED)
    );
    // A three-integer array is a version: the spline struct does not bind `format_version`.
    loads(spline(&format!(
        r#"{{"format_version": [1, 26, 0], {payload}}}"#
    )));
    for version in ["1.25.90", "1.26.61"] {
        let d = rejects(spline(&format!(
            r#"{{"format_version": "{version}", {payload}}}"#
        )));
        assert_eq!(
            errors(&d)[0],
            finding(
                "UnsupportedVersion",
                "format_version",
                &format!("unsupported version: no parser available for version {version}")
            ),
            "{version}"
        );
    }
}

/// The four camera documents read through the game's document input: the version is the first
/// textual `"format_version"`, a number read as its text (`1.26` is 1.26.0).
#[test]
fn a_number_version_is_read_as_its_text_as_minecraft_document_input_reads_it() {
    // A preset routes on the number, then fails the string bind of `format_version`, like the
    // three-integer array; below the floor the same number is unsupported.
    let d = rejects(preset(&format!(r#"{{"format_version": 1.21, {PRESET}}}"#)));
    let found = errors(&d);
    assert_eq!(found.len(), 2, "{found:?}");
    assert_eq!(
        (found[0].0, found[0].1.as_str()),
        ("InvalidType", "format_version")
    );
    assert_eq!(found[1], failed_presets());
    let d = rejects(preset(&format!(r#"{{"format_version": 1.19, {PRESET}}}"#)));
    assert_eq!(errors(&d)[0].0, "UnsupportedVersion");
    // A spline file and a camera entity document do not bind `format_version`: they load.
    let payload = format!(
        r#""minecraft:camera_custom_splines": {{"splines": [{}]}}"#,
        spline_json("ex:s", "")
    );
    let document = loads(spline(&format!(r#"{{"format_version": 1.26, {payload}}}"#)));
    assert_eq!(
        document.version().written,
        Some(bedrock_addon::version::FormatVersion::new(1, 26, 0))
    );
    let rejected = rejects(spline(&format!(r#"{{"format_version": 1.25, {payload}}}"#)));
    assert_eq!(errors(&rejected)[0].0, "UnsupportedVersion");
    let text = entity_json("minecraft:free", "").replace(r#""1.26.10""#, "1.26");
    let document = loads(entity(&text));
    assert_eq!(
        document.version().written,
        Some(bedrock_addon::version::FormatVersion::new(1, 26, 0))
    );
}

/// A camera list is read by a plain loader with a semantic-version constraint on its
/// `format_version` member, not through the game's document input: a number
/// stays a malformed version.
#[test]
fn a_number_version_of_a_camera_list_stays_malformed() {
    let text =
        list_json("1.21.60", "ex:l", r#"["minecraft:free"]"#).replace(r#""1.21.60""#, "1.21");
    let d = rejects(list(&text));
    assert_eq!(
        errors(&d)[0],
        finding("MalformedFormatVersion", "format_version", MALFORMED)
    );
}
