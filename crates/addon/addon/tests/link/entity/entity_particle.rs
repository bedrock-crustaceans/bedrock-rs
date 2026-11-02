//! 20 E17: the particles an entity's components and events name.

#![allow(unused_imports, reason = "each feature set compiles part of the tests")]

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::pack::LinkOptions;

use super::super::common::{hosted, link, link_with};
use super::{FILE, entity_with, kept_unresolved, said_of, unresolved};

const SPARK: &str = r#"{"format_version": "1.10.0", "particle_effect": {"description": {"identifier": "demo:spark", "basic_render_parameters": {"material": "particles_alpha", "texture": "textures/particle/particles"}}, "components": {"minecraft:emitter_lifetime_once": {}, "minecraft:emitter_rate_instant": {"num_particles": 1}}}}"#;

fn pet(a: &str, b: &str, c: &str, d: &str, e: &str) -> String {
    entity_with(
        "1.26.50",
        "demo:pet",
        &format!(
            r#""minecraft:explode": {{"particle_effect": "{a}"}},
            "minecraft:projectile": {{"particle": "{b}", "on_hit": {{"particle_on_hit": {{"particle_type": "{c}"}}}}}},
            "minecraft:interact": {{"interactions": [{{"particle_on_start": {{"particle_type": "{d}"}}}}]}}"#
        ),
        &format!(
            r#", "events": {{"demo:puff": {{"sequence": [{{"emit_particle": {{"particle": "{e}"}}}}]}}}}"#
        ),
    )
}

/// A particle effect name (`explode.particle_effect`, `emit_particle.particle`) the client
/// cannot find is "Particle emitter has invalid effect name" when the emitter is spawned
/// (`ParticleEmitterActual` constructor), and the
/// entity goes on: a kept error. A `particle_type` member is an enumeration the game maps
/// without a line: an information.
#[cfg(feature = "particle")]
#[test]
fn e17_a_particle_nothing_defines_is_a_kept_error_an_unknown_particle_type_an_information() {
    let text = pet(
        "demo:none1",
        "demo:none2",
        "demo:none3",
        "demo:none4",
        "demo:none5",
    );
    let found = link(&[(FILE, &text)], &[]);
    let c = "components/minecraft:";
    assert_eq!(
        said_of(&found, "particle"),
        [
            kept_unresolved(
                &format!("{c}explode/particle_effect"),
                "particle",
                "demo:none1"
            ),
            unresolved(
                &format!("{c}projectile/on_hit/particle_on_hit/particle_type"),
                "particle",
                "demo:none3"
            ),
            unresolved(&format!("{c}projectile/particle"), "particle", "demo:none2"),
            unresolved(
                &format!("{c}interact/interactions/0/particle_on_start/particle_type"),
                "particle",
                "demo:none4"
            ),
            kept_unresolved(
                "events/demo:puff/sequence/0/emit_particle/particle",
                "particle",
                "demo:none5"
            ),
        ]
    );
    assert!(
        found
            .iter()
            .filter(|finding| finding.severity == Severity::Error)
            .all(|finding| finding.kept),
        "{found:?}"
    );
}

/// A particle type of the game's own list resolves, in any case; so does a particle effect a
/// resource pack defines, by its identifier as written.
#[cfg(feature = "particle")]
#[test]
fn e17_a_built_in_particle_type_and_a_packs_particle_effect_resolve() {
    let text = pet("largeexplode", "Smoke", "demo:spark", "food", "demo:Spark");
    let found = link(&[(FILE, &text)], &[("particles/spark.json", SPARK)]);
    assert_eq!(
        said_of(&found, "particle"),
        [kept_unresolved(
            "events/demo:puff/sequence/0/emit_particle/particle",
            "particle",
            "demo:Spark"
        )]
    );
}

/// With a host, a name that is no particle type and no pack's effect is the host's to
/// answer for.
#[cfg(feature = "particle")]
#[test]
fn e17_a_host_answers_for_the_games_particle_effects() {
    let text = pet(
        "heart",
        "heart",
        "heart",
        "heart",
        "minecraft:basic_flame_particle",
    );
    let files = [(FILE, &*text)];
    assert_eq!(said_of(&link(&files, &[]), "particle").len(), 1);
    let found = link_with(
        &files,
        &[],
        hosted(&[("particle", "minecraft:basic_flame_particle")]),
        &LinkOptions::default(),
    );
    assert!(said_of(&found, "particle").is_empty(), "{found:?}");
}

#[cfg(not(feature = "particle"))]
#[test]
fn e17_without_the_particle_feature_no_particle_is_told() {
    let text = pet(
        "demo:none1",
        "demo:none2",
        "demo:none3",
        "demo:none4",
        "demo:none5",
    );
    let found = link(&[(FILE, &text)], &[]);
    assert!(said_of(&found, "particle").is_empty(), "{found:?}");
    let _ = (SPARK, hosted(&[]), unresolved("", "", ""));
    let _ = (LinkOptions::default(), link_with as fn(_, _, _, _) -> _);
}
