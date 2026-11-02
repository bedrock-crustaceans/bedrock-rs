//! The chain `v1_16_100 → v1_21_30`: its parameters, the exact-match selection, the one
//! (identity) step, `parse_as_written` / `history::upgrade`, a file read
//! on its own, and the write side.

use bedrock_addon::diagnostic::Severity::Warning;
use bedrock_addon::document::Content;
use bedrock_addon::families::texture_set::history::{self, Any, TextureSetFamily, v1_16_100};
use bedrock_addon::families::texture_set::{
    LayerKind, LayerValue, Rgb, Rgba, TextureName, TextureSet,
};
use bedrock_addon::json::Value;
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::{FormatVersion, FormatVersionInput};
use bedrock_addon::versioning::testing::{Checks, Run, step_output};
use bedrock_addon::versioning::{
    Family, MissingVersion, RangeModel, Selection, VersionChain, Versioned, check_chain, select,
};

use super::common::{fixture_file, rendered};
use super::{FAMILY, STONE_SET};

const V16: FormatVersion = FormatVersion::new(1, 16, 100);
const V21: FormatVersion = FormatVersion::new(1, 21, 30);

fn path() -> PackPath {
    PackPath::new(STONE_SET).unwrap()
}

/// A document of `version` with the payload members `layers`.
fn doc(version: &str, layers: &str) -> String {
    format!(r#"{{ "format_version": "{version}", "minecraft:texture_set": {{ {layers} }} }}"#)
}

fn parse(text: &str) -> TextureSet {
    TextureSet::parse(&path(), text.as_bytes(), &LoadContext::detached())
        .unwrap_or_else(|rejected| panic!("{rejected}"))
        .into_value()
}

#[test]
fn the_chain_is_v1_16_100_then_v1_21_30_with_no_beta() {
    const HAS_BETA: bool = <Any as VersionChain<TextureSetFamily>>::HAS_BETA;
    check_chain::<TextureSetFamily>().expect("the chain is well formed");
    assert_eq!(
        <Any as VersionChain<TextureSetFamily>>::VERSIONS,
        [V16, V21]
    );
    let has_beta = HAS_BETA;
    assert!(!has_beta, "Beta = Infallible");
    assert_eq!(<v1_16_100::TextureSet as Versioned>::VERSION, V16);
    assert_eq!(<TextureSet as Versioned>::VERSION, V21);
}

#[test]
fn the_family_parameters_are_the_plan_s() {
    const WARN: bool = TextureSetFamily::ENVELOPE_WARN_MISSING_VERSION;
    assert_eq!(TextureSetFamily::NAME, "texture_set");
    assert_eq!(TextureSetFamily::PAYLOAD_KEY, "minecraft:texture_set");
    assert_eq!(TextureSetFamily::DIRECTORIES, ["textures"]);
    assert_eq!(TextureSetFamily::FLOOR, V16);
    assert_eq!(TextureSetFamily::RANGE_MODEL, RangeModel::ExactOnly);
    // The ceiling is the newest struct's version, derived from the range model.
    assert_eq!(TextureSetFamily::CEILING, None);
    assert_eq!(TextureSetFamily::MISSING_VERSION, MissingVersion::Reject);
    let warn = WARN;
    assert!(!warn, "a missing version is an error, not a warning");
}

/// What `version` selects.
fn selects(version: &str) -> Selection {
    select::<TextureSetFamily>(&FormatVersionInput::parse(version).expect("a version"))
}

#[test]
fn only_the_two_exact_versions_select_a_struct() {
    assert_eq!(selects("1.16.100"), Selection::Struct(0));
    assert_eq!(selects("1.21.30"), Selection::Struct(1));
    assert_eq!(selects("1.16.99"), Selection::BelowFloor);
    for other in [
        "1.16.101", "1.21.0", "1.21.29", "1.21.31", "1.21.40", "1.26.0",
    ] {
        assert_eq!(selects(other), Selection::Unsupported, "{other}");
    }
}

// `vanilla = "LogLevel Warning; texture replaced by the missing-texture image"` (V2 to V4).
#[test]
fn the_version_and_parse_failures_are_the_loader_s_warnings_and_reject_the_set() {
    // `loadOrCreateTextureSetDefinition` logs each with `_contentLog(true, 2, 32)`
    // and goes on with the image alone.
    let rejects = |text: &str| {
        let rejected = TextureSet::parse(&path(), text.as_bytes(), &LoadContext::detached())
            .expect_err("rejected");
        let found = rendered(rejected.diagnostics.iter());
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].1, Warning);
        found[0].0
    };
    assert_eq!(rejects("{"), "MalformedInput");
    assert_eq!(rejects(&doc("x", "")), "MalformedFormatVersion");
    assert_eq!(
        rejects(r#"{ "format_version": 1 }"#),
        "MalformedFormatVersion"
    );
    assert_eq!(rejects("[]"), "MalformedFormatVersion");
    assert_eq!(rejects(&doc("1.21.31", "")), "UnsupportedVersion");
    assert_eq!(rejects(&doc("1.16.99", "")), "UnsupportedVersion");
    // A partial version is a version: 1.21 is 1.21.0, which has no schema.
    assert_eq!(rejects(&doc("1.21", "")), "UnsupportedVersion");
    assert_eq!(rejects(&doc("*", "")), "UnsupportedVersion");
}

#[test]
fn a_file_without_the_payload_misses_a_required_child() {
    let rejected = TextureSet::parse(
        &path(),
        br#"{ "format_version": "1.21.30" }"#,
        &LoadContext::detached(),
    )
    .expect_err("rejected");
    let found = rendered(rejected.diagnostics.iter());
    assert_eq!(found.len(), 2, "{found:?}");
    assert_eq!(
        (found[0].0, found[0].3.as_str()),
        (
            "MissingPayload",
            "Required child minecraft:texture_set not found"
        )
    );
    assert_eq!(found[1].3, "Failed to validate schema");
    // A payload that is no object.
    let rejected = TextureSet::parse(
        &path(),
        br#"{ "format_version": "1.21.30", "minecraft:texture_set": [] }"#,
        &LoadContext::detached(),
    )
    .expect_err("rejected");
    let found = rendered(rejected.diagnostics.iter());
    assert_eq!(
        (found[0].0, found[0].3.as_str()),
        (
            "InvalidType",
            "unknown child schema option type.  Allowed types: 'object'"
        )
    );
}

/// The identity step invents no `metalness_emissive_roughness_subsurface`, keeps `color` and
/// reports nothing.
#[test]
fn the_step_keeps_the_color_and_invents_no_mers() {
    let input: Value = facet_json::from_str(r#"{"color": "stone"}"#).unwrap();
    let out = step_output::<v1_16_100::TextureSet>(&input, Run::default(), &Checks::default())
        .expect("the step runs");
    let payload = out.payload.as_object().expect("the payload is an object");
    assert!(!payload.contains_key("metalness_emissive_roughness_subsurface"));
    assert_eq!(
        payload
            .get("color")
            .and_then(Value::as_string)
            .map(facet_value::VString::as_str),
        Some("stone")
    );
    assert!(out.diagnostics.is_empty());
}

#[test]
fn parse_as_written_gives_the_struct_of_the_file_s_version() {
    let old = doc(
        "1.16.100",
        r##""color": "stone", "metalness_emissive_roughness": "#102030""##,
    );
    let written =
        TextureSet::parse_as_written(&path(), old.as_bytes(), &LoadContext::detached()).unwrap();
    let Any::V1_16_100(set) = written.value() else {
        panic!("{:?}", written.value());
    };
    assert_eq!(
        set.metalness_emissive_roughness,
        Some(LayerValue::Hex(Rgb {
            r: 16.0,
            g: 32.0,
            b: 48.0
        }))
    );
    assert_eq!(written.version().parsed_as, V16);
    assert!(!written.version().upgraded);
    // The step gives what `parse` gives.
    let upgraded = history::upgrade(written, &LoadContext::detached()).unwrap();
    assert!(upgraded.version().upgraded);
    assert_eq!(*upgraded.value(), parse(&old));
    assert_eq!(upgraded.metalness_emissive_roughness_subsurface, None);
    // A 1.21.30 file is the newest struct already.
    let new = doc("1.21.30", r#""color": "stone""#);
    let written =
        TextureSet::parse_as_written(&path(), new.as_bytes(), &LoadContext::detached()).unwrap();
    assert!(matches!(written.value(), Any::V1_21_30(_)));
    let rejected =
        TextureSet::parse_as_written(&path(), b"{", &LoadContext::detached()).expect_err("no JSON");
    assert_eq!(rejected.origin.path.as_str(), STONE_SET);
}

#[test]
fn a_set_is_written_back_to_the_same_set() {
    let texts = [
        // The vanilla file with a uniform layer.
        String::from_utf8(fixture_file(
            FAMILY,
            "ts42_vanilla_census",
            "pack_a/textures/flame_atlas.texture_set.json",
        ))
        .unwrap(),
        doc(
            "1.21.30",
            r##""color": "#11223344", "normal": "n", "metalness_emissive_roughness": "#0a0B0c""##,
        ),
        doc(
            "1.21.30",
            r#""color": [1, 2, 3, 4], "heightmap": "h", "metalness_emissive_roughness": [5, 6, 7]"#,
        ),
        doc(
            "1.16.100",
            r#""color": "stone", "metalness_emissive_roughness": "stone_mer""#,
        ),
    ];
    for text in texts {
        let first = parse(&text);
        let written = first.to_json().unwrap();
        // The strict re-read: the written text is plain JSON.
        let value: Value = facet_json::from_str(&written).unwrap();
        let root = value.as_object().unwrap();
        assert_eq!(
            root.get("format_version")
                .and_then(Value::as_string)
                .unwrap()
                .as_str(),
            "1.21.30"
        );
        assert_eq!(root.len(), 2);
        let second = parse(&written);
        assert_eq!(second, first, "{text}");
        assert_eq!(second.to_json().unwrap(), written, "an identical rewrite");
        assert_eq!(root.get("minecraft:texture_set"), Some(&first.to_value()));
    }
}

#[test]
fn a_layer_the_set_does_not_hold_is_not_written() {
    let set = TextureSet {
        color: Some(LayerValue::Texture(TextureName("stone".to_owned()))),
        ..TextureSet::default()
    };
    assert_eq!(
        set.to_json()
            .unwrap()
            .split_whitespace()
            .collect::<String>(),
        r#"{"format_version":"1.21.30","minecraft:texture_set":{"color":"stone"}}"#
    );
    // A hex colour is written alpha first, in upper case; an array as integers in the range.
    let set = TextureSet {
        color: Some(LayerValue::Hex(Rgba {
            r: 187.0,
            g: 204.0,
            b: 221.0,
            a: 170.0,
        })),
        metalness_emissive_roughness: Some(LayerValue::Array(Rgb {
            r: 0.4,
            g: 300.0,
            b: -2.0,
        })),
        ..TextureSet::default()
    };
    let payload = set.to_value();
    let payload = payload.as_object().unwrap();
    assert_eq!(
        payload
            .get("color")
            .and_then(Value::as_string)
            .unwrap()
            .as_str(),
        "#AABBCCDD"
    );
    let mer: Value = facet_json::from_str("[0, 255, 0]").unwrap();
    assert_eq!(payload.get("metalness_emissive_roughness"), Some(&mer));
    // The members come in the schema's order.
    let keys: Vec<&str> = payload.iter().map(|(key, _)| key.as_str()).collect();
    assert_eq!(keys, ["color", "metalness_emissive_roughness"]);
    assert_eq!(set.texture(LayerKind::Color), None);
    assert_eq!(set.textures().count(), 0);
}

#[test]
fn the_image_layers_are_listed_in_the_schema_s_order() {
    let set = parse(&doc(
        "1.21.30",
        r#""metalness_emissive_roughness_subsurface": "s", "normal": "n", "color": "c""#,
    ));
    let listed: Vec<(LayerKind, &str)> = set
        .textures()
        .map(|(layer, name)| (layer, name.as_str()))
        .collect();
    assert_eq!(
        listed,
        [
            (LayerKind::Color, "c"),
            (LayerKind::Normal, "n"),
            (LayerKind::Mers, "s")
        ]
    );
    assert_eq!(
        LayerKind::Mers.key(),
        "metalness_emissive_roughness_subsurface"
    );
    assert_eq!(LayerKind::Heightmap.key(), "heightmap");
}
