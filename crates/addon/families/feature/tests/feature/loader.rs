//! The loader stage and the document rules every type shares: versions, identifiers, payload
//! keys, internal types (fixtures f07, f08, f11, f12; validation rules 1–3, 5–8, 10).

use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity, VanillaText};
use bedrock_addon_core::document::Content;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_feature::Feature;

use super::common::{base_game, cx, doc, engine, error_at, has_text, loads, rejects};

const ORE: &str = r#""count": 4"#;

#[test]
fn f07_any_type_at_1_12_0_is_unsupported() {
    for (key, members) in [
        ("ore_feature", ORE),
        ("aggregate_feature", r#""features": ["demo:y"]"#),
        (
            "scatter_feature",
            r#""places_feature": "demo:y", "iterations": 1, "x": 0, "y": 0, "z": 0"#,
        ),
    ] {
        let d = rejects(&doc("1.12.0", key, members), &cx());
        let finding = d
            .iter()
            .find(|f| f.severity == Severity::Error)
            .expect("an error");
        assert!(
            matches!(
                &finding.code,
                DiagnosticCode::UnsupportedVersion {
                    text: Some(VanillaText::NoParserForVersion { version })
                } if version == "1.12.0"
            ),
            "{d:#?}"
        );
        assert_eq!(finding.location.json_path.to_string(), "format_version");
    }
}

#[test]
fn f08_an_internal_type_needs_a_base_game_pack() {
    let json = doc(
        "1.13.0",
        "rect_layout",
        r#""feature_areas": [{"feature": "demo:y", "area_dimensions": [4, 4]}]"#,
    );
    let d = rejects(&json, &cx());
    error_at(&d, "MissingToggles", "minecraft:rect_layout");
    assert!(has_text(
        &d,
        "The feature uses the feature type 'minecraft:rect_layout', which is considered internal and thus not compatible with this engine version of the game."
    ));
    let feature = loads(&json, &base_game());
    assert!(matches!(feature.value(), Feature::RectLayout(_)));
}

/// Loads `bytes` as a feature file; its findings when rejected.
fn reject_bytes(bytes: &[u8]) -> bedrock_addon_core::diagnostic::Diagnostics {
    match Feature::parse(&PackPath::from_static("features/x.json"), bytes, &cx()) {
        Ok(document) => panic!("loaded: {:?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

#[test]
fn f11_the_four_loader_errors() {
    let d = reject_bytes(b"{ not json");
    assert!(matches!(
        d.iter().next().unwrap().code,
        DiagnosticCode::MalformedInput { .. }
    ));
    assert!(has_text(
        &d,
        "[Feature Registry] Passed in contents could not be evaluated as valid JSON in the following file: features/x.json"
    ));

    let d = reject_bytes(br#"{"format_version": "1.13.0", "minecraft:ore_feature": {"description": {}, "count": 1}}"#);
    error_at(&d, "MissingRequiredField", "");
    assert!(has_text(
        &d,
        "[Feature Registry] Missing description identifier for JSON in the following file: features/x.json"
    ));

    let d = reject_bytes(br#"{"format_version": "1.13.0", "minecraft:ore_feature": {"description": {"identifier": ""}, "count": 1}}"#);
    error_at(&d, "InvalidValue", "");
    assert!(has_text(
        &d,
        "[Feature Registry] Empty description identifier for JSON in the following file: features/x.json"
    ));

    let d = reject_bytes(br#"{"format_version": "1.13.0", "minecraft:ore_feature": {"description": {"identifier": "foo"}, "count": 1}}"#);
    error_at(&d, "NamespaceRule", "");
    assert!(has_text(
        &d,
        "The feature identifier 'foo' should be in the format namespace:id"
    ));
}

#[test]
fn f12_a_missing_format_version_rejects_the_file() {
    let json =
        r#"{"minecraft:ore_feature": {"description": {"identifier": "demo:x"}, "count": 1}}"#;
    let d = rejects(json, &cx());
    let finding = d.iter().find(|f| f.severity == Severity::Error).unwrap();
    assert!(
        finding
            .code
            .is(&bedrock_addon_core::diagnostic::codes::NO_VALID_VERSION)
    );
}

#[test]
fn rule_10_an_unregistered_payload_key_is_unknown() {
    let d = rejects(&doc("1.13.0", "nope_feature", ""), &cx());
    let finding = d.iter().find(|f| f.severity == Severity::Error).unwrap();
    assert!(
        matches!(&finding.code, DiagnosticCode::UnknownPayloadKey { name } if name == "minecraft:nope_feature")
    );
    assert!(has_text(
        &d,
        "Could not find registered feature type that matches name 'minecraft:nope_feature'"
    ));
}

#[test]
fn a_type_newer_than_the_engine_is_an_unknown_payload_key() {
    let json = doc(
        "1.13.0",
        "height_difference_filter_feature",
        r#""places_feature": "demo:y", "search_radius": 2, "max_allowed_upward_height_diff": 3"#,
    );
    let d = rejects(&json, &engine(1, 26, 20));
    assert!(matches!(
        d.iter().next().unwrap().code,
        DiagnosticCode::UnknownPayloadKey { .. }
    ));
    loads(&json, &engine(1, 26, 30));
}

#[test]
fn an_unknown_member_anywhere_rejects_the_file() {
    let d = rejects(
        &doc(
            "1.13.0",
            "ore_feature",
            r#""count": 1, "places_block": "minecraft:stone""#,
        ),
        &cx(),
    );
    error_at(
        &d,
        "UnrecognizedField",
        "minecraft:ore_feature/places_block",
    );
    let d = rejects(
        r#"{"format_version": "1.13.0", "minecraft:ore_feature": {"description": {"identifier": "demo:x", "extra": 1}, "count": 1}}"#,
        &cx(),
    );
    error_at(&d, "UnrecognizedField", "description/extra");
}

#[test]
fn a_wrong_type_rejects_the_file() {
    let d = rejects(&doc("1.13.0", "ore_feature", r#""count": "4""#), &cx());
    error_at(&d, "InvalidType", "count");
}

#[test]
fn an_integral_real_is_an_integer_as_jsoncpp_reads_it() {
    // The vanilla geode writes `"crack_point_offset": 2.0`.
    loads(&doc("1.13.0", "ore_feature", r#""count": 4.0"#), &cx());
    let d = rejects(&doc("1.13.0", "ore_feature", r#""count": 4.5"#), &cx());
    error_at(&d, "InvalidType", "count");
}

#[test]
fn an_engine_gated_member_newer_than_the_packs_min_engine_version_is_warned() {
    use bedrock_addon_core::version::{FormatVersion, MinEngineVersion};
    let json = doc(
        "1.13.0",
        "snap_to_surface_feature",
        r#""feature_to_snap": "demo:y", "vertical_search_range": 4, "embed_in_surface": true"#,
    );
    let mut old_pack = cx();
    old_pack.min_engine_version = Some(MinEngineVersion::new(1, 26, 10));
    let document = loads(&json, &old_pack);
    assert!(document.diagnostics().iter().any(|f| {
        f.severity == Severity::Info
            && f.code
                == DiagnosticCode::from(
                    bedrock_addon_core::diagnostic::codes::FIELD_SINCE_ENGINE
                        .with([FormatVersion::new(1, 26, 20).to_string()]),
                )
            && f.location
                .json_path
                .to_string()
                .ends_with("embed_in_surface")
    }));
    assert!(loads(&json, &cx()).diagnostics().is_empty());
}
