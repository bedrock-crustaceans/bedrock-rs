//! Quantities (fixture t04): the legacy forms, their clamps, and
//! the strict forms from 1.21.30 on.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_family_trade::Quantity;

use super::common::{doc, fixture, found_at, loads, rejects, trade, with};

/// The `wants` path of entry `n` of the t04 trade.
fn wants(n: usize) -> String {
    format!("tiers/0/trades/0/wants/{n}/quantity")
}

#[test]
fn t04_every_legacy_form_reads_as_parse_range_does() {
    let document = loads(&fixture("t04_quantity_forms.json"));
    let ranges: Vec<(u32, u32)> = trade(document.value(), 0, 0, 0)
        .wants
        .iter()
        .map(|list| list.items()[0].quantity_range())
        .collect();
    assert_eq!(
        ranges,
        [(3, 3), (2, 4), (2, 5), (5, 5), (1, 1), (1, 1), (1, 1)]
    );
    let d = document.diagnostics();
    // Op 5: the array swaps, the object raises `max` (ours: a warning; vanilla is silent).
    found_at(d, "ClampedValue", Severity::Info, &wants(2));
    found_at(d, "ClampedValue", Severity::Info, &wants(3));
    // Op 4: a bound of 0 or less becomes 1 (ours: a warning; vanilla is silent).
    found_at(d, "InvalidValue", Severity::Info, &wants(4));
    found_at(d, "InvalidValue", Severity::Info, &wants(5));
    assert_eq!(d.len(), 4, "{d:#?}");
}

#[test]
fn t04_a_stamped_string_or_array_quantity_is_a_type_error() {
    for quantity in [r#""2-4""#, "[2, 4]"] {
        let d = rejects(&doc(
            Some("1.21.30"),
            &format!(
                r#"[{{"total_exp_required": 0, "groups": [{{"trades": [{{
                    "wants": [{{"item": "minecraft:wheat", "quantity": {quantity}}}],
                    "gives": [{{"item": "minecraft:emerald"}}]}}]}}]}}]"#
            ),
        ));
        let at = "tiers/0/groups/0/trades/0/wants/0/quantity";
        found_at(&d, "InvalidType", Severity::Error, at);
    }
}

#[test]
fn t04_a_stamped_zero_quantity_violates_min_1() {
    for quantity in ["0", r#"{"min": 0, "max": 2}"#, r#"{"max": 0}"#] {
        let d = rejects(&doc(
            Some("1.21.30"),
            &format!(
                r#"[{{"total_exp_required": 0, "groups": [{{"trades": [{{
                    "wants": [{{"item": "minecraft:wheat", "quantity": {quantity}}}],
                    "gives": [{{"item": "minecraft:emerald"}}]}}]}}]}}]"#
            ),
        ));
        assert_eq!(
            with(&d, "InvalidValue", Severity::Error).len(),
            1,
            "{quantity}: {d:#?}"
        );
    }
}

/// Loads an unstamped one-trade table whose wanted item has `quantity` (JSON); the quantity,
/// its range and the codes of the findings.
fn legacy(quantity: &str) -> (Option<Quantity>, (u32, u32), Vec<&'static str>) {
    let document = loads(&doc(
        None,
        &format!(
            r#"[{{"trades": [{{"wants": [{{"item": "minecraft:wheat", "quantity": {quantity}}}],
                "gives": [{{"item": "minecraft:emerald"}}]}}]}}]"#
        ),
    ));
    let item = document.value().tiers[0].groups[0].trades[0].wants[0].items()[0].clone();
    let codes = document
        .diagnostics()
        .iter()
        .map(|f| {
            assert_eq!(f.severity, Severity::Info);
            assert_eq!(
                f.location.json_path.to_string(),
                "tiers/0/trades/0/wants/0/quantity"
            );
            f.code.name()
        })
        .collect();
    (item.quantity, item.quantity_range(), codes)
}

/// `{"min", "max"}` as written.
fn range(min: Option<u32>, max: Option<u32>) -> Quantity {
    Quantity::Range { min, max }
}

#[test]
fn a_legacy_bound_below_one_becomes_one_with_an_information_deviation() {
    // Vanilla clamps silently ().
    assert_eq!(legacy("1"), (Some(Quantity::Exact(1)), (1, 1), vec![]));
    assert_eq!(
        legacy("0"),
        (Some(Quantity::Exact(1)), (1, 1), vec!["InvalidValue"])
    );
    assert_eq!(
        legacy(r#"{"min": 0, "max": 3}"#),
        (Some(range(Some(1), Some(3))), (1, 3), vec!["InvalidValue"])
    );
    assert_eq!(
        legacy(r#"{"min": 1, "max": 3}"#),
        (Some(range(Some(1), Some(3))), (1, 3), vec![])
    );
    assert_eq!(
        legacy(r#""-2-3""#),
        (Some(range(Some(1), Some(3))), (1, 3), vec!["InvalidValue"])
    );
    assert_eq!(
        legacy("[]"),
        (Some(range(Some(1), Some(1))), (1, 1), vec!["InvalidValue"])
    );
    // `max` alone below 1: it is below `min` too, so it is raised as well.
    assert_eq!(
        legacy(r#"{"min": 2, "max": 0}"#),
        (
            Some(range(Some(2), Some(2))),
            (2, 2),
            vec!["InvalidValue", "ClampedValue"]
        )
    );
    assert_eq!(
        legacy(r#"{"max": 0}"#),
        (Some(range(None, Some(1))), (1, 1), vec!["InvalidValue"])
    );
}

#[test]
fn a_legacy_min_above_max_is_swapped_or_raised_with_an_information_deviation() {
    // Vanilla is silent (). The array swaps, the others raise `max`.
    assert_eq!(
        legacy("[5, 2]"),
        (Some(range(Some(2), Some(5))), (2, 5), vec!["ClampedValue"])
    );
    assert_eq!(
        legacy("[2, 5]"),
        (Some(range(Some(2), Some(5))), (2, 5), vec![])
    );
    assert_eq!(
        legacy("[3, 3]"),
        (Some(range(Some(3), Some(3))), (3, 3), vec![])
    );
    assert_eq!(
        legacy(r#"{"min": 5, "max": 2}"#),
        (Some(range(Some(5), Some(5))), (5, 5), vec!["ClampedValue"])
    );
    assert_eq!(
        legacy(r#"{"min": 5, "max": 5}"#),
        (Some(range(Some(5), Some(5))), (5, 5), vec![])
    );
    assert_eq!(
        legacy(r#""4-2""#),
        (Some(range(Some(4), Some(4))), (4, 4), vec!["ClampedValue"])
    );
    assert_eq!(
        legacy(r#""2-4""#),
        (Some(range(Some(2), Some(4))), (2, 4), vec![])
    );
}

#[test]
fn a_legacy_object_quantity_without_a_bound_reads_one_for_it_deviation() {
    // Vanilla reads an uninitialised int (); ours: 1,
    // the newest default, and the bound stays absent.
    assert_eq!(
        legacy(r#"{"max": 3}"#),
        (Some(range(None, Some(3))), (1, 3), vec![])
    );
    assert_eq!(legacy("{}"), (Some(range(None, None)), (1, 1), vec![]));
    // `max` follows a larger `min`; nothing the author wrote was changed, so no warning.
    assert_eq!(
        legacy(r#"{"min": 4}"#),
        (Some(range(Some(4), Some(4))), (4, 4), vec![])
    );
}

#[test]
fn the_other_legacy_forms() {
    // A lone number in a string, a real, a one-element array
    // (`[1]` reads 0).
    assert_eq!(
        legacy(r#""5""#),
        (Some(range(Some(5), Some(5))), (5, 5), vec![])
    );
    assert_eq!(
        legacy("2.7"),
        (Some(Quantity::Exact(2)), (2, 2), vec!["InvalidValue"])
    );
    assert_eq!(legacy("2.0"), (Some(Quantity::Exact(2)), (2, 2), vec![]));
    // A bound beyond the newest form's `u32`, or beyond an `int`: the bound of the type, said.
    assert_eq!(
        legacy("4294967296"),
        (
            Some(Quantity::Exact(u32::MAX)),
            (u32::MAX, u32::MAX),
            vec!["InvalidValue"]
        )
    );
    assert_eq!(
        legacy(r#"{"min": 1, "max": 1e39}"#),
        (
            Some(range(Some(1), Some(2_147_483_647))),
            (1, 2_147_483_647),
            vec!["InvalidValue"]
        )
    );
    assert_eq!(
        legacy("[5]"),
        (
            Some(range(Some(1), Some(5))),
            (1, 5),
            vec!["InvalidValue", "ClampedValue"]
        )
    );
    // A string that is no range, and a type `parseRange` does not read: 1..1 (vanilla: in
    // silence; ours says it).
    assert_eq!(
        legacy(r#""many""#),
        (Some(range(Some(1), Some(1))), (1, 1), vec!["InvalidValue"])
    );
    assert_eq!(legacy("true"), (None, (1, 1), vec!["InvalidType"]));
}

#[test]
fn a_stamped_quantity_of_one_or_more_loads_in_both_forms() {
    for (quantity, expected) in [
        ("1", Quantity::Exact(1)),
        ("64", Quantity::Exact(64)),
        (
            r#"{"min": 1, "max": 1}"#,
            Quantity::Range {
                min: Some(1),
                max: Some(1),
            },
        ),
        (
            r"{}",
            Quantity::Range {
                min: None,
                max: None,
            },
        ),
    ] {
        let document = loads(&doc(
            Some("1.21.30"),
            &format!(
                r#"[{{"total_exp_required": 0, "groups": [{{"trades": [{{
                    "wants": [{{"item": "minecraft:wheat", "quantity": {quantity}}}],
                    "gives": [{{"item": "minecraft:emerald"}}]}}]}}]}}]"#
            ),
        ));
        assert!(document.diagnostics().is_empty());
        let item = &document.value().tiers[0].groups[0].trades[0].wants[0].items()[0];
        assert_eq!(item.quantity, Some(expected));
    }
}

#[test]
fn a_stamped_min_above_max_loads_as_written_the_folder_gives_no_order_rule() {
    // bound is stated, so `min > max` is neither rejected nor reordered.
    let document = loads(&doc(
        Some("1.21.30"),
        r#"[{"total_exp_required": 0, "groups": [{"trades": [{
            "wants": [{"item": "minecraft:wheat", "quantity": {"min": 5, "max": 2}}],
            "gives": [{"item": "minecraft:emerald"}]}]}]}]"#,
    ));
    assert!(document.diagnostics().is_empty());
    let item = &document.value().tiers[0].groups[0].trades[0].wants[0].items()[0];
    assert_eq!(
        item.quantity,
        Some(Quantity::Range {
            min: Some(5),
            max: Some(2)
        })
    );
    assert_eq!(item.quantity_range(), (5, 2));
}

#[test]
fn a_stamped_negative_or_fractional_quantity_is_an_error() {
    for quantity in ["-1", "1.5", "true", r#"{"min": "1"}"#] {
        let d = rejects(&doc(
            Some("1.21.30"),
            &format!(
                r#"[{{"total_exp_required": 0, "groups": [{{"trades": [{{
                    "wants": [{{"item": "minecraft:wheat", "quantity": {quantity}}}],
                    "gives": [{{"item": "minecraft:emerald"}}]}}]}}]}}]"#
            ),
        ));
        let errors: Vec<_> = d.iter().filter(|f| f.severity == Severity::Error).collect();
        assert_eq!(errors.len(), 1, "{quantity}: {d:#?}");
        assert!(
            errors[0]
                .location
                .json_path
                .to_string()
                .starts_with("tiers/0/groups/0/trades/0/wants/0/quantity"),
            "{quantity}"
        );
    }
}

/// Both forms of a stamped quantity carry `min(1)`: a negative integer breaks the constraint
/// whether it is the quantity or one of its bounds, and a value that is no `u32` at all is of
/// the wrong type in both.
#[test]
fn a_negative_bound_is_an_invalid_value_like_a_negative_quantity() {
    let quantity = "tiers/0/groups/0/trades/0/wants/0/quantity";
    for (text, code, path) in [
        ("-1", "InvalidValue", quantity.to_owned()),
        (r#"{"min": -1}"#, "InvalidValue", format!("{quantity}/min")),
        (r#"{"max": -1}"#, "InvalidValue", format!("{quantity}/max")),
        (
            r#"{"min": 2, "max": -3}"#,
            "InvalidValue",
            format!("{quantity}/max"),
        ),
        ("0", "InvalidValue", quantity.to_owned()),
        (r#"{"min": 0}"#, "InvalidValue", format!("{quantity}/min")),
        ("4294967296", "InvalidType", quantity.to_owned()),
        (
            r#"{"min": 4294967296}"#,
            "InvalidType",
            format!("{quantity}/min"),
        ),
        (
            r#"{"max": 4294967296}"#,
            "InvalidType",
            format!("{quantity}/max"),
        ),
        (r#""1""#, "InvalidType", quantity.to_owned()),
        (r#"{"min": "1"}"#, "InvalidType", format!("{quantity}/min")),
        (r#"{"max": [1]}"#, "InvalidType", format!("{quantity}/max")),
        ("1.5", "InvalidType", quantity.to_owned()),
        (r#"{"max": 1.5}"#, "InvalidType", format!("{quantity}/max")),
    ] {
        let d = rejects(&doc(
            Some("1.21.30"),
            &format!(
                r#"[{{"total_exp_required": 0, "groups": [{{"trades": [{{
                    "wants": [{{"item": "minecraft:wheat", "quantity": {text}}}],
                    "gives": [{{"item": "minecraft:emerald"}}]}}]}}]}}]"#
            ),
        ));
        let errors: Vec<_> = d
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .map(|f| (f.code.name().to_owned(), f.location.json_path.to_string()))
            .collect();
        assert_eq!(errors, [(code.to_owned(), path)], "{text}");
    }
    // The largest `u32` is a quantity in both forms.
    for text in ["4294967295", r#"{"min": 1, "max": 4294967295}"#] {
        loads(&doc(
            Some("1.21.30"),
            &format!(
                r#"[{{"total_exp_required": 0, "groups": [{{"trades": [{{
                    "wants": [{{"item": "minecraft:wheat", "quantity": {text}}}],
                    "gives": [{{"item": "minecraft:emerald"}}]}}]}}]}}]"#
            ),
        ));
    }
}
