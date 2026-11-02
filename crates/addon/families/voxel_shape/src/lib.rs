//! Voxel shapes (`VoxelShape`): `shapes/**/*.json` of a behaviour pack (`minecraft:voxel_shape`),
//! the boxes a block's `minecraft:geometry.culling_shape` names. One struct covers every
//! `format_version` from 1.21.110 to the engine.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod load;
mod new;
mod parse;
mod texts;
mod validate;
mod write;

#[cfg(test)]
mod tests;

use bedrock_addon_core::id::VoxelShapeId;
use bedrock_addon_core::version::FormatVersion;

#[doc(hidden)]
pub use load::load_registry;

/// The first and only `format_version` vanilla has a voxel-shape parser for
/// (1.21.110); it reads every version from it up to the engine's.
pub(crate) const FLOOR: FormatVersion = FormatVersion::new(1, 21, 110);

/// The built-in shape without boxes (`minecraft:empty`): what a block culls with when its
/// geometry names no `culling_shape`. No pack defines it.
pub const EMPTY: VoxelShapeId = VoxelShapeId::from_static("minecraft:empty");

/// The built-in full block (`minecraft:unit_cube`): one box from 0 to 16 on every axis. No
/// pack defines it.
pub const UNIT_CUBE: VoxelShapeId = VoxelShapeId::from_static("minecraft:unit_cube");

/// A voxel shape (`minecraft:voxel_shape`): a named set of boxes, in pixels, that a block's
/// geometry uses to cull the faces of its neighbours.
///
/// JSON: `{"format_version": "1.21.110", "minecraft:voxel_shape": {"description":
/// {"identifier"}, "shape": {"boxes": [{"min": [x, y, z], "max": [x, y, z]}]}}}`. Every member
/// is required and an unknown member fails the file. A shape has 1 to 32 boxes; each lies within
/// -14 to 30 pixels, at least one overlaps the block itself (0 to 16 on every axis), and the
/// whole shape spans at most 30 pixels on any axis.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:voxel_shape")]
#[doc(alias = "shapes")]
pub struct VoxelShape {
    /// The shape's name (JSON `description`).
    pub description: Description,
    /// The shape's boxes (JSON `shape`).
    pub shape: Shape,
    /// The document's `format_version`. Not a JSON member; the writer writes it as
    /// `format_version`.
    pub source_format_version: FormatVersion,
}

/// `minecraft:voxel_shape/description`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct Description {
    /// The shape's identifier, `namespace:name` or a bare name, at most 256 bytes (JSON
    /// `identifier`). The `minecraft` namespace is the base game's: a base-game pack must use
    /// it and no other pack may.
    pub identifier: VoxelShapeId,
}

/// `minecraft:voxel_shape/shape`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct Shape {
    /// The boxes, 1 to 32 of them; they may overlap each other (JSON `boxes`).
    pub boxes: Vec<ShapeBox>,
}

/// One box of a shape: two corners in pixels (1/16 of a block), measured from the block's
/// minimum corner, so the block itself spans 0 to 16 on every axis.
///
/// JSON: `{"min": [x, y, z], "max": [x, y, z]}`; a corner may also be written `{"x", "y", "z"}`
/// with absent members 0, and is written back as the array.
#[derive(Clone, Copy, Debug, PartialEq, facet::Facet)]
pub struct ShapeBox {
    /// The corner with the smallest coordinates, each below `max`'s and at least -14 (JSON
    /// `min`).
    pub min: [f32; 3],
    /// The corner with the largest coordinates, each at most 30 (JSON `max`).
    pub max: [f32; 3],
}
