//! `VoxelShape::validate`: the rules of loading, on a value built by hand.

use bedrock_addon_core::id::VoxelShapeId;
use bedrock_addon_family_voxel_shape::{ShapeBox, VoxelShape};

use super::common::*;

const SLAB: ShapeBox = ShapeBox {
    min: [0.0, 0.0, 0.0],
    max: [16.0, 8.0, 16.0],
};

fn built(identifier: &str, boxes: Vec<ShapeBox>) -> VoxelShape {
    VoxelShape::new(VoxelShapeId::parse(identifier), boxes)
}

#[test]
fn a_shape_the_game_would_load_has_no_finding() {
    assert!(
        built("mypack:slab", vec![SLAB])
            .validate(&user_pack())
            .is_empty()
    );
    assert!(
        built("minecraft:slab", vec![SLAB])
            .validate(&base_game())
            .is_empty()
    );
}

#[test]
fn validate_reports_what_loading_would_without_the_file_s_closing_line() {
    let shape = built("minecraft:slab", vec![SLAB]);
    assert_eq!(
        errors(&shape.validate(&user_pack())),
        [error(
            "InvalidValue",
            "The `minecraft` namespace is not valid for custom voxel shape: `minecraft:slab`",
            "minecraft:voxel_shape/description/identifier"
        )]
    );
    let shape = built("mypack:slab", vec![SLAB]);
    assert_eq!(
        errors(&shape.validate(&base_game())),
        [error(
            "InvalidValue",
            "The `minecraft` namespace is required for Vanilla shape: `mypack:slab`",
            "minecraft:voxel_shape/description/identifier"
        )]
    );
    let flat = built(
        "mypack:flat",
        vec![SLAB, ShapeBox::new([0.0; 3], [16.0, 0.0, 16.0])],
    );
    assert_eq!(
        errors(&flat.validate(&user_pack())),
        [error(
            "InvalidValue",
            "Box at index 1 is invalid for voxel shape `mypack:flat`",
            "minecraft:voxel_shape/shape/boxes/1"
        )]
    );
}

#[test]
fn validate_checks_the_schema_s_constraints_first() {
    // Both constraints are reported; the rules after them (here V2) are not reached.
    let long = format!("minecraft:{}", "a".repeat(250));
    assert_eq!(
        kinds(&built(&long, Vec::new()).validate(&user_pack())),
        [
            kind(
                "InvalidValue",
                "minecraft:voxel_shape/description/identifier"
            ),
            kind("InvalidValue", "minecraft:voxel_shape/shape/boxes"),
        ]
    );
    assert_eq!(
        errors(&built("mypack:many", vec![SLAB; 33]).validate(&user_pack())),
        [error(
            "InvalidValue",
            " -> shape -> boxes: too many input elements, expected no more than 32",
            "minecraft:voxel_shape/shape/boxes"
        )]
    );
    assert!(
        built("mypack:many", vec![SLAB; 32])
            .validate(&user_pack())
            .is_empty()
    );
}

#[test]
fn validate_does_not_change_the_shape() {
    let shape = built("minecraft:slab", Vec::new());
    let before = shape.clone();
    let _ = shape.validate(&user_pack());
    assert_eq!(shape, before);
}
