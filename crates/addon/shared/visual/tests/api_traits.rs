//! The std traits the public types of the crate implement (moved from core's `api_traits`).

use bedrock_addon_shared_visual::{Color, Visuals};

fn assert_debug<T: std::fmt::Debug>() {}
fn assert_copy<T: Copy>() {}

#[test]
fn the_borrowing_types_are_debug() {
    assert_debug::<Visuals<'_>>();
    assert_copy::<Visuals<'_>>();
}

#[test]
fn color_defaults_to_nil() {
    assert_eq!(Color::default(), Color::NIL);
}
