//! `j01`–`j09`): the strict schema of a
//! document below 1.26.20.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::fog::DistanceType;

use super::common::{count, expect, expect_at, lines, loaded, rejected};
use super::support::{AIR, air, density, doc, named, pack, parse, refused};

const ALPHA: &str = "Color Value was not in the correct format as Alpha is ignored for these colors. Please use a hex value with 6 characters. EX. #FFFFFF";

#[test]
fn j01_unknown_member() {
    let d = rejected(parse(
        &doc("1.21.90", &air(&format!(r#"{AIR}, "fog_density": 1"#))),
        &pack(),
    ));
    expect_at(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "child 'fog_density' not valid here.",
        "minecraft:fog_settings/distance/air/fog_density",
    );
    refused(&d);
    // At every depth.
    for members in [
        r#""nope": 1"#.to_owned(),
        r#""distance": {"sky": {}}"#.to_owned(),
        r#""volumetric": {"nope": {}}"#.to_owned(),
        density(r#""max_density": 0.1, "nope": 1"#),
        r#""volumetric": {"media_coefficients": {"lava": {}}}"#.to_owned(),
        r#""volumetric": {"henyey_greenstein_g": {"lava": {}}}"#.to_owned(),
    ] {
        let d = rejected(parse(&doc("1.21.90", &members), &pack()));
        assert_eq!(
            count(&d, "UnrecognizedField", Severity::Error),
            1,
            "{members}: {:?}",
            lines(&d)
        );
        refused(&d);
    }
    let d = rejected(parse(
        &doc("1.21.90", "").replace(
            r#""identifier": "demo:fog""#,
            r#""identifier": "demo:fog", "x": 1"#,
        ),
        &pack(),
    ));
    expect(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "child 'x' not valid here.",
    );
}

#[test]
fn j02_missing_required() {
    for (member, name) in [
        (
            r#""render_distance_type": "render""#,
            "render_distance_type",
        ),
        (r#""fog_start": 0.9, "#, "fog_start"),
        (r#""fog_end": 1.0, "#, "fog_end"),
        (r##""fog_color": "#ABD2FF", "##, "fog_color"),
    ] {
        let without = AIR.replace(member, "");
        let without = without.trim_end_matches([',', ' ']);
        let d = rejected(parse(&doc("1.21.90", &air(without)), &pack()));
        expect_at(
            &d,
            "MissingRequiredField",
            Severity::Error,
            &format!("Required child {name} not found"),
            "minecraft:fog_settings/distance/air",
        );
        refused(&d);
    }
    for (members, name) in [
        (density(r#""uniform": true"#), "max_density"),
        (
            r#""volumetric": {"media_coefficients": {"air": {"scattering": [0, 0, 0]}}}"#
                .to_owned(),
            "absorption",
        ),
        (
            r#""volumetric": {"media_coefficients": {"air": {"absorption": [0, 0, 0]}}}"#
                .to_owned(),
            "scattering",
        ),
        (
            r#""volumetric": {"henyey_greenstein_g": {"air": {}}}"#.to_owned(),
            "henyey_greenstein_g",
        ),
        (
            format!(
                r#""distance": {{"water": {{{AIR}, "transition_fog": {{"init_fog": {{{AIR}}}, "min_percent": 0.2, "mid_seconds": 2, "mid_percent": 0.5}}}}}}"#
            ),
            "max_seconds",
        ),
        (
            format!(
                r#""distance": {{"water": {{{AIR}, "transition_fog": {{"min_percent": 0.2, "mid_seconds": 2, "mid_percent": 0.5, "max_seconds": 4}}}}}}"#
            ),
            "init_fog",
        ),
    ] {
        let d = rejected(parse(&doc("1.21.90", &members), &pack()));
        expect(
            &d,
            "MissingRequiredField",
            Severity::Error,
            &format!("Required child {name} not found"),
        );
    }
    // A root member beside the two the schema declares is an unknown member (V8).
    let d = rejected(parse(
        r#"{"format_version": "1.21.90", "minecraft:fog_settings": {"description": {"identifier": "demo:fog"}}, "x": 1}"#,
        &pack(),
    ));
    expect(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "child 'x' not valid here.",
    );
}

#[test]
fn j03_range() {
    let d = rejected(parse(
        &doc("1.21.90", &density(r#""max_density": 1.5"#)),
        &pack(),
    ));
    expect_at(
        &d,
        "InvalidValue",
        Severity::Error,
        "Value '1.500000' outside valid range [0.000000, 1.000000]",
        "minecraft:fog_settings/volumetric/density/air/max_density",
    );
    refused(&d);
    // Both ends of every [0, 1] and [-1, 1] range (`uniform` keeps V19 out of the way).
    for (members, ok) in [
        (density(r#""max_density": 1, "uniform": true"#), true),
        (density(r#""max_density": 0, "uniform": true"#), true),
        (density(r#""max_density": -0.01, "uniform": true"#), false),
        (
            r#""volumetric": {"henyey_greenstein_g": {"air": {"henyey_greenstein_g": -1}}}"#
                .to_owned(),
            true,
        ),
        (
            r#""volumetric": {"henyey_greenstein_g": {"air": {"henyey_greenstein_g": 1}}}"#
                .to_owned(),
            true,
        ),
        (
            r#""volumetric": {"henyey_greenstein_g": {"air": {"henyey_greenstein_g": 1.01}}}"#
                .to_owned(),
            false,
        ),
        (
            r#""volumetric": {"henyey_greenstein_g": {"water": {"henyey_greenstein_g": -1.01}}}"#
                .to_owned(),
            false,
        ),
    ] {
        let result = parse(&doc("1.21.90", &members), &pack());
        assert_eq!(result.is_ok(), ok, "{members}");
    }
    let percent = |min: &str, mid: &str| {
        format!(
            r#""distance": {{"water": {{{AIR}, "transition_fog": {{"init_fog": {{{AIR}}}, "min_percent": {min}, "mid_seconds": 2, "mid_percent": {mid}, "max_seconds": 4}}}}}}"#
        )
    };
    for (min, mid, ok) in [
        ("0", "1", true),
        ("1", "0", true),
        ("1.1", "0.5", false),
        ("0.5", "-0.1", false),
    ] {
        let result = parse(&doc("1.21.90", &percent(min, mid)), &pack());
        assert_eq!(result.is_ok(), ok, "{min} {mid}");
    }
}

/// `mid_seconds` and `max_seconds` have a minimum of 0, and so has the
/// `fog_end` of `init_fog`, like that of every distance entry; zero is allowed.
#[test]
fn j03_v10a_negative_seconds_and_a_negative_init_fog_end_are_out_of_range() {
    const RANGE: &str =
        "outside valid range [0.000000, 340282346638528859811704183484516925440.000000]";
    let transition = |init_end: &str, mid: &str, max: &str| {
        // The init fog starts at 0, so that an end of 0 is within V15.
        let init = AIR
            .replace("\"fog_end\": 1.0", &format!("\"fog_end\": {init_end}"))
            .replace("\"fog_start\": 0.9", "\"fog_start\": 0");
        format!(
            r#""distance": {{"water": {{{AIR}, "transition_fog": {{"init_fog": {{{init}}}, "min_percent": 0.2, "mid_seconds": {mid}, "mid_percent": 0.5, "max_seconds": {max}}}}}}}"#
        )
    };
    for version in ["1.16.100", "1.21.0", "1.21.90"] {
        for (init_end, mid, max, member, value) in [
            ("1.0", "2", "-1", "max_seconds", "-1.000000"),
            ("1.0", "-0.5", "4", "mid_seconds", "-0.500000"),
            ("-5", "2", "4", "init_fog/fog_end", "-5.000000"),
        ] {
            let d = rejected(parse(
                &doc(version, &transition(init_end, mid, max)),
                &pack(),
            ));
            expect_at(
                &d,
                "InvalidValue",
                Severity::Error,
                &format!("Value '{value}' {RANGE}"),
                &format!("minecraft:fog_settings/distance/water/transition_fog/{member}"),
            );
            assert_eq!(
                count(&d, "InvalidValue", Severity::Error),
                1,
                "{version} {member}: {:?}",
                lines(&d)
            );
            refused(&d);
        }
        // Zero is inside each of the three ranges.
        let document = loaded(parse(&doc(version, &transition("0", "0", "0")), &pack()));
        assert_eq!(
            count(document.diagnostics(), "InvalidValue", Severity::Error),
            0,
            "{version}"
        );
    }
}

#[test]
fn j04_height_range() {
    let d = rejected(parse(
        &doc(
            "1.21.90",
            &density(r#""max_density": 0.1, "max_density_height": 400"#),
        ),
        &pack(),
    ));
    expect(
        &d,
        "InvalidValue",
        Severity::Error,
        "Value '400.000000' outside valid range [-64.000000, 320.000000]",
    );
    refused(&d);
    loaded(parse(
        &doc(
            "1.21.90",
            &density(
                r#""max_density": 0.1, "max_density_height": -10, "zero_density_height": 300"#,
            ),
        ),
        &pack(),
    ));
    for (heights, ok) in [
        (
            r#""max_density_height": -64, "zero_density_height": 320"#,
            true,
        ),
        // The top of the range for `max_density_height` too: V20 wants the zero height at or
        // above it.
        (
            r#""max_density_height": 320, "zero_density_height": 320"#,
            true,
        ),
        (r#""zero_density_height": 320.5"#, false),
        (r#""zero_density_height": -64.5"#, false),
        (r#""max_density_height": -65"#, false),
    ] {
        let members = density(&format!(r#""max_density": 0.1, {heights}"#));
        assert_eq!(
            parse(&doc("1.21.90", &members), &pack()).is_ok(),
            ok,
            "{heights}"
        );
    }
}

#[test]
fn j05_bad_render_type() {
    let with = |kind: &str| {
        doc(
            "1.21.90",
            &air(&AIR.replace("\"render\"", &format!("\"{kind}\""))),
        )
    };
    for kind in ["dynamic", "prerender", "render ", ""] {
        let d = rejected(parse(&with(kind), &pack()));
        expect_at(
            &d,
            "InvalidValue",
            Severity::Error,
            &format!(
                "Value '{kind}' not allowed.  Must match at least one of the following regex patterns: fixed or render"
            ),
            "minecraft:fog_settings/distance/air/render_distance_type",
        );
        refused(&d);
    }
    for (kind, expected) in [
        ("Render", DistanceType::Render),
        ("FIXED", DistanceType::Fixed),
    ] {
        let document = loaded(parse(&with(kind), &pack()));
        let air = document.distance.as_ref().unwrap().air.as_ref().unwrap();
        assert_eq!(air.render_distance_type, Some(expected));
        // Written back in the spelling the newer parser takes.
        assert!(
            document
                .to_json()
                .unwrap()
                .contains(&format!("\"{}\"", expected.as_str()))
        );
    }
}

#[test]
fn j06_alpha_hex() {
    let with = |color: &str| doc("1.21.90", &air(&AIR.replace("#ABD2FF", color)));
    for color in ["#FF44AFF5", "#FFF", "red", "44AFF5", "#44AFFG", ""] {
        let d = rejected(parse(&with(color), &pack()));
        expect_at(
            &d,
            "InvalidValue",
            Severity::Error,
            ALPHA,
            "minecraft:fog_settings/distance/air/fog_color",
        );
        refused(&d);
    }
    // Hex digits of either case pass.
    loaded(parse(&with("#abCDef"), &pack()));
    // A coefficient string obeys the same rule.
    let media = |color: &str| {
        doc(
            "1.21.90",
            &format!(
                r#""volumetric": {{"media_coefficients": {{"air": {{"scattering": "{color}", "absorption": [0, 0, 0]}}}}}}"#
            ),
        )
    };
    let d = rejected(parse(&media("#FFF"), &pack()));
    expect(&d, "InvalidValue", Severity::Error, ALPHA);
    loaded(parse(&media("#0a0B0c"), &pack()));
}

#[test]
fn j07_bad_colour_shape() {
    let with = |color: &str| doc("1.21.90", &air(&AIR.replace("\"#ABD2FF\"", color)));
    for color in [
        r#"{"r": 1}"#,
        "[1, 0]",
        "[]",
        "7",
        "true",
        r#"[1, "2", 3]"#,
        "[1, [], 3]",
    ] {
        let d = rejected(parse(&with(color), &pack()));
        refused(&d);
        assert!(
            d.iter().any(|f| f.severity == Severity::Error
                && f.location
                    .json_path
                    .to_string()
                    .starts_with("minecraft:fog_settings/distance/air/fog_color")),
            "{color}: {:?}",
            lines(&d)
        );
    }
    let d = rejected(parse(&with("[1, 0]"), &pack()));
    expect(
        &d,
        "InvalidValue",
        Severity::Error,
        "Expected [ r, g, b ] or \"#RRGGBB\".",
    );
    let media = |color: &str| {
        doc(
            "1.21.90",
            &format!(
                r#""volumetric": {{"media_coefficients": {{"air": {{"scattering": {color}, "absorption": [0, 0, 0]}}}}}}"#
            ),
        )
    };
    for color in ["[0.5, 0.5]", "{}", "[0.5, true, 0]", "null"] {
        refused(&rejected(parse(&media(color), &pack())));
    }
    // More than three elements are read as three.
    loaded(parse(&media("[0.5, 0.5, 0.5, 1]"), &pack()));
}

#[test]
fn j08_identifier_without_namespace() {
    for identifier in [
        "myfog",
        ":fog",
        "demo:",
        "demo:fog:x",
        "demo fog:x",
        "de-mo:fog",
    ] {
        let d = rejected(parse(&named("1.21.90", identifier, ""), &pack()));
        assert!(
            d.iter().any(|f| f.code.name() == "InvalidValue"
                && f.severity == Severity::Error
                && f.location.json_path.to_string()
                    == "minecraft:fog_settings/description/identifier"),
            "{identifier}: {:?}",
            lines(&d)
        );
        refused(&d);
    }
    // The match is on the lower-cased text; `.` and `_` are name characters.
    for identifier in ["Demo:Fog", "de.mo:f_og9"] {
        let document = loaded(parse(&named("1.21.90", identifier, ""), &pack()));
        assert_eq!(document.description.identifier.as_str(), identifier);
    }
}

#[test]
fn j09_start_after_end_old_is_refused() {
    let members = air(&AIR.replace("0.9", "10").replace("1.0", "5"));
    // V15 is a rule of the older schemas too: the fog setting's validation
    // logs the line at level 3 and fails, and
    // the game refuses the file with "Failed to validate fog schema".
    let d = rejected(parse(&doc("1.21.90", &members), &pack()));
    expect(
        &d,
        "InvalidValue",
        Severity::Error,
        "Value for 'fog_end' was less than 'fog_start', 'fog_start' has to be less than or equal to 'fog_end'.",
    );
    assert!(
        d.iter().any(|f| f.code.name() == "InvalidValue"
            && f.location.json_path.to_string() == "minecraft:fog_settings/distance/air"),
        "{:?}",
        lines(&d)
    );
    refused(&d);
}

#[test]
fn wrong_types_fail_the_old_schema() {
    for members in [
        r#""distance": []"#.to_owned(),
        r#""distance": {"air": 1}"#.to_owned(),
        air(&AIR.replace("0.9", "\"0.9\"")),
        air(&AIR.replace("\"render\"", "1")),
        r#""volumetric": 1"#.to_owned(),
        density(r#""max_density": 0.1, "uniform": 1"#),
        density(r#""max_density": true"#),
        r#""volumetric": {"density": {"air": []}}"#.to_owned(),
        r#""volumetric": {"media_coefficients": {"air": 1}}"#.to_owned(),
        r#""volumetric": {"henyey_greenstein_g": {"air": 1}}"#.to_owned(),
    ] {
        let d = rejected(parse(&doc("1.21.90", &members), &pack()));
        assert!(
            d.iter().any(|f| f.code.name() == "InvalidType"
                && f.code
                    .to_string()
                    .starts_with("unknown child schema option type.  Allowed types: ")),
            "{members}: {:?}",
            lines(&d)
        );
        refused(&d);
    }
}

/// The legacy colour array is read by its first three
/// elements; an element behind them
/// is noted where it is, not dropped without a word.
#[test]
fn an_element_of_a_legacy_colour_behind_the_third_is_noted_as_dropped() {
    let text = doc(
        "1.21.90",
        &format!(
            r#"{}, "volumetric": {{"media_coefficients": {{"air": {{"scattering": [0.5, 0.25, 0.125, 9], "absorption": [0, 0, 0]}}}}}}"#,
            air(
                r#""fog_color": [1, 2, 3, 4], "fog_start": 0.5, "fog_end": 1.5, "render_distance_type": "render""#
            )
        ),
    );
    let fog = loaded(parse(&text, &pack()));
    for at in [
        "minecraft:fog_settings/distance/air/fog_color/3",
        "minecraft:fog_settings/volumetric/media_coefficients/air/scattering/3",
    ] {
        let told: Vec<String> = fog
            .diagnostics()
            .iter()
            .filter(|f| f.location.json_path.to_string() == at)
            .map(|f| format!("{} {:?}", f.code.name(), f.severity))
            .collect();
        assert_eq!(told, ["DroppedField Info"], "{at}");
    }
    let written = fog.to_json().expect("the fog writes");
    assert!(
        !written.contains('9') && !written.contains('4'),
        "{written}"
    );
}

/// W1: below 1.26.20 the members of an object are read in the sorted order of their keys
/// (jsoncpp's `std::map`), not in the file's: the findings follow that order.
#[test]
fn members_are_read_in_sorted_key_order() {
    let members = air(
        r##""render_distance_type": 7, "fog_start": "a", "fog_end": 1.0, "fog_color": "#ABD2FF""##,
    );
    let d = rejected(parse(&doc("1.21.90", &members), &pack()));
    let told: Vec<_> = d
        .iter()
        .filter(|f| f.code.name() == "InvalidType")
        .map(|f| f.location.json_path.to_string())
        .collect();
    assert_eq!(
        told,
        [
            "minecraft:fog_settings/distance/air/fog_start",
            "minecraft:fog_settings/distance/air/render_distance_type"
        ],
        "{:?}",
        lines(&d)
    );
}

/// W3: below 1.26.20 a number a float does not hold is read with a `LossyConversion`
/// information at the number.
#[test]
fn a_float_that_narrows_is_told_as_lossy() {
    let members = air(&AIR.replace("0.9", "0.123456789012"));
    let document = loaded(parse(&doc("1.21.90", &members), &pack()));
    let told: Vec<_> = document
        .diagnostics()
        .iter()
        .filter(|f| f.code.name() == "LossyConversion")
        .map(|f| (f.severity, f.location.json_path.to_string()))
        .collect();
    assert_eq!(
        told,
        [(
            Severity::Info,
            "minecraft:fog_settings/distance/air/fog_start".to_owned()
        )],
        "{:?}",
        lines(document.diagnostics())
    );
}
