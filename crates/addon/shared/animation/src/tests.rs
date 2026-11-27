//! Unit tests of the crate-private parts; the fixture rows are in `tests/animation.rs` and
//! `tests/animation_controller.rs`.

use super::read::text_of;

#[test]
fn a_root_with_comments_is_read_at_its_value() {
    let text = text_of(b"// note\n { \"a\": 1 }").unwrap();
    assert_eq!(
        text.value.as_object().map(facet_value::VObject::len),
        Some(1)
    );
}
