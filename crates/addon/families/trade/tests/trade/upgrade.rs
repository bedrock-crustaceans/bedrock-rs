//! The `Upgrade` impl of `v0` (the legacy parser's table) one behaviour at a time: the step runs
//! on a hand-built legacy document and its output is checked.

use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::testing::{Checks, Run, StepOutput, step_output};
use bedrock_addon_family_trade::history::v0;
use bedrock_addon_test_support::json;

/// The JSON at `path` (`/`-separated; a number indexes an array) of `value`.
fn at<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('/')
        .try_fold(value, |v, key| match v.as_array() {
            Some(array) => array.get(key.parse().ok()?),
            None => v.as_object()?.get(key),
        })
}

/// A legacy document with one flat trade whose wanted item is `item` (JSON).
fn wanting(item: &str) -> Value {
    facet_json::from_str(&format!(
        r#"{{"tiers": [{{"trades": [{{"wants": [{item}], "gives": [{{"item": "minecraft:emerald"}}]}}]}}]}}"#
    ))
    .unwrap()
}

/// The written wanted entry of [`wanting`]'s output.
const WANTED: &str = "0/groups/0/trades/0/wants/0";

/// Runs the step on [`wanting`]`(item)` at `version` (the step's own when `None`).
#[track_caller]
fn run(item: &str, version: Option<(u16, u16, u16)>) -> StepOutput {
    let settings = Run {
        original_version: version.map(|(a, b, c)| FormatVersion::new(a, b, c)),
        skip_section_a: false,
    };
    step_output::<v0::TradeTable>(&wanting(item), settings, &Checks::default())
        .unwrap_or_else(|_| panic!("the step failed"))
}

/// Whether `out` has a finding of code `name`.
fn reported(out: &StepOutput, name: &str) -> bool {
    out.diagnostics.iter().any(|f| f.code.id() == name)
}

#[test]
fn flat_trades_become_a_group_of_all() {
    let item = r#"{"item": "minecraft:carrot"}"#;
    let out = run(item, None);
    assert_eq!(
        at(&out.payload, "0/groups/0/num_to_select"),
        Some(&json("-1")),
        "the flat group does not select -1"
    );
    let expected = at(&wanting(item), "tiers/0/trades").cloned();
    assert_eq!(
        at(&out.payload, "0/groups/0/trades"),
        expected.as_ref(),
        "the flat trades are not the group's trades"
    );
    assert!(
        at(&out.payload, "0/trades").is_none() && at(&out.payload, "0/groups/1").is_none(),
        "the flat trades were kept, or a second group written"
    );
}

#[test]
fn legacy_blobs_move_to_the_side_cars() {
    let functions = r#"[{"function": "set_data", "data": 0}]"#;
    let filters = r#"{"test": "is_daytime", "value": true}"#;
    let out = run(
        &format!(
            r#"{{"item": "minecraft:wheat", "functions": {functions}, "filters": {filters}}}"#
        ),
        // Above both gates: nothing else touches the blobs.
        Some((1, 21, 20)),
    );
    assert_eq!(
        at(&out.payload, &format!("{WANTED}/__DEP_fnctns")),
        Some(&json(functions)),
        "functions did not move to __DEP_fnctns verbatim"
    );
    assert_eq!(
        at(&out.payload, &format!("{WANTED}/__DEP_fltrs")),
        Some(&json(filters)),
        "filters did not move to __DEP_fltrs verbatim"
    );
    for typed in ["functions", "filters"] {
        assert!(
            at(&out.payload, &format!("{WANTED}/{typed}")).is_none(),
            "a typed member was written"
        );
    }
    let plain = run(r#"{"item": "minecraft:wheat"}"#, None);
    for member in ["__DEP_fnctns", "__DEP_fltrs"] {
        assert!(
            at(&plain.payload, &format!("{WANTED}/{member}")).is_none(),
            "a side-car was written for an item without the member"
        );
    }
}

#[test]
fn a_quantity_bound_below_one_becomes_one() {
    let item = |quantity: &str| format!(r#"{{"item": "minecraft:wheat", "quantity": {quantity}}}"#);
    let out = run(&item("0"), None);
    assert_eq!(
        at(&out.payload, &format!("{WANTED}/quantity")),
        Some(&json("1")),
        "0 did not become 1"
    );
    assert!(
        reported(&out, "core/invalid_value"),
        "the clamp raised no core/invalid_value"
    );
    let out = run(&item("1"), None);
    assert_eq!(
        at(&out.payload, &format!("{WANTED}/quantity")),
        Some(&json("1")),
        "1 changed"
    );
    assert!(out.diagnostics.is_empty(), "a quantity of 1 was reported");
}

#[test]
fn a_quantity_with_min_above_max_is_ordered() {
    let item = |quantity: &str| format!(r#"{{"item": "minecraft:wheat", "quantity": {quantity}}}"#);
    for (written, expected) in [
        (r#"{"min": 5, "max": 2}"#, r#"{"min": 5, "max": 5}"#),
        ("[5, 2]", r#"{"min": 2, "max": 5}"#),
    ] {
        let out = run(&item(written), None);
        assert_eq!(
            at(&out.payload, &format!("{WANTED}/quantity")),
            Some(&json(expected)),
            "the range was not ordered as vanilla orders it"
        );
        assert!(
            reported(&out, "core/clamped_value"),
            "the reordering raised no core/clamped_value"
        );
    }
    let out = run(&item(r#"{"min": 2, "max": 5}"#), None);
    assert!(out.diagnostics.is_empty(), "an ordered range was reported");
}

#[test]
fn an_absent_quantity_bound_reads_one_and_is_not_written() {
    let out = run(
        r#"{"item": "minecraft:wheat", "quantity": {"max": 3}}"#,
        None,
    );
    assert_eq!(
        at(&out.payload, &format!("{WANTED}/quantity")),
        Some(&json(r#"{"max": 3}"#)),
        "the absent bound was written, or max changed"
    );
    assert!(out.diagnostics.is_empty(), "an absent bound was reported");
}

#[test]
fn a_biomes_filter_becomes_a_generic_filter_below_1_20_30() {
    let item = r#"{"item": "minecraft:boat", "biomes": ["desert", "snow"]}"#;
    let expected = json(
        r#"{"any_of": [
            {"test": "is_mark_variant", "subject": "self", "operator": "=", "value": 1},
            {"test": "is_mark_variant", "subject": "self", "operator": "=", "value": 4}]}"#,
    );
    let below = run(item, Some((1, 20, 20)));
    assert_eq!(
        at(&below.payload, &format!("{WANTED}/__DEP_fltrs")),
        Some(&expected),
        "biomes were not converted below 1.20.30"
    );
    assert!(
        at(&below.payload, &format!("{WANTED}/biomes")).is_none(),
        "biomes was written"
    );
    let at_gate = run(item, Some((1, 20, 30)));
    assert!(
        at(&at_gate.payload, &format!("{WANTED}/__DEP_fltrs")).is_none(),
        "biomes were converted at 1.20.30"
    );
}

#[test]
fn the_trader_material_function_expands_below_1_18_20() {
    let item = r#"{"item": "minecraft:boat", "quantity": 2,
        "functions": [{"function": "trader_material_type"}]}"#;
    let below = run(item, Some((1, 18, 10)));
    let choice = at(&below.payload, &format!("{WANTED}/choice"))
        .and_then(Value::as_array)
        .expect("the entry did not become a choice below 1.18.20");
    let ids: Vec<Option<&Value>> = choice.iter().map(|c| at(c, "item")).collect();
    let expected: Vec<Value> = [0, 1, 3, 4, 5]
        .iter()
        .map(|n| json(&format!(r#""minecraft:boat:{n}""#)))
        .collect();
    assert_eq!(
        ids,
        expected.iter().map(Some).collect::<Vec<_>>(),
        "the five choices are not item:0, :1, :3, :4, :5"
    );
    // Ours: the quantity is kept (vanilla copies `quanitity`).
    assert!(
        choice.iter().all(|c| at(c, "quantity") == Some(&json("2"))),
        "a choice lost its quantity"
    );
    assert!(
        choice.iter().all(|c| at(c, "__DEP_fnctns").is_none()),
        "the emptied functions were kept"
    );
    assert!(
        choice.iter().all(|c| at(c, "__DEP_fltrs").is_some()),
        "a choice has no biome filter"
    );
    let at_gate = run(item, Some((1, 18, 20)));
    assert!(
        at(&at_gate.payload, &format!("{WANTED}/choice")).is_none(),
        "the entry expanded at 1.18.20"
    );
    assert!(
        at(&at_gate.payload, &format!("{WANTED}/__DEP_fnctns")).is_some(),
        "the function was not kept at 1.18.20"
    );
}

/// The trade members the author did not write stay absent after the step.
#[test]
fn trade_defaults_stay_absent() {
    let out = run(r#"{"item": "minecraft:wheat"}"#, None);
    for member in ["trader_exp", "max_uses", "reward_exp", "weight"] {
        assert!(
            at(&out.payload, &format!("0/groups/0/trades/0/{member}")).is_none(),
            "a default the author did not write was written: {member}"
        );
    }
    assert_eq!(
        at(&out.payload, "0/groups/0/trades/0/wants/0/item"),
        Some(&json(r#""minecraft:wheat""#)),
        "the trade was not carried over"
    );
}

/// The deviations of the rules, each with its pinned test.
#[test]
fn every_deviation_test_of_the_rules_exists() {
    let tests = [
        include_str!("quantity.rs"),
        include_str!("gates.rs"),
        include_str!("rules.rs"),
    ]
    .concat();
    for test in [
        "a_legacy_bound_below_one_becomes_one_with_an_information_deviation",
        "a_legacy_min_above_max_is_swapped_or_raised_with_an_information_deviation",
        "a_legacy_object_quantity_without_a_bound_reads_one_for_it_deviation",
        "t05_the_expanded_choices_keep_their_quantity_deviation",
        "t03_num_to_select_is_an_error_from_1_21_30_too_deviation",
        "empty_tiers_is_an_error_deviation",
        "the_weight_note_from_1_21_30_is_an_information_deviation",
    ] {
        assert!(tests.contains(&format!("fn {test}()")), "{test}");
    }
}
