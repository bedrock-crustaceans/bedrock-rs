//! `minecraft:voxel_shape` bodies.

use super::{Body, assert_unsupported, document};
use crate::storage::mcb::to_json_text;

const VERSION: (u16, u16, u32) = (1, 21, 110);

#[expect(
    clippy::needless_pass_by_value,
    reason = "a test hands over what it built"
)]
fn shape(body: Body) -> Result<String, crate::storage::StorageError> {
    to_json_text(&document("minecraft:voxel_shape", VERSION, &body.0))
}

#[test]
fn a_shape_decodes_to_its_boxes_min_before_max() {
    let body = Body::default()
        .string("demo:step")
        .raw(&[2])
        .f32s(&[0.0, 0.0, 0.0, 16.0, 8.0, 16.0])
        .f32s(&[0.0, 8.0, 0.5, 16.0, 16.0, 8.0]);
    assert_eq!(
        shape(body).unwrap(),
        concat!(
            r#"{"format_version":"1.21.110","minecraft:voxel_shape":{"description":"#,
            r#"{"identifier":"demo:step"},"shape":{"boxes":["#,
            r#"{"min":[0.0,0.0,0.0],"max":[16.0,8.0,16.0]},"#,
            r#"{"min":[0.0,8.0,0.5],"max":[16.0,16.0,8.0]}]}}}"#
        )
    );
}

#[test]
fn a_shape_with_bytes_after_its_last_box_is_unsupported() {
    let body = Body::default()
        .string("demo:step")
        .raw(&[1])
        .f32s(&[0.0, 0.0, 0.0, 16.0, 8.0, 16.0])
        .raw(&[0]);
    assert_unsupported(shape(body));
}

#[test]
fn a_shape_with_fewer_boxes_than_it_counts_is_unsupported() {
    let body = Body::default()
        .string("demo:step")
        .raw(&[2])
        .f32s(&[0.0, 0.0, 0.0, 16.0, 8.0, 16.0]);
    assert_unsupported(shape(body));
}
