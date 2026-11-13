//! The JSON spellings of the family's closed sets: each one read from literal
//! text in both eras, and each one found in the text a hand-built value writes. A set whose
//! reader and writer share one table round-trips under any spelling, so neither half is
//! tested through the other.

use bedrock_addon_core::document::{Content, UnknownMembers};
use bedrock_addon_family_particle::component::{
    DirectionMode, DirectionSettings, EmitterDirection, EmitterShapeDisc, EmitterShapeSphere,
    ParticleAppearanceBillboard, PlaneNormal,
};
use bedrock_addon_family_particle::{
    BezierChainCurve, BezierChainNode, Curve, EventNode, ParticleEffect, ParticleEffectType,
    SimpleCurve, SimpleCurveType, VisualEffectEvent,
};
use indexmap::IndexMap;

use super::common::{CEREAL, LEGACY, effect, effect_with, findings, loaded, number};

/// One closed set: the member that holds it, and how a document and a built effect carry
/// one of its values.
struct Set<T> {
    /// The JSON key of the member.
    member: &'static str,
    /// The spellings, copied from the schema, each with the value it reads as.
    spellings: Vec<(&'static str, T)>,
    /// A document of `version` whose member is the JSON string `spelling`.
    document: fn(version: &str, spelling: &str) -> String,
    /// The member's value in a loaded effect.
    read: fn(&ParticleEffect) -> Option<T>,
    /// An effect built by hand whose member is `value`.
    built: fn(T) -> ParticleEffect,
}

impl<T: Clone + PartialEq + std::fmt::Debug> Set<T> {
    /// Reads every spelling from literal text in both eras, and finds every spelling in the
    /// text its value writes.
    #[track_caller]
    fn check(&self) {
        for (spelling, value) in &self.spellings {
            for version in [LEGACY, CEREAL] {
                let loaded = loaded(&(self.document)(version, spelling));
                assert_eq!(
                    (self.read)(loaded.value()).as_ref(),
                    Some(value),
                    "{spelling} at {version}"
                );
                assert_eq!(
                    findings(loaded.diagnostics()),
                    [] as [&str; 0],
                    "{spelling} at {version}"
                );
            }
            let text = (self.built)(value.clone()).to_json().expect("it writes");
            let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
            let member = format!(r#""{}":"{spelling}""#, self.member);
            assert_eq!(compact.matches(&member).count(), 1, "{member} in {text}");
            // No other spelling of the set is in the text in its place.
            for (other, _) in &self.spellings {
                let wrong = format!(r#""{}":"{other}""#, self.member);
                assert_eq!(
                    other == spelling,
                    compact.contains(&wrong),
                    "{wrong} in {text}"
                );
            }
        }
    }
}

/// A valid effect to put a value in.
fn base() -> ParticleEffect {
    ParticleEffect::new(
        bedrock_addon_core::id::ParticleId::parse("demo:spark"),
        "particles_alpha",
        "textures/particle/particles",
    )
}

/// The schema: `type` spellings are `bindConst` 1 `particle`, 2
/// `particle_with_velocity`, 3 `emitter`, 4 `emitter_bound`.
#[test]
fn the_particle_effect_types_are_spelled_as_the_schema_spells_them() {
    Set {
        member: "type",
        spellings: vec![
            ("particle", ParticleEffectType::Particle),
            (
                "particle_with_velocity",
                ParticleEffectType::ParticleWithVelocity,
            ),
            ("emitter", ParticleEffectType::Emitter),
            ("emitter_bound", ParticleEffectType::EmitterBound),
        ],
        document: |version, spelling| {
            effect_with(
                version,
                &format!(
                    r#""events": {{"e": {{"particle_effect": {{"effect": "demo:other", "type": "{spelling}"}}}}}}"#
                ),
            )
        },
        read: |effect| {
            let started = effect.events.as_ref()?.get("e")?.particle_effect.as_ref()?;
            Some(started.kind)
        },
        built: |kind| {
            let mut node = EventNode::default();
            node.particle_effect = Some(VisualEffectEvent {
                effect: "demo:other".to_owned(),
                kind,
                pre_effect_expression: None,
                extra: UnknownMembers::default(),
            });
            let mut effect = base();
            effect.events = Some(IndexMap::from([("e".to_owned(), node)]));
            effect
        },
    }
    .check();
}

/// What a curve document and a built curve are compared by: the kind of a simple curve, or
/// that it is a chain.
#[derive(Clone, Debug, PartialEq)]
enum CurveKind {
    Simple(SimpleCurveType),
    BezierChain,
}

/// shape.
#[test]
fn the_curve_types_are_spelled_as_the_schema_spells_them() {
    Set {
        member: "type",
        spellings: vec![
            ("linear", CurveKind::Simple(SimpleCurveType::Linear)),
            ("bezier", CurveKind::Simple(SimpleCurveType::Bezier)),
            ("catmull_rom", CurveKind::Simple(SimpleCurveType::CatmullRom)),
            ("bezier_chain", CurveKind::BezierChain),
        ],
        document: |version, spelling| {
            let nodes = if spelling == "bezier_chain" {
                r#"{"0.0": {"value": 0, "slope": 0}}"#
            } else {
                "[0, 0, 0, 0]"
            };
            effect_with(
                version,
                &format!(
                    r#""curves": {{"variable.c": {{"type": "{spelling}", "input": 0, "nodes": {nodes}}}}}"#
                ),
            )
        },
        read: |effect| match effect.curves.as_ref()?.get("variable.c")? {
            Curve::Simple(curve) => Some(CurveKind::Simple(curve.kind)),
            Curve::BezierChain(_) => Some(CurveKind::BezierChain),
        },
        built: |kind| {
            let curve = match kind {
                CurveKind::Simple(kind) => Curve::Simple(SimpleCurve {
                    kind,
                    input: number(0.0),
                    horizontal_range: None,
                    nodes: vec![number(0.0); 4],
                    extra: UnknownMembers::default(),
                }),
                CurveKind::BezierChain => {
                    let node = BezierChainNode {
                        value: Some(0.0),
                        slope: Some(0.0),
                        ..BezierChainNode::default()
                    };
                    Curve::BezierChain(BezierChainCurve {
                        input: number(0.0),
                        nodes: IndexMap::from([("0.0".to_owned(), node)]),
                        extra: UnknownMembers::default(),
                    })
                }
            };
            let mut effect = base();
            effect.curves = Some(IndexMap::from([("variable.c".to_owned(), curve)]));
            effect
        },
    }
    .check();
}

#[test]
fn the_emitter_directions_are_spelled_as_the_schema_spells_them() {
    Set {
        member: "direction",
        spellings: vec![
            ("inwards", EmitterDirection::Inwards),
            ("outwards", EmitterDirection::Outwards),
        ],
        document: |version, spelling| {
            effect(
                version,
                &format!(r#""minecraft:emitter_shape_sphere": {{"direction": "{spelling}"}}"#),
            )
        },
        read: |effect| {
            let sphere = effect.components.get::<EmitterShapeSphere>()?;
            sphere.direction.clone()
        },
        built: |direction| {
            let mut effect = base();
            effect.components.insert(EmitterShapeSphere {
                direction: Some(direction),
                ..EmitterShapeSphere::default()
            });
            effect
        },
    }
    .check();
}

#[test]
fn the_plane_normals_are_spelled_as_the_schema_spells_them() {
    Set {
        member: "plane_normal",
        spellings: vec![
            ("x", PlaneNormal::X),
            ("y", PlaneNormal::Y),
            ("z", PlaneNormal::Z),
        ],
        document: |version, spelling| {
            effect(
                version,
                &format!(r#""minecraft:emitter_shape_disc": {{"plane_normal": "{spelling}"}}"#),
            )
        },
        read: |effect| {
            let disc = effect.components.get::<EmitterShapeDisc>()?;
            disc.plane_normal.clone()
        },
        built: |normal| {
            let mut effect = base();
            effect.components.insert(EmitterShapeDisc {
                plane_normal: Some(normal),
                ..EmitterShapeDisc::default()
            });
            effect
        },
    }
    .check();
}

/// `bindConst<Mode 1>("custom")`.
#[test]
fn the_direction_modes_are_spelled_as_the_schema_spells_them() {
    Set {
        member: "mode",
        spellings: vec![
            ("derive_from_velocity", DirectionMode::DeriveFromVelocity),
            ("custom", DirectionMode::Custom),
        ],
        document: |version, spelling| {
            effect(
                version,
                &format!(
                    r#""minecraft:particle_appearance_billboard": {{"size": [1, 1], "facing_camera_mode": "lookat_direction", "direction": {{"mode": "{spelling}", "custom_direction": [0, 1, 0], "min_speed_threshold": 0.1}}}}"#
                ),
            )
        },
        read: |effect| {
            let billboard = effect.components.get::<ParticleAppearanceBillboard>()?;
            Some(billboard.direction.as_ref()?.mode)
        },
        built: |mode| {
            let mut effect = base();
            effect.components.insert(ParticleAppearanceBillboard {
                size: [number(1.0), number(1.0)],
                direction: Some(DirectionSettings {
                    mode,
                    custom_direction: Some([number(0.0), number(1.0), number(0.0)]),
                    ..DirectionSettings::default()
                }),
                ..ParticleAppearanceBillboard::default()
            });
            effect
        },
    }
    .check();
}
