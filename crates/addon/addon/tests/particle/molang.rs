//! Molang in particle effects: the forms a member
//! takes and the version each compiles at in both builds, and, with the `molang` feature,
//! the compile check and what a member that does not compile becomes.

#![allow(
    unused_imports,
    dead_code,
    reason = "part of the tests needs the `molang` feature, part its absence"
)]

use bedrock_addon::diagnostic::{Code, Diagnostics, Severity};
use bedrock_addon::document::JsonDocument as _;
use bedrock_addon::families::particle::ParticleEffect;
use bedrock_addon::families::particle::component::{
    EmitterLifetimeExpression, EmitterRateInstant, EmitterShapeSphere, ParticleInitialization,
};
use bedrock_addon::molang::{Molang, MolangSource, RawVersion};

use super::common::{
    BASE, CEREAL, LEGACY, component, document, effect, effect_with, findings, json, loaded, loads,
    number, object, pack_context, rejected, texts,
};

/// A document whose rate component has `num_particles`.
fn rate(version: &str, num_particles: &str) -> String {
    document(
        version,
        &format!(
            r#""components": {{"minecraft:emitter_lifetime_once": {{}}, "minecraft:emitter_rate_instant": {{"num_particles": {num_particles}}}}}"#
        ),
    )
}

fn num_particles(effect: &ParticleEffect) -> Molang {
    let rate = effect.components.get::<EmitterRateInstant>().unwrap();
    rate.num_particles.clone()
}

/// The Molang findings: `InvalidMolang` and `MolangLint`, as `Severity path code-name`.
fn molang_findings(d: &Diagnostics) -> Vec<String> {
    d.iter()
        // By legacy name: the variant and the definitions that were it.
        .filter(|f| matches!(f.code.id(), "core/invalid_molang" | "core/molang_lint"))
        .map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code.id()))
        .collect()
}

/// `p25_molang_forms`: the forms of one member in a pack at 1.21.80 (Molang version 12).
#[test]
fn p25_molang_forms() {
    let cx = pack_context(1, 21, 80);
    let constant = loads(&rate(LEGACY, "5"), &cx);
    assert_eq!(num_particles(&constant), number(5.0));
    assert_eq!(findings(constant.diagnostics()), [] as [&str; 0]);
    // A legacy string compiles at the pack's version, which the upgrade writes out.
    let string = loads(&rate(LEGACY, r#""variable.n""#), &cx);
    assert_eq!(num_particles(&string), object("variable.n", 12));
    assert_eq!(findings(string.diagnostics()), [] as [&str; 0]);
    // As written, before the upgrade, it is the string form at the pack's version.
    let written = ParticleEffect::parse_as_written(
        &super::common::path(),
        rate(LEGACY, r#""variable.n""#).as_bytes(),
        &cx,
    )
    .unwrap();
    let bedrock_addon::families::particle::history::ParticleHistory::V1_10_0(legacy) =
        written.into_value()
    else {
        panic!();
    };
    let bedrock_addon::families::particle::history::v1_10_0::Component::Typed(component) =
        &legacy.components["minecraft:emitter_rate_instant"]
    else {
        panic!();
    };
    assert!(
        format!("{component:?}").contains("context_version: Some(RawVersion(12))"),
        "{component:?}"
    );
    // The object form keeps its version and is written back as the same object.
    let form = r#"{"expression": "variable.n", "version": 12}"#;
    let cereal = loads(&rate(CEREAL, form), &cx);
    assert_eq!(num_particles(&cereal), object("variable.n", 12));
    assert_eq!(findings(cereal.diagnostics()), [] as [&str; 0]);
    let rewritten = component_member(&cereal, "num_particles");
    assert_eq!(rewritten, json(form));
    // A bare string from 1.20.80 takes the pack's version and stays a string.
    let bare = loads(&rate(CEREAL, r#""variable.n""#), &cx);
    assert_eq!(
        num_particles(&bare),
        Molang::Expr(MolangSource::string("variable.n", RawVersion(12)))
    );
    assert_eq!(
        component_member(&bare, "num_particles"),
        json(r#""variable.n""#)
    );
    // The strict reader takes no bool.
    assert_eq!(
        texts(&rejected(&rate(CEREAL, "true"))),
        [
            "Error particle_effect/components/minecraft:emitter_rate_instant/num_particles core/invalid_type: wrong JSON type"
        ]
    );
    // An empty expression is no expression: not compiled, nothing said.
    let empty = loads(&rate(CEREAL, r#"{"expression": "", "version": 12}"#), &cx);
    assert_eq!(num_particles(&empty), object("", 12));
    assert_eq!(findings(empty.diagnostics()), [] as [&str; 0]);
}

fn component_member(
    effect: &bedrock_addon::document::Document<ParticleEffect>,
    member: &str,
) -> facet_value::Value {
    let written = component(effect, "minecraft:emitter_rate_instant");
    written.as_object().unwrap().get(member).cloned().unwrap()
}

/// Without the `molang` feature nothing is compiled: one info says how many expressions the
/// document holds, and a legacy source that would not compile is kept.
#[cfg(not(feature = "molang"))]
#[test]
fn without_the_molang_feature_the_expressions_are_counted_and_kept() {
    let effect = loaded(&effect(
        LEGACY,
        r#""minecraft:emitter_shape_sphere": {"radius": "1 +", "offset": ["variable.x", 0, "variable.z"]}"#,
    ));
    assert_eq!(
        texts(effect.diagnostics()),
        [
            "Info particle_effect core/molang_not_checked: 3 Molang field(s) not checked: the `molang` feature is off"
        ]
    );
    let sphere = effect.components.get::<EmitterShapeSphere>().unwrap();
    assert_eq!(sphere.radius, Some(object("1 +", 13)));
    // A document without an expression gets no info.
    assert_eq!(
        texts(loaded(&super::common::effect(CEREAL, "")).diagnostics()),
        [] as [&str; 0]
    );
}

/// `p26_molang_version_edges`: an object's version is unconstrained; one no query table
/// knows resolves no query, and the document still loads with the node kept.
#[cfg(feature = "molang")]
#[test]
fn p26_molang_version_edges() {
    let at = "particle_effect/components/minecraft:emitter_rate_instant/num_particles";
    let future = loaded(&rate(
        CEREAL,
        r#"{"expression": "query.is_baby", "version": 99}"#,
    ));
    assert_eq!(num_particles(&future), object("query.is_baby", 99));
    let found = molang_findings(future.diagnostics());
    assert!(
        found.contains(&format!("Error {at} core/invalid_molang")),
        "{found:#?}"
    );
    let messages: Vec<String> = future
        .diagnostics()
        .iter()
        .map(ToString::to_string)
        .collect();
    assert!(
        messages.iter().any(|m| m.contains(
            "Failed to resolve query query.is_baby.  Either the query does not exist or it is not supported in this context."
        )),
        "{messages:#?}"
    );
    // The parser's line is a kept error: nothing rejects the document.
    assert!(!future.diagnostics().has_errors());
    // −1 is compiled (vanilla: a development assert only), with an info.
    let minus = loaded(&rate(CEREAL, r#"{"expression": "1", "version": -1}"#));
    assert_eq!(num_particles(&minus), object("1", -1));
    assert!(
        minus
            .diagnostics()
            .iter()
            .all(|d| d.severity <= Severity::Info),
        "{:#?}",
        texts(minus.diagnostics())
    );
    assert!(!minus.diagnostics().is_empty());
    // A version the tables know has nothing to say.
    let known = loaded(&rate(
        CEREAL,
        r#"{"expression": "query.is_baby", "version": 13}"#,
    ));
    assert_eq!(texts(known.diagnostics()), [] as [&str; 0]);
}

/// `p27_molang_client_queries`: particle Molang compiles against the client's queries.
#[cfg(feature = "molang")]
#[test]
fn p27_molang_client_queries() {
    let effect = loaded(&effect(
        CEREAL,
        r#""minecraft:particle_initialization": {"per_update_expression": "variable.d = query.distance_from_camera; variable.n = query.effect_particle_count;"}, "minecraft:emitter_lifetime_expression": {"activation_expression": "query.is_on_screen"}"#,
    ));
    assert_eq!(texts(effect.diagnostics()), [] as [&str; 0]);
    assert!(effect.components.get::<ParticleInitialization>().is_some());
    assert!(
        effect
            .components
            .get::<EmitterLifetimeExpression>()
            .is_some()
    );
}

/// `p28_molang_legacy_failure`: a legacy member that does not compile becomes its default
/// constant; a 1.20.80 one, and a legacy event's, keeps its text. Every one of them is the
/// Molang parser's content-log error after which the effect loads: kept errors, for the
/// legacy member, the 1.20.80 one and the legacy event.
#[cfg(feature = "molang")]
#[test]
fn p28_molang_legacy_failure() {
    let sphere = r#""minecraft:emitter_shape_sphere": {"radius": "1 +"}"#;
    let at = "particle_effect/components/minecraft:emitter_shape_sphere/radius";
    let radius = |effect: &ParticleEffect| {
        let sphere = effect.components.get::<EmitterShapeSphere>().unwrap();
        sphere.radius.clone()
    };
    let legacy = loaded(&effect(LEGACY, sphere));
    assert_eq!(radius(&legacy), Some(number(1.0)));
    assert_eq!(
        molang_findings(legacy.diagnostics()),
        [format!("Error {at} core/invalid_molang")]
    );
    assert!(
        !legacy.diagnostics().has_errors(),
        "a legacy Molang line rejects"
    );
    let cereal = loaded(&effect(CEREAL, sphere));
    assert_eq!(
        radius(&cereal),
        Some(Molang::Expr(MolangSource::string("1 +", RawVersion(13))))
    );
    assert_eq!(
        molang_findings(cereal.diagnostics()),
        [format!("Error {at} core/invalid_molang")]
    );
    assert!(
        !cereal.diagnostics().has_errors(),
        "a 1.20.80 Molang line rejects"
    );
    let event = loaded(&effect_with(
        LEGACY,
        r#""events": {"e": {"expression": "1 +"}}"#,
    ));
    assert_eq!(
        event.events.as_ref().unwrap()["e"].expression,
        Some(object("1 +", 13))
    );
    assert_eq!(
        molang_findings(event.diagnostics()),
        ["Error particle_effect/events/e/expression core/invalid_molang"]
    );
    assert!(
        !event.diagnostics().has_errors(),
        "a legacy event's Molang line rejects"
    );
}

/// An expression of an event node behind a node the legacy reader leaves out (one that is
/// no object, a random option without a weight) is told at the index the file has it at.
#[cfg(feature = "molang")]
#[test]
fn a_rejected_expression_behind_a_node_left_out_is_at_the_index_of_the_file() {
    let event = loaded(&effect_with(
        LEGACY,
        r#""events": {"e": {"sequence": [7, {"expression": "1 +"}], "randomize": [{}, {"weight": 1, "expression": "1 +", "sequence": [7, 7, {"expression": "1 +"}]}]}}"#,
    ));
    let at = "Error particle_effect/events/e";
    assert_eq!(
        molang_findings(event.diagnostics()),
        [
            format!("{at}/sequence/1/expression core/invalid_molang"),
            format!("{at}/randomize/1/sequence/2/expression core/invalid_molang"),
            format!("{at}/randomize/1/expression core/invalid_molang"),
        ]
    );
}

/// The legacy default is the member's own: each element of an array, a curve node, a
/// required member; a member that compiles is reported once and kept.
#[cfg(feature = "molang")]
#[test]
fn a_rejected_legacy_member_becomes_its_own_default() {
    let effect = loaded(&document(
        LEGACY,
        &format!(
            r#""components": {{{BASE}, "minecraft:emitter_initialization": {{"creation_expression": "(", "per_update_expression": "variable.ok = 1;"}}, "minecraft:particle_appearance_billboard": {{"size": ["(", "variable.s"]}}, "minecraft:particle_lifetime_expression": {{"max_lifetime": "("}}, "minecraft:emitter_lifetime_expression": {{"activation_expression": "("}}}}, "curves": {{"variable.c": {{"type": "linear", "input": "(", "horizontal_range": "(", "nodes": ["(", "variable.n"]}}}}"#
        ),
    ));
    let written = super::common::written(&effect);
    let at =
        |pointer: &str| super::common::at(&written, &format!("particle_effect/{pointer}")).cloned();
    assert_eq!(
        at("components/minecraft:emitter_initialization/creation_expression"),
        Some(json("1"))
    );
    assert_eq!(
        at("components/minecraft:emitter_initialization/per_update_expression"),
        Some(json(r#"{"expression": "variable.ok = 1;", "version": 13}"#))
    );
    assert_eq!(
        at("components/minecraft:particle_appearance_billboard/size"),
        Some(json(r#"[1, {"expression": "variable.s", "version": 13}]"#))
    );
    assert_eq!(
        at("components/minecraft:particle_lifetime_expression/max_lifetime"),
        Some(json("1"))
    );
    assert_eq!(
        at("components/minecraft:emitter_lifetime_expression/activation_expression"),
        Some(json("1"))
    );
    assert_eq!(at("curves/variable.c/input"), Some(json("0")));
    assert_eq!(at("curves/variable.c/horizontal_range"), Some(json("1")));
    assert_eq!(
        at("curves/variable.c/nodes"),
        Some(json(r#"[0, {"expression": "variable.n", "version": 13}]"#))
    );
    // Seven rejected members, each reported once, where it is.
    let mut found = molang_findings(effect.diagnostics());
    found.dedup();
    assert_eq!(found.len(), 7, "{found:#?}");
    assert!(found.contains(
        &"Error particle_effect/curves/variable.c/nodes/0 core/invalid_molang".to_owned()
    ));
    // The upgraded document, read again, has nothing left to report.
    let again = loaded(&bedrock_addon::document::Content::to_json(effect.value()).unwrap());
    assert_eq!(texts(again.diagnostics()), [] as [&str; 0]);
}

/// A legacy bool is the number the 1.20.80 reader can hold.
#[test]
fn a_legacy_bool_is_upgraded_to_its_number() {
    let effect = loaded(&rate(LEGACY, "true"));
    assert_eq!(num_particles(&effect), number(1.0));
    assert_eq!(component_member(&effect, "num_particles"), json("1"));
}

/// The built-in direction of a legacy point emitter is at Molang version 13 in a pack of
/// any version, and compiles clean.
#[test]
fn the_built_in_point_direction_is_at_version_13() {
    let text = effect(LEGACY, r#""minecraft:emitter_shape_point": {}"#);
    for (minor, patch) in [(13, 0), (16, 0), (20, 0)] {
        let effect = loads(&text, &pack_context(1, minor, patch));
        let written = component(&effect, "minecraft:emitter_shape_point");
        assert_eq!(
            written,
            json(
                r#"{"direction": [{"expression": "math.random(-1, 1)", "version": 13}, {"expression": "math.random(-1, 1)", "version": 13}, {"expression": "math.random(-1, 1)", "version": 13}]}"#
            )
        );
        assert_eq!(findings(effect.diagnostics()), [] as [&str; 0]);
    }
}
