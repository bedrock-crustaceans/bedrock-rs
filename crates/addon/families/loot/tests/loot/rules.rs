//! The validation rules: one test per rule, matched on the code, the severity and
//! the location; vanilla's texts are compared where the rule quotes one.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::document::Content;
use bedrock_addon_family_loot::{ConditionKind, EntryKind, LootTable, Pool};

use super::common::{cx, findings, load, path, table_path, text};

#[test]
fn v01_a_file_that_is_not_a_json_object_is_not_loaded() {
    for bytes in [
        &b"{ not json"[..],
        b"\"pools\"",
        b"",
        b"null",
        b"\xEF\xBB\xBF",
        b" // nothing\n",
    ] {
        let rejected = LootTable::parse(&table_path(), bytes, &cx()).unwrap_err();
        let error = rejected.diagnostics.first_error().unwrap();
        // A file of no value at all (07: "the file is empty, or parses to nothing / `null`")
        // is `EmptyDocument`, as in every family; vanilla's line is the same.
        let nothing = !bytes.starts_with(b"{") && !bytes.starts_with(b"\"");
        assert_eq!(
            // By legacy name: the variant and the definitions that were it.
            matches!(error.code.name(), "EmptyDocument"),
            nothing,
            "{error:?}"
        );
        assert_eq!(
            // By legacy name: the variant and the definitions that were it.
            matches!(error.code.name(), "MalformedInput"),
            !nothing,
            "{error:?}"
        );
        assert_eq!(
            text(&error.code),
            "Loot table for loot_tables/test.json is invalid."
        );
    }
}

#[test]
fn v03_a_pool_that_is_not_an_object_is_dropped_with_a_warning_deviation() {
    // Vanilla appends an empty pool (0 rolls, no entries) and says nothing; we drop the
    // element with our `InvalidType` warning, so nothing is invented on write-back.
    let table = load(r#"{"pools": [5, {"rolls": 1, "entries": []}, "x", null]}"#);
    let pools = table.pools.as_deref().unwrap();
    assert_eq!(pools.len(), 1);
    assert_eq!(
        pools[0],
        Pool {
            rolls: Some(bedrock_addon_family_loot::RandomValueBounds::Exact(1.0)),
            entries: Some(Vec::new()),
            ..Pool::default()
        }
    );
    let found = findings(&table);
    assert_eq!(found.len(), 3, "{found:?}");
    assert!(
        found
            .iter()
            .all(|d| d.severity == Severity::Info && d.is(&core_codes::INVALID_TYPE))
    );
    let paths: Vec<String> = found.iter().map(|d| path(d)).collect();
    assert_eq!(paths, ["pools/0", "pools/2", "pools/3"]);
    // The rewrite invents no `{}` and raises nothing the document did not.
    let written = table.to_json().unwrap();
    assert!(!written.contains("{}"), "{written}");
    let again = load(&written);
    assert!(findings(&again).is_empty(), "{:?}", findings(&again));
    assert_eq!(again.value(), table.value());
    assert_eq!(again.to_json().unwrap(), written);

    // The same in an entry's inline sub-table.
    let nested = load(r#"{"pools": [{"rolls": 1, "entries": [{"type": "empty", "pools": [7]}]}]}"#);
    let entry = &nested.pools.as_deref().unwrap()[0].entries()[0];
    assert_eq!(entry.pools, Some(Vec::new()));
    let found = findings(&nested);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(path(found[0]), "pools/0/entries/0/pools/0");
}

#[test]
fn v04_entries_that_are_not_an_array_are_the_game_s_content_log_error() {
    // The same test and line as a missing `entries`: an error, the
    // pool kept empty with the table.
    let table = load(r#"{"pools": [{"rolls": 1, "entries": {"type": "empty"}}]}"#);
    assert!(table.pools.as_deref().unwrap()[0].entries().is_empty());
    let found = findings(&table);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].code.name() == "InvalidType" && found[0].code.vanilla().is_some());
    assert_eq!(found[0].severity, Severity::Error);
    // A kept error: the table loads with it.
    assert!(found[0].kept, "{found:?}");
    assert_eq!(path(found[0]), "pools/0/entries");
    assert_eq!(
        text(&found[0].code),
        "There must be an \"entries\" array inside the loot table pool."
    );
}

#[test]
fn v05_an_entry_that_is_not_an_object_is_skipped() {
    let table = load(r#"{"pools": [{"rolls": 1, "entries": ["item", {"type": "empty"}]}]}"#);
    let entries = table.pools.as_deref().unwrap()[0].entries();
    assert_eq!(entries.len(), 1);
    assert!(matches!(entries[0].kind, EntryKind::Empty));
    let found = findings(&table);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].code.name() == "InvalidType" && found[0].code.vanilla().is_some());
    assert_eq!(found[0].severity, Severity::Error);
    assert_eq!(path(found[0]), "pools/0/entries/0");
    assert_eq!(
        text(&found[0].code),
        "Every entry of \"entries\" must be a JSON object - it has to start with an \"{\" and end with a \"}\""
    );
}

#[test]
fn v08_a_function_that_is_not_an_object_is_skipped() {
    let table = load(
        r#"{"pools": [{"rolls": 1, "entries": [{"type": "item", "name": "minecraft:stick",
            "functions": ["set_count", {"function": "furnace_smelt"}]}]}]}"#,
    );
    let EntryKind::Item { functions, .. } = &table.pools.as_deref().unwrap()[0].entries()[0].kind
    else {
        panic!("an item entry");
    };
    assert_eq!(functions.as_deref().unwrap().len(), 1);
    let found = findings(&table);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].code.name() == "InvalidType" && found[0].code.vanilla().is_some());
    assert_eq!(found[0].severity, Severity::Error);
    assert_eq!(path(found[0]), "pools/0/entries/0/functions/0");
    assert_eq!(
        text(&found[0].code),
        "Error loading loot table \"functions\" as it is not a valid Json object. \"functions\" must be key: value pairs"
    );
}

#[test]
fn v12_a_condition_that_is_not_an_object_is_skipped() {
    let table = load(
        r#"{"pools": [{"rolls": 1, "conditions": [true, {"condition": "is_baby"}], "entries": []}]}"#,
    );
    let pool = &table.pools.as_deref().unwrap()[0];
    assert_eq!(pool.conditions.as_deref().unwrap().len(), 1);
    let found = findings(&table);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].code.name() == "InvalidType" && found[0].code.vanilla().is_some());
    assert_eq!(found[0].severity, Severity::Error);
    assert_eq!(path(found[0]), "pools/0/conditions/0");
    assert_eq!(
        text(&found[0].code),
        "Error loading loot table \"conditions\" as it is not a valid Json object. \"conditions\" must be key: value pairs"
    );
}

#[test]
fn v03_pools_functions_and_conditions_given_as_objects_are_read_by_member_value() {
    // nothing, with our `InvalidType` warning. The object form is written back as an
    // array.
    let table = load(
        r#"{"pools": {"first": {"rolls": 1, "conditions": {"a": {"condition": "is_baby"}},
            "entries": [{"type": "item", "name": "minecraft:stick",
                         "functions": {"f": {"function": "furnace_smelt"}}}]}}}"#,
    );
    assert!(findings(&table).is_empty(), "{:?}", findings(&table));
    let pools = table.pools.as_deref().unwrap();
    assert_eq!(pools.len(), 1);
    assert_eq!(pools[0].conditions.as_deref().unwrap().len(), 1);
    let EntryKind::Item { functions, .. } = &pools[0].entries()[0].kind else {
        panic!("an item entry");
    };
    assert_eq!(functions.as_deref().unwrap().len(), 1);
    assert!(table.to_json().unwrap().contains(r#""pools": ["#));

    let scalar = load(
        r#"{"pools": [{"rolls": 1, "conditions": 5, "entries": [{"type": "item", "name": "minecraft:stick", "functions": "x"}]}]}"#,
    );
    let wrong: Vec<String> = findings(&scalar)
        .into_iter()
        .filter(|d| d.is(&core_codes::INVALID_TYPE))
        .map(path)
        .collect();
    assert_eq!(wrong, ["pools/0/conditions", "pools/0/entries/0/functions"]);
    let none = load(r#"{"pools": 5}"#);
    assert_eq!(none.pools, None);
    assert_eq!(findings(&none).len(), 1);
}

#[test]
fn v03_a_min_max_object_on_an_int_range_member_is_warned_and_kept() {
    // The game's integer range reader reads `range_min` / `range_max`; a `{min, max}` object
    // on such a member is the family's `InvalidType` warning, the value kept. A swapped range is
    // the shared type's `InvalidValue` info.
    let table = load(
        r#"{"pools": [{"rolls": 1, "conditions": [{"condition": "match_tool", "count": {"min": 1, "max": 2}, "durability": [5, 1]}], "entries": []}]}"#,
    );
    let found = findings(&table);
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found[0].is(&core_codes::INVALID_TYPE));
    assert_eq!(found[0].severity, Severity::Info);
    assert_eq!(path(found[0]), "pools/0/conditions/0/count");
    assert!(found[1].is(&bedrock_addon_core::diagnostic::codes::INVALID_VALUE));
    assert_eq!(found[1].severity, Severity::Info);
    assert_eq!(path(found[1]), "pools/0/conditions/0/durability");
    let ConditionKind::MatchTool {
        count, durability, ..
    } = &table.pools.as_deref().unwrap()[0]
        .conditions
        .as_deref()
        .unwrap()[0]
        .kind
    else {
        panic!("not match_tool");
    };
    assert_eq!(count.map(|c| (c.min, c.max)), Some((1, 2)));
    assert_eq!(
        durability.map(|d| d.resolve(i32::MIN, i32::MAX)),
        Some((1, 5))
    );
}

#[test]
fn v19_nothing_else_is_checked_when_loading() {
    // the validation rules: weights, qualities, chances, bounds order, potion ids, trims,
    // destinations and paths are accepted as read.
    let table = load(
        r#"{"pools": [{"rolls": {"min": 5, "max": -1}, "conditions": [{"condition": "random_chance", "chance": 7.5}],
            "entries": [{"type": "item", "name": "minecraft:potion", "weight": 0, "quality": -3, "functions": [
                {"function": "set_potion", "id": "no_such_potion"},
                {"function": "set_armor_trim", "material": "cheese", "pattern": "none"},
                {"function": "exploration_map", "destination": "nowhere"},
                {"function": "fill_container", "loot_table": "loot_tables/missing.json"},
                {"function": "set_stew_effect", "effects": {"id": 99}}]},
             {"type": "loot_table", "name": "nowhere.json", "weight": -1}]}]}"#,
    );
    assert!(findings(&table).is_empty(), "{:?}", findings(&table));
}
