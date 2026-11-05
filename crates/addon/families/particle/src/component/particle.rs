//! The per-particle components other than the appearance: initial state, lifetime and motion.

use facet_value::VArray;
use indexmap::IndexMap;

use super::types::VectorEvents;
use crate::read::{Era, Members, Node, Opt, Reader, Writer, object, read_list};
use crate::texts;
use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_core::molang::{Molang, MolangVisitor};
use bedrock_addon_shared_shapes::OneOrMany;

/// A Molang field that is 0 when the document does not write it.
const ZERO: Molang = Molang::Const(0.0);

object! {
    /// `minecraft:particle_initialization`: Molang run for each particle.
    pub struct ParticleInitialization {
        /// Run every particle update (JSON `per_update_expression`; default 0).
        opt per_update_expression: Molang = "per_update_expression", Opt::NONE, default ZERO;
        /// Run every time the particle is drawn (JSON `per_render_expression`; default 0).
        opt per_render_expression: Molang = "per_render_expression", Opt::NONE, default ZERO;
    }
}

object! {
    /// `minecraft:particle_initial_spin`: the rotation a particle starts with.
    pub struct ParticleInitialSpin {
        /// The starting rotation in degrees (JSON `rotation`; default 0).
        opt rotation: Molang = "rotation", Opt::NONE, default ZERO;
        /// The starting spin in degrees per second (JSON `rotation_rate`; default 0).
        opt rotation_rate: Molang = "rotation_rate", Opt::NONE, default ZERO;
    }
}

/// `minecraft:particle_initial_speed`: the speed a particle starts with.
///
/// JSON: the component's value itself is one Molang value (the speed along the direction the
/// emitter shape gives) or an array of three (a velocity that replaces that direction).
#[derive(Clone, Debug, PartialEq)]
pub enum ParticleInitialSpeed {
    /// One speed, along the shape's direction.
    Uniform(Molang),
    /// A velocity `[x, y, z]`, which overrides the shape's direction.
    Vector([Molang; 3]),
}

impl ParticleInitialSpeed {
    /// The speed on each axis: a uniform speed three times, or the vector.
    pub fn speed(&self) -> [Molang; 3] {
        match self {
            Self::Uniform(speed) => [speed.clone(), speed.clone(), speed.clone()],
            Self::Vector(speed) => speed.clone(),
        }
    }

    /// Whether the speed replaces the direction the emitter shape gives: the array form
    /// does (the game keeps this as a flag of its own, which no JSON key sets).
    pub fn overrides_initial_direction(&self) -> bool {
        matches!(self, Self::Vector(_))
    }
}

impl Node for ParticleInitialSpeed {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        if value.is_array() {
            <[Molang; 3]>::read(value, r, o).map(Self::Vector)
        } else {
            <Molang as Node>::read(value, r, o).map(Self::Uniform)
        }
    }

    fn fallback(o: Opt) -> Self {
        Self::Uniform(Molang::fallback(o))
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        match self {
            Self::Uniform(speed) => Node::write(speed, w),
            Self::Vector(speed) => speed.write(w),
        }
    }

    fn walk(&mut self, era: Era, o: Opt, v: &mut MolangVisitor<'_>) {
        match self {
            Self::Uniform(speed) => speed.walk(era, o, v),
            Self::Vector(speed) => speed.walk(era, o, v),
        }
    }
}

object! {
    /// `minecraft:particle_lifetime_expression`: Molang decides how long a particle lives.
    pub struct ParticleLifetimeExpression {
        /// The particle expires when this is not 0 (JSON `expiration_expression`; default 0).
        opt expiration_expression: Molang = "expiration_expression", Opt::NONE, default ZERO;
        /// The particle's lifetime in seconds, evaluated once (JSON `max_lifetime`; no
        /// default).
        opt max_lifetime: Molang = "max_lifetime", Opt::with_default(1.0);
    }
}

object! {
    /// `minecraft:particle_lifetime_events`: events a particle fires over its life.
    pub struct ParticleLifetimeEvents {
        /// Fired when the particle is created (JSON `creation_event`).
        opt creation_event: VectorEvents = "creation_event", Opt::NONE;
        /// Fired when the particle expires (JSON `expiration_event`).
        opt expiration_event: VectorEvents = "expiration_event", Opt::NONE;
        /// Events by the particle's age in seconds, the keys decimal strings (JSON
        /// `timeline`).
        opt timeline: IndexMap<String, VectorEvents> = "timeline",
            Opt::NONE.wrong(texts::TIMELINE);
    }
}

/// A block list: the value of `minecraft:particle_expire_if_in_blocks` and its opposite.
fn read_blocks(value: &Value, r: &mut Reader<'_>) -> Option<Vec<String>> {
    let Some(elements) = value.as_array() else {
        r.wrong_type(value, Some(texts::NO_BLOCK_LIST));
        return None;
    };
    let mut blocks = Vec::with_capacity(elements.len());
    let mut places = Vec::with_capacity(elements.len());
    for (i, element) in elements.iter().enumerate() {
        match element.as_string() {
            Some(name) => {
                blocks.push(name.as_str().to_owned());
                places.push(i);
            }
            None => r.index(i, |r| r.wrong_type(element, Some(texts::BLOCK_NOT_STRING))),
        }
    }
    r.kept(&places);
    Some(blocks)
}

fn write_blocks(blocks: &[String]) -> Value {
    blocks
        .iter()
        .map(|name| Value::from(name.as_str()))
        .collect::<VArray>()
        .into()
}

/// `minecraft:particle_expire_if_in_blocks`: the particle expires inside one of these blocks.
///
/// JSON: the component's value itself is an array of block names.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParticleExpireIfInBlocks(pub Vec<String>);

impl Node for ParticleExpireIfInBlocks {
    fn read(value: &Value, r: &mut Reader<'_>, _: Opt) -> Option<Self> {
        read_blocks(value, r).map(Self)
    }

    fn fallback(_: Opt) -> Self {
        Self::default()
    }

    fn write(&self, _: &mut Writer) -> Result<Value, EncodeError> {
        Ok(write_blocks(&self.0))
    }
}

/// `minecraft:particle_expire_if_not_in_blocks`: the particle expires outside these blocks.
///
/// JSON: the component's value itself is an array of block names.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParticleExpireIfNotInBlocks(pub Vec<String>);

impl Node for ParticleExpireIfNotInBlocks {
    fn read(value: &Value, r: &mut Reader<'_>, _: Opt) -> Option<Self> {
        read_blocks(value, r).map(Self)
    }

    fn fallback(_: Opt) -> Self {
        Self::default()
    }

    fn write(&self, _: &mut Writer) -> Result<Value, EncodeError> {
        Ok(write_blocks(&self.0))
    }
}

/// `minecraft:particle_kill_plane`: the particle expires when it crosses a plane.
///
/// JSON: the component's value itself is `[a, b, c, d]`, the plane `ax + by + cz + d = 0` in
/// the emitter's space.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ParticleKillPlane(pub [f32; 4]);

impl Node for ParticleKillPlane {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        <[f32; 4]>::read(value, r, o).map(Self)
    }

    fn fallback(_: Opt) -> Self {
        Self::default()
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        self.0.write(w)
    }
}

object! {
    /// `minecraft:particle_motion_dynamic`: the particle moves under acceleration and drag.
    pub struct ParticleMotionDynamic {
        /// The acceleration `[x, y, z]` in blocks per second squared (JSON
        /// `linear_acceleration`; default `[0, 0, 0]`).
        opt linear_acceleration: [Molang; 3] = "linear_acceleration", Opt::NONE,
            default [ZERO, ZERO, ZERO];
        /// The drag on the velocity (JSON `linear_drag_coefficient`; default 0).
        opt linear_drag_coefficient: Molang = "linear_drag_coefficient", Opt::NONE,
            default ZERO;
        /// The acceleration of the spin in degrees per second squared (JSON
        /// `rotation_acceleration`; default 0).
        opt rotation_acceleration: Molang = "rotation_acceleration", Opt::NONE, default ZERO;
        /// The drag on the spin (JSON `rotation_drag_coefficient`; default 0).
        opt rotation_drag_coefficient: Molang = "rotation_drag_coefficient", Opt::NONE,
            default ZERO;
    }
}

object! {
    /// `minecraft:particle_motion_parametric`: Molang gives the particle's position and
    /// rotation directly.
    pub struct ParticleMotionParametric {
        /// The position relative to the emitter (JSON `relative_position`).
        opt relative_position: [Molang; 3] = "relative_position", Opt::NONE;
        /// The direction, for particles that face it (JSON `direction`).
        opt direction: [Molang; 3] = "direction", Opt::NONE;
        /// The rotation in degrees (JSON `rotation`).
        opt rotation: Molang = "rotation", Opt::NONE;
    }
}

object! {
    /// One entry of `minecraft:particle_motion_collision/events`: an event fired when the
    /// particle hits something fast enough.
    pub struct CollisionEvent finish = collision_event {
        /// The event fired (JSON `event`, required).
        req event: String = "event", Opt::NONE.missing(texts::NO_EVENT_FIELD);
        /// The least speed of the hit, in blocks per second (JSON `min_speed`; default 2).
        opt min_speed: f32 = "min_speed", Opt::with_default(2.0), default 2.0;
    }
}

/// The least `min_speed` the legacy parser stores.
const LEGACY_MIN_SPEED: f32 = 2.0;

/// The legacy parser stores `max(min_speed, 2)`, and 2 for a value that is no number. The
/// crate says so at the information level when this changes what the document writes.
fn collision_event(event: &mut CollisionEvent, m: Members<'_>, r: &mut Reader<'_>) {
    let written = m.get("min_speed").is_some();
    if r.legacy() && written && event.min_speed.is_none_or(|speed| speed < LEGACY_MIN_SPEED) {
        event.min_speed = Some(LEGACY_MIN_SPEED);
        r.key("min_speed", |r| {
            r.push(DiagnosticCode::InvalidValue { text: None }, Severity::Info);
        });
    }
}

/// `minecraft:particle_motion_collision/events`: one event or a list.
///
/// JSON: an object, or an array of objects.
pub type CollisionEvents = OneOrMany<CollisionEvent>;

impl Node for CollisionEvents {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        let element = o.wrong(texts::EVENT_NOT_PAIR);
        // The legacy parser leaves out an entry without an `event` string.
        let usable = |event: &CollisionEvent, legacy: bool| !legacy || !event.event.is_empty();
        // An entry that is left out is noted where it is, at the information level: the
        // game's line at its `event` does not say that the entry's other members go with it.
        let named = |entry: &Value| {
            entry
                .as_object()
                .and_then(|entry| entry.get("event"))
                .and_then(Value::as_string)
                .is_some_and(|name| !name.as_str().is_empty())
        };
        let left_out = |r: &mut Reader<'_>| {
            r.push(DiagnosticCode::DroppedField { text: None }, Severity::Info);
        };
        if let Some(entries) = value.as_array() {
            let legacy = r.legacy();
            let (events, places) = read_list::<CollisionEvent>(value, r, element)?;
            if legacy {
                for (index, entry) in entries.iter().enumerate() {
                    if entry.is_object() && !named(entry) {
                        r.index(index, left_out);
                    }
                }
            }
            // The events that stay, each with the index the file has it at.
            let (events, places): (Vec<_>, Vec<_>) = events
                .into_iter()
                .zip(places)
                .filter(|(event, _)| usable(event, legacy))
                .unzip();
            r.kept(&places);
            return Some(Self::Many(events));
        }
        if value.is_object() {
            let event = CollisionEvent::read(value, r, element)?;
            if r.legacy() && !named(value) {
                left_out(r);
            }
            return usable(&event, r.legacy()).then_some(Self::One(event));
        }
        r.wrong_type(value, Some(texts::EVENTS_NOT_PAIRS));
        None
    }

    fn fallback(_: Opt) -> Self {
        Self::Many(Vec::new())
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        match self {
            Self::One(event) => event.write(w),
            Self::Many(events) => events.write(w),
        }
    }
}

object! {
    /// `minecraft:particle_motion_collision`: the particle collides with blocks.
    pub struct ParticleMotionCollision finish = collision {
        /// Collision is on while this is not 0 (JSON `enabled`; default 1).
        opt enabled: Molang = "enabled", Opt::with_default(1.0), default Molang::Const(1.0);
        /// How fast the particle slows down while it touches a block, in blocks per second
        /// squared (JSON `collision_drag`; default 0).
        opt collision_drag: f32 = "collision_drag", Opt::NONE.wrong(texts::DRAG_NOT_FLOAT),
            default 0.0;
        /// The bounciness, 0 to 1 (JSON `coefficient_of_restitution`; default 0).
        opt coefficient_of_restitution: f32 = "coefficient_of_restitution",
            Opt::NONE.wrong(texts::RESTITUTION_NOT_FLOAT), default 0.0;
        /// The radius the particle collides with, 0 to 0.5 (JSON `collision_radius`,
        /// required).
        req collision_radius: f32 = "collision_radius",
            Opt::required(0.0).wrong(texts::RADIUS_NOT_FLOAT);
        /// The particle expires when it hits something (JSON `expire_on_contact`; default
        /// `false`).
        opt expire_on_contact: bool = "expire_on_contact",
            Opt::NONE.wrong(texts::EXPIRE_NOT_BOOL), default false;
        /// Events fired on a hit (JSON `events`).
        opt events: CollisionEvents = "events", Opt::NONE;
    }
}

/// The radius is 0 to 0.5: the legacy parser clamps it, and a 1.20.80 document with a radius
/// outside the range is refused.
fn collision(component: &mut ParticleMotionCollision, _: Members<'_>, r: &mut Reader<'_>) {
    let radius = component.collision_radius;
    if !(0.0..=0.5).contains(&radius) {
        r.key("collision_radius", |r| {
            if r.legacy() {
                r.warn(if radius < 0.0 {
                    texts::RADIUS_TOO_SMALL
                } else {
                    texts::RADIUS_TOO_LARGE
                });
            } else {
                r.error(DiagnosticCode::InvalidValue { text: None });
            }
        });
        if r.legacy() {
            component.collision_radius = radius.clamp(0.0, 0.5);
        }
    }
}
