//! The two JSON forms of a corner  and the texts of a
//! value that is neither: rows vs21, vs21b, vs21c.

use super::common::*;

const MIN: &str = "minecraft:voxel_shape/shape/boxes/0/min";
const MAX: &str = "minecraft:voxel_shape/shape/boxes/0/max";
const EXPECTED: &str = "Expected [x, y, z].";
const AN_OBJECT: &str = "expected an object";
const NOT_IN_SCHEMA: &str = "this member was found in the input, but is not present in the Schema";

/// One box from `min` to `max`.
fn one(min: &str, max: &str) -> String {
    shape("test:corner", &[(min, max)])
}

#[test]
fn vs21_a_corner_may_be_an_object() {
    let shape = loads(&one(r#"{"x": 0, "y": 0, "z": 0}"#, "[16, 8, 16]"));
    assert_eq!(shape.shape.boxes[0].min, [0.0, 0.0, 0.0]);
    assert_eq!(shape.shape.boxes[0].max, [16.0, 8.0, 16.0]);
}

#[test]
fn vs21b_absent_members_of_an_object_corner_are_zero() {
    let shape = loads(&one("{}", r#"{"x": 16, "y": 8, "z": 16}"#));
    assert_eq!(shape.shape.boxes[0].min, [0.0, 0.0, 0.0]);
    assert_eq!(shape.shape.boxes[0].max, [16.0, 8.0, 16.0]);
}

#[test]
fn each_member_of_an_object_corner_is_its_own_axis() {
    for (min, expected) in [
        (r#"{"x": -1}"#, [-1.0, 0.0, 0.0]),
        (r#"{"y": -2}"#, [0.0, -2.0, 0.0]),
        (r#"{"z": -3.5}"#, [0.0, 0.0, -3.5]),
        (r#"{"z": 3, "x": 1, "y": 2}"#, [1.0, 2.0, 3.0]),
    ] {
        let shape = loads(&one(min, "[16, 8, 16]"));
        assert_eq!(shape.shape.boxes[0].min, expected, "{min}");
    }
    let shape = loads(&one("[0, 0, 0]", r#"{"y": 8, "x": 4, "z": 2}"#));
    assert_eq!(shape.shape.boxes[0].max, [4.0, 8.0, 2.0]);
}

#[test]
fn an_array_corner_is_three_numbers_in_x_y_z_order() {
    let shape = loads(&one("[-1e0, -0.5, 2]", "[1.6e1, 8.25, 16.0]"));
    assert_eq!(shape.shape.boxes[0].min, [-1.0, -0.5, 2.0]);
    assert_eq!(shape.shape.boxes[0].max, [16.0, 8.25, 16.0]);
}

#[test]
fn vs21c_a_corner_that_is_neither_form_fails_with_both_texts() {
    let neither = [
        "[0, 0]",
        "0",
        "[0, 0, 0, 0]",
        "[]",
        r#"["0", 0, 0]"#,
        r#"[0, "0", 0]"#,
        r#"[0, 0, "0"]"#,
        "[true, 0, 0]",
        "[null, 0, 0]",
        "[[0], 0, 0]",
        "[0, 0, 1e39]",
        r#""0 0 0""#,
        "null",
        "true",
    ];
    for corner in neither {
        assert_eq!(
            errors(&rejects(&one(corner, "[16, 8, 16]"))),
            [
                error("InvalidType", EXPECTED, MIN),
                error("InvalidType", AN_OBJECT, MIN),
                failed()
            ],
            "min: {corner}"
        );
        assert_eq!(
            errors(&rejects(&one("[0, 0, 0]", corner))),
            [
                error("InvalidType", EXPECTED, MAX),
                error("InvalidType", AN_OBJECT, MAX),
                failed()
            ],
            "max: {corner}"
        );
    }
}

#[test]
fn vs21c_an_unknown_member_of_an_object_corner_fails_the_file() {
    assert_eq!(
        errors(&rejects(&one(r#"{"x": 0, "w": 1}"#, "[16, 8, 16]"))),
        [
            error("UnrecognizedField", NOT_IN_SCHEMA, &format!("{MIN}/w")),
            failed()
        ]
    );
    // Each one is reported, in document order; the known members beside them are fine.
    assert_eq!(
        errors(&rejects(&one(
            "[0, 0, 0]",
            r#"{"X": 16, "x": 16, "y": 8, "z": 16, "extra": 0}"#
        ))),
        [
            error("UnrecognizedField", NOT_IN_SCHEMA, &format!("{MAX}/X")),
            error("UnrecognizedField", NOT_IN_SCHEMA, &format!("{MAX}/extra")),
            failed()
        ]
    );
}

#[test]
fn a_member_of_an_object_corner_that_is_no_number_fails_both_forms() {
    // The array setter failed and the member load failed: the help text stays, beside the
    // member's own error.
    for (corner, member) in [
        (r#"{"x": "0"}"#, "x"),
        (r#"{"y": null}"#, "y"),
        (r#"{"z": true}"#, "z"),
        (r#"{"z": [0]}"#, "z"),
        (r#"{"x": 0, "y": 1e39}"#, "y"),
    ] {
        assert_eq!(
            errors(&rejects(&one(corner, "[16, 8, 16]"))),
            [
                error("InvalidType", EXPECTED, MIN),
                error("InvalidType", "wrong JSON type", &format!("{MIN}/{member}")),
                failed()
            ],
            "{corner}"
        );
    }
}

#[test]
fn every_bad_corner_of_a_file_is_reported() {
    let text = shape(
        "test:corner",
        &[("5", "[16, 8, 16]"), ("[0, 0, 0]", r#"{"q": 1}"#)],
    );
    assert_eq!(
        errors(&rejects(&text)),
        [
            error("InvalidType", EXPECTED, MIN),
            error("InvalidType", AN_OBJECT, MIN),
            error(
                "UnrecognizedField",
                NOT_IN_SCHEMA,
                "minecraft:voxel_shape/shape/boxes/1/max/q"
            ),
            failed()
        ]
    );
}
