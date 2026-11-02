//! The 1.10.0 → 1.20.80 step, the chain and the history path.

use bedrock_addon::families::particle::ParticleEffect;
use bedrock_addon::families::particle::component::ParticleComponent;
use bedrock_addon::families::particle::history::{self, Any, ParticleFamily, v1_10_0};
use bedrock_addon::json::Value;
use bedrock_addon::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput};
use bedrock_addon::versioning::testing::{Checks, Run, step_output};
use bedrock_addon::versioning::{
    EnvelopeCheck, Family, MissingVersion, RangeModel, Selection, VersionChain, check_chain, select,
};

use super::common::{CEREAL, LEGACY, at, context, effect, fixture, json, loaded, path};

fn selects(version: &str) -> Selection {
    select::<ParticleFamily>(&FormatVersionInput::parse(version).expect("a version"))
}

/// The payload of fixture p06: one of each of the 29 components, two curves, one event.
fn full_payload() -> Value {
    let document = json(&fixture("p06_upgrade_full.before.json"));
    at(&document, "particle_effect").cloned().unwrap()
}

/// The 1.10.0 step on `input`: the 1.20.80 payload it writes.
#[track_caller]
fn step(input: &Value) -> Value {
    step_output::<v1_10_0::ParticleEffect>(input, Run::default(), &Checks::default())
        .unwrap_or_else(|_| panic!("the step failed"))
        .payload
}

/// Every string of the legacy document that is Molang is an expression object at the load
/// context's version afterwards, and every number is the same number.
#[test]
fn the_step_turns_molang_into_expression_nodes() {
    let output = step(&full_payload());
    let version = i64::from(
        bedrock_addon::load::LoadContext::detached()
            .molang_version
            .0,
    );
    let expect = |pointer: &str, expected: Value| {
        assert_eq!(
            at(&output, pointer),
            Some(&expected),
            "a Molang member is not converted: {pointer}"
        );
    };
    let node = |text: &str| {
        json(&format!(
            r#"{{"expression": "{text}", "version": {version}}}"#
        ))
    };
    expect(
        "components/minecraft:emitter_rate_steady/spawn_rate",
        node("variable.rate"),
    );
    expect(
        "components/minecraft:emitter_rate_steady/max_particles",
        json("50"),
    );
    expect(
        "components/minecraft:emitter_shape_box/half_dimensions",
        json(&format!(
            r#"[1, {{"expression": "variable.height", "version": {version}}}, 1]"#
        )),
    );
    expect("curves/variable.fade/input", node("variable.particle_age"));
    expect(
        "curves/variable.fade/nodes/1",
        node("variable.particle_random_1"),
    );
    expect(
        "components/minecraft:particle_appearance_tinting/color/gradient/1.0/3",
        node("variable.fade"),
    );
}

/// The 29 components arrive under their names, in the document's order, each with the members it
/// had.
#[test]
fn the_step_moves_the_29_components_by_name() {
    let input = full_payload();
    let output = step(&input);
    let names = |payload: &Value| -> Vec<String> {
        at(payload, "components")
            .and_then(Value::as_object)
            .map(|o| o.iter().map(|(key, _)| key.as_str().to_owned()).collect())
            .unwrap_or_default()
    };
    assert_eq!(names(&input).len(), 29, "the input has the 29 components");
    assert_eq!(
        names(&output),
        names(&input),
        "a component is missing, renamed or out of order"
    );
    for component in [
        "minecraft:particle_kill_plane",
        "minecraft:emitter_lifetime_events",
        "minecraft:particle_expire_if_in_blocks",
        "minecraft:emitter_local_space",
    ] {
        let pointer = format!("components/{component}");
        assert_eq!(
            at(&output, &pointer),
            at(&input, &pointer),
            "a component without Molang changed: {component}"
        );
    }
}

/// The curves keep their names, order, kinds and nodes; a document without curves has none
/// afterwards.
#[test]
fn the_step_keys_the_curves_by_name() {
    let input = full_payload();
    let output = step(&input);
    let names: Vec<String> = at(&output, "curves")
        .and_then(Value::as_object)
        .map(|o| o.iter().map(|(key, _)| key.as_str().to_owned()).collect())
        .unwrap_or_default();
    assert_eq!(
        names,
        ["variable.fade", "variable.swell"],
        "the curves are not keyed by their names in order"
    );
    for pointer in [
        "curves/variable.fade/type",
        "curves/variable.swell/type",
        "curves/variable.swell/nodes",
    ] {
        assert_eq!(at(&output, pointer), at(&input, pointer), "{pointer}");
    }
    let bare = step(&json(
        r#"{"description": {"identifier": "demo:spark", "basic_render_parameters": {"material": "particles_alpha", "texture": "textures/particle/particles"}}, "components": {"minecraft:emitter_lifetime_once": {}}}"#,
    ));
    assert!(at(&bare, "curves").is_none(), "curves were invented");
}

#[test]
fn the_chain_has_the_two_structs_of_the_plan() {
    assert_eq!(
        <Any as VersionChain<ParticleFamily>>::VERSIONS,
        [FormatVersion::new(1, 10, 0), FormatVersion::new(1, 20, 80)]
    );
}

#[test]
fn the_chain_is_the_plans() {
    const HAS_BETA: bool = <Any as VersionChain<ParticleFamily>>::HAS_BETA;
    check_chain::<ParticleFamily>().expect("the chain is well formed");
    let has_beta = HAS_BETA;
    assert!(!has_beta);
    assert_eq!(
        <ParticleFamily as Family>::RANGE_MODEL,
        RangeModel::ClosedToEngine
    );
    assert_eq!(
        <ParticleFamily as Family>::FLOOR,
        FormatVersion::new(1, 10, 0)
    );
    assert_eq!(<ParticleFamily as Family>::CEILING, None);
    assert_eq!(<ParticleFamily as Family>::NAME, "particle");
    assert_eq!(<ParticleFamily as Family>::PAYLOAD_KEY, "particle_effect");
    assert_eq!(<ParticleFamily as Family>::DIRECTORIES, &["particles"]);
    assert_eq!(
        <ParticleFamily as Family>::MISSING_VERSION,
        MissingVersion::Floor
    );
    assert_eq!(
        <ParticleFamily as Family>::ENVELOPE,
        EnvelopeCheck::From(FormatVersion::new(1, 20, 80))
    );
    assert_eq!(<ParticleFamily as Family>::SKIP_UPGRADES_KEY, None);
    assert!(<ParticleFamily as Family>::DUPLICATES_KEPT_IN.is_empty());
}

#[test]
fn each_version_selects_the_struct_whose_range_holds_it() {
    for (version, index) in [
        ("1.10.0", 0),
        ("1.10.1", 0),
        ("1.15.0", 0),
        ("1.20.79", 0),
        ("1.20.80", 1),
        ("1.20.81", 1),
        ("1.26.10", 1),
        ("1.26.30", 1),
    ] {
        assert_eq!(selects(version), Selection::Struct(index), "{version}");
    }
    let engine = ENGINE_VERSION.to_string();
    assert_eq!(selects(&engine), Selection::Struct(1));
    assert!(!matches!(selects("1.9.99"), Selection::Struct(_)));
    assert!(!matches!(selects("1.26.61"), Selection::Struct(_)));
    assert!(!matches!(selects("beta"), Selection::Struct(_)));
}

/// The component table: 29 rows, sorted, each released at the floor without a gate.
#[test]
fn the_component_table_has_the_29_rows_of_the_metadata() {
    let table = <ParticleFamily as Family>::components();
    assert_eq!(table.0.len(), 29);
    assert!(table.is_sorted());
    for row in table.0 {
        assert!(row.name.starts_with("minecraft:"), "{}", row.name);
        assert_eq!(
            row.released,
            Some(FormatVersion::new(1, 10, 0)),
            "{}",
            row.name
        );
        assert_eq!(row.deprecated_since, None);
        assert_eq!(row.supersedes, None);
        assert!(row.requires.is_empty() && row.fields.is_empty());
    }
    // The 1.26.30 legacy registry's 30th name is no row: it has no 1.20.80 type.
    assert!(
        table
            .get("minecraft:particle_appearance_camera_distance_fade")
            .is_none()
    );
    // A row's parser reads its component from its slice, leniently or strictly.
    let row = table.get("minecraft:emitter_rate_instant").unwrap();
    let slice = bedrock_addon::json::JsonSlice::new("{}");
    let mut found = bedrock_addon::diagnostic::Diagnostics::default();
    let lenient = (row.parse)(
        slice,
        bedrock_addon::versioning::ConvertMode::Lenient,
        &context(),
        &mut found,
    );
    assert!(matches!(
        lenient,
        Some(ParticleComponent::EmitterRateInstant(_))
    ));
    assert_eq!(found.len(), 1);
    let mut found = bedrock_addon::diagnostic::Diagnostics::default();
    let strict = (row.parse)(
        slice,
        bedrock_addon::versioning::ConvertMode::Strict,
        &context(),
        &mut found,
    );
    assert!(strict.is_some() && found.has_errors());
}

/// `parse_as_written` gives the struct the version selects, not upgraded; `history::upgrade`
/// finishes it to what `parse` gives.
#[test]
fn history_parses_as_written_and_upgrades_to_the_same_document() {
    let cx = context();
    for (version, legacy) in [
        (LEGACY, true),
        ("1.20.79", true),
        ("1.20.80", false),
        (CEREAL, false),
    ] {
        let text = effect(
            version,
            r#""minecraft:emitter_shape_point": {}, "minecraft:emitter_shape_sphere": {"radius": "variable.r"}"#,
        );
        let written = ParticleEffect::parse_as_written(&path(), text.as_bytes(), &cx).unwrap();
        assert!(!written.version().upgraded);
        match written.value() {
            Any::V1_10_0(effect) => {
                assert!(legacy, "{version}");
                assert_eq!(effect.format_version.to_string(), version);
                assert_eq!(effect.components.len(), 4);
                // As written: the point has no direction yet.
                let v1_10_0::Component::Typed(ParticleComponent::EmitterShapePoint(point)) =
                    &effect.components["minecraft:emitter_shape_point"]
                else {
                    panic!();
                };
                assert_eq!(point.direction, None);
            }
            Any::V1_20_80(_) => assert!(!legacy, "{version}"),
        }
        let upgraded = history::upgrade(written, &cx).unwrap();
        let direct = loaded(&text);
        assert_eq!(upgraded.value(), direct.value(), "{version}");
        assert_eq!(upgraded.version().upgraded, legacy, "{version}");
        assert_eq!(
            super::common::texts(upgraded.diagnostics()),
            super::common::texts(direct.diagnostics()),
            "{version}"
        );
    }
}

/// The 1.10.0 struct writes the document it read, at its own version.
#[test]
fn the_1_10_0_struct_writes_its_own_document() {
    use bedrock_addon::versioning::VersionStruct;
    let text = effect(
        "1.16.0",
        r#""minecraft:emitter_shape_sphere": {"radius": "variable.r"}"#,
    );
    let written = ParticleEffect::parse_as_written(&path(), text.as_bytes(), &context()).unwrap();
    let Any::V1_10_0(effect) = written.value() else {
        panic!();
    };
    assert_eq!(effect.write().unwrap(), json(&text));
}
