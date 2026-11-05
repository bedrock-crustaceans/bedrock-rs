//! `HexOr`: the text form is kept as written (even a text that is no colour), the array form
//! reads exactly its channels, and both write back as read (a unit channel written as an
//! integer token comes back as a real).

mod common;

use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::JsonType;
use bedrock_addon_core::shared::ShapeErrorKind;
use bedrock_addon_shared_shapes::HexOr;
use common::{compact, read, round_trip};

/// The read error of `HexOr<C>` for `json` (the value itself, not through facet).
fn error<C: bedrock_addon_shared_shapes::Channels + std::fmt::Debug>(json: &str) -> ShapeErrorKind {
    let value: Value = bedrock_addon_core::json::from_str_jsonc(json).expect("JSON");
    HexOr::<C>::try_from(value).expect_err("must be rejected")
}

#[test]
fn the_text_form_is_kept_as_written() {
    let color: HexOr<[i32; 3]> = round_trip(r##""#a0B1c2""##);
    assert_eq!(color, HexOr::Hex("#a0B1c2".to_owned()));
    // A text that is no colour is kept, so a later check can report it.
    let color: HexOr<[i32; 3]> = round_trip(r#""red""#);
    assert_eq!(color, HexOr::Hex("red".to_owned()));
    assert_eq!(color.rgb(), None);
}

#[test]
fn integer_channels_are_kept_unclamped_and_clamped_on_use() {
    let color: HexOr<[i32; 3]> = round_trip("[-5, 300, 7]");
    assert_eq!(color, HexOr::Array([-5, 300, 7]));
    assert_eq!(color.rgb(), Some([0, 255, 7]));
    let color: HexOr<[i32; 4]> = round_trip("[1, 2, 3, 400]");
    assert_eq!(color.rgba(), Some([1, 2, 3, 255]));
    let hex: HexOr<[i32; 4]> = round_trip(r##""#01020304""##);
    assert_eq!(hex.rgba(), Some([1, 2, 3, 4]));
    assert_eq!(HexOr::<[i32; 4]>::Hex("#010203".into()).rgba(), None);
}

#[test]
fn an_array_of_the_wrong_shape_is_refused() {
    assert_eq!(
        error::<[i32; 3]>("[1, 2]"),
        ShapeErrorKind::ArrayLength {
            expected: "exactly 3",
            len: 2
        }
    );
    assert_eq!(
        error::<[i32; 4]>("[1, 2, 3]"),
        ShapeErrorKind::ArrayLength {
            expected: "exactly 4",
            len: 3
        }
    );
    assert_eq!(
        error::<[i32; 3]>("[1, 2.0, 3]"),
        ShapeErrorKind::WrongType {
            expected: "a colour string or an array of integers",
            found: JsonType::Number
        }
    );
    assert_eq!(
        error::<[i32; 3]>("7"),
        ShapeErrorKind::WrongType {
            expected: "a colour string or an array of integers",
            found: JsonType::Number
        }
    );
    assert_eq!(
        error::<[i32; 3]>("[1, 2, 3000000000]"),
        ShapeErrorKind::OutOfRange
    );
    assert_eq!(error::<[u8; 3]>("[1, 2, 256]"), ShapeErrorKind::OutOfRange);
    assert_eq!(error::<[u8; 3]>("[1, 2, -1]"), ShapeErrorKind::OutOfRange);
    assert!(read::<HexOr<[i32; 3]>>("[1, 2]").is_err());
}

#[test]
fn byte_and_unit_channels_read_and_write_as_written() {
    let color: HexOr<[u8; 3]> = round_trip("[255, 128, 0]");
    assert_eq!(color.unit_rgb(), [1.0, 128.0 / 255.0, 0.0]);
    let color: HexOr<[f32; 3]> = round_trip("[1.5, -0.2, 0.5]");
    assert_eq!(color, HexOr::Array([1.5, -0.2, 0.5]));
    assert_eq!(color.unit_rgb(), [1.0, 0.0, 0.5]);
    // A unit channel is a number: an integer token reads, and writes as a real.
    let color: HexOr<[f32; 3]> = read("[1, 0, 0.1]").unwrap();
    assert_eq!(color, HexOr::Array([1.0, 0.0, 0.1]));
    assert_eq!(compact(&color), "[1.0,0.0,0.1]");
    // The fog's reading of a text: 0 for a digit pair that is not hex.
    assert_eq!(
        HexOr::<[u8; 3]>::Hex("#ff00zz".into()).unit_rgb(),
        [1.0, 0.0, 0.0]
    );
    assert_eq!(
        HexOr::<[f32; 3]>::Hex("#FF0080".into()).unit_rgb(),
        [1.0, 0.0, 128.0 / 255.0]
    );
}
