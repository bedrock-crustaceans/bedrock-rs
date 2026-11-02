//! The schema's constraints: 1 to 32 boxes, an identifier of at most 256
//! bytes. Rows vs19, vs20, vs22.

use super::common::*;

const BOXES: &str = "minecraft:voxel_shape/shape/boxes";
const IDENTIFIER: &str = "minecraft:voxel_shape/description/identifier";

/// A document of `count` boxes, every one the lower half of the block.
fn of_boxes(count: usize) -> String {
    let boxes = vec![("[0, 0, 0]", "[16, 8, 16]"); count];
    shape("test:many_boxes", &boxes)
}

/// A document whose identifier is `identifier`.
fn named(identifier: &str) -> String {
    shape(identifier, &[("[0, 0, 0]", "[16, 8, 16]")])
}

#[test]
fn vs19_a_33rd_box_is_one_too_many() {
    assert_eq!(
        errors(&rejects(&of_boxes(33))),
        [
            error(
                "InvalidValue",
                " -> shape -> boxes: too many input elements, expected no more than 32",
                BOXES
            ),
            failed()
        ]
    );
}

#[test]
fn a_shape_may_have_32_boxes() {
    assert_eq!(loads(&of_boxes(32)).shape.boxes.len(), 32);
}

#[test]
fn vs20_a_shape_without_a_box_fails_the_file() {
    assert_eq!(
        kinds(&rejects(&of_boxes(0))),
        [kind("InvalidValue", BOXES), kind("DocumentRejected", "")]
    );
}

#[test]
fn a_shape_may_have_one_box() {
    assert_eq!(loads(&of_boxes(1)).shape.boxes.len(), 1);
}

#[test]
fn vs22_an_identifier_of_257_bytes_fails_the_file() {
    let long = format!("test:{}", "a".repeat(252));
    assert_eq!(long.len(), 257);
    assert_eq!(
        kinds(&rejects(&named(&long))),
        [
            kind("InvalidValue", IDENTIFIER),
            kind("DocumentRejected", "")
        ]
    );
}

#[test]
fn an_identifier_may_have_256_bytes() {
    let longest = format!("test:{}", "a".repeat(251));
    assert_eq!(longest.len(), 256);
    let shape = loads(&named(&longest));
    assert_eq!(shape.description.identifier.as_str(), longest);
}

#[test]
fn the_identifier_s_length_is_counted_in_bytes() {
    // The binary's `StringConstraint::maxSize(256)` compares the string's byte length.
    // 256 characters of two bytes each are 512 bytes: too long.
    let wide = "é".repeat(256);
    assert_eq!((wide.chars().count(), wide.len()), (256, 512));
    assert_eq!(
        kinds(&rejects(&named(&wide))),
        [
            kind("InvalidValue", IDENTIFIER),
            kind("DocumentRejected", "")
        ]
    );
    // At the boundary with multi-byte characters: 255 and 256 bytes load, 257 do not.
    let bytes_255 = format!("a{}", "é".repeat(127));
    let bytes_256 = "é".repeat(128);
    let bytes_257 = format!("a{}", "é".repeat(128));
    assert_eq!(
        (bytes_255.len(), bytes_256.len(), bytes_257.len()),
        (255, 256, 257)
    );
    loads(&named(&bytes_255));
    loads(&named(&bytes_256));
    assert_eq!(
        kinds(&rejects(&named(&bytes_257))),
        [
            kind("InvalidValue", IDENTIFIER),
            kind("DocumentRejected", "")
        ]
    );
}

#[test]
fn an_empty_identifier_is_not_checked() {
    // Vanilla checks only the length; a custom pack's name need not have a namespace.
    let shape = loads(&named(""));
    assert_eq!(shape.description.identifier.as_str(), "");
}
