//! Write-back and the placement of each root type: a value
//! built in code is written as text this loader reads back, or refused with the member's path.

use bedrock_addon_core::document::{Content, Placed, UnknownMembers};
use bedrock_addon_core::id::{
    AtmosphereSettingsId, BlockId, ColorGradingSettingsId, CubemapSettingsId, LightingSettingsId,
    WaterSettingsId,
};
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_family_deferred_settings::{
    AmbientLight, AtmosphereSettings, Caustics, ColorGrading, ColorGradingSettings,
    CubemapLighting, CubemapSettings, DirectionalLights, Emissive, Flash, Keyframe, Keyframes,
    LightType, LightingSettings, LocalLight, LocalLightSettings, Mer, Midtones, OrbitalLight,
    OrbitalLights, PbrFallback, PbrFallbackSettings, PbrValue, Rgb8, ShadowSettings, ShadowStyle,
    Sky, Temperature, Vec3, WaterSettings, Waves,
};

use super::common::*;

/// The path of the member `value`'s writer refuses.
#[track_caller]
fn refused<T: Content>(value: &T) -> String {
    match value.to_json() {
        Err(EncodeError::NotRepresentable { at, .. }) => at.to_string(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_constructors_write_documents_that_read_back() {
    round_trip(&LightingSettings::new(LightingSettingsId::parse("t:l")));
    round_trip(&WaterSettings::new(WaterSettingsId::parse("t:w")));
    round_trip(&AtmosphereSettings::new(AtmosphereSettingsId::parse("t:a")));
    round_trip(&ColorGradingSettings::new(ColorGradingSettingsId::parse(
        "t:g",
    )));
    round_trip(&CubemapSettings::new(CubemapSettingsId::parse("t:c")));
    round_trip(&ShadowSettings::default());
    round_trip(&PbrFallbackSettings::default());
    round_trip(&LocalLightSettings::default());
    assert_eq!(
        json(
            &WaterSettings::new(WaterSettingsId::parse("t:w"))
                .to_json()
                .unwrap()
        ),
        json(&doc("1.26.20", WATER, &named("t:w", "")))
    );
}

#[test]
fn a_full_lighting_value_round_trips() {
    let frames = |values: &[(f32, f32)]| {
        Keyframes::Keys(
            values
                .iter()
                .map(|&(time, value)| Keyframe { time, value })
                .collect(),
        )
    };
    let lighting = LightingSettings {
        directional_lights: Some(DirectionalLights {
            orbital: Some(OrbitalLights {
                sun: Some(OrbitalLight {
                    illuminance: Some(frames(&[(0.0, 110_000.0), (0.5, 0.0), (1.0, 0.25)])),
                    color: Some(Keyframes::Single(Rgb8::new(1, 2, 3))),
                    extra: UnknownMembers::default(),
                }),
                moon: Some(OrbitalLight::default()),
                orbital_offset_degrees: Some(Keyframes::Single(359.99)),
                extra: UnknownMembers::default(),
            }),
            flash: Some(Flash {
                color: Some(Rgb8::WHITE),
                illuminance: Some(-3.5),
                extra: UnknownMembers::default(),
            }),
            extra: UnknownMembers::default(),
        }),
        emissive: Some(Emissive {
            desaturation: Some(1.0),
            extra: UnknownMembers::default(),
        }),
        ambient: Some(AmbientLight::default()),
        sky: Some(Sky {
            intensity: Some(frames(&[(0.25, 0.1), (0.75, 1.0)])),
            extra: UnknownMembers::default(),
        }),
        ..LightingSettings::new(LightingSettingsId::parse("t:l"))
    };
    let written = round_trip(&lighting);
    // Numbers are written as the shortest text that reads back to the same `f32`.
    assert!(written.contains("359.99"), "{written}");
    assert!(!written.contains("359.989"), "{written}");
    assert!(written.contains("0.02"), "{written}");
    assert!(!written.contains("0.0199"), "{written}");
}

#[test]
fn a_value_outside_its_range_is_refused_with_its_path() {
    let mut shadows = ShadowSettings {
        texel_size: Some(0),
        ..ShadowSettings::default()
    };
    assert_eq!(refused(&shadows), "minecraft:shadow_settings/texel_size");
    shadows.texel_size = Some(1025);
    assert_eq!(refused(&shadows), "minecraft:shadow_settings/texel_size");
    shadows.texel_size = Some(1024);
    shadows.shadow_style = Some(ShadowStyle::BlockyShadows);
    round_trip(&shadows);

    let water = |waves: Waves| WaterSettings {
        waves: Some(waves),
        ..WaterSettings::new(WaterSettingsId::parse("t:w"))
    };
    assert_eq!(
        refused(&water(Waves {
            octaves: Some(31),
            ..Waves::default()
        })),
        "minecraft:water_settings/waves/octaves"
    );
    assert_eq!(
        refused(&water(Waves {
            pull: Some(-1.5),
            ..Waves::default()
        })),
        "minecraft:water_settings/waves/pull"
    );
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(
            refused(&water(Waves {
                mix: Some(bad),
                ..Waves::default()
            })),
            "minecraft:water_settings/waves/mix"
        );
    }
    round_trip(&water(Waves {
        octaves: Some(30),
        pull: Some(-1.0),
        mix: Some(1.0),
        ..Waves::default()
    }));
    let caustics = WaterSettings {
        caustics: Some(Caustics {
            power: Some(7.0),
            ..Caustics::default()
        }),
        ..WaterSettings::new(WaterSettingsId::parse("t:w"))
    };
    assert_eq!(
        refused(&caustics),
        "minecraft:water_settings/caustics/power"
    );

    let grading = ColorGradingSettings {
        color_grading: Some(ColorGrading {
            midtones: Some(Midtones {
                gain: Some(Vec3::new(1.0, 11.0, 1.0)),
                ..Midtones::default()
            }),
            temperature: Some(Temperature {
                temperature: Some(999.0),
                ..Temperature::default()
            }),
            ..ColorGrading::default()
        }),
        ..ColorGradingSettings::default()
    };
    assert_eq!(
        refused(&grading),
        "minecraft:color_grading_settings/color_grading/midtones/gain/1"
    );

    let cubemap = CubemapSettings {
        lighting: Some(CubemapLighting {
            ambient_light_illuminance: Some(Keyframes::Keys(vec![
                Keyframe {
                    time: 0.0,
                    value: 1.0,
                },
                Keyframe {
                    time: 0.5,
                    value: 100_001.0,
                },
            ])),
            ..CubemapLighting::default()
        }),
        ..CubemapSettings::default()
    };
    assert_eq!(
        refused(&cubemap),
        "minecraft:cubemap_settings/lighting/ambient_light_illuminance/0.5"
    );
}

#[test]
fn an_unbounded_number_that_is_not_finite_is_refused() {
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let lighting = LightingSettings {
            directional_lights: Some(DirectionalLights {
                flash: Some(Flash {
                    illuminance: Some(bad),
                    ..Flash::default()
                }),
                ..DirectionalLights::default()
            }),
            ..LightingSettings::default()
        };
        assert_eq!(
            refused(&lighting),
            "minecraft:lighting_settings/directional_lights/flash/illuminance"
        );
    }
}

/// Cubemap settings whose `ambient_light_illuminance` is `keyframes`.
fn keyframed(keyframes: Keyframes<f32>) -> CubemapSettings {
    CubemapSettings {
        lighting: Some(CubemapLighting {
            ambient_light_illuminance: Some(keyframes),
            ..CubemapLighting::default()
        }),
        ..CubemapSettings::new(CubemapSettingsId::parse("t:c"))
    }
}

#[test]
fn keyframes_the_loader_refuses_are_not_written() {
    const AT: &str = "minecraft:cubemap_settings/lighting/ambient_light_illuminance";
    let frame = |time: f32| Keyframe { time, value: 1.0 };
    // No keyframe at all: the loader refuses an empty object.
    assert_eq!(refused(&keyframed(Keyframes::Keys(Vec::new()))), AT);
    // A time outside the day would be dropped on load.
    for time in [1.5, -0.25, f32::NAN, f32::INFINITY] {
        assert_eq!(
            refused(&keyframed(Keyframes::Keys(vec![frame(0.0), frame(time)]))),
            AT,
            "{time}"
        );
    }
    // Two keyframes of one time would be one member.
    assert_eq!(
        refused(&keyframed(Keyframes::Keys(vec![
            frame(0.5),
            frame(0.25),
            frame(0.5)
        ]))),
        AT
    );
    // Both ends of the day are written; unsorted keyframes read back sorted.
    let written = keyframed(Keyframes::Keys(vec![frame(1.0), frame(0.0)]))
        .to_json()
        .unwrap();
    let read = loads::<CubemapSettings>(&written);
    assert!(read.diagnostics().is_empty());
    assert_eq!(
        read.lighting.clone().unwrap().ambient_light_illuminance,
        Some(Keyframes::Keys(vec![frame(0.0), frame(1.0)]))
    );
}

#[test]
fn a_value_without_the_description_its_newest_format_requires_is_refused() {
    assert_eq!(
        refused(&WaterSettings::default()),
        "minecraft:water_settings/description"
    );
    assert_eq!(
        refused(&AtmosphereSettings::default()),
        "minecraft:atmosphere_settings/description"
    );
    // The other three are optional in their newest format.
    round_trip_unnamed(&LightingSettings::default());
    round_trip_unnamed(&ColorGradingSettings::default());
    round_trip_unnamed(&CubemapSettings::default());
}

/// `value` is written and reads back equal, with the one `MissingIdentifier` information
/// (ours).
#[track_caller]
fn round_trip_unnamed<T: Content + std::fmt::Debug + PartialEq>(value: &T) {
    let written = value.to_json().unwrap();
    let read = loads::<T>(&written);
    assert_eq!(read.value(), value);
    assert_eq!(at(&infos(read.diagnostics())).len(), 1);
    assert_eq!(read.to_json().unwrap(), written);
}

#[test]
fn an_unknown_member_named_like_a_known_one_is_refused() {
    let mut extra = UnknownMembers::default();
    extra.insert("texel_size".to_owned(), Value::from(4_i64));
    let shadows = ShadowSettings {
        extra,
        ..ShadowSettings::default()
    };
    assert_eq!(refused(&shadows), "minecraft:shadow_settings/texel_size");
    let mut extra = UnknownMembers::default();
    extra.insert("light_color".to_owned(), Value::from("red"));
    let mut lights = LocalLightSettings::default();
    lights.lights.insert(
        BlockId::parse("minecraft:torch"),
        LocalLight {
            extra,
            ..LocalLight::default()
        },
    );
    assert_eq!(
        refused(&lights),
        "minecraft:local_light_settings/minecraft:torch/light_color"
    );
    // Any other unknown member is written after the known ones and reads back.
    let mut extra = UnknownMembers::default();
    extra.insert("note".to_owned(), Value::from("x"));
    let shadows = ShadowSettings {
        texel_size: Some(8),
        extra,
        ..ShadowSettings::default()
    };
    let written = shadows.to_json().unwrap();
    assert_eq!(
        json(&written),
        json(&doc("1.21.80", SHADOWS, r#""texel_size": 8, "note": "x""#))
    );
    assert_eq!(loads::<ShadowSettings>(&written).value(), &shadows);
}

#[test]
fn both_pbr_forms_and_local_lights_built_by_hand_round_trip() {
    let pbr = PbrFallbackSettings {
        blocks: Some(PbrFallback {
            value: Some(PbrValue::Mer(Mer {
                metalness: 1,
                emissive: 2,
                roughness: 3,
            })),
            ..PbrFallback::default()
        }),
        items: Some(PbrFallback::default()),
        ..PbrFallbackSettings::default()
    };
    round_trip(&pbr);
    let mut lights = LocalLightSettings::default();
    lights.lights.insert(
        BlockId::parse("demo:lamp"),
        LocalLight {
            light_type: Some(LightType::PointLight),
            light_color: Some(Rgb8::new(1, 2, 3)),
            ..LocalLight::default()
        },
    );
    lights
        .lights
        .insert(BlockId::parse("demo:other"), LocalLight::default());
    assert_eq!(
        json(&round_trip(&lights)),
        json(&doc(
            "1.21.120",
            LOCAL,
            r#""demo:lamp": {"light_type": "point_light", "light_color": [1, 2, 3]}, "demo:other": {}"#
        ))
    );
}

#[test]
fn each_root_type_has_its_default_path() {
    assert_eq!(
        LightingSettings::new(LightingSettingsId::parse("demo:night"))
            .default_path()
            .as_str(),
        "lighting/demo_night.json"
    );
    assert_eq!(
        WaterSettings::new(WaterSettingsId::parse("demo:sea"))
            .default_path()
            .as_str(),
        "water/demo_sea.json"
    );
    assert_eq!(
        AtmosphereSettings::new(AtmosphereSettingsId::parse("demo:haze"))
            .default_path()
            .as_str(),
        "atmospherics/demo_haze.json"
    );
    assert_eq!(
        ColorGradingSettings::new(ColorGradingSettingsId::parse("demo:warm"))
            .default_path()
            .as_str(),
        "color_grading/demo_warm.json"
    );
    assert_eq!(
        CubemapSettings::new(CubemapSettingsId::parse("demo:stars"))
            .default_path()
            .as_str(),
        "cubemaps/demo_stars.json"
    );
    // A value without an identifier is named after its payload.
    assert_eq!(
        LightingSettings::default().default_path().as_str(),
        "lighting/lighting.json"
    );
    assert_eq!(
        ColorGradingSettings::default().default_path().as_str(),
        "color_grading/color_grading.json"
    );
    assert_eq!(
        ShadowSettings::default().default_path().as_str(),
        "shadows/global.json"
    );
    assert_eq!(
        PbrFallbackSettings::default().default_path().as_str(),
        "pbr/global.json"
    );
    assert_eq!(
        LocalLightSettings::default().default_path().as_str(),
        "local_lighting/local_lighting.json"
    );
}

#[test]
fn an_unknown_number_beyond_a_double_is_kept_as_the_largest_finite_one_with_a_warning_deviation() {
    // Ours: the kept value has no form for `1e999`, and an infinity has none in JSON.
    let text = doc(
        "1.21.80",
        SHADOWS,
        r#""texel_size": 8, "zz": 1e999, "yy": {"deep": [1, -1e999, {"n": 1e400}], "s": "1e999"}, "ok": 1e39"#,
    );
    let document = loads::<ShadowSettings>(&text);
    let key = "minecraft:shadow_settings";
    // Each unknown member, and after it the numbers in it that were replaced, each at its own
    // place in the text.
    let found: Vec<(&str, String, &str)> = document
        .diagnostics()
        .iter()
        .map(|f| {
            let span = f.location.span.expect("a span");
            let (from, len) = (span.offset as usize, span.len as usize);
            (
                f.code.id(),
                f.location.json_path.to_string(),
                &text[from..from + len],
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            ("core/unrecognized_field", format!("{key}/zz"), "1e999"),
            ("core/invalid_value", format!("{key}/zz"), "1e999"),
            (
                "core/unrecognized_field",
                format!("{key}/yy"),
                r#"{"deep": [1, -1e999, {"n": 1e400}], "s": "1e999"}"#
            ),
            ("core/invalid_value", format!("{key}/yy/deep/1"), "-1e999"),
            ("core/invalid_value", format!("{key}/yy/deep/2/n"), "1e400"),
            ("core/unrecognized_field", format!("{key}/ok"), "1e39"),
        ]
    );
    // The three unknown members and the three replaced numbers are all informations of ours
    // (the game is silent on them).
    assert_eq!(warnings(document.diagnostics()).len(), 0);
    assert_eq!(infos(document.diagnostics()).len(), 6);
    assert_eq!(document.extra.get("zz"), Some(&Value::from(f64::MAX)));
    assert_eq!(
        document.extra.get("yy"),
        Some(&json(
            r#"{"deep": [1, -1.7976931348623157e308, {"n": 1.7976931348623157e308}], "s": "1e999"}"#
        ))
    );
    // A number a double holds is kept as written.
    assert_eq!(document.extra.get("ok"), Some(&Value::from(1e39_f64)));
    // The document is written, and what is written reads back equal without a replacement.
    let written = document.to_json().unwrap();
    let again = loads::<ShadowSettings>(&written);
    assert_eq!(again.value(), document.value());
    assert_eq!(
        at(&infos(again.diagnostics())),
        [
            ("core/unrecognized_field", format!("{key}/zz").as_str()),
            ("core/unrecognized_field", format!("{key}/yy").as_str()),
            ("core/unrecognized_field", format!("{key}/ok").as_str()),
        ]
    );
    assert_eq!(again.to_json().unwrap(), written);
}
