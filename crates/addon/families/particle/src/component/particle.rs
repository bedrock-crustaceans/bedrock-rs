//! The per-particle components other than the appearance: initial state, lifetime and motion.

use indexmap::IndexMap;

use super::types::{VectorEvents, timeline, vector_events};
use crate::codes;
use crate::derived::{
    KEPT, L0, L1, LEGACY, STRICT, elements, float, legacy, line, molang_array_of, molang_of, note,
    object, refuse, string, walk_each, wrong,
};
use crate::read::defaults;
use bedrock_addon_core::diagnostic::Code;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::molang::{
    Molang, MolangFieldMut, MolangFieldSpec, MolangFields, MolangVisitor,
};
use bedrock_addon_core::schema::{self, FromJson, Schema};
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

/// The speed the legacy parser gives a component it cannot read: 0.
impl Default for ParticleInitialSpeed {
    fn default() -> Self {
        Self::Uniform(Molang::Const(0.0))
    }
}

impl FromJson for ParticleInitialSpeed {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if node.value().is_array() {
            molang_array_of(node, read, 0.0).map(Self::Vector)
        } else {
            molang_of(node, read, 0.0).map(Self::Uniform)
        }
    }
}

impl ParticleInitialSpeed {
    /// Hands the speed to the walk under `spec`.
    pub(crate) fn walk_under(&mut self, spec: &MolangFieldSpec, v: &mut MolangVisitor<'_>) {
        match self {
            Self::Uniform(speed) => v.emit(spec, MolangFieldMut::Value(speed)),
            Self::Vector(speed) => walk_each(speed, spec, v),
        }
    }
}

/// The walk of a 1.20.80 document's component: kept, as every field of it.
impl MolangFields for ParticleInitialSpeed {
    fn visit_molang(&mut self, v: &mut MolangVisitor<'_>) {
        self.walk_under(&KEPT, v);
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
    #[facet(schema::read_with = vector_events)]
    pub creation_event: Option<VectorEvents>,
    /// Fired when the particle expires (JSON `expiration_event`).
    #[facet(schema::read_with = vector_events)]
    pub expiration_event: Option<VectorEvents>,
    /// Events by the particle's age in seconds, the keys decimal strings (JSON
    /// `timeline`).
    #[facet(schema::read_with = timeline)]
    pub timeline: Option<IndexMap<String, VectorEvents>>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

/// A block list: the value of `minecraft:particle_expire_if_in_blocks` and its opposite. An
/// element that is no string is left out with the legacy parser's line.
fn blocks(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<String>> {
    elements(node, read, Some(&codes::NO_BLOCK_LIST), |element, read| {
        string(element, read, Some(&codes::BLOCK_NOT_STRING))
    })
}

/// `minecraft:particle_expire_if_in_blocks`: the particle expires inside one of these blocks.
///
/// JSON: the component's value itself is an array of block names.
#[derive(Clone, Debug, Default, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct ParticleExpireIfInBlocks(pub Vec<String>);

impl FromJson for ParticleExpireIfInBlocks {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        blocks(node, read).map(Self)
    }
}

/// `minecraft:particle_expire_if_not_in_blocks`: the particle expires outside these blocks.
///
/// JSON: the component's value itself is an array of block names.
#[derive(Clone, Debug, Default, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct ParticleExpireIfNotInBlocks(pub Vec<String>);

impl FromJson for ParticleExpireIfNotInBlocks {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        blocks(node, read).map(Self)
    }
}

/// `minecraft:particle_kill_plane`: the particle expires when it crosses a plane.
///
/// JSON: the component's value itself is `[a, b, c, d]`, the plane `ax + by + cz + d = 0` in
/// the emitter's space.
#[derive(Clone, Copy, Debug, Default, PartialEq, Facet)]
#[facet(transparent)]
pub struct ParticleKillPlane(pub [f32; 4]);

impl FromJson for ParticleKillPlane {
    /// An array of four numbers; the crate's own finding for one that is not.
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if node.value().as_array().is_none_or(|array| array.len() != 4) {
            wrong(read, node, None);
            return None;
        }
        let elements = read.array(node)?;
        let mut plane = [0.0; 4];
        let mut complete = true;
        for (i, element) in elements.iter().enumerate() {
            if let Some(value) = float(&element, read) {
                plane[i] = value;
            } else {
                wrong(read, &element, None);
                complete = false;
            }
        }
        complete.then_some(Self(plane))
    }
}

bedrock_addon_core::impl_no_molang!(
    ParticleExpireIfInBlocks,
    ParticleExpireIfNotInBlocks,
    ParticleKillPlane
);
bedrock_addon_core::impl_schema_leaf!(
    ParticleInitialSpeed,
    ParticleExpireIfInBlocks,
    ParticleExpireIfNotInBlocks,
    ParticleKillPlane
);

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
    if legacy(read) && written && event.min_speed.is_none_or(|speed| speed < LEGACY_MIN_SPEED) {
        event.min_speed = Some(LEGACY_MIN_SPEED);
        note(
            read,
            &node.path().child("min_speed"),
            &core_codes::INVALID_VALUE,
        );
    }
}

/// `minecraft:particle_motion_collision/events`: one event or a list.
///
/// JSON: an object, or an array of objects.
pub type CollisionEvents = OneOrMany<CollisionEvent>;

/// [`CollisionEvents`]: the legacy parser leaves out an entry without an `event` string. An
/// entry that is left out is noted where it is, at the information level: the game's line at
/// its `event` does not say that the entry's other members go with it.
fn collision_events(node: &Node<'_>, read: &mut Read<'_>) -> Option<CollisionEvents> {
    let element = Some(&codes::EVENT_NOT_PAIR);
    let legacy = legacy(read);
    let named = |entry: &Value| {
        entry
            .as_object()
            .and_then(|entry| entry.get("event"))
            .and_then(Value::as_string)
            .is_some_and(|name| !name.as_str().is_empty())
    };
    let usable = |event: &CollisionEvent| !legacy || !event.event.is_empty();
    if node.value().is_object() {
        let event = object::<CollisionEvent>(node, read, element)?;
        if legacy && !named(node.value()) {
            note(read, node.path(), &core_codes::DROPPED_FIELD);
        }
        return usable(&event).then_some(OneOrMany::One(event));
    }
    if !node.value().is_array() {
        wrong(read, node, Some(&codes::EVENTS_NOT_PAIRS));
        return None;
    }
    let entries = read.array(node)?;
    let mut events = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        if let Some(event) = object::<CollisionEvent>(&entry, read, element) {
            events.push((index, event));
        }
    }
    if legacy {
        for entry in entries.iter() {
            if entry.value().is_object() && !named(entry.value()) {
                note(read, entry.path(), &core_codes::DROPPED_FIELD);
            }
        }
    }
    // The events that stay, each with the index the file has it at.
    let (places, events): (Vec<_>, Vec<_>) = events
        .into_iter()
        .filter(|(_, event)| usable(event))
        .unzip();
    read.findings_mut().left_out(node.path(), &places);
    Some(OneOrMany::Many(events))
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
    #[facet(schema::read_with = collision_events)]
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
    let radius = component.collision_radius;
    if (0.0..=0.5).contains(&radius) {
        return;
    }
    let at = node.path().child("collision_radius");
    if legacy(read) {
        let code = if radius < 0.0 {
            &codes::RADIUS_TOO_SMALL
        } else {
            &codes::RADIUS_TOO_LARGE
        };
        line(read, &at, code);
        component.collision_radius = radius.clamp(0.0, 0.5);
    } else {
        refuse(read, &at, Code::from(&core_codes::INVALID_VALUE));
    }
}

/// Whether the object at `node` has the member `key`.
fn member(node: &Node<'_>, key: &str) -> bool {
    node.value()
        .as_object()
        .is_some_and(|object| object.contains_key(key))
}
