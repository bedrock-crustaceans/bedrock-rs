//! The engine's own client-biome inputs of 1.26.60 (copied under
//! `fixtures/biome/biomes_client/engine/`), and the drop-4 ice caves
//! client biome. No file names its test; each
//! expectation is the game's file-load path: a document that loads has no warning or error (the upgrade's notes are information)
//! and round-trips; a refused one names its first error's code, severity and path, the rest
//!
//! Client-biome documents the game's own unit tests feed its loader.

use bedrock_addon::diagnostic::{DiagnosticCode, Diagnostics, Severity};

use super::common::{base_game, context, fixture, json, load, load_beta, loads, rejected, written};

/// An engine input by file name.
fn engine(name: &str) -> String {
    fixture(&format!("engine/{name}"))
}

/// The first error of a refused document has `code` at `at`; every other error is one of the
/// loader's closing lines ("Invalid Client Biome file", "Failed to load client biome
/// document: {}").
#[track_caller]
fn first_error(found: &Diagnostics, code: impl Fn(&DiagnosticCode) -> bool, at: &str) {
    let errors: Vec<_> = found
        .iter()
        .filter(|d| d.severity >= Severity::Warning)
        .collect();
    assert!(!errors.is_empty(), "{found:#?}");
    assert!(code(&errors[0].code), "{found:#?}");
    assert_eq!(errors[0].severity, Severity::Error, "{found:#?}");
    assert_eq!(errors[0].location.json_path.to_string(), at, "{found:#?}");
    assert!(
        errors[1..]
            .iter()
            // By legacy name: the variant and the definitions that were it.
            .all(|d| matches!(d.code.name(), "MalformedInput") && d.severity == Severity::Error),
        "{found:#?}"
    );
}

/// The stable documents load without a warning or an error. A 1.21.130 document writes back
/// what it read; an upgraded one writes its upgraded value, which reads back to the same
/// written value and writes the same text again.
#[test]
fn the_engine_s_client_biomes_load_and_round_trip() {
    for name in [
        "case-4.json",
        "case-5.json",
        "case-6.json",
        "case-8.json",
        "case-9.json",
        "case-14.json",
        "case-16.json",
        "case-17.json",
        "case-18.json",
        "case-19.json",
    ] {
        let text = engine(name);
        let document = loads(&text, &context());
        assert!(
            document
                .diagnostics()
                .iter()
                .all(|d| d.severity < Severity::Warning),
            "{name}: {:#?}",
            document.diagnostics()
        );
        if document.version().upgraded {
            let once = document.to_json().unwrap();
            let again = loads(&once, &context());
            // The same components and description; only the read-back's "synthesized" marks
            // differ, which are not written.
            assert_eq!(again.description, document.description, "{name}");
            assert_eq!(written(&again), json(&once), "{name}");
            assert_eq!(again.to_json().unwrap(), once, "{name}");
        } else {
            assert_eq!(written(&document), json(&text), "{name}");
        }
    }
}

/// The two `"beta"` documents load from a base-game pack without a finding and read back equal.
#[test]
fn the_engine_s_beta_client_biomes_load_from_a_base_game_pack() {
    for name in ["case-1.json", "case-12.json"] {
        let beta = load_beta(&engine(name), &base_game())
            .unwrap_or_else(|r| panic!("{name}: {:#?}", r.diagnostics));
        assert!(
            beta.diagnostics().is_empty(),
            "{name}: {:#?}",
            beta.diagnostics()
        );
        let once = beta.to_json().unwrap();
        let again = load_beta(&once, &base_game()).unwrap();
        assert_eq!(again.value(), beta.value(), "{name}");
        assert_eq!(again.to_json().unwrap(), once, "{name}");
    }
}

/// The refused inputs: an empty `minecraft:cubemap_identifier` (its member is required), a
/// version below the floor 1.21.40, an `{asset, chance}` addition in a 1.21.90 document (the
/// legacy parser reads it with the 1.21.90 struct, whose `addition` is a sound name:
/// the legacy client-biome parser, then the logger's error check),
/// and a `water_appearance` with neither colour nor opacity
/// (the game's constraint: "No surface color or opacity specified. Specify one
/// or remove the component.").
#[test]
fn the_engine_s_refused_client_biomes_name_their_first_error() {
    let at = "minecraft:client_biome/components";
    first_error(
        &rejected(&engine("case-10.json"), &context()),
        // By legacy name: the variant and the definitions that were it.
        |c| matches!(c.name(), "MissingRequiredField"),
        &format!("{at}/minecraft:cubemap_identifier/cubemap_identifier"),
    );
    first_error(
        &rejected(&engine("case-11.json"), &context()),
        // By legacy name: the variant and the definitions that were it.
        |c| matches!(c.name(), "UnsupportedVersion"),
        "format_version",
    );
    for name in ["case-7.json", "case-13.json"] {
        first_error(
            &rejected(&engine(name), &context()),
            // By legacy name: the variant and the definitions that were it.
            |c| matches!(c.name(), "InvalidType"),
            &format!("{at}/minecraft:ambient_sounds/addition"),
        );
    }
    for name in ["case-3.json", "case-15.json"] {
        let found = rejected(&engine(name), &context());
        first_error(
            &found,
            // By legacy name: the variant and the definitions that were it.
            |c| matches!(c.name(), "InvalidValue"),
            &format!("{at}/minecraft:water_appearance"),
        );
        assert_eq!(
            found.iter().next().unwrap().code.to_string(),
            "No surface color or opacity specified. Specify one or remove the component."
        );
    }
}

/// `environment_attributes` is a member of the beta struct only: in the 1.21.130 slice, read
/// with `StrictErrorCodes` (0xfffc), it is an unknown member, an error, and the
/// document is refused.
#[test]
fn the_engine_s_attributes_outside_the_beta_struct_refuse_the_client_biome() {
    let found = load(&engine("case-2.json"), &context()).expect_err("refused");
    first_error(
        &found.diagnostics,
        // By legacy name: the variant and the definitions that were it.
        |c| matches!(c.name(), "UnrecognizedField"),
        "minecraft:client_biome/environment_attributes",
    );
}

/// `experimental_y_2026_drop_4/biomes/ice_caves.client_biome.json` of 1.26.60: format 1.26.0, read by the 1.21.130 struct. It loads from the
/// base-game pack without a finding and writes back what it read.
#[test]
fn the_drop_4_ice_caves_client_biome_loads_and_writes_what_it_read() {
    let text = fixture("drop4/ice_caves.client_biome.json");
    let document = loads(&text, &base_game());
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert!(!document.version().upgraded);
    let source = json(&text);
    assert_eq!(
        written(&document)
            .as_object()
            .unwrap()
            .get("minecraft:client_biome"),
        source.as_object().unwrap().get("minecraft:client_biome")
    );
}
