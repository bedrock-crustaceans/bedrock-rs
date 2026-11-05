//! The version chain: contiguity, struct selection, the
//! floor and the ceiling, beta documents, the in-payload version.

use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity, VanillaText};
use bedrock_addon_core::document::JsonDocument;
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    Loaded, Selection, VersionChain, check_chain, parse_as_written, select,
};
use bedrock_addon_family_entity::{Entity, history};

use super::common::{codes, cx, doc, outcome, with_code};

const MINIMAL: &str = r#"{ "description": { "identifier": "demo:x" }, "components": {} }"#;

fn v(text: &str) -> FormatVersion {
    FormatVersionInput::parse(text).unwrap().version.unwrap()
}

#[test]
fn the_chain_has_42_contiguous_structs_from_the_floor() {
    check_chain::<Entity>().expect("a well-formed chain");
    let versions = <history::Any as VersionChain<Entity>>::VERSIONS;
    let expected = [
        "1.8.0", "1.10.0", "1.11.0", "1.12.0", "1.13.0", "1.15.0", "1.16.0", "1.16.100",
        "1.16.200", "1.17.10", "1.17.20", "1.18.0", "1.18.10", "1.18.20", "1.18.30", "1.19.0",
        "1.19.50", "1.19.60", "1.20.0", "1.20.60", "1.20.70", "1.20.80", "1.21.0", "1.21.10",
        "1.21.20", "1.21.40", "1.21.60", "1.21.70", "1.21.80", "1.21.89", "1.21.90", "1.21.100",
        "1.21.110", "1.21.120", "1.21.130", "1.26.0", "1.26.10", "1.26.20", "1.26.30", "1.26.40",
        "1.26.50", "1.26.60",
    ];
    assert_eq!(versions.len(), 42);
    assert_eq!(versions, expected.map(v).as_slice());
    const { assert!(<history::Any as VersionChain<Entity>>::HAS_BETA) };
}

#[test]
fn a_version_selects_the_struct_whose_range_holds_it() {
    let pick = |text: &str| select::<Entity>(&FormatVersionInput::parse(text).unwrap());
    let versions = <history::Any as VersionChain<Entity>>::VERSIONS;
    let index = |text: &str| versions.iter().position(|x| *x == v(text)).unwrap();
    assert_eq!(pick("1.8.0"), Selection::Struct(index("1.8.0")));
    assert_eq!(pick("1.9.5"), Selection::Struct(index("1.8.0")));
    assert_eq!(pick("1.14.0"), Selection::Struct(index("1.13.0")));
    assert_eq!(pick("1.21.85"), Selection::Struct(index("1.21.80")));
    assert_eq!(pick("1.21.89"), Selection::Struct(index("1.21.89")));
    assert_eq!(pick("1.26.45"), Selection::Struct(index("1.26.40")));
    assert_eq!(pick("1.26.50"), Selection::Struct(index("1.26.50")));
    assert_eq!(pick("1.7.9"), Selection::BelowFloor);
    assert_eq!(pick("1.26.55"), Selection::Struct(index("1.26.50")));
    assert_eq!(pick("1.26.60"), Selection::Struct(index("1.26.60")));
    assert_eq!(pick("1.26.61"), Selection::Unsupported);
}

#[test]
fn parse_as_written_keeps_the_struct_of_the_file() {
    let text = doc("1.16.0", MINIMAL);
    let parsed = parse_as_written::<Entity>(JsonSlice::new(&text), &cx());
    assert!(matches!(parsed.value, Some(history::Any::V1_16_0(_))));
    assert_eq!(parsed.version.parsed_as, v("1.16.0"));
}

#[test]
fn a_floor_document_reaches_the_newest_struct() {
    let outcome = outcome(&doc("1.8.0", MINIMAL), &cx());
    assert!(matches!(outcome.value, Some(Loaded::Stable(_))));
    assert!(outcome.version.upgraded);
    assert_eq!(outcome.version.parsed_as, v("1.8.0"));
    assert_eq!(outcome.original_version, v("1.8.0"));
}

#[test]
fn below_the_floor_is_unsupported_with_the_floor_text() {
    let outcome = outcome(&doc("1.7.0", MINIMAL), &cx());
    assert!(outcome.value.is_none());
    let found = with_code(&outcome.diagnostics, "UnsupportedVersion");
    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].code,
        DiagnosticCode::UnsupportedVersion {
            text: Some(VanillaText::UseFormatVersion {
                what: "entity",
                name: "demo:x".to_owned(),
                version: v("1.8.0"),
            })
        }
    );
}

#[test]
fn above_the_engine_is_unsupported_with_vanillas_resource_pack_text() {
    let document = super::common::parse(&doc("1.26.61", MINIMAL)).unwrap_err();
    let found = with_code(&document.diagnostics, "UnsupportedVersion");
    assert_eq!(
        found[0].code,
        DiagnosticCode::UnsupportedVersion {
            text: Some(VanillaText::ActorAboveEngine {
                version: "1.26.61".to_owned(),
                engine: v("1.26.60"),
            })
        }
    );
}

#[test]
fn a_missing_version_is_an_information_and_parses_at_the_floor_deviation() {
    // vanilla reads it as 0.0.0 silently (`vanilla = "0.0.0 silent"`): the text
    // is the crate's own, so the finding is an information.
    let text = format!(r#"{{"minecraft:entity": {MINIMAL}}}"#);
    let outcome = outcome(&text, &cx());
    assert!(matches!(outcome.value, Some(Loaded::Stable(_))));
    assert_eq!(outcome.version.parsed_as, v("1.8.0"));
    let found = with_code(&outcome.diagnostics, "MissingFormatVersion");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].severity, Severity::Info);
    assert_eq!(
        found[0].code,
        DiagnosticCode::MissingFormatVersion {
            text: Some(VanillaText::NoFormatVersionField)
        }
    );
}

#[test]
fn the_in_payload_version_stands_in_for_a_missing_root_one() {
    // `v00_floor_envelope`: a 1.2-era file gets the floor text, not the
    // missing-version warning.
    let text = r#"{"minecraft:entity": {"format_version": "1.2.0", "description": {"identifier": "demo:old"}, "components": {}}}"#;
    let outcome = outcome(text, &cx());
    assert!(outcome.value.is_none());
    assert_eq!(
        codes(&outcome.diagnostics, Severity::Info),
        ["UnsupportedVersion"]
    );
    // An in-payload version the chain covers is read and not written back.
    let text = r#"{"minecraft:entity": {"format_version": "1.26.50", "description": {"identifier": "demo:new"}, "components": {}}}"#;
    let outcome = super::common::outcome(text, &cx());
    assert_eq!(
        codes(&outcome.diagnostics, Severity::Info),
        Vec::<&str>::new()
    );
    let Some(Loaded::Stable(entity)) = outcome.value else {
        panic!("{:?}", outcome.diagnostics)
    };
    assert!(entity.extra.get("format_version").is_none());
}

#[test]
fn a_beta_document_needs_the_beta_apis_toggle() {
    let text = doc("beta", MINIMAL);
    let refused = outcome(&text, &cx());
    assert!(refused.value.is_none());
    assert_eq!(
        codes(&refused.diagnostics, Severity::Error),
        ["BetaFeaturesNotEnabled"]
    );
    let mut beta = cx();
    beta.beta_apis = true;
    let loaded = outcome(&text, &beta);
    let Some(Loaded::Beta(entity)) = loaded.value else {
        panic!("{:?}", loaded.diagnostics)
    };
    assert_eq!(entity.description.identifier.as_str(), "demo:x");
    let mut base_game = cx();
    base_game.is_base_game_pack = true;
    assert!(matches!(
        outcome(&text, &base_game).value,
        Some(Loaded::Beta(_))
    ));
}

#[test]
fn a_prerelease_version_selects_the_struct_below_its_release() {
    let outcome = outcome(&doc("1.21.90-beta.3", MINIMAL), &cx());
    assert!(matches!(outcome.value, Some(Loaded::Stable(_))));
    assert_eq!(outcome.version.parsed_as, v("1.21.89"));
    assert_eq!(
        codes(&outcome.diagnostics, Severity::Info),
        ["PrereleaseFormatVersion"]
    );
}

#[test]
fn every_struct_round_trips_its_own_version() {
    for version in <history::Any as VersionChain<Entity>>::VERSIONS {
        let text = doc(&version.to_string(), MINIMAL);
        let parsed = parse_as_written::<Entity>(JsonSlice::new(&text), &cx());
        let any = parsed
            .value
            .unwrap_or_else(|| panic!("{version}: {:?}", parsed.diagnostics));
        assert_eq!(any.parsed_version(), *version);
        let written = any.to_json().unwrap();
        let again = parse_as_written::<Entity>(JsonSlice::new(&written), &cx());
        assert_eq!(again.value.as_ref(), Some(&any), "{version}: {written}");
    }
}

#[test]
fn parse_as_written_then_upgrade_equals_parse() {
    use bedrock_addon_core::document::Content;
    let path = bedrock_addon_core::pack_identity::PackPath::new("entities/x.json").unwrap();
    for version in ["1.12.0", "1.20.0", "1.26.50"] {
        let text = doc(version, MINIMAL);
        let written = Entity::parse_as_written(&path, text.as_bytes(), &cx()).unwrap();
        assert_eq!(written.parsed_version(), v(version));
        let upgraded = Entity::upgrade(written, &cx()).unwrap();
        let direct = Entity::parse(&path, text.as_bytes(), &cx()).unwrap();
        assert_eq!(upgraded.value(), direct.value(), "{version}");
        assert_eq!(upgraded.version(), direct.version(), "{version}");
        assert_eq!(upgraded.diagnostics(), direct.diagnostics(), "{version}");
    }
}

/// 1.26.50 is a slice of its own below the newest, 1.26.60
/// (the `upgrade_v1_26_60` step); a 1.26.50 document is that slice's struct and upgrades to
/// the newest, a 1.26.60 document is the newest itself.
#[test]
fn a_1_26_50_document_is_its_own_slice_below_the_newest_1_26_60() {
    let text = doc("1.26.50", MINIMAL);
    let parsed = parse_as_written::<Entity>(JsonSlice::new(&text), &cx());
    assert!(matches!(parsed.value, Some(history::Any::V1_26_50(_))));
    assert_eq!(parsed.version.parsed_as, v("1.26.50"));
    let text = doc("1.26.60", MINIMAL);
    let parsed = parse_as_written::<Entity>(JsonSlice::new(&text), &cx());
    assert!(matches!(parsed.value, Some(history::Any::V1_26_60(_))));
    assert_eq!(parsed.version.parsed_as, v("1.26.60"));
    for version in ["1.26.50", "1.26.60"] {
        let loaded = outcome(&doc(version, MINIMAL), &cx());
        assert!(
            matches!(loaded.value, Some(Loaded::Stable(_))),
            "{version}: {:?}",
            loaded.diagnostics
        );
    }
}
