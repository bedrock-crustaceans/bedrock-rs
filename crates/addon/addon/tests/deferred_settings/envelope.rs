//! The envelope and version rules every payload shares: fixtures.md "Shared" rows c01–c09,
//! c16–c19, c21 and the below-floor rows (l10, w07, k04); validation.md E1–E7.

use bedrock_addon::document::Content;
use bedrock_addon::families::deferred_settings::{
    AtmosphereSettings, ColorGradingSettings, CubemapSettings, LightingSettings,
    LocalLightSettings, PbrFallbackSettings, ShadowSettings, WaterSettings,
};
use bedrock_addon::load::LoadContext;
use bedrock_addon::version::FormatVersion;

use super::common::*;

const TWO_MEMBERS: &str =
    "A Minecraft Document must have exactly two members: \"format_version\" and a payload member";
const NO_VERSION: &str = "no valid version passed by the caller or found in the input";
const MALFORMED: &str = "Malformed format version";

fn no_parser(version: &str) -> String {
    format!("unsupported version: no parser available for version {version}")
}

/// The one error of `json` loaded as every payload in turn must be `expected`.
#[track_caller]
fn every_payload(json: impl Fn(&str) -> String, expected: (&'static str, &str, &str)) {
    let expected = finding(expected.0, expected.1, expected.2);
    assert_eq!(one_error::<LightingSettings>(&json(LIGHTING)), expected);
    assert_eq!(one_error::<WaterSettings>(&json(WATER)), expected);
    assert_eq!(one_error::<AtmosphereSettings>(&json(ATMOSPHERE)), expected);
    assert_eq!(one_error::<ColorGradingSettings>(&json(GRADING)), expected);
    assert_eq!(one_error::<CubemapSettings>(&json(CUBEMAP)), expected);
    assert_eq!(one_error::<ShadowSettings>(&json(SHADOWS)), expected);
    assert_eq!(one_error::<PbrFallbackSettings>(&json(PBR)), expected);
    assert_eq!(one_error::<LocalLightSettings>(&json(LOCAL)), expected);
}

#[test]
fn c01_a_root_with_the_payload_alone_is_not_two_members() {
    every_payload(
        |key| format!(r#"{{"{key}": {{}}}}"#),
        ("MissingFormatVersion", "", TWO_MEMBERS),
    );
}

#[test]
fn c02_a_third_root_member_is_not_two_members() {
    every_payload(
        |key| format!(r#"{{"format_version": "1.21.40", "{key}": {{}}, "extra": 1}}"#),
        ("MalformedInput", "", TWO_MEMBERS),
    );
}

#[test]
fn c02_three_members_without_a_version_are_not_two_members() {
    every_payload(
        |key| format!(r#"{{"{key}": {{}}, "a": 1, "b": 2}}"#),
        ("MissingFormatVersion", "", TWO_MEMBERS),
    );
}

#[test]
fn c03_two_format_version_members_have_no_payload() {
    every_payload(
        |_| r#"{"format_version": "1.21.40", "format_version": "1.21.40"}"#.to_owned(),
        (
            "MissingPayload",
            "",
            "The payload of the Minecraft Document is missing",
        ),
    );
}

/// The members are counted as written.
#[test]
fn a_payload_written_twice_makes_three_members() {
    let json = format!(
        r#"{{"format_version": "1.21.80", "{SHADOWS}": {{"texel_size": 4}}, "{SHADOWS}": {{"texel_size": 8}}}}"#
    );
    assert_eq!(
        one_error::<ShadowSettings>(&json),
        finding("MalformedInput", "", TWO_MEMBERS)
    );
}

#[test]
fn c03b_a_foreign_payload_key_is_a_missing_payload_deviation() {
    // vanilla = "silent, defaults": a release build loads the defaults under "".
    assert_eq!(
        one_error::<LightingSettings>(r#"{"format_version": "1.21.40", "foo": {}}"#),
        finding(
            "MissingPayload",
            "",
            "The payload key does not match the expected one. Expected: 'minecraft:lighting_settings'. Actual: 'foo'."
        )
    );
    // Optional below 1.26.20 for water and atmosphere too.
    assert_eq!(
        one_error::<WaterSettings>(r#"{"format_version": "1.26.0", "foo": {}}"#).0,
        "MissingPayload"
    );
    assert_eq!(
        one_error::<AtmosphereSettings>(r#"{"format_version": "1.21.40", "foo": {}}"#).0,
        "MissingPayload"
    );
    assert_eq!(
        one_error::<LocalLightSettings>(r#"{"format_version": "1.21.120", "foo": {}}"#),
        finding(
            "MissingPayload",
            "",
            "The payload key does not match the expected one. Expected: 'minecraft:local_light_settings'. Actual: 'foo'."
        )
    );
}

#[test]
fn c03b_a_foreign_payload_key_at_1_26_20_is_a_missing_required_member() {
    assert_eq!(
        at(&[one_error::<WaterSettings>(
            r#"{"format_version": "1.26.20", "foo": {}}"#
        )]),
        [("MissingRequiredField", WATER)]
    );
    assert_eq!(
        at(&[one_error::<AtmosphereSettings>(
            r#"{"format_version": "1.26.20", "foo": {}}"#
        )]),
        [("MissingRequiredField", ATMOSPHERE)]
    );
}

#[test]
fn c04_a_syntax_error_is_the_document_json_text() {
    let (code, path, text) = one_error::<ShadowSettings>(&format!(
        r#"{{"format_version": "1.21.80" "{SHADOWS}": {{}}}}"#
    ));
    assert_eq!((code, path.as_str()), ("MalformedInput", ""));
    assert!(
        text.starts_with("Failed to parse the Minecraft Document JSON: "),
        "{text}"
    );
}

#[test]
fn c05_an_unreadable_format_version_is_malformed() {
    for version in [
        r#""x.y""#,
        "1.21",
        "null",
        "true",
        "[1, 21]",
        r#"[1, 21, "40"]"#,
        r#""""#,
        r#""1.21.*""#,
    ] {
        every_payload(
            |key| format!(r#"{{"format_version": {version}, "{key}": {{}}}}"#),
            ("MalformedFormatVersion", "format_version", MALFORMED),
        );
    }
}

#[test]
fn c06_c17_a_partial_version_is_x_y_0_and_needs_an_exact_struct() {
    assert_eq!(
        one_error::<LightingSettings>(&doc("1.21", LIGHTING, "")),
        finding("UnsupportedVersion", "format_version", &no_parser("1.21.0"))
    );
    assert_eq!(
        one_error::<CubemapSettings>(&doc("1.21", CUBEMAP, "")),
        finding("UnsupportedVersion", "format_version", &no_parser("1.21.0"))
    );
    let lighting = loads::<LightingSettings>(&doc("1.26", LIGHTING, ""));
    assert_eq!(lighting.version().parsed_as, FormatVersion::new(1, 26, 0));
    assert!(!lighting.version().upgraded);
    let water = loads::<WaterSettings>(&doc("1.26", WATER, &named("test:w", "")));
    assert_eq!(water.version().parsed_as, FormatVersion::new(1, 26, 0));
    assert!(water.version().upgraded);
}

#[test]
fn c07_above_the_last_struct_there_is_no_parser() {
    for (found, version) in [
        (
            one_error::<WaterSettings>(&doc("1.26.30", WATER, &named("t:w", ""))),
            "1.26.30",
        ),
        (
            one_error::<ColorGradingSettings>(&doc("1.21.100", GRADING, "")),
            "1.21.100",
        ),
        (
            one_error::<LightingSettings>(&doc("1.26.10", LIGHTING, "")),
            "1.26.10",
        ),
        (
            one_error::<AtmosphereSettings>(&doc("1.26.21", ATMOSPHERE, &named("t:a", ""))),
            "1.26.21",
        ),
        (
            one_error::<CubemapSettings>(&doc("1.21.131", CUBEMAP, "")),
            "1.21.131",
        ),
        (
            one_error::<ShadowSettings>(&doc("1.21.81", SHADOWS, "")),
            "1.21.81",
        ),
        (
            one_error::<PbrFallbackSettings>(&doc("1.21.41", PBR, "")),
            "1.21.41",
        ),
        (
            one_error::<LocalLightSettings>(&doc("1.21.121", LOCAL, "")),
            "1.21.121",
        ),
    ] {
        assert_eq!(
            found,
            finding("UnsupportedVersion", "format_version", &no_parser(version))
        );
    }
}

#[test]
fn c08_the_beta_literal_has_no_struct_deviation() {
    // vanilla = "Malformed format version": `fromString("beta")` is no version there, and the
    // envelope refuses the file with that text. Ours: no beta struct, so no parser with or
    // without the toggle, with the game's text.
    let json = doc("beta", SHADOWS, "");
    assert_eq!(
        one_error::<ShadowSettings>(&json),
        finding("UnsupportedVersion", "format_version", MALFORMED)
    );
    let mut cx = LoadContext::detached();
    cx.beta_apis = true;
    let d = parse_with::<ShadowSettings>(&json, &cx)
        .unwrap_err()
        .diagnostics;
    assert_eq!(
        errors(&d),
        [finding("UnsupportedVersion", "format_version", MALFORMED)]
    );
    // A payload with a chain says the same.
    assert_eq!(
        one_error::<WaterSettings>(&doc("beta", WATER, "")),
        finding("UnsupportedVersion", "format_version", MALFORMED)
    );
    let d = parse_with::<WaterSettings>(&doc("beta", WATER, ""), &cx)
        .unwrap_err()
        .diagnostics;
    assert_eq!(
        errors(&d),
        [finding("UnsupportedVersion", "format_version", MALFORMED)]
    );
    // A pre-release of a struct's version is no "beta": no parser, by its own text.
    assert_eq!(
        one_error::<ShadowSettings>(&doc("1.21.80-beta.1", SHADOWS, "")),
        finding(
            "UnsupportedVersion",
            "format_version",
            &no_parser("1.21.80-beta.1")
        )
    );
}

#[test]
fn c16_between_two_structs_there_is_no_parser() {
    assert_eq!(
        one_error::<LightingSettings>(&doc("1.21.50", LIGHTING, "")),
        finding(
            "UnsupportedVersion",
            "format_version",
            &no_parser("1.21.50")
        )
    );
    assert_eq!(
        one_error::<WaterSettings>(&doc("1.21.100", WATER, "")),
        finding(
            "UnsupportedVersion",
            "format_version",
            &no_parser("1.21.100")
        )
    );
    assert_eq!(
        one_error::<AtmosphereSettings>(&doc("1.26.0", ATMOSPHERE, "")),
        finding("UnsupportedVersion", "format_version", &no_parser("1.26.0"))
    );
    assert_eq!(
        one_error::<ColorGradingSettings>(&doc("1.21.80", GRADING, "")),
        finding(
            "UnsupportedVersion",
            "format_version",
            &no_parser("1.21.80")
        )
    );
}

#[test]
fn c18_a_prerelease_tag_equals_no_struct_version() {
    assert_eq!(
        one_error::<LightingSettings>(&doc("1.21.80-beta.1", LIGHTING, "")),
        finding(
            "UnsupportedVersion",
            "format_version",
            &no_parser("1.21.80-beta.1")
        )
    );
    // The first struct's pre-release is no "use json format version … or higher" case here.
    assert_eq!(
        one_error::<LightingSettings>(&doc("1.21.40-beta.1", LIGHTING, "")),
        finding(
            "UnsupportedVersion",
            "format_version",
            &no_parser("1.21.40-beta.1")
        )
    );
}

#[test]
fn c19_two_members_and_no_version_is_no_valid_version() {
    every_payload(
        |key| format!(r#"{{"{key}": {{}}, "foo": 1}}"#),
        ("MissingFormatVersion", "", NO_VERSION),
    );
    every_payload(
        |_| r#"{"foo": {}, "bar": 1}"#.to_owned(),
        ("MissingFormatVersion", "", NO_VERSION),
    );
}

#[test]
fn c21_an_array_version_selects_a_struct_and_then_is_no_string() {
    assert_eq!(
        at(&[one_error::<PbrFallbackSettings>(&format!(
            r#"{{"format_version": [1, 21, 40], "{PBR}": {{}}}}"#
        ))]),
        [("InvalidType", "format_version")]
    );
    assert_eq!(
        at(&[one_error::<LightingSettings>(&format!(
            r#"{{"format_version": [1, 26, 0], "{LIGHTING}": {{}}}}"#
        ))]),
        [("InvalidType", "format_version")]
    );
    // An array naming no struct version has no parser first.
    assert_eq!(
        one_error::<PbrFallbackSettings>(&format!(
            r#"{{"format_version": [1, 21, 50], "{PBR}": {{}}}}"#
        )),
        finding(
            "UnsupportedVersion",
            "format_version",
            &no_parser("1.21.50")
        )
    );
}

#[test]
fn l10_w07_k04_below_the_first_struct_there_is_no_parser() {
    assert_eq!(
        one_error::<LightingSettings>(&doc("1.21.30", LIGHTING, "")),
        finding(
            "UnsupportedVersion",
            "format_version",
            &no_parser("1.21.30")
        )
    );
    assert_eq!(
        one_error::<WaterSettings>(&doc("1.21.30", WATER, "")),
        finding(
            "UnsupportedVersion",
            "format_version",
            &no_parser("1.21.30")
        )
    );
    assert_eq!(
        one_error::<CubemapSettings>(&doc("1.21.120", CUBEMAP, "")),
        finding(
            "UnsupportedVersion",
            "format_version",
            &no_parser("1.21.120")
        )
    );
    assert_eq!(
        one_error::<ShadowSettings>(&doc("1.21.70", SHADOWS, "")),
        finding(
            "UnsupportedVersion",
            "format_version",
            &no_parser("1.21.70")
        )
    );
    assert_eq!(
        one_error::<LocalLightSettings>(&doc("1.21.110", LOCAL, "")),
        finding(
            "UnsupportedVersion",
            "format_version",
            &no_parser("1.21.110")
        )
    );
    assert_eq!(
        one_error::<PbrFallbackSettings>(&doc("1.21.39", PBR, "")),
        finding(
            "UnsupportedVersion",
            "format_version",
            &no_parser("1.21.39")
        )
    );
}

#[test]
fn c09_an_unknown_member_is_kept_with_a_warning_deviation() {
    // vanilla = "silent": the Puv log of a file that loads is dropped.
    let document = loads::<ShadowSettings>(&doc(
        "1.21.80",
        SHADOWS,
        r#""shadow_style": "soft_shadows", "mystery": {"a": [1, 2]}, "extra": 3"#,
    ));
    assert_eq!(
        infos(document.diagnostics()),
        [
            finding(
                "UnrecognizedField",
                "minecraft:shadow_settings/mystery",
                "unrecognized field"
            ),
            finding(
                "UnrecognizedField",
                "minecraft:shadow_settings/extra",
                "unrecognized field"
            ),
        ]
    );
    assert!(errors(document.diagnostics()).is_empty());
    assert_eq!(
        json(&document.to_json().unwrap()),
        json(&doc(
            "1.21.80",
            SHADOWS,
            r#""shadow_style": "soft_shadows", "mystery": {"a": [1, 2]}, "extra": 3"#
        ))
    );
}

#[test]
fn a_member_written_twice_keeps_its_last_value() {
    let document = loads::<ShadowSettings>(&doc(
        "1.21.80",
        SHADOWS,
        r#""texel_size": 4096, "texel_size": 8"#,
    ));
    assert_eq!(document.texel_size, Some(8));
    assert!(
        document.diagnostics().is_empty(),
        "{:?}",
        document.diagnostics()
    );
}

#[test]
fn an_empty_file_and_a_non_object_root_are_rejected() {
    assert_eq!(rejects::<ShadowSettings>("").iter().count(), 1);
    assert_eq!(
        at(&errors(&rejects::<ShadowSettings>("[]"))),
        [("MalformedInput", "")]
    );
    let d = ShadowSettings::parse(
        &bedrock_addon::storage::PackPath::from_static("shadows/global.json"),
        &[0xff, 0xfe, 0x00],
        &LoadContext::detached(),
    )
    .unwrap_err()
    .diagnostics;
    assert_eq!(at(&errors(&d)), [("MalformedInput", "")]);
}

#[test]
fn a_payload_that_is_no_object_is_a_type_error() {
    for payload in ["[]", "5", r#""x""#, "true"] {
        for key in [
            LIGHTING, WATER, ATMOSPHERE, GRADING, CUBEMAP, SHADOWS, PBR, LOCAL,
        ] {
            let json = format!(
                r#"{{"format_version": "{}", "{key}": {payload}}}"#,
                version_of(key)
            );
            let expected = finding("InvalidType", key, "expected an object");
            let found = match key {
                LIGHTING => one_error::<LightingSettings>(&json),
                WATER => one_error::<WaterSettings>(&json),
                ATMOSPHERE => one_error::<AtmosphereSettings>(&json),
                GRADING => one_error::<ColorGradingSettings>(&json),
                CUBEMAP => one_error::<CubemapSettings>(&json),
                SHADOWS => one_error::<ShadowSettings>(&json),
                PBR => one_error::<PbrFallbackSettings>(&json),
                _ => one_error::<LocalLightSettings>(&json),
            };
            assert_eq!(found, expected, "{json}");
        }
    }
}

/// The newest version of the payload `key`.
fn version_of(key: &str) -> &'static str {
    match key {
        LIGHTING => "1.26.0",
        WATER | ATMOSPHERE => "1.26.20",
        GRADING => "1.21.90",
        CUBEMAP => "1.21.130",
        SHADOWS => "1.21.80",
        PBR => "1.21.40",
        _ => "1.21.120",
    }
}
