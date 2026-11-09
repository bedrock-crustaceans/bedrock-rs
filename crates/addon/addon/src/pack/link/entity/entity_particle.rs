//! The particles an entity's components and events name: a particle type of the game
//! (compared in lower case) or a particle effect of the resource stack (by its identifier as
//! written). An effect name (`explode.particle_effect`, `emit_particle.particle`) the client
//! cannot find is logged as "Particle emitter has invalid effect name" at the error level
//! when the emitter is spawned, after which the entity goes on: a kept error. A
//! `particle_type` member is an enumeration the game maps without a line: an information.
use bedrock_addon_core::diagnostic::{JsonPath, Severity};
use bedrock_addon_core::document::Origin;
use bedrock_addon_core::id::Id;

use super::table::PARTICLES;
use super::walk::{Entities, members};
use crate::pack::link::prelude::*;

/// The component whose member holds an effect name, not a particle type.
const EXPLODE: &str = "minecraft:explode";

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings, entities: &Entities<'_>) {
    entities.each(PARTICLES, &mut |component, at, value| {
        if let Some(name) = super::name(value) {
            let effect = component.key == EXPLODE;
            particle(cx, out, component.origin, at, name, effect);
        }
    });
    // The event response `emit_particle`, in an event's body or in any node nested in it.
    for node in &entities.events {
        members(
            node.json,
            "emit_particle.particle",
            &node.at,
            &mut |at, value| {
                if let Some(name) = super::name(value) {
                    particle(cx, out, node.origin, at, name, true);
                }
            },
        );
    }
}

/// One particle name; `effect` when the member holds an effect name the client spawns an
/// emitter for (the kept error), not a particle type (the information).
fn particle(
    cx: &LinkCx<'_>,
    out: &mut Findings,
    origin: &Origin,
    at: JsonPath,
    name: &str,
    effect: bool,
) {
    if !cx.particle_type(&name.to_ascii_lowercase()).is_missing() {
        return;
    }
    if let Target::Missing(hint) = cx.particle(&Id::parse(name)) {
        if effect {
            let at = At::new(origin, at, Severity::Error);
            out.push(unresolved(at, kinds::PARTICLE, name, hint).into_kept());
        } else {
            let at = At::new(origin, at, Severity::Info);
            out.push(unresolved(at, kinds::PARTICLE, name, hint));
        }
    }
}
