//! `v01`–`v14`) and the gates before either parser

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::fog::{DistanceType, FogColor, Keyframed};
use bedrock_addon::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput};
use bedrock_addon::versioning::{RangeModel, Selection, select_versions};

use super::common::{
    clean, count, expect, expect_at, expect_prefix, find, fixture, lines, loaded, rejected,
};
use super::support::{
    AIR, FAMILY, V1_16_100, V1_21_0, V1_21_90, V1_26_20, air, base_game, density, doc, engine,
    pack, parse, refused,
};

const INVALID: &str = "Invalid 'format_version' in fog settings JSON";
const UNSUPPORTED: &str = "Unsupported 'format_version' value in fog settings JSON";

/// `doc("1.21.90", …)` with its version member replaced by `written` (raw JSON).
fn versioned(written: &str) -> String {
    doc("1.21.90", "").replace("\"1.21.90\"", written)
}

#[test]
fn v01_missing_version() {
    let text = doc("1.21.90", "").replace(r#""format_version": "1.21.90", "#, "");
    let d = rejected(parse(&text, &pack()));
    expect(&d, "MissingFormatVersion", Severity::Error, INVALID);
    assert_eq!(d.len(), 1, "{:?}", lines(&d));
}

#[test]
fn v02_number_version() {
    for written in ["1.16", "null", "true", "{}", "\"abc\"", "\"\"", "[1, 2]"] {
        let d = rejected(parse(&versioned(written), &pack()));
        expect(&d, "MissingFormatVersion", Severity::Error, INVALID);
        assert_eq!(d.len(), 1, "{written}: {:?}", lines(&d));
    }
}

#[test]
fn v03_below_floor() {
    for version in ["1.16.0", "1.16.99", "1.16.100-beta.1", "0.0.1"] {
        let d = rejected(parse(&doc(version, ""), &pack()));
        expect(&d, "UnsupportedVersion", Severity::Error, UNSUPPORTED);
        assert_eq!(d.len(), 1, "{version}: {:?}", lines(&d));
    }
    // The floor itself is read.
    let document = loaded(parse(&doc("1.16.100", ""), &pack()));
    assert_eq!(document.version().parsed_as, V1_16_100);
}

#[test]
fn v04_floor_the_vanilla_default_fog() {
    let text = fixture(FAMILY, "default_fog_setting.json");
    let document = loaded(parse(&text, &base_game()));
    clean(document.diagnostics());
    assert_eq!(document.version().written, Some(V1_16_100));
    assert_eq!(document.version().parsed_as, V1_16_100);
    assert!(document.version().upgraded);
    assert_eq!(
        document.description.identifier.as_str(),
        "minecraft:fog_default"
    );
    let distance = document.distance.as_ref().unwrap();
    let air = distance.air.as_ref().unwrap();
    assert_eq!((air.fog_start(), air.fog_end()), (0.92, 1.0));
    assert_eq!(air.render_distance_type(), DistanceType::Render);
    assert_eq!(
        air.fog_color.as_ref().unwrap().rgb(),
        [171.0 / 255.0, 210.0 / 255.0, 1.0]
    );
    let water = distance.water.as_ref().unwrap();
    assert_eq!((water.fog_start(), water.fog_end()), (0.0, 60.0));
    assert_eq!(water.render_distance_type(), DistanceType::Fixed);
    let transition = water.transition_fog.as_ref().unwrap();
    assert_eq!(transition.init_fog.as_ref().unwrap().fog_end, Some(0.01));
    assert_eq!(
        (transition.min_percent, transition.mid_seconds),
        (Some(0.25), Some(5.0))
    );
    assert_eq!(
        (transition.mid_percent, transition.max_seconds),
        (Some(0.6), Some(30.0))
    );
    assert!(distance.powder_snow.is_none() && document.volumetric.is_none());
    assert_eq!(distance.lava_resistance.as_ref().unwrap().fog_start(), 2.0);
}

#[test]
fn v05_1_21_0_weather_density() {
    let weather =
        r#""volumetric": {"density": {"weather": {"max_density": 0.1, "uniform": true}}}"#;
    let document = loaded(parse(&doc("1.21.0", weather), &pack()));
    assert_eq!(document.version().parsed_as, V1_21_0);
    let density = document
        .volumetric
        .as_ref()
        .unwrap()
        .density
        .as_ref()
        .unwrap();
    assert_eq!(
        density.weather.as_ref().unwrap().max_density,
        Some(Keyframed::Single(0.1))
    );

    let d = rejected(parse(&doc("1.20.80", weather), &pack()));
    expect_at(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "child 'weather' not valid here.",
        "minecraft:fog_settings/volumetric/density/weather",
    );
    refused(&d);
}

fn lush_caves() -> String {
    fixture(FAMILY, "lush_caves_fog_setting.json")
}

#[test]
fn v06_1_21_90_henyey_greenstein() {
    let document = loaded(parse(&lush_caves(), &base_game()));
    clean(document.diagnostics());
    assert_eq!(document.version().parsed_as, V1_21_90);
    let volumetric = document.volumetric.as_ref().unwrap();
    let g = volumetric.henyey_greenstein_g.as_ref().unwrap();
    assert_eq!(
        g.air.as_ref().unwrap().henyey_greenstein_g,
        Some(Keyframed::Single(0.4))
    );
    let media = volumetric.media_coefficients.as_ref().unwrap();
    let scattering = media.air.as_ref().unwrap().scattering.as_ref().unwrap();
    assert_eq!(scattering.single().unwrap().rgb(), [0.06, 0.06, 0.06]);

    let d = rejected(parse(
        &lush_caves().replace("1.21.90", "1.21.80"),
        &base_game(),
    ));
    expect_at(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "child 'henyey_greenstein_g' not valid here.",
        "minecraft:fog_settings/volumetric/henyey_greenstein_g",
    );
    refused(&d);
}

#[test]
fn v07_1_26_20_cereal_gives_the_same_definition() {
    let converted = loaded(parse(&lush_caves(), &base_game()));
    let direct = loaded(parse(
        &lush_caves().replace("1.21.90", "1.26.20"),
        &base_game(),
    ));
    clean(direct.diagnostics());
    assert_eq!(direct.version().parsed_as, V1_26_20);
    assert!(!direct.version().upgraded);
    assert!(converted.version().upgraded);
    assert_eq!(direct.value(), converted.value());
}

/// `experimental_y_2026_drop_4/fogs/ice_caves_fog_setting.json` of 1.26.60 (fog 1.21.90 → 1.26.20): it loads from the base-game pack without a
/// warning, upgraded; the same text stamped 1.26.20 is the same definition; it round-trips.
#[test]
fn the_drop_4_ice_caves_fog_upgrades_and_round_trips() {
    let text = fixture(FAMILY, "ice_caves_fog_setting.json");
    let converted = loaded(parse(&text, &base_game()));
    clean(converted.diagnostics());
    assert_eq!(converted.version().parsed_as, V1_21_90);
    assert!(converted.version().upgraded);
    let direct = loaded(parse(&text.replace("1.21.90", "1.26.20"), &base_game()));
    clean(direct.diagnostics());
    assert_eq!(direct.value(), converted.value());
    super::common::round_trip(&converted, |t| parse(t, &base_game()));
}

#[test]
fn v08_above_engine() {
    let above = FormatVersion::new(
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch + 1,
    )
    .to_string();
    let d = rejected(parse(&doc(&above, ""), &pack()));
    expect(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        &format!("unsupported version: no parser available for version {above}"),
    );
    refused(&d);
}

#[test]
fn v08b_above_latest_below_engine() {
    // The game's parser is exact: 1.26.30 has no parser although the engine is 1.26.30.
    let d = rejected(parse(&doc("1.26.30", ""), &engine(1, 26, 30)));
    expect(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "unsupported version: no parser available for version 1.26.30",
    );
    refused(&d);
    // One patch above the newest struct is already too far.
    let d = rejected(parse(&doc("1.26.21", ""), &pack()));
    expect_prefix(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "unsupported version",
    );
}

#[test]
fn v09_prerelease() {
    let g = r#""volumetric": {"henyey_greenstein_g": {"air": {"henyey_greenstein_g": 0.4}}}"#;
    let d = rejected(parse(&doc("1.21.90-beta.1", g), &pack()));
    assert!(find(&d, "PrereleaseFormatVersion", Severity::Info).is_some());
    expect(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "child 'henyey_greenstein_g' not valid here.",
    );
    let document = loaded(parse(&doc("1.21.90-beta.1", ""), &pack()));
    assert_eq!(document.version().parsed_as, V1_21_0);
    assert!(
        find(
            document.diagnostics(),
            "PrereleaseFormatVersion",
            Severity::Info
        )
        .is_some()
    );
}

/// TOML `vanilla = "Invalid 'format_version' in fog settings JSON"`: the literal is a beta
/// document to us, and the fog family has no beta struct, so it has no parser with or without
/// the toggle (the slice is selected before the permission).
#[test]
fn v10_beta_literal_deviation() {
    let d = rejected(parse(&doc("beta", ""), &pack()));
    assert_eq!(count(&d, "UnsupportedVersion", Severity::Error), 1);
    assert_eq!(d.len(), 1, "{:?}", lines(&d));

    let mut toggled = pack();
    toggled.beta_apis = true;
    let d = rejected(parse(&doc("beta", ""), &toggled));
    assert_eq!(count(&d, "UnsupportedVersion", Severity::Error), 1);
    assert_eq!(d.len(), 1, "{:?}", lines(&d));
}

#[test]
fn v11_partial_version() {
    let weather =
        r#""volumetric": {"density": {"weather": {"max_density": 0.1, "uniform": true}}}"#;
    let document = loaded(parse(&doc("1.21", weather), &pack()));
    assert_eq!(document.version().parsed_as, V1_21_0);

    let red = air(&AIR.replace("\"#ABD2FF\"", "[255, 0, 0]"));
    let document = loaded(parse(&doc("1.26", &red), &pack()));
    assert_eq!(document.version().parsed_as, V1_21_90);
    let color = document
        .distance
        .as_ref()
        .unwrap()
        .air
        .as_ref()
        .unwrap()
        .fog_color
        .clone();
    assert_eq!(color, Some(FogColor::Rgb255([255, 0, 0])));
    assert_eq!(color.unwrap().rgb(), [1.0, 0.0, 0.0]);
}

/// a version array is refused at the envelope; the game's dispatcher
/// takes it and the parser it selects then fails.
#[test]
fn v12_array_version_deviation() {
    for written in ["[1, 21, 90]", "[1, 26, 20]"] {
        let d = rejected(parse(&versioned(written), &pack()));
        assert_eq!(count(&d, "MalformedFormatVersion", Severity::Error), 1);
        assert_eq!(d.len(), 1, "{written}: {:?}", lines(&d));
    }
}

#[test]
fn v13_boundary_1_26_20() {
    let members = air(&format!(r#"{AIR}, "fog_density": 1"#));
    for version in ["1.26.19", "1.26.20-beta.1"] {
        let d = rejected(parse(&doc(version, &members), &pack()));
        expect_at(
            &d,
            "UnrecognizedField",
            Severity::Error,
            "child 'fog_density' not valid here.",
            "minecraft:fog_settings/distance/air/fog_density",
        );
        refused(&d);
    }
    // The two below the boundary are read by the 1.21.90 struct.
    let below = loaded(parse(&doc("1.26.20-beta.1", &air(AIR)), &pack()));
    assert_eq!(below.version().parsed_as, V1_21_90);

    let document = loaded(parse(&doc("1.26.20", &members), &pack()));
    assert_eq!(document.version().parsed_as, V1_26_20);
    let unknown = find(document.diagnostics(), "UnrecognizedField", Severity::Info).unwrap();
    assert_eq!(
        unknown.location.json_path.to_string(),
        "minecraft:fog_settings/distance/air/fog_density"
    );
}

#[test]
fn v14_extra_root_member() {
    let extra = |version: &str| doc(version, "").replacen('{', r#"{"extra": 1, "#, 1);
    let d = rejected(parse(&extra("1.26.20"), &pack()));
    expect(
        &d,
        "MalformedInput",
        Severity::Error,
        "A Minecraft Document must have exactly two members: \"format_version\" and a payload member",
    );
    refused(&d);

    let d = rejected(parse(&extra("1.21.90"), &pack()));
    expect_at(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "child 'extra' not valid here.",
        "extra",
    );
    refused(&d);
}

// ---- the gates before either parser ----

#[test]
fn a_syntax_error_is_the_reader_s_parse_error() {
    let d = rejected(parse(r#"{"format_version": "1.26.20","#, &pack()));
    expect_prefix(&d, "MalformedInput", Severity::Error, "parse error:\n");
    assert_eq!(d.len(), 1, "{:?}", lines(&d));
}

#[test]
fn s03_an_empty_file_and_an_empty_value() {
    let d = rejected(parse("", &pack()));
    expect(
        &d,
        "EmptyDocument",
        Severity::Error,
        "The raw Json in was empty after getting resources.",
    );
    for text in ["{}", "null", "[]"] {
        let d = rejected(parse(text, &pack()));
        expect(
            &d,
            "EmptyDocument",
            Severity::Error,
            "The raw Json was empty after parse.",
        );
        assert_eq!(d.len(), 1, "{text}: {:?}", lines(&d));
    }
}

#[test]
fn the_identifier_is_read_before_the_version() {
    const TEXT: &str = "Could not parse Fog Settings Identifier";
    for text in [
        // No version either: the identifier gate comes first.
        r#"{"minecraft:fog_settings": {"description": {}}}"#.to_owned(),
        r#"{"format_version": "1.26.20", "minecraft:fog_settings": {}}"#.to_owned(),
        r#"{"format_version": "1.26.20", "other": {}}"#.to_owned(),
        r#"[1]"#.to_owned(),
        doc("1.26.20", "").replace("\"demo:fog\"", "\"\""),
        doc("1.26.20", "").replace("\"demo:fog\"", "null"),
        doc("1.26.20", "").replace("\"demo:fog\"", "7"),
        doc("1.21.90", "").replace("\"demo:fog\"", "[]"),
    ] {
        let d = rejected(parse(&text, &pack()));
        expect(&d, "MissingIdentifier", Severity::Error, TEXT);
        assert_eq!(d.len(), 1, "{text}: {:?}", lines(&d));
    }
}

#[test]
fn a_gate_s_finding_has_no_closing_line() {
    for text in [doc("1.16.0", ""), versioned("7"), "{}".to_owned()] {
        let d = rejected(parse(&text, &pack()));
        assert!(
            d.iter().all(|f| f.code.name() != "DocumentRejected"),
            "{:?}",
            lines(&d)
        );
    }
}

/// The chain is contiguous from the floor and closed at its newest struct.
#[test]
fn the_chain_selects_by_range_and_is_closed_at_1_26_20() {
    let versions = [V1_16_100, V1_21_0, V1_21_90, V1_26_20];
    let select = |version: (u16, u16, u16), prerelease| {
        select_versions(
            &versions,
            V1_16_100,
            RangeModel::ClosedToLatest,
            None,
            &FormatVersionInput {
                version: Some(FormatVersion::new(version.0, version.1, version.2)),
                beta_literal: false,
                prerelease,
            },
        )
    };
    assert_eq!(select((1, 16, 99), false), Selection::BelowFloor);
    assert_eq!(select((1, 16, 100), false), Selection::Struct(0));
    assert_eq!(select((1, 20, 80), false), Selection::Struct(0));
    assert_eq!(select((1, 21, 0), false), Selection::Struct(1));
    assert_eq!(select((1, 21, 89), false), Selection::Struct(1));
    assert_eq!(select((1, 21, 90), false), Selection::Struct(2));
    assert_eq!(select((1, 26, 19), false), Selection::Struct(2));
    // The range model covers no pre-release of the newest struct; the game's dispatcher is a
    // plain `<` and sends it to the 1.21.90 root, which the loader does itself
    // (`v13_boundary_1_26_20`).
    assert_eq!(select((1, 26, 20), true), Selection::Unsupported);
    assert_eq!(select((1, 26, 20), false), Selection::Struct(3));
    assert_eq!(select((1, 26, 21), false), Selection::Unsupported);
    assert_eq!(select((1, 26, 50), false), Selection::Unsupported);
}

#[test]
fn a_density_without_its_heights_is_refused_on_both_sides_of_1_26_20() {
    // V19 and V19a are the density node's rules too, not the 1.26.20 loader's alone.
    let only_max = density(r#""max_density": 0.1"#);
    refused(&rejected(parse(&doc("1.21.90", &only_max), &pack())));
    refused(&rejected(parse(&doc("1.26.20", &only_max), &pack())));
}
