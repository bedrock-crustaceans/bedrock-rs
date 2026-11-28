//! The emitter components other than the shapes: initialisation, local space, lifetime and
//! rate.

use indexmap::IndexMap;

use super::types::{VectorEvents, events_by, timeline, vector_events};
use crate::codes;
use crate::derived::{KEPT, L0, L1, LEGACY, STRICT, elements, legacy, line, note, object, refuse};
use crate::read::defaults;
use bedrock_addon_core::diagnostic::Code;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

/// A Molang field that is 0 when the document does not write it.
const ZERO: Molang = Molang::Const(0.0);

/// `minecraft:emitter_initialization`: Molang run for the emitter.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report", schema::molang = KEPT)]
pub struct EmitterInitialization {
    /// Run once when the emitter starts (JSON `creation_expression`; default 0).
    #[facet(schema::era(LEGACY, molang = L1))]
    pub creation_expression: Option<Molang>,
    /// Run every emitter update (JSON `per_update_expression`; default 0).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub per_update_expression: Option<Molang>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(EmitterInitialization {
    creation_expression: Molang = "creation_expression", ZERO;
    per_update_expression: Molang = "per_update_expression", ZERO;
});

/// `minecraft:emitter_local_space`: which of the emitter's motions its particles follow.
///
/// The game does not support `rotation` without `position`, and reads `velocity` only
/// when both are `false`; a document that asks for more is loaded as what the game makes
/// of it, with a warning.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report", schema::after_read = local_space)]
pub struct EmitterLocalSpace {
    /// Particles move with the emitter (JSON `position`; default `false`).
    pub position: Option<bool>,
    /// Particles rotate with the emitter (JSON `rotation`; default `false`).
    pub rotation: Option<bool>,
    /// The emitter's velocity is added to new particles (JSON `velocity`; default
    /// `false`).
    pub velocity: Option<bool>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(EmitterLocalSpace {
    position: bool = "position", false;
    rotation: bool = "rotation", false;
    velocity: bool = "velocity", false;
});

/// The coercion both loaders apply.
fn local_space(space: &mut EmitterLocalSpace, node: &Node<'_>, read: &mut Read<'_>) {
    let (position, rotation) = (space.position(), space.rotation());
    if rotation && !position {
        let at = node.path().child("rotation");
        line(read, &at, &codes::ROTATION_WITHOUT_POSITION);
        space.rotation = Some(false);
    }
    // Vanilla stores `velocity` as written only when `position` and `rotation` are both
    // false as written, and says nothing when it does not.
    if space.velocity() && (position || rotation) {
        note(
            read,
            &node.path().child("velocity"),
            &core_codes::INVALID_VALUE,
        );
        space.velocity = Some(false);
    }
}

/// `minecraft:emitter_lifetime_once`: the emitter runs once.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report", schema::molang = KEPT)]
pub struct EmitterLifetimeOnce {
    /// How long the emitter emits, in seconds (JSON `active_time`; default 0).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub active_time: Option<Molang>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(EmitterLifetimeOnce {
    active_time: Molang = "active_time", ZERO;
});

/// `minecraft:emitter_lifetime_looping`: the emitter emits, sleeps, and starts again.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report", schema::molang = KEPT)]
pub struct EmitterLifetimeLooping {
    /// How long the emitter emits each loop, in seconds (JSON `active_time`; default 0).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub active_time: Option<Molang>,
    /// How long it pauses each loop, in seconds (JSON `sleep_time`; default 0).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub sleep_time: Option<Molang>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(EmitterLifetimeLooping {
    active_time: Molang = "active_time", ZERO;
    sleep_time: Molang = "sleep_time", ZERO;
});

/// `minecraft:emitter_lifetime_expression`: Molang decides when the emitter emits.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report", schema::molang = KEPT)]
pub struct EmitterLifetimeExpression {
    /// The emitter emits while this is not 0 (JSON `activation_expression`; default 1).
    #[facet(schema::era(LEGACY, molang = L1))]
    pub activation_expression: Option<Molang>,
    /// The emitter expires when this is not 0 (JSON `expiration_expression`; default 0).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub expiration_expression: Option<Molang>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(EmitterLifetimeExpression {
    activation_expression: Molang = "activation_expression", Molang::Const(1.0);
    expiration_expression: Molang = "expiration_expression", ZERO;
});

/// One entry of `looping_travel_distance_events`: events fired every `distance` blocks
/// the emitter travels.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report")]
pub struct DistanceEvent {
    /// The distance between two firings, 0 or more (JSON `distance`, required).
    #[facet(default = 0.0, schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD))]
    pub distance: f32,
    /// The events fired (JSON `effects`, required).
    #[facet(
        default = VectorEvents::Many(Vec::new()),
        schema::read_with = vector_events,
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub effects: VectorEvents,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

/// `travel_distance_events`, with the legacy parser's line for a value that is no object.
fn travel_distance(node: &Node<'_>, read: &mut Read<'_>) -> Option<IndexMap<String, VectorEvents>> {
    events_by(node, read, &codes::TRAVEL_DISTANCE)
}

/// `looping_travel_distance_events`: the legacy parser's line for a value that is no array is
/// its line for an entry that is no object too.
fn looping(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<DistanceEvent>> {
    let code = Some(&codes::LOOPING);
    elements(node, read, code, |entry, read| object(entry, read, code))
}

/// `minecraft:emitter_lifetime_events`: events the emitter fires over its life.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report", schema::after_read = lifetime_events)]
pub struct EmitterLifetimeEvents {
    /// Fired when the emitter is created (JSON `creation_event`).
    #[facet(schema::read_with = vector_events)]
    pub creation_event: Option<VectorEvents>,
    /// Fired when the emitter expires (JSON `expiration_event`).
    #[facet(schema::read_with = vector_events)]
    pub expiration_event: Option<VectorEvents>,
    /// Events by the emitter's age in seconds, the keys decimal strings (JSON
    /// `timeline`).
    #[facet(schema::read_with = timeline)]
    pub timeline: Option<IndexMap<String, VectorEvents>>,
    /// Events by the distance the emitter has travelled, the keys decimal strings (JSON
    /// `travel_distance_events`).
    #[facet(schema::read_with = travel_distance)]
    pub travel_distance_events: Option<IndexMap<String, VectorEvents>>,
    /// Events fired again every so many blocks (JSON `looping_travel_distance_events`).
    #[facet(schema::read_with = looping)]
    pub looping_travel_distance_events: Option<Vec<DistanceEvent>>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

/// The entry rules of `looping_travel_distance_events`: a negative `distance` refuses a
/// 1.20.80 document; the legacy parser drops an entry with a negative distance or without
/// effects, with its line.
fn lifetime_events(events: &mut EmitterLifetimeEvents, node: &Node<'_>, read: &mut Read<'_>) {
    let Some(entries) = &mut events.looping_travel_distance_events else {
        return;
    };
    let at = node.path().child("looping_travel_distance_events");
    let legacy = legacy(read);
    // The entries are counted as the list's reader kept them: it has recorded where each is
    // in the file, so a line told here is at the file's index. The entries that stay are
    // recorded in their turn.
    let mut index = 0;
    let mut places = Vec::with_capacity(entries.len());
    entries.retain(|entry| {
        let here = at.index(index);
        places.push(index);
        index += 1;
        if legacy {
            let usable = entry.distance >= 0.0 && !entry.effects.is_empty();
            if !usable {
                places.pop();
                line(read, &here, &codes::LOOPING_ENTRY);
            }
            return usable;
        }
        if entry.distance < 0.0 {
            refuse(
                read,
                &here.child("distance"),
                Code::from(&core_codes::INVALID_VALUE),
            );
        }
        true
    });
    read.findings_mut().left_out(&at, &places);
}

/// `minecraft:emitter_rate_instant`: all particles at once.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report", schema::molang = KEPT)]
pub struct EmitterRateInstant {
    /// How many particles are emitted (JSON `num_particles`, required).
    #[facet(
        default = ZERO,
        schema::era(LEGACY, missing = codes::REQUIRED_MISSING, molang = L0),
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub num_particles: Molang,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

/// `minecraft:emitter_rate_steady`: particles at a steady rate.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report", schema::molang = KEPT)]
pub struct EmitterRateSteady {
    /// Particles emitted per second (JSON `spawn_rate`, required).
    #[facet(
        default = ZERO,
        schema::era(LEGACY, missing = codes::REQUIRED_MISSING, molang = L0),
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub spawn_rate: Molang,
    /// The most particles alive at once (JSON `max_particles`, required).
    #[facet(
        default = ZERO,
        schema::era(LEGACY, missing = codes::REQUIRED_MISSING, molang = L0),
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub max_particles: Molang,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

/// `minecraft:emitter_rate_manual`: particles are emitted only when the game asks.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report", schema::molang = KEPT)]
pub struct EmitterRateManual {
    /// The most particles alive at once (JSON `max_particles`, required).
    #[facet(
        default = ZERO,
        schema::era(LEGACY, missing = codes::REQUIRED_MISSING, molang = L0),
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub max_particles: Molang,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}
