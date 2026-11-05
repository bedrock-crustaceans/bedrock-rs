//! A particle effect's references inside the particle family: the events its components fire
//! are events of the effect itself, and the effect an event starts is a particle effect of the
//! stack.
//!
//! The game looks an event up by name when a component fires it and an effect when the event
//! runs; nothing is said at load. A name that resolves to nothing is told here as an information,
//! at the member that holds it, with the name as written: particle identifiers and event names
//! are compared byte for byte.
//!
//! The crate has no list of the game's own particle effects: without a host that answers for
//! the kind `particle`, an effect only the game's resource pack defines is missing unless that
//! pack is on the stack.
// `VectorEvents` names are checked against the effect's own `events`, and
// `particle_effect.effect` against the particle registry; identifiers are registered as
// written, with no case folding.

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::{Document, Origin};
use bedrock_addon_core::id::ParticleId;

use super::super::{At, Findings, LinkCx, Target, kinds, unresolved};
use crate::families::particle::component::{
    EmitterLifetimeEvents, ParticleLifetimeEvents, ParticleMotionCollision, VectorEvents,
};
use crate::families::particle::{EventNode, ParticleEffect};

/// What a finding calls an event of a particle effect (a key of its `events`): the crate has
/// no shared kind for it.
pub(super) const EVENT: &str = "particle event";

/// The path of the payload: every member of an effect is below `particle_effect`.
pub(super) fn payload() -> JsonPath {
    let mut path = JsonPath::new();
    path.push("particle_effect");
    path
}

/// The path of the component `name` of an effect.
pub(super) fn component(name: &str) -> JsonPath {
    let mut path = payload();
    path.push("components");
    path.push(name);
    path
}

/// Calls `visit` with every event node of the effect `document` and the node's path in the
/// file: each event of `events` in order, a node before the nodes of its `sequence`, those
/// before the options of its `randomize`. The walk keeps its own stack: an event tree built
/// in memory may be deeper than the call stack.
///
/// The walk counts the nodes the effect holds; the path given is the one the file has. The
/// reader of an effect below 1.20.80 leaves out an element it cannot use, and the document
/// keeps where each node it holds is in the file.
pub(super) fn each_event_node<'e>(
    document: &'e Document<ParticleEffect>,
    mut visit: impl FnMut(&JsonPath, &'e EventNode),
) {
    let effect = document.value();
    let places = document.diagnostics();
    let mut events_at = payload();
    events_at.push("events");
    let mut waiting: Vec<(JsonPath, &'e EventNode)> = Vec::new();
    for (name, event) in effect.events.iter().flatten() {
        let at = events_at.child(name);
        waiting.push((at, event));
        while let Some((at, node)) = waiting.pop() {
            visit(&places.place_of(&at), node);
            // Pushed last first, so that the first child is the next one visited.
            let below = |member: &str, index: usize| {
                let mut child = at.clone();
                child.push(member);
                child.push_index(index);
                child
            };
            let options = node.randomize.as_deref().unwrap_or_default();
            for (index, option) in options.iter().enumerate().rev() {
                waiting.push((below("randomize", index), &option.node));
            }
            let sequence = node.sequence.as_deref().unwrap_or_default();
            for (index, child) in sequence.iter().enumerate().rev() {
                waiting.push((below("sequence", index), child));
            }
        }
    }
}

/// The events a component fires: each name of `minecraft:emitter_lifetime_events`
/// (`creation_event`, `expiration_event`, `timeline`, `travel_distance_events`,
/// `looping_travel_distance_events[].effects`), of `minecraft:particle_lifetime_events`
/// (`creation_event`, `expiration_event`, `timeline`) and of
/// `minecraft:particle_motion_collision` (`events[].event`) is a key of the effect's own
/// `events`, byte for byte. A name that is none is an `UnresolvedReference` information at the
/// name. An empty name is no reference: the load has told the member it stands for.
pub(super) fn own_events(cx: &LinkCx<'_>, out: &mut Findings) {
    for (_, document) in cx.resource().particles() {
        let effect = document.value();
        let mut check = OwnEvents {
            effect,
            origin: document.origin(),
            places: document.diagnostics(),
            out: &mut *out,
        };
        if let Some(events) = effect.components.get::<EmitterLifetimeEvents>() {
            let at = component("minecraft:emitter_lifetime_events");
            check.member(&at, "creation_event", events.creation_event.as_ref());
            check.member(&at, "expiration_event", events.expiration_event.as_ref());
            check.keyed(&at, "timeline", events.timeline.as_ref());
            let travelled = events.travel_distance_events.as_ref();
            check.keyed(&at, "travel_distance_events", travelled);
            let looping = at.child("looping_travel_distance_events");
            let entries = events.looping_travel_distance_events.iter().flatten();
            for (index, entry) in entries.enumerate() {
                let entry_at = looping.index(index);
                check.member(&entry_at, "effects", Some(&entry.effects));
            }
        }
        if let Some(events) = effect.components.get::<ParticleLifetimeEvents>() {
            let at = component("minecraft:particle_lifetime_events");
            check.member(&at, "creation_event", events.creation_event.as_ref());
            check.member(&at, "expiration_event", events.expiration_event.as_ref());
            check.keyed(&at, "timeline", events.timeline.as_ref());
        }
        if let Some(collision) = effect.components.get::<ParticleMotionCollision>()
            && let Some(events) = &collision.events
        {
            use crate::families::particle::component::CollisionEvents;
            let mut at = component("minecraft:particle_motion_collision");
            at.push("events");
            for (index, event) in events.events().iter().enumerate() {
                let mut name_at = at.clone();
                // One event is written as an object, a list as an array of objects.
                if matches!(events, CollisionEvents::Many(_)) {
                    name_at.push_index(index);
                }
                name_at.push("event");
                check.name(&name_at, &event.event);
            }
        }
    }
}

/// The event names of one effect's components, held to the effect's `events`.
#[derive(Debug)]
struct OwnEvents<'r, 'o> {
    effect: &'r ParticleEffect,
    origin: &'r Origin,
    /// The document's findings: they give the place in the file of a member the effect
    /// holds, whose lists are counted as held.
    places: &'r Diagnostics,
    out: &'o mut Findings,
}

impl OwnEvents<'_, '_> {
    /// One name, at `at` as the effect holds it; told at the place the file has.
    fn name(&mut self, at: &JsonPath, name: &str) {
        if name.is_empty() {
            return;
        }
        let defined = self
            .effect
            .events
            .as_ref()
            .is_some_and(|events| events.contains_key(name));
        if !defined {
            // The game fires nothing for a name the effect lacks and logs nothing (vanilla's
            // `obsidian_tear` fires `hit_ground` with no `events`): the crate reports an
            // information.
            let at = At::new(self.origin, self.places.place_of(at), Severity::Info);
            self.out.push(unresolved(at, EVENT, name, None));
        }
    }

    /// The names of the member `key` below `parent`: at the member when it is one string, at
    /// each element when it is an array.
    fn member(&mut self, parent: &JsonPath, key: &str, events: Option<&VectorEvents>) {
        let Some(events) = events else {
            return;
        };
        let at = parent.child(key);
        match events {
            VectorEvents::One(name) => self.name(&at, name),
            VectorEvents::Many(names) => {
                for (index, name) in names.iter().enumerate() {
                    let element = at.index(index);
                    self.name(&element, name);
                }
            }
        }
    }

    /// The names of each entry of the map `key` below `parent` (a timeline).
    fn keyed(
        &mut self,
        parent: &JsonPath,
        key: &str,
        map: Option<&indexmap::IndexMap<String, VectorEvents>>,
    ) {
        let at = parent.child(key);
        for (time, events) in map.into_iter().flatten() {
            self.member(&at, time, Some(events));
        }
    }
}

/// The effect an event starts: `particle_effect.effect` of every event node names a
/// particle effect of the resource stack (or one the host says the game has), byte for byte.
/// One that is none is an `UnresolvedReference` information at the member, with the near miss when
/// an effect differs from the name only in case. An empty name is no reference: the load has
/// told the member it stands for.
pub(super) fn effects(cx: &LinkCx<'_>, out: &mut Findings) {
    for (_, document) in cx.resource().particles() {
        each_event_node(document, |at, node| {
            let Some(started) = &node.particle_effect else {
                return;
            };
            if started.effect.is_empty() {
                return;
            }
            let id = ParticleId::parse(started.effect.as_str());
            if let Target::Missing(hint) = cx.particle(&id) {
                let mut at = at.clone();
                at.push("particle_effect");
                at.push("effect");
                // The event node stores the name and resolves only the sound; the game writes no
                // content-log line and the linked-asset check holds no reference, so the crate
                // reports an information.
                let at = At::new(document.origin(), at, Severity::Info);
                out.push(unresolved(at, kinds::PARTICLE, &started.effect, hint));
            }
        });
    }
}
