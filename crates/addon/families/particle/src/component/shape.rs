//! The emitter shapes: where an emitter's particles appear and which way they leave.

use super::types::{EmitterDirection, PlaneNormal};
use crate::codes;
use crate::read::{Members, Opt, Reader, object};
use bedrock_addon_core::molang::Molang;

/// The offset or direction a shape has when the document does not write one.
fn zeros() -> [Molang; 3] {
    [Molang::Const(0.0), Molang::Const(0.0), Molang::Const(0.0)]
}

object! {
    /// `minecraft:emitter_shape_point`: every particle appears at one point.
    pub struct EmitterShapePoint {
        /// The point, relative to the emitter (JSON `offset`; default `[0, 0, 0]`).
        opt offset: [Molang; 3] = "offset", Opt::NONE, default zeros();
        /// The direction particles leave in (JSON `direction`). A document below format
        /// 1.20.80 that does not write it gets three `math.random(-1, 1)`; a newer one has
        /// no default.
        opt direction: [Molang; 3] = "direction", Opt::NONE;
    }
}

object! {
    /// `minecraft:emitter_shape_box`: particles appear in a box.
    pub struct EmitterShapeBox finish = box_shape {
        /// The centre, relative to the emitter (JSON `offset`; default `[0, 0, 0]`).
        opt offset: [Molang; 3] = "offset", Opt::NONE, default zeros();
        /// Half the box's size on each axis (JSON `half_dimensions`, required).
        /// A document below format 1.20.80 without it logs "offset field required!" and the
        /// legacy parser reads neither `surface_only` nor `direction`.
        req half_dimensions: [Molang; 3] = "half_dimensions", Opt::with_default(0.0);
        /// Particles appear on the surface only (JSON `surface_only`; default `false`).
        opt surface_only: bool = "surface_only", Opt::NONE, default false;
        /// The direction particles leave in (JSON `direction`).
        opt direction: EmitterDirection = "direction", Opt::NONE;
    }
}

/// The legacy box parser returns at a missing `half_dimensions`, after logging its line, so
/// `surface_only` and `direction` are never read: the crate leaves them out and reports them
/// as unread, because the game says nothing of them.
fn box_shape(shape: &mut EmitterShapeBox, m: Members<'_>, r: &mut Reader<'_>) {
    if !r.legacy() || m.get("half_dimensions").is_some() {
        return;
    }
    // The game writes this line to the content log at the error level.
    r.key("half_dimensions", |r| {
        r.content_error(&codes::OFFSET_REQUIRED);
    });
    if shape.surface_only.take().is_some() {
        r.unread("surface_only");
    }
    if shape.direction.take().is_some() {
        r.unread("direction");
    }
}

object! {
    /// `minecraft:emitter_shape_sphere`: particles appear in a sphere.
    pub struct EmitterShapeSphere {
        /// The centre, relative to the emitter (JSON `offset`; default `[0, 0, 0]`).
        opt offset: [Molang; 3] = "offset", Opt::NONE, default zeros();
        /// The radius (JSON `radius`; default 1).
        opt radius: Molang = "radius", Opt::with_default(1.0), default Molang::Const(1.0);
        /// Particles appear on the surface only (JSON `surface_only`; default `false`).
        opt surface_only: bool = "surface_only", Opt::NONE, default false;
        /// The direction particles leave in (JSON `direction`).
        opt direction: EmitterDirection = "direction", Opt::NONE;
    }
}

object! {
    /// `minecraft:emitter_shape_disc`: particles appear in a disc.
    pub struct EmitterShapeDisc {
        /// The centre, relative to the emitter (JSON `offset`; default `[0, 0, 0]`).
        opt offset: [Molang; 3] = "offset", Opt::NONE, default zeros();
        /// The radius (JSON `radius`; default 1).
        opt radius: Molang = "radius", Opt::with_default(1.0), default Molang::Const(1.0);
        /// Particles appear on the edge only (JSON `surface_only`; default `false`).
        opt surface_only: bool = "surface_only", Opt::NONE, default false;
        /// The direction particles leave in (JSON `direction`).
        opt direction: EmitterDirection = "direction", Opt::NONE;
        /// The normal of the disc's plane (JSON `plane_normal`; default `[0, 1, 0]`).
        opt plane_normal: PlaneNormal = "plane_normal", Opt::NONE,
            default PlaneNormal::Custom([
                Molang::Const(0.0),
                Molang::Const(1.0),
                Molang::Const(0.0),
            ]);
    }
}

object! {
    /// `minecraft:emitter_shape_custom`: the document gives the position and the direction.
    pub struct EmitterShapeCustom {
        /// Where particles appear, relative to the emitter (JSON `offset`; default
        /// `[0, 0, 0]`).
        opt offset: [Molang; 3] = "offset", Opt::NONE, default zeros();
        /// The direction particles leave in (JSON `direction`; default `[0, 0, 0]`).
        opt direction: [Molang; 3] = "direction", Opt::NONE, default zeros();
    }
}

object! {
    /// `minecraft:emitter_shape_entity_aabb`: particles appear in the bounding box of the
    /// entity the emitter is attached to.
    pub struct EmitterShapeEntityAabb {
        /// Particles appear on the surface only (JSON `surface_only`; default `false`).
        opt surface_only: bool = "surface_only", Opt::NONE, default false;
        /// The direction particles leave in (JSON `direction`).
        opt direction: EmitterDirection = "direction", Opt::NONE;
    }
}
