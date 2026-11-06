//! `c01`–`c14`; `c15` is a unit test beside
//! the rule): a document read at 1.26.20.

use bedrock_addon::diagnostic::{Diagnostics, Severity, codes};
use bedrock_addon::document::Content;
use bedrock_addon::families::fog::{ColorNorm, DistanceType, FogColor, FogSettings, Keyframed};

use super::common::{count, expect, expect_at, find, lines, loaded, rejected, round_trip};
use super::support::{air, density, doc, engine, named, pack, parse, refused};

const V15: &str = "Value for 'fog_end' was less than 'fog_start', 'fog_start' has to be less than or equal to 'fog_end'.";
const V16: &str =
    "Value for 'fog_start' should not be greater than 1 when using 'render' distance type.";
const V16A: &str =
    "Value for 'fog_start' should not be less than -1 when using 'render' distance type.";
const V16B: &str =
    "Value for 'fog_end' should not be greater than 1 when using 'render' distance type.";
const V17: &str =
    "Value for 'mid_seconds' should be less than 'max_seconds' unless they are both set to 0.";
const V18: &str = "Values for 'max_density_height' and 'zero_density_height' are not used when 'uniform' equals true.";
const V19: &str = "Value for 'max_density_height' is required unless 'uniform' equals true.";
const V19A: &str = "Value for 'zero_density_height' is required unless 'uniform' equals true.";
const V20: &str =
    "Value for 'zero_density_height' has to be greater than or equal to 'max_density_height'.";

fn latest(members: &str) -> String {
    doc("1.26.20", members)
}

fn air_of(document: &FogSettings) -> &bedrock_addon::families::fog::FogDistanceSetting {
    document.distance.as_ref().unwrap().air.as_ref().unwrap()
}

/// `vanilla = "accepted without a log line"`.
#[test]
fn c01_unknown_member_deviation() {
    let document = loaded(parse(&latest(&air(r#""fog_density": 1"#)), &pack()));
    let warning = find(document.diagnostics(), "UnrecognizedField", Severity::Info).unwrap();
    // Our own finding: no text of the game's.
    assert!(warning.code.is(&codes::UNRECOGNIZED_FIELD));
    assert_eq!(
        warning.location.json_path.to_string(),
        "minecraft:fog_settings/distance/air/fog_density"
    );
    assert_eq!(document.diagnostics().len(), 1);
    // The member is kept and written back, at every level.
    assert!(air_of(&document).extra.get("fog_density").is_some());
    let everywhere = named(
        "1.26.20",
        "demo:fog\", \"a\": 1}, \"b\": [2], \"x\": {\"y\": \"z",
        r#""distance": {"c": 3, "water": {"d": 4, "transition_fog": {"e": 5, "init_fog": {"f": 6}}}},
           "volumetric": {"g": 7, "density": {"h": 8, "air": {"uniform": true, "i": 9}},
             "media_coefficients": {"j": 10, "cloud": {"k": 11}},
             "henyey_greenstein_g": {"l": 12, "water": {"m": 13}}}"#,
    );
    let document = loaded(parse(&everywhere, &pack()));
    assert_eq!(
        count(document.diagnostics(), "UnrecognizedField", Severity::Info),
        14,
        "{:?}",
        lines(document.diagnostics())
    );
    let text = document.to_json().unwrap();
    for member in [
        "\"a\": 1",
        "\"c\": 3",
        "\"f\": 6",
        "\"i\": 9",
        "\"k\": 11",
        "\"m\": 13",
        "\"y\": \"z\"",
    ] {
        assert!(text.contains(member), "{member}: {text}");
    }
    round_trip(&document, |t| parse(t, &pack()));
}

#[test]
fn c01_a_member_named_extra_is_an_unknown_member_like_any_other() {
    let document = loaded(parse(&latest(r#""extra": {"deep": true}"#), &pack()));
    assert_eq!(
        count(document.diagnostics(), "UnrecognizedField", Severity::Info),
        1
    );
    assert!(document.extra.get("extra").is_some());
    round_trip(&document, |t| parse(t, &pack()));
}

#[test]
fn c02_identifier_without_namespace() {
    let document = loaded(parse(&named("1.26.20", "myfog", ""), &pack()));
    assert_eq!(document.description.identifier.as_str(), "myfog");
    assert!(document.diagnostics().is_empty());
}

#[test]
fn c03_start_after_end() {
    let d = rejected(parse(
        &latest(&air(r#""fog_start": 10, "fog_end": 5"#)),
        &pack(),
    ));
    expect_at(
        &d,
        "InvalidValue",
        Severity::Error,
        V15,
        "minecraft:fog_settings/distance/air",
    );
    refused(&d);
    // Equal values are in order; one distance alone is not checked.
    for members in [
        r#""fog_start": 5, "fog_end": 5"#,
        r#""fog_start": 10"#,
        r#""fog_end": 0"#,
    ] {
        let document = loaded(parse(&latest(&air(members)), &pack()));
        assert!(document.diagnostics().is_empty(), "{members}");
    }
    // Every entry is held to it.
    for entry in ["weather", "water", "lava", "lava_resistance", "powder_snow"] {
        let members = format!(r#""distance": {{"{entry}": {{"fog_start": 10, "fog_end": 5}}}}"#);
        let d = rejected(parse(&latest(&members), &pack()));
        expect_at(
            &d,
            "InvalidValue",
            Severity::Error,
            V15,
            &format!("minecraft:fog_settings/distance/{entry}"),
        );
    }
    let init = r#""distance": {"water": {"transition_fog": {"init_fog": {"fog_start": 10, "fog_end": 5}}}}"#;
    let d = rejected(parse(&latest(init), &pack()));
    expect_at(
        &d,
        "InvalidValue",
        Severity::Error,
        V15,
        "minecraft:fog_settings/distance/water/transition_fog/init_fog",
    );
}

#[test]
fn c04_render_ranges() {
    let both = r#""render_distance_type": "render", "fog_start": 1.5, "fog_end": 2"#;
    let alone = r#""render_distance_type": "render", "fog_start": 1.5"#;
    let wrap = |place: &str, members: &str| match place {
        "air" => air(members),
        "water" => format!(r#""distance": {{"water": {{{members}}}}}"#),
        _ => format!(
            r#""distance": {{"water": {{"transition_fog": {{"init_fog": {{{members}}}}}}}}}"#
        ),
    };
    for place in ["air", "water", "init_fog"] {
        let document = loaded(parse(&latest(&wrap(place, both)), &pack()));
        let d = document.diagnostics();
        expect(d, "InvalidValue", Severity::Warning, V16);
        expect(d, "InvalidValue", Severity::Warning, V16B);
        assert_eq!(d.len(), 2, "{place}: {:?}", lines(d));
        // Without `fog_end` nothing is checked.
        let document = loaded(parse(&latest(&wrap(place, alone)), &pack()));
        assert!(document.diagnostics().is_empty(), "{place}");
    }
    // Each bound on its own: 1 and -1 are inside.
    for (members, texts) in [
        (r#""fog_start": 1, "fog_end": 1"#, vec![]),
        (r#""fog_start": -1, "fog_end": 1"#, vec![]),
        (r#""fog_start": -1.5, "fog_end": 1"#, vec![V16A]),
        (r#""fog_start": 0, "fog_end": 1.5"#, vec![V16B]),
        (r#""fog_start": 1.25, "fog_end": 1.5"#, vec![V16, V16B]),
    ] {
        let document = loaded(parse(
            &latest(&air(&format!(
                r#""render_distance_type": "render", {members}"#
            ))),
            &pack(),
        ));
        let found: Vec<String> = document
            .diagnostics()
            .iter()
            .map(|f| f.code.to_string())
            .collect();
        assert_eq!(found, texts, "{members}");
    }
    // `fixed`, or no type at all, has no such range.
    for kind in [r#""render_distance_type": "fixed", "#, ""] {
        let document = loaded(parse(
            &latest(&air(&format!(r#"{kind}"fog_start": 5, "fog_end": 50"#))),
            &pack(),
        ));
        assert!(document.diagnostics().is_empty(), "{kind}");
    }
}

/// the constraint `fog_end ≥ 0` holds for `init_fog` as for every distance
/// entry; the specific entry is not forwarded, the document is refused.
#[test]
fn v13_a_negative_fog_end_of_init_fog_is_a_constraint_violation() {
    let init = |end: &str| {
        latest(&format!(
            r#""distance": {{"water": {{"transition_fog": {{"init_fog": {{"fog_end": {end}}}}}}}}}"#
        ))
    };
    let d = rejected(parse(&init("-5"), &pack()));
    let errors: Vec<(&str, String)> = d
        .iter()
        .filter(|f| f.severity >= Severity::Error && f.code.name() != "DocumentRejected")
        .map(|f| (f.code.name(), f.location.json_path.to_string()))
        .collect();
    assert_eq!(
        errors,
        [(
            "InvalidValue",
            "minecraft:fog_settings/distance/water/transition_fog/init_fog/fog_end".to_owned()
        )],
        "{:?}",
        lines(&d)
    );
    refused(&d);
    // Zero is allowed.
    let document = loaded(parse(&init("0"), &pack()));
    assert!(document.diagnostics().is_empty());
}

fn transition(members: &str) -> String {
    latest(&format!(
        r#""distance": {{"water": {{"transition_fog": {{{members}}}}}}}"#
    ))
}

#[test]
fn c05_mid_max_seconds() {
    for members in [
        r#""mid_seconds": 5, "max_seconds": 5"#,
        r#""mid_seconds": 6, "max_seconds": 5"#,
    ] {
        let d = rejected(parse(&transition(members), &pack()));
        expect_at(
            &d,
            "InvalidValue",
            Severity::Error,
            V17,
            "minecraft:fog_settings/distance/water/transition_fog",
        );
        refused(&d);
    }
    for members in [
        r#""mid_seconds": 0, "max_seconds": 0"#,
        r#""mid_seconds": 4.5, "max_seconds": 5"#,
        r#""mid_seconds": 5"#,
        r#""max_seconds": 0"#,
    ] {
        let document = loaded(parse(&transition(members), &pack()));
        assert!(document.diagnostics().is_empty(), "{members}");
    }
    // A negative value is a constraint violation, not V17.
    let d = rejected(parse(
        &transition(r#""mid_seconds": 0, "max_seconds": -1"#),
        &pack(),
    ));
    assert_eq!(
        count(&d, "InvalidValue", Severity::Error),
        1,
        "{:?}",
        lines(&d)
    );
    assert!(d.iter().all(|f| f.code.to_string() != V17));
    refused(&d);
}

#[test]
fn c06_uniform_with_heights() {
    for heights in [
        r#""max_density_height": 20, "zero_density_height": 10"#,
        r#""max_density_height": 20"#,
        r#""zero_density_height": 10"#,
    ] {
        let d = rejected(parse(
            &latest(&density(&format!(r#""uniform": true, {heights}"#))),
            &pack(),
        ));
        expect_at(
            &d,
            "InvalidValue",
            Severity::Error,
            V18,
            "minecraft:fog_settings/volumetric/density/air",
        );
        // V18 only: the lambda returns before V19–V20, also with zero below max.
        assert_eq!(
            count(&d, "InvalidValue", Severity::Error),
            1,
            "{:?}",
            lines(&d)
        );
        refused(&d);
    }
    let document = loaded(parse(&latest(&density(r#""uniform": true"#)), &pack()));
    assert!(document.diagnostics().is_empty());
}

#[test]
fn c07_heights_required() {
    for members in [
        r#""max_density": 0.1"#,
        r#""max_density": 0.1, "uniform": false"#,
    ] {
        let d = rejected(parse(&latest(&density(members)), &pack()));
        expect(&d, "InvalidValue", Severity::Error, V19);
        expect(&d, "InvalidValue", Severity::Error, V19A);
        assert_eq!(count(&d, "InvalidValue", Severity::Error), 2);
        refused(&d);
    }
    // One height alone misses the other.
    let d = rejected(parse(
        &latest(&density(r#""max_density_height": 5"#)),
        &pack(),
    ));
    assert_eq!(
        d.iter()
            .map(|f| f.code.to_string())
            .filter(|t| t.starts_with("Value for"))
            .collect::<Vec<_>>(),
        [V19A]
    );
    let d = rejected(parse(
        &latest(&density(r#""zero_density_height": 5"#)),
        &pack(),
    ));
    assert_eq!(
        d.iter()
            .map(|f| f.code.to_string())
            .filter(|t| t.starts_with("Value for"))
            .collect::<Vec<_>>(),
        [V19]
    );
    // Every density entry is held to it.
    for entry in ["weather", "water", "lava", "lava_resistance"] {
        let members =
            format!(r#""volumetric": {{"density": {{"{entry}": {{"max_density": 0.1}}}}}}"#);
        let d = rejected(parse(&latest(&members), &pack()));
        expect_at(
            &d,
            "InvalidValue",
            Severity::Error,
            V19,
            &format!("minecraft:fog_settings/volumetric/density/{entry}"),
        );
    }
}

#[test]
fn c08_zero_below_max() {
    let d = rejected(parse(
        &latest(&density(
            r#""zero_density_height": 10, "max_density_height": 20"#,
        )),
        &pack(),
    ));
    expect(&d, "InvalidValue", Severity::Error, V20);
    assert_eq!(count(&d, "InvalidValue", Severity::Error), 1);
    refused(&d);
    for heights in [
        r#""zero_density_height": 20, "max_density_height": 20"#,
        r#""zero_density_height": 21, "max_density_height": 20"#,
    ] {
        let document = loaded(parse(&latest(&density(heights)), &pack()));
        assert!(document.diagnostics().is_empty(), "{heights}");
    }
}

#[test]
fn c09_constraints() {
    for (members, at) in [
        (
            transition_members(r#""min_percent": 1.2"#),
            "distance/water/transition_fog/min_percent",
        ),
        (
            transition_members(r#""min_percent": -0.1"#),
            "distance/water/transition_fog/min_percent",
        ),
        (
            transition_members(r#""mid_percent": 1.2"#),
            "distance/water/transition_fog/mid_percent",
        ),
        (
            transition_members(r#""mid_seconds": -1"#),
            "distance/water/transition_fog/mid_seconds",
        ),
        (
            r#""volumetric": {"henyey_greenstein_g": {"air": {"henyey_greenstein_g": -1.5}}}"#
                .to_owned(),
            "volumetric/henyey_greenstein_g/air/henyey_greenstein_g",
        ),
        (
            r#""volumetric": {"henyey_greenstein_g": {"water": {"henyey_greenstein_g": 1.5}}}"#
                .to_owned(),
            "volumetric/henyey_greenstein_g/water/henyey_greenstein_g",
        ),
        (
            density(r#""zero_density_height": 320.5, "max_density_height": 0"#),
            "volumetric/density/air/zero_density_height",
        ),
        (
            density(r#""zero_density_height": 0, "max_density_height": -64.5"#),
            "volumetric/density/air/max_density_height",
        ),
        (
            density(r#""uniform": true, "max_density": 1.01"#),
            "volumetric/density/air/max_density",
        ),
        (air(r#""fog_end": -0.5"#), "distance/air/fog_end"),
    ] {
        let d = rejected(parse(&latest(&members), &pack()));
        let errors: Vec<_> = d
            .iter()
            .filter(|f| f.severity == Severity::Error && f.code.name() == "InvalidValue")
            .collect();
        assert_eq!(errors.len(), 1, "{members}: {:?}", lines(&d));
        assert_eq!(
            errors[0].location.json_path.to_string(),
            format!("minecraft:fog_settings/{at}"),
            "{members}"
        );
        refused(&d);
    }
    // The bounds themselves are inside.
    for members in [
        transition_members(r#""min_percent": 0, "mid_percent": 1"#),
        r#""volumetric": {"henyey_greenstein_g": {"air": {"henyey_greenstein_g": -1}, "water": {"henyey_greenstein_g": 1}}}"#.to_owned(),
        density(r#""zero_density_height": 320, "max_density_height": -64, "max_density": 1"#),
        density(r#""uniform": true, "max_density": 0"#),
        air(r#""fog_end": 0"#),
    ] {
        loaded(parse(&latest(&members), &pack()));
    }
}

fn transition_members(members: &str) -> String {
    format!(r#""distance": {{"water": {{"transition_fog": {{{members}}}}}}}"#)
}

fn color_of(members: &str) -> bedrock_addon::document::Document<FogSettings> {
    loaded(parse(&latest(&air(members)), &pack()))
}

/// TOML `vanilla = "clamped silently by colorFromIntArray3"`.
#[test]
fn c10_colour_255_deviation() {
    let document = color_of(r#""fog_color": [255, 128, 0]"#);
    assert!(document.diagnostics().is_empty());
    let color = air_of(&document).fog_color.clone().unwrap();
    assert_eq!(color, FogColor::Array([255, 128, 0]));
    assert_eq!(color.unit_rgb(), [1.0, 128.0 / 255.0, 0.0]);
    round_trip(&document, |t| parse(t, &pack()));

    let document = color_of(r#""fog_color": [300, -5, 0]"#);
    assert_eq!(
        air_of(&document).fog_color,
        Some(FogColor::Array([255, 0, 0]))
    );
    assert_eq!(
        count(document.diagnostics(), "ClampedValue", Severity::Info),
        2
    );
    assert_eq!(document.diagnostics().len(), 2);
    // The bounds are not clamped.
    assert!(
        color_of(r#""fog_color": [0, 255, 254]"#)
            .diagnostics()
            .is_empty()
    );

    // The hex form takes digits of either case and nothing but six of them.
    let document = color_of(r##""fog_color": "#44aFF5""##);
    assert_eq!(
        air_of(&document).fog_color,
        Some(FogColor::Hex("#44aFF5".to_owned()))
    );
    for color in [
        r##""#FFF""##,
        r##""#44AFF5FF""##,
        r#""44AFF5""#,
        r##""#44AFFG""##,
    ] {
        let d = rejected(parse(
            &latest(&air(&format!(r#""fog_color": {color}"#))),
            &pack(),
        ));
        assert_eq!(count(&d, "InvalidValue", Severity::Error), 1, "{color}");
        refused(&d);
    }
}

#[test]
fn c10b_coeff_clamp() {
    let media = r##""volumetric": {"media_coefficients": {"air": {"scattering": [1.5, -0.2, 0.5], "absorption": "#FF8000"}}}"##;
    let document = loaded(parse(&latest(media), &pack()));
    assert!(document.diagnostics().is_empty());
    let air = document
        .volumetric
        .as_ref()
        .unwrap()
        .media_coefficients
        .as_ref()
        .unwrap()
        .air
        .clone()
        .unwrap();
    let scattering = air.scattering.unwrap();
    assert_eq!(
        scattering,
        Keyframed::Single(ColorNorm::Array([1.5, -0.2, 0.5]))
    );
    assert_eq!(scattering.single().unwrap().unit_rgb(), [1.0, 0.0, 0.5]);
    assert_eq!(
        air.absorption.unwrap().single().unwrap().unit_rgb(),
        [1.0, 128.0 / 255.0, 0.0]
    );
    round_trip(&document, |t| parse(t, &pack()));
    for color in [
        "[0.5, 0.5]",
        "[0.5, 0.5, 0.5, 1]",
        r#"[0.5, "0.5", 0.5]"#,
        "7",
    ] {
        let media = format!(
            r#""volumetric": {{"media_coefficients": {{"air": {{"scattering": {color}}}}}}}"#
        );
        refused(&rejected(parse(&latest(&media), &pack())));
    }
}

#[test]
fn c11_colour_float_array_deviation() {
    for color in [
        "[0.5, 0.5, 0.5]",
        "[1.0, 0, 0]",
        r#"["1", 0, 0]"#,
        "[1, 0]",
        "[1, 0, 0, 0]",
        "[true, 0, 0]",
    ] {
        let d = rejected(parse(
            &latest(&air(&format!(r#""fog_color": {color}"#))),
            &pack(),
        ));
        assert!(
            count(&d, "InvalidValue", Severity::Error) >= 1,
            "{color}: {:?}",
            lines(&d)
        );
        refused(&d);
    }
    let d = rejected(parse(&latest(&air(r#""fog_color": 7"#)), &pack()));
    assert_eq!(count(&d, "InvalidType", Severity::Error), 1);
}

#[test]
fn c12_transition_on_air() {
    let members = air(r#""fog_start": 0, "transition_fog": {"min_percent": 0.5}"#);
    let document = loaded(parse(&latest(&members), &pack()));
    let unknown = find(document.diagnostics(), "UnrecognizedField", Severity::Info).unwrap();
    assert_eq!(
        unknown.location.json_path.to_string(),
        "minecraft:fog_settings/distance/air/transition_fog"
    );
    assert!(air_of(&document).extra.get("transition_fog").is_some());
    round_trip(&document, |t| parse(t, &pack()));
}

#[test]
fn c13_partial_setting_zero_fill() {
    let document = color_of(r##""fog_color": "#FF0000""##);
    let air = air_of(&document);
    assert_eq!(
        (air.fog_start, air.fog_end, air.render_distance_type),
        (None, None, None)
    );
    assert_eq!((air.fog_start(), air.fog_end()), (0.0, 0.0));
    assert_eq!(air.render_distance_type(), DistanceType::Fixed);
    assert_eq!(air.fog_color.as_ref().unwrap().unit_rgb(), [1.0, 0.0, 0.0]);
    // Nothing the author left out is written.
    let text = document.to_json().unwrap();
    assert!(
        !text.contains("fog_start") && !text.contains("render_distance_type"),
        "{text}"
    );
    round_trip(&document, |t| parse(t, &pack()));
    // The water setting fills the same way.
    let document = loaded(parse(&latest(r#""distance": {"water": {}}"#), &pack()));
    let water = document.distance.as_ref().unwrap().water.as_ref().unwrap();
    assert_eq!((water.fog_start(), water.fog_end()), (0.0, 0.0));
    assert_eq!(water.render_distance_type(), DistanceType::Fixed);
    assert!(document.to_json().unwrap().contains(r#""water": {}"#));
}

#[test]
fn c14_keyframes_engine_gate() {
    let keyed = density(r#""uniform": true, "max_density": {"0.5": 0.1, "0.0": 0.02}"#);
    let d = rejected(parse(&latest(&keyed), &engine(1, 26, 20)));
    let gate = find(&d, "UnsupportedValueForm", Severity::Error).unwrap();
    assert_eq!(
        gate.location.json_path.to_string(),
        "minecraft:fog_settings/volumetric/density/air/max_density"
    );
    refused(&d);
    rejected(parse(&latest(&keyed), &engine(1, 26, 29)));

    let document = loaded(parse(&latest(&keyed), &engine(1, 26, 30)));
    assert!(document.diagnostics().is_empty());
    let air = document
        .volumetric
        .as_ref()
        .unwrap()
        .density
        .as_ref()
        .unwrap()
        .air
        .clone()
        .unwrap();
    // Sorted by key.
    assert_eq!(
        air.max_density,
        Some(Keyframed::from_pairs([(0.0, 0.02), (0.5, 0.1)]))
    );
    round_trip(&document, |t| parse(t, &engine(1, 26, 30)));
    // A single value needs no such engine.
    loaded(parse(
        &latest(&density(r#""uniform": true, "max_density": 0.1"#)),
        &engine(1, 26, 20),
    ));

    // Every keyed member is gated, each at its own path.
    let all = r##""volumetric": {
        "density": {"air": {"zero_density_height": {"0": 100}, "max_density_height": {"0": 50}}},
        "media_coefficients": {"cloud": {"scattering": {"0": [0.5, 0.5, 0.5]}, "absorption": {"0.5": "#000000"}}},
        "henyey_greenstein_g": {"water": {"henyey_greenstein_g": {"0.25": 0.5}}}}"##;
    let d = rejected(parse(&latest(all), &engine(1, 26, 20)));
    assert_eq!(
        count(&d, "UnsupportedValueForm", Severity::Error),
        5,
        "{:?}",
        lines(&d)
    );
    let document = loaded(parse(&latest(all), &pack()));
    assert!(document.diagnostics().is_empty());
    round_trip(&document, |t| parse(t, &pack()));

    // Each keyframe value obeys the member's range, and a key is a number.
    for members in [
        density(r#""uniform": true, "max_density": {"0": 1.5}"#),
        density(r#""uniform": true, "max_density": {"noon": 0.5}"#),
    ] {
        let d = rejected(parse(&latest(&members), &pack()));
        assert_eq!(
            count(&d, "InvalidValue", Severity::Error),
            1,
            "{members}: {:?}",
            lines(&d)
        );
    }
    // Below 1.26.20 an object where a number belongs is a type error.
    let d = rejected(parse(
        &doc("1.21.90", &density(r#""max_density": {"0": 0.5}"#)),
        &pack(),
    ));
    assert_eq!(count(&d, "InvalidType", Severity::Error), 1);
}

#[test]
fn c14_keyframed_heights_are_compared_at_equal_keys_deviation() {
    for (heights, ok) in [
        (
            r#""zero_density_height": {"0": 10, "0.5": 30}, "max_density_height": {"0": 5, "0.5": 30}"#,
            true,
        ),
        (
            r#""zero_density_height": {"0": 10, "0.5": 30}, "max_density_height": {"0": 5, "0.5": 31}"#,
            false,
        ),
        (
            r#""zero_density_height": 10, "max_density_height": {"0": 5, "0.5": 11}"#,
            false,
        ),
        (
            r#""zero_density_height": {"0": 10, "0.5": 4}, "max_density_height": 5"#,
            false,
        ),
        (
            r#""zero_density_height": {"0": 10}, "max_density_height": {"0.5": 50}"#,
            true,
        ),
    ] {
        let result = parse(&latest(&density(heights)), &pack());
        assert_eq!(result.is_ok(), ok, "{heights}");
        if let Err(rejected) = result {
            expect(&rejected.diagnostics, "InvalidValue", Severity::Error, V20);
        }
    }
}

#[test]
fn wrong_types_fail_the_cereal_parse() {
    for members in [
        r#""distance": []"#.to_owned(),
        air(r#""fog_start": "1""#),
        air(r#""render_distance_type": 1"#),
        density(r#""uniform": 1"#),
        density(r#""uniform": true, "max_density": true"#),
        r#""volumetric": {"henyey_greenstein_g": []}"#.to_owned(),
        r#""volumetric": {"media_coefficients": {"air": 3}}"#.to_owned(),
        transition_members(r#""init_fog": 1"#),
    ] {
        let d = rejected(parse(&latest(&members), &pack()));
        assert_eq!(
            count(&d, "InvalidType", Severity::Error),
            1,
            "{members}: {:?}",
            lines(&d)
        );
        // Our own finding: no text of the game's.
        assert!(
            d.iter()
                .all(|f| !(f.code.name() == "InvalidType" && f.vanilla().is_some()))
        );
        refused(&d);
    }
    // The enum is spelled exactly.
    for kind in ["Render", "FIXED", "dynamic"] {
        let d = rejected(parse(
            &latest(&air(&format!(r#""render_distance_type": "{kind}""#))),
            &pack(),
        ));
        assert_eq!(count(&d, "InvalidEnumValue", Severity::Error), 1, "{kind}");
        refused(&d);
    }
}

#[test]
fn c14_two_keys_of_one_time_are_one_keyframe() {
    let keyed = density(r#""uniform": true, "max_density": {"0": 0.1, "0.5": 0.2, "0.0": 0.3}"#);
    let document = loaded(parse(&latest(&keyed), &pack()));
    let air = document
        .volumetric
        .as_ref()
        .unwrap()
        .density
        .as_ref()
        .unwrap()
        .air
        .clone()
        .unwrap();
    // The later spelling of the time stands.
    assert_eq!(
        air.max_density,
        Some(Keyframed::from_pairs([(0.0, 0.3), (0.5, 0.2)]))
    );
    round_trip(&document, |t| parse(t, &pack()));
}

/// The reader keeps one keyframe per time, so only a caller can build a list
/// that holds a time twice. Written, the two would be one member, or (`-0` and `0`) two that
/// read back as one: the writer refuses the value at the second keyframe of the time. Distinct times are written.
#[test]
fn a_hand_built_keyframe_list_holding_one_time_twice_is_not_written() {
    use bedrock_addon::json::{EncodeError, Unrepresentable};
    let keyed = density(r#""uniform": true, "max_density": {"0": 0.1, "0.5": 0.2}"#);
    let document = loaded(parse(&latest(&keyed), &pack()));
    let with = |keys: Vec<(f32, f32)>| {
        let mut settings = document.value().clone();
        let density = settings.volumetric.as_mut().unwrap().density.as_mut();
        density.unwrap().air.as_mut().unwrap().max_density = Some(Keyframed::from_pairs(keys));
        settings.to_json()
    };
    let member = "minecraft:fog_settings/volumetric/density/air/max_density";
    for (keys, at) in [
        (vec![(0.0, 0.1), (0.5, 0.2), (0.5, 0.3)], "0.5"),
        (vec![(0.5, 0.2), (0.0, 0.1), (0.5, 0.2)], "0.5"),
        (vec![(0.0, 0.1), (-0.0, 0.2), (0.5, 0.3)], "-0"),
        (vec![(-0.0, 0.1), (0.25, 0.2), (0.0, 0.3)], "0"),
    ] {
        match with(keys.clone()) {
            Err(EncodeError::NotRepresentable {
                at: found,
                what: Unrepresentable::Unsupported,
            }) => assert_eq!(found.to_string(), format!("{member}/{at}"), "{keys:?}"),
            other => panic!("{keys:?}: {other:?}"),
        }
    }
    let written = with(vec![(0.0, 0.1), (0.25, 0.2), (0.5, 0.3)]).unwrap();
    let again = loaded(parse(&written, &pack()));
    assert_eq!(
        max_density(&again),
        Keyframed::from_pairs([(0.0, 0.1), (0.25, 0.2), (0.5, 0.3)])
    );
}

/// The `max_density` of `volumetric.density.air`.
fn max_density(document: &FogSettings) -> Keyframed<f32> {
    let density = document.volumetric.as_ref().unwrap().density.as_ref();
    let air = density.unwrap().air.as_ref().unwrap();
    air.max_density.clone().unwrap()
}

/// The paths of the dropped keyframes, in the order they were reported.
fn dropped_keyframes(d: &Diagnostics) -> Vec<String> {
    d.iter()
        .filter(|f| f.code.name() == "DroppedField" && f.severity == Severity::Info)
        .map(|f| f.location.json_path.to_string())
        .collect()
}

#[test]
fn c14_minus_zero_and_zero_are_one_keyframe_deviation() {
    const AT: &str = "minecraft:fog_settings/volumetric/density/air/max_density";
    for (keys, sign_negative, value, dropped) in [
        (r#""0": 0.1, "0.5": 0.2, "-0": 0.3"#, true, 0.3, "0"),
        (r#""-0": 0.1, "0.5": 0.2, "0": 0.3"#, false, 0.3, "-0"),
        (r#""-0.0": 0.1, "0.5": 0.2, "-0": 0.4"#, true, 0.4, "-0.0"),
    ] {
        let keyed = density(&format!(r#""uniform": true, "max_density": {{{keys}}}"#));
        let document = loaded(parse(&latest(&keyed), &pack()));
        let Keyframed::Keys(times) = max_density(&document) else {
            panic!("a single value: {keys}");
        };
        // The later key stands whole: its value and its time as written.
        assert_eq!(
            Keyframed::Keys(times.clone()),
            Keyframed::from_pairs([(0.0, value), (0.5, 0.2)]),
            "{keys}"
        );
        assert_eq!(times[0].time.is_sign_negative(), sign_negative, "{keys}");
        assert_eq!(
            dropped_keyframes(document.diagnostics()),
            vec![format!("{AT}/{dropped}")],
            "{keys}"
        );
        round_trip(&document, |t| parse(t, &pack()));
    }
}

#[test]
fn c14_a_dropped_keyframe_is_reported_at_its_key() {
    const AT: &str = "minecraft:fog_settings/volumetric/density/air/max_density";
    let keys = r#""1": 0.1, "0": 0.2, "1.0": 0.3, "0.5": 0.4, "0.0": 0.5, "5e-1": 0.6, "2": 0.7, "0.50": 0.8, "00": 0.9"#;
    let keyed = density(&format!(r#""uniform": true, "max_density": {{{keys}}}"#));
    let document = loaded(parse(&latest(&keyed), &pack()));
    // One finding per keyframe that does not stand, in the file's order; nothing else.
    assert_eq!(
        dropped_keyframes(document.diagnostics()),
        ["1", "0", "0.5", "0.0", "5e-1"].map(|key| format!("{AT}/{key}"))
    );
    assert_eq!(document.diagnostics().iter().count(), 5);
    let finding = find(document.diagnostics(), "DroppedField", Severity::Info).unwrap();
    let span = finding.location.span.unwrap();
    let text = latest(&keyed);
    let start = usize::try_from(span.offset).unwrap();
    assert_eq!(
        &text[start..start + usize::try_from(span.len).unwrap()],
        "0.1"
    );
    // Keys of different times drop nothing.
    let keyed = density(r#""uniform": true, "max_density": {"0": 0.1, "0.5": 0.2}"#);
    let document = loaded(parse(&latest(&keyed), &pack()));
    assert!(document.diagnostics().is_empty());
}

#[test]
fn c14_a_key_that_is_no_finite_number_fails_the_document() {
    // Rust's float parser reads `nan`, `inf` and `infinity`, and an overflow as infinite:
    // none of them is a keyframe time.
    for key in [
        "nan", "NaN", "-nan", "inf", "-inf", "infinity", "1e39", "", "a",
    ] {
        let keyed = density(&format!(
            r#""uniform": true, "max_density": {{"{key}": 0.1}}"#
        ));
        let d = rejected(parse(&latest(&keyed), &pack()));
        let at = find(&d, "InvalidValue", Severity::Error).unwrap();
        assert_eq!(
            at.location.json_path.to_string(),
            format!("minecraft:fog_settings/volumetric/density/air/max_density/{key}"),
            "{key}"
        );
    }
}

#[test]
fn c14_keyframed_heights_at_minus_zero_and_zero_are_compared() {
    // `-0` and `0` are one time: the heights at them are a pair V20 checks.
    let heights = r#""zero_density_height": {"0": 10}, "max_density_height": {"-0": 50}"#;
    let d = rejected(parse(&latest(&density(heights)), &pack()));
    expect(&d, "InvalidValue", Severity::Error, V20);
    let heights = r#""zero_density_height": {"-0": 10}, "max_density_height": {"0": 5}"#;
    loaded(parse(&latest(&density(heights)), &pack()));
}
