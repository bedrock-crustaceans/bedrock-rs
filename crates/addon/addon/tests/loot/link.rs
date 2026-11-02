//! Rules that resolve names against other families and run in the link pass, through
//! `StackSet::link`.

use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackOrigin, PackRepository, PackSelection, StackSet, WorldPacks};

use super::common::fixture;
use super::pack_common::{Spec, bp};

/// The stacks of one behaviour pack built from `spec`.
fn stacks(spec: &Spec) -> StackSet {
    let id = spec.id();
    let mut repository = PackRepository::new();
    repository.add_pack(spec.open(), PackOrigin::World);
    let world = WorldPacks {
        behavior: vec![PackSelection {
            pack_id: id.uuid,
            version: id.version,
            subpack: None,
        }],
        resource: Vec::new(),
    };
    repository.stacks(&world, LoadOptions::default())
}

/// The findings of code `name` and `severity` at the file `path`, with their messages.
fn messages(d: &Diagnostics, name: &str, severity: Severity, path: &str) -> Vec<String> {
    d.iter()
        .filter(|f| {
            f.code.name() == name
                && f.severity == severity
                && f.location.path.as_ref().is_some_and(|p| p.as_str() == path)
        })
        .map(|f| f.code.to_string())
        .collect()
}

/// Leaks a fixture's bytes: `Spec::file` takes `'static` bytes.
fn leaked(name: &str) -> &'static [u8] {
    Box::leak(fixture(name).into_boxed_slice())
}

#[test]
fn lt09_an_unknown_item_name_is_warned_with_vanillas_text() {
    let spec = bp(1).file("loot_tables/bad_item.json", leaked("bad_item.json"));
    let set = stacks(&spec);
    // The entry is kept: the table loads with both entries.
    let table = set
        .behavior
        .loot_tables()
        .iter()
        .next()
        .map(|(_, table)| table.pools.as_deref().unwrap()[0].entries().len());
    assert_eq!(table, Some(2));
    let d = set.link();
    // X14: vanilla logs at level 3 and keeps the entry: a kept error.
    assert_eq!(
        messages(
            &d,
            "UnresolvedReference",
            Severity::Error,
            "loot_tables/bad_item.json"
        ),
        ["minecraft:not_an_item is not a valid item name."],
        "{d:#?}"
    );
    assert!(
        d.iter()
            .filter(|f| f.code.name() == "UnresolvedReference")
            .all(|f| f.kept),
        "{d:#?}"
    );
}

#[test]
fn l2_an_unresolved_loot_table_reference_is_an_info_deviation() {
    // `vanilla = "missing referenced loot table is silent"`.
    let spec = bp(1)
        .file(
            "loot_tables/refs.json",
            br#"{"pools": [{"rolls": 1, "entries": [
                {"type": "loot_table", "name": "minecraft:loot_tables/chests/missing.json"},
                {"type": "loot_table", "name": "loot_tables/chests/there.json"},
                {"type": "item", "name": "minecraft:chest", "functions": [
                    {"function": "fill_container", "loot_table": "loot_tables/chests/gone.json"}]}]}]}"#,
        )
        .file(
            "loot_tables/chests/there.json",
            br#"{"pools": []}"#,
        );
    let set = stacks(&spec);
    let d = set.link();
    let found: Vec<_> = d
        .iter()
        .filter(|f| {
            f.code.name() == "UnresolvedReference"
                && f.location
                    .path
                    .as_ref()
                    .is_some_and(|p| p.as_str() == "loot_tables/refs.json")
        })
        .collect();
    assert_eq!(found.len(), 2, "{d:#?}");
    assert!(found.iter().all(|f| f.severity == Severity::Info));
}

/// The `UnresolvedReference` findings of `d` at `loot_tables/refs.json`, as `(severity, JSON
/// path)`.
fn unresolved(d: &Diagnostics) -> Vec<(Severity, String)> {
    d.iter()
        .filter(|f| {
            f.code.name() == "UnresolvedReference"
                && f.location
                    .path
                    .as_ref()
                    .is_some_and(|p| p.as_str() == "loot_tables/refs.json")
        })
        .map(|f| (f.severity, f.location.json_path.to_string()))
        .collect()
}

#[test]
fn l5_an_unresolved_set_actor_id_is_an_info() {
    let spec = bp(1).file(
        "loot_tables/refs.json",
        br#"{"pools": [{"rolls": 1, "entries": [{"type": "item", "name": "minecraft:stick",
            "functions": [{"function": "set_actor_id", "id": "demo:nobody"}]}]}]}"#,
    );
    let d = stacks(&spec).link();
    // Vanilla accepts the id as read; the info is the link pass's (`I`).
    assert_eq!(
        unresolved(&d),
        [(
            Severity::Info,
            "pools/0/entries/0/functions/0/id".to_owned()
        )],
        "{d:#?}"
    );
}

#[test]
fn l5_an_unresolved_condition_entity_type_is_an_info() {
    let spec = bp(1).file(
        "loot_tables/refs.json",
        br#"{"pools": [{"rolls": 1, "conditions": [
            {"condition": "killed_by_entity", "entity_type": "demo:nobody"},
            {"condition": "damaged_by_entity", "entity_type": "demo:nobody"},
            {"condition": "passenger_of_entity", "entity_type": "demo:nobody"},
            {"condition": "entity_killed", "entity_type": "demo:nobody"}],
            "entries": [{"type": "empty"}]}]}"#,
    );
    let d = stacks(&spec).link();
    let expected: Vec<(Severity, String)> = (0..4)
        .map(|i| {
            (
                Severity::Info,
                format!("pools/0/conditions/{i}/entity_type"),
            )
        })
        .collect();
    assert_eq!(unresolved(&d), expected, "{d:#?}");
}

#[test]
fn l5_an_unresolved_match_tool_item_is_an_info() {
    let spec = bp(1).file(
        "loot_tables/refs.json",
        br#"{"pools": [{"rolls": 1, "conditions": [
            {"condition": "match_tool", "item": "demo:no_tool"}],
            "entries": [{"type": "empty"}]}]}"#,
    );
    let d = stacks(&spec).link();
    assert_eq!(
        unresolved(&d),
        [(Severity::Info, "pools/0/conditions/0/item".to_owned())],
        "{d:#?}"
    );
}
