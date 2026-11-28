//! The components of a particle effect (`particle_effect/components`): one type per
//! component, named after the JSON key without its namespace, in `PascalCase`.
//!
//! [`ParticleComponent`] is the closed set a [`bedrock_addon_core::document::ComponentMap`] of an effect
//! holds; `effect.components.get::<EmitterRateSteady>()` reads one. Every component exists
//! from format 1.10.0 and has the same key and members in every format version; none is
//! gated or deprecated. A key the set does not know is reported and kept as written.

mod appearance;
mod emitter;
mod particle;
mod shape;
pub(crate) mod types;

pub use appearance::{
    DirectionMode, DirectionSettings, Flipbook, ParticleAppearanceBillboard,
    ParticleAppearanceLighting, ParticleAppearanceTinting, Uv,
};
pub use emitter::{
    DistanceEvent, EmitterInitialization, EmitterLifetimeEvents, EmitterLifetimeExpression,
    EmitterLifetimeLooping, EmitterLifetimeOnce, EmitterLocalSpace, EmitterRateInstant,
    EmitterRateManual, EmitterRateSteady,
};
pub use particle::{
    CollisionEvent, CollisionEvents, ParticleExpireIfInBlocks, ParticleExpireIfNotInBlocks,
    ParticleInitialSpeed, ParticleInitialSpin, ParticleInitialization, ParticleKillPlane,
    ParticleLifetimeEvents, ParticleLifetimeExpression, ParticleMotionCollision,
    ParticleMotionDynamic, ParticleMotionParametric,
};
pub use shape::{
    EmitterShapeBox, EmitterShapeCustom, EmitterShapeDisc, EmitterShapeEntityAabb,
    EmitterShapePoint, EmitterShapeSphere,
};
pub use types::{
    ColorExpr, ColorGradient, EmitterDirection, Gradient, PlaneNormal, TintColor, VectorEvents,
};

use bedrock_addon_core::diagnostic::Class;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_derive::Components;

use super::codes;
use super::derived::{kit_read, legacy, legacy_spec, note, object, walk_in};
use super::history::ParticleFamily;
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::document::Component;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::MolangVisitor;
use bedrock_addon_core::schema::{self, FromJson};
use bedrock_addon_core::versioning::ConvertMode;

// The game looks a component name up in one registry and compares no version, and the
// 1.20.80 document holds all 29 in the one struct. Every row is released at the floor,
// public, without requirement, deprecation or field gate.

/// One component of a particle effect: the closed set of 29 `minecraft:` components.
#[derive(Clone, Debug, PartialEq, Components)]
#[component(family = ParticleFamily)]
#[non_exhaustive]
#[allow(
    clippy::large_enum_variant,
    reason = "a component map holds few, and boxing every variant would cost each access"
)]
pub enum ParticleComponent {
    /// `minecraft:emitter_initialization`.
    #[component(name = "minecraft:emitter_initialization", released = "1.10.0", molang, parse = row::<EmitterInitialization>)]
    EmitterInitialization(EmitterInitialization),
    /// `minecraft:emitter_lifetime_events`.
    #[component(name = "minecraft:emitter_lifetime_events", released = "1.10.0", parse = row::<EmitterLifetimeEvents>)]
    EmitterLifetimeEvents(EmitterLifetimeEvents),
    /// `minecraft:emitter_lifetime_expression`.
    #[component(name = "minecraft:emitter_lifetime_expression", released = "1.10.0", molang, parse = row::<EmitterLifetimeExpression>)]
    EmitterLifetimeExpression(EmitterLifetimeExpression),
    /// `minecraft:emitter_lifetime_looping`.
    #[component(name = "minecraft:emitter_lifetime_looping", released = "1.10.0", molang, parse = row::<EmitterLifetimeLooping>)]
    EmitterLifetimeLooping(EmitterLifetimeLooping),
    /// `minecraft:emitter_lifetime_once`.
    #[component(name = "minecraft:emitter_lifetime_once", released = "1.10.0", molang, parse = row::<EmitterLifetimeOnce>)]
    EmitterLifetimeOnce(EmitterLifetimeOnce),
    /// `minecraft:emitter_local_space`.
    #[component(name = "minecraft:emitter_local_space", released = "1.10.0", parse = row::<EmitterLocalSpace>)]
    EmitterLocalSpace(EmitterLocalSpace),
    /// `minecraft:emitter_rate_instant`.
    #[component(name = "minecraft:emitter_rate_instant", released = "1.10.0", molang, parse = row::<EmitterRateInstant>)]
    EmitterRateInstant(EmitterRateInstant),
    /// `minecraft:emitter_rate_manual`.
    #[component(name = "minecraft:emitter_rate_manual", released = "1.10.0", molang, parse = row::<EmitterRateManual>)]
    EmitterRateManual(EmitterRateManual),
    /// `minecraft:emitter_rate_steady`.
    #[component(name = "minecraft:emitter_rate_steady", released = "1.10.0", molang, parse = row::<EmitterRateSteady>)]
    EmitterRateSteady(EmitterRateSteady),
    /// `minecraft:emitter_shape_box`.
    #[component(name = "minecraft:emitter_shape_box", released = "1.10.0", molang, parse = row::<EmitterShapeBox>)]
    EmitterShapeBox(EmitterShapeBox),
    /// `minecraft:emitter_shape_custom`.
    #[component(name = "minecraft:emitter_shape_custom", released = "1.10.0", molang, parse = row::<EmitterShapeCustom>)]
    EmitterShapeCustom(EmitterShapeCustom),
    /// `minecraft:emitter_shape_disc`.
    #[component(name = "minecraft:emitter_shape_disc", released = "1.10.0", molang, parse = row::<EmitterShapeDisc>)]
    EmitterShapeDisc(EmitterShapeDisc),
    /// `minecraft:emitter_shape_entity_aabb`.
    #[component(name = "minecraft:emitter_shape_entity_aabb", released = "1.10.0", molang, parse = row::<EmitterShapeEntityAabb>)]
    EmitterShapeEntityAabb(EmitterShapeEntityAabb),
    /// `minecraft:emitter_shape_point`.
    #[component(name = "minecraft:emitter_shape_point", released = "1.10.0", molang, parse = row::<EmitterShapePoint>)]
    EmitterShapePoint(EmitterShapePoint),
    /// `minecraft:emitter_shape_sphere`.
    #[component(name = "minecraft:emitter_shape_sphere", released = "1.10.0", molang, parse = row::<EmitterShapeSphere>)]
    EmitterShapeSphere(EmitterShapeSphere),
    /// `minecraft:particle_appearance_billboard`.
    #[component(name = "minecraft:particle_appearance_billboard", released = "1.10.0", molang, parse = row::<ParticleAppearanceBillboard>)]
    ParticleAppearanceBillboard(ParticleAppearanceBillboard),
    /// `minecraft:particle_appearance_lighting`.
    #[component(name = "minecraft:particle_appearance_lighting", released = "1.10.0", parse = row::<ParticleAppearanceLighting>)]
    ParticleAppearanceLighting(ParticleAppearanceLighting),
    /// `minecraft:particle_appearance_tinting`.
    #[component(name = "minecraft:particle_appearance_tinting", released = "1.10.0", molang, parse = row::<ParticleAppearanceTinting>)]
    ParticleAppearanceTinting(ParticleAppearanceTinting),
    /// `minecraft:particle_expire_if_in_blocks`.
    #[component(name = "minecraft:particle_expire_if_in_blocks", released = "1.10.0", parse = row::<ParticleExpireIfInBlocks>)]
    ParticleExpireIfInBlocks(ParticleExpireIfInBlocks),
    /// `minecraft:particle_expire_if_not_in_blocks`.
    #[component(name = "minecraft:particle_expire_if_not_in_blocks", released = "1.10.0", parse = row::<ParticleExpireIfNotInBlocks>)]
    ParticleExpireIfNotInBlocks(ParticleExpireIfNotInBlocks),
    /// `minecraft:particle_initial_speed`.
    #[component(name = "minecraft:particle_initial_speed", released = "1.10.0", molang, parse = row::<ParticleInitialSpeed>)]
    ParticleInitialSpeed(ParticleInitialSpeed),
    /// `minecraft:particle_initial_spin`.
    #[component(name = "minecraft:particle_initial_spin", released = "1.10.0", molang, parse = row::<ParticleInitialSpin>)]
    ParticleInitialSpin(ParticleInitialSpin),
    /// `minecraft:particle_initialization`.
    #[component(name = "minecraft:particle_initialization", released = "1.10.0", molang, parse = row::<ParticleInitialization>)]
    ParticleInitialization(ParticleInitialization),
    /// `minecraft:particle_kill_plane`.
    #[component(name = "minecraft:particle_kill_plane", released = "1.10.0", parse = row::<ParticleKillPlane>)]
    ParticleKillPlane(ParticleKillPlane),
    /// `minecraft:particle_lifetime_events`.
    #[component(name = "minecraft:particle_lifetime_events", released = "1.10.0", parse = row::<ParticleLifetimeEvents>)]
    ParticleLifetimeEvents(ParticleLifetimeEvents),
    /// `minecraft:particle_lifetime_expression`.
    #[component(name = "minecraft:particle_lifetime_expression", released = "1.10.0", molang, parse = row::<ParticleLifetimeExpression>)]
    ParticleLifetimeExpression(ParticleLifetimeExpression),
    /// `minecraft:particle_motion_collision`.
    #[component(name = "minecraft:particle_motion_collision", released = "1.10.0", molang, parse = row::<ParticleMotionCollision>)]
    ParticleMotionCollision(ParticleMotionCollision),
    /// `minecraft:particle_motion_dynamic`.
    #[component(name = "minecraft:particle_motion_dynamic", released = "1.10.0", molang, parse = row::<ParticleMotionDynamic>)]
    ParticleMotionDynamic(ParticleMotionDynamic),
    /// `minecraft:particle_motion_parametric`.
    #[component(name = "minecraft:particle_motion_parametric", released = "1.10.0", molang, parse = row::<ParticleMotionParametric>)]
    ParticleMotionParametric(ParticleMotionParametric),
}

/// What the legacy parser does with a component whose value it cannot read at all.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Unreadable {
    /// It logs "Component has no data fields!" and keeps every default.
    Line,
    /// It says nothing and keeps every default (`emitter_local_space`, and
    /// `particle_appearance_lighting`, which has nothing to read).
    Silent,
    /// The component's own reader logs its own line.
    Own,
}

/// Applies `$body` to the component a variant holds, whichever it is.
macro_rules! each {
    ($value:expr, $c:ident => $body:expr) => {
        match $value {
            ParticleComponent::EmitterInitialization($c) => $body,
            ParticleComponent::EmitterLifetimeEvents($c) => $body,
            ParticleComponent::EmitterLifetimeExpression($c) => $body,
            ParticleComponent::EmitterLifetimeLooping($c) => $body,
            ParticleComponent::EmitterLifetimeOnce($c) => $body,
            ParticleComponent::EmitterLocalSpace($c) => $body,
            ParticleComponent::EmitterRateInstant($c) => $body,
            ParticleComponent::EmitterRateManual($c) => $body,
            ParticleComponent::EmitterRateSteady($c) => $body,
            ParticleComponent::EmitterShapeBox($c) => $body,
            ParticleComponent::EmitterShapeCustom($c) => $body,
            ParticleComponent::EmitterShapeDisc($c) => $body,
            ParticleComponent::EmitterShapeEntityAabb($c) => $body,
            ParticleComponent::EmitterShapePoint($c) => $body,
            ParticleComponent::EmitterShapeSphere($c) => $body,
            ParticleComponent::ParticleAppearanceBillboard($c) => $body,
            ParticleComponent::ParticleAppearanceLighting($c) => $body,
            ParticleComponent::ParticleAppearanceTinting($c) => $body,
            ParticleComponent::ParticleExpireIfInBlocks($c) => $body,
            ParticleComponent::ParticleExpireIfNotInBlocks($c) => $body,
            ParticleComponent::ParticleInitialSpeed($c) => $body,
            ParticleComponent::ParticleInitialSpin($c) => $body,
            ParticleComponent::ParticleInitialization($c) => $body,
            ParticleComponent::ParticleKillPlane($c) => $body,
            ParticleComponent::ParticleLifetimeEvents($c) => $body,
            ParticleComponent::ParticleLifetimeExpression($c) => $body,
            ParticleComponent::ParticleMotionCollision($c) => $body,
            ParticleComponent::ParticleMotionDynamic($c) => $body,
            ParticleComponent::ParticleMotionParametric($c) => $body,
        }
    };
}

/// One typed component read from its value; `None` when the 1.20.80 reader refused it.
fn typed<T>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    unreadable: Unreadable,
) -> Option<ParticleComponent>
where
    T: FromJson + Default + Into<ParticleComponent>,
{
    let legacy = legacy(read);
    let read_value = match unreadable {
        Unreadable::Line => object::<T>(node, read, Some(&codes::NO_DATA_FIELDS)),
        // The game says nothing and keeps every default; what the author wrote there is not
        // read, which is noted at the component.
        Unreadable::Silent if legacy && !node.value().is_object() => {
            note(read, node.path(), &core_codes::INVALID_TYPE);
            None
        }
        Unreadable::Silent => object::<T>(node, read, None),
        Unreadable::Own => T::from_json(node, read),
    };
    match read_value {
        Some(component) => Some(component.into()),
        // The legacy parser keeps the component, with every field at its default.
        None if legacy => Some(T::default().into()),
        None => None,
    }
}

/// Reads the component `name` from `value`. The outer `None` is a name the set does not
/// know; the inner one a component the 1.20.80 reader refused (the finding is reported).
#[allow(
    clippy::option_option,
    reason = "an unknown name and a refused component are two outcomes beside the component"
)]
pub(crate) fn read_named(
    name: &str,
    node: &Node<'_>,
    read: &mut Read<'_>,
) -> Option<Option<ParticleComponent>> {
    macro_rules! dispatch {
        ($($ty:ident => $unreadable:ident,)*) => {
            $(if name == <$ty as Component>::NAME {
                return Some(typed::<$ty>(node, read, Unreadable::$unreadable));
            })*
        };
    }
    dispatch! {
        EmitterInitialization => Line,
        EmitterLifetimeEvents => Line,
        EmitterLifetimeExpression => Line,
        EmitterLifetimeLooping => Line,
        EmitterLifetimeOnce => Line,
        EmitterLocalSpace => Silent,
        EmitterRateInstant => Line,
        EmitterRateManual => Line,
        EmitterRateSteady => Line,
        EmitterShapeBox => Line,
        EmitterShapeCustom => Line,
        EmitterShapeDisc => Line,
        EmitterShapeEntityAabb => Line,
        EmitterShapePoint => Line,
        EmitterShapeSphere => Line,
        ParticleAppearanceBillboard => Line,
        ParticleAppearanceLighting => Silent,
        ParticleAppearanceTinting => Line,
        ParticleExpireIfInBlocks => Own,
        ParticleExpireIfNotInBlocks => Own,
        ParticleInitialSpeed => Own,
        ParticleInitialSpin => Line,
        ParticleInitialization => Line,
        ParticleKillPlane => Own,
        ParticleLifetimeEvents => Line,
        ParticleLifetimeExpression => Line,
        ParticleMotionCollision => Line,
        ParticleMotionDynamic => Line,
        ParticleMotionParametric => Line,
    }
    None
}

/// The table's row parser: the component read from its slice of the text, leniently as the
/// legacy parser or strictly as the 1.20.80 reader. Findings are relative to the component.
fn row<T>(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<ParticleComponent>
where
    T: Component<Set = ParticleComponent>,
{
    let value = crate::read::value_of(slice.text).ok()?;
    let legacy = mode == ConvertMode::Lenient;
    let mut read = kit_read(legacy, cx.molang_version, cx.grade(Class::Unknown), d);
    let root = Node::new(&value, JsonPath::new());
    schema::read_document::<Named<T>>(&root, &mut read).map(|named| named.0)
}

/// The component `T` names, read by its row of the dispatch, as a root the read's machinery
/// takes (`schema::read_document`).
struct Named<T>(ParticleComponent, std::marker::PhantomData<T>);

impl<T: Component<Set = ParticleComponent>> FromJson for Named<T> {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let component = read_named(T::NAME, node, read).flatten()?;
        Some(Self(component, std::marker::PhantomData))
    }
}

/// One typed component as JSON, at `at` in a document of the era.
pub(crate) fn write_component(
    component: &ParticleComponent,
    legacy: bool,
    at: &JsonPath,
) -> Result<Value, EncodeError> {
    each!(component, c => crate::write::json(c, legacy, at))
}

/// Hands a component of a legacy document to the walk: the speed of
/// `particle_initial_speed` under the legacy parser's default, as a leaf read by hand.
pub(crate) fn walk_legacy(component: &mut ParticleComponent, v: &mut MolangVisitor<'_>) {
    match component {
        ParticleComponent::ParticleInitialSpeed(speed) => speed.walk_under(&legacy_spec(0.0), v),
        component => each!(component, c => walk_in(c, true, v)),
    }
}
