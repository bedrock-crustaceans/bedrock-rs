//! Characterization of `Bounds` (`minecraft:random_offset`'s `range`), the pin the plan asks for
//! before it could become core's `FloatRange`. Today it is a `{min, max}` object read by facet:
//! a number or an array is refused, an end left out is 0 (the struct derives `Default`) and is
//! written, `range_min`/`range_max` are no ends, and a reversed pair is kept as written.
//! `FloatRange` reads all of those otherwise (a number, an array, `range_min`/`range_max`, a
//! missing end it marks, a reversed pair it swaps), so `Bounds` stays.
//!
//! This pins today's code, not the game: no capture of the game reading these forms is at
//! hand. It reads `OffsetAxis` through facet directly, not through the block loader, so it
//! pins neither what `parse_component` reports for a refused `range` nor the findings of
//! `random_offset` in `validate.rs` (`latest.rs` beside this file exercises some of those).

use bedrock_addon_core::json::{from_str_jsonc, to_json};
use bedrock_addon_core::shared::{FloatRange, RangeForm};
use bedrock_addon_family_block::component::{Bounds, OffsetAxis};

/// `range` read as the axis reads it.
fn axis(range: &str) -> Option<OffsetAxis> {
    from_str_jsonc::<OffsetAxis>(&format!(r#"{{"range": {range}}}"#)).ok()
}

/// The axis's `range`, read and then written on one line.
fn written(range: &str) -> String {
    let axis = axis(range).expect("the axis reads");
    to_json(&axis).unwrap().split_whitespace().collect()
}

/// `range` read as a `FloatRange` member would be.
fn float_range(range: &str) -> Option<FloatRange> {
    #[derive(facet::Facet)]
    struct Holder {
        range: FloatRange,
    }
    from_str_jsonc::<Holder>(&format!(r#"{{"range": {range}}}"#))
        .ok()
        .map(|holder| holder.range)
}

/// The ends `range` reads to as `Bounds`.
fn bounds(range: &str) -> Option<(f32, f32)> {
    axis(range).map(|axis| (axis.range.min, axis.range.max))
}

#[test]
fn bounds_is_an_object_of_two_ends_kept_as_written() {
    assert_eq!(
        axis(r#"{"min": -2, "max": 3.5}"#).map(|axis| axis.range),
        Some(Bounds {
            min: -2.0,
            max: 3.5
        })
    );
    assert_eq!(
        written(r#"{"min": -2, "max": 3.5}"#),
        r#"{"range":{"min":-2.0,"max":3.5}}"#
    );
    // A reversed pair is kept as written; a FloatRange swaps it and says so.
    assert_eq!(bounds(r#"{"min": 4, "max": 1}"#), Some((4.0, 1.0)));
    let swapped = float_range(r#"{"min": 4, "max": 1}"#).unwrap();
    assert_eq!(
        (swapped.min, swapped.max, swapped.swapped),
        (1.0, 4.0, true)
    );
}

#[test]
fn a_number_or_an_array_is_no_bounds() {
    for other in ["1.5", "[1, 2]"] {
        assert_eq!(bounds(other), None, "{other}");
        assert!(float_range(other).is_some(), "{other}");
    }
}

#[test]
fn an_end_left_out_is_zero_and_written() {
    assert_eq!(bounds(r#"{"min": 1}"#), Some((1.0, 0.0)));
    assert_eq!(
        written(r#"{"min": 1}"#),
        r#"{"range":{"min":1.0,"max":0.0}}"#
    );
    assert_eq!(bounds(r#"{"max": 1}"#), Some((0.0, 1.0)));
    assert_eq!(bounds("{}"), Some((0.0, 0.0)));
    // A FloatRange marks the end it was not given.
    let missing = float_range(r#"{"min": 1}"#).unwrap();
    assert!(missing.missing_max && !missing.missing_min);
}

#[test]
fn range_min_and_range_max_are_no_ends() {
    let text = r#"{"range_min": 1, "range_max": 2}"#;
    assert_eq!(bounds(text), Some((0.0, 0.0)));
    let range = float_range(text).unwrap();
    assert_eq!(
        (range.min, range.max, range.form),
        (1.0, 2.0, RangeForm::RangeMinMax)
    );
}
