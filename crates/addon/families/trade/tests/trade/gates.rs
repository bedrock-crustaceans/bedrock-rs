//! The two `original_version` gates of the legacy parser: below 1.18.20 the
//! `trader_material_type` expansion, below 1.20.30 `biomes` → `filters.any_of` (fixtures t05,
//! t06 and the F.6.6 list).

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_family_trade::{ItemList, Quantity, TradeItem, TradeTable};

use super::common::{doc, found_at, loads, trade};

/// A one-trade table whose `wants` entry is `entry` (JSON).
fn table_with(version: Option<&str>, entry: &str) -> String {
    doc(
        version,
        &format!(
            r#"[{{"trades": [{{"wants": [{entry}], "gives": [{{"item": "minecraft:emerald"}}]}}]}}]"#
        ),
    )
}

/// The items of the first `wants` entry.
fn wanted(table: &TradeTable) -> &[TradeItem] {
    trade(table, 0, 0, 0).wants[0].items()
}

/// `{"any_of": [is_mark_variant tests of values]}`.
fn mark_variants(values: &[i32]) -> Value {
    let tests: Vec<String> = values
        .iter()
        .map(|v| {
            format!(
                r#"{{"test": "is_mark_variant", "subject": "self", "operator": "=", "value": {v}}}"#
            )
        })
        .collect();
    facet_json::from_str(&format!(r#"{{"any_of": [{}]}}"#, tests.join(","))).unwrap()
}

/// Whether `finding` is the information that a legacy blob is written under its deprecated
/// name (`round_trip.rs`, the `k107_` test): not what these tests are about.
fn announcement(finding: &bedrock_addon_core::diagnostic::Diagnostic) -> bool {
    finding.code.name() == "ValueRewritten" && finding.severity == Severity::Info
}

/// JSON text as a `Value`.
fn json(text: &str) -> Value {
    facet_json::from_str(text).unwrap()
}

#[test]
fn t06_biomes_become_an_any_of_of_mark_variant_tests_below_1_20_30() {
    let document = loads(&table_with(
        Some("1.20.20"),
        r#"{"item": "minecraft:boat", "biomes": ["desert", "snow"]}"#,
    ));
    assert!(
        document.diagnostics().iter().all(announcement),
        "{:#?}",
        document.diagnostics()
    );
    let item = &wanted(document.value())[0];
    // The converted filter is a legacy one: it moves to `__DEP_fltrs` with the rest (op 3).
    assert_eq!(item.legacy_filters, Some(mark_variants(&[1, 4])));
    assert_eq!(item.filters, None);
}

#[test]
fn the_biomes_gate_is_below_1_20_30_exactly() {
    // The last version below the gate still converts; the gate version itself does not (F.6.4:
    // `version < 1.20.30`).
    for (version, converted) in [
        ("1.20.21", true),
        ("1.20.29", true),
        ("1.20.30", false),
        ("1.20.31", false),
    ] {
        let document = loads(&table_with(
            Some(version),
            r#"{"item": "minecraft:boat", "biomes": ["desert"]}"#,
        ));
        let expected = converted.then(|| mark_variants(&[1]));
        assert_eq!(
            wanted(document.value())[0].legacy_filters,
            expected,
            "{version}"
        );
    }
}

#[test]
fn f6_6_item_3_biomes_are_not_converted_from_1_20_30() {
    let document = loads(&table_with(
        Some("1.20.30"),
        r#"{"item": "minecraft:boat", "biomes": ["desert"], "filters": {"test": "is_daytime"}}"#,
    ));
    let item = &wanted(document.value())[0];
    assert_eq!(item.legacy_filters, Some(json(r#"{"test": "is_daytime"}"#)));
}

#[test]
fn f6_6_item_7_converted_biomes_replace_existing_filters() {
    let document = loads(&table_with(
        Some("1.20.0"),
        r#"{"item": "minecraft:boat", "biomes": "taiga", "filters": {"test": "is_daytime"}}"#,
    ));
    let item = &wanted(document.value())[0];
    assert_eq!(item.legacy_filters, Some(mark_variants(&[6])));
}

#[test]
fn unknown_biome_names_add_nothing_and_leave_filters_alone() {
    let document = loads(&table_with(
        Some("1.20.0"),
        r#"{"item": "minecraft:boat", "biomes": ["mesa", "plains", "ocean"]}"#,
    ));
    assert_eq!(
        wanted(document.value())[0].legacy_filters,
        Some(mark_variants(&[0]))
    );
    let document = loads(&table_with(
        Some("1.20.0"),
        r#"{"item": "minecraft:boat", "biomes": ["ocean"], "filters": {"test": "is_daytime"}}"#,
    ));
    assert_eq!(
        wanted(document.value())[0].legacy_filters,
        Some(json(r#"{"test": "is_daytime"}"#))
    );
}

#[test]
fn every_biome_name_maps_to_its_mark_variant() {
    let document = loads(&table_with(
        None,
        r#"{"choice": [{"item": "minecraft:boat", "biomes": ["plains", "desert", "jungle", "savannah", "snow", "swamp", "taiga"]}]}"#,
    ));
    assert_eq!(
        wanted(document.value())[0].legacy_filters,
        Some(mark_variants(&[0, 1, 2, 3, 4, 5, 6]))
    );
}

/// A `wants` entry that asks for the trader's material.
const MATERIAL: &str = r#"{"item": "minecraft:boat", "quantity": 2, "price_multiplier": 0.05,
    "filters": {"test": "is_daytime"},
    "functions": [{"function": "trader_material_type"}, {"function": "set_data", "data": 1}]}"#;

/// The five choices of an expansion: the aux suffix and the mark variants of its biomes.
const EXPANSION: [(&str, &[i32]); 5] = [
    ("minecraft:boat:0", &[0]),
    ("minecraft:boat:1", &[6, 4]),
    ("minecraft:boat:3", &[1, 2]),
    ("minecraft:boat:4", &[3]),
    ("minecraft:boat:5", &[5]),
];

#[test]
fn t05_trader_material_type_expands_into_five_biome_filtered_choices_at_1_18_10() {
    let document = loads(&table_with(Some("1.18.10"), MATERIAL));
    // The source's `filters` is not copied into the choices; ours:
    // the member dropped is told, and nothing else is.
    let told: Vec<String> = document
        .diagnostics()
        .iter()
        .filter(|f| !announcement(f))
        .map(|f| format!("{} at {}", f.code.name(), f.location.json_path))
        .collect();
    assert_eq!(told.len(), 1, "{told:?}");
    assert!(
        told[0].starts_with("DroppedField at ") && told[0].ends_with("/wants/0/filters"),
        "{told:?}"
    );
    let entry = &trade(document.value(), 0, 0, 0).wants[0];
    assert!(matches!(entry, ItemList::Choice { .. }), "{entry:?}");
    let items = entry.items();
    assert_eq!(items.len(), 5);
    for (item, (id, variants)) in items.iter().zip(EXPANSION) {
        assert_eq!(item.item.as_str(), id);
        assert_eq!(item.price_multiplier, Some(0.05));
        // The function is removed, the others stay; the source's `filters` is not copied and
        // the biomes of the choice become the filter (the 1.20.30 gate runs second).
        assert_eq!(
            item.legacy_functions,
            Some(json(r#"[{"function": "set_data", "data": 1}]"#))
        );
        assert_eq!(item.legacy_filters, Some(mark_variants(variants)));
    }
}

#[test]
fn t05_the_expanded_choices_keep_their_quantity_deviation() {
    // Vanilla copies the misspelt key `quanitity`, so its choices lose the quantity; we
    // copy `quantity`.
    let document = loads(&table_with(Some("1.18.10"), MATERIAL));
    assert_eq!(wanted(document.value()).len(), 5);
    for item in wanted(document.value()) {
        assert_eq!(item.quantity, Some(Quantity::Exact(2)));
    }
    // The typo key is not read: it is an unknown member, and gives no quantity.
    let document = loads(&table_with(
        Some("1.18.10"),
        r#"{"item": "minecraft:boat", "quanitity": 3, "functions": [{"function": "trader_material_type"}]}"#,
    ));
    for item in wanted(document.value()) {
        assert_eq!(item.quantity, None);
    }
    found_at(
        document.diagnostics(),
        "UnrecognizedField",
        Severity::Info,
        "tiers/0/trades/0/wants/0/quanitity",
    );
}

#[test]
fn f6_6_item_1_an_unstamped_material_entry_expands_and_its_emptied_functions_go() {
    let document = loads(&table_with(
        None,
        r#"{"item": "minecraft:boat", "functions": [{"function": "trader_material_type"}]}"#,
    ));
    let items = wanted(document.value());
    assert_eq!(items.len(), 5);
    for (item, (id, variants)) in items.iter().zip(EXPANSION) {
        assert_eq!(item.item.as_str(), id);
        assert_eq!(item.legacy_functions, None);
        assert_eq!(item.legacy_filters, Some(mark_variants(variants)));
        assert_eq!(item.quantity, None);
    }
}

#[test]
fn the_material_gate_is_below_1_18_20_exactly() {
    // The last version below the gate still expands; the gate version itself does not (F.6.4:
    // `version < 1.18.20`).
    for (version, expanded) in [
        ("1.18.11", true),
        ("1.18.19", true),
        ("1.18.20", false),
        ("1.18.21", false),
    ] {
        let document = loads(&table_with(Some(version), MATERIAL));
        let items = wanted(document.value());
        assert_eq!(items.len(), if expanded { 5 } else { 1 }, "{version}");
        assert_eq!(
            matches!(
                trade(document.value(), 0, 0, 0).wants[0],
                ItemList::Choice { .. }
            ),
            expanded,
            "{version}"
        );
    }
}

#[test]
fn f6_6_item_2_nothing_expands_at_1_18_20_and_biomes_still_convert() {
    let document = loads(&doc(
        Some("1.18.20"),
        &format!(
            r#"[{{"trades": [{{"wants": [{MATERIAL}, {{"item": "minecraft:bed", "biomes": ["desert", "snow"]}}],
                "gives": [{{"item": "minecraft:emerald"}}]}}]}}]"#
        ),
    ));
    let wants = &trade(document.value(), 0, 0, 0).wants;
    assert!(matches!(wants[0], ItemList::Item(_)), "{:?}", wants[0]);
    let kept = &wants[0].items()[0];
    assert_eq!(kept.item.as_str(), "minecraft:boat");
    assert_eq!(
        kept.legacy_functions,
        Some(json(
            r#"[{"function": "trader_material_type"}, {"function": "set_data", "data": 1}]"#
        ))
    );
    assert_eq!(kept.legacy_filters, Some(json(r#"{"test": "is_daytime"}"#)));
    assert_eq!(
        wants[1].items()[0].legacy_filters,
        Some(mark_variants(&[1, 4]))
    );
}

#[test]
fn f6_6_item_5_a_number_format_version_reads_as_0_0_0_and_is_told() {
    let document = loads(
        r#"{"format_version": 1.18, "tiers": [{"trades": [{"wants": [
            {"item": "minecraft:boat", "functions": [{"function": "trader_material_type"}]}],
            "gives": [{"item": "minecraft:emerald", "biomes": "jungle"}]}]}]}"#,
    );
    // Vanilla reads it so without a message; ours says at the member that the version is
    // not read, and nothing else.
    let told: Vec<_> = document
        .diagnostics()
        .iter()
        .filter(|f| !announcement(f))
        .map(|f| (f.code.name(), f.severity, f.location.json_path.to_string()))
        .collect();
    assert_eq!(
        told,
        [("InvalidType", Severity::Info, "format_version".to_owned())],
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(document.version().written, None);
    assert_eq!(document.version().effective, FormatVersion::new(0, 0, 0));
    assert_eq!(document.version().parsed_as, FormatVersion::new(0, 0, 0));
    // Both gates ran.
    let traded = trade(document.value(), 0, 0, 0);
    assert_eq!(traded.wants[0].items().len(), 5);
    assert_eq!(
        traded.gives[0].items()[0].legacy_filters,
        Some(mark_variants(&[2]))
    );
}

#[test]
fn a_null_format_version_is_malformed_not_0_0_0() {
    // `initAsJson` finds the key with a value that gives no version: "Malformed format
    // version", no fallback; the legacy parser never runs.
    let d = super::common::rejects(
        r#"{"format_version": null, "tiers": [{"trades": [{"wants": [{"item": "minecraft:boat"}],
            "gives": [{"item": "minecraft:emerald"}]}]}]}"#,
    );
    let errors: Vec<_> = d
        .iter()
        .filter(|f| f.severity == bedrock_addon_core::diagnostic::Severity::Error)
        .map(|f| (f.code.name(), f.location.json_path.to_string()))
        .collect();
    assert_eq!(
        errors,
        [("MalformedFormatVersion", "format_version".to_owned())]
    );
}

#[test]
fn an_expanding_choice_element_is_flattened_into_its_choice() {
    let document = loads(&table_with(
        Some("1.18.10"),
        r#"{"choice": [
            {"item": "minecraft:bed"},
            {"item": "minecraft:boat", "functions": [{"function": "trader_material_type"}]},
            {"item": "minecraft:door"}]}"#,
    ));
    let ids: Vec<&str> = wanted(document.value())
        .iter()
        .map(|i| i.item.as_str())
        .collect();
    assert_eq!(
        ids,
        [
            "minecraft:bed",
            "minecraft:boat:0",
            "minecraft:boat:1",
            "minecraft:boat:3",
            "minecraft:boat:4",
            "minecraft:boat:5",
            "minecraft:door"
        ]
    );
}

#[test]
fn an_entry_without_the_material_function_does_not_expand() {
    let document = loads(&table_with(
        Some("1.18.10"),
        r#"{"item": "minecraft:boat", "functions": [{"function": "set_data", "data": 1}]}"#,
    ));
    let wants = &trade(document.value(), 0, 0, 0).wants;
    assert!(matches!(wants[0], ItemList::Item(_)));
    assert_eq!(
        wants[0].items()[0].legacy_functions,
        Some(json(r#"[{"function": "set_data", "data": 1}]"#))
    );
}

/// what the two gates and the reader take from an item is told at the member the
/// file has; the table holds what the game ends up with.
#[test]
fn a_member_a_gate_drops_or_replaces_is_told() {
    let told = |version: Option<&str>, entry: &str| -> Vec<String> {
        loads(&table_with(version, entry))
            .diagnostics()
            .iter()
            .filter(|f| !announcement(f))
            .map(|f| {
                let path = f.location.json_path.to_string();
                let member = path.split("/wants/0/").nth(1).unwrap_or(&path).to_owned();
                format!("{} at {member}", f.code.name())
            })
            .collect()
    };
    // `biomes` beside `filters`: the filters are replaced; an element that is no name and a
    // name the gate does not know add nothing.
    assert_eq!(
        told(
            None,
            r#"{"item": "minecraft:emerald", "biomes": ["plains", 7, "mushroom"], "filters": {"test": "is_daytime"}}"#
        ),
        [
            "InvalidType at biomes/1",
            "InvalidValue at biomes/2",
            "DroppedField at filters"
        ]
    );
    assert_eq!(
        told(None, r#"{"item": "minecraft:emerald", "biomes": 7}"#),
        ["InvalidType at biomes"]
    );
    assert_eq!(
        told(
            None,
            r#"{"item": "minecraft:emerald", "biomes": "mushroom"}"#
        ),
        ["InvalidValue at biomes"]
    );
    // From 1.20.30 no gate reads `biomes`.
    assert_eq!(
        told(
            Some("1.20.30"),
            r#"{"item": "minecraft:emerald", "biomes": ["plains"]}"#
        ),
        ["DroppedField at biomes"]
    );
    // An entry whose `choice` is empty goes; where other entries stay, it is told.
    let document = loads(&doc(
        None,
        r#"[{"trades": [{"wants": [{"item": "a:b"}, {"choice": []}], "gives": [{"item": "minecraft:bread"}]}]}]"#,
    ));
    let found: Vec<String> = document
        .diagnostics()
        .iter()
        .map(|f| format!("{} at {}", f.code.name(), f.location.json_path))
        .collect();
    assert_eq!(found, ["InvalidValue at tiers/0/trades/0/wants/1/choice"]);
}
