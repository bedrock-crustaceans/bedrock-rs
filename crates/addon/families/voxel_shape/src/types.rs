//! The family's types, their constructors and `validate`.

use super::FLOOR;
use super::read::constraints;
use super::validate::rules;
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::id::VoxelShapeId;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::schema::Schema;
use bedrock_addon_core::version::FormatVersion;

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
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct Description {
    /// The shape's identifier, `namespace:name` or a bare name, at most 256 bytes (JSON
    /// `identifier`). The `minecraft` namespace is the base game's: a base-game pack must use
    /// it and no other pack may.
    pub identifier: VoxelShapeId,
}

/// `minecraft:voxel_shape/shape`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct Shape {
    /// The boxes, 1 to 32 of them; they may overlap each other (JSON `boxes`).
    pub boxes: Vec<ShapeBox>,
}

/// One box of a shape: two corners in pixels (1/16 of a block), measured from the block's
/// minimum corner, so the block itself spans 0 to 16 on every axis.
///
/// JSON: `{"min": [x, y, z], "max": [x, y, z]}`; a corner may also be written `{"x", "y", "z"}`
/// with absent members 0, and is written back as the array.
#[derive(Clone, Copy, Debug, PartialEq, facet::Facet, Schema)]
pub struct ShapeBox {
    /// The corner with the smallest coordinates, each below `max`'s and at least -14 (JSON
    /// `min`).
    pub min: [f32; 3],
    /// The corner with the largest coordinates, each at most 30 (JSON `max`).
    pub max: [f32; 3],
}

impl VoxelShape {
    /// A shape named `identifier` made of `boxes`, at format version 1.21.110.
    pub fn new(identifier: VoxelShapeId, boxes: Vec<ShapeBox>) -> VoxelShape {
        VoxelShape {
            description: Description::new(identifier),
            shape: Shape::new(boxes),
            source_format_version: FLOOR,
        }
    }

    /// Runs the family's rules as if the shape were loaded from a pack with `cx`: the
    /// identifier's length and the number of boxes, then, when those hold, the namespace rule
    /// of the pack and the geometry rules, of which only the first failure is reported (as the
    /// game stops there). An error means the game would not load the shape.
    pub fn validate(&self, cx: &LoadContext) -> Diagnostics {
        let mut diagnostics = Diagnostics::default();
        let at = JsonPath::new().child("minecraft:voxel_shape");
        constraints(
            self.description.identifier.as_str(),
            self.shape.boxes.len(),
            &at,
            &mut diagnostics,
        );
        if diagnostics.is_empty() {
            rules(self, cx.is_base_game_pack, &at, &mut diagnostics);
        }
        diagnostics
    }
}

impl Description {
    /// The description of the shape `identifier`.
    pub fn new(identifier: VoxelShapeId) -> Description {
        Description { identifier }
    }
}

impl Shape {
    /// A shape of `boxes`.
    pub fn new(boxes: Vec<ShapeBox>) -> Shape {
        Shape { boxes }
    }
}

impl ShapeBox {
    /// The box from `min` to `max`, in pixels.
    pub fn new(min: [f32; 3], max: [f32; 3]) -> ShapeBox {
        ShapeBox { min, max }
    }
}
