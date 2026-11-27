//! The per-particle components other than the appearance: initial state, lifetime and motion.

use indexmap::IndexMap;

use super::types::{VectorEvents, timeline};
use crate::codes;
use crate::derived::{L0, L1, LEGACY, STRICT, hand, write_derived};
use crate::read::{
    Era, KEPT, Node as HandNode, Opt, Reader, Writer, defaults, derived_node, read_list,
};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Severity};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_core::molang::{Molang, MolangVisitor};
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_shared_shapes::OneOrMany;
use facet::Facet;

/// A Molang field that is 0 when the document does not write it.
const ZERO: Molang = Molang::Const(0.0);

/// `minecraft:particle_initialization`: Molang run for each particle.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report", schema::molang = KEPT)]
pub struct ParticleInitialization {
    /// Run every particle update (JSON `per_update_expression`; default 0).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub per_update_expression: Option<Molang>,
    /// Run every time the particle is drawn (JSON `per_render_expression`; default 0).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub per_render_expression: Option<Molang>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(ParticleInitialization {
    per_update_expression: Molang = "per_update_expression", ZERO;
    per_render_expression: Molang = "per_render_expression", ZERO;
});

/// `minecraft:particle_initial_spin`: the rotation a particle starts with.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report", schema::molang = KEPT)]
pub struct ParticleInitialSpin {
    /// The starting rotation in degrees (JSON `rotation`; default 0).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub rotation: Option<Molang>,
    /// The starting spin in degrees per second (JSON `rotation_rate`; default 0).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub rotation_rate: Option<Molang>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(ParticleInitialSpin {
    rotation: Molang = "rotation", ZERO;
    rotation_rate: Molang = "rotation_rate", ZERO;
});

/// `minecraft:particle_initial_speed`: the speed a particle starts with.
///
/// JSON: the component's value itself is one Molang value (the speed along the direction the
/// emitter shape gives) or an array of three (a velocity that replaces that direction).
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
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

impl HandNode for ParticleInitialSpeed {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        if value.is_array() {
            <[Molang; 3]>::read(value, r, o).map(Self::Vector)
        } else {
            <Molang as HandNode>::read(value, r, o).map(Self::Uniform)
        }
    }

    fn fallback(o: Opt) -> Self {
        Self::Uniform(Molang::fallback(o))
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        write_derived(self, w)
    }

    fn walk(&mut self, era: Era, o: Opt, v: &mut MolangVisitor<'_>) {
        match self {
            Self::Uniform(speed) => speed.walk(era, o, v),
            Self::Vector(speed) => speed.walk(era, o, v),
        }
    }
}

/// `minecraft:particle_lifetime_expression`: Molang decides how long a particle lives.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report", schema::molang = KEPT)]
pub struct ParticleLifetimeExpression {
    /// The particle expires when this is not 0 (JSON `expiration_expression`; default 0).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub expiration_expression: Option<Molang>,
    /// The particle's lifetime in seconds, evaluated once (JSON `max_lifetime`; no
    /// default).
    #[facet(schema::era(LEGACY, molang = L1))]
    pub max_lifetime: Option<Molang>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(ParticleLifetimeExpression {
    expiration_expression: Molang = "expiration_expression", ZERO;
});

/// `minecraft:particle_lifetime_events`: events a particle fires over its life.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report")]
pub struct ParticleLifetimeEvents {
    /// Fired when the particle is created (JSON `creation_event`).
    #[facet(schema::read_with = hand)]
    pub creation_event: Option<VectorEvents>,
    /// Fired when the particle expires (JSON `expiration_event`).
    #[facet(schema::read_with = hand)]
    pub expiration_event: Option<VectorEvents>,
    /// Events by the particle's age in seconds, the keys decimal strings (JSON
    /// `timeline`).
    #[facet(schema::read_with = timeline)]
    pub timeline: Option<IndexMap<String, VectorEvents>>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

/// A block list: the value of `minecraft:particle_expire_if_in_blocks` and its opposite.
fn read_blocks(value: &Value, r: &mut Reader<'_>) -> Option<Vec<String>> {
    let Some(elements) = value.as_array() else {
        r.wrong_type(value, Some(&codes::NO_BLOCK_LIST));
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
            None => r.index(i, |r| r.wrong_type(element, Some(&codes::BLOCK_NOT_STRING))),
        }
    }
    r.kept(&places);
    Some(blocks)
}

/// `minecraft:particle_expire_if_in_blocks`: the particle expires inside one of these blocks.
///
/// JSON: the component's value itself is an array of block names.
#[derive(Clone, Debug, Default, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct ParticleExpireIfInBlocks(pub Vec<String>);

impl HandNode for ParticleExpireIfInBlocks {
    fn read(value: &Value, r: &mut Reader<'_>, _: Opt) -> Option<Self> {
        read_blocks(value, r).map(Self)
    }

    fn fallback(_: Opt) -> Self {
        Self::default()
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        write_derived(self, w)
    }
}

/// `minecraft:particle_expire_if_not_in_blocks`: the particle expires outside these blocks.
///
/// JSON: the component's value itself is an array of block names.
#[derive(Clone, Debug, Default, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct ParticleExpireIfNotInBlocks(pub Vec<String>);

impl HandNode for ParticleExpireIfNotInBlocks {
    fn read(value: &Value, r: &mut Reader<'_>, _: Opt) -> Option<Self> {
        read_blocks(value, r).map(Self)
    }

    fn fallback(_: Opt) -> Self {
        Self::default()
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        write_derived(self, w)
    }
}

/// `minecraft:particle_kill_plane`: the particle expires when it crosses a plane.
///
/// JSON: the component's value itself is `[a, b, c, d]`, the plane `ax + by + cz + d = 0` in
/// the emitter's space.
#[derive(Clone, Copy, Debug, Default, PartialEq, Facet)]
#[facet(transparent)]
pub struct ParticleKillPlane(pub [f32; 4]);

impl HandNode for ParticleKillPlane {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        <[f32; 4]>::read(value, r, o).map(Self)
    }

    fn fallback(_: Opt) -> Self {
        Self::default()
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        write_derived(self, w)
    }
}

/// `minecraft:particle_motion_dynamic`: the particle moves under acceleration and drag.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report", schema::molang = KEPT)]
pub struct ParticleMotionDynamic {
    /// The acceleration `[x, y, z]` in blocks per second squared (JSON
    /// `linear_acceleration`; default `[0, 0, 0]`).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub linear_acceleration: Option<[Molang; 3]>,
    /// The drag on the velocity (JSON `linear_drag_coefficient`; default 0).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub linear_drag_coefficient: Option<Molang>,
    /// The acceleration of the spin in degrees per second squared (JSON
    /// `rotation_acceleration`; default 0).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub rotation_acceleration: Option<Molang>,
    /// The drag on the spin (JSON `rotation_drag_coefficient`; default 0).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub rotation_drag_coefficient: Option<Molang>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(ParticleMotionDynamic {
    linear_acceleration: [Molang; 3] = "linear_acceleration", [ZERO, ZERO, ZERO];
    linear_drag_coefficient: Molang = "linear_drag_coefficient", ZERO;
    rotation_acceleration: Molang = "rotation_acceleration", ZERO;
    rotation_drag_coefficient: Molang = "rotation_drag_coefficient", ZERO;
});

/// `minecraft:particle_motion_parametric`: Molang gives the particle's position and
/// rotation directly.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report", schema::molang = KEPT)]
pub struct ParticleMotionParametric {
    /// The position relative to the emitter (JSON `relative_position`).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub relative_position: Option<[Molang; 3]>,
    /// The direction, for particles that face it (JSON `direction`).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub direction: Option<[Molang; 3]>,
    /// The rotation in degrees (JSON `rotation`).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub rotation: Option<Molang>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

/// One entry of `minecraft:particle_motion_collision/events`: an event fired when the
/// particle hits something fast enough.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report", schema::after_read = collision_event)]
pub struct CollisionEvent {
    /// The event fired (JSON `event`, required).
    #[facet(
        default,
        schema::era(LEGACY, missing = codes::NO_EVENT_FIELD),
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub event: String,
    /// The least speed of the hit, in blocks per second (JSON `min_speed`; default 2).
    pub min_speed: Option<f32>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(CollisionEvent {
    min_speed: f32 = "min_speed", 2.0;
});

/// The least `min_speed` the legacy parser stores.
const LEGACY_MIN_SPEED: f32 = 2.0;

/// The legacy parser stores `max(min_speed, 2)`, and 2 for a value that is no number. The
/// crate says so at the information level when this changes what the document writes.
fn collision_event(event: &mut CollisionEvent, node: &Node<'_>, read: &mut Read<'_>) {
    let written = member(node, "min_speed");
    let r = &mut Reader::of(node, read);
    if r.legacy() && written && event.min_speed.is_none_or(|speed| speed < LEGACY_MIN_SPEED) {
        event.min_speed = Some(LEGACY_MIN_SPEED);
        r.key("min_speed", |r| {
            r.push(Code::from(&core_codes::INVALID_VALUE), Severity::Info);
        });
    }
}

/// `minecraft:particle_motion_collision/events`: one event or a list.
///
/// JSON: an object, or an array of objects.
pub type CollisionEvents = OneOrMany<CollisionEvent>;

impl HandNode for CollisionEvents {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        let element = o.wrong(&codes::EVENT_NOT_PAIR);
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
            r.push(Code::from(&core_codes::DROPPED_FIELD), Severity::Info);
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
        r.wrong_type(value, Some(&codes::EVENTS_NOT_PAIRS));
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

/// `minecraft:particle_motion_collision`: the particle collides with blocks.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(
    schema::unknown = "report",
    schema::molang = KEPT,
    schema::after_read = collision
)]
pub struct ParticleMotionCollision {
    /// Collision is on while this is not 0 (JSON `enabled`; default 1).
    #[facet(schema::era(LEGACY, molang = L1))]
    pub enabled: Option<Molang>,
    /// How fast the particle slows down while it touches a block, in blocks per second
    /// squared (JSON `collision_drag`; default 0).
    #[facet(schema::era(LEGACY, wrong_type = codes::DRAG_NOT_FLOAT))]
    pub collision_drag: Option<f32>,
    /// The bounciness, 0 to 1 (JSON `coefficient_of_restitution`; default 0).
    #[facet(schema::era(LEGACY, wrong_type = codes::RESTITUTION_NOT_FLOAT))]
    pub coefficient_of_restitution: Option<f32>,
    /// The radius the particle collides with, 0 to 0.5 (JSON `collision_radius`,
    /// required).
    #[facet(
        default = 0.0,
        schema::era(
            LEGACY,
            wrong_type = codes::RADIUS_NOT_FLOAT,
            missing = codes::REQUIRED_MISSING
        ),
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub collision_radius: f32,
    /// The particle expires when it hits something (JSON `expire_on_contact`; default
    /// `false`).
    #[facet(schema::era(LEGACY, wrong_type = codes::EXPIRE_NOT_BOOL))]
    pub expire_on_contact: Option<bool>,
    /// Events fired on a hit (JSON `events`).
    #[facet(schema::read_with = hand)]
    pub events: Option<CollisionEvents>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(ParticleMotionCollision {
    enabled: Molang = "enabled", Molang::Const(1.0);
    collision_drag: f32 = "collision_drag", 0.0;
    coefficient_of_restitution: f32 = "coefficient_of_restitution", 0.0;
    expire_on_contact: bool = "expire_on_contact", false;
});

/// The radius is 0 to 0.5: the legacy parser clamps it, and a 1.20.80 document with a radius
/// outside the range is refused.
fn collision(component: &mut ParticleMotionCollision, node: &Node<'_>, read: &mut Read<'_>) {
    let r = &mut Reader::of(node, read);
    let radius = component.collision_radius;
    if !(0.0..=0.5).contains(&radius) {
        r.key("collision_radius", |r| {
            if r.legacy() {
                r.warn(if radius < 0.0 {
                    &codes::RADIUS_TOO_SMALL
                } else {
                    &codes::RADIUS_TOO_LARGE
                });
            } else {
                r.error(Code::from(&core_codes::INVALID_VALUE));
            }
        });
        if r.legacy() {
            component.collision_radius = radius.clamp(0.0, 0.5);
        }
    }
}

/// Whether the object at `node` has the member `key`.
fn member(node: &Node<'_>, key: &str) -> bool {
    node.value()
        .as_object()
        .is_some_and(|object| object.contains_key(key))
}

derived_node!(
    ParticleInitialization,
    ParticleInitialSpin,
    ParticleLifetimeExpression,
    ParticleLifetimeEvents,
    ParticleMotionDynamic,
    ParticleMotionParametric,
    CollisionEvent,
    ParticleMotionCollision,
);
