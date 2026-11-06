//! The upgrade steps of the deferred settings, each run on a hand-built input, and the eight
//! chains.

use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::testing::{Checks, Run, StepOutput, step_output};
use bedrock_addon_core::versioning::{
    EnvelopeCheck, Family, MissingVersion, RangeModel, Selection, Upgrade, VersionChain,
    VersionStruct, check_chain, select,
};
use bedrock_addon_family_deferred_settings::history::{
    atmosphere, color_grading, cubemap, lighting, local_lighting, pbr_fallback, shadows, water,
};

fn v(major: u16, minor: u16, patch: u16) -> FormatVersion {
    FormatVersion::new(major, minor, patch)
}

/// The JSON at `path` (`/`-separated) of `value`.
fn at<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('/')
        .try_fold(value, |v, key| v.as_object().and_then(|o| o.get(key)))
}

/// `value` as JSON text.
fn text(value: &Value) -> String {
    facet_json::to_string(value).unwrap()
}

fn parsed(text: &str) -> Value {
    facet_json::from_str(text).unwrap()
}

/// Runs the step `S` on `input`.
#[track_caller]
fn run<S>(input: &Value) -> StepOutput
where
    S: Upgrade + VersionStruct,
    S::Next: VersionStruct,
{
    let checks = Checks {
        identifier: "test:x",
        ..Checks::default()
    };
    step_output::<S>(input, Run::default(), &checks).expect("the step runs")
}

/// The step copies every member of its base and reports nothing.
#[track_caller]
fn copies_every_member<S>(base: &Value)
where
    S: Upgrade + VersionStruct,
    S::Next: VersionStruct,
{
    let out = run::<S>(base);
    assert!(out.diagnostics.is_empty(), "the step reported something");
    // The 1.21.40 lighting step alone changes a copied member: the colours lose the alpha.
    let expected = parsed(
        &text(base)
            .replace("[1,2,3,255]", "[1,2,3]")
            .replace("[4,5,6,255]", "[4,5,6]"),
    );
    assert_eq!(out.payload, expected, "a member was not copied");
}

/// The step copies every member of its base and adds exactly `added` at `path`.
#[track_caller]
fn copies_and_adds<S>(base: &Value, path: &str, added: &str)
where
    S: Upgrade + VersionStruct,
    S::Next: VersionStruct,
{
    let out = run::<S>(base);
    assert!(out.diagnostics.is_empty(), "the step reported something");
    assert_eq!(
        at(&out.payload, path),
        Some(&parsed(added)),
        "the member was not added"
    );
    // Without the added member, the output is the base.
    let key = path.rsplit_once('/').map_or(path, |(_, key)| key);
    let written = text(&out.payload);
    let needle = format!("\"{key}\":{}", text(&parsed(added)));
    let without = written
        .replacen(&format!(",{needle}"), "", 1)
        .replacen(&needle, "", 1);
    assert_eq!(parsed(&without), *base, "a member was not copied");
}

/// The member at `path` of the base becomes one keyframe at `0.0` with the same value.
#[track_caller]
fn keyframes<S>(base: &Value, path: &str)
where
    S: Upgrade + VersionStruct,
    S::Next: VersionStruct,
{
    let out = run::<S>(base);
    let before = at(base, path).expect("the base has the member");
    let after = at(&out.payload, path).expect("the member is not gone");
    let frames = after.as_object().expect("the member is a keyframe object");
    assert!(
        frames.len() == 1 && frames.get("0.0") == Some(before),
        "the member is not one keyframe at 0.0 with its value"
    );
    assert!(out.diagnostics.is_empty(), "the step reported something");
}

#[test]
fn the_1_21_40_lighting_step_copies_every_member_but_the_colour_alpha() {
    let base = parsed(
        r#"{"description": {"identifier": "test:l"}, "directional_lights": {"sun": {"illuminance": 5.0, "color": [1, 2, 3, 255]}, "moon": {"illuminance": 0.5, "color": [4, 5, 6, 255]}, "orbital_offset_degrees": 10.0}, "emissive": {"desaturation": 0.5}, "ambient": {"color": [7, 8, 9], "illuminance": 0.5}}"#,
    );
    copies_every_member::<lighting::v1_21_40::LightingSettings>(&base);
}

#[test]
fn the_1_21_60_lighting_step_adds_the_sky_intensity() {
    let base = parsed(
        r#"{"description": {"identifier": "test:l"}, "directional_lights": {"sun": {"illuminance": 5.0, "color": [1, 2, 3]}, "orbital_offset_degrees": 10.0}, "emissive": {"desaturation": 0.5}, "ambient": {"color": [7, 8, 9], "illuminance": 0.5}}"#,
    );
    copies_and_adds::<lighting::v1_21_60::LightingSettings>(&base, "sky", r#"{"intensity": 1.0}"#);
}

/// 1.21.70 → 1.21.80 copies the other members and never writes `flash`, with or without
/// directional lights.
#[test]
fn the_1_21_70_lighting_step_copies_every_member_and_writes_no_flash() {
    let base = parsed(
        r#"{"description": {"identifier": "test:l"}, "emissive": {"desaturation": 0.5}, "ambient": {"color": [7, 8, 9], "illuminance": 0.5}, "sky": {"intensity": 0.5}}"#,
    );
    let with = parsed(&text(&base).replacen(
        '{',
        r#"{"directional_lights":{"sun":{"illuminance":5.0}},"#,
        1,
    ));
    for input in [&base, &with] {
        let out = run::<lighting::v1_21_70::LightingSettings>(input);
        assert!(
            at(&out.payload, "directional_lights/flash").is_none(),
            "flash was written"
        );
    }
    let out = run::<lighting::v1_21_70::LightingSettings>(&base);
    assert!(
        at(&out.payload, "directional_lights").is_none(),
        "directional lights were invented"
    );
    assert!(out.diagnostics.is_empty());
}

/// 1.21.70 → 1.21.80 moves the sun and the moon under `orbital` and keeps the other members.
#[test]
fn the_1_21_70_lighting_step_moves_the_sun_and_the_moon_under_orbital() {
    let base = parsed(
        r#"{"description": {"identifier": "test:l"}, "directional_lights": {"sun": {"illuminance": 5.0, "color": [1, 2, 3]}, "moon": {"illuminance": {"0.5": 0.25}}, "orbital_offset_degrees": 10.0}, "emissive": {"desaturation": 0.5}, "ambient": {"color": [7, 8, 9], "illuminance": 0.5}, "sky": {"intensity": 0.5}}"#,
    );
    let out = run::<lighting::v1_21_70::LightingSettings>(&base);
    assert!(out.diagnostics.is_empty());
    for path in ["emissive", "ambient", "sky", "description"] {
        assert_eq!(at(&out.payload, path), at(&base, path), "{path}");
    }
    assert_eq!(
        at(&out.payload, "directional_lights/orbital/sun"),
        at(&base, "directional_lights/sun")
    );
    assert_eq!(
        at(&out.payload, "directional_lights/orbital/moon"),
        at(&base, "directional_lights/moon")
    );
    assert_eq!(
        at(
            &out.payload,
            "directional_lights/orbital/orbital_offset_degrees"
        ),
        at(&base, "directional_lights/orbital_offset_degrees")
    );
    for old in ["sun", "moon", "orbital_offset_degrees"] {
        assert!(
            at(&out.payload, &format!("directional_lights/{old}")).is_none(),
            "{old} stays at the old place"
        );
    }
}

/// 1.21.80 → 1.26.0 turns the ambient colour and illuminance and the sky intensity into one
/// keyframe each at `0.0`.
#[test]
fn the_1_21_80_lighting_step_keyframes_the_ambient_light_and_the_sky() {
    let base = parsed(
        r#"{"description": {"identifier": "test:l"}, "directional_lights": {"orbital": {"sun": {"illuminance": 5.0}}, "flash": {"illuminance": 3.0, "color": [1, 2, 3]}}, "emissive": {"desaturation": 0.5}, "ambient": {"color": [7, 8, 9], "illuminance": 0.5}, "sky": {"intensity": 0.25}}"#,
    );
    for path in ["ambient/color", "ambient/illuminance", "sky/intensity"] {
        keyframes::<lighting::v1_21_80::LightingSettings>(&base, path);
    }
}

#[test]
fn the_water_steps_copy_every_member() {
    let one = parsed(
        r#"{"description": {"identifier": "test:w"}, "particle_concentrations": {"chlorophyll": 1.0, "suspended_sediment": 2.0, "cdom": 3.0}, "waves": {"enabled": true, "frequency": 1.5, "octaves": 10, "depth": 0.5, "sampleWidth": 0.25, "speed": 3.0, "pull": -0.5, "mix": 0.25, "frequency_scaling": 1.5, "speed_scaling": 1.25, "shape": 2.0}}"#,
    );
    copies_every_member::<water::v1_21_40::WaterSettings>(&one);
    let two = parsed(
        r#"{"description": {"identifier": "test:w"}, "particle_concentrations": {"chlorophyll": 1.0, "suspended_sediment": 2.0, "cdom": 3.0}, "caustics": {"enabled": false, "frame_length": 0.5, "texture": "textures/caustics", "scale": 2.0, "power": 3.0}, "waves": {"enabled": true, "frequency": 1.5, "octaves": 10, "depth": 0.5, "speed": 3.0, "pull": -0.5, "mix": 0.25, "frequency_scaling": 1.5, "speed_scaling": 1.25, "direction_increment": 45.0, "shape": 2.0}}"#,
    );
    copies_every_member::<water::v1_21_80::WaterSettings>(&two);
    copies_every_member::<water::v1_21_120::WaterSettings>(&two);
    let four = parsed(
        r#"{"description": {"identifier": "test:w"}, "particle_concentrations": {"chlorophyll": 1.0, "suspended_sediment": 2.0, "cdom": 3.0}, "caustics": {"enabled": false, "frame_length": 0.5, "texture": "textures/caustics", "scale": 2.0, "power": 3.0}, "waves": {"enabled": true, "frequency": 1.5, "octaves": 10, "depth": 0.5, "speed": 3.0, "pull": -0.5, "mix": 0.25, "frequency_scaling": 1.5, "speed_scaling": 1.25, "direction_increment": 45.0, "shape": 2.0}, "biome_water_color_contribution": 0.5}"#,
    );
    copies_every_member::<water::v1_26_0::WaterSettings>(&four);
}

#[test]
fn the_atmosphere_step_copies_every_member() {
    let base = parsed(
        r#"{"description": {"identifier": "test:a"}, "horizon_blend_stops": {"min": 0.5, "start": {"0.5": 0.25}, "mie_start": 1.0, "max": 0.75}, "rayleigh_strength": {"0.0": 1.0, "0.5": 2.0}, "sun_mie_strength": 3.0, "moon_mie_strength": 4.0, "sun_glare_shape": 5.0, "sky_zenith_color": [1, 2, 3], "sky_horizon_color": {"0.25": [4, 5, 6]}}"#,
    );
    copies_every_member::<atmosphere::v1_21_40::AtmosphereSettings>(&base);
}

#[test]
fn the_1_21_40_colour_grading_step_adds_the_temperature_switched_off() {
    let base = parsed(
        r#"{"description": {"identifier": "test:g"}, "color_grading": {"midtones": {"contrast": [1.5, 1.5, 1.5], "gain": [2.0, 2.0, 2.0], "gamma": [2.5, 2.5, 2.5], "offset": [0.5, 0.5, 0.5], "saturation": [3.0, 3.0, 3.0]}, "highlights": {"enabled": true, "highlightsMin": 2.0, "contrast": [1.5, 1.5, 1.5]}, "shadows": {"enabled": true, "shadowsMax": 0.5, "gain": [2.0, 2.0, 2.0]}}, "tone_mapping": {"operator": "aces"}}"#,
    );
    copies_and_adds::<color_grading::v1_21_40::ColorGradingSettings>(
        &base,
        "color_grading/temperature",
        r#"{"enabled": false, "temperature": 6500.0, "type": "white_balance"}"#,
    );
}

#[test]
fn the_chains_hold_the_structs_of_the_plan() {
    assert_eq!(
        <lighting::LightingHistory as VersionChain<lighting::Family>>::VERSIONS,
        [
            v(1, 21, 40),
            v(1, 21, 60),
            v(1, 21, 70),
            v(1, 21, 80),
            v(1, 26, 0)
        ]
    );
    assert_eq!(
        <water::WaterHistory as VersionChain<water::Family>>::VERSIONS,
        [
            v(1, 21, 40),
            v(1, 21, 80),
            v(1, 21, 120),
            v(1, 26, 0),
            v(1, 26, 20)
        ]
    );
    assert_eq!(
        <atmosphere::AtmosphereHistory as VersionChain<atmosphere::Family>>::VERSIONS,
        [v(1, 21, 40), v(1, 26, 20)]
    );
    assert_eq!(
        <color_grading::ColorGradingHistory as VersionChain<color_grading::Family>>::VERSIONS,
        [v(1, 21, 40), v(1, 21, 90)]
    );
    assert_eq!(
        <cubemap::CubemapHistory as VersionChain<cubemap::Family>>::VERSIONS,
        [v(1, 21, 130)]
    );
    assert_eq!(
        <shadows::ShadowsHistory as VersionChain<shadows::Family>>::VERSIONS,
        [v(1, 21, 80)]
    );
    assert_eq!(
        <pbr_fallback::PbrFallbackHistory as VersionChain<pbr_fallback::Family>>::VERSIONS,
        [v(1, 21, 40)]
    );
    assert_eq!(
        <local_lighting::LocalLightingHistory as VersionChain<local_lighting::Family>>::VERSIONS,
        [v(1, 21, 120)]
    );
}

/// What `version` selects in family `F`.
fn selects<F: Family>(version: &str) -> Selection {
    select::<F>(&FormatVersionInput::parse(version).expect("a version"))
}

/// The plan's parameters of one chain: its key, folder, floor, exact versions only, no beta
/// struct, a missing version rejected, the checked envelope.
#[track_caller]
fn chain<F: Family>(key: &str, directory: &str, floor: FormatVersion, versions: &[&str])
where
    F::History: VersionChain<F>,
{
    check_chain::<F>().expect(key);
    assert_eq!(F::PAYLOAD_KEY, key);
    assert!(
        bedrock_addon_family_deferred_settings::LOADER
            .walk
            .directories
            .contains(&directory),
        "{directory}"
    );
    assert_eq!(F::FLOOR, floor);
    assert_eq!(F::RANGE_MODEL, RangeModel::ExactOnly);
    assert!(matches!(F::ENVELOPE.missing, MissingVersion::Reject { .. }));
    assert_eq!(F::ENVELOPE.check, EnvelopeCheck::Always);
    let has_beta = <F::History as VersionChain<F>>::HAS_BETA;
    assert!(!has_beta, "Beta = Infallible");
    for (index, version) in versions.iter().enumerate() {
        assert_eq!(
            selects::<F>(version),
            Selection::Struct(index),
            "{key} {version}"
        );
        // One patch above a struct version selects nothing.
        let parsed = FormatVersionInput::parse(version).unwrap().version.unwrap();
        let above = format!("{}.{}.{}", parsed.major, parsed.minor, parsed.patch + 1);
        assert_eq!(
            selects::<F>(&above),
            Selection::Unsupported,
            "{key} {above}"
        );
        let tagged = format!("{version}-beta.1");
        assert_ne!(
            selects::<F>(&tagged),
            Selection::Struct(index),
            "{key} {tagged}"
        );
    }
    assert_eq!(
        F::FLOOR.to_string(),
        versions[0],
        "the floor is the first struct"
    );
}

#[test]
fn the_chains_are_the_plans() {
    chain::<lighting::Family>(
        "minecraft:lighting_settings",
        "lighting",
        v(1, 21, 40),
        &["1.21.40", "1.21.60", "1.21.70", "1.21.80", "1.26.0"],
    );
    chain::<water::Family>(
        "minecraft:water_settings",
        "water",
        v(1, 21, 40),
        &["1.21.40", "1.21.80", "1.21.120", "1.26.0", "1.26.20"],
    );
    chain::<atmosphere::Family>(
        "minecraft:atmosphere_settings",
        "atmospherics",
        v(1, 21, 40),
        &["1.21.40", "1.26.20"],
    );
    chain::<color_grading::Family>(
        "minecraft:color_grading_settings",
        "color_grading",
        v(1, 21, 40),
        &["1.21.40", "1.21.90"],
    );
    chain::<cubemap::Family>(
        "minecraft:cubemap_settings",
        "cubemaps",
        v(1, 21, 130),
        &["1.21.130"],
    );
    chain::<shadows::Family>(
        "minecraft:shadow_settings",
        "shadows",
        v(1, 21, 80),
        &["1.21.80"],
    );
    chain::<pbr_fallback::Family>(
        "minecraft:pbr_fallback_settings",
        "pbr",
        v(1, 21, 40),
        &["1.21.40"],
    );
    chain::<local_lighting::Family>(
        "minecraft:local_light_settings",
        "local_lighting",
        v(1, 21, 120),
        &["1.21.120"],
    );
}
