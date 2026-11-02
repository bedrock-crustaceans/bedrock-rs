//! Deviations from vanilla: each pinned by a test whose name says `deviation` with the D- entry or
//! the folder row it implements.

use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::document::{Content, Extra};
use bedrock_addon_family_loot::{
    ConditionKind, EnchantOption, EntryKind, FunctionKind, LootFunction, RandomValueBounds,
};

use super::common::{findings, load, path};

/// The paths of the `InvalidType` warnings without a text (ours) of `table`.
fn wrong_types(
    table: &bedrock_addon_core::document::Document<bedrock_addon_family_loot::LootTable>,
) -> Vec<String> {
    findings(table)
        .into_iter()
        .filter(|d| {
            d.severity == Severity::Info
                && matches!(d.code, DiagnosticCode::InvalidType { text: None })
        })
        .map(path)
        .collect()
}

#[test]
fn a_member_of_the_wrong_json_type_is_warned_and_read_as_absent_deviation() {
    // vanilla's `asInt` / `asFloat` / `asBool` / `asString` convert (or throw); we warn
    // and read the member as absent, so its default applies and it is not written back.
    let table = load(
        r#"{"pools": [{"rolls": "2", "tiers": {"bonus_chance": true},
            "conditions": [{"condition": "random_chance", "chance": null}],
            "entries": [{"type": "item", "name": 5, "weight": "3", "functions": [
                {"function": "enchant_randomly", "treasure": 1},
                {"function": "set_count", "count": {"min": "a", "max": 2}},
                {"function": "enchant_book_for_trading", "enchantments": [{"name": "sharpness", "min": 1.5}]}]}]}]}"#,
    );
    assert_eq!(
        wrong_types(&table),
        [
            "pools/0/rolls",
            "pools/0/conditions/0/chance",
            "pools/0/tiers/bonus_chance",
            "pools/0/entries/0/weight",
            "pools/0/entries/0/name",
            "pools/0/entries/0/functions/0/treasure",
            "pools/0/entries/0/functions/1/count/min",
            "pools/0/entries/0/functions/2/enchantments/0/min",
        ]
    );
    let pool = &table.pools.as_deref().unwrap()[0];
    assert_eq!(pool.rolls, None);
    assert_eq!(pool.tiers.as_ref().unwrap().bonus_chance, None);
    assert_eq!(
        pool.conditions.as_deref().unwrap()[0].kind,
        ConditionKind::RandomChance { chance: None }
    );
    let entry = &pool.entries()[0];
    assert_eq!(entry.weight, None);
    let EntryKind::Item { name, functions } = &entry.kind else {
        panic!("an item entry");
    };
    assert_eq!(*name, None);
    let functions = functions.as_deref().unwrap();
    assert_eq!(
        functions[0].kind,
        FunctionKind::EnchantRandomly { treasure: None }
    );
    assert_eq!(
        functions[1].kind,
        FunctionKind::SetCount {
            count: Some(RandomValueBounds::Range {
                min: None,
                max: Some(2.0)
            })
        }
    );
    let FunctionKind::EnchantBookForTrading { enchantments, .. } = &functions[2].kind else {
        panic!("not enchant_book_for_trading");
    };
    assert_eq!(
        enchantments.as_deref().unwrap(),
        [EnchantOption {
            name: "sharpness".to_owned(),
            min: None,
            max: None,
            extra: Extra::default(),
        }]
    );
    // Nothing of the wrong type is written back.
    let written = table.to_json().unwrap();
    for gone in ["\"2\"", "\"3\"", "\"a\"", "null", "1.5", "true"] {
        assert!(!written.contains(gone), "{gone} written:\n{written}");
    }
}

#[test]
fn unknown_members_of_a_bounds_object_are_reported_and_not_kept_deviation() {
    // `RandomValueBounds` has no `extra` (as `IntRange`); vanilla ignores the
    // member, we report it at info and do not write it back.
    let table = load(r#"{"pools": [{"rolls": {"min": 1, "max": 2, "step": 1}, "entries": []}]}"#);
    let found = findings(&table);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(matches!(
        found[0].code,
        DiagnosticCode::UnrecognizedField { text: None }
    ));
    assert_eq!(found[0].severity, Severity::Info);
    assert_eq!(path(found[0]), "pools/0/rolls/step");
    assert!(!table.to_json().unwrap().contains("step"));
}

#[test]
fn an_unknown_condition_is_an_information_where_vanilla_only_logs_for_developers_deviation() {
    // drops it, with an information carrying the game's developer-log line
    // (the game's developer log).
    let table = load(
        r#"{"pools": [{"rolls": 1, "conditions": [{"condition": "entity_properties", "entity": "this"}], "entries": []}]}"#,
    );
    assert_eq!(
        table.pools.as_deref().unwrap()[0].conditions.as_deref(),
        Some(&[][..])
    );
    let found = findings(&table);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].severity, Severity::Info);
    assert_eq!(
        found[0].code.to_string(),
        "There is unknown or supressed condition[entity_properties]"
    );
}

#[test]
fn a_function_name_is_written_back_with_its_namespace_deviation() {
    // the name vanilla strips is kept as written for write-back; a function built in
    // code has none.
    let function = LootFunction::new(FunctionKind::ExplosionDecay);
    assert_eq!(function.function_name(), "explosion_decay");
}
