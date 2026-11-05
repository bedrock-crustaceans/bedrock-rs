//! Constructors and `validate`.

use super::parse::constraints;
use super::validate::rules;
use super::{Description, FLOOR, Shape, ShapeBox, VoxelShape};
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::id::VoxelShapeId;
use bedrock_addon_core::load::LoadContext;

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
