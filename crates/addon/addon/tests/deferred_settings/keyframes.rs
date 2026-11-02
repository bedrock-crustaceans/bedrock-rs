//! The value forms every payload shares, through the smallest payload that has each: keyframes
//! `f32`, and the
//! three-number form of colour grading.

use bedrock_addon::document::Content;
use bedrock_addon::families::deferred_settings::{
    ColorGradingSettings, CubemapSettings, Keyframe, Keyframes, LightingSettings, Vec3,
};

use super::common::*;
use super::single::out_of_range;

const AT: &str = "minecraft:cubemap_settings/lighting/ambient_light_illuminance";

/// A cubemap file whose `ambient_light_illuminance` is `value`.
fn illuminance(value: &str) -> String {
    doc(
        "1.21.130",
        CUBEMAP,
        &named(
            "t:c",
            &format!(r#""lighting": {{"ambient_light_illuminance": {value}}}"#),
        ),
    )
}

/// The keyframes `value` loads as.
#[track_caller]
fn frames(value: &str) -> Keyframes<f32> {
    loads::<CubemapSettings>(&illuminance(value))
        .lighting
        .clone()
        .unwrap()
        .ambient_light_illuminance
        .unwrap()
}

fn frame(time: f32, value: f32) -> Keyframe<f32> {
    Keyframe { time, value }
}

#[test]
fn a02_a_single_value_is_one_keyframe_and_is_written_back_as_it_was() {
    let text = illuminance("7.5");
    let document = loads::<CubemapSettings>(&text);
    let read = frames("7.5");
    assert_eq!(read, Keyframes::Constant(7.5));
    assert_eq!(read.frames(), [frame(0.0, 7.5)]);
    assert_eq!(json(&round_trip(document.value())), json(&text));
}

#[test]
fn a01_keyframes_are_sorted_by_time() {
    assert_eq!(
        frames(r#"{"1.000000": 3, "0.250000": 2, "0.000000": 1, "0.5": 4}"#),
        Keyframes::Frames(vec![
            frame(0.0, 1.0),
            frame(0.25, 2.0),
            frame(0.5, 4.0),
            frame(1.0, 3.0)
        ])
    );
}

#[test]
fn a05_a_key_that_is_no_number_fails_the_file_deviation() {
    // vanilla = "key 0.0, loaded": the text is logged and the key is read as 0.
    for key in ["noon", "", "x0.5", "-", ".", "e5"] {
        assert_eq!(
            one_error::<CubemapSettings>(&illuminance(&format!(r#"{{"{key}": 1.0}}"#))),
            finding(
                "InvalidType",
                &format!("{AT}/{key}"),
                "String-to-float conversion failed"
            ),
            "{key:?}"
        );
    }
}

#[test]
fn a05_a_key_with_a_numeric_prefix_is_read_as_the_prefix() {
    for (key, time) in [
        ("0.5x", 0.5),
        (" 0.25", 0.25),
        ("+0.75", 0.75),
        ("1e-1", 0.1),
        ("5e-1junk", 0.5),
        (".5", 0.5),
        ("1.", 1.0),
        ("0.5e", 0.5),
        ("0.5e+", 0.5),
        // A hexadecimal float, as C99's `strtof` reads it.
        ("0x1", 1.0),
        ("0x.8p0", 0.5),
        ("0x.8", 0.5),
        ("0x0.4", 0.25),
        ("0x1p-2", 0.25),
        ("0X1P-1", 0.5),
        ("0x8p-4", 0.5),
        ("0x1.8p-1", 0.75),
        ("0x1p-1junk", 0.5),
        ("+0x1p-1", 0.5),
        // An exponent without a digit is not part of the number.
        ("0x1p", 1.0),
        ("0x1p-", 1.0),
        ("0x.8pq", 0.5),
        // No hexadecimal digit after `0x`: the number is the `0`.
        ("0x", 0.0),
        ("0xg", 0.0),
        ("0x.p1", 0.0),
    ] {
        let document = loads::<CubemapSettings>(&illuminance(&format!(r#"{{"{key}": 2.0}}"#)));
        assert!(document.diagnostics().is_empty(), "{key:?}");
        assert_eq!(
            frames(&format!(r#"{{"{key}": 2.0}}"#)),
            Keyframes::Frames(vec![frame(time, 2.0)]),
            "{key:?}"
        );
    }
}

#[test]
fn a_hexadecimal_key_outside_the_day_is_dropped_like_any_other() {
    // `0x10` is 16, not the `0` before the `x`.
    for key in [
        "0x10",
        "0x1p1",
        "0x1.000002",
        "-0x.8",
        "0x1p99999999999999999999",
    ] {
        let document =
            loads::<CubemapSettings>(&illuminance(&format!(r#"{{"{key}": 1.0, "0.5": 2.0}}"#)));
        assert_eq!(
            at(&infos(document.diagnostics())),
            [("KeyframeOutOfDomain", format!("{AT}/{key}").as_str())],
            "{key:?}"
        );
        assert_eq!(
            frames(&format!(r#"{{"{key}": 1.0, "0.5": 2.0}}"#)),
            Keyframes::Frames(vec![frame(0.5, 2.0)]),
            "{key:?}"
        );
    }
}

/// The frames of `value` with the bits of their times, the keys that stand in place of another
/// and the keys they replace.
#[allow(clippy::type_complexity, reason = "three lists of one test")]
fn merged(
    value: &str,
) -> (
    Vec<(u32, f32)>,
    Vec<(&'static str, String)>,
    Vec<(&'static str, String)>,
) {
    let document = loads::<CubemapSettings>(&illuminance(value));
    let Some(Keyframes::Frames(read)) = document
        .lighting
        .as_ref()
        .unwrap()
        .ambient_light_illuminance
        .clone()
    else {
        panic!("frames")
    };
    let own = |found: Vec<(&'static str, String, String)>| -> Vec<(&'static str, String)> {
        found
            .into_iter()
            .map(|(code, path, _)| (code, path.strip_prefix(AT).unwrap().to_owned()))
            .collect()
    };
    // Both the key that stands and the one it replaces are informations (ours).
    assert!(warnings(document.diagnostics()).is_empty());
    let (stands, dropped) = own(infos(document.diagnostics()))
        .into_iter()
        .partition(|(code, _)| *code == "InvalidValue");
    (
        read.iter().map(|f| (f.time.to_bits(), f.value)).collect(),
        stands,
        dropped,
    )
}

#[test]
fn minus_zero_and_zero_are_one_time_and_the_later_key_stands_deviation() {
    // Ours: vanilla keeps both keyframes. The time is held as `0.0` either way.
    let zero = 0.0_f32.to_bits();
    assert_eq!(
        merged(r#"{"0.0": 1, "-0": 2}"#),
        (
            vec![(zero, 2.0)],
            vec![("InvalidValue", "/-0".to_owned())],
            vec![("DroppedField", "/0.0".to_owned())],
        )
    );
    assert_eq!(
        merged(r#"{"-0": 1, "0.0": 2}"#),
        (
            vec![(zero, 2.0)],
            vec![("InvalidValue", "/0.0".to_owned())],
            vec![("DroppedField", "/-0".to_owned())],
        )
    );
    // Three spellings, among other keys: the last of them stands, each earlier one is reported.
    assert_eq!(
        merged(r#"{"1": 9, "-0.0": 1, "0.5": 8, "0": 2, "-0e0": 3}"#),
        (
            vec![
                (zero, 3.0),
                (0.5_f32.to_bits(), 8.0),
                (1.0_f32.to_bits(), 9.0)
            ],
            vec![
                ("InvalidValue", "/0".to_owned()),
                ("InvalidValue", "/-0e0".to_owned()),
            ],
            vec![
                ("DroppedField", "/-0.0".to_owned()),
                ("DroppedField", "/0".to_owned()),
            ],
        )
    );
    // A lone `-0` is the time 0 and no finding.
    assert_eq!(merged(r#"{"-0": 4}"#), (vec![(zero, 4.0)], vec![], vec![]));
    round_trip(&loads::<CubemapSettings>(&illuminance(r#"{"0.0": 1, "-0": 2}"#)).into_value());
}

#[test]
fn a06_an_empty_keyframe_object_is_refused() {
    assert_eq!(
        one_error::<CubemapSettings>(&illuminance("{}")),
        finding("InvalidValue", AT, "invalid value")
    );
}

#[test]
fn a6_a_keyframe_member_of_a_wrong_type_is_expected_keyframes() {
    for value in [r#""x""#, "true", "[1]", "[]"] {
        assert_eq!(
            one_error::<CubemapSettings>(&illuminance(value)),
            finding("InvalidType", AT, "Expected keyframes."),
            "{value}"
        );
    }
    // Inside the object form the value's own text stands, at its key.
    assert_eq!(
        one_error::<CubemapSettings>(&illuminance(r#"{"0.5": "x"}"#)),
        finding(
            "InvalidType",
            &format!("{AT}/0.5"),
            "the input must be a number"
        )
    );
}

#[test]
fn a08_l13_a_key_outside_the_day_is_dropped_with_a_warning_deviation() {
    // vanilla = "dropped silently".
    let text = illuminance(r#"{"0.0": 10, "1.5": 20, "-0.1": 7, "nan": 3, "inf": 4, "1.0": 30}"#);
    let document = loads::<CubemapSettings>(&text);
    assert_eq!(
        document
            .lighting
            .as_ref()
            .unwrap()
            .ambient_light_illuminance,
        Some(Keyframes::Frames(vec![frame(0.0, 10.0), frame(1.0, 30.0)]))
    );
    assert_eq!(
        infos(document.diagnostics()),
        [
            finding(
                "KeyframeOutOfDomain",
                &format!("{AT}/1.5"),
                "keyframe key outside [0, 1]; dropped"
            ),
            finding(
                "KeyframeOutOfDomain",
                &format!("{AT}/-0.1"),
                "keyframe key outside [0, 1]; dropped"
            ),
            finding(
                "KeyframeOutOfDomain",
                &format!("{AT}/nan"),
                "keyframe key outside [0, 1]; dropped"
            ),
            finding(
                "KeyframeOutOfDomain",
                &format!("{AT}/inf"),
                "keyframe key outside [0, 1]; dropped"
            ),
        ]
    );
    // The dropped keys are not written back.
    assert_eq!(
        json(&document.to_json().unwrap()),
        json(&illuminance(r#"{"0.0": 10.0, "1.0": 30.0}"#))
    );
}

#[test]
fn the_bounds_of_the_day_are_inside_it() {
    let document = loads::<CubemapSettings>(&illuminance(r#"{"0": 1, "1": 2}"#));
    assert!(document.diagnostics().is_empty());
    assert_eq!(
        frames(r#"{"0": 1, "1": 2}"#),
        Keyframes::Frames(vec![frame(0.0, 1.0), frame(1.0, 2.0)])
    );
    let just_outside = loads::<CubemapSettings>(&illuminance(r#"{"0.5": 1, "1.0001": 2}"#));
    assert_eq!(infos(just_outside.diagnostics()).len(), 1);
}

#[test]
fn keyframes_all_outside_the_day_fail_the_file_deviation() {
    // Ours; unverified in vanilla.
    let d = rejects::<CubemapSettings>(&illuminance(r#"{"1.5": 20, "2": 1}"#));
    assert_eq!(at(&errors(&d)), [("InvalidValue", AT)]);
    assert_eq!(infos(&d).len(), 2);
}

#[test]
fn two_keys_of_one_time_keep_the_later_value_deviation() {
    // Ours: vanilla keeps both keyframes of the one time.
    let document = loads::<CubemapSettings>(&illuminance(
        r#"{"0.5": 1, "0.25": 9, "0.50": 2, "0.5x": 3}"#,
    ));
    assert_eq!(
        document
            .lighting
            .as_ref()
            .unwrap()
            .ambient_light_illuminance,
        Some(Keyframes::Frames(vec![frame(0.25, 9.0), frame(0.5, 3.0)]))
    );
    // The key that stands in place of another is an information (ours),
    // and each keyframe that does not stand is reported at its own key.
    assert!(warnings(document.diagnostics()).is_empty());
    assert_eq!(
        infos(document.diagnostics()),
        [
            finding("InvalidValue", &format!("{AT}/0.50"), "invalid value"),
            finding("DroppedField", &format!("{AT}/0.5"), "field dropped"),
            finding("InvalidValue", &format!("{AT}/0.5x"), "invalid value"),
            finding("DroppedField", &format!("{AT}/0.50"), "field dropped"),
        ]
    );
    round_trip(&loads::<CubemapSettings>(&document.to_json().unwrap()).into_value());
}

#[test]
fn two_thousand_keyframes_load_sorted_and_are_written_back() {
    // The content of a long keyframe set; its cost is measured on the reader itself
    // (src/render/deferred/tests.rs, the_keyframes_of_one_member_…).
    let keys: Vec<String> = (0..2_000)
        .rev()
        .map(|i| format!(r#""{:.6}": {}"#, f64::from(i) / 2_000.0, i % 100))
        .collect();
    let text = illuminance(&format!("{{{}}}", keys.join(",")));
    let document = loads::<CubemapSettings>(&text);
    let written = document.to_json().unwrap();
    let Some(Keyframes::Frames(read)) = document
        .lighting
        .as_ref()
        .unwrap()
        .ambient_light_illuminance
        .clone()
    else {
        panic!("frames")
    };
    assert_eq!(read.len(), 2_000);
    assert!(read.windows(2).all(|pair| pair[0].time < pair[1].time));
    assert_eq!(read[1], frame(0.0005, 1.0));
    assert!(document.diagnostics().is_empty());
    assert_eq!(loads::<CubemapSettings>(&written).value(), document.value());
}

// ---- numbers beyond f32 --------------------------------------------------------------------

#[test]
fn a_number_beyond_f32_is_outside_every_range() {
    for value in ["1e39", "-1e39", "1e999"] {
        let found = one_error::<CubemapSettings>(&illuminance(value));
        assert_eq!((found.0, found.1.as_str()), ("InvalidValue", AT), "{value}");
    }
}

/// `directional_lights.flash.illuminance` is the one number the game sets no bound on.
fn flash(value: &str) -> String {
    doc(
        "1.26.0",
        LIGHTING,
        &named(
            "t:l",
            &format!(r#""directional_lights": {{"flash": {{"illuminance": {value}}}}}"#),
        ),
    )
}

#[test]
fn an_unbounded_number_beyond_f32_is_the_largest_finite_float_with_a_warning_deviation() {
    // Ours: never an infinity the writer would refuse.
    const AT: &str = "minecraft:lighting_settings/directional_lights/flash/illuminance";
    for (value, expected) in [("1e39", f32::MAX), ("-1e39", f32::MIN), ("1e999", f32::MAX)] {
        let document = loads::<LightingSettings>(&flash(value));
        let read = document
            .directional_lights
            .as_ref()
            .unwrap()
            .flash
            .as_ref()
            .unwrap()
            .illuminance;
        assert_eq!(read, Some(expected), "{value}");
        assert_eq!(
            infos(document.diagnostics()),
            [finding("InvalidValue", AT, "invalid value")],
            "{value}"
        );
        // What was read is written and reads back without a finding.
        round_trip(document.value());
    }
    let largest = loads::<LightingSettings>(&flash("3.4028235e38"));
    assert!(largest.diagnostics().is_empty());
}

// ---- the three-number form -----------------------------------------------------------------

fn contrast(value: &str) -> String {
    doc(
        "1.21.90",
        GRADING,
        &named(
            "t:g",
            &format!(r#""color_grading": {{"midtones": {{"contrast": {value}}}}}"#),
        ),
    )
}

#[track_caller]
fn read_contrast(value: &str) -> Vec3 {
    loads::<ColorGradingSettings>(&contrast(value))
        .color_grading
        .clone()
        .unwrap()
        .midtones
        .unwrap()
        .contrast
        .unwrap()
}

#[test]
fn three_numbers_are_an_array_or_an_object_with_absent_members_0() {
    assert_eq!(read_contrast("[1, 2.5, 4]"), Vec3::new(1.0, 2.5, 4.0));
    assert_eq!(
        read_contrast(r#"{"x": 1, "y": 2, "z": 3}"#),
        Vec3::new(1.0, 2.0, 3.0)
    );
    assert_eq!(read_contrast(r#"{"y": 2}"#), Vec3::new(0.0, 2.0, 0.0));
    assert_eq!(read_contrast("{}"), Vec3::default());
    // Written back as the array.
    let document = loads::<ColorGradingSettings>(&contrast(r#"{"z": 3}"#));
    assert_eq!(
        json(&document.to_json().unwrap()),
        json(&contrast("[0.0, 0.0, 3.0]"))
    );
}

#[test]
fn three_numbers_in_no_form_are_a_type_error() {
    const AT: &str = "minecraft:color_grading_settings/color_grading/midtones/contrast";
    for value in ["[1, 2]", "[1, 2, 3, 4]", "5", r#""1 2 3""#, "true"] {
        assert_eq!(
            one_error::<ColorGradingSettings>(&contrast(value)),
            finding("InvalidType", AT, "Expected [x, y, z]."),
            "{value}"
        );
    }
    assert_eq!(
        one_error::<ColorGradingSettings>(&contrast(r#"[1, "2", 3]"#)),
        finding(
            "InvalidType",
            &format!("{AT}/1"),
            "the input must be a number"
        )
    );
    out_of_range(
        one_error::<ColorGradingSettings>(&contrast(r#"{"x": 1, "y": 5}"#)),
        &format!("{AT}/y"),
    );
    // An unknown member of the object form is reported and not kept.
    let document = loads::<ColorGradingSettings>(&contrast(r#"{"x": 1, "w": 9}"#));
    assert_eq!(
        at(&infos(document.diagnostics())),
        [("UnrecognizedField", &*format!("{AT}/w"))]
    );
    assert!(!document.to_json().unwrap().contains("\"w\""));
}

/// the float and the colour keyframes of
/// an atmosphere and of a lighting file, each an instantiation of its own in the game and one
/// reader here.
#[test]
fn a4_a5_a6_l4_the_keyframe_rules_hold_for_atmosphere_and_lighting_floats_and_colours() {
    use bedrock_addon::families::deferred_settings::AtmosphereSettings;
    fn rules<T: Content + std::fmt::Debug>(file: impl Fn(&str) -> String, at: &str, good: &str) {
        // A4 / L4: a key that is no number.
        assert_eq!(
            one_error::<T>(&file(&format!(r#"{{"noon": {good}}}"#))),
            finding(
                "InvalidType",
                &format!("{at}/noon"),
                "String-to-float conversion failed"
            ),
            "{at}"
        );
        // A5 / L4: an empty keyframe object.
        assert_eq!(
            one_error::<T>(&file("{}")),
            finding("InvalidValue", at, "invalid value"),
            "{at}"
        );
        // A6 / L4: a member of a wrong JSON type.
        assert_eq!(
            one_error::<T>(&file("true")),
            finding("InvalidType", at, "Expected keyframes."),
            "{at}"
        );
        // The same member with keys loads without a finding.
        let document = loads::<T>(&file(&format!(r#"{{"0.0": {good}, "1.0": {good}}}"#)));
        assert!(document.diagnostics().is_empty(), "{at}");
    }
    let atmosphere = |member: &'static str| {
        move |value: &str| {
            doc(
                "1.26.20",
                ATMOSPHERE,
                &named("t:a", &format!(r#""{member}": {value}"#)),
            )
        }
    };
    rules::<AtmosphereSettings>(
        atmosphere("rayleigh_strength"),
        "minecraft:atmosphere_settings/rayleigh_strength",
        "1.0",
    );
    rules::<AtmosphereSettings>(
        atmosphere("sky_zenith_color"),
        "minecraft:atmosphere_settings/sky_zenith_color",
        "[1, 2, 3]",
    );
    let sun = |member: &'static str| {
        move |value: &str| {
            doc(
                "1.26.0",
                LIGHTING,
                &named(
                    "t:l",
                    &format!(
                        r#""directional_lights": {{"orbital": {{"sun": {{"{member}": {value}}}}}}}"#
                    ),
                ),
            )
        }
    };
    rules::<LightingSettings>(
        sun("illuminance"),
        "minecraft:lighting_settings/directional_lights/orbital/sun/illuminance",
        "1.0",
    );
    rules::<LightingSettings>(
        sun("color"),
        "minecraft:lighting_settings/directional_lights/orbital/sun/color",
        "[1, 2, 3]",
    );
    // The RGBA keyframes of the oldest lighting struct.
    rules::<LightingSettings>(
        |value: &str| {
            doc(
                "1.21.40",
                LIGHTING,
                &named(
                    "t:l",
                    &format!(r#""directional_lights": {{"sun": {{"color": {value}}}}}"#),
                ),
            )
        },
        "minecraft:lighting_settings/directional_lights/sun/color",
        "[1, 2, 3, 255]",
    );
}
