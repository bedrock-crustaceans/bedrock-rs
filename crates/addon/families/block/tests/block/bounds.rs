//! Characterization of `Bounds` (`minecraft:random_offset`'s `range`), the pin the plan asks for
//! before it could become core's `FloatRange`. Today it is a `{min, max}` object, read by the
//! component's reader: a number or an array is refused, an end left out is a missing required
//! member (the component does not read), `range_min`/`range_max` are no ends, and a reversed
//! pair is kept as written. `FloatRange` reads all of those otherwise (a number, an array,
//! `range_min`/`range_max`, a missing end it marks, a reversed pair it swaps), so `Bounds` stays.
//!
//! This pins today's code, not the game: no capture of the game reading these forms is at
//! hand. It reads the component through `read_component` as the block's row does, not through
//! the block loader, so it pins none of the findings of `random_offset` in `validate.rs`
//! (`latest.rs` beside this file exercises some of those).

use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::{JsonSlice, from_str_jsonc, to_json};
use bedrock_addon_core::schema::read_value;
use bedrock_addon_core::shared::{FloatRange, RangeForm};
use bedrock_addon_core::versioning::{ConvertMode, read_component};
use bedrock_addon_family_block::component::{BlockComponent, Bounds, OffsetAxis, RandomOffset};

/// `range` read as the component reads its `x` axis.
fn axis(range: &str) -> Option<OffsetAxis> {
    let text = format!(r#"{{"x": {{"range": {range}}}}}"#);
    let mut d = Diagnostics::default();
    let read = read_component::<RandomOffset, BlockComponent>(
        JsonSlice::new(&text),
        ConvertMode::Strict,
        &super::common::ucf(),
        &mut d,
    );
    match read? {
        BlockComponent::RandomOffset(offset) => offset.x,
        _ => None,
    }
}

/// The axis's `range`, read and then written on one line.
fn written(range: &str) -> String {
    let axis = axis(range).expect("the axis reads");
    to_json(&axis).unwrap().split_whitespace().collect()
}

/// `range` read as a `FloatRange` member would be.
fn float_range(range: &str) -> Option<FloatRange> {
    read_value::<FloatRange>(&from_str_jsonc(range).ok()?)
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
fn an_end_left_out_is_a_missing_member() {
    assert_eq!(bounds(r#"{"min": 1}"#), None);
    assert_eq!(bounds(r#"{"max": 1}"#), None);
    assert_eq!(bounds("{}"), None);
    // A FloatRange marks the end it was not given.
    let missing = float_range(r#"{"min": 1}"#).unwrap();
    assert!(missing.missing_max && !missing.missing_min);
}

#[test]
fn range_min_and_range_max_are_no_ends() {
    let text = r#"{"range_min": 1, "range_max": 2}"#;
    assert_eq!(bounds(text), None);
    let range = float_range(text).unwrap();
    assert_eq!(
        (range.min, range.max, range.form),
        (1.0, 2.0, RangeForm::RangeMinMax)
    );
}
