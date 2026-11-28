//! The emitter shapes: where an emitter's particles appear and which way they leave.

use super::types::{EmitterDirection, PlaneNormal};
use crate::codes;
use crate::derived::{KEPT, L0, L1, LEGACY, STRICT, legacy, line, unread};
use crate::read::defaults;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::schema::FromJson;
use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

/// A Molang field that is 0 when the document does not write it.
const ZERO: Molang = Molang::Const(0.0);

/// Three Molang fields that are 0.
const ZEROS: [Molang; 3] = [ZERO, ZERO, ZERO];

/// The offset or direction a shape has when the document does not write one.
fn zeros() -> [Molang; 3] {
    ZEROS
}

/// `minecraft:emitter_shape_point`: every particle appears at one point.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report", schema::molang = KEPT)]
pub struct EmitterShapePoint {
    /// The point, relative to the emitter (JSON `offset`; default `[0, 0, 0]`).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub offset: Option<[Molang; 3]>,
    /// The direction particles leave in (JSON `direction`). A document below format
    /// 1.20.80 that does not write it gets three `math.random(-1, 1)`; a newer one has
    /// no default.
    #[facet(schema::era(LEGACY, molang = L0))]
    pub direction: Option<[Molang; 3]>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(EmitterShapePoint {
    offset: [Molang; 3] = "offset", zeros();
});

/// `minecraft:emitter_shape_box`: particles appear in a box.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(
    schema::unknown = "report",
    schema::molang = KEPT,
    schema::after_read = box_shape
)]
pub struct EmitterShapeBox {
    /// The centre, relative to the emitter (JSON `offset`; default `[0, 0, 0]`).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub offset: Option<[Molang; 3]>,
    /// Half the box's size on each axis (JSON `half_dimensions`, required).
    /// A document below format 1.20.80 without it logs "offset field required!" and the
    /// legacy parser reads neither `surface_only` nor `direction`.
    #[facet(
        default = ZEROS,
        schema::era(LEGACY, molang = L0),
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub half_dimensions: [Molang; 3],
    /// Particles appear on the surface only (JSON `surface_only`; default `false`).
    pub surface_only: Option<bool>,
    /// The direction particles leave in (JSON `direction`).
    #[facet(schema::read_with = EmitterDirection::from_json)]
    pub direction: Option<EmitterDirection>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(EmitterShapeBox {
    offset: [Molang; 3] = "offset", zeros();
    surface_only: bool = "surface_only", false;
});

/// The legacy box parser returns at a missing `half_dimensions`, after logging its line, so
/// `surface_only` and `direction` are never read: the crate leaves them out and reports them
/// as unread, because the game says nothing of them.
fn box_shape(shape: &mut EmitterShapeBox, node: &Node<'_>, read: &mut Read<'_>) {
    let written = node
        .value()
        .as_object()
        .is_some_and(|object| object.contains_key("half_dimensions"));
    if !legacy(read) || written {
        return;
    }
    // The game writes this line to the content log at the error level.
    line(
        read,
        &node.path().child("half_dimensions"),
        &codes::OFFSET_REQUIRED,
    );
    if shape.surface_only.take().is_some() {
        unread(read, &node.path().child("surface_only"));
    }
    if shape.direction.take().is_some() {
        unread(read, &node.path().child("direction"));
    }
}

/// `minecraft:emitter_shape_sphere`: particles appear in a sphere.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report", schema::molang = KEPT)]
pub struct EmitterShapeSphere {
    /// The centre, relative to the emitter (JSON `offset`; default `[0, 0, 0]`).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub offset: Option<[Molang; 3]>,
    /// The radius (JSON `radius`; default 1).
    #[facet(schema::era(LEGACY, molang = L1))]
    pub radius: Option<Molang>,
    /// Particles appear on the surface only (JSON `surface_only`; default `false`).
    pub surface_only: Option<bool>,
    /// The direction particles leave in (JSON `direction`).
    #[facet(schema::read_with = EmitterDirection::from_json)]
    pub direction: Option<EmitterDirection>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(EmitterShapeSphere {
    offset: [Molang; 3] = "offset", zeros();
    radius: Molang = "radius", Molang::Const(1.0);
    surface_only: bool = "surface_only", false;
});

/// `minecraft:emitter_shape_disc`: particles appear in a disc.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report", schema::molang = KEPT)]
pub struct EmitterShapeDisc {
    /// The centre, relative to the emitter (JSON `offset`; default `[0, 0, 0]`).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub offset: Option<[Molang; 3]>,
    /// The radius (JSON `radius`; default 1).
    #[facet(schema::era(LEGACY, molang = L1))]
    pub radius: Option<Molang>,
    /// Particles appear on the edge only (JSON `surface_only`; default `false`).
    pub surface_only: Option<bool>,
    /// The direction particles leave in (JSON `direction`).
    #[facet(schema::read_with = EmitterDirection::from_json)]
    pub direction: Option<EmitterDirection>,
    /// The normal of the disc's plane (JSON `plane_normal`; default `[0, 1, 0]`).
    #[facet(schema::read_with = PlaneNormal::from_json)]
    pub plane_normal: Option<PlaneNormal>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(EmitterShapeDisc {
    offset: [Molang; 3] = "offset", zeros();
    radius: Molang = "radius", Molang::Const(1.0);
    surface_only: bool = "surface_only", false;
    plane_normal: PlaneNormal = "plane_normal", PlaneNormal::Custom([ZERO, Molang::Const(1.0), ZERO]);
});

/// `minecraft:emitter_shape_custom`: the document gives the position and the direction.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report", schema::molang = KEPT)]
pub struct EmitterShapeCustom {
    /// Where particles appear, relative to the emitter (JSON `offset`; default
    /// `[0, 0, 0]`).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub offset: Option<[Molang; 3]>,
    /// The direction particles leave in (JSON `direction`; default `[0, 0, 0]`).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub direction: Option<[Molang; 3]>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(EmitterShapeCustom {
    offset: [Molang; 3] = "offset", zeros();
    direction: [Molang; 3] = "direction", zeros();
});

/// `minecraft:emitter_shape_entity_aabb`: particles appear in the bounding box of the
/// entity the emitter is attached to.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report")]
pub struct EmitterShapeEntityAabb {
    /// Particles appear on the surface only (JSON `surface_only`; default `false`).
    pub surface_only: Option<bool>,
    /// The direction particles leave in (JSON `direction`).
    #[facet(schema::read_with = EmitterDirection::from_json)]
    pub direction: Option<EmitterDirection>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(EmitterShapeEntityAabb {
    surface_only: bool = "surface_only", false;
});
