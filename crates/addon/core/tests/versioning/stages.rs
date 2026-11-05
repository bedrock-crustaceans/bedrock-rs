//! The two public stages of `load`: `parse_as_written` (envelope, version, struct
//! selection, parse) and `finish` (upgrade, checks, Molang pass 2, family rules); the payload
//! version fallback; what a step's `UpgradeCx` carries; the older era's
//! `from_only` component enum; Molang through the component map.

use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity, VanillaText};
use bedrock_addon_core::document::{ComponentRef, VersionInfo};
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::format_molang_version;
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    ComponentTable, Family, LoadOutcome, Loaded, finish, load, member_version, parse_as_written,
};

use crate::chain::Routed;
use crate::envelope::{Stable, Strict, codes, load_toy};
use crate::toy::{Light, Speed, Toy, Toy120, ToyBeta, ToyComponent, history, v1_0_0, v1_1_0};

const V100: FormatVersion = FormatVersion::new(1, 0, 0);
const V110: FormatVersion = FormatVersion::new(1, 1, 0);

fn document(version: &str, payload: &str) -> String {
    format!(r#"{{"format_version": "{version}", "minecraft:toy": {payload}}}"#)
}

/// `load` and `finish(parse_as_written)` give the same outcome.
fn same_as_load<F: Family>(text: &str, cx: &LoadContext)
where
    F::Latest: PartialEq + std::fmt::Debug,
    F::Beta: PartialEq + std::fmt::Debug,
{
    let staged: LoadOutcome<F> = finish(parse_as_written::<F>(JsonSlice::new(text), cx), cx);
    let direct: LoadOutcome<F> = load(JsonSlice::new(text), cx);
    assert_eq!(staged.value, direct.value, "{text}");
    assert_eq!(staged.version, direct.version, "{text}");
    assert_eq!(staged.original_version, direct.original_version, "{text}");
    assert_eq!(staged.diagnostics, direct.diagnostics, "{text}");
}

#[test]
fn parse_as_written_gives_the_struct_the_version_selects() {
    let cx = LoadContext::detached();
    let text = document(
        "1.0.3",
        r#"{"description": {"identifier": "minecraft:pig"}, "old_speed": 1.0}"#,
    );
    let parsed = parse_as_written::<Toy>(JsonSlice::new(&text), &cx);
    let Some(history::Any::V1_0_0(floor)) = &parsed.value else {
        panic!("the floor struct: {:?}", parsed.diagnostics);
    };
    assert_eq!(floor.old_speed, Some(1.0));
    assert_eq!(
        parsed.version,
        VersionInfo::new(
            Some(FormatVersion::new(1, 0, 3)),
            false,
            FormatVersion::new(1, 0, 3),
            V100,
            false
        )
    );
    assert_eq!(parsed.identity.identifier.as_deref(), Some("minecraft:pig"));
    assert_eq!(parsed.actor_type, ActorType::Pig);
    assert!(!parsed.skip_upgrades);
    assert_eq!(parsed.at.to_string(), "minecraft:toy");
    assert!(parsed.diagnostics.is_empty());
    let parsed_version = parsed.value.as_ref().unwrap().parsed_version();
    assert_eq!(parsed_version, V100);
    // `finish` runs the steps and sets `upgraded`.
    let outcome = finish(parsed, &cx);
    assert!(outcome.version.upgraded);
    assert!(matches!(outcome.value, Some(Loaded::Stable(_))));
}

#[test]
fn finish_of_parse_as_written_equals_load() {
    let detached = LoadContext::detached();
    let mut beta = LoadContext::detached();
    beta.beta_apis = true;
    let texts = [
        document(
            "1.0.0",
            r#"{"description": {"identifier": "demo:x"}, "old_speed": 1.5, "legacy_flag": true}"#,
        ),
        document(
            "1.1.0-beta.1",
            r#"{"description": {"identifier": "demo:x"}}"#,
        ),
        document(
            "1.2.0",
            r#"{"description": {"identifier": "demo:x"}, "components": {"toy:gated": {"on": true}}}"#,
        ),
        document("0.5.0", r#"{"description": {"identifier": "demo:x"}}"#),
        document("beta", r#"{"description": {"identifier": "demo:x"}}"#),
        r#"{"minecraft:toy": {"description": {"identifier": "demo:x"}}}"#.to_owned(),
        r#"{"format_version": 7}"#.to_owned(),
        String::new(),
    ];
    for text in &texts {
        for cx in [&detached, &beta] {
            same_as_load::<Toy>(text, cx);
            same_as_load::<Strict>(text, cx);
            same_as_load::<Stable>(text, cx);
            same_as_load::<Routed>(text, cx);
        }
    }
}

#[test]
fn parse_as_written_rejects_as_load_does() {
    let cx = LoadContext::detached();
    let beta = document("beta", r#"{"description": {"identifier": "demo:x"}}"#);
    let parsed = parse_as_written::<Toy>(JsonSlice::new(&beta), &cx);
    assert!(parsed.value.is_none());
    assert!(parsed.version.beta);
    assert!(matches!(
        parsed.diagnostics.first_error().map(|e| &e.code),
        Some(DiagnosticCode::BetaFeaturesNotEnabled { .. })
    ));
    let missing = r#"{"minecraft:toy": {"description": {"identifier": "demo:x"}}}"#;
    let parsed = parse_as_written::<Strict>(JsonSlice::new(missing), &cx);
    assert!(parsed.value.is_none());
    // A rejected parse passes through `finish` with its findings.
    let outcome = finish(parsed, &cx);
    assert!(outcome.value.is_none());
    assert!(outcome.diagnostics.has_errors());
}

#[test]
fn parse_as_written_takes_the_route_and_the_skip_key() {
    let cx = LoadContext::detached();
    let text = document(
        "1.2.0",
        r#"{"description": {"identifier": "demo:x"}, "speed": 1.0, "do_not_upgrade": true}"#,
    );
    let parsed = parse_as_written::<Routed>(JsonSlice::new(&text), &cx);
    assert_eq!(parsed.version.parsed_as, V110);
    let parsed = parse_as_written::<Toy>(JsonSlice::new(&text), &cx);
    assert!(parsed.skip_upgrades && parsed.skip_section_a());
}

/// A family that reads the payload's own `format_version` when the envelope has none
/// (entity: `minecraft:entity.format_version`).
pub struct PayloadVersioned;
pub mod payload_versioned {
    use super::*;
    bedrock_addon_core::version_chain! {
        family = PayloadVersioned;
        /// The toy's chain with a payload version.
        pub enum Any {
            V1_0_0(v1_0_0::Toy),
            V1_1_0(v1_1_0::Toy),
            V1_2_0(Toy120),
        }
        beta = Beta(ToyBeta);
    }
}
impl Family for PayloadVersioned {
    const NAME: &'static str = "toy";
    const PAYLOAD_KEY: &'static str = "minecraft:toy";
    const FLOOR: FormatVersion = V100;
    type Latest = Toy120;
    type Beta = ToyBeta;
    type Component = ToyComponent;
    type Any = payload_versioned::Any;
    fn components() -> &'static ComponentTable<Self> {
        &ComponentTable::EMPTY
    }
    fn payload_version(payload: JsonSlice<'_>) -> Option<FormatVersionInput> {
        member_version(payload, "format_version")
    }
}

#[test]
fn the_payload_version_stands_in_for_an_absent_envelope_version() {
    let cx = LoadContext::detached();
    let inner = |version: &str| {
        format!(
            r#"{{"minecraft:toy": {{"format_version": "{version}", "description": {{"identifier": "demo:x"}}}}}}"#
        )
    };
    // Below the floor: the floor text, as for an envelope version.
    let outcome = load::<PayloadVersioned>(JsonSlice::new(&inner("0.2.0")), &cx);
    assert!(outcome.value.is_none());
    assert!(matches!(
        outcome.diagnostics.first_error().map(|e| &e.code),
        Some(DiagnosticCode::UnsupportedVersion {
            text: Some(VanillaText::UseFormatVersion { .. })
        })
    ));
    // Inside the range: parsed by its struct, no missing-version warning.
    let outcome = load::<PayloadVersioned>(JsonSlice::new(&inner("1.1.0")), &cx);
    assert!(outcome.value.is_some(), "{:?}", outcome.diagnostics);
    assert_eq!(outcome.version.written, Some(V110));
    assert_eq!(outcome.version.parsed_as, V110);
    assert!(
        !codes(&outcome)
            .iter()
            .any(|(code, _)| matches!(code, DiagnosticCode::MissingFormatVersion { .. }))
    );
    // The envelope's version wins when present; without either, the floor and a warning.
    let both = r#"{"format_version": "1.2.0", "minecraft:toy": {"format_version": "1.1.0", "description": {"identifier": "demo:x"}}}"#;
    let outcome = load::<PayloadVersioned>(JsonSlice::new(both), &cx);
    assert_eq!(outcome.version.parsed_as, FormatVersion::new(1, 2, 0));
    let neither = r#"{"minecraft:toy": {"description": {"identifier": "demo:x"}}}"#;
    let outcome = load::<PayloadVersioned>(JsonSlice::new(neither), &cx);
    assert!(matches!(
        codes(&outcome).first(),
        Some((DiagnosticCode::MissingFormatVersion { .. }, Severity::Info))
    ));
}

#[test]
fn member_version_reads_a_string_or_three_integers() {
    let read = |text: &str| member_version(JsonSlice::new(text), "format_version");
    assert_eq!(
        read(r#"{"format_version": "1.2.3"}"#).and_then(|v| v.version),
        Some(FormatVersion::new(1, 2, 3))
    );
    assert_eq!(
        read(r#"{"format_version": [1, 2, 3]}"#).and_then(|v| v.version),
        Some(FormatVersion::new(1, 2, 3))
    );
    assert_eq!(read(r#"{"format_version": true}"#), None);
    assert_eq!(read(r"{}"), None);
}

/// `Family::identity` and `Family::actor_type` reach a step's `UpgradeCx`.
#[test]
fn a_step_sees_the_identifier_and_the_actor_type() {
    let seen = |payload: &str| {
        let outcome = load_toy(&document("1.0.0", payload));
        match outcome.value {
            Some(Loaded::Stable(toy)) => toy.upgraded_as,
            other => panic!("{other:?}: {:?}", outcome.diagnostics),
        }
    };
    assert_eq!(
        seen(r#"{"description": {"identifier": "minecraft:pig"}}"#),
        Some(("minecraft:pig".to_owned(), ActorType::Pig))
    );
    assert_eq!(
        seen(r#"{"description": {"identifier": "demo:x", "runtime_identifier": "minecraft:cow"}}"#),
        Some(("demo:x".to_owned(), ActorType::Cow))
    );
    assert_eq!(
        seen(r#"{"description": {"identifier": "demo:x"}}"#),
        Some(("demo:x".to_owned(), ActorType::Mob))
    );
}

/// The floor era's `from_only` enum moves into Latest's map by `From` and
/// `insert_typed`; an unknown key stays unknown.
#[test]
fn the_floor_eras_components_move_into_latests_map() {
    let outcome = load_toy(&document(
        "1.0.0",
        r#"{"description": {"identifier": "demo:x"}, "components": {"toy:speed": {"value": 2.0}, "toy:odd": 1}}"#,
    ));
    let Some(Loaded::Stable(toy)) = &outcome.value else {
        panic!("{:?}", outcome.diagnostics);
    };
    assert_eq!(toy.components.get::<Speed>().map(|s| s.value), Some(2.0));
    let entries: Vec<_> = toy
        .components
        .iter()
        .map(|entry| match entry {
            ComponentRef::Typed {
                name, synthesized, ..
            } => (name.to_owned(), !synthesized),
            ComponentRef::Unknown { name, .. } => (name.to_owned(), false),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        entries,
        [
            ("toy:speed".to_owned(), true),
            ("toy:odd".to_owned(), false),
            ("toy:marker".to_owned(), false),
        ]
    );
    // A failing component of the old era is reported below its key.
    let outcome = load_toy(&document(
        "1.0.0",
        r#"{"description": {"identifier": "demo:x"}, "components": {"toy:speed": {"value": "fast"}}}"#,
    ));
    let finding = outcome.diagnostics.iter().next().expect("a finding");
    assert_eq!(
        finding.location.json_path.to_string(),
        "minecraft:toy/components/toy:speed/value"
    );
    let v1: v1_0_0::Component = Speed::default().into();
    assert!(matches!(v1, v1_0_0::Component::Speed(_)));
}

/// Pass 1 reaches a component's Molang field through the map and the derive's walk, and pass 2
/// (here: unchecked) counts it.
#[test]
fn molang_reaches_a_component_through_the_map() {
    let outcome = load_toy(&document(
        "1.2.0",
        r#"{"description": {"identifier": "demo:x"}, "components": {"toy:light": {"condition": "q.is_baby"}}}"#,
    ));
    let Some(Loaded::Stable(toy)) = &outcome.value else {
        panic!("{:?}", outcome.diagnostics);
    };
    let light = toy.components.get::<Light>().expect("light");
    let source = light
        .condition
        .as_ref()
        .and_then(|c| c.source())
        .expect("an expression");
    let document_version = FormatVersionInput::parse("1.2.0").unwrap();
    assert_eq!(
        source.raw_version(),
        Some(format_molang_version(&document_version))
    );
    assert_eq!(
        codes(&outcome),
        [(
            DiagnosticCode::MolangNotChecked { fields: 1 },
            Severity::Info
        )]
    );
}

/// With `LoadContext::molang`, pass 2 compiles every field before the family rules; a rejected
/// expression is `InvalidMolang` at its path and rejects the document.
#[cfg(feature = "molang")]
#[test]
fn molang_pass_two_runs_with_a_check() {
    use bedrock_addon_core::molang::MolangCheck;
    use std::sync::Arc;
    let mut cx = LoadContext::detached();
    cx.molang = Some(Arc::new(MolangCheck::new()));
    let outcome = bedrock_addon_core::versioning::load::<Toy>(
        JsonSlice::new(&document(
            "1.2.0",
            r#"{"description": {"identifier": "demo:x"}, "components": {"toy:light": {"condition": "1 +"}}}"#,
        )),
        &cx,
    );
    let error = outcome.diagnostics.first_error().expect("an error");
    assert!(
        matches!(error.code, DiagnosticCode::InvalidMolang { .. }),
        "{:?}",
        error.code
    );
    assert_eq!(
        error.location.json_path.to_string(),
        "minecraft:toy/components/toy:light/condition"
    );
    assert!(outcome.value.is_none());
    // A valid one passes without `MolangNotChecked`.
    let outcome = bedrock_addon_core::versioning::load::<Toy>(
        JsonSlice::new(&document(
            "1.2.0",
            r#"{"description": {"identifier": "demo:x"}, "components": {"toy:light": {"condition": "1 + 1"}}}"#,
        )),
        &cx,
    );
    assert!(outcome.value.is_some(), "{:?}", outcome.diagnostics);
    assert!(outcome.diagnostics.is_empty(), "{:?}", outcome.diagnostics);
}

/// A family that reads a file by hand builds its parse from the value: the identity, the actor
/// type and the skip key take their defaults until it says otherwise, and `finish` takes it as
/// it takes the pipeline's own.
#[test]
fn a_parse_built_from_its_value_finishes_as_the_pipeline_s() {
    use bedrock_addon_core::diagnostic::Diagnostics;
    use bedrock_addon_core::versioning::{Identity, Parsed};
    let cx = LoadContext::detached();
    let text = document("1.1.0", r#"{"description": {"identifier": "demo:x"}}"#);
    let own = parse_as_written::<Toy>(JsonSlice::new(&text), &cx);
    let built = Parsed::<Toy>::from_value(
        own.value.clone(),
        own.version.clone(),
        own.written,
        own.at.clone(),
        Diagnostics::default(),
    );
    assert_eq!(built.identity, Identity::default());
    assert_eq!(built.actor_type, ActorType::Mob);
    assert!(!built.skip_upgrades);
    let identity = own.identity.clone();
    let built = built
        .with_identity(identity.clone())
        .with_actor(ActorType::Mob, true);
    assert_eq!(built.identity, identity);
    assert!(built.skip_upgrades);
    let built = built.with_actor(own.actor_type, own.skip_upgrades);
    let (own, built) = (finish(own, &cx), finish(built, &cx));
    assert_eq!(built.value, own.value);
    assert_eq!(built.version, own.version);
    assert_eq!(built.original_version, own.original_version);
}
