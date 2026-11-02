//! The payloads with older structs: lighting (l01–l08, l11–l13, L1–L5), water (w01–w05, W1–W4),
//! atmosphere (a01–a04, a07, A1–A3) and colour grading (g01–g07, G1–G4, G6).

use bedrock_addon::document::Content;
use bedrock_addon::families::deferred_settings::history;
use bedrock_addon::families::deferred_settings::{
    AtmosphereSettings, ColorGradingSettings, DEFAULT_CAUSTICS_TEXTURE, Keyframe, Keyframes,
    LightingSettings, Rgb8, Rgba8, TemperatureType, ToneMappingOperator, Vec3, WaterSettings,
};
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::FormatVersion;

use super::common::*;
use super::single::out_of_range;

const RGB_TEXT: &str = "Expected [r, g, b], with values in the 0-255 range, or a hex value with 6 characters (e.g. RGB as #FFFFFF).";
const RGBA_TEXT: &str = "Expected [r, g, b, a], with values in the 0-255 range, or a hex value with 8 characters (e.g. ARGB as #FFFFFFFF).";

fn lighting(version: &str, members: &str) -> String {
    doc(version, LIGHTING, &named("t:l", members))
}

fn water(version: &str, members: &str) -> String {
    doc(version, WATER, &named("t:w", members))
}

fn atmosphere(version: &str, members: &str) -> String {
    doc(version, ATMOSPHERE, &named("t:a", members))
}

fn grading(version: &str, members: &str) -> String {
    doc(version, GRADING, &named("t:g", members))
}

fn version(major: u16, minor: u16, patch: u16) -> FormatVersion {
    FormatVersion::new(major, minor, patch)
}

/// Loads `<name>.before.json`, requires the value of `<name>.after.json` and returns the
/// upgraded document.
#[track_caller]
fn upgrades<T: Content + std::fmt::Debug + PartialEq>(
    name: &str,
) -> bedrock_addon::document::Document<T> {
    let before = loads::<T>(&fixture(&format!("{name}.before.json")));
    let after = loads::<T>(&fixture(&format!("{name}.after.json")));
    assert!(before.version().upgraded, "{name}");
    assert!(!after.version().upgraded, "{name}");
    assert!(
        after.diagnostics().is_empty(),
        "{name}: {:?}",
        after.diagnostics()
    );
    assert_eq!(before.value(), after.value(), "{name}");
    // The upgraded value is written in the newest shape and reads back the same.
    round_trip(before.value());
    before
}

// ---- lighting ------------------------------------------------------------------------------

#[test]
fn l01_the_1_21_40_alpha_is_dropped_with_an_info_deviation() {
    // vanilla = "silent".
    let document = upgrades::<LightingSettings>("lighting/l01_1_21_40_rgba_to_rgb");
    assert_eq!(document.version().parsed_as, version(1, 21, 40));
    assert_eq!(
        infos(document.diagnostics()),
        [finding(
            "DroppedField",
            "minecraft:lighting_settings/directional_lights/sun/color/0.0",
            "field dropped"
        )]
    );
    assert!(warnings(document.diagnostics()).is_empty());
}

#[test]
fn l07b_the_eight_digit_hex_colour_is_argb() {
    let document = loads::<LightingSettings>(&lighting(
        "1.21.40",
        r##""directional_lights": {"sun": {"color": "#80FFC864"}, "moon": {"color": "#FF010203"}}"##,
    ));
    let orbital = document
        .directional_lights
        .clone()
        .unwrap()
        .orbital
        .unwrap();
    assert_eq!(
        orbital.sun.unwrap().color,
        Some(Keyframes::Constant(Rgb8::new(255, 200, 100)))
    );
    assert_eq!(
        orbital.moon.unwrap().color,
        Some(Keyframes::Constant(Rgb8::new(1, 2, 3)))
    );
    assert_eq!(
        at(&infos(document.diagnostics())),
        [(
            "DroppedField",
            "minecraft:lighting_settings/directional_lights/sun/color"
        )]
    );
}

#[test]
fn the_1_21_40_struct_keeps_the_alpha_as_written() {
    let text = lighting(
        "1.21.40",
        r#""directional_lights": {"sun": {"color": {"0.0": [1, 2, 3, 4], "0.5": [5, 6, 7, 8]}}}"#,
    );
    let document = LightingSettings::parse_as_written(
        &PackPath::from_static("lighting/x.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .unwrap();
    let history::lighting::Any::V1_21_40(old) = document.value() else {
        panic!("{:?}", document.value())
    };
    assert_eq!(
        old.directional_lights.clone().unwrap().sun.unwrap().color,
        Some(Keyframes::Frames(vec![
            Keyframe {
                time: 0.0,
                value: Rgba8::new(1, 2, 3, 4)
            },
            Keyframe {
                time: 0.5,
                value: Rgba8::new(5, 6, 7, 8)
            },
        ]))
    );
    assert_eq!(json(&document.value().to_json().unwrap()), json(&text));
    // Both alphas are dropped by the step: two infos, at their keys.
    let upgraded = history::lighting::upgrade(document, &LoadContext::detached()).unwrap();
    assert_eq!(
        at(&infos(upgraded.diagnostics())),
        [
            (
                "DroppedField",
                "minecraft:lighting_settings/directional_lights/sun/color/0.0"
            ),
            (
                "DroppedField",
                "minecraft:lighting_settings/directional_lights/sun/color/0.5"
            ),
        ]
    );
    assert_eq!(upgraded.value(), loads::<LightingSettings>(&text).value());
}

#[test]
fn l02_a_1_21_60_file_gets_the_sky_intensity() {
    let document = upgrades::<LightingSettings>("lighting/l02_1_21_60_sky_inserted");
    assert_eq!(document.version().parsed_as, version(1, 21, 60));
    assert!(
        document.diagnostics().is_empty(),
        "{:?}",
        document.diagnostics()
    );
}

#[test]
fn l03_the_vanilla_nether_file_moves_under_orbital() {
    let document = upgrades::<LightingSettings>("lighting/l03_1_21_70_nether");
    assert_eq!(document.version().parsed_as, version(1, 21, 70));
    assert!(
        document.diagnostics().is_empty(),
        "{:?}",
        document.diagnostics()
    );
    let lights = document.directional_lights.clone().unwrap();
    assert!(lights.flash.is_none(), "flash is not written by the step");
    assert!(lights.orbital.unwrap().sun.is_some());
}

#[test]
fn l04_the_vanilla_global_file_is_only_keyframed() {
    let document = upgrades::<LightingSettings>("lighting/l04_1_21_80_global");
    assert_eq!(document.version().parsed_as, version(1, 21, 80));
    assert!(
        document.diagnostics().is_empty(),
        "{:?}",
        document.diagnostics()
    );
    let flash = document.directional_lights.clone().unwrap().flash.unwrap();
    assert_eq!(flash.color, Some(Rgb8::new(228, 93, 255)));
    assert_eq!(flash.illuminance, Some(10.0));
}

#[test]
fn l12_a_1_21_80_file_without_ambient_and_sky_gets_the_struct_defaults() {
    let document = upgrades::<LightingSettings>("lighting/l12_1_21_80_no_ambient");
    assert!(
        document.diagnostics().is_empty(),
        "{:?}",
        document.diagnostics()
    );
}

#[test]
fn a_sky_without_intensity_is_keyframed_at_the_default() {
    let document = loads::<LightingSettings>(&lighting("1.21.80", r#""sky": {}"#));
    assert_eq!(
        document.sky.clone().unwrap().intensity,
        Some(Keyframes::at_noon(1.0))
    );
    let written = loads::<LightingSettings>(&lighting("1.21.70", r#""sky": {"intensity": 0.5}"#));
    assert_eq!(
        written.sky.clone().unwrap().intensity,
        Some(Keyframes::at_noon(0.5))
    );
}

#[test]
fn l05_keyframed_ambient_needs_1_26_0() {
    let members = r##""ambient": {"color": {"0.0": "#FFFFFF", "0.5": [1, 2, 3]}, "illuminance": {"0.0": 0.02, "0.5": 5}}"##;
    let document = loads::<LightingSettings>(&lighting("1.26.0", members));
    assert!(document.diagnostics().is_empty());
    assert!(!document.version().upgraded);
    let ambient = document.ambient.clone().unwrap();
    assert_eq!(ambient.illuminance.frames().len(), 2);
    assert_eq!(ambient.color.frames()[1].value, Rgb8::new(1, 2, 3));
    round_trip(document.value());
    // At 1.21.80 both are plain values.
    let d = rejects::<LightingSettings>(&lighting("1.21.80", members));
    assert_eq!(
        errors(&d),
        [
            finding(
                "InvalidType",
                "minecraft:lighting_settings/ambient/color",
                RGB_TEXT
            ),
            finding(
                "InvalidType",
                "minecraft:lighting_settings/ambient/illuminance",
                "the input must be a number"
            ),
        ]
    );
    // And at 1.21.80 `sky.intensity` is a plain number too.
    assert_eq!(
        at(&[one_error::<LightingSettings>(&lighting(
            "1.21.80",
            r#""sky": {"intensity": {"0.0": 1.0}}"#
        ))]),
        [("InvalidType", "minecraft:lighting_settings/sky/intensity")]
    );
}

#[test]
fn l06_l3_one_file_with_five_lighting_violations_reports_each() {
    // One file, five violations: each is reported.
    let d = rejects::<LightingSettings>(&lighting(
        "1.26.0",
        r##""directional_lights": {"orbital": {"sun": {"illuminance": 110001}, "orbital_offset_degrees": 360}},
            "emissive": {"desaturation": 1.5},
            "ambient": {"color": "#FFFFFF", "illuminance": 5.5},
            "sky": {"intensity": 0.05}"##,
    ));
    assert_eq!(
        at(&errors(&d)),
        [
            (
                "InvalidValue",
                "minecraft:lighting_settings/directional_lights/orbital/sun/illuminance"
            ),
            (
                "InvalidValue",
                "minecraft:lighting_settings/directional_lights/orbital/orbital_offset_degrees"
            ),
            (
                "InvalidValue",
                "minecraft:lighting_settings/emissive/desaturation"
            ),
            (
                "InvalidValue",
                "minecraft:lighting_settings/ambient/illuminance"
            ),
            ("InvalidValue", "minecraft:lighting_settings/sky/intensity"),
        ]
    );
}

#[test]
fn l06_l3_each_lighting_range_is_checked_at_both_ends_of_its_bounds() {
    let orbital = |members: &str| {
        lighting(
            "1.26.0",
            &format!(r#""directional_lights": {{"orbital": {{{members}}}}}"#),
        )
    };
    let cases: [(String, [&str; 2], [&str; 2]); 7] = [
        (
            "directional_lights/orbital/sun/illuminance".into(),
            ["110000.5", "-1"],
            ["0", "110000"],
        ),
        (
            "directional_lights/orbital/moon/illuminance".into(),
            ["110001", "-0.01"],
            ["0", "110000"],
        ),
        (
            "directional_lights/orbital/orbital_offset_degrees".into(),
            ["359.995", "-1"],
            ["0", "359.98999"],
        ),
        ("emissive/desaturation".into(), ["1.01", "-0.1"], ["0", "1"]),
        ("ambient/illuminance".into(), ["5.01", "-0.01"], ["0", "5"]),
        ("sky/intensity".into(), ["0.09", "1.01"], ["0.1", "1"]),
        ("sky/intensity/0.5".into(), ["0.05", "2"], ["0.1", "1.0"]),
    ];
    for (path, bad, good) in cases {
        let text = |value: &str| match path.as_str() {
            "directional_lights/orbital/sun/illuminance" => {
                orbital(&format!(r#""sun": {{"illuminance": {value}}}"#))
            }
            "directional_lights/orbital/moon/illuminance" => {
                orbital(&format!(r#""moon": {{"illuminance": {value}}}"#))
            }
            "directional_lights/orbital/orbital_offset_degrees" => {
                orbital(&format!(r#""orbital_offset_degrees": {value}"#))
            }
            "emissive/desaturation" => lighting(
                "1.26.0",
                &format!(r#""emissive": {{"desaturation": {value}}}"#),
            ),
            "ambient/illuminance" => lighting(
                "1.26.0",
                &format!(r#""ambient": {{"color": [1, 2, 3], "illuminance": {value}}}"#),
            ),
            "sky/intensity" => lighting("1.26.0", &format!(r#""sky": {{"intensity": {value}}}"#)),
            _ => lighting(
                "1.26.0",
                &format!(r#""sky": {{"intensity": {{"0.5": {value}}}}}"#),
            ),
        };
        for value in bad {
            out_of_range(
                &one_error::<LightingSettings>(&text(value)),
                &format!("{LIGHTING}/{path}"),
            );
        }
        for value in good {
            let document = loads::<LightingSettings>(&text(value));
            assert!(document.diagnostics().is_empty(), "{path} {value}");
        }
    }
}

#[test]
fn l06_l3_the_older_lighting_structs_hold_the_same_bounds() {
    // The older structs hold the same bounds.
    out_of_range(
        &one_error::<LightingSettings>(&lighting(
            "1.21.40",
            r#""ambient": {"color": [1, 2, 3], "illuminance": 5.5}"#,
        )),
        "minecraft:lighting_settings/ambient/illuminance",
    );
    out_of_range(
        &one_error::<LightingSettings>(&lighting("1.21.70", r#""sky": {"intensity": 0.05}"#)),
        "minecraft:lighting_settings/sky/intensity",
    );
    out_of_range(
        &one_error::<LightingSettings>(&lighting(
            "1.21.60",
            r#""directional_lights": {"sun": {"illuminance": 110001}}"#,
        )),
        "minecraft:lighting_settings/directional_lights/sun/illuminance",
    );
}

#[test]
fn the_flash_illuminance_has_no_bound() {
    let document = loads::<LightingSettings>(&lighting(
        "1.26.0",
        r#""directional_lights": {"flash": {"illuminance": 1000000, "color": [1, 2, 3]}}"#,
    ));
    assert!(document.diagnostics().is_empty());
    let negative = loads::<LightingSettings>(&lighting(
        "1.21.80",
        r#""directional_lights": {"flash": {"illuminance": -5}}"#,
    ));
    assert!(negative.diagnostics().is_empty());
}

#[test]
fn l07_l2_ambient_needs_both_members() {
    for version in ["1.21.40", "1.21.60", "1.21.70", "1.21.80", "1.26.0"] {
        assert_eq!(
            at(&[one_error::<LightingSettings>(&lighting(
                version,
                r##""ambient": {"color": "#FFFFFF"}"##
            ))]),
            [(
                "MissingRequiredField",
                "minecraft:lighting_settings/ambient/illuminance"
            )],
            "{version}"
        );
        assert_eq!(
            at(&[one_error::<LightingSettings>(&lighting(
                version,
                r#""ambient": {"illuminance": 0.02}"#
            ))]),
            [(
                "MissingRequiredField",
                "minecraft:lighting_settings/ambient/color"
            )],
            "{version}"
        );
        let d = rejects::<LightingSettings>(&lighting(version, r#""ambient": {}"#));
        assert_eq!(errors(&d).len(), 2, "{version}");
    }
}

#[test]
fn l07a_l5_a_colour_in_the_other_form_is_a_type_error() {
    // The documentation's 1.21.40 example: an eight-digit ambient colour.
    assert_eq!(
        one_error::<LightingSettings>(&lighting(
            "1.21.40",
            r##""ambient": {"color": "#ffffffff", "illuminance": 0.02}"##
        )),
        finding(
            "InvalidType",
            "minecraft:lighting_settings/ambient/color",
            RGB_TEXT
        )
    );
    // 1.21.40 sun and moon colours have alpha: three channels are refused, at their key.
    assert_eq!(
        one_error::<LightingSettings>(&lighting(
            "1.21.40",
            r#""directional_lights": {"sun": {"color": {"0.5": [1, 2, 3]}}}"#
        )),
        finding(
            "InvalidType",
            "minecraft:lighting_settings/directional_lights/sun/color/0.5",
            RGBA_TEXT
        )
    );
    assert_eq!(
        one_error::<LightingSettings>(&lighting(
            "1.21.40",
            r##""directional_lights": {"moon": {"color": {"0.5": "#010203"}}}"##
        )),
        finding(
            "InvalidType",
            "minecraft:lighting_settings/directional_lights/moon/color/0.5",
            RGBA_TEXT
        )
    );
    // From 1.21.60 they have none.
    for version in ["1.21.60", "1.21.70"] {
        assert_eq!(
            one_error::<LightingSettings>(&lighting(
                version,
                r#""directional_lights": {"sun": {"color": {"0.5": [1, 2, 3, 4]}}}"#
            )),
            finding(
                "InvalidType",
                "minecraft:lighting_settings/directional_lights/sun/color/0.5",
                RGB_TEXT
            ),
            "{version}"
        );
    }
    assert_eq!(
        one_error::<LightingSettings>(&lighting(
            "1.26.0",
            r##""directional_lights": {"flash": {"color": "#FF010203"}}"##
        )),
        finding(
            "InvalidType",
            "minecraft:lighting_settings/directional_lights/flash/color",
            RGB_TEXT
        )
    );
}

#[test]
fn l08_flash_below_1_21_80_is_unknown_and_dropped_by_the_step() {
    let text = lighting(
        "1.21.70",
        r#""directional_lights": {"sun": {"illuminance": 5}, "flash": {"illuminance": 3}, "note": 1}"#,
    );
    let document = loads::<LightingSettings>(&text);
    assert_eq!(
        infos(document.diagnostics()),
        [
            finding(
                "UnrecognizedField",
                "minecraft:lighting_settings/directional_lights/flash",
                "flash needs format_version 1.21.80"
            ),
            finding(
                "UnrecognizedField",
                "minecraft:lighting_settings/directional_lights/note",
                "unrecognized field"
            ),
        ]
    );
    let lights = document.directional_lights.clone().unwrap();
    assert!(lights.flash.is_none());
    assert_eq!(
        lights.orbital.unwrap().sun.unwrap().illuminance,
        Some(Keyframes::Constant(5.0))
    );
    // The member a newer struct declares is gone; the other unknown member stays.
    let written = document.to_json().unwrap();
    assert!(!written.contains("flash"), "{written}");
    assert!(written.contains("\"note\""), "{written}");
    // The unknown member warns again on the re-read; the value and the text are the same.
    let again = loads::<LightingSettings>(&written);
    assert_eq!(again.value(), document.value());
    assert_eq!(again.to_json().unwrap(), written);
    assert_eq!(
        at(&infos(again.diagnostics())),
        [(
            "UnrecognizedField",
            "minecraft:lighting_settings/directional_lights/note"
        )]
    );
}

#[test]
fn a_member_of_a_newer_struct_names_the_version_it_needs() {
    for (version, members, path, text) in [
        (
            "1.21.60",
            r#""sky": {"intensity": 0.5}"#,
            "minecraft:lighting_settings/sky",
            "sky needs format_version 1.21.70",
        ),
        (
            "1.21.40",
            r#""sky": {"intensity": 0.5}"#,
            "minecraft:lighting_settings/sky",
            "sky needs format_version 1.21.70",
        ),
        (
            "1.21.70",
            r#""directional_lights": {"orbital": {}}"#,
            "minecraft:lighting_settings/directional_lights/orbital",
            "orbital needs format_version 1.21.80",
        ),
    ] {
        let document = loads::<LightingSettings>(&lighting(version, members));
        assert_eq!(
            infos(document.diagnostics()),
            [finding("UnrecognizedField", path, text)],
            "{version}"
        );
        // The step's own value stands: an inserted sky is 1.0, not the unknown member's.
        if path.ends_with("/sky") {
            assert_eq!(
                document.sky.clone().unwrap().intensity,
                Some(Keyframes::at_noon(1.0))
            );
        }
        round_trip(&loads::<LightingSettings>(&document.to_json().unwrap()).into_value());
    }
}

#[test]
fn l11_l1_a_lighting_file_without_description_loads_unnamed_deviation() {
    // vanilla = "empty identifier".
    for version in ["1.21.40", "1.21.60", "1.21.70", "1.21.80", "1.26.0"] {
        let document = loads::<LightingSettings>(&doc(version, LIGHTING, ""));
        assert!(document.description.is_none());
        assert_eq!(
            infos(document.diagnostics()),
            [finding(
                "MissingIdentifier",
                LIGHTING,
                "no usable identifier"
            )],
            "{version}"
        );
        assert_eq!(
            at(&[one_error::<LightingSettings>(&doc(
                version,
                LIGHTING,
                r#""description": {}"#
            ))]),
            [(
                "MissingRequiredField",
                "minecraft:lighting_settings/description/identifier"
            )],
            "{version}"
        );
    }
}

#[test]
fn l13_a_sun_key_outside_the_day_is_dropped() {
    let document = loads::<LightingSettings>(&lighting(
        "1.26.0",
        r#""directional_lights": {"orbital": {"sun": {"illuminance": {"0.0": 1, "1.5": 5, "-0.1": 7}}}}"#,
    ));
    assert_eq!(
        document
            .directional_lights
            .clone()
            .unwrap()
            .orbital
            .unwrap()
            .sun
            .unwrap()
            .illuminance,
        Some(Keyframes::Frames(vec![Keyframe {
            time: 0.0,
            value: 1.0
        }]))
    );
    assert_eq!(
        at(&infos(document.diagnostics())),
        [
            (
                "KeyframeOutOfDomain",
                "minecraft:lighting_settings/directional_lights/orbital/sun/illuminance/1.5"
            ),
            (
                "KeyframeOutOfDomain",
                "minecraft:lighting_settings/directional_lights/orbital/sun/illuminance/-0.1"
            ),
        ]
    );
}

#[test]
fn the_games_lighting_values_stand_in_for_absent_members() {
    let document = loads::<LightingSettings>(&lighting("1.26.0", ""));
    let lights = document.directional_lights();
    let orbital = lights.orbital();
    assert_eq!(orbital.sun_illuminance().frames().len(), 7);
    assert_eq!(orbital.sun_illuminance().frames()[0].value, 109_880.0);
    assert_eq!(
        orbital.sun_illuminance().frames()[3],
        Keyframe {
            time: 0.5,
            value: 1.0
        }
    );
    assert_eq!(orbital.moon_illuminance(), Keyframes::Constant(0.27));
    assert_eq!(orbital.sun_color(), Keyframes::Constant(Rgb8::WHITE));
    assert_eq!(orbital.moon_color(), Keyframes::Constant(Rgb8::WHITE));
    assert_eq!(orbital.orbital_offset_degrees(), Keyframes::Constant(0.0));
    assert_eq!(lights.flash().color(), Rgb8::WHITE);
    assert_eq!(lights.flash().illuminance(), 1.0);
    assert_eq!(document.emissive().desaturation(), 0.1);
    assert_eq!(document.ambient().color, Keyframes::at_noon(Rgb8::WHITE));
    assert_eq!(document.ambient().illuminance, Keyframes::at_noon(0.02));
    assert_eq!(document.sky().intensity(), Keyframes::Constant(1.0));
    // Nothing of that is written back.
    assert_eq!(
        json(&document.to_json().unwrap()),
        json(&lighting("1.26.0", ""))
    );
}

#[test]
fn every_lighting_struct_writes_what_it_read() {
    for (version, members) in [
        (
            "1.21.40",
            r#""directional_lights": {"sun": {"illuminance": 5.0, "color": [1, 2, 3, 4]}, "orbital_offset_degrees": 10.0}, "emissive": {"desaturation": 0.5}, "ambient": {"color": [1, 2, 3], "illuminance": 1.0}"#,
        ),
        (
            "1.21.60",
            r#""directional_lights": {"moon": {"illuminance": {"0.5": 5.0}, "color": [1, 2, 3]}}"#,
        ),
        (
            "1.21.70",
            r#""directional_lights": {"sun": {"color": {"0.0": [1, 2, 3]}}}, "sky": {"intensity": 0.5}"#,
        ),
        (
            "1.21.80",
            r#""directional_lights": {"orbital": {"sun": {"illuminance": 5.0}}, "flash": {"illuminance": 3.0, "color": [1, 2, 3]}}, "ambient": {"color": [1, 2, 3], "illuminance": 1.0}, "sky": {"intensity": 0.5}"#,
        ),
        (
            "1.26.0",
            r#""ambient": {"color": {"0.0": [1, 2, 3]}, "illuminance": 1.0}, "sky": {"intensity": {"0.5": 0.5}}"#,
        ),
    ] {
        let text = lighting(version, members);
        let document = LightingSettings::parse_as_written(
            &PackPath::from_static("lighting/x.json"),
            text.as_bytes(),
            &LoadContext::detached(),
        )
        .unwrap();
        assert!(document.diagnostics().is_empty(), "{version}");
        assert_eq!(
            json(&document.value().to_json().unwrap()),
            json(&text),
            "{version}"
        );
    }
}

// ---- water ---------------------------------------------------------------------------------

#[test]
fn w01_sample_width_is_dropped_with_an_info_deviation() {
    // vanilla = "silent".
    let document = upgrades::<WaterSettings>("water/w01_1_21_40_to_latest");
    assert_eq!(document.version().parsed_as, version(1, 21, 40));
    assert_eq!(
        infos(document.diagnostics()),
        [finding(
            "DroppedField",
            "minecraft:water_settings/waves/sampleWidth",
            "field dropped"
        )]
    );
    assert!(document.caustics.is_none(), "nothing is inserted");
    assert_eq!(document.waves.clone().unwrap().direction_increment, None);
    // At 1.21.80 the member is dropped the same way.
    let later = loads::<WaterSettings>(&water(
        "1.21.80",
        r#""waves": {"sampleWidth": 0.5, "direction_increment": 90}"#,
    ));
    assert_eq!(
        at(&infos(later.diagnostics())),
        [("DroppedField", "minecraft:water_settings/waves/sampleWidth")]
    );
    assert_eq!(later.waves.clone().unwrap().direction_increment, Some(90.0));
    // Without the member the steps report nothing.
    let none = loads::<WaterSettings>(&water("1.21.40", r#""waves": {"enabled": true}"#));
    assert!(none.diagnostics().is_empty());
}

#[test]
fn w02_the_vanilla_water_file_loads_and_resolves_the_games_values() {
    let document = loads::<WaterSettings>(&fixture("water/w02_vanilla_1_21_120.json"));
    assert_eq!(document.version().parsed_as, version(1, 21, 120));
    assert!(document.version().upgraded);
    assert!(
        document.diagnostics().is_empty(),
        "{:?}",
        document.diagnostics()
    );
    assert_eq!(
        document.description.as_ref().unwrap().identifier.as_str(),
        "minecraft:default_water"
    );
    let caustics = document.caustics();
    assert_eq!(caustics.texture, None);
    assert_eq!(caustics.texture(), DEFAULT_CAUSTICS_TEXTURE);
    assert_eq!(DEFAULT_CAUSTICS_TEXTURE, "renderer/caustics");
    assert_eq!(caustics.power, Some(1.0));
    let waves = document.waves();
    assert_eq!(waves.direction_increment, None);
    assert_eq!(waves.direction_increment(), 80.0);
    assert_eq!(waves.pull, Some(0.38));
    assert_eq!(waves.octaves, Some(28));
    assert_eq!(document.biome_water_color_contribution(), 0.0);
    round_trip(document.value());
    // The 1.21.80 vanilla file of the older pack layer too; it still has `sampleWidth`, at
    // the lower bound of its range.
    let older = loads::<WaterSettings>(&fixture("water/vanilla_1_21_80.json"));
    assert_eq!(
        found(
            older.diagnostics(),
            bedrock_addon::diagnostic::Severity::Info
        ),
        [finding(
            "DroppedField",
            "minecraft:water_settings/waves/sampleWidth",
            "field dropped"
        )]
    );
    assert_eq!(older.diagnostics().len(), 1);
    assert_eq!(older.value(), document.value());
}

#[test]
fn the_games_water_values_stand_in_for_absent_members() {
    let document = loads::<WaterSettings>(&water("1.26.20", ""));
    let particles = document.particle_concentrations();
    assert_eq!(
        (
            particles.chlorophyll(),
            particles.suspended_sediment(),
            particles.cdom()
        ),
        (0.5, 1.0, 0.5)
    );
    let waves = document.waves();
    assert!(!waves.enabled());
    assert_eq!(
        (
            waves.frequency(),
            waves.octaves(),
            waves.depth(),
            waves.speed(),
            waves.pull(),
            waves.mix()
        ),
        (1.0, 28, 1.0, 2.0, 0.38, 0.2)
    );
    assert_eq!(
        (
            waves.frequency_scaling(),
            waves.speed_scaling(),
            waves.direction_increment(),
            waves.shape()
        ),
        (1.2, 1.03, 80.0, 1.5)
    );
    let caustics = document.caustics();
    assert!(caustics.enabled());
    assert_eq!(
        (caustics.frame_length(), caustics.scale(), caustics.power()),
        (0.09, 0.5, 1.0)
    );
    assert_eq!(
        json(&document.to_json().unwrap()),
        json(&water("1.26.20", ""))
    );
}

#[test]
fn w03_w4_the_colour_contribution_needs_1_26_0() {
    let members = r#""biome_water_color_contribution": 0.5"#;
    for version in ["1.26.0", "1.26.20"] {
        let document = loads::<WaterSettings>(&water(version, members));
        assert_eq!(
            document.biome_water_color_contribution,
            Some(0.5),
            "{version}"
        );
        assert!(document.diagnostics().is_empty(), "{version}");
    }
    for version in ["1.21.40", "1.21.80", "1.21.120"] {
        let document = loads::<WaterSettings>(&water(version, members));
        assert_eq!(document.biome_water_color_contribution, None, "{version}");
        assert_eq!(
            infos(document.diagnostics()),
            [finding(
                "UnrecognizedField",
                "minecraft:water_settings/biome_water_color_contribution",
                "biome_water_color_contribution needs format_version 1.26.0"
            )],
            "{version}"
        );
        assert!(!document.to_json().unwrap().contains("contribution"));
    }
    assert_eq!(
        one_error::<WaterSettings>(&water("1.26.10", members)).0,
        "UnsupportedVersion"
    );
}

#[test]
fn w4_members_of_another_struct_are_unknown_with_the_version_they_need() {
    for (version, members, path, text) in [
        (
            "1.21.40",
            r#""caustics": {"enabled": false}"#,
            "minecraft:water_settings/caustics",
            "caustics needs format_version 1.21.80",
        ),
        (
            "1.21.40",
            r#""waves": {"direction_increment": 90}"#,
            "minecraft:water_settings/waves/direction_increment",
            "direction_increment needs format_version 1.21.80",
        ),
    ] {
        let document = loads::<WaterSettings>(&water(version, members));
        assert_eq!(
            infos(document.diagnostics()),
            [finding("UnrecognizedField", path, text)]
        );
        assert!(document.caustics.is_none());
        assert_eq!(document.waves().direction_increment, None);
        round_trip(&loads::<WaterSettings>(&document.to_json().unwrap()).into_value());
    }
    // `sampleWidth` from 1.21.120 is unknown with the version that took it away; it is kept
    // like any other unknown member.
    for version in ["1.21.120", "1.26.0", "1.26.20"] {
        let document = loads::<WaterSettings>(&water(version, r#""waves": {"sampleWidth": 0.5}"#));
        assert_eq!(
            infos(document.diagnostics()),
            [finding(
                "UnrecognizedField",
                "minecraft:water_settings/waves/sampleWidth",
                "sampleWidth is no longer used (format_version 1.21.120)"
            )],
            "{version}"
        );
        assert!(document.to_json().unwrap().contains("sampleWidth"));
    }
    // Where the struct declares it, it is a member and says nothing.
    for version in ["1.21.40", "1.21.80"] {
        let d = loads::<WaterSettings>(&water(version, r#""waves": {"sampleWidth": 0.5}"#));
        assert_eq!(warnings(d.diagnostics()), [], "{version}");
    }
}

#[test]
fn w04_w1_w2_description_is_required_from_1_26_20_deviation() {
    assert_eq!(
        at(&[one_error::<WaterSettings>(&doc("1.26.20", WATER, ""))]),
        [(
            "MissingRequiredField",
            "minecraft:water_settings/description"
        )]
    );
    // vanilla = "empty identifier" below 1.26.20.
    for version in ["1.21.40", "1.21.80", "1.21.120", "1.26.0"] {
        let document = loads::<WaterSettings>(&doc(version, WATER, ""));
        assert!(document.description.is_none());
        assert_eq!(
            infos(document.diagnostics()),
            [finding("MissingIdentifier", WATER, "no usable identifier")],
            "{version}"
        );
        // Such a value has no 1.26.20 form: the writer refuses it rather than write a file
        // the loader refuses.
        assert!(document.to_json().is_err(), "{version}");
    }
    for version in ["1.21.40", "1.21.80", "1.21.120", "1.26.0", "1.26.20"] {
        assert_eq!(
            at(&[one_error::<WaterSettings>(&doc(
                version,
                WATER,
                r#""description": {}"#
            ))]),
            [(
                "MissingRequiredField",
                "minecraft:water_settings/description/identifier"
            )],
            "{version}"
        );
    }
}

#[test]
fn w05_w3_one_file_with_five_water_violations_reports_each() {
    let d = rejects::<WaterSettings>(&water(
        "1.26.20",
        r#""waves": {"octaves": 31, "pull": -1.5}, "caustics": {"frame_length": 0, "power": 7},
           "particle_concentrations": {"cdom": 16}"#,
    ));
    let found = errors(&d);
    assert_eq!(
        at(&found),
        [
            (
                "InvalidValue",
                "minecraft:water_settings/particle_concentrations/cdom"
            ),
            (
                "InvalidValue",
                "minecraft:water_settings/caustics/frame_length"
            ),
            ("InvalidValue", "minecraft:water_settings/caustics/power"),
            ("InvalidValue", "minecraft:water_settings/waves/octaves"),
            ("InvalidValue", "minecraft:water_settings/waves/pull"),
        ]
    );
    for (_, _, text) in &found {
        assert!(
            text.starts_with("the input is outside of the accepted range"),
            "{text}"
        );
    }
}

#[test]
fn w05_w3_each_water_range_is_checked_at_both_ends_of_its_bounds() {
    let cases: [(&str, &str, [&str; 2], [&str; 2]); 20] = [
        (
            "particle_concentrations",
            "chlorophyll",
            ["-0.1", "10.5"],
            ["0", "10"],
        ),
        (
            "particle_concentrations",
            "suspended_sediment",
            ["-1", "301"],
            ["0", "300"],
        ),
        (
            "particle_concentrations",
            "cdom",
            ["-1", "15.5"],
            ["0", "15"],
        ),
        ("waves", "frequency", ["0.009", "3.5"], ["0.01", "3"]),
        ("waves", "octaves", ["0", "31"], ["1", "30"]),
        ("waves", "depth", ["-0.1", "3.1"], ["0", "3"]),
        ("waves", "speed", ["0.009", "10.5"], ["0.01", "10"]),
        ("waves", "pull", ["-1.1", "1.1"], ["-1", "1"]),
        ("waves", "mix", ["-0.1", "1.1"], ["0", "1"]),
        ("waves", "frequency_scaling", ["-0.1", "2.1"], ["0", "2"]),
        ("waves", "speed_scaling", ["-0.1", "2.1"], ["0", "2"]),
        ("waves", "direction_increment", ["-1", "361"], ["0", "360"]),
        ("waves", "shape", ["0.9", "10.1"], ["1", "10"]),
        ("caustics", "frame_length", ["0.009", "5.1"], ["0.01", "5"]),
        ("caustics", "scale", ["0.09", "5.1"], ["0.1", "5"]),
        ("caustics", "power", ["0.9", "6.1"], ["1", "6"]),
        (
            "",
            "biome_water_color_contribution",
            ["-0.1", "1.1"],
            ["0", "1"],
        ),
        ("waves", "octaves", ["1.5", "30.5"], ["1.0", "30.0"]),
        ("waves", "mix", ["1e39", "-1e39"], ["0.5", "0.25"]),
        ("caustics", "power", ["100", "-100"], ["2", "3"]),
    ];
    for (group, member, bad, good) in cases {
        let text = |value: &str| {
            if group.is_empty() {
                water("1.26.20", &format!(r#""{member}": {value}"#))
            } else {
                water("1.26.20", &format!(r#""{group}": {{"{member}": {value}}}"#))
            }
        };
        let path = if group.is_empty() {
            format!("{WATER}/{member}")
        } else {
            format!("{WATER}/{group}/{member}")
        };
        for value in bad {
            let found = one_error::<WaterSettings>(&text(value));
            assert_eq!(
                (found.0, found.1.as_str()),
                ("InvalidValue", path.as_str()),
                "{member} {value}"
            );
        }
        for value in good {
            let document = loads::<WaterSettings>(&text(value));
            assert!(document.diagnostics().is_empty(), "{member} {value}");
        }
    }
}

#[test]
fn w05_w3_sample_width_has_its_own_range_and_a_number_is_the_one_type() {
    // `sampleWidth` has its own range while it exists.
    for value in ["0.009", "1.1"] {
        out_of_range(
            &one_error::<WaterSettings>(&water(
                "1.21.40",
                &format!(r#""waves": {{"sampleWidth": {value}}}"#),
            )),
            "minecraft:water_settings/waves/sampleWidth",
        );
    }
    // A number is the one type a ranged member takes.
    assert_eq!(
        one_error::<WaterSettings>(&water("1.26.20", r#""waves": {"depth": "1"}"#)),
        finding(
            "InvalidType",
            "minecraft:water_settings/waves/depth",
            "the input must be a number"
        )
    );
    assert_eq!(
        at(&[one_error::<WaterSettings>(&water(
            "1.26.20",
            r#""waves": {"enabled": 1}"#
        ))]),
        [("InvalidType", "minecraft:water_settings/waves/enabled")]
    );
    assert_eq!(
        at(&[one_error::<WaterSettings>(&water(
            "1.26.20",
            r#""caustics": {"texture": 5}"#
        ))]),
        [("InvalidType", "minecraft:water_settings/caustics/texture")]
    );
}

#[test]
fn every_water_struct_writes_what_it_read() {
    let waves = r#""enabled": true, "frequency": 1.5, "octaves": 10, "depth": 0.5"#;
    for (version, members) in [
        (
            "1.21.40",
            format!(
                r#""particle_concentrations": {{"cdom": 1.0}}, "waves": {{{waves}, "sampleWidth": 0.25}}"#
            ),
        ),
        (
            "1.21.80",
            format!(
                r#""caustics": {{"enabled": false, "texture": "textures/c"}}, "waves": {{{waves}, "sampleWidth": 0.25, "direction_increment": 45.0}}"#
            ),
        ),
        (
            "1.21.120",
            format!(
                r#""caustics": {{"scale": 2.0}}, "waves": {{{waves}, "direction_increment": 45.0}}"#
            ),
        ),
        (
            "1.26.0",
            format!(r#""waves": {{{waves}}}, "biome_water_color_contribution": 0.25"#),
        ),
        (
            "1.26.20",
            format!(r#""waves": {{{waves}}}, "biome_water_color_contribution": 0.25"#),
        ),
    ] {
        let text = water(version, &members);
        let document = WaterSettings::parse_as_written(
            &PackPath::from_static("water/x.json"),
            text.as_bytes(),
            &LoadContext::detached(),
        )
        .unwrap();
        assert!(document.diagnostics().is_empty(), "{version}");
        assert_eq!(
            json(&document.value().to_json().unwrap()),
            json(&text),
            "{version}"
        );
        let upgraded = history::water::upgrade(document, &LoadContext::detached()).unwrap();
        assert_eq!(upgraded.value(), loads::<WaterSettings>(&text).value());
    }
}

// ---- atmosphere ----------------------------------------------------------------------------

#[test]
fn a01_the_vanilla_atmosphere_file_upgrades_unchanged() {
    let text = fixture("atmosphere/a01_vanilla_default.json");
    let document = loads::<AtmosphereSettings>(&text);
    assert_eq!(document.version().parsed_as, version(1, 21, 40));
    assert!(document.version().upgraded);
    assert!(
        document.diagnostics().is_empty(),
        "{:?}",
        document.diagnostics()
    );
    // The same text stamped 1.26.20 is the same value.
    let stamped = loads::<AtmosphereSettings>(&text.replace("1.21.40", "1.26.20"));
    assert_eq!(stamped.value(), document.value());
    // Keys are sorted by time.
    let Some(Keyframes::Frames(start)) = document.horizon_blend_stops.clone().unwrap().start else {
        panic!("frames")
    };
    let times: Vec<f32> = start.iter().map(|frame| frame.time).collect();
    assert_eq!(times, [0.0, 0.25, 0.300_912, 0.75, 0.827_004, 1.0]);
    round_trip(document.value());
}

#[test]
fn a02_every_member_takes_a_single_value() {
    let text = atmosphere(
        "1.26.20",
        r##""horizon_blend_stops": {"min": 0.1, "start": 0.2, "mie_start": 1.2, "max": 1},
            "rayleigh_strength": 50, "sun_mie_strength": 60, "moon_mie_strength": 20,
            "sun_glare_shape": 50, "sky_zenith_color": "#010203", "sky_horizon_color": [4, 5, 6]"##,
    );
    let document = loads::<AtmosphereSettings>(&text);
    assert!(document.diagnostics().is_empty());
    assert_eq!(document.rayleigh_strength, Some(Keyframes::Constant(50.0)));
    assert_eq!(
        document.sky_zenith_color,
        Some(Keyframes::Constant(Rgb8::new(1, 2, 3)))
    );
    assert_eq!(
        document.sky_horizon_color.clone().unwrap().frames(),
        [Keyframe {
            time: 0.0,
            value: Rgb8::new(4, 5, 6)
        }]
    );
    assert_eq!(
        document.horizon_blend_stops.clone().unwrap().mie_start,
        Some(Keyframes::Constant(1.2))
    );
    round_trip(document.value());
}

#[test]
fn a03_an_absent_member_alone_takes_the_games_keyframes() {
    let document =
        loads::<AtmosphereSettings>(&atmosphere("1.26.20", r#""sky_zenith_color": [9, 9, 9]"#));
    assert_eq!(
        document.sky_zenith_color(),
        Keyframes::Constant(Rgb8::new(9, 9, 9))
    );
    let horizon = document.sky_horizon_color().frames();
    assert_eq!(horizon.len(), 11);
    assert_eq!(horizon[0].value, Rgb8::new(255, 255, 254));
    assert_eq!(
        horizon[10],
        Keyframe {
            time: 0.84,
            value: Rgb8::new(242, 168, 152)
        }
    );
    let value = |keyframes: Keyframes<f32>| -> Vec<f32> {
        keyframes.frames().iter().map(|frame| frame.value).collect()
    };
    let time = |keyframes: Keyframes<f32>| -> Vec<f32> {
        keyframes.frames().iter().map(|frame| frame.time).collect()
    };
    assert_eq!(
        value(document.rayleigh_strength()),
        [10.26125, 1.625, 1.625, 1.625, 1.625, 5.0, 1.98254, 1.25]
    );
    assert_eq!(
        time(document.rayleigh_strength()),
        [0.0, 0.25, 0.3, 0.30375, 0.325, 0.6, 0.7075, 0.75]
    );
    assert_eq!(
        value(document.sun_mie_strength()),
        [20.0, 60.0, 60.0, 57.57, 20.0, 20.0, 20.0, 20.0]
    );
    assert_eq!(
        value(document.moon_mie_strength()),
        [0.0, 0.0, 0.0, 2.28, 20.0, 20.0, 0.0, 0.0]
    );
    assert_eq!(
        value(document.sun_glare_shape()),
        [15.899, 15.899, 15.899, 15.8444, 15.0, 4.0, 4.0, 4.0]
    );
    let stops = document.horizon_blend_stops();
    assert_eq!(
        time(stops.min()),
        [0.0, 0.19625, 0.25, 0.34, 0.6, 0.675, 0.74875, 0.8175]
    );
    assert_eq!(
        value(stops.min()),
        [0.0, 0.008, 0.0, 0.11, 0.027, 0.01333, 0.0, 0.0]
    );
    assert_eq!(
        value(stops.start()),
        [0.25, 0.41791, 0.213, 0.401, 0.23, 0.21431, 0.19901, 0.618]
    );
    assert_eq!(
        value(stops.mie_start()),
        [0.5, 1.172, 0.5, 0.501, 0.167, 0.15, 0.55, 0.79]
    );
    assert_eq!(
        value(stops.max()),
        [0.25, 0.137, 0.0, 0.467, 0.2, 0.1919, 0.214, 0.0]
    );
    let zenith = loads::<AtmosphereSettings>(&atmosphere("1.26.20", ""))
        .sky_zenith_color()
        .frames();
    assert_eq!(
        zenith,
        [
            Keyframe {
                time: 0.0,
                value: Rgb8::new(0, 125, 164)
            },
            Keyframe {
                time: 0.25,
                value: Rgb8::new(11, 55, 97)
            },
            Keyframe {
                time: 0.5,
                value: Rgb8::new(7, 10, 36)
            },
            Keyframe {
                time: 0.75,
                value: Rgb8::new(50, 88, 170)
            },
        ]
    );
    // Only the member written is written back.
    assert_eq!(
        json(&document.to_json().unwrap()),
        json(&atmosphere("1.26.20", r#""sky_zenith_color": [9, 9, 9]"#))
    );
}

#[test]
fn a04_a3_each_atmosphere_range_is_checked_at_both_ends() {
    let cases: [(&str, [&str; 2], [&str; 2]); 8] = [
        ("rayleigh_strength", ["50.5", "-0.1"], ["0", "50"]),
        ("sun_mie_strength", ["61", "-1"], ["0", "60"]),
        ("moon_mie_strength", ["20.5", "-1"], ["0", "20"]),
        ("sun_glare_shape", ["50.5", "-1"], ["0", "50"]),
        ("horizon_blend_stops/min", ["1.1", "-0.1"], ["0", "1"]),
        ("horizon_blend_stops/start", ["1.1", "-0.1"], ["0", "1"]),
        (
            "horizon_blend_stops/mie_start",
            ["1.3", "-0.1"],
            ["0", "1.2"],
        ),
        ("horizon_blend_stops/max", ["1.1", "-0.1"], ["0", "1"]),
    ];
    for (member, bad, good) in cases {
        let text = |value: &str| match member.strip_prefix("horizon_blend_stops/") {
            Some(stop) => atmosphere(
                "1.26.20",
                &format!(r#""horizon_blend_stops": {{"{stop}": {{"0.5": {value}}}}}"#),
            ),
            None => atmosphere("1.26.20", &format!(r#""{member}": {{"0.5": {value}}}"#)),
        };
        for value in bad {
            // The finding's context is the key.
            out_of_range(
                &one_error::<AtmosphereSettings>(&text(value)),
                &format!("{ATMOSPHERE}/{member}/0.5"),
            );
        }
        for value in good {
            let document = loads::<AtmosphereSettings>(&text(value));
            assert!(document.diagnostics().is_empty(), "{member} {value}");
        }
    }
    // The same at 1.21.40.
    out_of_range(
        &one_error::<AtmosphereSettings>(&atmosphere("1.21.40", r#""sun_mie_strength": 61"#)),
        "minecraft:atmosphere_settings/sun_mie_strength",
    );
}

#[test]
fn a07_a1_a2_description_is_required_from_1_26_20_deviation() {
    assert_eq!(
        at(&[one_error::<AtmosphereSettings>(&doc(
            "1.26.20", ATMOSPHERE, ""
        ))]),
        [(
            "MissingRequiredField",
            "minecraft:atmosphere_settings/description"
        )]
    );
    // vanilla = "empty identifier" at 1.21.40.
    let document = loads::<AtmosphereSettings>(&doc("1.21.40", ATMOSPHERE, ""));
    assert!(document.description.is_none());
    assert_eq!(
        infos(document.diagnostics()),
        [finding(
            "MissingIdentifier",
            ATMOSPHERE,
            "no usable identifier"
        )]
    );
    assert!(document.to_json().is_err());
    for version in ["1.21.40", "1.26.20"] {
        assert_eq!(
            at(&[one_error::<AtmosphereSettings>(&doc(
                version,
                ATMOSPHERE,
                r#""description": {}"#
            ))]),
            [(
                "MissingRequiredField",
                "minecraft:atmosphere_settings/description/identifier"
            )]
        );
    }
}

#[test]
fn the_1_21_40_atmosphere_struct_writes_what_it_read() {
    let text = atmosphere("1.21.40", r#""rayleigh_strength": {"0.5": 2.0}, "more": 1"#);
    let document = AtmosphereSettings::parse_as_written(
        &PackPath::from_static("atmospherics/x.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .unwrap();
    assert!(matches!(
        document.value(),
        history::atmosphere::Any::V1_21_40(_)
    ));
    assert_eq!(json(&document.value().to_json().unwrap()), json(&text));
    let upgraded = history::atmosphere::upgrade(document, &LoadContext::detached()).unwrap();
    assert_eq!(upgraded.value(), loads::<AtmosphereSettings>(&text).value());
    assert!(upgraded.to_json().unwrap().contains("\"more\""));
}

// ---- color_grading -------------------------------------------------------------------------

#[test]
fn g01_a_1_21_40_file_gets_the_temperature_switched_off() {
    let document =
        upgrades::<ColorGradingSettings>("color_grading/g01_1_21_40_temperature_inserted");
    assert_eq!(document.version().parsed_as, version(1, 21, 40));
    assert!(
        document.diagnostics().is_empty(),
        "{:?}",
        document.diagnostics()
    );
    let temperature = document.color_grading.clone().unwrap().temperature.unwrap();
    assert_eq!(temperature.enabled, Some(false));
    assert_eq!(temperature.temperature, Some(6500.0));
    assert_eq!(temperature.kind, Some(TemperatureType::WhiteBalance));
    // Also when the file has no `color_grading` at all.
    let bare = loads::<ColorGradingSettings>(&grading("1.21.40", ""));
    assert_eq!(
        bare.color_grading.clone().unwrap().temperature,
        Some(temperature)
    );
}

#[test]
fn g02_the_vanilla_colour_grading_file_loads_unchanged() {
    let text = fixture("color_grading/g02_vanilla_1_21_90.json");
    let document = loads::<ColorGradingSettings>(&text);
    assert!(!document.version().upgraded);
    assert!(
        document.diagnostics().is_empty(),
        "{:?}",
        document.diagnostics()
    );
    let grading = document.color_grading.clone().unwrap();
    assert_eq!(
        grading.midtones.clone().unwrap().contrast,
        Some(Vec3::splat(1.15))
    );
    let temperature = grading.temperature.unwrap();
    assert_eq!(temperature.enabled, Some(true));
    assert_eq!(temperature.kind, Some(TemperatureType::ColorTemperature));
    assert_eq!(
        document.tone_mapping.clone().unwrap().operator,
        Some(ToneMappingOperator::Generic)
    );
    round_trip(document.value());
}

/// A colour-grading file with `members` in the tonal range `range`.
fn tones(range: &str, members: &str) -> String {
    grading(
        "1.21.90",
        &format!(r#""color_grading": {{"{range}": {{{members}}}}}"#),
    )
}

#[test]
fn g03_g1_g2_each_grading_range_is_checked_at_both_ends() {
    let d = rejects::<ColorGradingSettings>(&grading(
        "1.21.90",
        r#""color_grading": {"midtones": {"contrast": [1, 5, 1]}, "highlights": {"highlightsMin": 21},
           "shadows": {"shadowsMax": 1.1}, "temperature": {"temperature": 900}}"#,
    ));
    assert_eq!(
        at(&errors(&d)),
        [
            (
                "InvalidValue",
                "minecraft:color_grading_settings/color_grading/midtones/contrast/1"
            ),
            (
                "InvalidValue",
                "minecraft:color_grading_settings/color_grading/highlights/highlightsMin"
            ),
            (
                "InvalidValue",
                "minecraft:color_grading_settings/color_grading/shadows/shadowsMax"
            ),
            (
                "InvalidValue",
                "minecraft:color_grading_settings/color_grading/temperature/temperature"
            ),
        ]
    );
    for range in ["midtones", "highlights", "shadows"] {
        for (member, bad, good) in [
            ("contrast", ["4.1", "-0.1"], ["0", "4"]),
            ("gain", ["10.1", "-0.1"], ["0", "10"]),
            ("gamma", ["4.1", "-0.1"], ["0", "4"]),
            ("offset", ["1.1", "-1.1"], ["-1", "1"]),
            ("saturation", ["10.1", "-0.1"], ["0", "10"]),
        ] {
            // Per component: each index is checked.
            for index in 0..3 {
                for value in bad {
                    let mut components = ["1", "1", "1"];
                    components[index] = value;
                    out_of_range(
                        &one_error::<ColorGradingSettings>(&tones(
                            range,
                            &format!(r#""{member}": [{}]"#, components.join(", ")),
                        )),
                        &format!("{GRADING}/color_grading/{range}/{member}/{index}"),
                    );
                }
            }
            for value in good {
                let document = loads::<ColorGradingSettings>(&tones(
                    range,
                    &format!(r#""{member}": [{value}, {value}, {value}]"#),
                ));
                assert!(
                    document.diagnostics().is_empty(),
                    "{range} {member} {value}"
                );
            }
        }
    }
    for (range, member, bad, good) in [
        ("highlights", "highlightsMin", ["0.9", "20.1"], ["1", "20"]),
        ("shadows", "shadowsMax", ["-0.1", "1.1"], ["0", "1"]),
        (
            "temperature",
            "temperature",
            ["999", "15001"],
            ["1000", "15000"],
        ),
    ] {
        for value in bad {
            out_of_range(
                &one_error::<ColorGradingSettings>(&tones(
                    range,
                    &format!(r#""{member}": {value}"#),
                )),
                &format!("{GRADING}/color_grading/{range}/{member}"),
            );
        }
        for value in good {
            let document =
                loads::<ColorGradingSettings>(&tones(range, &format!(r#""{member}": {value}"#)));
            assert!(document.diagnostics().is_empty(), "{member} {value}");
        }
    }
    // The 1.21.40 struct holds the same bounds.
    out_of_range(
        &one_error::<ColorGradingSettings>(&grading(
            "1.21.40",
            r#""color_grading": {"highlights": {"highlightsMin": 21}}"#,
        )),
        "minecraft:color_grading_settings/color_grading/highlights/highlightsMin",
    );
}

#[test]
fn g04_the_binarys_bounds_are_wider_than_the_documentations() {
    let document = loads::<ColorGradingSettings>(&grading(
        "1.21.90",
        r#""color_grading": {"highlights": {"highlightsMin": 10}, "shadows": {"shadowsMax": 0.05}}"#,
    ));
    assert!(document.diagnostics().is_empty());
    let grading = document.color_grading.clone().unwrap();
    assert_eq!(grading.highlights.unwrap().highlights_min, Some(10.0));
    assert_eq!(grading.shadows.unwrap().shadows_max, Some(0.05));
}

#[test]
fn g05_g3_an_unknown_operator_or_temperature_type_is_an_enum_error() {
    assert_eq!(
        at(&[one_error::<ColorGradingSettings>(&grading(
            "1.21.90",
            r#""tone_mapping": {"operator": "filmic"}"#
        ))]),
        [(
            "InvalidEnumValue",
            "minecraft:color_grading_settings/tone_mapping/operator"
        )]
    );
    assert_eq!(
        at(&[one_error::<ColorGradingSettings>(&tones(
            "temperature",
            r#""type": "kelvin""#
        ))]),
        [(
            "InvalidEnumValue",
            "minecraft:color_grading_settings/color_grading/temperature/type"
        )]
    );
    for (name, operator) in [
        ("reinhard", ToneMappingOperator::Reinhard),
        ("reinhard_luma", ToneMappingOperator::ReinhardLuma),
        ("reinhard_luminance", ToneMappingOperator::ReinhardLuminance),
        ("hable", ToneMappingOperator::Hable),
        ("aces", ToneMappingOperator::Aces),
        ("generic", ToneMappingOperator::Generic),
    ] {
        let text = grading(
            "1.21.90",
            &format!(r#""tone_mapping": {{"operator": "{name}"}}"#),
        );
        let document = loads::<ColorGradingSettings>(&text);
        assert_eq!(
            document.tone_mapping.clone().unwrap().operator,
            Some(operator)
        );
        assert_eq!(json(&round_trip(document.value())), json(&text));
    }
    for (name, kind) in [
        ("white_balance", TemperatureType::WhiteBalance),
        ("color_temperature", TemperatureType::ColorTemperature),
    ] {
        let text = tones("temperature", &format!(r#""type": "{name}""#));
        let document = loads::<ColorGradingSettings>(&text);
        assert_eq!(
            document
                .color_grading
                .clone()
                .unwrap()
                .temperature
                .unwrap()
                .kind,
            Some(kind)
        );
        assert_eq!(json(&round_trip(document.value())), json(&text));
    }
}

#[test]
fn g06_g4_temperature_at_1_21_40_is_unknown_with_the_version_it_needs() {
    let document = loads::<ColorGradingSettings>(&grading(
        "1.21.40",
        r#""color_grading": {"temperature": {"enabled": true, "temperature": 3000}}"#,
    ));
    assert_eq!(
        infos(document.diagnostics()),
        [finding(
            "UnrecognizedField",
            "minecraft:color_grading_settings/color_grading/temperature",
            "temperature needs format_version 1.21.90"
        )]
    );
    // The step's own value stands: switched off, 6500.
    let temperature = document.color_grading.clone().unwrap().temperature.unwrap();
    assert_eq!(
        (temperature.enabled, temperature.temperature),
        (Some(false), Some(6500.0))
    );
    round_trip(&loads::<ColorGradingSettings>(&document.to_json().unwrap()).into_value());
}

#[test]
fn g07_g6_description_and_its_identifier_are_optional_deviation() {
    // vanilla = "empty identifier".
    for members in ["", r#""description": {}"#] {
        for version in ["1.21.40", "1.21.90"] {
            let text = doc(version, GRADING, members);
            let document = loads::<ColorGradingSettings>(&text);
            assert_eq!(
                infos(document.diagnostics()),
                [finding(
                    "MissingIdentifier",
                    GRADING,
                    "no usable identifier"
                )],
                "{version} {members}"
            );
            assert!(errors(document.diagnostics()).is_empty());
        }
    }
    // Written back as it was: no identifier is invented.
    let text = doc("1.21.90", GRADING, r#""description": {}"#);
    assert_eq!(
        json(&loads::<ColorGradingSettings>(&text).to_json().unwrap()),
        json(&text)
    );
}

#[test]
fn the_games_grading_values_stand_in_for_absent_members() {
    let document = loads::<ColorGradingSettings>(&grading("1.21.90", r#""color_grading": {}"#));
    let grading = document.color_grading.clone().unwrap();
    let midtones = grading.midtones.clone().unwrap_or_default();
    assert_eq!(midtones.contrast(), Vec3::splat(1.0));
    assert_eq!(midtones.gain(), Vec3::splat(1.0));
    assert_eq!(midtones.gamma(), Vec3::splat(2.2));
    assert_eq!(midtones.offset(), Vec3::splat(0.0));
    assert_eq!(midtones.saturation(), Vec3::splat(1.0));
    let highlights = grading.highlights.clone().unwrap_or_default();
    assert!(!highlights.enabled());
    assert_eq!(highlights.highlights_min(), 1.0);
    assert_eq!(highlights.gamma(), Vec3::splat(2.2));
    let shadows = grading.shadows.clone().unwrap_or_default();
    assert!(!shadows.enabled());
    assert_eq!(shadows.shadows_max(), 1.0);
    let temperature = grading.temperature.clone().unwrap_or_default();
    assert!(!temperature.enabled());
    assert_eq!(temperature.temperature(), 6500.0);
    assert_eq!(temperature.kind(), TemperatureType::WhiteBalance);
    assert_eq!(
        document.tone_mapping.clone().unwrap_or_default().operator(),
        ToneMappingOperator::Reinhard
    );
}

#[test]
fn the_1_21_40_grading_struct_writes_what_it_read() {
    let text = grading(
        "1.21.40",
        r#""color_grading": {"midtones": {"gain": [1.0, 2.0, 3.0]}, "highlights": {"enabled": true}}, "tone_mapping": {"operator": "hable"}"#,
    );
    let document = ColorGradingSettings::parse_as_written(
        &PackPath::from_static("color_grading/x.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .unwrap();
    assert!(matches!(
        document.value(),
        history::color_grading::Any::V1_21_40(_)
    ));
    assert_eq!(json(&document.value().to_json().unwrap()), json(&text));
    let upgraded = history::color_grading::upgrade(document, &LoadContext::detached()).unwrap();
    assert_eq!(
        upgraded.value(),
        loads::<ColorGradingSettings>(&text).value()
    );
}

// ---- the drop-4 files of 1.26.60 --------------------------------------------------------

/// `experimental_y_2026_drop_4/{lighting/ice_caves_lighting.json (1.21.70),
/// atmospherics/ice_caves_atmospherics.json (1.21.40), color_grading/ice_caves_color_grading.json
/// (1.21.90)}` of 1.26.60 (all go through upgrade steps;
/// the colour grading's 1.21.90 is its newest struct's own format): each loads without a
/// finding and round-trips; the atmosphere stamped 1.26.20 is the same value.
#[test]
fn the_drop_4_ice_caves_files_load_and_round_trip() {
    let lighting = loads::<LightingSettings>(&fixture("drop4/ice_caves_lighting.json"));
    assert_eq!(lighting.version().parsed_as, version(1, 21, 70));
    assert!(lighting.version().upgraded);
    assert!(
        lighting.diagnostics().is_empty(),
        "{:?}",
        lighting.diagnostics()
    );
    round_trip(lighting.value());

    let text = fixture("drop4/ice_caves_atmospherics.json");
    let atmosphere = loads::<AtmosphereSettings>(&text);
    assert_eq!(atmosphere.version().parsed_as, version(1, 21, 40));
    assert!(atmosphere.version().upgraded);
    assert!(
        atmosphere.diagnostics().is_empty(),
        "{:?}",
        atmosphere.diagnostics()
    );
    let stamped = loads::<AtmosphereSettings>(&text.replace("1.21.40", "1.26.20"));
    assert_eq!(stamped.value(), atmosphere.value());
    round_trip(atmosphere.value());

    let grading = loads::<ColorGradingSettings>(&fixture("drop4/ice_caves_color_grading.json"));
    assert_eq!(grading.version().parsed_as, version(1, 21, 90));
    assert!(!grading.version().upgraded);
    assert!(
        grading.diagnostics().is_empty(),
        "{:?}",
        grading.diagnostics()
    );
    round_trip(grading.value());
}
