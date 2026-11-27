//! The entities and the level sound events an entity's components name: the
//! pair inside the `entity` module (a file `entity.rs` would repeat its directory's name).
use bedrock_addon_core::diagnostic::{JsonPath, Severity};
use bedrock_addon_core::document::Origin;
use bedrock_addon_core::id::EntityId;

use super::walk::Entities;
use crate::families::entity::links::{Reference, ReferenceKind};
use crate::pack::link::prelude::*;

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings, entities: &Entities<'_>) {
    entities.each(ReferenceKind::Entity, &mut |definition, at, found| {
        match &found.reference {
            Reference::Entity { name, .. } => entity(cx, out, definition.origin, at, name),
            Reference::Offspring { mate, child } => {
                // The pair's place in the file: a pair a step made of an older file's
                // `breeds_with` entry is told where the entry has the baby. Its mate is that
                // entry's, told by `breeds_with`; a pair of the file's own names its mate
                // here.
                if let Some(mate) = mate
                    && at == found.at
                {
                    entity(cx, out, definition.origin, at.clone(), mate);
                }
                if let Some(child) = child {
                    entity(cx, out, definition.origin, at, child);
                }
            }
            // The 1.21.130+ map: each key is a mate.
            Reference::Mate(mate) => entity(cx, out, definition.origin, at, mate),
            _ => {}
        }
    });
    sounds(cx, out, entities);
}

/// `name` is an entity identifier, with or without an `<event>` (which is no part of the name
/// looked up); a bare name is the game's. No loader of these members logs a name it lacks,
/// and the game's linked-asset check holds no entity component's reference: the game writes
/// no content-log line for this, so the crate reports it as an information.
fn entity(cx: &LinkCx<'_>, out: &mut Findings, origin: &Origin, at: JsonPath, id: &EntityId) {
    if let Target::Missing(hint) = cx.entity(id) {
        let at = At::new(origin, at, Severity::Info);
        out.push(unresolved(at, kinds::ENTITY, id.as_str(), hint));
    }
}

/// A sound a component holds as a plain string is the name of a level sound event, looked up
/// in lower case. The game answers a name it lacks without a line (it logs one only for sound
/// definitions and particle events): the game writes no content-log line for this, so the
/// crate reports it as an information.
fn sounds(cx: &LinkCx<'_>, out: &mut Findings, entities: &Entities<'_>) {
    entities.each(ReferenceKind::Sound, &mut |definition, at, found| {
        let Reference::Sound(name) = found.reference else {
            return;
        };
        if cx
            .level_sound_event(&name.to_ascii_lowercase())
            .is_missing()
        {
            let at = At::new(definition.origin, at, Severity::Info);
            out.push(unknown_sound(at, kinds::LEVEL_SOUND_EVENT, name));
        }
    });
}
