//! `Vec3` and `Vec2`: both forms read and write as written (the object form with its unknown
//! members), and a vector of the wrong shape is refused with entity's texts.

mod common;

use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::JsonType;
use bedrock_addon_shared_shapes::{Vec2, Vec3, VecForm, VectorError};
use common::round_trip;

/// The read error of a `Vec3` for `json`.
fn error(json: &str) -> VectorError {
    let value: Value = bedrock_addon_core::json::from_str_jsonc(json).expect("JSON");
    Vec3::<f32>::try_from(value).expect_err("must be rejected")
}

#[test]
fn the_array_form_reads_exactly_its_coordinates() {
    let v: Vec3 = round_trip("[1.5, 2.0, -3.25]");
    assert_eq!(v, Vec3::new(1.5, 2.0, -3.25));
    assert_eq!(v.to_array(), [1.5, 2.0, -3.25]);
    // An integer token is a coordinate; it writes as the real it is.
    let v: Vec3 = common::read("[1, 2, 3]").unwrap();
    assert_eq!(common::compact(&v), "[1.0,2.0,3.0]");
    let v: Vec2 = round_trip("[0.1, 4.0]");
    assert_eq!(v, Vec2::from([0.1, 4.0]));
}

#[test]
fn the_object_form_keeps_its_unknown_members() {
    let v: Vec3 = round_trip(r#"{"x": 1.0, "y": 0.5, "z": 2.0, "w": true}"#);
    assert_eq!((v.x, v.y, v.z), (1.0, 0.5, 2.0));
    assert_eq!(v.form, VecForm::Object);
    assert_eq!(v.extra.iter().count(), 1);
    // A coordinate left out is 0, and written: the object form writes every coordinate.
    let v: Vec2 = common::read(r#"{"y": 3.0}"#).unwrap();
    assert_eq!(v.to_array(), [0.0, 3.0]);
    assert_eq!(common::compact(&v), r#"{"x":0.0,"y":3.0}"#);
}

#[test]
fn a_vector_of_the_wrong_shape_is_refused() {
    assert_eq!(
        error("[1, 2]"),
        VectorError::ArrayLength {
            expected: 3,
            len: 2
        }
    );
    assert_eq!(
        error(r#"[1, "2", 3]"#),
        VectorError::WrongType {
            expected: "a number",
            found: JsonType::String
        }
    );
    assert_eq!(
        error("7"),
        VectorError::WrongType {
            expected: "an array or an object",
            found: JsonType::Number
        }
    );
    assert_eq!(error("[1, 2, 1e39]"), VectorError::OutOfRange);
    assert_eq!(
        error("[1, 2]").to_string(),
        "expected an array of 3 numbers, found 2 elements"
    );
    assert_eq!(Vec3::splat(2.0_f32), Vec3::new(2.0, 2.0, 2.0));
}
