//! The chain: contiguity, the range model's selection, floor, ceiling and beta
//! history without upgrade; and the deviations from the game,
//! each pinned by a test named `deviation`.

use bedrock_addon::diagnostic::{DiagnosticCode, Severity, VanillaText};
use bedrock_addon::document::{Content, JsonDocument as _};
use bedrock_addon::families::biome::Biome;
use bedrock_addon::families::biome::history::{self, BiomeFamily, BiomeHistory};
use bedrock_addon::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput};
use bedrock_addon::versioning::{
    Family, MissingVersion, RangeModel, Selection, VersionChain, check_chain, select,
};

use super::common::{
    base_game, biome, context, document, fixture, has, load_beta, loads, path, rejected, with,
};
use super::rows::tags;

/// The chain's versions, in order.
const VERSIONS: [(u16, u16, u16); 4] = [(1, 21, 110), (1, 21, 120), (1, 26, 0), (1, 26, 30)];

/// What `version` selects.
fn selects(version: &str) -> Selection {
    select::<BiomeFamily>(&FormatVersionInput::parse(version).expect("a version"))
}

#[test]
fn the_chain_is_contiguous_and_ends_at_1_26_30_with_a_beta_struct() {
    const HAS_BETA: bool = <BiomeHistory as VersionChain<BiomeFamily>>::HAS_BETA;

    check_chain::<BiomeFamily>().expect("the chain is well formed");
    let versions: Vec<_> = VERSIONS
        .iter()
        .map(|&(a, b, c)| FormatVersion::new(a, b, c))
        .collect();
    assert_eq!(
        <BiomeHistory as VersionChain<BiomeFamily>>::VERSIONS,
        versions.as_slice()
    );
    let has_beta = HAS_BETA;
    assert!(has_beta);
    assert_eq!(
        <BiomeFamily as Family>::RANGE_MODEL,
        RangeModel::ClosedToEngine
    );
    assert_eq!(
        <BiomeFamily as Family>::FLOOR,
        FormatVersion::new(1, 21, 110)
    );
}

#[test]
fn each_version_selects_the_struct_whose_range_holds_it() {
    for (index, &(a, b, c)) in VERSIONS.iter().enumerate() {
        assert_eq!(selects(&format!("{a}.{b}.{c}")), Selection::Struct(index));
    }
    assert_eq!(selects("1.21.115"), Selection::Struct(0));
    assert_eq!(selects("1.21.130"), Selection::Struct(1));
    assert_eq!(selects("1.26.10"), Selection::Struct(2));
    // vanilla's own 1.26.50 files parse as 1.26.30, up to the engine.
    assert_eq!(selects("1.26.50"), Selection::Struct(3));
    assert_eq!(selects(&ENGINE_VERSION.to_string()), Selection::Struct(3));
    assert_eq!(selects("1.26.61"), Selection::Unsupported);
    assert_eq!(selects("1.21.100"), Selection::BelowFloor);
    // A pre-release of the floor is below it.
    assert_eq!(selects("1.21.110-beta.1"), Selection::BelowFloor);
}

#[test]
fn above_the_engine_there_is_no_parser() {
    let found = rejected(&biome("1.26.61", "test:x", ""), &context());
    assert!(has(
        &found,
        |c| matches!(
            c,
            DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::NoParserForVersion { .. })
            }
        ),
        Severity::Error,
        "unsupported version: no parser available for version 1.26.61"
    ));
}

#[test]
fn a_prerelease_loads_through_the_struct_below_it_with_an_info() {
    let document = loads(&biome("1.26.0-beta.2", "test:x", ""), &context());
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 21, 120));
    assert!(document.diagnostics().iter().any(|f| matches!(
        f.code,
        DiagnosticCode::PrereleaseFormatVersion { .. }
    ) && f.severity == Severity::Info));
}

// `1.21.120-beta.1` sorts below 1.21.120, so the legacy struct
// reads it from the whole text, as it reads every document below the envelope minimum.
#[test]
fn a_prerelease_of_1_21_120_is_read_from_the_raw_text_by_the_legacy_struct() {
    let three = |version: &str| {
        format!(
            r#"{{"format_version": "{version}", "minecraft:biome": {}, "x": 1}}"#,
            super::common::payload("test:x", "")
        )
    };
    let document = loads(&biome("1.21.120-beta.1", "test:x", ""), &context());
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 21, 110));
    assert_eq!(document.value().description.identifier.as_str(), "test:x");
    // There the third root member is no envelope finding but an unknown member of the legacy
    // struct, which its strict loader refuses (0xfffc).
    let d = rejected(&three("1.21.120-beta.1"), &context());
    assert!(
        d.iter().any(
            |f| matches!(f.code, DiagnosticCode::UnrecognizedField { .. })
                && f.severity == Severity::Error
                && f.location.json_path.to_string() == "x"
        ),
        "{d:#?}"
    );
    assert!(
        !d.iter()
            .any(|f| matches!(f.code, DiagnosticCode::MalformedInput { .. }))
    );
    // The release itself is the checked envelope: a third root member rejects the document.
    let d = rejected(&three("1.21.120"), &context());
    assert!(
        d.iter()
            .any(|f| matches!(f.code, DiagnosticCode::MalformedInput { .. }))
    );
}

#[test]
fn every_struct_writes_its_own_version_and_reads_back() {
    for version in ["1.21.110", "1.21.120", "1.26.0", "1.26.30"] {
        let text = biome(
            version,
            "test:x",
            r#""minecraft:climate": {"temperature": 0.5}, "minecraft:tags": {"tags": ["a"]}"#,
        );
        let written =
            Biome::parse_as_written(&path(), text.as_bytes(), &context()).expect("parses");
        assert_eq!(written.value().parsed_version().to_string(), version);
        let json = written.to_json().expect("writes");
        let again =
            Biome::parse_as_written(&path(), json.as_bytes(), &context()).expect("reads back");
        assert_eq!(again.value(), written.value(), "{version}");
        let upgraded = history::upgrade(again, &context()).expect("upgrades");
        assert_eq!(
            upgraded.value(),
            loads(&text, &context()).value(),
            "{version}"
        );
    }
    // The beta struct.
    let beta = document(
        "beta",
        r#"{"description": {"identifier": "test:b"}, "components": {}}"#,
    );
    let written = Biome::parse_as_written(&path(), beta.as_bytes(), &base_game()).expect("parses");
    assert!(matches!(written.value(), BiomeHistory::Beta(_)));
    assert!(
        written
            .to_json()
            .unwrap()
            .contains(r#""format_version":"beta""#)
            || written
                .to_json()
                .unwrap()
                .contains(r#""format_version": "beta""#)
    );
    assert!(
        history::upgrade(written, &base_game()).is_err(),
        "a beta document is no Biome"
    );
}

/// the tag appends skip a tag already present (vanilla: "appends without
/// duplicate check"): vanilla would list `high_seas` twice.
#[test]
fn tag_appends_skip_present_tags_deviation() {
    let ocean = loads(
        &biome(
            "1.21.120",
            "minecraft:ocean",
            r#""minecraft:tags": {"tags": ["high_seas", "fast_fishing"]}"#,
        ),
        &context(),
    );
    assert_eq!(tags(ocean.value()), ["high_seas", "fast_fishing"]);
}

/// the 1.21.120, 1.26.0 and 1.26.30 slices are registered with
/// `StrictErrorCodes` (0xfffc), so an unknown component (`0x400`) or member (`0x4`) is an
/// error and the document is refused. The legacy slice below 1.21.120 refuses too: its `StrictJsonLoader` is built with
/// 0xfffc (`b13_unknown_component`).
#[test]
fn unknown_components_and_members_refuse_a_cereal_biome() {
    for version in ["1.21.120", "1.26.0", "1.26.30"] {
        let found = rejected(
            &biome(
                version,
                "test:x",
                r#""demo:mystery": {}, "minecraft:climate": {"temperature": 0.5, "ash": 2.0}"#,
            ),
            &context(),
        );
        let components = with(&found, |c| {
            matches!(c, DiagnosticCode::UnrecognizedComponent { .. })
        });
        let fields = with(&found, |c| {
            matches!(c, DiagnosticCode::UnrecognizedField { .. })
        });
        assert_eq!(components.len(), 1, "{version}: {found:#?}");
        assert_eq!(
            components[0].location.json_path.to_string(),
            "minecraft:biome/components/demo:mystery"
        );
        assert_eq!(fields.len(), 1, "{version}: {found:#?}");
        assert_eq!(
            fields[0].location.json_path.to_string(),
            "minecraft:biome/components/minecraft:climate/ash"
        );
        assert!(
            components
                .iter()
                .chain(&fields)
                .all(|d| d.severity == Severity::Error),
            "{version}: {found:#?}"
        );
    }
}

/// The beta slice too is the strict beta slice (mask 0xfffc): an unknown component of a
/// `"beta"` biome refuses it, from a base-game pack as from any other.
#[test]
fn an_unknown_component_refuses_a_beta_biome() {
    let text = biome("beta", "test:x", r#""demo:mystery": {}"#);
    let found = load_beta(&text, &base_game()).expect_err("refused");
    let components = with(&found.diagnostics, |c| {
        matches!(c, DiagnosticCode::UnrecognizedComponent { .. })
    });
    assert_eq!(components.len(), 1, "{:#?}", found.diagnostics);
    assert_eq!(components[0].severity, Severity::Error);
    assert_eq!(
        components[0].location.json_path.to_string(),
        "minecraft:biome/components/demo:mystery"
    );
}

/// The game's document input: the version is the first
/// textual `format_version`, a number read as its text: `1.26` is 1.26.0, read by its struct.
#[test]
fn the_version_is_read_as_minecraft_document_input_reads_it() {
    let text = biome("X", "test:x", "").replace(r#""X""#, "1.26");
    let document = loads(&text, &context());
    assert_eq!(
        document.version().written,
        Some(FormatVersion::new(1, 26, 0))
    );
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 26, 0));
}

/// a missing `format_version` is an error, not the
/// warn-and-floor default of.
#[test]
fn a_missing_format_version_is_an_error_deviation() {
    assert!(matches!(
        <BiomeFamily as Family>::ENVELOPE.missing,
        MissingVersion::Reject { .. }
    ));
    let found = rejected(
        r#"{"minecraft:biome": {"description": {"identifier": "test:x"}, "components": {}}}"#,
        &context(),
    );
    assert!(
        with(&found, |c| matches!(
            c,
            DiagnosticCode::MissingFormatVersion { .. }
        ))
        .iter()
        .all(|d| d.severity == Severity::Error)
    );
    assert_eq!(found.len(), 1);
}

/// the beta struct follows the 1.26.51 headers (the 1.26.30 noise gradient and
/// `environment_attributes`), not the 1.26.20 binary's Beta (`b19` pins the dev file).
#[test]
fn the_beta_struct_follows_the_1_26_51_headers_deviation() {
    let gradient = fixture("b18_1_26_30_files/sulfur_caves.biome.json").replacen(
        r#""format_version": "1.26.30""#,
        r#""format_version": "beta""#,
        1,
    );
    let beta = super::common::load_beta(&gradient, &base_game()).expect("the 1.26.30 shape");
    assert!(beta.diagnostics().is_empty(), "{:#?}", beta.diagnostics());
    let _ = Content::to_json(beta.value()).unwrap();
}

/// A biome reads every member strictly, so a file's unknown member refuses it; one put in the
/// table of a document as written (an edit) is upgraded with its object, as a step moves it.
#[test]
fn upgrade_keeps_the_unknown_members_of_the_document_as_written() {
    use bedrock_addon::diagnostic::JsonPath;
    use bedrock_addon::json::Value;
    let description = JsonPath::new()
        .child("minecraft:biome")
        .child("description");
    for version in ["1.21.110", "1.21.120", "1.26.0", "1.26.30"] {
        let text = biome(
            version,
            "test:x",
            r#""minecraft:climate": {"temperature": 0.5}"#,
        );
        let mut written =
            Biome::parse_as_written(&path(), text.as_bytes(), &context()).expect("parses");
        written
            .unknown_mut()
            .put(&description, "zz_description", Value::from(1i64));
        let expected = written.unknown().clone();
        let upgraded = history::upgrade(written, &context()).expect("upgrades");
        assert_eq!(
            upgraded.value(),
            loads(&text, &context()).value(),
            "{version}"
        );
        assert_eq!(upgraded.unknown(), &expected, "{version}");
    }
}
