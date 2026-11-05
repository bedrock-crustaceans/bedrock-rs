//! The emitter components other than the shapes: initialisation, local space, lifetime and
//! rate.

use indexmap::IndexMap;

use super::types::VectorEvents;
use crate::read::{Members, Opt, Reader, object};
use crate::texts;
use bedrock_addon_core::diagnostic::DiagnosticCode;
use bedrock_addon_core::molang::Molang;

/// A Molang field that is 0 when the document does not write it.
const ZERO: Molang = Molang::Const(0.0);

object! {
    /// `minecraft:emitter_initialization`: Molang run for the emitter.
    pub struct EmitterInitialization {
        /// Run once when the emitter starts (JSON `creation_expression`; default 0).
        opt creation_expression: Molang = "creation_expression", Opt::with_default(1.0), default ZERO;
        /// Run every emitter update (JSON `per_update_expression`; default 0).
        opt per_update_expression: Molang = "per_update_expression", Opt::NONE, default ZERO;
    }
}

object! {
    /// `minecraft:emitter_local_space`: which of the emitter's motions its particles follow.
    ///
    /// The game does not support `rotation` without `position`, and reads `velocity` only
    /// when both are `false`; a document that asks for more is loaded as what the game makes
    /// of it, with a warning.
    pub struct EmitterLocalSpace finish = local_space {
        /// Particles move with the emitter (JSON `position`; default `false`).
        opt position: bool = "position", Opt::NONE, default false;
        /// Particles rotate with the emitter (JSON `rotation`; default `false`).
        opt rotation: bool = "rotation", Opt::NONE, default false;
        /// The emitter's velocity is added to new particles (JSON `velocity`; default
        /// `false`).
        opt velocity: bool = "velocity", Opt::NONE, default false;
    }
}

/// The coercion both loaders apply.
fn local_space(space: &mut EmitterLocalSpace, _: Members<'_>, r: &mut Reader<'_>) {
    let (position, rotation) = (space.position(), space.rotation());
    if rotation && !position {
        r.key("rotation", |r| r.warn(texts::ROTATION_WITHOUT_POSITION));
        space.rotation = Some(false);
    }
    // Vanilla stores `velocity` as written only when `position` and `rotation` are both
    // false as written, and says nothing when it does not.
    if space.velocity() && (position || rotation) {
        r.key("velocity", Reader::warn_ours);
        space.velocity = Some(false);
    }
}

object! {
    /// `minecraft:emitter_lifetime_once`: the emitter runs once.
    pub struct EmitterLifetimeOnce {
        /// How long the emitter emits, in seconds (JSON `active_time`; default 0).
        opt active_time: Molang = "active_time", Opt::NONE, default ZERO;
    }
}

object! {
    /// `minecraft:emitter_lifetime_looping`: the emitter emits, sleeps, and starts again.
    pub struct EmitterLifetimeLooping {
        /// How long the emitter emits each loop, in seconds (JSON `active_time`; default 0).
        opt active_time: Molang = "active_time", Opt::NONE, default ZERO;
        /// How long it pauses each loop, in seconds (JSON `sleep_time`; default 0).
        opt sleep_time: Molang = "sleep_time", Opt::NONE, default ZERO;
    }
}

object! {
    /// `minecraft:emitter_lifetime_expression`: Molang decides when the emitter emits.
    pub struct EmitterLifetimeExpression {
        /// The emitter emits while this is not 0 (JSON `activation_expression`; default 1).
        opt activation_expression: Molang = "activation_expression", Opt::with_default(1.0),
            default Molang::Const(1.0);
        /// The emitter expires when this is not 0 (JSON `expiration_expression`; default 0).
        opt expiration_expression: Molang = "expiration_expression", Opt::NONE, default ZERO;
    }
}

object! {
    /// One entry of `looping_travel_distance_events`: events fired every `distance` blocks
    /// the emitter travels.
    pub struct DistanceEvent {
        /// The distance between two firings, 0 or more (JSON `distance`, required).
        req distance: f32 = "distance", Opt::NONE;
        /// The events fired (JSON `effects`, required).
        req effects: VectorEvents = "effects", Opt::NONE;
    }
}

object! {
    /// `minecraft:emitter_lifetime_events`: events the emitter fires over its life.
    pub struct EmitterLifetimeEvents finish = lifetime_events {
        /// Fired when the emitter is created (JSON `creation_event`).
        opt creation_event: VectorEvents = "creation_event", Opt::NONE;
        /// Fired when the emitter expires (JSON `expiration_event`).
        opt expiration_event: VectorEvents = "expiration_event", Opt::NONE;
        /// Events by the emitter's age in seconds, the keys decimal strings (JSON
        /// `timeline`).
        opt timeline: IndexMap<String, VectorEvents> = "timeline",
            Opt::NONE.wrong(texts::TIMELINE);
        /// Events by the distance the emitter has travelled, the keys decimal strings (JSON
        /// `travel_distance_events`).
        opt travel_distance_events: IndexMap<String, VectorEvents> = "travel_distance_events",
            Opt::NONE.wrong(texts::TRAVEL_DISTANCE);
        /// Events fired again every so many blocks (JSON `looping_travel_distance_events`).
        opt looping_travel_distance_events: Vec<DistanceEvent> = "looping_travel_distance_events",
            Opt::NONE.wrong(texts::LOOPING);
    }
}

/// The entry rules of `looping_travel_distance_events`: a negative `distance` refuses a
/// 1.20.80 document; the legacy parser drops an entry with a negative distance or without
/// effects, with its line.
fn lifetime_events(events: &mut EmitterLifetimeEvents, _: Members<'_>, r: &mut Reader<'_>) {
    let Some(entries) = &mut events.looping_travel_distance_events else {
        return;
    };
    r.key("looping_travel_distance_events", |r| {
        // The entries are counted as the list's reader kept them: it has recorded where
        // each is in the file, so a line told here is at the file's index. The entries
        // that stay are recorded in their turn.
        let mut index = 0;
        let mut places = Vec::with_capacity(entries.len());
        entries.retain(|entry| {
            let at = index;
            index += 1;
            if r.legacy() {
                let usable = entry.distance >= 0.0 && !entry.effects.is_empty();
                if !usable {
                    r.index(at, |r| r.warn(texts::LOOPING_ENTRY));
                }
                if usable {
                    places.push(at);
                }
                return usable;
            }
            places.push(at);
            if entry.distance < 0.0 {
                r.index(at, |r| {
                    r.key("distance", |r| {
                        r.error(DiagnosticCode::InvalidValue { text: None });
                    });
                });
            }
            true
        });
        r.kept(&places);
    });
}

object! {
    /// `minecraft:emitter_rate_instant`: all particles at once.
    pub struct EmitterRateInstant {
        /// How many particles are emitted (JSON `num_particles`, required).
        req num_particles: Molang = "num_particles", Opt::required(0.0);
    }
}

object! {
    /// `minecraft:emitter_rate_steady`: particles at a steady rate.
    pub struct EmitterRateSteady {
        /// Particles emitted per second (JSON `spawn_rate`, required).
        req spawn_rate: Molang = "spawn_rate", Opt::required(0.0);
        /// The most particles alive at once (JSON `max_particles`, required).
        req max_particles: Molang = "max_particles", Opt::required(0.0);
    }
}

object! {
    /// `minecraft:emitter_rate_manual`: particles are emitted only when the game asks.
    pub struct EmitterRateManual {
        /// The most particles alive at once (JSON `max_particles`, required).
        req max_particles: Molang = "max_particles", Opt::required(0.0);
    }
}
