//! The chain: contiguity, the range model's selection, floor, ceiling and beta; the Value round
//! trip; history without upgrade; and the deviations the folder lists, each pinned by a test named `deviation`.

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::document::{Content, JsonDocument};
use bedrock_addon::families::block::Block;
use bedrock_addon::families::block::component::{Friction, LightEmission};
use bedrock_addon::families::block::history::{self, Any, BlockFamily};
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput};
use bedrock_addon::versioning::{RangeModel, Selection, VersionChain, check_chain, select};

use super::common::{context, document, fixture, loads, rejected};

/// The chain's versions, in order.
const VERSIONS: [(u16, u16, u16); 16] = [
    (1, 16, 0),
    (1, 16, 100),
    (1, 18, 0),
    (1, 19, 10),
    (1, 19, 20),
    (1, 19, 30),
    (1, 19, 40),
    (1, 19, 80),
    (1, 20, 10),
    (1, 20, 60),
    (1, 21, 30),
    (1, 21, 60),
    (1, 21, 90),
    (1, 26, 0),
    (1, 26, 20),
    (1, 26, 60),
];

/// What `version` selects.
fn selects(version: &str) -> Selection {
    select::<BlockFamily>(&FormatVersionInput::parse(version).expect("a version"))
}

#[test]
fn the_chain_is_contiguous_and_ends_at_1_26_60() {
    const HAS_BETA: bool = <Any as VersionChain<BlockFamily>>::HAS_BETA;

    check_chain::<BlockFamily>().expect("the chain is well formed");
    let versions: Vec<_> = VERSIONS
        .iter()
        .map(|&(a, b, c)| FormatVersion::new(a, b, c))
        .collect();
    assert_eq!(
        <Any as VersionChain<BlockFamily>>::VERSIONS,
        versions.as_slice()
    );
    let has_beta = HAS_BETA;
    assert!(!has_beta, "Beta = Infallible");
}

#[test]
fn each_version_selects_the_struct_whose_range_holds_it() {
    for (index, &(a, b, c)) in VERSIONS.iter().enumerate() {
        assert_eq!(selects(&format!("{a}.{b}.{c}")), Selection::Struct(index));
    }
    // Inside a range, up to but excluding the next struct's version.
    assert_eq!(selects("1.19.60"), Selection::Struct(6));
    assert_eq!(selects("1.21.120"), Selection::Struct(12));
    // A pre-release sorts below its release (1.21.60-beta.3 is parsed by v1_21_30).
    assert_eq!(selects("1.21.60-beta.3"), Selection::Struct(10));
    // Stamps from 1.26.20 below 1.26.60 parse as 1.26.20; from 1.26.60 up to the engine as
    // the newest struct; the model closes at the engine.
    assert_eq!(selects("1.26.30"), Selection::Struct(14));
    assert_eq!(selects("1.26.55"), Selection::Struct(14));
    assert_eq!(selects(&ENGINE_VERSION.to_string()), Selection::Struct(15));
    assert_eq!(selects("1.26.61"), Selection::Unsupported);
    assert_eq!(selects("1.15.0"), Selection::BelowFloor);
    assert_eq!(
        <BlockFamily as bedrock_addon::versioning::Family>::RANGE_MODEL,
        RangeModel::ClosedToEngine
    );
}

#[test]
fn a_prerelease_loads_through_the_struct_below_it_with_an_info() {
    let block = loads(
        &document(
            "1.21.60-beta.3",
            r#"{"description": {"identifier": "demo:x"}, "components": {}}"#,
        ),
        &context(),
    );
    assert_eq!(block.version().parsed_as, FormatVersion::new(1, 21, 30));
    assert!(block.diagnostics().iter().any(|f| matches!(
        f.code,
        DiagnosticCode::PrereleaseFormatVersion { .. }
    ) && f.severity == Severity::Info));
}

// `1.26.20-beta.1` sorts below 1.26.20, so the legacy struct
// reads it from the whole text, as it reads every document below the envelope minimum.
#[test]
fn a_prerelease_of_1_26_20_is_read_from_the_raw_text_by_the_legacy_struct() {
    let block = loads(
        r#"{"format_version": "1.26.20-beta.1", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:friction": 0.5}}, "x": 1}"#,
        &context(),
    );
    assert_eq!(block.version().parsed_as, FormatVersion::new(1, 26, 0));
    assert_eq!(block.value().description.identifier.as_str(), "demo:x");
    assert!(
        block.components.get::<Friction>().is_some(),
        "the legacy struct read the payload's components"
    );
    // The release itself is the checked envelope: a third root member rejects the document.
    let d = rejected(
        r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {}}, "x": 1}"#,
        &context(),
    );
    assert!(
        d.iter()
            .any(|f| matches!(f.code, DiagnosticCode::MalformedInput { .. }))
    );
}

/// The block loader has no Beta slice: selection comes before the permission, so `beta` has
/// no parser with or without the Beta APIs.
#[test]
fn the_beta_literal_has_no_struct_whatever_the_beta_apis() {
    let text = document(
        "beta",
        r#"{"description": {"identifier": "demo:x"}, "components": {}}"#,
    );
    let mut cx = context();
    cx.beta_apis = true;
    for cx in [context(), cx] {
        let d = rejected(&text, &cx);
        assert!(
            d.iter()
                .any(|f| matches!(f.code, DiagnosticCode::UnsupportedVersion { .. })),
            "{d:#?}"
        );
        assert!(
            !d.iter()
                .any(|f| matches!(f.code, DiagnosticCode::BetaFeaturesNotEnabled { .. })),
            "{d:#?}"
        );
    }
}

/// The Value round trip of a newest-version document: parse, write, parse again, equal.
#[test]
fn a_1_26_20_document_round_trips_at_the_value_level() {
    for name in ["v06_shelf_mushroom.json", "v06_red_shrub.json"] {
        let cx = super::common::ucf();
        let first = loads(&fixture(name), &cx);
        let written = first.to_json().unwrap();
        let second = loads(&written, &cx);
        assert_eq!(second.value(), first.value(), "{name}");
        assert_eq!(second.to_json().unwrap(), written, "{name}: stable");
    }
    // Unknown members of a cereal document reject it, each told where it is (strict slice);
    // a legacy document keeps them at their level.
    let strict = document(
        "1.26.20",
        r#"{"description": {"identifier": "demo:x", "note": 1}, "components": {
            "minecraft:material_instances": {"*": {"texture": "x", "shininess": 2}}},
          "permutations": [{"condition": "1", "components": {}, "comment": "hi"}], "author": "me"}"#,
    );
    let d = super::common::rejected(&strict, &context());
    for member in [
        "description/note",
        "components/minecraft:material_instances/*/shininess",
        "permutations/0/comment",
        "author",
    ] {
        assert!(
            d.iter()
                .any(|f| f.severity == bedrock_addon::diagnostic::Severity::Error
                    && f.location.json_path.to_string() == format!("minecraft:block/{member}")),
            "{member}: {d:#?}"
        );
    }
    let text = document(
        "1.26.0",
        r#"{"description": {"identifier": "demo:x", "note": 1}, "components": {
            "minecraft:collision_box": {"origin": [-8, 0, -8], "size": [16, 8, 16], "color": "red"},
            "minecraft:material_instances": {"*": {"texture": "x", "shininess": 2}},
            "minecraft:geometry": "geometry.x", "demo:custom": {"a": [1, 2]}},
          "permutations": [{"condition": "1", "components": {}, "comment": "hi"}], "author": "me"}"#,
    );
    let block = loads(&text, &context());
    let written = block.to_json().unwrap();
    for member in [
        "\"note\"",
        "\"color\"",
        "\"shininess\"",
        "\"demo:custom\"",
        "\"comment\"",
        "\"author\"",
    ] {
        assert!(written.contains(member), "{member} kept: {written}");
    }
    assert_eq!(loads(&written, &context()).value(), block.value());
}

#[test]
fn parse_as_written_keeps_the_old_struct_and_upgrade_gives_the_parse_result() {
    let text = fixture("upgrade/s00_1_16_0_description_flags.before.json");
    let path = PackPath::from_static("blocks/demo/x.json");
    let cx = context();
    let written = Block::parse_as_written(&path, text.as_bytes(), &cx).expect("parses");
    assert!(matches!(written.value(), Any::V1_16_0(_)));
    assert_eq!(written.parsed_version(), FormatVersion::new(1, 16, 0));
    // Written in its own version, unchanged.
    let own: bedrock_addon::json::Value =
        facet_json::from_str(&written.to_json().unwrap()).unwrap();
    let input: bedrock_addon::json::Value = facet_json::from_str(&text).unwrap();
    assert_eq!(own, input);
    let upgraded = history::upgrade(written, &cx).expect("upgrades");
    let parsed = Block::parse(&path, text.as_bytes(), &cx).expect("loads");
    assert_eq!(upgraded.value(), parsed.value());
}

// ---- deviations ----------------------------------------------------------------------------

#[test]
fn a_document_below_1_16_0_is_rejected_deviation() {
    // vanilla: no floor; the 1.19.10 unwraps discard the object-form components.
    let d = rejected(
        &document(
            "1.10.0",
            r#"{"description": {"identifier": "demo:x"}, "components": {}}"#,
        ),
        &context(),
    );
    assert!(
        d.iter()
            .any(|f| matches!(f.code, DiagnosticCode::UnsupportedVersion { .. }))
    );
}

#[test]
fn a_missing_format_version_parses_as_1_16_0_deviation() {
    // vanilla: "no valid version passed by the caller or found in the input", not loaded.
    let block = loads(
        r#"{"minecraft:block": {"description": {"identifier": "demo:x"}, "components": {}}}"#,
        &context(),
    );
    assert_eq!(block.version().effective, FormatVersion::new(1, 16, 0));
}

#[test]
fn friction_is_clamped_after_its_inversion_deviation() {
    // vanilla: unclamped 1 - n.
    let block = loads(
        &document(
            "1.19.10",
            r#"{"description": {"identifier": "demo:x"}, "components": {},
                "permutations": [{"condition": "1", "components": {"minecraft:friction": 0.01}}]}"#,
        ),
        &context(),
    );
    assert_eq!(
        block.permutations[0].components.get::<Friction>(),
        Some(&Friction(bedrock_addon::json::StrictFloat(0.9)))
    );
}

#[test]
fn light_emission_is_clamped_deviation() {
    // vanilla: unclamped.
    let block = loads(
        &document(
            "1.18.0",
            r#"{"description": {"identifier": "demo:x"}, "components": {},
                "permutations": [{"condition": "1", "components": {"minecraft:block_light_emission": 3.0}}]}"#,
        ),
        &context(),
    );
    assert_eq!(
        block.permutations[0].components.get::<LightEmission>(),
        Some(&LightEmission(bedrock_addon::json::StrictInt(15)))
    );
}

#[test]
fn a_third_root_member_below_1_26_20_is_reported_deviation() {
    // vanilla ignores it silently on the raw-text path.
    let block = loads(
        r#"{"format_version": "1.20.0", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {}}, "x": 1}"#,
        &context(),
    );
    assert!(
        block
            .diagnostics()
            .iter()
            .any(|f| matches!(f.code, DiagnosticCode::UnrecognizedField { .. }))
    );
}

#[test]
fn a_load_context_is_all_a_block_needs() {
    let cx = LoadContext::for_pack(
        &LoadOptions::default(),
        uuid::Uuid::nil(),
        bedrock_addon::pack::PackType::Behavior,
        None,
        false,
    );
    loads(
        &document(
            "1.26.20",
            r#"{"description": {"identifier": "demo:x"}, "components": {}}"#,
        ),
        &cx,
    );
}

/// `events` is the JSON name of the deprecated handlers in a 1.26.20 document,
/// bound and deprecated by name (`deprecateName`): cereal's 0x200
/// `member "events" is deprecated`, an error in both masks that rejects the block.
#[test]
fn events_of_a_1_26_20_document_reject_the_block() {
    let text = document(
        "1.26.20",
        r#"{"description": {"identifier": "demo:x"}, "components": {},
            "events": {"hiss": {"condition": "q.block_state('demo:a')", "responses": [{"run_command": {"command": ["say hi"]}}], "note": 1}}}"#,
    );
    let d = rejected(&text, &context());
    assert!(
        d.iter().any(
            |f| matches!(f.code, DiagnosticCode::DeprecatedMember { .. })
                && f.severity == Severity::Error
                && f.code.to_string() == "member \"events\" is deprecated"
                && f.location.json_path.to_string().ends_with("events")
        ),
        "{d:#?}"
    );
}

/// The game checks that a member is deprecated before it
/// reads it and then skips the value, so `"events": null` gets the
/// deprecation alone, not also the null rule's type error.
#[test]
fn a_null_events_member_of_a_1_26_20_document_is_told_as_deprecated_alone() {
    let text = document(
        "1.26.20",
        r#"{"description": {"identifier": "demo:x"}, "components": {}, "events": null}"#,
    );
    let d = rejected(&text, &context());
    let at_events: Vec<String> = d
        .iter()
        .filter(|f| f.location.json_path.to_string().ends_with("events"))
        .map(|f| format!("{:?} {}", f.severity, f.code))
        .collect();
    assert_eq!(
        at_events,
        ["Error member \"events\" is deprecated"],
        "{d:#?}"
    );
}

#[test]
fn upgrade_keeps_the_unknown_members_a_parse_keeps() {
    // A 1.16.0 document: the upgrade rebuilds the payload, so its members are moved.
    let text = r#"{"format_version": "1.16.0", "minecraft:block": {
        "description": {"identifier": "design:x", "zz_description": 1},
        "components": {
            "minecraft:destroy_time": 4.0,
            "minecraft:flammable": {"flame_odds": 50, "burn_odds": 0, "zz_flammable": 2}
        },
        "zz_payload": 3
    }}"#;
    let path = PackPath::from_static("blocks/demo/x.json");
    let cx = context();
    let parsed = Block::parse(&path, text.as_bytes(), &cx).expect("loads");
    assert!(!parsed.unknown().is_empty(), "the parse keeps a member");
    let written = Block::parse_as_written(&path, text.as_bytes(), &cx).expect("parses");
    let upgraded = history::upgrade(written, &cx).expect("upgrades");
    assert_eq!(upgraded.value(), parsed.value());
    assert_eq!(upgraded.unknown(), parsed.unknown());
}
