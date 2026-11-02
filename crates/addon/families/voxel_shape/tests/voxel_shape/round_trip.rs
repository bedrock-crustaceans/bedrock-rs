//! Writing shapes: the value round trip of a vanilla file, the written text, and
//! the constructors.

use bedrock_addon_core::document::{Content, Placed};
use bedrock_addon_core::id::VoxelShapeId;
use bedrock_addon_core::json::{EncodeError, Indent, WriteStyle};
use bedrock_addon_core::version::{ENGINE_VERSION, FormatVersion};
use bedrock_addon_family_voxel_shape::{
    Description, EMPTY, Shape, ShapeBox, UNIT_CUBE, VoxelShape,
};

use super::common::*;

const SLAB_TEXT: &str = r#"{
  "format_version": "1.21.110",
  "minecraft:voxel_shape": {
    "description": {
      "identifier": "mypack:slab_bottom"
    },
    "shape": {
      "boxes": [
        {
          "min": [
            0.0,
            0.0,
            0.0
          ],
          "max": [
            16.0,
            8.0,
            16.0
          ]
        }
      ]
    }
  }
}
"#;

#[test]
fn the_vanilla_anvil_is_written_back_byte_for_byte() {
    // Vanilla writes its shapes the way the default style does, without a final newline.
    let text = fixture("vs01_anvil_base_game/shapes/anvil.json");
    let anvil = loads_in(&text, &base_game());
    let written = anvil.to_json().unwrap();
    assert_eq!(written.trim_end(), text.trim_end());
    // Read again, strictly: the same value, and the same text when written again.
    let again = loads_in(&written, &base_game());
    assert_eq!(again.value(), anvil.value());
    assert_eq!(again.to_json().unwrap(), written);
}

#[test]
fn a_shape_is_written_with_its_envelope() {
    let slab = loads(&slab());
    assert_eq!(slab.to_json().unwrap(), SLAB_TEXT);
    let mut compact = Vec::new();
    slab.write_json(
        &mut compact,
        WriteStyle {
            indent: Indent::Compact,
            trailing_newline: false,
        },
    )
    .unwrap();
    assert_eq!(
        String::from_utf8(compact).unwrap(),
        r#"{"format_version":"1.21.110","minecraft:voxel_shape":{"description":{"identifier":"mypack:slab_bottom"},"shape":{"boxes":[{"min":[0.0,0.0,0.0],"max":[16.0,8.0,16.0]}]}}}"#
    );
}

#[test]
fn a_shape_built_by_hand_equals_the_parsed_one_and_writes_the_same() {
    let built = VoxelShape::new(
        VoxelShapeId::new("mypack:slab_bottom").unwrap(),
        vec![ShapeBox::new([0.0, 0.0, 0.0], [16.0, 8.0, 16.0])],
    );
    assert_eq!(&built, loads(&slab()).value());
    assert_eq!(built.to_json().unwrap(), SLAB_TEXT);
    let literal = VoxelShape {
        description: Description::new(VoxelShapeId::new("mypack:slab_bottom").unwrap()),
        shape: Shape::new(vec![ShapeBox {
            min: [0.0; 3],
            max: [16.0, 8.0, 16.0],
        }]),
        source_format_version: FormatVersion::new(1, 21, 110),
    };
    assert_eq!(literal, built);
}

#[test]
fn an_object_corner_is_written_back_as_the_array() {
    let text = shape(
        "test:corner",
        &[(r#"{"y": -2.5}"#, r#"{"x": 16, "y": 8, "z": 16}"#)],
    );
    let written = loads(&text).to_json().unwrap();
    assert!(!written.contains(r#""x""#), "{written}");
    let again = loads(&written);
    assert_eq!(again.shape.boxes[0].min, [0.0, -2.5, 0.0]);
    assert_eq!(again.value(), loads(&text).value());
    assert_eq!(again.to_json().unwrap(), written);
}

#[test]
fn an_empty_object_corner_is_written_back_as_three_zeros() {
    // A write-back loss: the author wrote `{}`, the file gets three values.
    let text = shape("test:corner", &[("{}", "[16, 8, 16]")]);
    let compact = WriteStyle {
        indent: Indent::Compact,
        trailing_newline: false,
    };
    let mut written = Vec::new();
    loads(&text).write_json(&mut written, compact).unwrap();
    let written = String::from_utf8(written).unwrap();
    assert!(
        written.contains(r#"{"min":[0.0,0.0,0.0],"max":[16.0,8.0,16.0]}"#),
        "{written}"
    );
    assert_eq!(loads(&written).value(), loads(&text).value());
}

#[test]
fn the_document_s_format_version_is_written_back() {
    let newest = slab().replace("1.21.110", &ENGINE_VERSION.to_string());
    let written = loads(&newest).to_json().unwrap();
    assert!(
        written.contains(&format!(r#""format_version": "{ENGINE_VERSION}""#)),
        "{written}"
    );
    // The array form is written as the string.
    let array = slab().replace(r#""1.21.110""#, "[1, 21, 110]");
    assert_eq!(loads(&array).to_json().unwrap(), SLAB_TEXT);
    // A version below the first one the game reads is written as that first one.
    let mut old = loads(&slab()).into_value();
    old.source_format_version = FormatVersion::new(1, 20, 0);
    assert_eq!(old.to_json().unwrap(), SLAB_TEXT);
}

#[test]
fn fractions_are_written_in_their_shortest_form() {
    let text = shape("test:fractions", &[("[0.1, 0, 0]", "[15.5, 8.25, 16]")]);
    let written = loads(&text).to_json().unwrap();
    for number in ["0.1,", "15.5,", "8.25,", "16.0\n"] {
        assert!(written.contains(number), "{number}: {written}");
    }
    assert_eq!(loads(&written).value(), loads(&text).value());
}

#[test]
fn a_coordinate_that_is_not_finite_cannot_be_written() {
    let shape = VoxelShape::new(
        VoxelShapeId::new("test:nan").unwrap(),
        vec![ShapeBox::new([f32::NAN, 0.0, 0.0], [16.0, 8.0, 16.0])],
    );
    assert!(matches!(
        shape.to_json(),
        Err(EncodeError::NotRepresentable { .. })
    ));
}

#[test]
fn the_default_path_is_under_shapes() {
    let slab = loads(&slab());
    assert_eq!(
        slab.default_path().as_str(),
        "shapes/mypack_slab_bottom.json"
    );
    let bare = loads(&shape("half_block", &[("[0, 0, 0]", "[16, 8, 16]")]));
    assert_eq!(bare.default_path().as_str(), "shapes/half_block.json");
}

#[test]
fn the_built_in_shapes_have_vanilla_s_names() {
    assert_eq!(EMPTY.as_str(), "minecraft:empty");
    assert_eq!(UNIT_CUBE.as_str(), "minecraft:unit_cube");
}
