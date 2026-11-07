//! The engine's own biome inputs of 1.26.60 (copied under
//! `fixtures/biome/biome/engine/`). No file names its test; each expectation below is the game's file-load
//! path: a document that loads has no finding
//! and round-trips at the family's value level (`rows::round_trip`); a refused one names its
//!
//! Biome documents the game's own unit tests feed its loader; `case-11.fmt.txt` is a
//! format-string template the test fills before parsing.

use bedrock_addon::diagnostic::{Code, Diagnostics, Severity};
use bedrock_addon::version::FormatVersion;

use super::common::{base_game, context, fixture, load, load_beta, loads, rejected};
use super::rows::round_trip;

/// An engine input by file name.
fn engine(name: &str) -> String {
    fixture(&format!("engine/{name}"))
}

/// The one finding of a refused document: its code (by `matches`), severity and path.
#[track_caller]
fn only(found: &Diagnostics, code: impl Fn(&Code) -> bool, at: &str) {
    let all: Vec<_> = found
        .iter()
        .filter(|d| d.severity >= Severity::Warning)
        .collect();
    assert_eq!(all.len(), 1, "{found:#?}");
    assert!(code(&all[0].code), "{found:#?}");
    assert_eq!(all[0].severity, Severity::Error, "{found:#?}");
    assert_eq!(all[0].location.json_path.to_string(), at, "{found:#?}");
}

/// The stable documents load as the newest struct without a finding and round-trip.
#[test]
fn the_engine_s_stable_biomes_load_and_round_trip() {
    for (name, written) in [
        ("case-1.json", (1, 21, 120)),
        ("case-2.json", (1, 21, 120)),
        ("case-3.json", (1, 21, 120)),
        ("case-7.json", (1, 21, 120)),
        ("case-8.json", (1, 26, 0)),
        ("case-10.json", (1, 21, 120)),
        ("case-12.json", (1, 26, 0)),
        ("case-15.json", (1, 21, 120)),
        ("case-16.json", (1, 21, 120)),
        ("case-21.json", (1, 21, 110)),
    ] {
        let document = loads(&engine(name), &context());
        assert!(
            document.diagnostics().is_empty(),
            "{name}: {:#?}",
            document.diagnostics()
        );
        let (major, minor, patch) = written;
        assert_eq!(
            document.version().written,
            Some(FormatVersion::new(major, minor, patch)),
            "{name}"
        );
        round_trip(document.value());
    }
}

/// case-11 is a template (`{}` = the identifier): instantiated, it is an overworld
/// surface builder at 1.21.120, which loads and round-trips.
#[test]
fn the_engine_s_surface_builder_template_loads_and_round_trips() {
    let text = engine("case-11.fmt.txt")
        .replacen("{}", "test:biome", 1)
        .replace("{{", "{")
        .replace("}}", "}");
    let document = loads(&text, &context());
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    round_trip(document.value());
}

/// The two `"beta"` documents (one with the four experimental attributes) load from a base-game
/// pack without a finding, and read back equal.
#[test]
fn the_engine_s_beta_biomes_load_from_a_base_game_pack() {
    for name in ["case-13.json", "case-17.json"] {
        let beta = load_beta(&engine(name), &base_game())
            .unwrap_or_else(|r| panic!("{name}: {:#?}", r.diagnostics));
        assert!(
            beta.diagnostics().is_empty(),
            "{name}: {:#?}",
            beta.diagnostics()
        );
        let written = beta.to_json().unwrap();
        let again = load_beta(&written, &base_game()).unwrap();
        assert_eq!(again.value(), beta.value(), "{name}");
        assert_eq!(again.to_json().unwrap(), written, "{name}");
    }
    let attributes = load_beta(&engine("case-13.json"), &base_game()).unwrap();
    assert!(attributes.environment_attributes.is_some());
}

/// A document without `format_version` is refused: "no valid version passed by the caller or
/// found in the input" (`MissingVersion::Reject`).
#[test]
fn the_engine_s_biomes_without_a_version_are_refused() {
    for name in [
        "case-6.json",
        "case-9.json",
        "case-14.json",
        "case-19.json",
        "case-20.json",
    ] {
        let found = rejected(&engine(name), &context());
        only(
            &found,
            // By legacy name: the variant and the definitions that were it.
            |c| matches!(c.name(), "MissingFormatVersion"),
            "",
        );
    }
}

/// Below the floor 1.21.110 no slice parses the document: "unsupported version: no parser
/// available for version ", with the format hint.
#[test]
fn the_engine_s_biomes_below_the_floor_are_refused() {
    for name in ["case-4.json", "case-22.json"] {
        let found = rejected(&engine(name), &context());
        only(
            &found,
            // By legacy name: the variant and the definitions that were it.
            |c| matches!(c.name(), "UnsupportedVersion"),
            "format_version",
        );
        assert!(
            found
                .iter()
                // By legacy name: the variant and the definitions that were it.
                .any(|d| matches!(d.code.name(), "BiomeFormatHint")),
            "{name}"
        );
    }
}

/// A swamp surface builder on a biome that replaces a nether biome: "Biomes in Nether are only
/// compatible with following Surface Builders: overworld, `frozen_ocean`, capped, `the_end`", and the document is refused.
#[test]
fn the_engine_s_swamp_builder_in_the_nether_is_refused() {
    let found = rejected(&engine("case-5.json"), &context());
    only(
        &found,
        // By legacy name: the variant and the definitions that were it.
        |c| matches!(c.name(), "InvalidValue"),
        "minecraft:biome/components/minecraft:surface_builder/builder/type",
    );
    assert!(found.iter().any(|d| d.code.to_string()
        == "Biomes in Nether are only compatible with following Surface Builders: overworld, frozen_ocean, capped, the_end"));
}

/// `environment_attributes` is a member of the beta struct only: in the 1.26.0 slice, read with
/// `StrictErrorCodes` (0xfffc), it is an unknown member, an error, and the document is refused.
#[test]
fn the_engine_s_attributes_outside_the_beta_struct_refuse_the_biome() {
    let found = load(&engine("case-18.json"), &context()).expect_err("refused");
    only(
        &found.diagnostics,
        // By legacy name: the variant and the definitions that were it.
        |c| matches!(c.name(), "UnrecognizedField"),
        "minecraft:biome/environment_attributes",
    );
}

/// `experimental_y_2026_drop_4/biomes/ice_caves.biome.json` of 1.26.60: format 1.26.30, the noise gradient in both builders with `range`-only
/// specifiers. It loads from the base-game pack without a finding, as the newest struct, and
/// round-trips.
#[test]
fn the_drop_4_ice_caves_biome_loads_and_round_trips() {
    use bedrock_addon::families::biome::component::{Builder, SubsurfaceBuilder, SurfaceBuilder};

    let document = loads(&fixture("drop4/ice_caves.biome.json"), &base_game());
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 26, 30));
    assert!(!document.version().upgraded);
    round_trip(document.value());
    for builder in [
        &document.components.get::<SurfaceBuilder>().unwrap().builder,
        &document
            .components
            .get::<SubsurfaceBuilder>()
            .unwrap()
            .builder,
    ] {
        let Builder::NoiseGradient(gradient) = builder else {
            panic!("a noise gradient")
        };
        assert_eq!(gradient.noise_block_specifiers.len(), 5);
        assert_eq!(gradient.noise_block_specifiers[0].range(), (0.5, 0.7));
        assert!(
            gradient
                .noise_block_specifiers
                .iter()
                .all(|s| s.noise.is_none())
        );
    }
}
