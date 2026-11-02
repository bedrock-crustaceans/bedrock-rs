//! The payloads with one struct: shadows (h01–h03, H1, H2), PBR fallback (p01–p06, P1–P2a),
//! cubemap (k01, k02, K1, K3) and local lights (t01, t06, t07, T1, T2, L5a).

use bedrock_addon::document::Content;
use bedrock_addon::families::deferred_settings::{
    CubemapSettings, Keyframe, Keyframes, LightType, LocalLightSettings, Mer, Mers,
    PbrFallbackSettings, PbrValue, Rgb8, ShadowSettings, ShadowStyle,
};
use bedrock_addon::id::BlockId;

use super::common::*;

const RGB_TEXT: &str = "Expected [r, g, b], with values in the 0-255 range, or a hex value with 6 characters (e.g. RGB as #FFFFFF).";
const RGBA_TEXT: &str = "Expected [r, g, b, a], with values in the 0-255 range, or a hex value with 8 characters (e.g. ARGB as #FFFFFFFF).";
const RANGE: &str = "the input is outside of the accepted range";
const NUMBER: &str = "the input must be a number";

fn shadows(members: &str) -> String {
    doc("1.21.80", SHADOWS, members)
}

fn pbr(members: &str) -> String {
    doc("1.21.40", PBR, members)
}

fn cubemap(members: &str) -> String {
    doc("1.21.130", CUBEMAP, members)
}

fn local(members: &str) -> String {
    doc("1.21.120", LOCAL, members)
}

/// The one error must be an `InvalidValue` at `path` with cereal's range text.
#[track_caller]
pub fn out_of_range(found: (&'static str, String, String), path: &str) {
    assert_eq!((found.0, found.1.as_str()), ("InvalidValue", path));
    assert!(found.2.starts_with(RANGE), "{}", found.2);
}

// ---- shadows -------------------------------------------------------------------------------

#[test]
fn h01_the_vanilla_shadow_file_loads() {
    let document = loads::<ShadowSettings>(&fixture("shadows/vanilla_global.json"));
    assert_eq!(document.shadow_style, Some(ShadowStyle::BlockyShadows));
    assert_eq!(document.texel_size, Some(16));
    assert!(document.diagnostics().is_empty());
    round_trip(document.value());
}

#[test]
fn h02_an_empty_payload_uses_the_games_values_and_writes_none_back() {
    let document = loads::<ShadowSettings>(&shadows(""));
    assert_eq!((document.shadow_style, document.texel_size), (None, None));
    assert_eq!(document.shadow_style(), ShadowStyle::SoftShadows);
    assert_eq!(document.texel_size(), 16);
    assert_eq!(json(&round_trip(document.value())), json(&shadows("")));
    let soft = loads::<ShadowSettings>(&shadows(r#""shadow_style": "soft_shadows""#));
    assert_eq!(soft.shadow_style, Some(ShadowStyle::SoftShadows));
    assert_eq!(
        json(&round_trip(soft.value())),
        json(&shadows(r#""shadow_style": "soft_shadows""#))
    );
}

#[test]
fn h03_texel_size_is_an_integer_from_1_to_1024() {
    out_of_range(
        one_error::<ShadowSettings>(&shadows(r#""texel_size": 0"#)),
        "minecraft:shadow_settings/texel_size",
    );
    out_of_range(
        one_error::<ShadowSettings>(&shadows(r#""texel_size": 2048"#)),
        "minecraft:shadow_settings/texel_size",
    );
    out_of_range(
        one_error::<ShadowSettings>(&shadows(r#""texel_size": 1025"#)),
        "minecraft:shadow_settings/texel_size",
    );
    out_of_range(
        one_error::<ShadowSettings>(&shadows(r#""texel_size": -1"#)),
        "minecraft:shadow_settings/texel_size",
    );
    assert_eq!(
        loads::<ShadowSettings>(&shadows(r#""texel_size": 1"#)).texel_size,
        Some(1)
    );
    assert_eq!(
        loads::<ShadowSettings>(&shadows(r#""texel_size": 1024"#)).texel_size,
        Some(1024)
    );
}

#[test]
fn h2_a_texel_size_with_a_fraction_or_of_another_type_is_refused() {
    assert_eq!(
        one_error::<ShadowSettings>(&shadows(r#""texel_size": 16.5"#)),
        finding(
            "InvalidValue",
            "minecraft:shadow_settings/texel_size",
            "invalid value"
        )
    );
    assert_eq!(
        one_error::<ShadowSettings>(&shadows(r#""texel_size": "16""#)),
        finding(
            "InvalidType",
            "minecraft:shadow_settings/texel_size",
            NUMBER
        )
    );
    assert_eq!(
        one_error::<ShadowSettings>(&shadows(r#""texel_size": [16]"#)).0,
        "InvalidType"
    );
    // A real token without a fraction is the integer (ours; unverified in vanilla).
    assert_eq!(
        loads::<ShadowSettings>(&shadows(r#""texel_size": 32.0"#)).texel_size,
        Some(32)
    );
}

#[test]
fn h1_an_unknown_shadow_style_is_an_enum_error() {
    assert_eq!(
        at(&[one_error::<ShadowSettings>(&shadows(
            r#""shadow_style": "hard_shadows""#
        ))]),
        [("InvalidEnumValue", "minecraft:shadow_settings/shadow_style")]
    );
    assert_eq!(
        at(&[one_error::<ShadowSettings>(&shadows(
            r#""shadow_style": 1"#
        ))]),
        [("InvalidType", "minecraft:shadow_settings/shadow_style")]
    );
    assert_eq!(
        at(&[one_error::<ShadowSettings>(&shadows(
            r#""shadow_style": "Soft_Shadows""#
        ))]),
        [("InvalidEnumValue", "minecraft:shadow_settings/shadow_style")]
    );
}

#[test]
fn every_failing_member_of_a_file_is_reported() {
    let d = rejects::<ShadowSettings>(&shadows(r#""shadow_style": "hard", "texel_size": 0"#));
    assert_eq!(
        at(&errors(&d)),
        [
            ("InvalidEnumValue", "minecraft:shadow_settings/shadow_style"),
            ("InvalidValue", "minecraft:shadow_settings/texel_size"),
        ]
    );
}

#[test]
fn a_null_member_reads_as_absent() {
    let document = loads::<ShadowSettings>(&shadows(r#""texel_size": null"#));
    assert_eq!(document.texel_size, None);
    assert!(document.diagnostics().is_empty());
}

#[test]
fn a_finding_has_the_place_of_its_own_value() {
    let text = shadows(r#""shadow_style": "soft_shadows", "texel_size": 4096"#);
    let d = rejects::<ShadowSettings>(&text);
    let span = d.iter().next().unwrap().location.span.expect("a span");
    let (start, end) = (span.offset as usize, (span.offset + span.len) as usize);
    assert_eq!(&text[start..end], "4096");
}

// ---- pbr_fallback --------------------------------------------------------------------------

const MERS_KEY: &str = "global_metalness_emissive_roughness_subsurface";
const MER_KEY: &str = "global_metalness_emissive_roughness";

fn mers(metalness: u8, emissive: u8, roughness: u8, subsurface: u8) -> Mers {
    Mers {
        metalness,
        emissive,
        roughness,
        subsurface,
    }
}

#[test]
fn the_vanilla_pbr_file_loads() {
    let document = loads::<PbrFallbackSettings>(&fixture("pbr_fallback/vanilla_global.json"));
    for kind in [
        document.blocks(),
        document.actors(),
        document.particles(),
        document.items(),
    ] {
        assert_eq!(kind, Mers::DEFAULT);
    }
    assert!(document.diagnostics().is_empty());
    round_trip(document.value());
}

#[test]
fn p01_integral_floats_are_channel_values_and_a_fraction_is_refused() {
    let document = loads::<PbrFallbackSettings>(&pbr(&format!(
        r#""blocks": {{"{MERS_KEY}": [10.0, 20.0, 255.0, 0.0]}}, "actors": {{"{MERS_KEY}": [1, 2, 3, 4]}}"#
    )));
    assert_eq!(document.blocks(), mers(10, 20, 255, 0));
    assert_eq!(document.actors(), mers(1, 2, 3, 4));
    assert!(document.diagnostics().is_empty());
    // Ours: whether cereal's integer reader takes a fraction is unverified.
    assert_eq!(
        one_error::<PbrFallbackSettings>(&pbr(&format!(
            r#""blocks": {{"{MERS_KEY}": [0, 0, 254.5, 0]}}"#
        ))),
        finding(
            "InvalidValue",
            &format!("{PBR}/blocks/{MERS_KEY}/2"),
            "invalid value"
        )
    );
}

#[test]
fn p02_the_three_value_form_is_read_with_subsurface_0_and_written_back_as_it_was() {
    let text = pbr(&format!(r#""blocks": {{"{MER_KEY}": [10, 20, 30]}}"#));
    let document = loads::<PbrFallbackSettings>(&text);
    assert_eq!(
        document.blocks.as_ref().unwrap().value,
        Some(PbrValue::Mer(Mer {
            metalness: 10,
            emissive: 20,
            roughness: 30
        }))
    );
    assert_eq!(document.blocks(), mers(10, 20, 30, 0));
    assert_eq!(json(&round_trip(document.value())), json(&text));
    // The hex form of three values is `#MMEERR`.
    let hex =
        loads::<PbrFallbackSettings>(&pbr(&format!(r##""items": {{"{MER_KEY}": "#0A141E"}}"##)));
    assert_eq!(hex.items(), mers(10, 20, 30, 0));
}

#[test]
fn p03_p1_an_empty_kind_is_the_four_value_form_with_the_games_values() {
    let text = pbr(r#""blocks": {}"#);
    let document = loads::<PbrFallbackSettings>(&text);
    assert_eq!(document.blocks.as_ref().unwrap().value, None);
    assert_eq!(document.blocks(), Mers::DEFAULT);
    assert!(document.diagnostics().is_empty());
    assert_eq!(json(&round_trip(document.value())), json(&text));
}

#[test]
fn p04_a_kind_left_out_uses_the_games_values() {
    let text = pbr(&format!(r#""items": {{"{MERS_KEY}": [9, 8, 7, 6]}}"#));
    let document = loads::<PbrFallbackSettings>(&text);
    assert!(document.blocks.is_none() && document.actors.is_none());
    assert!(document.particles.is_none());
    assert_eq!(document.blocks(), Mers::DEFAULT);
    assert_eq!(document.actors(), Mers::DEFAULT);
    assert_eq!(document.particles(), Mers::DEFAULT);
    assert_eq!(document.items(), mers(9, 8, 7, 6));
    assert_eq!(json(&round_trip(document.value())), json(&text));
}

#[test]
fn p05_the_eight_digit_hex_form_has_subsurface_first() {
    let document = loads::<PbrFallbackSettings>(&pbr(&format!(
        r##""blocks": {{"{MERS_KEY}": "#000000FF"}}, "actors": {{"{MERS_KEY}": "#0000FF00"}},
            "particles": {{"{MERS_KEY}": "#0a141e28"}}, "items": {{"{MERS_KEY}": "#FF000000"}}"##
    )));
    assert_eq!(document.blocks(), mers(0, 0, 255, 0));
    assert_eq!(document.actors(), mers(0, 255, 0, 0));
    assert_eq!(document.particles(), mers(0x14, 0x1e, 0x28, 0x0a));
    assert_eq!(document.items(), mers(0, 0, 0, 255));
    round_trip(document.value());
}

#[test]
fn p06_p2a_an_array_element_outside_0_255_is_clamped_with_an_info_deviation() {
    // vanilla = "clamped silently".
    let document = loads::<PbrFallbackSettings>(&pbr(&format!(
        r#""blocks": {{"{MERS_KEY}": [0, 0, 300, -5]}}, "actors": {{"{MER_KEY}": [256, 255, 0]}}"#
    )));
    assert_eq!(document.blocks(), mers(0, 0, 255, 0));
    assert_eq!(document.actors(), mers(255, 255, 0, 0));
    assert_eq!(
        infos(document.diagnostics()),
        [
            finding(
                "ClampedValue",
                &format!("{PBR}/blocks/{MERS_KEY}/2"),
                "value clamped into its range"
            ),
            finding(
                "ClampedValue",
                &format!("{PBR}/blocks/{MERS_KEY}/3"),
                "value clamped into its range"
            ),
            finding(
                "ClampedValue",
                &format!("{PBR}/actors/{MER_KEY}/0"),
                "value clamped into its range"
            ),
        ]
    );
    assert!(warnings(document.diagnostics()).is_empty());
    // The bounds themselves are not clamped.
    let bounds = loads::<PbrFallbackSettings>(&pbr(&format!(
        r#""blocks": {{"{MERS_KEY}": [0, 255, 0, 255]}}"#
    )));
    assert!(bounds.diagnostics().is_empty());
}

#[test]
fn p2_a_value_in_neither_form_is_a_type_error_with_the_colour_text() {
    for value in [
        "[0, 0, 255]",
        "[0, 0, 255, 0, 0]",
        r##""#FFF""##,
        r##""#GG000000""##,
        r##""#000000""##,
        r#""000000FF""#,
        "5",
        "{}",
        "true",
    ] {
        assert_eq!(
            one_error::<PbrFallbackSettings>(&pbr(&format!(
                r#""blocks": {{"{MERS_KEY}": {value}}}"#
            ))),
            finding(
                "InvalidType",
                &format!("{PBR}/blocks/{MERS_KEY}"),
                RGBA_TEXT
            ),
            "{value}"
        );
    }
    for value in [
        "[0, 0]",
        "[0, 0, 0, 0]",
        r##""#00000000""##,
        r##""#00000""##,
    ] {
        assert_eq!(
            one_error::<PbrFallbackSettings>(&pbr(&format!(
                r#""items": {{"{MER_KEY}": {value}}}"#
            ))),
            finding("InvalidType", &format!("{PBR}/items/{MER_KEY}"), RGB_TEXT),
            "{value}"
        );
    }
    // A bad element is reported at its own index.
    assert_eq!(
        one_error::<PbrFallbackSettings>(&pbr(&format!(
            r#""blocks": {{"{MERS_KEY}": [0, "a", 0, 0]}}"#
        ))),
        finding(
            "InvalidType",
            &format!("{PBR}/blocks/{MERS_KEY}/1"),
            RGBA_TEXT
        )
    );
}

#[test]
fn a_kind_with_both_members_is_read_by_the_four_value_one() {
    let text = pbr(&format!(
        r#""blocks": {{"{MER_KEY}": [1, 2, 3], "{MERS_KEY}": [4, 5, 6, 7]}}"#
    ));
    let document = loads::<PbrFallbackSettings>(&text);
    assert_eq!(document.blocks(), mers(4, 5, 6, 7));
    assert_eq!(
        at(&infos(document.diagnostics())),
        [("UnrecognizedField", &*format!("{PBR}/blocks/{MER_KEY}"))]
    );
}

#[test]
fn a_kind_that_is_no_object_is_a_type_error() {
    assert_eq!(
        one_error::<PbrFallbackSettings>(&pbr(r#""blocks": [0, 0, 255, 0]"#)),
        finding(
            "InvalidType",
            &format!("{PBR}/blocks"),
            "expected an object"
        )
    );
}

// ---- cubemap -------------------------------------------------------------------------------

/// The documentation's example of a cubemap file.
const K01: &str = r#"{
  "format_version": "1.21.130",
  "minecraft:cubemap_settings": {
    "description": { "identifier": "my_pack:default_cubemap" },
    "lighting": {
      "ambient_light_illuminance": { "0.0": 5.625, "1.0": 5.625 },
      "sky_light_contribution": 1.0,
      "directional_light_contribution": 0.0,
      "affected_by_atmospheric_scattering": false,
      "affected_by_volumetric_scattering": true
    }
  }
}"#;

#[test]
fn k01_the_documentation_example_loads_and_its_values_are_the_games() {
    let document = loads::<CubemapSettings>(K01);
    assert!(
        document.diagnostics().is_empty(),
        "{:?}",
        document.diagnostics()
    );
    assert_eq!(
        document.description.as_ref().unwrap().identifier.as_str(),
        "my_pack:default_cubemap"
    );
    let written = document.lighting.clone().unwrap();
    // A file that sets nothing resolves to the same values.
    let bare = loads::<CubemapSettings>(&cubemap(&named("t:c", r#""lighting": {}"#)));
    let lighting = bare.lighting.clone().unwrap();
    assert_eq!(lighting.ambient_light_illuminance, None);
    assert_eq!(
        lighting.ambient_light_illuminance(),
        Keyframes::Frames(vec![
            Keyframe {
                time: 0.0,
                value: 5.625
            },
            Keyframe {
                time: 1.0,
                value: 5.625
            }
        ])
    );
    assert_eq!(
        written.ambient_light_illuminance(),
        lighting.ambient_light_illuminance()
    );
    assert_eq!(lighting.sky_light_contribution(), 1.0);
    assert_eq!(lighting.directional_light_contribution(), 0.0);
    assert!(!lighting.affected_by_atmospheric_scattering());
    assert!(lighting.affected_by_volumetric_scattering());
    assert_eq!(written.sky_light_contribution, Some(1.0));
    assert_eq!(written.affected_by_volumetric_scattering, Some(true));
    round_trip(document.value());
    assert_eq!(
        json(&round_trip(bare.value())),
        json(&cubemap(&named("t:c", r#""lighting": {}"#)))
    );
}

#[test]
fn k02_k1_the_cubemap_ranges_include_both_ends() {
    let lighting = |members: &str| cubemap(&named("t:c", &format!(r#""lighting": {{{members}}}"#)));
    for (member, bad, good) in [
        ("sky_light_contribution", ["1.5", "-0.1"], ["0", "1"]),
        (
            "directional_light_contribution",
            ["1.01", "-1"],
            ["0.0", "1.0"],
        ),
        (
            "ambient_light_illuminance",
            ["100001", "-0.5"],
            ["0", "100000"],
        ),
    ] {
        for value in bad {
            out_of_range(
                one_error::<CubemapSettings>(&lighting(&format!(r#""{member}": {value}"#))),
                &format!("{CUBEMAP}/lighting/{member}"),
            );
        }
        for value in good {
            let document = loads::<CubemapSettings>(&lighting(&format!(r#""{member}": {value}"#)));
            assert!(document.diagnostics().is_empty(), "{member} {value}");
        }
    }
    // The range applies to each keyframe value; the finding names the key.
    out_of_range(
        one_error::<CubemapSettings>(&lighting(
            r#""ambient_light_illuminance": {"0.0": 5, "0.5": 100001}"#,
        )),
        &format!("{CUBEMAP}/lighting/ambient_light_illuminance/0.5"),
    );
}

#[test]
fn k3_a_cubemap_without_description_loads_unnamed_deviation() {
    // vanilla = "empty identifier": the file is keyed by "".
    let document = loads::<CubemapSettings>(&cubemap(r#""lighting": {}"#));
    assert!(document.description.is_none());
    assert_eq!(
        infos(document.diagnostics()),
        [finding(
            "MissingIdentifier",
            CUBEMAP,
            "no usable identifier"
        )]
    );
    assert_eq!(
        at(&[one_error::<CubemapSettings>(&cubemap(
            r#""description": {}"#
        ))]),
        [(
            "MissingRequiredField",
            "minecraft:cubemap_settings/description/identifier"
        )]
    );
    assert_eq!(
        at(&[one_error::<CubemapSettings>(&cubemap(
            r#""description": {"identifier": 5}"#
        ))]),
        [(
            "InvalidType",
            "minecraft:cubemap_settings/description/identifier"
        )]
    );
}

#[test]
fn an_identifier_without_a_namespace_loads_with_an_info_deviation() {
    // vanilla: accepted (`Identifier<AssetType>::bindType` has no constraint).
    let document = loads::<CubemapSettings>(&cubemap(&named("plain", "")));
    assert_eq!(
        infos(document.diagnostics()),
        [finding(
            "NamespaceRule",
            "minecraft:cubemap_settings/description/identifier",
            "namespace rule failed"
        )]
    );
    assert!(warnings(document.diagnostics()).is_empty());
    let namespaced = loads::<CubemapSettings>(&cubemap(&named("a:b", "")));
    assert!(namespaced.diagnostics().is_empty());
}

/// The namespace note of validation.md for every payload with a `description`: colour grading
/// reads its description with a reader of its own (its `identifier` is optional), the other
/// four with the shared one.
#[test]
fn an_identifier_without_a_namespace_is_an_info_in_every_payload_deviation() {
    use bedrock_addon::families::deferred_settings::{
        AtmosphereSettings, ColorGradingSettings, LightingSettings, WaterSettings,
    };
    fn infos_of<T: Content + std::fmt::Debug>(
        version: &str,
        key: &str,
        identifier: &str,
    ) -> Vec<(&'static str, String, String)> {
        let document = loads::<T>(&doc(version, key, &named(identifier, "")));
        assert!(warnings(document.diagnostics()).is_empty(), "{key}");
        infos(document.diagnostics())
    }
    let note = |key: &str| {
        vec![finding(
            "NamespaceRule",
            &format!("{key}/description/identifier"),
            "namespace rule failed",
        )]
    };
    assert_eq!(
        infos_of::<ColorGradingSettings>("1.21.90", GRADING, "plain"),
        note(GRADING)
    );
    assert_eq!(
        infos_of::<ColorGradingSettings>("1.21.40", GRADING, "plain"),
        note(GRADING)
    );
    assert_eq!(
        infos_of::<LightingSettings>("1.26.0", LIGHTING, "plain"),
        note(LIGHTING)
    );
    assert_eq!(
        infos_of::<WaterSettings>("1.26.20", WATER, "plain"),
        note(WATER)
    );
    assert_eq!(
        infos_of::<AtmosphereSettings>("1.26.20", ATMOSPHERE, "plain"),
        note(ATMOSPHERE)
    );
    assert_eq!(
        infos_of::<CubemapSettings>("1.21.130", CUBEMAP, "plain"),
        note(CUBEMAP)
    );
    // With a namespace none has anything to say.
    assert_eq!(
        infos_of::<ColorGradingSettings>("1.21.90", GRADING, "a:b"),
        []
    );
    assert_eq!(infos_of::<LightingSettings>("1.26.0", LIGHTING, "a:b"), []);
    assert_eq!(infos_of::<WaterSettings>("1.26.20", WATER, "a:b"), []);
    assert_eq!(
        infos_of::<AtmosphereSettings>("1.26.20", ATMOSPHERE, "a:b"),
        []
    );
}

#[test]
fn unknown_members_are_kept_at_their_level() {
    let text = cubemap(&named(
        "t:c",
        r#""lighting": {"sky_light_contribution": 0.5, "glow": true}, "other": [1]"#,
    ));
    let text = text.replace(
        r#""identifier": "t:c""#,
        r#""identifier": "t:c", "author": "me""#,
    );
    let document = loads::<CubemapSettings>(&text);
    assert_eq!(
        at(&infos(document.diagnostics())),
        [
            (
                "UnrecognizedField",
                "minecraft:cubemap_settings/description/author"
            ),
            (
                "UnrecognizedField",
                "minecraft:cubemap_settings/lighting/glow"
            ),
            ("UnrecognizedField", "minecraft:cubemap_settings/other"),
        ]
    );
    let written = document.to_json().unwrap();
    assert_eq!(json(&written), json(&text));
    // Written again from the re-read value: the same text.
    let again = loads::<CubemapSettings>(&written);
    assert_eq!(again.value(), document.value());
    assert_eq!(again.to_json().unwrap(), written);
}

// ---- local_lighting ------------------------------------------------------------------------

fn block(name: &str) -> BlockId {
    BlockId::parse(name)
}

#[test]
fn t01_the_documentation_example_loads() {
    let document = loads::<LocalLightSettings>(&local(
        r##""minecraft:torch": {"light_type": "point_light", "light_color": "#EFE39D"},
            "minecraft:sea_lantern": {"light_type": "static_light", "light_color": [1, 2, 3]}"##,
    ));
    assert!(document.diagnostics().is_empty());
    let torch = &document.lights[&block("minecraft:torch")];
    assert_eq!(torch.light_type, Some(LightType::PointLight));
    assert_eq!(torch.light_color, Some(Rgb8::new(0xEF, 0xE3, 0x9D)));
    let lantern = &document.lights[&block("minecraft:sea_lantern")];
    assert_eq!(lantern.light_type(), LightType::StaticLight);
    assert_eq!(lantern.light_color, Some(Rgb8::new(1, 2, 3)));
    let keys: Vec<&str> = document.lights.keys().map(BlockId::as_str).collect();
    assert_eq!(keys, ["minecraft:torch", "minecraft:sea_lantern"]);
    round_trip(document.value());
}

#[test]
fn the_technical_preview_local_light_file_loads() {
    let document =
        loads::<LocalLightSettings>(&fixture("local_lighting/vanilla_technical_preview.json"));
    assert!(
        document.diagnostics().is_empty(),
        "{:?}",
        document.diagnostics()
    );
    assert!(document.lights.len() > 50);
    assert_eq!(
        document.lights[&block("minecraft:torch")].light_color,
        Some(Rgb8::new(0xf3, 0x9a, 0x5e))
    );
    round_trip(document.value());
}

#[test]
fn t06_t1_an_unknown_light_type_fails_the_whole_file() {
    let d = rejects::<LocalLightSettings>(&local(
        r##""minecraft:torch": {"light_type": "point_light"},
            "minecraft:lantern": {"light_type": "spot_light"}"##,
    ));
    assert_eq!(
        at(&errors(&d)),
        [(
            "InvalidEnumValue",
            "minecraft:local_light_settings/minecraft:lantern/light_type"
        )]
    );
}

#[test]
fn t2_a_malformed_light_colour_fails_the_whole_file() {
    for value in [
        r##""#FFFFFFFF""##,
        r#""red""#,
        "[1, 2]",
        "[1, 2, 3, 4]",
        "7",
    ] {
        assert_eq!(
            one_error::<LocalLightSettings>(&local(&format!(
                r##""minecraft:torch": {{"light_color": {value}}}"##
            ))),
            finding(
                "InvalidType",
                "minecraft:local_light_settings/minecraft:torch/light_color",
                RGB_TEXT
            ),
            "{value}"
        );
    }
}

#[test]
fn t07_the_colour_and_the_type_are_optional() {
    let text = local(r#""minecraft:torch": {"light_type": "point_light"}, "minecraft:fire": {}"#);
    let document = loads::<LocalLightSettings>(&text);
    let torch = &document.lights[&block("minecraft:torch")];
    assert_eq!(torch.light_color, None);
    let fire = &document.lights[&block("minecraft:fire")];
    assert_eq!((fire.light_type, fire.light_color), (None, None));
    assert_eq!(fire.light_type(), LightType::StaticLight);
    assert_eq!(json(&round_trip(document.value())), json(&text));
}

#[test]
fn l5a_a_colour_element_outside_0_255_is_clamped_with_an_info_deviation() {
    // vanilla = "clamped silently".
    let document = loads::<LocalLightSettings>(&local(
        r#""minecraft:torch": {"light_color": [300, -1, 5]}"#,
    ));
    assert_eq!(
        document.lights[&block("minecraft:torch")].light_color,
        Some(Rgb8::new(255, 0, 5))
    );
    assert_eq!(
        at(&infos(document.diagnostics())),
        [
            (
                "ClampedValue",
                "minecraft:local_light_settings/minecraft:torch/light_color/0"
            ),
            (
                "ClampedValue",
                "minecraft:local_light_settings/minecraft:torch/light_color/1"
            ),
        ]
    );
}

#[test]
fn an_entry_that_is_no_object_fails_the_file() {
    assert_eq!(
        one_error::<LocalLightSettings>(&local(r##""minecraft:torch": "#FFFFFF""##)),
        finding(
            "InvalidType",
            "minecraft:local_light_settings/minecraft:torch",
            "expected an object"
        )
    );
}

#[test]
fn an_entry_keeps_its_unknown_members() {
    let text = local(r#""minecraft:torch": {"light_type": "point_light", "radius": 4}"#);
    let document = loads::<LocalLightSettings>(&text);
    assert_eq!(
        at(&infos(document.diagnostics())),
        [(
            "UnrecognizedField",
            "minecraft:local_light_settings/minecraft:torch/radius"
        )]
    );
    assert_eq!(json(&document.to_json().unwrap()), json(&text));
}

#[test]
fn two_spellings_of_one_block_keep_the_later_entry_deviation() {
    // Ours: the map is keyed by block identifier, and `torch` is `minecraft:torch`. Vanilla
    // keeps both strings and resolves each to the same block.
    let document = loads::<LocalLightSettings>(&local(
        r#""torch": {"light_type": "point_light"}, "minecraft:lantern": {},
            "minecraft:torch": {"light_type": "static_light"}"#,
    ));
    assert_eq!(document.lights.len(), 2);
    assert_eq!(
        document.lights[&block("minecraft:torch")].light_type,
        Some(LightType::StaticLight)
    );
    assert_eq!(
        at(&infos(document.diagnostics())),
        [(
            "DuplicateIdentifier",
            "minecraft:local_light_settings/minecraft:torch"
        )]
    );
}

#[test]
fn two_thousand_local_lights_load_in_file_order_and_are_written_back() {
    // The content of a long file; its cost is measured in src/render/deferred/tests.rs
    // (the_entries_of_a_local_light_file_…).
    let entries: Vec<String> = (0..2_000)
        .map(|i| format!(r#""test:block_{i}": {{"light_type": "point_light", "light_color": [{}, 300, 3], "extra": {i}}}"#, i % 256))
        .collect();
    let text = local(&entries.join(","));
    let document = loads::<LocalLightSettings>(&text);
    let written = document.to_json().unwrap();
    assert_eq!(document.lights.len(), 2_000);
    assert!(
        document
            .lights
            .keys()
            .enumerate()
            .all(|(i, block)| block.as_str() == format!("test:block_{i}"))
    );
    // One clamp info and one unknown-member info per entry.
    assert_eq!(infos(document.diagnostics()).len(), 4_000);
    assert_eq!(warnings(document.diagnostics()).len(), 0);
    assert_eq!(document.diagnostics().len(), 4_000);
    assert_eq!(
        loads::<LocalLightSettings>(&written).value(),
        document.value()
    );
}

#[test]
fn an_empty_identifier_names_nothing_deviation() {
    // vanilla = "empty identifier": the game keys the file by "", as it does a file without
    // `description`, and no default is taken from it.
    let document = loads::<CubemapSettings>(&cubemap(&named("", r#""lighting": {}"#)));
    assert_eq!(
        document.description.as_ref().unwrap().identifier.as_str(),
        ""
    );
    // Both notes are ours: the empty name breaks the namespace rule and
    // names nothing.
    assert_eq!(
        infos(document.diagnostics()),
        [
            finding(
                "NamespaceRule",
                "minecraft:cubemap_settings/description/identifier",
                "namespace rule failed"
            ),
            finding("MissingIdentifier", CUBEMAP, "no usable identifier")
        ]
    );
    // Written back as it was.
    assert_eq!(
        json(&document.to_json().unwrap()),
        json(&cubemap(&named("", r#""lighting": {}"#)))
    );
}

#[test]
fn the_two_scattering_flags_are_read_and_written_either_way() {
    for (atmospheric, volumetric) in [(true, false), (false, true), (true, true), (false, false)] {
        let text = cubemap(&named(
            "t:c",
            &format!(
                r#""lighting": {{"affected_by_atmospheric_scattering": {atmospheric}, "affected_by_volumetric_scattering": {volumetric}}}"#
            ),
        ));
        let document = loads::<CubemapSettings>(&text);
        let lighting = document.lighting.clone().unwrap();
        assert_eq!(
            lighting.affected_by_atmospheric_scattering,
            Some(atmospheric)
        );
        assert_eq!(lighting.affected_by_volumetric_scattering, Some(volumetric));
        assert_eq!(lighting.affected_by_atmospheric_scattering(), atmospheric);
        assert_eq!(lighting.affected_by_volumetric_scattering(), volumetric);
        assert_eq!(json(&round_trip(document.value())), json(&text));
    }
    assert_eq!(
        at(&[one_error::<CubemapSettings>(&cubemap(
            r#""lighting": {"affected_by_atmospheric_scattering": "true"}"#
        ))]),
        [(
            "InvalidType",
            "minecraft:cubemap_settings/lighting/affected_by_atmospheric_scattering"
        )]
    );
}

#[test]
fn a_deeply_nested_unknown_member_is_kept() {
    // Run once with full debug info too (COMMON "Build profiles differ"): the readers do not
    // recurse on the depth of the text.
    let depth = 60;
    let nested = format!("{}1{}", "[".repeat(depth), "]".repeat(depth));
    let text = shadows(&format!(r#""texel_size": 8, "deep": {nested}"#));
    let document = loads::<ShadowSettings>(&text);
    assert_eq!(document.texel_size, Some(8));
    assert_eq!(infos(document.diagnostics()).len(), 1);
    assert_eq!(json(&document.to_json().unwrap()), json(&text));
    // Beyond the reader's nesting limit the file is no JSON to it.
    let depth = 600;
    let nested = format!("{}1{}", "[".repeat(depth), "]".repeat(depth));
    let d = rejects::<ShadowSettings>(&shadows(&format!(r#""deep": {nested}"#)));
    assert_eq!(at(&errors(&d)), [("MalformedInput", "")]);
}
