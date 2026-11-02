//! Both ends of every range and every half of every compound condition of the loot walk
//! (reviewers mutate each rule; a test here fails for each mutation).

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Severity};
use bedrock_addon_core::document::{Content, Document};
use bedrock_addon_core::json::{EncodeError, Unrepresentable};
use bedrock_addon_core::load::{Experiment, LoadContext};
use bedrock_addon_core::shared::RangeForm;
use bedrock_addon_family_loot::{
    BannerPattern, BannerType, ConditionKind, EnchantOption, Entry, EntryKind, FunctionKind,
    ItemColor, ItemData, ItemDataKind, LootCondition, LootFunction, LootTable, Pool,
    RandomValueBounds, SpecificEnchant, StewEffect, ToolEnchantment,
};

use super::common::{cx, cx_at, findings, load, load_in, path, table_path, text};

/// A table whose one item entry has `functions`.
fn with_functions(functions: &str) -> String {
    format!(
        r#"{{"pools": [{{"rolls": 1, "entries": [{{"type": "item", "name": "minecraft:stick", "functions": [{functions}]}}]}}]}}"#
    )
}

/// A table whose one pool has `conditions`.
fn with_conditions(conditions: &str) -> String {
    format!(r#"{{"pools": [{{"rolls": 1, "conditions": [{conditions}], "entries": []}}]}}"#)
}

/// The functions of the first entry.
fn functions(table: &Document<LootTable>) -> Vec<LootFunction> {
    match &table.pools.as_deref().unwrap()[0].entries()[0].kind {
        EntryKind::Item { functions, .. } => functions.clone().unwrap_or_default(),
        other => panic!("not an item: {other:?}"),
    }
}

/// The kinds of the conditions of the first pool.
fn conditions(table: &Document<LootTable>) -> Vec<ConditionKind> {
    table.pools.as_deref().unwrap()[0]
        .conditions
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|c| c.kind)
        .collect()
}

/// `(code name, severity, path)` of every finding.
fn summary(table: &Document<LootTable>) -> Vec<(&'static str, Severity, String)> {
    findings(table)
        .into_iter()
        .map(|d| (d.code.name(), d.severity, path(d)))
        .collect()
}

/// The paths of the findings named `name`.
fn paths_of(table: &Document<LootTable>, name: &str) -> Vec<String> {
    findings(table)
        .into_iter()
        .filter(|d| d.code.name() == name)
        .map(path)
        .collect()
}

fn texts(found: &[&Diagnostic]) -> Vec<String> {
    found.iter().map(|d| text(&d.code)).collect()
}

const F0: &str = "pools/0/entries/0/functions/0";

// ---- the file ------------------------------------------------------------------------------

#[test]
fn a_byte_order_mark_is_skipped() {
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(br#"{"pools": []}"#);
    let table = LootTable::parse(&table_path(), &bytes, &cx()).unwrap();
    assert_eq!(table.pools, Some(Vec::new()));
}

#[test]
fn nesting_past_the_depth_limit_and_invalid_utf8_are_invalid_tables() {
    let deep = format!(r#"{{"pools": {}{}}}"#, "[".repeat(200), "]".repeat(200));
    for bytes in [deep.as_bytes(), &b"{\"pools\": \"\xFF\"}"[..]] {
        let rejected = LootTable::parse(&table_path(), bytes, &cx()).unwrap_err();
        let error = rejected.diagnostics.first_error().unwrap();
        assert!(matches!(error.code, DiagnosticCode::MalformedInput { .. }));
        assert_eq!(
            text(&error.code),
            "Loot table for loot_tables/test.json is invalid."
        );
    }
    // Nesting below the limit loads.
    let shallow = format!(
        r#"{{"x": {}{}, "pools": []}}"#,
        "[".repeat(50),
        "]".repeat(50)
    );
    assert!(LootTable::parse(&table_path(), shallow.as_bytes(), &cx()).is_ok());
}

#[test]
fn format_version_is_silent_at_the_root_only() {
    let table = load(
        r#"{"format_version": 2, "pools": [{"format_version": "1.0.0", "rolls": 1, "entries": []}]}"#,
    );
    assert_eq!(
        summary(&table),
        [(
            "UnrecognizedField",
            Severity::Info,
            "pools/0/format_version".to_owned()
        )]
    );
    assert!(table.extra.get("format_version").is_some());
}

// ---- scalars -------------------------------------------------------------------------------

#[test]
fn an_integer_member_takes_the_whole_i32_range_and_truncates_reals() {
    let weights = |text_: &str| -> (Vec<Option<i32>>, Vec<String>) {
        let table = load(&format!(
            r#"{{"pools": [{{"rolls": 1, "entries": [{text_}]}}]}}"#
        ));
        let weights = table.pools.as_deref().unwrap()[0]
            .entries()
            .iter()
            .map(|e| e.weight)
            .collect();
        (weights, paths_of(&table, "InvalidType"))
    };
    let (read, wrong) = weights(
        r#"{"type": "empty", "weight": 2147483647}, {"type": "empty", "weight": -2147483648},
           {"type": "empty", "weight": 2.9}, {"type": "empty", "weight": -2.9},
           {"type": "empty", "weight": 2147483647.5}, {"type": "empty", "weight": -2147483648.5},
           {"type": "empty", "weight": 0}"#,
    );
    assert_eq!(
        read,
        [
            Some(i32::MAX),
            Some(i32::MIN),
            Some(2),
            Some(-2),
            Some(i32::MAX),
            Some(i32::MIN),
            Some(0)
        ]
    );
    assert!(wrong.is_empty(), "{wrong:?}");
    // Ours: each real that lost a fraction is told at the member; an integer, and a
    // real that is a whole number, are not.
    let table = load(
        r#"{"pools": [{"rolls": 1, "entries": [{"type": "empty", "weight": 2.9},
           {"type": "empty", "weight": 3.0}, {"type": "empty", "weight": 3},
           {"type": "empty", "weight": -2147483648.5}]}]}"#,
    );
    assert_eq!(
        paths_of(&table, "InvalidValue"),
        ["pools/0/entries/0/weight", "pools/0/entries/3/weight"]
    );

    let (read, wrong) = weights(
        r#"{"type": "empty", "weight": 2147483648}, {"type": "empty", "weight": -2147483649},
           {"type": "empty", "weight": 3000000000.0}, {"type": "empty", "weight": -3000000000.0},
           {"type": "empty", "weight": true}, {"type": "empty", "weight": null}"#,
    );
    assert_eq!(read, [None; 6]);
    assert_eq!(
        wrong,
        (0..6)
            .map(|i| format!("pools/0/entries/{i}/weight"))
            .collect::<Vec<_>>()
    );
}

#[test]
fn a_float_member_takes_any_finite_number() {
    let table = load(&with_conditions(
        r#"{"condition": "random_chance", "chance": 1},
           {"condition": "random_chance", "chance": -0.5},
           {"condition": "random_chance", "chance": 1e39},
           {"condition": "random_chance", "chance": "0.5"},
           {"condition": "random_chance", "chance": false}"#,
    ));
    assert_eq!(
        conditions(&table),
        [Some(1.0), Some(-0.5), None, None, None]
            .map(|chance| ConditionKind::RandomChance { chance })
    );
    assert_eq!(
        paths_of(&table, "InvalidType"),
        [
            "pools/0/conditions/2/chance",
            "pools/0/conditions/3/chance",
            "pools/0/conditions/4/chance"
        ]
    );
}

#[test]
fn bool_and_string_members_take_only_their_own_type() {
    let table = load(&with_conditions(
        r#"{"condition": "bool_property", "domain": "d", "value": false},
           {"condition": "bool_property", "domain": 1, "value": "true"},
           {"condition": "enum_property", "domain": "", "value": true}"#,
    ));
    assert_eq!(
        conditions(&table),
        [
            ConditionKind::BoolProperty {
                domain: Some("d".to_owned()),
                value: Some(false)
            },
            ConditionKind::BoolProperty {
                domain: None,
                value: None
            },
            ConditionKind::EnumProperty {
                domain: Some(String::new()),
                value: None
            },
        ]
    );
    assert_eq!(
        paths_of(&table, "InvalidType"),
        [
            "pools/0/conditions/1/domain",
            "pools/0/conditions/1/value",
            "pools/0/conditions/2/value"
        ]
    );
}

#[test]
fn bounds_take_a_number_or_an_object_and_nothing_else() {
    let table = load(
        r#"{"pools": [{"rolls": [1, 2], "bonus_rolls": {}, "entries": []},
                      {"rolls": {"max": 4}, "bonus_rolls": true, "entries": []}]}"#,
    );
    let pools = table.pools.as_deref().unwrap();
    assert_eq!(pools[0].rolls, None);
    assert_eq!(
        pools[0].bonus_rolls,
        Some(RandomValueBounds::Range {
            min: None,
            max: None
        })
    );
    assert_eq!(
        pools[1].rolls,
        Some(RandomValueBounds::Range {
            min: None,
            max: Some(4.0)
        })
    );
    assert_eq!((pools[1].rolls().min(), pools[1].rolls().max()), (0.0, 4.0));
    assert_eq!(pools[1].bonus_rolls, None);
    assert_eq!(
        paths_of(&table, "InvalidType"),
        ["pools/0/rolls", "pools/1/bonus_rolls"]
    );
}

#[test]
fn string_lists_take_a_string_or_an_array_and_skip_other_elements() {
    let table = load(&with_functions(
        r#"{"function": "set_lore", "lore": "one line"},
           {"function": "set_lore", "lore": ["a", 5, "b", null]},
           {"function": "set_lore", "lore": {"a": "b"}},
           {"function": "set_lore", "lore": []}"#,
    ));
    let lore: Vec<Option<Vec<String>>> = functions(&table)
        .into_iter()
        .map(|f| match f.kind {
            FunctionKind::SetLore { lore } => lore,
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        lore,
        [
            Some(vec!["one line".to_owned()]),
            Some(vec!["a".to_owned(), "b".to_owned()]),
            None,
            Some(Vec::new())
        ]
    );
    assert_eq!(
        paths_of(&table, "InvalidType"),
        [
            "pools/0/entries/0/functions/1/lore/1",
            "pools/0/entries/0/functions/1/lore/3",
            "pools/0/entries/0/functions/2/lore"
        ]
    );
}

#[test]
fn int_ranges_take_the_json_util_forms() {
    let table = load(&with_conditions(
        r#"{"condition": "match_tool", "count": 3, "durability": {"range_max": 9}},
           {"condition": "match_tool", "count": "3", "durability": null},
           {"condition": "match_tool", "count": [1], "durability": [1, 2, 3]},
           {"condition": "match_tool", "count": 1.0}"#,
    ));
    let ranges: Vec<_> = conditions(&table)
        .into_iter()
        .map(|kind| match kind {
            ConditionKind::MatchTool {
                count, durability, ..
            } => (
                count.map(|c| (c.resolve(i32::MIN, i32::MAX), c.form)),
                durability.map(|d| (d.resolve(i32::MIN, i32::MAX), d.form)),
            ),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        ranges,
        [
            (
                Some(((3, 3), RangeForm::Scalar)),
                Some(((i32::MIN, 9), RangeForm::RangeMinMax))
            ),
            (None, None),
            (None, Some(((1, 2), RangeForm::Array))),
            (None, None),
        ]
    );
    assert_eq!(
        paths_of(&table, "InvalidType"),
        [
            "pools/0/conditions/1/count",
            "pools/0/conditions/1/durability",
            "pools/0/conditions/2/count",
            "pools/0/conditions/3/count"
        ]
    );
}

#[test]
fn nested_objects_of_the_wrong_type_are_warned() {
    // tiers, stew effects and match_tool enchantments.
    let table = load(
        r#"{"pools": [{"rolls": 1, "tiers": [1, 2],
            "conditions": [{"condition": "match_tool", "enchantments": {"a": 1}},
                           {"condition": "match_tool", "enchantments": [5, {"enchantment": "x", "levels": 1, "note": true}]}],
            "entries": [{"type": "item", "name": "minecraft:stew", "functions": [
                {"function": "set_stew_effect", "effects": {"id": 4}},
                {"function": "set_stew_effect", "effects": [7, {"id": 1.9}, {}]},
                {"function": "set_stew_effect", "effects": "x"}]}]}]}"#,
    );
    let pool = &table.pools.as_deref().unwrap()[0];
    assert_eq!(pool.tiers, None);
    let kinds = conditions(&table);
    let ConditionKind::MatchTool { enchantments, .. } = &kinds[0] else {
        panic!()
    };
    assert_eq!(*enchantments, None);
    let ConditionKind::MatchTool { enchantments, .. } = &kinds[1] else {
        panic!()
    };
    let enchantments = enchantments.as_deref().unwrap();
    assert_eq!(enchantments.len(), 1);
    assert_eq!(enchantments[0].enchantment.as_deref(), Some("x"));
    assert!(enchantments[0].extra.get("note").is_some());
    let effects: Vec<Option<Vec<Option<i32>>>> = functions(&table)
        .into_iter()
        .map(|f| match f.kind {
            FunctionKind::SetStewEffect { effects } => {
                effects.map(|e| e.into_iter().map(|e| e.id).collect())
            }
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        effects,
        [
            Some(vec![Some(4)]),
            Some(vec![Some(1), None]),
            Some(Vec::new())
        ]
    );
    assert_eq!(
        paths_of(&table, "InvalidType"),
        [
            "pools/0/conditions/0/enchantments",
            "pools/0/conditions/1/enchantments/0",
            "pools/0/tiers",
            "pools/0/entries/0/functions/1/effects/0",
            "pools/0/entries/0/functions/2/effects",
        ]
    );
    assert_eq!(
        paths_of(&table, "UnrecognizedField"),
        ["pools/0/conditions/1/enchantments/1/note"]
    );
}

// ---- entries -------------------------------------------------------------------------------

#[test]
fn an_entry_type_is_matched_exactly_and_printed_as_written() {
    let table = load(
        r#"{"pools": [{"rolls": 1, "entries": [
            {"type": "Item"}, {"type": "items"}, {"type": "loot_tables"}, {"type": " empty"},
            {}, {"type": 5}, {"type": null}, {"type": ["item"]}]}]}"#,
    );
    assert!(table.pools.as_deref().unwrap()[0].entries().is_empty());
    let found = findings(&table);
    assert!(found.iter().all(
        |d| matches!(d.code, DiagnosticCode::InvalidValue { text: Some(_) })
            && d.severity == Severity::Error
    ));
    let given: Vec<String> = texts(&found)
        .into_iter()
        .map(|t| t.rsplit_once("Given: ").unwrap().1.to_owned())
        .collect();
    assert_eq!(
        given,
        [
            "Item",
            "items",
            "loot_tables",
            " empty",
            "",
            "5",
            "",
            "[\"item\"]"
        ]
    );
}

#[test]
fn an_entrys_conditions_are_read_before_its_type_is_checked() {
    // `LootPoolEntry::deserialize` reads `conditions` first, so a dropped entry still reports
    // them.
    let table =
        load(r#"{"pools": [{"rolls": 1, "entries": [{"type": "nothing", "conditions": [1]}]}]}"#);
    assert_eq!(
        summary(&table),
        [
            (
                "InvalidType",
                Severity::Error,
                "pools/0/entries/0/conditions/0".to_owned()
            ),
            (
                "InvalidValue",
                Severity::Error,
                "pools/0/entries/0".to_owned()
            ),
        ]
    );
}

#[test]
fn every_entry_type_takes_an_inline_sub_table_and_only_items_take_functions() {
    let table = load(
        r#"{"pools": [{"rolls": 1, "entries": [
            {"type": "empty", "pools": [{"rolls": 1, "entries": []}]},
            {"type": "loot_table", "name": "a.json", "pools": [], "functions": []},
            {"type": "item", "pools": 3}]}]}"#,
    );
    let entries = table.pools.as_deref().unwrap()[0].entries();
    assert_eq!(entries[0].pools.as_deref().map(<[Pool]>::len), Some(1));
    assert_eq!(entries[1].pools.as_deref().map(<[Pool]>::len), Some(0));
    assert_eq!(entries[2].pools, None);
    assert_eq!(entries[0].table(), None);
    assert_eq!(entries[1].table().unwrap().as_str(), "a.json");
    assert_eq!(entries[2].table(), None);
    assert_eq!(
        summary(&table),
        [
            (
                "UnrecognizedField",
                Severity::Info,
                "pools/0/entries/1/functions".to_owned()
            ),
            (
                "InvalidType",
                Severity::Info,
                "pools/0/entries/2/pools".to_owned()
            ),
        ]
    );
}

#[test]
fn a_table_reference_keeps_the_text_after_its_last_colon_only() {
    let entry = |name: &str| {
        Entry::new(EntryKind::LootTable {
            name: Some(bedrock_addon_core::id::LootTablePath::parse(
                name.to_owned(),
            )),
        })
    };
    assert_eq!(entry("a:b:c/d.json").table().unwrap().as_str(), "c/d.json");
    assert_eq!(entry("c/d.json").table().unwrap().as_str(), "c/d.json");
    assert_eq!(entry("ns:").table().unwrap().as_str(), "");
    assert_eq!(
        Entry::new(EntryKind::LootTable { name: None }).table(),
        None
    );
}

// ---- functions -----------------------------------------------------------------------------

#[test]
fn a_function_name_is_split_at_its_last_colon_and_printed_as_written() {
    let table = load(&with_functions(
        r#"{"function": "a:b:furnace_smelt"}, {"function": ":furnace_smelt"},
           {"function": "furnace_smelt:"}, {"function": "foo:set_counts"},
           {}, {"function": 7}, {"function": "Furnace_Smelt"}"#,
    ));
    let kept = functions(&table);
    assert_eq!(kept.len(), 2);
    assert_eq!(kept[0].namespace.as_deref(), Some("a:b:"));
    assert_eq!(kept[1].namespace.as_deref(), Some(":"));
    assert_eq!(kept[1].function_name(), ":furnace_smelt");
    let found = findings(&table);
    assert!(found.iter().all(|d| matches!(
        d.code,
        DiagnosticCode::UnrecognizedComponent { text: Some(_) }
    ) && d.severity == Severity::Error));
    assert_eq!(
        texts(&found),
        [
            "Unsupported looting function: furnace_smelt:",
            "Unsupported looting function: foo:set_counts",
            "Unsupported looting function: ",
            "Unsupported looting function: 7",
            "Unsupported looting function: Furnace_Smelt",
        ]
    );
}

#[test]
fn a_functions_conditions_are_read_before_its_name_is_looked_up() {
    let table = load(&with_functions(
        r#"{"function": "nope", "conditions": [{"condition": "nope"}]}"#,
    ));
    assert_eq!(
        summary(&table),
        [
            (
                "UnrecognizedComponent",
                Severity::Info,
                format!("{F0}/conditions/0")
            ),
            ("UnrecognizedComponent", Severity::Error, F0.to_owned()),
        ]
    );
}

/// The findings of a `carry_over_block_entity_data` function named `name` in `cx`, and whether
/// it was kept.
fn carry_over(name: &str, cx: &LoadContext) -> (bool, Vec<String>) {
    let table = load_in(&with_functions(&format!(r#"{{"function": "{name}"}}"#)), cx).unwrap();
    let kept = !functions(&table).is_empty();
    (kept, texts(&findings(&table)))
}

#[test]
fn carry_over_is_gated_at_exactly_engine_1_26_40() {
    const NAME: &str = "carry_over_block_entity_data";
    const UCF: &str = "Unsupported looting function outside of Upcoming Creator Features Experiment: carry_over_block_entity_data";
    const UNKNOWN: &str = "Unsupported looting function: carry_over_block_entity_data";
    // At the engine that has it: the experiment decides.
    assert_eq!(carry_over(NAME, &cx_at(1, 26, 40, true)), (true, vec![]));
    assert_eq!(
        carry_over(NAME, &cx_at(1, 26, 40, false)),
        (false, vec![UCF.to_owned()])
    );
    // One patch below: unknown, with or without the experiment.
    assert_eq!(
        carry_over(NAME, &cx_at(1, 26, 39, true)),
        (false, vec![UNKNOWN.to_owned()])
    );
    assert_eq!(
        carry_over(NAME, &cx_at(1, 26, 39, false)),
        (false, vec![UNKNOWN.to_owned()])
    );
}

#[test]
fn carry_over_takes_either_spelling_of_the_experiment_and_any_namespace() {
    // The experiment set alone turns it on, as the toggle alone does.
    let mut by_experiment = cx_at(1, 26, 50, false);
    by_experiment
        .experiments
        .insert(Experiment::UpcomingCreatorFeatures);
    assert_eq!(
        carry_over("carry_over_block_entity_data", &by_experiment),
        (true, vec![])
    );
    assert_eq!(
        carry_over("carry_over_block_entity_data", &cx_at(1, 26, 50, true)),
        (true, vec![])
    );
    // The gate looks at the name after the last colon and prints the name as written.
    assert_eq!(
        carry_over("minecraft:carry_over_block_entity_data", &cx_at(1, 26, 50, false)),
        (
            false,
            vec!["Unsupported looting function outside of Upcoming Creator Features Experiment: minecraft:carry_over_block_entity_data".to_owned()]
        )
    );
    // Another function is not gated by the experiment.
    let table = load_in(
        &with_functions(r#"{"function": "furnace_smelt"}"#),
        &cx_at(1, 26, 50, false),
    )
    .unwrap();
    assert_eq!(functions(&table).len(), 1);
}

#[test]
fn carry_over_reads_a_bool_of_either_value() {
    let read = |value: &str| {
        let table = load_in(
            &with_functions(&format!(
                r#"{{"function": "carry_over_block_entity_data", "dynamic_properties": {value}}}"#
            )),
            &cx_at(1, 26, 50, true),
        )
        .unwrap();
        (functions(&table)[0].kind.clone(), findings(&table).len())
    };
    for (value, expected) in [("true", Some(true)), ("false", Some(false))] {
        assert_eq!(
            read(value),
            (
                FunctionKind::CarryOverBlockEntityData {
                    dynamic_properties: expected
                },
                0
            )
        );
    }
    for value in ["1", "null", "\"true\""] {
        assert_eq!(
            read(value),
            (
                FunctionKind::CarryOverBlockEntityData {
                    dynamic_properties: None
                },
                1
            )
        );
    }
}

#[test]
fn set_data_warns_only_when_neither_key_is_written() {
    let table = load(&with_functions(
        r#"{"function": "set_data", "data": 1, "values": {"min": 5, "max": 6}},
           {"function": "set_data", "values": 2},
           {"function": "set_data", "data": "x"},
           {"function": "random_aux_value"}"#,
    ));
    let data: Vec<ItemData> = functions(&table)
        .into_iter()
        .map(|f| match f.kind {
            FunctionKind::SetData(data) | FunctionKind::RandomAuxValue(data) => data,
            other => panic!("{other:?}"),
        })
        .collect();
    // `data` wins over `values`.
    assert_eq!(data[0].kind(), ItemDataKind::SetItemData);
    assert_eq!(data[0].range(), RandomValueBounds::Exact(1.0));
    assert_eq!(data[1].kind(), ItemDataKind::RandomAuxValue);
    assert_eq!(data[1].range(), RandomValueBounds::Exact(2.0));
    // A written key of the wrong type is our wrong-type warning, not vanilla's missing-key one.
    assert_eq!(
        summary(&table),
        [
            (
                "InvalidType",
                Severity::Info,
                "pools/0/entries/0/functions/2/data".to_owned()
            ),
            (
                "MissingRequiredField",
                Severity::Warning,
                "pools/0/entries/0/functions/3".to_owned()
            ),
        ]
    );
}

// ---- banners -------------------------------------------------------------------------------

/// The `type`, `base_color` and `patterns` of a `set_banner_details` function.
type Banner = (
    Option<BannerType>,
    Option<ItemColor>,
    Option<Vec<BannerPattern>>,
);

/// The parameters of the `set_banner_details` functions of `table`.
fn banners(table: &Document<LootTable>) -> Vec<Banner> {
    functions(table)
        .into_iter()
        .map(|f| match f.kind {
            FunctionKind::SetBannerDetails {
                banner_type,
                base_color,
                patterns,
            } => (banner_type, base_color, patterns),
            other => panic!("{other:?}"),
        })
        .collect()
}

#[test]
fn a_banner_type_is_a_name_or_0_or_1() {
    let table = load(&with_functions(
        r#"{"function": "set_banner_details", "type": "default"},
           {"function": "set_banner_details", "type": "illager_captain"},
           {"function": "set_banner_details", "type": 0},
           {"function": "set_banner_details", "type": 1},
           {"function": "set_banner_details", "type": 2},
           {"function": "set_banner_details", "type": -1},
           {"function": "set_banner_details", "type": 1.0},
           {"function": "set_banner_details", "type": "captain"},
           {"function": "set_banner_details", "type": {"a": [true]}}"#,
    ));
    let types: Vec<Option<BannerType>> = banners(&table).into_iter().map(|b| b.0).collect();
    assert_eq!(
        types,
        [
            Some(BannerType::Default),
            Some(BannerType::IllagerCaptain),
            Some(BannerType::Default),
            Some(BannerType::IllagerCaptain),
            None,
            None,
            None,
            None,
            None
        ]
    );
    let found = findings(&table);
    assert!(found.iter().all(
        |d| matches!(d.code, DiagnosticCode::InvalidValue { text: Some(_) })
            && d.severity == Severity::Error
    ));
    assert_eq!(
        texts(&found),
        [
            "Invalid banner type input: 2\n",
            "Invalid banner type input: -1\n",
            "Invalid banner type input: 1.0\n",
            "Invalid banner type input: \"captain\"\n",
            "Invalid banner type input: {\"a\":[true]}\n",
        ]
    );
    assert_eq!(path(found[0]), "pools/0/entries/0/functions/4/type");
}

#[test]
fn a_colour_is_one_of_16_names_or_0_to_15() {
    let names = [
        "black",
        "red",
        "green",
        "brown",
        "blue",
        "purple",
        "cyan",
        "silver",
        "gray",
        "pink",
        "lime",
        "yellow",
        "light_blue",
        "magenta",
        "orange",
        "white",
    ];
    for (index, name) in names.iter().enumerate() {
        let table = load(&with_functions(&format!(
            r#"{{"function": "set_banner_details", "base_color": "{name}"}},
               {{"function": "set_banner_details", "base_color": {index}}}"#
        )));
        assert!(findings(&table).is_empty(), "{name}");
        let read = banners(&table);
        assert_eq!(read[0].1, read[1].1, "{name}");
        let color = read[0].1.unwrap();
        assert_eq!(color, ItemColor::ALL[index]);
        assert_eq!((color.name(), usize::from(color.index())), (*name, index));
        assert_eq!(ItemColor::from_name(name), Some(color));
    }
    assert_eq!(ItemColor::from_name("plaid"), None);
    assert_eq!(ItemColor::default(), ItemColor::White);

    let table = load(&with_functions(
        r#"{"function": "set_banner_details", "base_color": 16},
           {"function": "set_banner_details", "base_color": -1},
           {"function": "set_banner_details", "base_color": 3.0},
           {"function": "set_banner_details", "base_color": "White"}"#,
    ));
    assert!(banners(&table).iter().all(|b| b.1.is_none()));
    assert_eq!(
        texts(&findings(&table)),
        [
            "Invalid banner base_color input: 16\n",
            "Invalid banner base_color input: -1\n",
            "Invalid banner base_color input: 3.0\n",
            "Invalid banner base_color input: \"White\"\n",
        ]
    );
}

#[test]
fn six_patterns_fit_and_a_seventh_does_not() {
    let patterns = |count: usize| {
        let list: Vec<String> = (0..count)
            .map(|i| format!(r#"{{"pattern": "p{i}"}}"#))
            .collect();
        let table = load(&with_functions(&format!(
            r#"{{"function": "set_banner_details", "patterns": [{}]}}"#,
            list.join(", ")
        )));
        (
            banners(&table)[0].2.as_ref().map(Vec::len),
            texts(&findings(&table)),
        )
    };
    assert_eq!(patterns(0), (Some(0), vec![]));
    assert_eq!(patterns(6), (Some(6), vec![]));
    assert_eq!(
        patterns(7),
        (
            Some(6),
            vec!["Banner exceeds maximum number of 6 patterns! Patterns given: 7".to_owned()]
        )
    );
    assert_eq!(
        patterns(9),
        (
            Some(6),
            vec!["Banner exceeds maximum number of 6 patterns! Patterns given: 9".to_owned()]
        )
    );
}

#[test]
fn patterns_need_a_default_banner() {
    let table = load(&with_functions(
        r#"{"function": "set_banner_details", "type": 0, "patterns": {"pattern": "bo", "color": 3}},
           {"function": "set_banner_details", "type": 1, "patterns": []},
           {"function": "set_banner_details", "type": "nope", "patterns": [{"pattern": "bo"}]},
           {"function": "set_banner_details", "type": 1}"#,
    ));
    let read = banners(&table);
    // A single object is one pattern; the type given as 0 is the default.
    assert_eq!(
        read[0].2,
        Some(vec![BannerPattern {
            pattern: "bo".to_owned(),
            color: Some(ItemColor::Brown),
            extra: Default::default()
        }])
    );
    // A captain's banner takes no patterns, not even none.
    assert_eq!(read[1].2, None);
    // A type that is not one is the default type, which takes patterns.
    assert_eq!(read[2].2.as_ref().map(Vec::len), Some(1));
    assert_eq!(read[3], (Some(BannerType::IllagerCaptain), None, None));
    assert_eq!(
        summary(&table),
        [
            (
                "InvalidValue",
                Severity::Error,
                "pools/0/entries/0/functions/1/patterns".to_owned()
            ),
            (
                "InvalidValue",
                Severity::Error,
                "pools/0/entries/0/functions/2/type".to_owned()
            ),
        ]
    );
}

#[test]
fn a_pattern_without_a_name_is_dropped_with_vanillas_text() {
    let table = load(&with_functions(
        r#"{"function": "set_banner_details", "patterns": [
            {"pattern": ""}, {}, 5, {"pattern": null}, {"pattern": 5}, {"pattern": "bo", "tint": 1}]}"#,
    ));
    let read = banners(&table);
    let patterns = read[0].2.as_ref().unwrap();
    assert_eq!(patterns.len(), 1);
    assert!(patterns[0].extra.get("tint").is_some());
    let found = findings(&table);
    let unresolved: Vec<_> = found
        .iter()
        .filter(|d| {
            matches!(
                d.code,
                DiagnosticCode::UnresolvedReference { text: Some(_) }
            )
        })
        .collect();
    assert_eq!(unresolved.len(), 4);
    assert!(
        unresolved
            .iter()
            .all(|d| d.severity == Severity::Error
                && text(&d.code) == "Invalid banner pattern name: ")
    );
    assert_eq!(
        summary(&table)
            .into_iter()
            .map(|(name, _, at)| (name, at))
            .collect::<Vec<_>>(),
        [
            ("UnresolvedReference", format!("{F0}/patterns/0")),
            ("UnresolvedReference", format!("{F0}/patterns/1")),
            ("UnresolvedReference", format!("{F0}/patterns/2")),
            ("UnresolvedReference", format!("{F0}/patterns/3")),
            ("InvalidType", format!("{F0}/patterns/4/pattern")),
            ("UnrecognizedField", format!("{F0}/patterns/5/tint")),
        ]
    );
}

/// The game reads a banner `pattern` through `asString` with an
/// empty default, so an absent member, `null` and `""` are one case;
/// the pattern lookup returns 0 for it, and the branch logs
/// and returns, appending nothing. Each shape
/// is one kept error and no pattern.
#[test]
fn a_nameless_banner_pattern_is_dropped_with_one_kept_error() {
    for entry in [
        r#"{"color": "red"}"#,
        r#"{"pattern": null}"#,
        r#"{"pattern": ""}"#,
    ] {
        let table = load(&with_functions(&format!(
            r#"{{"function": "set_banner_details", "patterns": [{entry}]}}"#
        )));
        let read = banners(&table);
        let count = read[0].2.as_ref().map_or(0, Vec::len);
        assert_eq!(
            count, 0,
            "{entry}: the game appends no pattern: {:?}",
            read[0].2
        );
        let found = findings(&table);
        assert_eq!(found.len(), 1, "{entry}: {found:?}");
        let error = &found[0];
        assert!(
            error.severity == Severity::Error
                && error.kept
                && text(&error.code) == "Invalid banner pattern name: ",
            "{entry}: {error:?}"
        );
    }
}

// ---- enchantments --------------------------------------------------------------------------

#[test]
fn an_enchantment_is_named_without_a_namespace() {
    let table = load(&with_functions(
        r#"{"function": "specific_enchants", "enchants": ["sharpness", "infinity", "bow_infinity", "minecraft:sharpness", "Sharpness", ""]}"#,
    ));
    let FunctionKind::SpecificEnchants { enchants } = &functions(&table)[0].kind else {
        panic!()
    };
    let ids: Vec<&str> = enchants
        .as_deref()
        .unwrap()
        .iter()
        .map(|e| e.id.as_str())
        .collect();
    assert_eq!(ids, ["sharpness", "infinity", "bow_infinity"]);
    assert_eq!(
        paths_of(&table, "UnresolvedReference"),
        [
            format!("{F0}/enchants/3"),
            format!("{F0}/enchants/4"),
            format!("{F0}/enchants/5")
        ]
    );
}

#[test]
fn specific_enchants_take_a_single_value_and_objects_with_or_without_a_level() {
    let table = load(&with_functions(
        r#"{"function": "specific_enchants", "enchants": "mending"},
           {"function": "specific_enchants", "enchants": {"id": "mending", "level": 2, "note": "x"}},
           {"function": "specific_enchants", "enchants": [{"id": "mending"}, {"level": 1}, {"id": 4}, 4, {"id": "mending", "level": "high"}]}"#,
    ));
    let enchants: Vec<Vec<SpecificEnchant>> = functions(&table)
        .into_iter()
        .map(|f| match f.kind {
            FunctionKind::SpecificEnchants { enchants } => enchants.unwrap(),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(enchants[0], [SpecificEnchant::new("mending")]);
    assert_eq!(enchants[1].len(), 1);
    assert_eq!(enchants[1][0].levels(), (2, 2));
    assert!(enchants[1][0].extra.get("note").is_some());
    assert_eq!(
        enchants[2],
        [
            SpecificEnchant::new("mending"),
            SpecificEnchant::new("mending")
        ]
    );
    assert_eq!(
        summary(&table)
            .into_iter()
            .map(|(name, _, at)| (name, at))
            .collect::<Vec<_>>(),
        [
            (
                "UnrecognizedField",
                "pools/0/entries/0/functions/1/enchants/note".to_owned()
            ),
            (
                "UnresolvedReference",
                "pools/0/entries/0/functions/2/enchants/1".to_owned()
            ),
            (
                "InvalidType",
                "pools/0/entries/0/functions/2/enchants/2/id".to_owned()
            ),
            (
                "InvalidType",
                "pools/0/entries/0/functions/2/enchants/3".to_owned()
            ),
            (
                "InvalidType",
                "pools/0/entries/0/functions/2/enchants/4/level".to_owned()
            ),
        ]
    );
    // The object form is written when it holds a level or another member, else the id alone.
    let written = table.to_json().unwrap();
    assert!(
        written.contains(
            r#""enchants": [
                "mending"
              ]"#
        ),
        "{written}"
    );
    assert!(written.contains(r#""note": "x""#), "{written}");
    assert!(written.contains(r#""level": 2"#), "{written}");
    let reread = load(&written);
    assert_eq!(reread.value(), table.value());
    let leveled = SpecificEnchant {
        level: Some(bedrock_addon_core::shared::IntRange::new(
            1,
            3,
            RangeForm::Array,
        )),
        ..SpecificEnchant::new("unbreaking")
    };
    assert_eq!(leveled.levels(), (1, 3));
}

#[test]
fn enchant_book_options_read_whole_levels_and_keep_other_members() {
    let table = load(&with_functions(
        r#"{"function": "enchant_book_for_trading", "enchantments": [
            {"name": "infinity", "min": 2, "max": 3.0, "weight": 1},
            {"name": "bow_infinity", "min": "2", "max": 2.5},
            {"name": "minecraft:sharpness"}, {"min": 1}, 9]}"#,
    ));
    let FunctionKind::EnchantBookForTrading { enchantments, .. } = &functions(&table)[0].kind
    else {
        panic!()
    };
    let options = enchantments.as_deref().unwrap();
    assert_eq!(options.len(), 2);
    assert_eq!((options[0].min, options[0].max), (Some(2), Some(3)));
    assert!(options[0].extra.get("weight").is_some());
    assert_eq!(
        options[1],
        EnchantOption {
            name: "bow_infinity".to_owned(),
            min: None,
            max: None,
            extra: Default::default()
        }
    );
    assert_eq!(
        summary(&table)
            .into_iter()
            .map(|(name, _, at)| (name, at))
            .collect::<Vec<_>>(),
        [
            ("UnrecognizedField", format!("{F0}/enchantments/0/weight")),
            ("InvalidType", format!("{F0}/enchantments/1/min")),
            ("InvalidType", format!("{F0}/enchantments/1/max")),
            ("UnresolvedReference", format!("{F0}/enchantments/2")),
            ("UnresolvedReference", format!("{F0}/enchantments/3")),
            ("UnresolvedReference", format!("{F0}/enchantments/4")),
        ]
    );
    let names: Vec<String> = findings(&table)
        .into_iter()
        .filter(|d| d.code.name() == "UnresolvedReference")
        .map(|d| text(&d.code))
        .collect();
    assert_eq!(
        names,
        [
            "Error loading \"enchant_book_for_trading\" function, \"name\": minecraft:sharpness is not a valid enchantment name",
            "Error loading \"enchant_book_for_trading\" function, \"name\":  is not a valid enchantment name",
            "Error loading \"enchant_book_for_trading\" function, \"name\":  is not a valid enchantment name",
        ]
    );
}

// ---- conditions ----------------------------------------------------------------------------

#[test]
fn a_condition_name_is_matched_exactly() {
    let table = load(&with_conditions(
        r#"{"condition": "is_baby"}, {"condition": "Is_Baby"}, {"condition": "minecraft:is_baby"},
           {}, {"condition": 3}"#,
    ));
    assert_eq!(conditions(&table), [ConditionKind::IsBaby]);
    assert_eq!(
        texts(&findings(&table)),
        [
            "There is unknown or supressed condition[Is_Baby]",
            "There is unknown or supressed condition[minecraft:is_baby]",
            "There is unknown or supressed condition[]",
            "There is unknown or supressed condition[3]",
        ]
    );
}

#[test]
fn biome_has_tag_is_gated_at_exactly_engine_1_26_30() {
    let text_ = with_conditions(r#"{"condition": "biome_has_tag", "tag": "caves"}"#);
    for (patch, known) in [(29, false), (30, true), (31, true)] {
        let table = load_in(&text_, &cx_at(1, 26, patch, false)).unwrap();
        assert_eq!(conditions(&table).len(), usize::from(known), "1.26.{patch}");
        assert_eq!(findings(&table).len(), usize::from(!known), "1.26.{patch}");
    }
    // The other conditions are not gated by the engine.
    let old = load_in(
        &with_conditions(r#"{"condition": "is_baby"}"#),
        &cx_at(1, 26, 29, false),
    )
    .unwrap();
    assert_eq!(conditions(&old), [ConditionKind::IsBaby]);
}

#[test]
fn each_difficulty_chance_defaults_on_its_own() {
    let chances = |kind: ConditionKind| kind.difficulty_chances();
    let base =
        |default_chance, peaceful, easy, normal, hard| ConditionKind::RandomDifficultyChance {
            default_chance,
            peaceful,
            easy,
            normal,
            hard,
        };
    assert_eq!(chances(base(None, None, None, None, None)), Some([0.0; 4]));
    assert_eq!(
        chances(base(Some(0.5), Some(0.1), None, None, None)),
        Some([0.1, 0.5, 0.5, 0.5])
    );
    assert_eq!(
        chances(base(Some(0.5), None, Some(0.2), None, None)),
        Some([0.5, 0.2, 0.5, 0.5])
    );
    assert_eq!(
        chances(base(Some(0.5), None, None, Some(0.3), None)),
        Some([0.5, 0.5, 0.3, 0.5])
    );
    assert_eq!(
        chances(base(None, None, None, None, Some(0.4))),
        Some([0.0, 0.0, 0.0, 0.4])
    );
    assert_eq!(chances(ConditionKind::IsBaby), None);
}

// ---- names and writing ---------------------------------------------------------------------

#[test]
fn every_function_and_condition_is_written_under_its_registry_name() {
    let functions = [
        "set_count",
        "set_data",
        "set_damage",
        "looting_enchant",
        "enchant_with_levels",
        "enchant_book_for_trading",
        "enchant_randomly",
        "furnace_smelt",
        "set_data_from_color_index",
        "enchant_random_gear",
        "random_aux_value",
        "random_block_state",
        "random_dye",
        "exploration_map",
        "set_banner_details",
        "explosion_decay",
        "set_name",
        "set_lore",
        "specific_enchants",
        "fill_container",
        "set_actor_id",
        "set_book_contents",
        "set_stew_effect",
        "set_ominous_bottle_amplifier",
        "set_armor_trim",
        "set_potion",
        "carry_over_block_entity_data",
    ];
    let list: Vec<String> = functions
        .iter()
        .map(|name| format!(r#"{{"function": "{name}", "data": 1}}"#))
        .collect();
    let table = load_in(&with_functions(&list.join(", ")), &cx_at(1, 26, 50, true)).unwrap();
    let read: Vec<&str> = self::functions(&table)
        .iter()
        .map(|f| f.kind.name())
        .collect::<Vec<_>>();
    assert_eq!(read, functions);
    let written = table.to_json().unwrap();
    for name in functions {
        assert!(
            written.contains(&format!(r#""function": "{name}""#)),
            "{name}"
        );
    }

    let conditions = [
        "killed_by_player",
        "killed_by_player_or_pets",
        "killed_by_entity",
        "entity_killed",
        "random_chance",
        "random_difficulty_chance",
        "random_chance_with_looting",
        "random_regional_difficulty_chance",
        "has_mark_variant",
        "has_variant",
        "match_tool",
        "damaged_by_entity",
        "passenger_of_entity",
        "is_baby",
        "has_property",
        "bool_property",
        "int_property",
        "float_property",
        "enum_property",
        "biome_has_tag",
    ];
    let list: Vec<String> = conditions
        .iter()
        .map(|name| format!(r#"{{"condition": "{name}"}}"#))
        .collect();
    let table = load(&with_conditions(&list.join(", ")));
    assert!(findings(&table).is_empty(), "{:?}", findings(&table));
    let read: Vec<&str> = self::conditions(&table)
        .iter()
        .map(ConditionKind::name)
        .collect();
    assert_eq!(read, conditions);
    let written = table.to_json().unwrap();
    for name in conditions {
        assert!(
            written.contains(&format!(r#""condition": "{name}""#)),
            "{name}"
        );
    }
    assert_eq!(load(&written).value(), table.value());
}

#[test]
fn a_number_without_a_json_form_is_not_written() {
    let table = |x: f32| LootTable {
        pools: Some(vec![Pool {
            rolls: Some(RandomValueBounds::Exact(1.0)),
            conditions: Some(vec![LootCondition::new(ConditionKind::RandomChance {
                chance: Some(x),
            })]),
            ..Pool::default()
        }]),
        ..LootTable::default()
    };
    for (x, expected) in [
        (f32::NAN, Unrepresentable::NaN),
        (f32::INFINITY, Unrepresentable::Infinity),
        (f32::NEG_INFINITY, Unrepresentable::NegativeInfinity),
    ] {
        let EncodeError::NotRepresentable { at, what } = table(x).to_json().unwrap_err() else {
            panic!("another error");
        };
        assert_eq!(what, expected);
        assert_eq!(at.to_string(), "pools/0/conditions/0/chance");
    }
    assert!(table(f32::MAX).to_json().is_ok());
    assert!(table(f32::MIN).to_json().is_ok());
}

#[test]
fn every_member_of_every_nested_object_is_written_back() {
    // Each optional member on its own level: written when held, and read back equal.
    let table = LootTable {
        pools: Some(vec![Pool {
            conditions: Some(vec![LootCondition::new(ConditionKind::MatchTool {
                item: None,
                count: None,
                durability: None,
                enchantments: Some(vec![ToolEnchantment {
                    enchantment: Some("silk_touch".to_owned()),
                    levels: None,
                    extra: Default::default(),
                }]),
                filter_any: None,
                filter_all: Some(vec!["minecraft:is_tool".to_owned()]),
                filter_none: None,
            })]),
            entries: Some(vec![Entry::new(EntryKind::Item {
                name: None,
                functions: Some(vec![
                    LootFunction::new(FunctionKind::SetStewEffect {
                        effects: Some(vec![StewEffect {
                            id: Some(3),
                            extra: Default::default(),
                        }]),
                    }),
                    LootFunction::new(FunctionKind::EnchantBookForTrading {
                        base_cost: None,
                        per_level_cost: Some(1),
                        base_random_cost: None,
                        per_level_random_cost: Some(2),
                        enchantments: Some(vec![EnchantOption {
                            name: "mending".to_owned(),
                            min: None,
                            max: Some(1),
                            extra: Default::default(),
                        }]),
                    }),
                ]),
            })]),
            ..Pool::default()
        }]),
        ..LootTable::default()
    };
    let written = table.to_json().unwrap();
    let again = load(&written);
    assert_eq!(*again.value(), table, "{written}");
    for key in [
        "\"min\"",
        "\"base_cost\"",
        "\"levels\"",
        "match_tool_filter_any",
        "\"name\": null",
    ] {
        assert!(!written.contains(key), "{key} written:\n{written}");
    }
}

#[test]
fn a_specific_enchant_with_another_member_and_no_level_is_written_as_an_object() {
    let table = load(&with_functions(
        r#"{"function": "specific_enchants", "enchants": [{"id": "mending", "note": "y"}]}"#,
    ));
    let written = table.to_json().unwrap();
    assert!(written.contains(r#""note": "y""#), "{written}");
    assert!(written.contains(r#""id": "mending""#), "{written}");
    assert!(!written.contains("level"), "{written}");
    assert_eq!(load(&written).value(), table.value());
}

#[test]
fn a_bound_without_a_json_form_is_not_written() {
    for bounds in [
        RandomValueBounds::Exact(f32::NAN),
        RandomValueBounds::Range {
            min: Some(f32::INFINITY),
            max: None,
        },
    ] {
        let table = LootTable {
            pools: Some(vec![Pool {
                rolls: Some(bounds),
                ..Pool::default()
            }]),
            ..LootTable::default()
        };
        let EncodeError::NotRepresentable { at, .. } = table.to_json().unwrap_err() else {
            panic!("another error");
        };
        assert!(at.to_string().starts_with("pools/0/rolls"), "{at}");
    }
}

#[test]
fn weight_and_quality_are_written_for_every_entry_type() {
    let table = load(
        r#"{"pools": [{"rolls": 1, "entries": [
            {"type": "item", "name": "minecraft:stick", "weight": 4, "quality": 5},
            {"type": "loot_table", "name": "a.json", "weight": 6, "quality": 7},
            {"type": "empty", "weight": 8, "quality": 9}]}]}"#,
    );
    let written = table.to_json().unwrap();
    let again = load(&written);
    let read: Vec<(Option<i32>, Option<i32>)> = again.pools.as_deref().unwrap()[0]
        .entries()
        .iter()
        .map(|e| (e.weight, e.quality))
        .collect();
    assert_eq!(
        read,
        [(Some(4), Some(5)), (Some(6), Some(7)), (Some(8), Some(9))]
    );
    assert_eq!(again.value(), table.value());
}

#[test]
fn carry_over_writes_its_parameter_back() {
    let cx = cx_at(1, 26, 50, true);
    let table = load_in(
        &with_functions(
            r#"{"function": "carry_over_block_entity_data", "dynamic_properties": true}"#,
        ),
        &cx,
    )
    .unwrap();
    let written = table.to_json().unwrap();
    assert!(
        written.contains(r#""dynamic_properties": true"#),
        "{written}"
    );
    assert_eq!(load_in(&written, &cx).unwrap().value(), table.value());
}

#[test]
// An authored `null` that vanilla reads as absent is not kept.
fn a_null_enchantments_member_is_vanillas_no_options_and_is_not_written_back() {
    // Vanilla tests `enchantments` for null itself and
    // logs nothing, so `null` reads as absent with no finding.
    let table = load(&with_functions(
        r#"{"function": "enchant_book_for_trading", "base_cost": 4, "enchantments": null}"#,
    ));
    assert!(findings(&table).is_empty(), "{:?}", findings(&table));
    assert_eq!(
        functions(&table)[0].kind,
        FunctionKind::EnchantBookForTrading {
            base_cost: Some(4),
            per_level_cost: None,
            base_random_cost: None,
            per_level_random_cost: None,
            enchantments: None,
        }
    );
    let written = table.to_json().unwrap();
    assert!(!written.contains("enchantments"), "{written}");
    let again = load(&written);
    assert_eq!(again.value(), table.value());
    assert_eq!(again.to_json().unwrap(), written);
}

#[test]
fn a_single_value_form_is_reported_at_the_members_own_path() {
    // `enchants`, `patterns` and `effects` take one value in place of an array: a finding
    // about that value is at the member, not at an index the document does not have.
    let table = load(&with_functions(
        r#"{"function": "specific_enchants", "enchants": 5},
           {"function": "specific_enchants", "enchants": "nope"},
           {"function": "set_banner_details", "patterns": 7},
           {"function": "set_banner_details", "patterns": {"pattern": "bo", "color": "plaid"}},
           {"function": "set_stew_effect", "effects": 3},
           {"function": "set_stew_effect", "effects": {"id": "x"}}"#,
    ));
    let at = |index: usize, rest: &str| format!("pools/0/entries/0/functions/{index}/{rest}");
    assert_eq!(
        summary(&table)
            .into_iter()
            .map(|(name, _, at)| (name, at))
            .collect::<Vec<_>>(),
        [
            ("InvalidType", at(0, "enchants")),
            ("UnresolvedReference", at(1, "enchants")),
            ("UnresolvedReference", at(2, "patterns")),
            ("InvalidValue", at(3, "patterns/color")),
            ("InvalidType", at(4, "effects")),
            ("InvalidType", at(5, "effects/id")),
        ]
    );
    // An array keeps its indices.
    let array = load(&with_functions(
        r#"{"function": "specific_enchants", "enchants": [5]},
           {"function": "set_banner_details", "patterns": [7]},
           {"function": "set_stew_effect", "effects": [3]}"#,
    ));
    assert_eq!(
        summary(&array)
            .into_iter()
            .map(|(_, _, at)| at)
            .collect::<Vec<_>>(),
        [at(0, "enchants/0"), at(1, "patterns/0"), at(2, "effects/0")]
    );
}
