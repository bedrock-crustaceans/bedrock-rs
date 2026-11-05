//! The engine's own feature inputs of 1.26.60 (copied under `fixtures/worldgen/feature/engine/`; the three feature-rule
//! documents among them are the feature-rules tests'). Most are payload bodies of one feature
//! type, put here in a document of the format whose shape they have, with a description; the
//! library they come from is stripped, so no file names its test. Each expectation is the
//! game's load path as the family reads it (the builders are
//! unchanged and the crate's values stand): a body that loads has no
//! warning or error and round-trips (value equality, a stable second write); a refused one
//! names its code, severity and path. `drop4.rs` has the snap bodies 2, 6, 10, 24, 30 and 34.
//!
//! Feature documents the game's own unit tests and server tests feed its loader.

use bedrock_addon_core::diagnostic::{Diagnostics, Severity};
use bedrock_addon_core::load::LoadContext;

use super::common::{base_game, cx, fixture, load, loads, rejects};

/// An engine input by file name.
fn engine(name: &str) -> String {
    fixture(&format!("engine/{name}"))
}

/// The payload body `name` as the feature of type `key` at `version`, named `test:engine`.
fn body(name: &str, key: &str, version: &str) -> String {
    let text = engine(name);
    let inner = text
        .trim()
        .strip_prefix('{')
        .and_then(|b| b.strip_suffix('}'))
        .expect("an object");
    format!(
        r#"{{"format_version": "{version}", "{key}": {{"description": {{"identifier": "test:engine"}}, {inner}}}}}"#
    )
}

const SEARCH: &str = "minecraft:search_feature";
const SCATTER: &str = "minecraft:scatter_feature";

/// Loads, has no warning or error, writes, reads back equal, writes the same text again.
#[track_caller]
fn round_trips(name: &str, text: &str, cx: &LoadContext) {
    let first = loads(text, cx);
    assert!(
        first
            .diagnostics()
            .iter()
            .all(|d| d.severity < Severity::Warning),
        "{name}: {:#?}",
        first.diagnostics()
    );
    let written = first.to_json().expect("written");
    let second = loads(&written, cx);
    assert_eq!(first.value(), second.value(), "{name}: {written}");
    assert_eq!(second.to_json().unwrap(), written, "{name}");
}

/// The findings at warning level or above: `(code name, severity, path)`.
fn found(d: &Diagnostics) -> Vec<(String, Severity, String)> {
    d.iter()
        .filter(|f| f.severity >= Severity::Warning)
        .map(|f| {
            let code = format!("{:?}", f.code);
            let name = code
                .split([' ', '{', '('])
                .next()
                .unwrap_or_default()
                .to_owned();
            (name, f.severity, f.location.json_path.to_string())
        })
        .collect()
}

/// The bodies and documents that load for any pack, and round-trip.
#[test]
fn the_engine_s_features_load_and_round_trip() {
    for (name, text) in [
        ("case-7.json", engine("case-7.json")),
        ("case-14.json", engine("case-14.json")),
        ("case-49.json", engine("case-49.json")),
        ("case-9.json", body("case-9.json", SEARCH, "1.21.40")),
        ("case-33.json", body("case-33.json", SEARCH, "1.21.40")),
        ("case-29.json", body("case-29.json", SCATTER, "1.13.0")),
        ("case-48.json", body("case-48.json", SCATTER, "1.13.0")),
        (
            "case-28.json",
            body(
                "case-28.json",
                "minecraft:surface_relative_threshold_feature",
                "1.21.40",
            ),
        ),
        (
            "case-42.json",
            body(
                "case-42.json",
                "minecraft:surface_relative_threshold_feature",
                "1.21.40",
            ),
        ),
        (
            "case-31.json",
            body(
                "case-31.json",
                "minecraft:vegetation_patch_feature",
                "1.21.40",
            ),
        ),
    ]
    .into_iter()
    .chain(
        [
            "case-36.json",
            "case-37.json",
            "case-38.json",
            "case-46.json",
            "case-51.json",
        ]
        .map(|n| {
            (
                n,
                body(n, "minecraft:height_difference_filter_feature", "1.21.40"),
            )
        }),
    ) {
        round_trips(name, &text, &cx());
    }
}

/// `scatter_chance` `{1, 1}` loads with the scatter setter's warning "Bad value for
/// `scatter_chance` - denominator should be greater than the numerator", in the
/// 1.13.0 shape (case-47) as in the 1.21.40 `distribution` object (case-25).
#[test]
fn the_engine_s_even_scatter_chance_loads_with_the_setter_warning() {
    for (name, version, at) in [
        (
            "case-47.json",
            "1.13.0",
            "minecraft:scatter_feature/scatter_chance",
        ),
        (
            "case-25.json",
            "1.21.40",
            "minecraft:scatter_feature/distribution/scatter_chance",
        ),
    ] {
        let document = loads(&body(name, SCATTER, version), &cx());
        assert_eq!(
            found(document.diagnostics()),
            [(
                "ScatterParamValue".to_owned(),
                Severity::Error,
                at.to_owned()
            )],
            "{name}"
        );
        assert_eq!(
            document
                .diagnostics()
                .iter()
                .next()
                .unwrap()
                .code
                .to_string(),
            "Bad value for scatter_chance - denominator should be greater than the numerator"
        );
    }
}

/// The internal types `minecraft:beards_and_shavers` (`BeardAndShaverFeatureTests`, "…`ValidJson`…
/// `ValidationSucceeds`") and `minecraft:scan_surface` load from a base-game pack and round-trip;
/// from any other pack they are refused with "The feature uses the feature type '%s', which
/// is considered internal and thus not compatible with this engine version of the game.".
#[test]
fn the_engine_s_internal_features_load_only_from_a_base_game_pack() {
    for (name, key) in [
        ("beard_shaver-1.json", "minecraft:beards_and_shavers"),
        ("case-32.json", "minecraft:scan_surface"),
    ] {
        let text = body(name, key, "1.21.40");
        round_trips(name, &text, &base_game());
        let d = rejects(&text, &cx());
        assert_eq!(
            found(&d),
            [("MissingToggles".to_owned(), Severity::Error, key.to_owned())],
            "{name}"
        );
    }
}

/// The refused `minecraft:search_feature` bodies: file, text, code and the path below the type key.
fn refused_search_bodies() -> Vec<(&'static str, String, &'static str, &'static str)> {
    let search = |n: &str| body(n, SEARCH, "1.21.40");
    vec![
        (
            "case-1.json",
            search("case-1.json"),
            "InvalidType",
            "search_volume/max/0",
        ),
        (
            "case-22.json",
            search("case-22.json"),
            "InvalidType",
            "search_volume/max/0",
        ),
        (
            "case-50.json",
            search("case-50.json"),
            "InvalidType",
            "search_volume/max/0",
        ),
        (
            "case-5.json",
            search("case-5.json"),
            "InvalidType",
            "search_volume/min",
        ),
        (
            "case-40.json",
            search("case-40.json"),
            "InvalidType",
            "search_volume/max",
        ),
        (
            "case-35.json",
            search("case-35.json"),
            "InvalidType",
            "search_volume",
        ),
        (
            "case-13.json",
            search("case-13.json"),
            "InvalidType",
            "places_feature",
        ),
        (
            "case-27.json",
            search("case-27.json"),
            "InvalidType",
            "places_feature",
        ),
        (
            "case-41.json",
            search("case-41.json"),
            "InvalidType",
            "places_feature",
        ),
        (
            "case-45.json",
            search("case-45.json"),
            "InvalidType",
            "places_feature",
        ),
        (
            "case-23.json",
            search("case-23.json"),
            "InvalidType",
            "required_successes",
        ),
        (
            "case-17.json",
            search("case-17.json"),
            "InvalidValue",
            "required_successes",
        ),
        (
            "case-39.json",
            search("case-39.json"),
            "InvalidValue",
            "search_volume/min",
        ),
        (
            "case-8.json",
            search("case-8.json"),
            "MissingRequiredField",
            "search_volume",
        ),
    ]
}

/// The other refused bodies, in the same shape as [`refused_search_bodies`].
fn refused_other_bodies() -> Vec<(&'static str, String, &'static str, &'static str)> {
    vec![
        (
            "case-21.json",
            body("case-21.json", SCATTER, "1.13.0"),
            "InvalidType",
            "project_input_to_floor",
        ),
        (
            "case-52.json",
            body("case-52.json", SCATTER, "1.13.0"),
            "InvalidType",
            "places_feature",
        ),
        (
            "case-4.json",
            body("case-4.json", SCATTER, "1.21.40"),
            "UnrecognizedField",
            "distribution/extra_field",
        ),
        (
            "case-43.json",
            body(
                "case-43.json",
                "minecraft:height_difference_filter_feature",
                "1.21.40",
            ),
            "MissingRequiredField",
            "search_radius",
        ),
    ]
}

/// The refused bodies: each one's only finding, as `(code, path below the type key)`.
#[test]
fn the_engine_s_refused_bodies_name_their_finding() {
    let mut cases = refused_search_bodies();
    cases.extend(refused_other_bodies());
    for (name, text, code, at) in cases {
        let key = if text.contains(SEARCH) {
            SEARCH
        } else if text.contains(SCATTER) {
            SCATTER
        } else {
            "minecraft:height_difference_filter_feature"
        };
        assert_eq!(
            found(&rejects(&text, &cx())),
            [(code.to_owned(), Severity::Error, format!("{key}/{at}"))],
            "{name}"
        );
    }
    // A `distribution` with `iterations` only: the three coordinates are required.
    let d = rejects(&body("case-26.json", SCATTER, "1.21.40"), &cx());
    assert_eq!(
        found(&d),
        ["x", "y", "z"].map(|c| (
            "MissingRequiredField".to_owned(),
            Severity::Error,
            format!("{SCATTER}/distribution/{c}")
        ))
    );
}

/// The whole documents the loader refuses before any schema: a malformed text (case-12,
/// whose `identifier` has no value), no description identifier (19), an empty one (44), one
/// without a namespace (3), and a type no registry has (`test:custom_feature`, at 1.12.0, 1.13.0
/// and 2.1.0; "Could not find registered feature type that matches name '%s'").
#[test]
fn the_engine_s_refused_documents_name_their_finding() {
    for (name, code, text) in [
        (
            "case-12.json",
            "MalformedInput",
            "[Feature Registry] Passed in contents could not be evaluated as valid JSON in the following file: features/test.json",
        ),
        (
            "case-19.json",
            "MissingRequiredField",
            "[Feature Registry] Missing description identifier for JSON in the following file: features/test.json",
        ),
        (
            "case-44.json",
            "InvalidValue",
            "[Feature Registry] Empty description identifier for JSON in the following file: features/test.json",
        ),
        (
            "case-3.json",
            "NamespaceRule",
            "The feature identifier 'test_feature' should be in the format namespace:id",
        ),
        (
            "main-2.json",
            "UnknownPayloadKey",
            "Could not find registered feature type that matches name 'test:custom_feature'",
        ),
        (
            "case-15.json",
            "UnknownPayloadKey",
            "Could not find registered feature type that matches name 'test:custom_feature'",
        ),
        (
            "case-16.json",
            "UnknownPayloadKey",
            "Could not find registered feature type that matches name 'test:custom_feature'",
        ),
    ] {
        let d = rejects(&engine(name), &cx());
        assert_eq!(
            found(&d),
            [(code.to_owned(), Severity::Error, String::new())],
            "{name}"
        );
        assert_eq!(d.iter().next().unwrap().code.to_string(), text, "{name}");
    }
}

/// Two inputs are only the start of a document whose identifier the test fills in at run time
/// (block_break-1 at 1.13.0, case-20 at 1.21.40):
/// alone each is malformed JSON; completed with an identifier and the three members every
/// `single_block_feature` needs (ours: the rest of the string is not in the binary), each loads.
#[test]
fn the_engine_s_document_starts_complete_to_single_block_features() {
    for name in ["block_break-1.json", "case-20.json"] {
        let start = engine(name);
        let d = rejects(&start, &cx());
        assert_eq!(
            found(&d),
            [("MalformedInput".to_owned(), Severity::Error, String::new())],
            "{name}"
        );
        let whole = format!(
            r#"{start}test:engine"}}, "places_block": "minecraft:stone", "enforce_placement_rules": false, "enforce_survivability_rules": false}}}}"#
        );
        round_trips(name, &whole, &cx());
        assert!(load(&whole, &cx()).is_ok());
    }
}
