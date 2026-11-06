//! Unit tests of the fog internals the integration tests cannot name: the chain's parameters,
//! the unreachable rule V14, and the field-gate table against the readers.

use super::{FLOOR, FOG_FIELD_GATES, LATEST, rules, texts};
use crate::history::FogFamily;
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon_core::versioning::{
    EnvelopeCheck, Family, MissingVersion, RangeModel, VersionChain,
};

#[test]
fn the_chain_is_1_16_100_to_1_26_20_closed_at_its_newest_struct() {
    let versions: Vec<String> =
        <<FogFamily as Family>::History as VersionChain<FogFamily>>::VERSIONS
            .iter()
            .map(ToString::to_string)
            .collect();
    assert_eq!(versions, ["1.16.100", "1.21.0", "1.21.90", "1.26.20"]);
    assert_eq!(FogFamily::FLOOR, FLOOR);
    assert_eq!(FogFamily::RANGE_MODEL, RangeModel::ClosedToLatest);
    assert!(matches!(
        FogFamily::ENVELOPE.missing,
        MissingVersion::Reject { .. }
    ));
    assert_eq!(FogFamily::ENVELOPE.check, EnvelopeCheck::From(LATEST));
    assert_eq!(FogFamily::PAYLOAD_KEY, "minecraft:fog_settings");
    assert_eq!(super::LOADER.walk.directories, ["fogs"]);
    let has_beta = <<FogFamily as Family>::History as VersionChain<FogFamily>>::HAS_BETA;
    assert_eq!(has_beta, std::hint::black_box(false));
}

/// `invalid_format_version_rule`: V14 cannot be reached from a file, whose
/// version was read before the cereal parser ran.
#[test]
fn invalid_format_version_rule() {
    let mut d = Diagnostics::default();
    rules::format_version("not a version", &JsonPath::new(), &mut d);
    let finding = d.iter().next().expect("V14 reports");
    assert_eq!(finding.severity, Severity::Error);
    assert_eq!(finding.code.name(), "InvalidValue");
    assert_eq!(finding.code.to_string(), texts::BAD_FORMAT_VERSION);
    assert_eq!(finding.location.json_path.to_string(), "format_version");

    let mut d = Diagnostics::default();
    rules::format_version("1.26.20", &JsonPath::new(), &mut d);
    assert!(d.is_empty());
}

#[test]
fn the_gate_table_names_the_chain_s_versions_and_the_keyframe_engine() {
    let versions = <<FogFamily as Family>::History as VersionChain<FogFamily>>::VERSIONS;
    for gate in FOG_FIELD_GATES {
        assert!(versions.contains(&gate.released), "{}", gate.path);
        if let Some(removed) = gate.removed {
            assert!(versions.contains(&removed), "{}", gate.path);
            assert!(removed > gate.released, "{}", gate.path);
        }
        if let Some(engine) = gate.since_engine {
            // The keyframe rows are the engine-gated ones, and the gate the rules apply.
            assert_eq!(engine, rules::KEYFRAMES_SINCE, "{}", gate.path);
            assert_eq!(gate.released, LATEST, "{}", gate.path);
        }
    }
    let gated = |path: &str| FOG_FIELD_GATES.iter().find(|g| g.path == path).unwrap();
    assert_eq!(
        gated("volumetric.density.weather").released.to_string(),
        "1.21.0"
    );
    assert_eq!(
        gated("volumetric.henyey_greenstein_g.*")
            .released
            .to_string(),
        "1.21.90"
    );
    assert_eq!(gated("distance.*.transition_fog").removed, Some(LATEST));
    assert_eq!(
        FOG_FIELD_GATES
            .iter()
            .filter(|g| g.since_engine.is_some())
            .count(),
        6
    );
}

/// A file of an older format that breaks V15 is refused, as the game refuses it: the strict
/// schema's typed nodes carry V15 and V17–V20 at level 3 and fail the validation, and
/// `registerFogFromJson` logs "Failed to validate fog schema" and registers nothing
/// (the file is refused). A file within the rule loads and
/// is written at 1.26.20.
#[test]
fn an_older_file_that_breaks_a_rule_of_the_schema_is_refused() {
    use bedrock_addon_core::document::Content;
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::pack_identity::PackPath;
    let fog = |start: &str| {
        format!(
            r#"{{"format_version": "1.16.100", "minecraft:fog_settings": {{"description": {{"identifier": "demo:x"}}, "distance": {{"air": {{"fog_start": {start}, "fog_end": 1, "fog_color": [1, 1, 1], "render_distance_type": "fixed"}}}}}}}}"#
        )
    };
    let path = PackPath::from_static("fogs/x.json");
    let cx = LoadContext::detached();
    // Within the rule: nothing to tell, and written at the newest format.
    let document = super::FogSettings::parse(&path, fog("1").as_bytes(), &cx).expect("loads");
    assert!(document.diagnostics().is_empty());
    let written = document.to_json().expect("written");
    assert!(
        written.contains(r#""format_version": "1.26.20""#),
        "{written}"
    );
    // Above `fog_end`: refused at the entry, with the game's closing line.
    let rejected = super::FogSettings::parse(&path, fog("7").as_bytes(), &cx)
        .map(|_| ())
        .expect_err("refused");
    let told: Vec<_> = rejected
        .diagnostics
        .iter()
        .map(|f| {
            format!(
                "{} {:?} {:?} at {}",
                f.code.name(),
                f.severity,
                f.phase,
                f.location.json_path
            )
        })
        .collect();
    assert_eq!(
        told,
        [
            "InvalidValue Error Validate at minecraft:fog_settings/distance/air",
            "DocumentRejected Error Parse at ",
        ]
    );
}

/// Settings that break a rule have no text any era reads: the older schemas hold the same
/// rules. The writer refuses them at the entry that breaks the rule, whether or
/// not an older schema holds their members (the game does not load such a file).
#[test]
fn settings_that_break_a_rule_of_the_schema_are_not_written() {
    use super::{
        DistanceType, FogColor, FogDistanceCollection, FogDistanceSetting, FogSettings,
        FogVolumetricCollection, FogVolumetricDensityCollection, FogVolumetricDensitySetting,
        Keyframed,
    };
    use bedrock_addon_core::document::Content;
    use bedrock_addon_core::json::EncodeError;
    let backwards = || FogDistanceSetting {
        fog_start: Some(7.0),
        fog_end: Some(1.0),
        ..FogDistanceSetting::default()
    };
    let refused_at = |settings: &FogSettings| match settings.to_json() {
        Err(EncodeError::NotRepresentable { at, .. }) => at.to_string(),
        other => panic!("{other:?}"),
    };
    let with_air = |air: FogDistanceSetting| {
        let mut settings = FogSettings::new(bedrock_addon_core::id::FogId::parse("demo:x"));
        settings.distance = Some(FogDistanceCollection {
            air: Some(air),
            ..FogDistanceCollection::default()
        });
        settings
    };
    // Two of the four members an older schema requires, and all four.
    assert_eq!(
        refused_at(&with_air(backwards())),
        "minecraft:fog_settings/distance/air"
    );
    let mut whole = backwards();
    whole.render_distance_type = Some(DistanceType::Fixed);
    whole.fog_color = Some(FogColor::Array([1, 1, 1]));
    assert_eq!(
        refused_at(&with_air(whole)),
        "minecraft:fog_settings/distance/air"
    );
    // A keyframed density without its heights.
    let mut keyed = FogSettings::new(bedrock_addon_core::id::FogId::parse("demo:x"));
    keyed.volumetric = Some(FogVolumetricCollection {
        density: Some(FogVolumetricDensityCollection {
            air: Some(FogVolumetricDensitySetting {
                max_density: Some(Keyframed::from_pairs([(0.0, 0.1), (0.5, 0.2)])),
                ..FogVolumetricDensitySetting::default()
            }),
            ..FogVolumetricDensityCollection::default()
        }),
        ..FogVolumetricCollection::default()
    });
    assert_eq!(
        refused_at(&keyed),
        "minecraft:fog_settings/volumetric/density/air"
    );
}

/// Every finding of `found` is in the app-data pack's file `path`.
fn assert_located(found: &Diagnostics, path: &'static str) {
    use bedrock_addon_core::pack_identity::{PackPath, PackRef};
    assert!(!found.is_empty(), "no finding to place");
    for finding in found {
        assert_eq!(
            finding.location.pack,
            Some(PackRef::app_data()),
            "{finding:?}"
        );
        assert_eq!(
            finding.location.path,
            Some(PackPath::from_static(path)),
            "{finding:?}"
        );
    }
}

#[test]
fn findings_carry_the_file_and_pack() {
    use bedrock_addon_core::document::Content;
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::pack_identity::PackPath;
    const PATH: &str = "fogs/x.json";
    let path = PackPath::from_static(PATH);
    let cx = LoadContext::detached();
    let fog = |version: &str, start: &str| {
        format!(
            r#"{{"format_version": "{version}", "minecraft:fog_settings": {{"description": {{"identifier": "demo:x"}}, "distance": {{"air": {{"fog_start": {start}, "fog_end": 1, "fog_color": [1, 1, 1], "render_distance_type": "fixed"}}}}}}}}"#
        )
    };
    // A loaded file with an information: a pre-release of the newest version.
    let document = super::FogSettings::parse(&path, fog("1.26.20-beta.1", "1").as_bytes(), &cx)
        .expect("loads");
    assert_located(document.diagnostics(), PATH);
    // Refused by a rule after the parse, and by the gates.
    for bytes in [fog("1.16.100", "7").into_bytes(), b"{".to_vec()] {
        let rejected = super::FogSettings::parse(&path, &bytes, &cx)
            .map(|_| ())
            .expect_err("refused");
        assert_located(&rejected.diagnostics, PATH);
    }
}

/// The gates check the nesting before the encoding: a text that is both too deep and not
/// UTF-8 is told as too deep.
#[test]
fn a_text_too_deep_and_not_utf8_is_told_as_too_deep() {
    use bedrock_addon_core::diagnostic::DiagnosticCode;
    use bedrock_addon_core::document::Content;
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::pack_identity::PackPath;
    let mut bytes = vec![b'['; bedrock_addon_core::json::MAX_JSON_DEPTH + 1];
    bytes.push(0xff);
    let rejected = super::FogSettings::parse(
        &PackPath::from_static("fogs/x.json"),
        &bytes,
        &LoadContext::detached(),
    )
    .map(|_| ())
    .expect_err("refused");
    let messages: Vec<_> = rejected
        .diagnostics
        .iter()
        .filter_map(|f| match &f.code {
            DiagnosticCode::MalformedInput { message, .. } => Some(message.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(messages, ["nesting too deep"]);
}

/// Behind a byte-order mark too: the mark is blanked before the nesting is scanned.
#[test]
fn a_text_too_deep_and_not_utf8_behind_a_bom_is_told_as_too_deep() {
    use bedrock_addon_core::diagnostic::DiagnosticCode;
    use bedrock_addon_core::document::Content;
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::pack_identity::PackPath;
    let mut bytes = b"\xEF\xBB\xBF".to_vec();
    bytes.extend(vec![b'['; bedrock_addon_core::json::MAX_JSON_DEPTH + 1]);
    bytes.push(0xff);
    let rejected = super::FogSettings::parse(
        &PackPath::from_static("fogs/x.json"),
        &bytes,
        &LoadContext::detached(),
    )
    .map(|_| ())
    .expect_err("refused");
    let messages: Vec<_> = rejected
        .diagnostics
        .iter()
        .filter_map(|f| match &f.code {
            DiagnosticCode::MalformedInput { message, .. } => Some(message.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(messages, ["nesting too deep"]);
}
