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
mod types;

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
use bedrock_addon_derive::Components;

use super::history::ParticleFamily;
use super::read::{Era, Node, Opt, Reader, Writer};
use super::texts;
use bedrock_addon_core::diagnostic::{DiagnosticCode, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::Component;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
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

impl MolangFields for ParticleInitialSpeed {
    fn visit_molang(&mut self, v: &mut MolangVisitor<'_>) {
        self.walk(Era::Cereal, Opt::NONE, v);
    }
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
fn typed<T>(value: &Value, r: &mut Reader<'_>, unreadable: Unreadable) -> Option<ParticleComponent>
where
    T: Node + Into<ParticleComponent>,
{
    let o = match unreadable {
        Unreadable::Line => Opt::NONE.wrong(texts::NO_DATA_FIELDS),
        Unreadable::Silent | Unreadable::Own => Opt::NONE,
    };
    let read = if unreadable == Unreadable::Silent && r.legacy() && !value.is_object() {
        // The game says nothing and keeps every default; what the author wrote there is
        // not read, which is noted at the component.
        r.push(DiagnosticCode::InvalidType { text: None }, Severity::Info);
        None
    } else {
        T::read(value, r, o)
    };
    match read {
        Some(component) => Some(component.into()),
        // The legacy parser keeps the component, with every field at its default.
        None if r.legacy() => Some(T::fallback(o).into()),
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
    value: &Value,
    r: &mut Reader<'_>,
) -> Option<Option<ParticleComponent>> {
    macro_rules! dispatch {
        ($($ty:ident => $unreadable:ident,)*) => {
            $(if name == <$ty as Component>::NAME {
                return Some(typed::<$ty>(value, r, Unreadable::$unreadable));
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
    let era = match mode {
        ConvertMode::Lenient => Era::Legacy,
        ConvertMode::Strict => Era::Cereal,
    };
    let mut r =
        Reader::new(era, cx.molang_version, JsonPath::new(), d).grading(cx.grade(Class::Unknown));
    read_named(T::NAME, &value, &mut r).flatten()
}

/// One typed component as JSON.
pub(crate) fn write_component(
    component: &ParticleComponent,
    w: &mut Writer,
) -> Result<Value, EncodeError> {
    each!(component, c => c.write(w))
}

/// Hands a component's Molang fields to the walk under `era`.
pub(crate) fn walk_component(
    component: &mut ParticleComponent,
    era: Era,
    v: &mut MolangVisitor<'_>,
) {
    each!(component, c => c.walk(era, Opt::NONE, v));
}
