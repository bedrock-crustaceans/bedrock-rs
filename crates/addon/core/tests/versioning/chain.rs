//! The toy family through `version_chain!` and `#[derive(Components)]`, the derive and the
//! version chain working together: the generated chain, the upgrade walk, section-A
//! suppression, the routing hook, Molang pass 1, the effective version and the generated
//! `history::ToyHistory` methods.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::{ComponentMap, ComponentRef, Document, Origin, VersionInfo};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::format_molang_version;
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    BetaPermission, ChainError, ComponentSection, ComponentTable, Envelope, Family, Loaded,
    MissingVersion, NoComponents, NoStable, VersionChain, check_chain, file_version,
    validate_components,
};

use crate::envelope::{codes, load_toy, load_with};
use crate::toy::{Marker, Toy, Toy120, ToyBeta, ToyComponent, history, v1_0_0, v1_1_0};

const V100: FormatVersion = FormatVersion::new(1, 0, 0);
const V110: FormatVersion = FormatVersion::new(1, 1, 0);
const V120: FormatVersion = FormatVersion::new(1, 2, 0);

fn document(version: &str, payload: &str) -> String {
    format!(r#"{{"format_version": "{version}", "minecraft:toy": {payload}}}"#)
}

fn latest(outcome: &bedrock_addon_core::versioning::LoadOutcome<Toy>) -> &Toy120 {
    match &outcome.value {
        Some(Loaded::Stable(toy)) => toy,
        other => panic!("a stable value, not {other:?}: {:?}", outcome.diagnostics),
    }
}

#[test]
fn the_toy_chain_is_well_formed() {
    assert_eq!(check_chain::<Toy>(), Ok(()));
    assert_eq!(
        <history::ToyHistory as VersionChain<Toy>>::VERSIONS,
        [V100, V110, V120]
    );
    const { assert!(<history::ToyHistory as VersionChain<Toy>>::HAS_BETA) };
    assert!(Toy::components().is_sorted());
    assert!(Toy::components().get("toy:speed").is_some());
}

#[test]
fn a_floor_document_walks_every_step_to_latest() {
    let outcome = load_toy(&document(
        "1.0.0",
        r#"{"description": {"identifier": "demo:x"}, "old_speed": 1.5, "legacy_flag": true}"#,
    ));
    let toy = latest(&outcome);
    assert_eq!(
        toy.movement.as_ref().map(|m| m.speed),
        Some(1.5),
        "renamed, then moved"
    );
    assert_eq!(toy.color, "RED", "default, then the callback");
    assert_eq!(toy.legacy, Some(true), "the gate below 1.0.5");
    assert_eq!(toy.marker, Some(true), "the section-A insert");
    assert!(matches!(
        toy.components.iter().next(),
        Some(ComponentRef::Typed {
            name: "toy:marker",
            synthesized: true,
            ..
        })
    ));
    assert_eq!(
        codes(&outcome),
        [(Code::from(&core_codes::DROPPED_FIELD), Severity::Warning)]
    );
    assert_eq!(
        outcome.version,
        VersionInfo::new(Some(V100), false, V100, V100, true)
    );
}

#[test]
fn a_gate_on_the_original_version_skips_newer_documents() {
    let outcome = load_toy(&document(
        "1.0.7",
        r#"{"description": {"identifier": "demo:x"}}"#,
    ));
    assert_eq!(latest(&outcome).legacy, None);
    assert_eq!(outcome.original_version, FormatVersion::new(1, 0, 7));
    assert_eq!(outcome.version.parsed_as, V100);
}

#[test]
fn a_latest_document_runs_no_step() {
    let outcome = load_toy(&document(
        "1.2.0",
        r#"{"description": {"identifier": "demo:x"}, "color": "blue"}"#,
    ));
    let toy = latest(&outcome);
    assert_eq!(toy.color, "blue");
    assert_eq!(toy.marker, None);
    assert!(!outcome.version.upgraded);
}

/// The skip key suppresses section-A ops only; B and C ops still run.
#[test]
fn the_skip_upgrades_key_suppresses_section_a_ops_only() {
    let outcome = load_toy(&document(
        "1.0.0",
        r#"{"description": {"identifier": "demo:x"}, "old_speed": 2.0, "do_not_upgrade": true}"#,
    ));
    let toy = latest(&outcome);
    assert_eq!(toy.marker, None, "section A skipped");
    assert!(toy.components.is_empty(), "no synthesized component");
    assert_eq!(
        toy.movement.as_ref().map(|m| m.speed),
        Some(2.0),
        "the B ops ran"
    );
    assert_eq!(toy.color, "RED");
}

/// A family whose routing hook sends a document with a `speed` member to the 1.1.0 struct,
/// whatever its version.
pub struct Routed;
pub mod routed {
    use super::*;
    bedrock_addon_core::version_chain! {
        family = Routed;
        /// The toy's chain with a routing hook.
        pub enum History {
            V1_0_0(v1_0_0::Toy),
            V1_1_0(v1_1_0::Toy),
            V1_2_0(Toy120),
        }
        beta = Beta(ToyBeta);
    }
}
impl Family for Routed {
    const NAME: &'static str = "toy";
    const PAYLOAD_KEY: &'static str = "minecraft:toy";
    const FLOOR: FormatVersion = V100;
    type Latest = Toy120;
    type Beta = ToyBeta;
    type Component = ToyComponent;
    type History = routed::History;
    fn components() -> &'static ComponentTable<Self> {
        &ComponentTable::EMPTY
    }
    fn route(payload: JsonSlice<'_>, _: FormatVersionInput, _: &LoadContext) -> Option<usize> {
        payload.text.contains("\"speed\"").then_some(1)
    }
}

#[test]
fn the_route_hook_picks_the_struct() {
    let outcome = load_with::<Routed>(
        &document(
            "1.2.0",
            r#"{"description": {"identifier": "demo:x"}, "speed": 3.0}"#,
        ),
        &LoadContext::detached(),
    );
    assert_eq!(outcome.version.parsed_as, V110);
    let Some(Loaded::Stable(toy)) = &outcome.value else {
        panic!("{:?}", outcome.diagnostics);
    };
    assert_eq!(toy.movement.as_ref().map(|m| m.speed), Some(3.0));
    // Without the member, version dispatch.
    let outcome = load_with::<Routed>(
        &document("1.2.0", r#"{"description": {"identifier": "demo:x"}}"#),
        &LoadContext::detached(),
    );
    assert_eq!(outcome.version.parsed_as, V120);
}

/// Section A is also skipped when `original_version >= ENGINE_VERSION`.
#[test]
fn section_a_is_skipped_at_the_engine_version() {
    let engine = ENGINE_VERSION.to_string();
    let outcome = load_with::<Routed>(
        &document(
            &engine,
            r#"{"description": {"identifier": "demo:x"}, "speed": 1.0}"#,
        ),
        &LoadContext::detached(),
    );
    let Some(Loaded::Stable(toy)) = &outcome.value else {
        panic!("{:?}", outcome.diagnostics);
    };
    assert_eq!(outcome.version.parsed_as, V110, "the step ran");
    assert_eq!(toy.marker, None, "but not its section-A op");
}

#[test]
fn file_version_reads_the_envelope_s_version_for_routing() {
    let text = document("1.1.0", "{}");
    let version = file_version(JsonSlice::new(&text));
    assert_eq!(version, Some(V110));
}

/// Pass 1 runs inside the parse: a string `condition` gets the document's version.
#[test]
fn molang_pass_one_gives_a_field_the_documents_version() {
    let outcome = load_toy(&document(
        "1.2.0",
        r#"{"description": {"identifier": "demo:x"}, "condition": "q.is_baby"}"#,
    ));
    let condition = latest(&outcome).condition.as_ref().expect("a condition");
    let source = condition.source().expect("an expression");
    let document = FormatVersionInput::parse("1.2.0").unwrap();
    assert_eq!(source.raw_version(), Some(format_molang_version(&document)));
}

#[test]
fn molang_pass_one_reports_a_refused_field_below_the_payload() {
    let outcome = load_toy(&document(
        "1.2.0",
        r#"{"description": {"identifier": "demo:x"}, "condition": {"no": "expression"}}"#,
    ));
    assert!(outcome.value.is_none());
    let error = outcome.diagnostics.first_error().expect("an error");
    assert!(
        error
            .location
            .json_path
            .to_string()
            .starts_with("minecraft:toy"),
        "{error:?}"
    );
}

#[test]
fn the_chain_parses_a_struct_without_upgrading_it() {
    let text = document(
        "1.1.0",
        r#"{"description": {"identifier": "demo:x"}, "speed": 4.0, "color": "green"}"#,
    );
    let envelope = bedrock_addon_core::json::probe_envelope(JsonSlice::new(&text)).unwrap();
    let payload = envelope.payload("minecraft:toy").unwrap();
    let mut d = Diagnostics::default();
    let any = <history::ToyHistory as VersionChain<Toy>>::parse(
        payload,
        FormatVersion::new(1, 1, 5),
        &LoadContext::detached(),
        &mut d,
    )
    .expect("parsed");
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(any.parsed_version(), V110);
    // Written in its own version, envelope included.
    let written = any.to_json().expect("written");
    let reread: Value = facet_json::from_str(&written).unwrap();
    let original: Value = facet_json::from_str(&text).unwrap();
    assert_eq!(reread, original);
    // An unsupported version is reported.
    let mut d = Diagnostics::default();
    let none = <history::ToyHistory as VersionChain<Toy>>::parse(
        payload,
        FormatVersion::new(0, 1, 0),
        &LoadContext::detached(),
        &mut d,
    );
    assert!(none.is_none());
    assert!(
        d.first_error()
            .is_some_and(|e| e.code.id() == "core/no_parser_for_version")
    );
    assert_eq!(
        d.first_error().unwrap().location.json_path.to_string(),
        "format_version"
    );
    // A typed failure is reported below the payload key, so the path points at the member
    // inside the payload and not at the document root.
    let text = document(
        "1.1.0",
        r#"{"description": {"identifier": "demo:x"}, "speed": "fast"}"#,
    );
    let envelope = bedrock_addon_core::json::probe_envelope(JsonSlice::new(&text)).unwrap();
    let mut d = Diagnostics::default();
    let none = <history::ToyHistory as VersionChain<Toy>>::parse(
        envelope.payload("minecraft:toy").unwrap(),
        FormatVersion::new(1, 1, 0),
        &LoadContext::detached(),
        &mut d,
    );
    assert!(none.is_none());
    assert_eq!(
        d.first_error().unwrap().location.json_path.to_string(),
        "minecraft:toy/speed"
    );
}

#[test]
fn a_beta_struct_is_written_with_the_beta_literal() {
    let any = history::ToyHistory::Beta(ToyBeta::default());
    let written: Value = facet_json::from_str(&any.to_json().unwrap()).unwrap();
    let version = written.as_object().unwrap().get("format_version").unwrap();
    assert_eq!(
        version.as_string().map(facet_value::VString::as_str),
        Some("beta")
    );
    assert_eq!(
        any.parsed_version(),
        bedrock_addon_core::versioning::BETA_VERSION
    );
}

/// `Document<history::ToyHistory>` derefs to `Document`'s own methods
/// first, so no generated method may be named like one of them.
#[test]
fn no_generated_method_is_named_like_a_document_method() {
    let source = include_str!("../../src/versioning/chain.rs");
    for name in [
        "value",
        "into_value",
        "origin",
        "version",
        "diagnostics",
        "map",
    ] {
        assert!(
            !source.contains(&format!("fn {name}(")),
            "version_chain! generates `{name}`"
        );
    }
    let origin = Origin::new(
        PackRef::app_data(),
        PackSide::Behavior,
        PackPath::new("toys/x.json").unwrap(),
        None,
        Vec::new(),
    );
    let info = VersionInfo::new(Some(V110), false, V110, V110, false);
    let document = Document::new(
        history::ToyHistory::V1_0_0(v1_0_0::Toy::default()),
        origin,
        info.clone(),
        Diagnostics::default(),
    );
    let version: &VersionInfo = document.version();
    assert_eq!(version, &info);
    assert_eq!(document.parsed_version(), V100, "reached through Deref");
}

#[test]
fn a_family_rule_rejects_the_document() {
    let outcome = load_toy(&document(
        "1.2.0",
        r#"{"description": {"identifier": "demo:x"}, "color": "BLACK"}"#,
    ));
    assert!(outcome.value.is_none());
    assert!(
        outcome
            .diagnostics
            .first_error()
            .is_some_and(|e| e.code.id() == "core/invalid_value")
    );
}

#[test]
fn the_derive_and_the_chain_share_one_component_enum() {
    let component: ToyComponent = Marker { on: true }.into();
    assert!(matches!(component, ToyComponent::Marker(_)));
}

/// A beta-only family (atomic client entities): no stable struct, every load is the beta
/// struct, a missing version rejected, beta allowed from any pack.
pub struct BetaOnly;
pub mod beta_only {
    use super::*;
    bedrock_addon_core::version_chain! {
        family = BetaOnly;
        /// Only the beta struct.
        pub enum History {}
        beta = Beta(ToyBeta);
    }
}
impl Family for BetaOnly {
    const NAME: &'static str = "beta toy";
    const PAYLOAD_KEY: &'static str = "minecraft:toy";
    const FLOOR: FormatVersion = V100;
    const BETA_PERMISSION: BetaPermission = BetaPermission::Always;
    const ENVELOPE: Envelope = Envelope {
        missing: MissingVersion::REJECT,
        ..Envelope::DEFAULT
    };
    type Latest = NoStable;
    type Beta = ToyBeta;
    type Component = NoComponents;
    type History = beta_only::History;
    // No `components`: the trait's default is the empty table.
}

#[test]
fn a_family_without_components_has_the_empty_table() {
    assert!(BetaOnly::components().0.is_empty());
    assert!(BetaOnly::components().get("minecraft:anything").is_none());
}

/// A family with components that forgot to override `components`: the trait's empty table.
pub struct ForgotTable;
impl Family for ForgotTable {
    const NAME: &'static str = "toy";
    const PAYLOAD_KEY: &'static str = "minecraft:toy";
    const FLOOR: FormatVersion = V100;
    type Latest = Toy120;
    type Beta = std::convert::Infallible;
    type Component = ToyComponent;
    type History = forgot_table::History;
}
pub mod forgot_table {
    use super::*;
    bedrock_addon_core::version_chain! {
        family = ForgotTable;
        /// The toy's chain, for a family without its table.
        pub enum History {
            V1_0_0(v1_0_0::Toy),
            V1_1_0(v1_1_0::Toy),
            V1_2_0(Toy120),
        }
    }
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "components")]
fn a_family_with_components_and_the_default_table_fails_a_debug_assertion() {
    let mut map = ComponentMap::<ToyComponent>::new();
    let mut d = Diagnostics::default();
    validate_components::<ForgotTable>(
        ComponentSection::new(JsonPath::new(), false, &mut map),
        V100,
        &LoadContext::detached(),
        &JsonPath::new(),
        &mut d,
    );
}

#[test]
fn a_family_without_components_reads_the_empty_table_without_a_panic() {
    let mut map = ComponentMap::<NoComponents>::new();
    let mut d = Diagnostics::default();
    assert!(validate_components::<BetaOnly>(
        ComponentSection::new(JsonPath::new(), false, &mut map),
        V100,
        &LoadContext::detached(),
        &JsonPath::new(),
        &mut d,
    ));
}

#[test]
fn a_beta_only_family_loads_only_beta_documents() {
    assert_eq!(check_chain::<BetaOnly>(), Ok(()));
    assert!(<beta_only::History as VersionChain<BetaOnly>>::VERSIONS.is_empty());
    let cx = LoadContext::detached();
    let outcome = load_with::<BetaOnly>(
        r#"{"format_version": "beta", "minecraft:toy": {"description": {"identifier": "demo:x"}}}"#,
        &cx,
    );
    assert!(
        matches!(outcome.value, Some(Loaded::Beta(_))),
        "{:?}",
        outcome.diagnostics
    );
    for text in [
        r#"{"format_version": "1.2.0", "minecraft:toy": {"description": {"identifier": "demo:x"}}}"#,
        r#"{"minecraft:toy": {"description": {"identifier": "demo:x"}}}"#,
    ] {
        let outcome = load_with::<BetaOnly>(text, &cx);
        assert!(outcome.value.is_none(), "{text}");
        assert!(
            outcome
                .diagnostics
                .first_error()
                .is_some_and(|e| e.code.id() == "core/no_parser_for_version"
                    || e.code.id() == "core/no_valid_version")
        );
    }
}

/// A family whose floor is above its first struct.
pub struct FloorAbove;
impl Family for FloorAbove {
    const NAME: &'static str = "toy";
    const PAYLOAD_KEY: &'static str = "minecraft:toy";
    const FLOOR: FormatVersion = V110;
    type Latest = Toy120;
    type Beta = std::convert::Infallible;
    type Component = ToyComponent;
    type History = floor_above::History;
    fn components() -> &'static ComponentTable<Self> {
        &ComponentTable::EMPTY
    }
}
pub mod floor_above {
    use super::*;
    bedrock_addon_core::version_chain! {
        family = FloorAbove;
        /// The toy's chain under a floor above its first struct.
        pub enum History {
            V1_0_0(v1_0_0::Toy),
            V1_1_0(v1_1_0::Toy),
            V1_2_0(Toy120),
        }
    }
}

#[test]
fn check_chain_rejects_a_floor_above_the_first_struct() {
    assert_eq!(
        check_chain::<FloorAbove>(),
        Err(ChainError::FloorAboveFirst {
            floor: V110,
            first: V100
        })
    );
}
