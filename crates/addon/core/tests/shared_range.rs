//! `IntRange` / `FloatRange`: every JSON form round trips with its `form`; a range with
//! `max < min` is swapped, with an `InvalidValue` info; and `range_min` / `range_max` (read by
//! one of the game's JSON readers) and `min` / `max` (read by another) are one type that
//! records which spelling was read.

#[path = "shared/common.rs"]
mod common;

use bedrock_addon_core::diagnostic::{DiagnosticCode, JsonPath, Severity};
use bedrock_addon_core::shared::{FloatRange, IntRange, Range, RangeForm};
use common::{compact, read, round_trip};

#[test]
fn a_number_is_both_ends() {
    let range: IntRange = round_trip("5");
    assert_eq!(
        (range.min, range.max, range.form),
        (5, 5, RangeForm::Scalar)
    );
    let range: FloatRange = round_trip("0.5");
    assert_eq!(
        (range.min, range.max, range.form),
        (0.5, 0.5, RangeForm::Scalar)
    );
}

#[test]
fn a_two_element_array_is_min_then_max() {
    let range: IntRange = round_trip("[1, 4]");
    assert_eq!((range.min, range.max, range.form), (1, 4, RangeForm::Array));
    let range: FloatRange = round_trip("[0.25, 1.5]");
    assert_eq!(
        (range.min, range.max, range.form),
        (0.25, 1.5, RangeForm::Array)
    );
}

#[test]
fn d4_jsonutil_readers_use_range_min_and_range_max() {
    let range: IntRange = round_trip(r#"{"range_min": 2, "range_max": 7}"#);
    assert_eq!((range.min, range.max), (2, 7));
    assert_eq!(range.form, RangeForm::RangeMinMax);
    let range: FloatRange = round_trip(r#"{"range_min": 0.1, "range_max": 0.9}"#);
    assert_eq!((range.min, range.max), (0.1, 0.9));
    assert_eq!(range.form, RangeForm::RangeMinMax);
}

#[test]
fn d4_cereal_readers_use_min_and_max() {
    let range: FloatRange = round_trip(r#"{"min": -0.5, "max": 2.0}"#);
    assert_eq!((range.min, range.max), (-0.5, 2.0));
    assert_eq!(range.form, RangeForm::MinMax);
    let range: IntRange = round_trip(r#"{"min": 2, "max": 4}"#);
    assert_eq!(range.form, RangeForm::MinMax);
}

#[test]
fn d1_max_below_min_is_swapped_with_an_info() {
    let range: IntRange = read(r#"{"range_min": 9, "range_max": 3}"#).unwrap();
    assert_eq!((range.min, range.max), (3, 9));
    assert!(range.swapped);
    let found = range.check(&JsonPath::new());
    let finding = found.iter().next().expect("one finding");
    assert!(matches!(
        finding.code,
        DiagnosticCode::InvalidValue { text: None }
    ));
    assert_eq!(finding.severity, Severity::Info);
    assert_eq!(found.len(), 1);
    // Written back in the document's order.
    assert_eq!(compact(&range), r#"{"range_min":9,"range_max":3}"#);

    let range: IntRange = round_trip("[5, 1]");
    assert_eq!((range.min, range.max, range.swapped), (1, 5, true));
    let range: FloatRange = round_trip(r#"{"min": 1.5, "max": 0.5}"#);
    assert_eq!((range.min, range.max, range.swapped), (0.5, 1.5, true));
}

#[test]
fn an_ordered_range_has_no_finding() {
    let range: IntRange = read("[1, 5]").unwrap();
    assert!(!range.swapped);
    assert!(range.check(&JsonPath::new()).is_empty());
}

#[test]
fn a_missing_end_takes_the_members_default_when_resolved() {
    // `IntRange::parseJson(json, lo, hi)`: each absent end is the member's default.
    let range: IntRange = round_trip(r#"{"range_min": 2}"#);
    assert!(!range.missing_min && range.missing_max);
    assert_eq!(range.resolve(0, i32::MAX), (2, i32::MAX));
    let range: IntRange = round_trip("{}");
    assert!(range.missing_min && range.missing_max);
    assert_eq!(range.resolve(1, 3), (1, 3));
    // The swap applies after the defaults.
    let range: IntRange = round_trip(r#"{"range_max": -4}"#);
    assert_eq!(range.resolve(0, 10), (-4, 0));
}

#[test]
fn an_int_range_takes_the_first_two_elements_of_a_longer_array() {
    // `IntRange::parseJson`: "an array of ≥ 2 → [a[0], a[1]]".
    let range: IntRange = read("[1, 2, 3]").unwrap();
    assert_eq!((range.min, range.max, range.form), (1, 2, RangeForm::Array));
    assert!(read::<IntRange>("[1]").is_err());
    assert!(read::<IntRange>("[]").is_err());
}

#[test]
fn an_int_range_array_longer_than_two_is_written_back_with_two_elements_deviation() {
    // Vanilla reads `[a[0], a[1]]` and ignores the rest; the rest is not written back.
    let range: IntRange = read("[1, 2, 3]").unwrap();
    assert_eq!(compact(&range), "[1,2]");
    let again: IntRange = read(&compact(&range)).unwrap();
    assert_eq!(again, range);
}

#[test]
fn a_float_range_array_of_another_length_is_a_read_error_deviation() {
    // "Float range arrays should have 2 values.": vanilla logs it and keeps the member's
    // default; the crate's own reader fails with a read error, which the family reports as
    // `InvalidType`.
    assert!(read::<FloatRange>("[1.0]").is_err());
    assert!(read::<FloatRange>("[1.0, 2.0, 3.0]").is_err());
}

#[test]
fn int_range_ends_are_strict_integers() {
    for json in [
        "1.0",
        "\"1\"",
        "true",
        "[1, 2.5]",
        r#"{"min": "1"}"#,
        "3000000000",
    ] {
        assert!(read::<IntRange>(json).is_err(), "{json} must be rejected");
    }
}

#[test]
fn float_range_ends_take_integers_and_reals_only() {
    let range: FloatRange = read("[1, 2]").unwrap();
    assert_eq!((range.min, range.max), (1.0, 2.0));
    // An `f32` is written with its real tag, which every float reader takes.
    assert_eq!(compact(&range), "[1.0,2.0]");
    for json in ["\"1\"", "true", "[1, \"2\"]", "1e39", "null"] {
        assert!(read::<FloatRange>(json).is_err(), "{json} must be rejected");
    }
}

#[test]
fn null_in_an_optional_member_is_absent() {
    let range: Option<IntRange> = read("null").unwrap();
    assert_eq!(range, None);
}

#[test]
fn a_float_keeps_its_shortest_form() {
    let range: FloatRange = read("[0.1, 0.3]").unwrap();
    assert_eq!(compact(&range), "[0.1,0.3]");
}

#[test]
fn a_constructed_range_writes_its_form() {
    let range = Range::new(1, 3, RangeForm::RangeMinMax);
    assert_eq!(compact(&range), r#"{"range_min":1,"range_max":3}"#);
    let range: FloatRange = Range::new(1.0, 1.0, RangeForm::Scalar);
    assert_eq!(compact(&range), "1.0");
    // A scalar form with two different ends has no scalar spelling: written as an array.
    let range = Range::new(1, 3, RangeForm::Scalar);
    assert_eq!(compact(&range), "[1,3]");
}

#[test]
fn an_unknown_member_of_a_range_object_is_dropped_deviation() {
    // A range has no `extra`; vanilla's readers ignore the member.
    let range: IntRange = read(r#"{"range_min": 1, "range_max": 2, "note": true}"#).unwrap();
    assert_eq!(compact(&range), r#"{"range_min":1,"range_max":2}"#);
}
