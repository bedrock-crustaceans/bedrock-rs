//! The validation rules the fixture rows do not name: the two texts of a failed strict load,
//! the members a struct's version does not bind yet, colours, the client's own checks, and the
//! round trip.

use bedrock_addon::diagnostic::{Disposition, Kind, Severity};
use bedrock_addon::document::{Content, Placed};
use bedrock_addon::families::client_biome::component::{
    AmbientSounds, ClientBiomeComponent, FoliageAppearance, FoliageColor, FoliageColorMap,
    GrassColorMap, SkyColor, SoundAddition,
};
use bedrock_addon::families::client_biome::{ClientBiome, Color255Rgb};
use bedrock_addon::id::BiomeId;

use super::common::{
    PATH, biome, component, context, count, errors, has, has_at, json, loads, rejected, texts,
    written,
};

/// V2.1: any failure of a struct's strict load is followed by vanilla's two texts.
#[test]
fn a_failed_strict_load_carries_vanillas_two_texts() {
    for version in ["1.21.40", "1.21.130"] {
        let found = rejected(
            &biome(version, "myns:glade", r#""minecraft:sky_color": {}"#),
            &context(),
        );
        assert!(
            has_at(
                &found,
                |c| matches!(c.id(), "core/missing_required_field"),
                Severity::Error,
                "/minecraft:client_biome/components/minecraft:sky_color/sky_color",
            ),
            "{:?}",
            texts(&found)
        );
        assert!(has(
            &found,
            |_| true,
            Severity::Error,
            "Invalid Client Biome file"
        ));
        assert!(has(
            &found,
            |_| true,
            Severity::Error,
            &format!("Failed to load client biome document: {PATH}"),
        ));
    }
    // A document that loads has neither.
    let loaded = loads(&biome("1.21.130", "myns:glade", ""), &context());
    assert!(loaded.diagnostics().is_empty());
    // `description` and `components` are required, and `identifier` in the description.
    for payload in [
        r#"{"components": {}}"#,
        r#"{"description": {"identifier": "a:b"}}"#,
        r#"{"description": {}, "components": {}}"#,
    ] {
        let found = rejected(&super::common::document("1.21.130", payload), &context());
        assert_eq!(
            count(&found, |c| matches!(c.id(), "core/missing_required_field")),
            1,
            "{payload}: {:?}",
            texts(&found)
        );
    }
    // Below 1.21.130 both root members are required of the one struct.
    let found = rejected(r#"{"format_version": "1.21.40"}"#, &context());
    assert_eq!(
        count(&found, |c| matches!(c.id(), "core/missing_required_field")),
        1
    );
}

/// A component that is no object rejects the document.
#[test]
fn a_component_of_the_wrong_type_is_an_error() {
    let found = rejected(
        &biome("1.21.130", "myns:glade", r#""minecraft:sky_color": 3"#),
        &context(),
    );
    assert!(
        count(&found, |c| matches!(c.id(), "core/invalid_type")) >= 1,
        "{:?}",
        texts(&found)
    );
    let found = rejected(
        &biome(
            "1.21.130",
            "myns:glade",
            r#""minecraft:biome_music": {"volume_multiplier": "loud"}"#,
        ),
        &context(),
    );
    assert!(has_at(
        &found,
        |c| matches!(c.id(), "core/invalid_type"),
        Severity::Error,
        "/minecraft:client_biome/components/minecraft:biome_music/volume_multiplier",
    ));
}

/// a member a later struct added is, in an older document, the error
/// vanilla's strict loader raises; from its version it is read.
#[test]
fn a_member_of_a_later_struct_is_an_error_below_its_version() {
    for (below, since, component, member, value) in [
        (
            "1.21.69",
            "1.21.70",
            "biome_music",
            "music_definition",
            r#""m""#,
        ),
        (
            "1.21.109",
            "1.21.110",
            "biome_music",
            "underwater_music",
            "true",
        ),
        (
            "1.21.99",
            "1.21.100",
            "grass_appearance",
            "grass_is_shaded",
            "true",
        ),
        (
            "1.21.119",
            "1.21.120",
            "ambient_sounds",
            "underwater_addition",
            r#"{"asset": "a", "chance": 0.5}"#,
        ),
        (
            "1.21.119",
            "1.21.120",
            "ambient_sounds",
            "underwater_loop",
            r#""l""#,
        ),
        (
            "1.21.119",
            "1.21.120",
            "ambient_sounds",
            "underwater_mood",
            r#""m""#,
        ),
    ] {
        let body = format!(r#""minecraft:{component}": {{"{member}": {value}}}"#);
        let found = rejected(&biome(below, "myns:glade", &body), &context());
        assert!(
            has_at(
                &found,
                |c| matches!(c.id(), "core/use_format_version"),
                Severity::Error,
                &format!("/minecraft:client_biome/components/minecraft:{component}/{member}"),
            ) && has(
                &found,
                |_| true,
                Severity::Error,
                &format!(
                    "To use field '{component}.{member}', use json format version {since} or higher"
                ),
            ),
            "{below} {member}: {:?}",
            texts(&found)
        );
        // One finding for the member: it is not also an unknown member.
        assert_eq!(
            count(&found, |c| {
                matches!(c.kind(), Kind::UnknownMember(Disposition::Kept))
            }),
            0
        );
        // Raised while the struct is read: a finding of the parse.
        assert!(
            found
                .iter()
                .filter(|f| matches!(f.code.id(), "core/use_format_version"))
                .all(|f| f.phase == bedrock_addon::diagnostic::Phase::Parse),
            "{below} {member}"
        );
        let loaded = loads(&biome(since, "myns:glade", &body), &context());
        assert!(
            loaded
                .diagnostics()
                .iter()
                .all(|d| d.severity < Severity::Warning),
            "{since} {member}: {:?}",
            texts(loaded.diagnostics())
        );
    }
}

/// the `dry_foliage` colour map is a value of the 1.21.70 enum.
#[test]
fn the_dry_foliage_map_is_an_enum_error_below_1_21_70() {
    let body = r#""minecraft:foliage_appearance": {"color": {"color_map": "dry_foliage"}}"#;
    let found = rejected(&biome("1.21.69", "myns:glade", body), &context());
    assert!(
        has_at(
            &found,
            |c| matches!(c.id(), "core/invalid_enum_value"),
            Severity::Error,
            "/minecraft:client_biome/components/minecraft:foliage_appearance/color/color_map",
        ),
        "{:?}",
        texts(&found)
    );
    let loaded = loads(&biome("1.21.70", "myns:glade", body), &context());
    assert_eq!(
        loaded.components.get::<FoliageAppearance>().unwrap().color,
        Some(FoliageColor::Map {
            color_map: FoliageColorMap::DryFoliage,
            extra: bedrock_addon::document::UnknownMembers::default(),
        })
    );
    // The five older maps are read by the first struct.
    for map in [
        "foliage",
        "birch",
        "evergreen",
        "mangrove_swamp_foliage",
        "swamp_foliage",
    ] {
        let body =
            format!(r#""minecraft:foliage_appearance": {{"color": {{"color_map": "{map}"}}}}"#);
        let loaded = loads(&biome("1.21.40", "myns:glade", &body), &context());
        assert_eq!(
            component(&loaded, "minecraft:foliage_appearance"),
            Some(json(&format!(r#"{{"color": {{"color_map": "{map}"}}}}"#)))
        );
    }
    assert_eq!(FoliageColorMap::DryFoliage.name(), "dry_foliage");
    assert_eq!(GrassColorMap::SwampGrass.name(), "swamp_grass");
    assert_eq!(GrassColorMap::Grass.name(), "grass");
}

/// A colour map's container: `color_map` is required and names a map of its component; other
/// members are kept with a warning.
#[test]
fn a_colour_map_container_is_checked() {
    let at = "/minecraft:client_biome/components/minecraft:grass_appearance/color/color_map";
    let grass = |color: &str| {
        biome(
            "1.21.130",
            "myns:glade",
            &format!(r#""minecraft:grass_appearance": {{"color": {color}}}"#),
        )
    };
    let unknown = rejected(&grass(r#"{"color_map": "birch"}"#), &context());
    assert!(has_at(
        &unknown,
        |c| matches!(c.id(), "core/invalid_enum_value"),
        Severity::Error,
        at
    ));
    let missing = rejected(&grass("{}"), &context());
    assert!(has_at(
        &missing,
        |c| matches!(c.id(), "core/missing_required_field"),
        Severity::Error,
        at
    ));
    let wrong = rejected(&grass(r#"{"color_map": 1}"#), &context());
    assert!(has_at(
        &wrong,
        |c| matches!(c.id(), "core/invalid_type"),
        Severity::Error,
        at
    ));
    for map in ["grass", "swamp_grass"] {
        loads(&grass(&format!(r#"{{"color_map": "{map}"}}"#)), &context());
    }
    // A member the container does not have is an unknown member: an error under the strict
    // mask, the document refused.
    let text = grass(r#"{"color_map": "grass", "note": 1}"#);
    let extra = rejected(&text, &context());
    assert!(has_at(
        &extra,
        |c| matches!(c.id(), "core/unrecognized_field"),
        Severity::Error,
        "/minecraft:client_biome/components/minecraft:grass_appearance/color/note",
    ));
}

const COLOR_TEXT: &str = "Expected [r, g, b], with values in the 0-255 range, or a hex value with 6 characters (e.g. RGB as #FFFFFF).";

/// V2.10: every `Color255RGB` member is `"#RRGGBB"` or three integers.
#[test]
fn a_colour_of_another_form_is_vanillas_error() {
    for (component, member) in [
        ("sky_color", "sky_color"),
        ("dry_foliage_color", "color"),
        ("foliage_appearance", "color"),
        ("grass_appearance", "color"),
        ("water_appearance", "surface_color"),
    ] {
        for bad in [
            r##""#FFFFFFFF""##,
            r##""#FFFFF""##,
            r#""FFFFFF""#,
            r##""#GGGGGG""##,
            "[1, 2]",
            "[1, 2, 3, 4]",
            "[1.5, 2, 3]",
            "7",
        ] {
            let found = rejected(
                &biome(
                    "1.21.130",
                    "myns:glade",
                    &format!(r#""minecraft:{component}": {{"{member}": {bad}}}"#),
                ),
                &context(),
            );
            assert!(
                has_at(
                    &found,
                    |c| matches!(c.id(), "client_biome/color"),
                    Severity::Error,
                    &format!("/minecraft:client_biome/components/minecraft:{component}/{member}"),
                ) && has(&found, |_| true, Severity::Error, COLOR_TEXT),
                "{component} {bad}: {:?}",
                texts(&found)
            );
        }
        for good in [r##""#a0B1c2""##, "[0, 128, 255]"] {
            let text = biome(
                "1.21.130",
                "myns:glade",
                &format!(r#""minecraft:{component}": {{"{member}": {good}}}"#),
            );
            let loaded = loads(&text, &context());
            assert!(loaded.diagnostics().is_empty(), "{component} {good}");
            assert_eq!(written(&loaded), json(&text));
        }
    }
}

/// V2.11 (ours; vanilla clamps silently in `colorFromIntArray3`, no line): an array channel
/// outside `0..=255` is clamped, with an information at the channel.
#[test]
fn a_colour_channel_out_of_range_is_clamped_deviation() {
    let text = biome(
        "1.21.130",
        "myns:glade",
        r#""minecraft:sky_color": {"sky_color": [-1, 256, 255]}"#,
    );
    let loaded = loads(&text, &context());
    for index in [0, 1] {
        assert!(has_at(
            loaded.diagnostics(),
            |c| matches!(c.id(), "core/invalid_value"),
            Severity::Info,
            &format!("/minecraft:client_biome/components/minecraft:sky_color/sky_color/{index}"),
        ));
    }
    assert_eq!(
        count(loaded.diagnostics(), |c| matches!(
            c.id(),
            "core/invalid_value"
        )),
        2
    );
    let sky = loaded.components.get::<SkyColor>().unwrap();
    assert_eq!(sky.sky_color.rgb(), Some([0, 255, 255]));
    // Written back as the author wrote it.
    assert_eq!(written(&loaded), json(&text));
    let exact = loads(
        &biome(
            "1.21.130",
            "myns:glade",
            r#""minecraft:sky_color": {"sky_color": [0, 255, 7]}"#,
        ),
        &context(),
    );
    assert!(exact.diagnostics().is_empty());
    assert_eq!(
        Color255Rgb::Hex("#0a10fF".to_owned()).rgb(),
        Some([10, 16, 255])
    );
    assert_eq!(Color255Rgb::Hex("0a10ff".to_owned()).rgb(), None);
}

/// One glue failure: the component, the error text, and the document that just passes.
fn glue(bad: &str, good: &[&str], at: &str, text: &str) {
    let found = rejected(&biome("1.21.130", "myns:glade", bad), &context());
    assert!(
        has_at(
            &found,
            |c| c.kind() == Kind::InvalidValue,
            Severity::Error,
            at
        ) && has(&found, |_| true, Severity::Error, text),
        "{bad}: {:?}",
        texts(&found)
    );
    assert_eq!(errors(&found).len(), 1, "{bad}: {:?}", texts(&found));
    for good in good {
        let loaded = loads(&biome("1.21.130", "myns:glade", good), &context());
        assert!(loaded.diagnostics().is_empty(), "{good}");
    }
}

/// V4.7: `biome_music.volume_multiplier` lies in `[0, 1]`.
#[test]
fn v4_7_volume_multiplier_range() {
    let at = "/minecraft:client_biome/components/minecraft:biome_music/volume_multiplier";
    let text = "Volume multiplier must be between 0 and 1, inclusive.";
    let good = [
        r#""minecraft:biome_music": {"volume_multiplier": 0}"#,
        r#""minecraft:biome_music": {"volume_multiplier": 1}"#,
        r#""minecraft:biome_music": {}"#,
    ];
    glue(
        r#""minecraft:biome_music": {"volume_multiplier": 1.01}"#,
        &good,
        at,
        text,
    );
    glue(
        r#""minecraft:biome_music": {"volume_multiplier": -0.01}"#,
        &[],
        at,
        text,
    );
}

/// V4.8: `water_appearance.surface_opacity` lies in `[0, 1]`.
#[test]
fn v4_8_surface_opacity_range() {
    let at = "/minecraft:client_biome/components/minecraft:water_appearance/surface_opacity";
    let text = "surface_opacity field must be between 0 and 1, inclusive.";
    let good = [
        r#""minecraft:water_appearance": {"surface_opacity": 0}"#,
        r#""minecraft:water_appearance": {"surface_opacity": 1}"#,
    ];
    glue(
        r#""minecraft:water_appearance": {"surface_opacity": 1.5}"#,
        &good,
        at,
        text,
    );
    glue(
        r#""minecraft:water_appearance": {"surface_opacity": -1}"#,
        &[],
        at,
        text,
    );
}

/// V4.9: `ambient_sounds` names at least one of its six sounds.
#[test]
fn v4_9_ambient_sounds_needs_a_sound() {
    let at = "/minecraft:client_biome/components/minecraft:ambient_sounds";
    let text = "No addition, loop, mood, or underwater variant sound specified. Specify at least one or remove the component.";
    let good = [
        r#""minecraft:ambient_sounds": {"addition": {"asset": "a", "chance": 1}}"#,
        r#""minecraft:ambient_sounds": {"loop": "a"}"#,
        r#""minecraft:ambient_sounds": {"mood": "a"}"#,
        r#""minecraft:ambient_sounds": {"underwater_addition": {"asset": "a", "chance": 1}}"#,
        r#""minecraft:ambient_sounds": {"underwater_loop": "a"}"#,
        r#""minecraft:ambient_sounds": {"underwater_mood": "a"}"#,
    ];
    glue(r#""minecraft:ambient_sounds": {}"#, &good, at, text);
    glue(
        r#""minecraft:ambient_sounds": {"loop": "", "mood": "", "underwater_loop": "", "underwater_mood": ""}"#,
        &[],
        at,
        text,
    );
}

/// V4.10: `precipitation` has at most one member, whatever its value.
#[test]
fn v4_10_precipitation_one_type() {
    let at = "/minecraft:client_biome/components/minecraft:precipitation";
    let text = "precipitation: multiple precipitation types set for biome";
    let good = [
        r#""minecraft:precipitation": {}"#,
        r#""minecraft:precipitation": {"ash": 0.5}"#,
        r#""minecraft:precipitation": {"blue_spores": 0}"#,
        r#""minecraft:precipitation": {"red_spores": 3}"#,
        r#""minecraft:precipitation": {"white_ash": 1}"#,
    ];
    glue(
        r#""minecraft:precipitation": {"ash": 0, "white_ash": 0}"#,
        &good,
        at,
        text,
    );
    glue(
        r#""minecraft:precipitation": {"blue_spores": 1, "red_spores": 1}"#,
        &[],
        at,
        text,
    );
}

/// The round trip: every component of the newest format is written as it was read, strictly
/// read again, and written identically.
#[test]
fn the_newest_document_round_trips() {
    let text = biome(
        "1.21.130",
        "myns:glade",
        r##""minecraft:ambient_sounds": {"addition": {"asset": "a", "chance": 0.25}, "loop": "l", "mood": "m",
              "underwater_addition": {"asset": "ua", "chance": 1}, "underwater_loop": "ul", "underwater_mood": "um"},
           "minecraft:biome_music": {"volume_multiplier": 0.5, "music_definition": "glade", "underwater_music": false},
           "minecraft:fog_appearance": {"fog_identifier": "myns:fog"},
           "minecraft:foliage_appearance": {"color": {"color_map": "dry_foliage"}},
           "minecraft:grass_appearance": {"color": [1, 2, 3], "grass_is_shaded": false},
           "minecraft:dry_foliage_color": {"color": "#A0B0C0"},
           "minecraft:sky_color": {"sky_color": "#102030"},
           "minecraft:water_appearance": {"surface_color": "#44AFF5", "surface_opacity": 0.65},
           "minecraft:atmosphere_identifier": {"atmosphere_identifier": "myns:a"},
           "minecraft:color_grading_identifier": {"color_grading_identifier": "myns:c"},
           "minecraft:lighting_identifier": {"lighting_identifier": "myns:l"},
           "minecraft:water_identifier": {"water_identifier": "myns:w"},
           "minecraft:cubemap_identifier": {"cubemap_identifier": "myns:sky"},
           "minecraft:precipitation": {"ash": 0.05}"##,
    );
    let loaded = loads(&text, &context());
    assert!(
        loaded.diagnostics().is_empty(),
        "{:?}",
        texts(loaded.diagnostics())
    );
    assert_eq!(loaded.components.len(), 14);
    assert_eq!(written(&loaded), json(&text));
    let once = loaded.to_json().unwrap();
    let again = loads(&once, &context());
    assert_eq!(again.value(), loaded.value());
    assert_eq!(again.to_json().unwrap(), once);
    let mut bytes = Vec::new();
    loaded
        .write_json(&mut bytes, bedrock_addon::json::WriteStyle::default())
        .unwrap();
    assert_eq!(String::from_utf8(bytes).unwrap(), once);
}

/// A member the author did not write is not written back; its accessor gives the default.
#[test]
fn an_absent_member_is_not_written_back() {
    let text = biome(
        "1.21.130",
        "myns:glade",
        r#""minecraft:grass_appearance": {}, "minecraft:biome_music": {}, "minecraft:foliage_appearance": {}"#,
    );
    let loaded = loads(&text, &context());
    assert_eq!(written(&loaded), json(&text));
}

/// A document built in code: the constructors, the default path and the document it writes.
#[test]
fn a_built_document_writes_and_loads() {
    let mut built = ClientBiome::new(BiomeId::from_static("demo:glade"));
    built
        .components
        .insert(SkyColor::new(Color255Rgb::Hex("#102030".to_owned())));
    built.components.insert(AmbientSounds {
        addition: Some(SoundAddition::new(
            bedrock_addon::id::Id::parse("ambient.cave"),
            0.5,
        )),
        ..AmbientSounds::default()
    });
    assert_eq!(built.default_path().as_str(), "biomes/demo/glade.json");
    let text = built.to_json().unwrap();
    assert_eq!(
        json(&text),
        json(
            r##"{"format_version": "1.21.130", "minecraft:client_biome": {
                "description": {"identifier": "demo:glade"},
                "components": {"minecraft:sky_color": {"sky_color": "#102030"},
                    "minecraft:ambient_sounds": {"addition": {"asset": "ambient.cave", "chance": 0.5}}}}}"##
        )
    );
    let loaded = loads(&text, &context());
    assert_eq!(*loaded.value(), built);
    let sky: ClientBiomeComponent = SkyColor::new(Color255Rgb::Array([1, 2, 3])).into();
    assert!(matches!(sky, ClientBiomeComponent::SkyColor(_)));
}

/// V1.2a: the game's document input takes the first `"format_version"` key of the text,
/// here the payload's 1.21.40, so the legacy parser reads the document and finds no version at
/// its root: "{}: No `format_version` string found" (the legacy parser's version check), and the
/// document is refused.
#[test]
fn a_nested_format_version_selects_the_legacy_parser_which_finds_none_at_the_root() {
    let text = r#"{"minecraft:client_biome": {"format_version": "1.21.40", "description": {"identifier": "a:b"}, "components": {}}}"#;
    let found = rejected(text, &context());
    assert!(
        found
            .iter()
            .any(|f| matches!(f.code.id(), "client_biome/no_version_string")
                && f.severity == Severity::Error
                && f.location.json_path.to_string() == "format_version"
                && f.code.to_string()
                    == "biomes/demo.client_biome.json: No format_version string found"),
        "{:?}",
        texts(&found)
    );
}

/// Of a member written twice the last stands, whole, as everywhere.
#[test]
fn the_last_of_a_doubled_member_stands() {
    let loaded = loads(
        &biome(
            "1.21.130",
            "myns:glade",
            r##""minecraft:sky_color": {"sky_color": "#111111"}, "minecraft:sky_color": {"sky_color": "#222222"}"##,
        ),
        &context(),
    );
    assert_eq!(
        component(&loaded, "minecraft:sky_color"),
        Some(json(r##"{"sky_color": "#222222"}"##))
    );
}

/// An empty file and a `null` document are not loaded.
#[test]
fn an_empty_document_is_rejected() {
    for text in ["", "  // nothing\n", "null"] {
        let found = rejected(text, &context());
        assert_eq!(
            count(&found, |c| matches!(c.id(), "core/empty_document")),
            1,
            "{text:?}: {:?}",
            texts(&found)
        );
    }
    let bytes = [0xff, 0xfe, 0x00];
    assert!(ClientBiome::parse(&super::common::path(), &bytes, &context()).is_err());
    // A byte-order mark is skipped.
    let mut marked = vec![0xef, 0xbb, 0xbf];
    marked.extend(biome("1.21.130", "myns:glade", "").into_bytes());
    assert!(ClientBiome::parse(&super::common::path(), &marked, &context()).is_ok());
}

/// A member gated in one component is, under another component, only a member that component
/// does not have.
#[test]
fn a_gated_member_name_in_another_component_is_an_unknown_member() {
    let found = rejected(
        &biome(
            "1.21.40",
            "myns:glade",
            r#""minecraft:grass_appearance": {"music_definition": "m", "underwater_loop": "l"}"#,
        ),
        &context(),
    );
    let unknown: Vec<_> = found
        .iter()
        .filter(|d| matches!(d.code.id(), "core/unrecognized_field"))
        .collect();
    assert_eq!(unknown.len(), 2, "{:?}", texts(&found));
    assert!(unknown.iter().all(|d| d.severity == Severity::Error));
}

/// A text that is no JSON and has no `"format_version"` the raw scan could find (ours: the
/// syntax finding, with Puv's document text; vanilla: its version scan and its fallback parse
/// both find nothing, so it says "no valid version passed by the caller or found in the
/// input" and never names the syntax error).
#[test]
fn a_text_that_is_no_json_and_has_no_version_keeps_its_syntax_finding_deviation() {
    for text in ["{ not json", r#"{"minecraft:client_biome" {}}"#, "[1, 2"] {
        let found = rejected(text, &context());
        let errors = errors(&found);
        assert_eq!(errors.len(), 1, "{text}: {:?}", texts(&found));
        assert!(
            matches!(errors[0].code.id(), "core/document_json"),
            "{text}"
        );
        assert_eq!(errors[0].severity, Severity::Error);
        assert_eq!(errors[0].location.json_path.to_string(), "");
        assert!(
            errors[0]
                .code
                .to_string()
                .starts_with("Failed to parse the Minecraft Document JSON: "),
            "{text}: {}",
            errors[0].code
        );
        assert_eq!(count(&found, |c| c.kind() == Kind::FormatVersion), 0);
    }
}

/// Rule N2: every client-biome slice reads a
/// the game's document input with a JSON reader that has no null test for a
/// member (the strict reader; the legacy loaders' reader is the same code),
/// so a member written `null`, an optional one included, fails its type and refuses the
/// document, in the legacy slice, the 1.21.130 slice and the beta slice.
#[test]
fn a_null_member_refuses_the_client_biome_on_every_slice() {
    let at = "/minecraft:client_biome/components";
    for version in ["1.21.40", "1.21.90", "1.21.130"] {
        let found = match super::common::load(
            &biome(
                version,
                "myns:glade",
                r##""minecraft:water_appearance": {"surface_color": "#14A2C5", "surface_opacity": null}"##,
            ),
            &context(),
        ) {
            Ok(document) => panic!(
                "{version} opacity: loaded, {:?}",
                texts(document.diagnostics())
            ),
            Err(refused) => refused.diagnostics,
        };
        assert!(
            has_at(
                &found,
                |c| matches!(c.id(), "core/invalid_type"),
                Severity::Error,
                &format!("{at}/minecraft:water_appearance/surface_opacity")
            ),
            "{version}: {:?}",
            texts(&found)
        );
        let found = match super::common::load(
            &biome(version, "myns:glade", r#""minecraft:sky_color": null"#),
            &context(),
        ) {
            Ok(document) => panic!("{version} sky: loaded, {:?}", texts(document.diagnostics())),
            Err(refused) => refused.diagnostics,
        };
        assert!(
            has_at(
                &found,
                |c| matches!(c.id(), "core/invalid_type"),
                Severity::Error,
                &format!("{at}/minecraft:sky_color")
            ),
            "{version}: {:?}",
            texts(&found)
        );
    }
    let beta = super::common::load_beta(
        &biome(
            "beta",
            "myns:glade",
            r##""minecraft:water_appearance": {"surface_color": "#14A2C5", "surface_opacity": null}"##,
        ),
        &super::common::base_game(),
    )
    .expect_err("refused");
    assert!(
        has_at(
            &beta.diagnostics,
            |c| matches!(c.id(), "core/invalid_type"),
            Severity::Error,
            &format!("{at}/minecraft:water_appearance/surface_opacity")
        ),
        "{:?}",
        texts(&beta.diagnostics)
    );
}
