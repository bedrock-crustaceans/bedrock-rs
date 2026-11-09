//! The entities and the level sound events an entity's components name: the
//! pair inside the `entity` module (a file `entity.rs` would repeat its directory's name).
use bedrock_addon_core::diagnostic::{JsonPath, Severity};
use bedrock_addon_core::document::Origin;
use bedrock_addon_core::id::EntityId;

use super::table::{BREEDS_WITH, ENTITIES, OFFSPRING, SOUNDS};
use super::walk::Entities;
use crate::pack::link::prelude::*;

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings, entities: &Entities<'_>) {
    entities.each(ENTITIES, &mut |component, at, value| {
        if let Some(name) = super::name(value) {
            entity(cx, out, component.origin, at, name);
        } else if component.key == OFFSPRING.0 {
            for (mate, child) in value.as_object().into_iter().flat_map(|pairs| pairs.iter()) {
                // The pair's place in the file: a pair a step made of an older file's
                // `breeds_with` entry is told where the entry has the baby.
                let mut held = component.at.clone();
                held.push(OFFSPRING.1);
                held.push(mate.as_str());
                let pair_at = component.places.place_of(&held);
                // Its mate is that entry's, told by `breeds_with`; a pair of the
                // file's own names its mate here.
                if !mate.as_str().is_empty() && pair_at == held {
                    entity(cx, out, component.origin, pair_at.clone(), mate.as_str());
                }
                if let Some(child) = super::name(child) {
                    entity(cx, out, component.origin, pair_at, child);
                }
            }
        } else if component.key == BREEDS_WITH.0 {
            // The 1.21.130+ map: each key is a mate.
            for (mate, _) in value.as_object().into_iter().flat_map(|mates| mates.iter()) {
                if mate.as_str().is_empty() {
                    continue;
                }
                let mut held = component.at.clone();
                held.push(BREEDS_WITH.1);
                held.push(mate.as_str());
                let mate_at = component.places.place_of(&held);
                entity(cx, out, component.origin, mate_at, mate.as_str());
            }
        }
    });
    sounds(cx, out, entities);
}

/// `name` is an entity identifier, with or without an `<event>` (which is no part of the name
/// looked up); a bare name is the game's. No loader of these members logs a name it lacks,
/// and the game's linked-asset check holds no entity component's reference: the game writes
/// no content-log line for this, so the crate reports it as an information.
fn entity(cx: &LinkCx<'_>, out: &mut Findings, origin: &Origin, at: JsonPath, name: &str) {
    let id = EntityId::parse(name);
    if let Target::Missing(hint) = cx.entity(&id) {
        let at = At::new(origin, at, Severity::Info);
        out.push(unresolved(at, kinds::ENTITY, name, hint));
    }
}

/// A sound a component holds as a plain string is the name of a level sound event, looked up
/// in lower case. The game answers a name it lacks without a line (it logs one only for sound
/// definitions and particle events): the game writes no content-log line for this, so the
/// crate reports it as an information.
fn sounds(cx: &LinkCx<'_>, out: &mut Findings, entities: &Entities<'_>) {
    entities.each(SOUNDS, &mut |component, at, value| {
        let Some(name) = super::name(value) else {
            return;
        };
        if cx
            .level_sound_event(&name.to_ascii_lowercase())
            .is_missing()
        {
            let at = At::new(component.origin, at, Severity::Info);
            out.push(unknown_sound(at, kinds::LEVEL_SOUND_EVENT, name));
        }
    });
}
