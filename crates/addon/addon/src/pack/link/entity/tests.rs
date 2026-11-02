//! The group's own parts: the member tables, the member patterns and the cost of a pass.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::json::Value;

use super::super::tests::support::{all_said, block, pack, set_of};
use super::super::{LinkOptions, link_with_rules};
use super::table::{self, Row};
use super::walk::{members, path};

/// Every table, by name.
fn tables() -> Vec<(&'static str, &'static [Row])> {
    vec![
        #[cfg(feature = "block")]
        ("BLOCK_NAMES", table::BLOCK_NAMES),
        #[cfg(feature = "block")]
        ("BLOCKS", table::BLOCKS),
        #[cfg(feature = "block")]
        ("BLOCKS_AS_ITEMS", table::BLOCKS_AS_ITEMS),
        #[cfg(feature = "loot")]
        ("LOOT_TABLES", table::LOOT_TABLES),
        #[cfg(feature = "trade")]
        ("TRADE_TABLES", table::TRADE_TABLES),
        ("ENTITIES", table::ENTITIES),
        #[cfg(feature = "item")]
        ("ITEMS", table::ITEMS),
        ("SOUNDS", table::SOUNDS),
        #[cfg(feature = "particle")]
        ("PARTICLES", table::PARTICLES),
        #[cfg(feature = "poi")]
        ("POIS", table::POIS),
    ]
}

/// A table is sorted by key and pattern and has no row twice (the order of a component's
/// findings is the table's), and every key is one the component table of the entity family
/// has: a row for a key the game does not bind would never match.
#[test]
fn every_table_is_sorted_unique_and_of_known_components() {
    use crate::families::entity::component::EntityComponent;
    for (name, rows) in tables() {
        assert!(!rows.is_empty(), "{name}");
        assert!(
            rows.windows(2).all(|pair| pair[0] < pair[1]),
            "{name} is sorted and unique"
        );
        for (key, pattern) in rows {
            assert!(
                EntityComponent::TABLE.get(key).is_some(),
                "{name}: {key} is a component"
            );
            assert!(
                !pattern.is_empty() && !pattern.contains(".."),
                "{name}: {key} {pattern}"
            );
        }
    }
}

/// The two members the game reads as lists of block names (`std::vector<std::string>` in
/// its definitions of `home` and of the `raid_garden` goal) are no rows of the table of the
/// members that hold a block DESCRIPTOR: no `{name, states}` and no `{tags}` can stand there.
#[cfg(feature = "block")]
#[test]
fn a_list_of_block_names_is_no_row_of_the_descriptor_table() {
    let names = [
        ("minecraft:behavior.raid_garden", "blocks[]"),
        ("minecraft:home", "home_block_list[]"),
    ];
    for row in names {
        assert!(
            !table::BLOCKS.contains(&row),
            "{row:?} is a list of names and is listed as a descriptor member"
        );
    }
    assert_eq!(table::BLOCK_NAMES, names);
    // The rule tells the two tables apart by the component: no component is in both.
    for (key, _) in table::BLOCK_NAMES {
        assert!(table::BLOCKS.iter().all(|(other, _)| other != key), "{key}");
    }
}

fn json(text: &str) -> Value {
    crate::json::from_str::<Value>(text).expect("json")
}

/// What `pattern` names below `text`: each path and value.
fn named(text: &str, pattern: &str) -> Vec<(String, String)> {
    let value = json(text);
    let mut found = Vec::new();
    members(
        &value,
        pattern,
        &path(&["c"]),
        &mut |at: JsonPath, value| {
            let text = bedrock_addon_core::json::to_json(value).expect("writes");
            // Written without the layout of the writer.
            let text: String = text.split_whitespace().collect();
            found.push((at.to_string(), text));
        },
    );
    found
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(at, value)| ((*at).to_owned(), (*value).to_owned()))
        .collect()
}

#[test]
fn a_pattern_names_a_member_each_element_of_a_list_or_the_one_value_in_its_place() {
    assert_eq!(named(r#"{"a": "x"}"#, "a"), pairs(&[("c/a", r#""x""#)]));
    // `[]`: each element of an array, with its index; a value that is no array, itself.
    assert_eq!(
        named(r#"{"a": ["x", "y"]}"#, "a[]"),
        pairs(&[("c/a/0", r#""x""#), ("c/a/1", r#""y""#)])
    );
    assert_eq!(named(r#"{"a": "x"}"#, "a[]"), pairs(&[("c/a", r#""x""#)]));
    assert_eq!(named(r#"{"a": []}"#, "a[]"), pairs(&[]));
    // Without `[]` an array is the value.
    assert_eq!(named(r#"{"a": ["x"]}"#, "a"), pairs(&[("c/a", r#"["x"]"#)]));
    // Nested, through a list of objects or one object.
    assert_eq!(
        named(r#"{"a": [{"b": 1}, {"z": 2}, {"b": [3, 4]}]}"#, "a[].b[]"),
        pairs(&[("c/a/0/b", "1"), ("c/a/2/b/0", "3"), ("c/a/2/b/1", "4")])
    );
    assert_eq!(
        named(r#"{"a": {"b": 1}}"#, "a[].b"),
        pairs(&[("c/a/b", "1")])
    );
    assert_eq!(
        named(r#"{"a": {"b": {"d": 1}}}"#, "a.b.d"),
        pairs(&[("c/a/b/d", "1")])
    );
}

#[test]
fn a_pattern_names_nothing_where_the_member_is_absent_or_its_parent_no_object() {
    assert_eq!(named(r#"{"z": 1}"#, "a"), pairs(&[]));
    assert_eq!(named(r#"{"a": 1}"#, "a.b"), pairs(&[]));
    assert_eq!(named(r#"{"a": [1, "x"]}"#, "a[].b"), pairs(&[]));
    assert_eq!(named("[]", "a"), pairs(&[]));
    assert_eq!(named("5", "a[]"), pairs(&[]));
}

#[test]
fn a_star_names_every_member_of_an_object() {
    assert_eq!(
        named(r#"{"a": {"x": {"b": 1}, "y": {"b": 2}, "z": 3}}"#, "a.*.b"),
        pairs(&[("c/a/x/b", "1"), ("c/a/y/b", "2")])
    );
    assert_eq!(named(r#"{"a": 1}"#, "a.*"), pairs(&[]));
}

/// A world of `n` entities and `n` blocks: each entity names three blocks (its own, one of the
/// game's and one nothing defines, in another case than a block that is), an entity and
/// itself.
fn world(n: usize) -> crate::pack::StackSet {
    use bedrock_addon_core::load::LoadOptions;
    let mut files: Vec<(&'static str, Option<String>)> = Vec::new();
    let leak = |text: String| -> &'static str { Box::leak(text.into_boxed_str()) };
    for i in 0..n {
        files.push((
            leak(format!("blocks/b{i}.json")),
            Some(block(&format!("demo:block_{i}"), "{}")),
        ));
        let next = (i + 1) % n;
        files.push((
            leak(format!("entities/e{i}.json")),
            Some(format!(
                r#"{{"format_version": "1.26.50", "minecraft:entity": {{"description": {{"identifier": "demo:pet_{i}"}}, "components": {{
                    "minecraft:break_blocks": {{"breakable_blocks": ["demo:block_{i}", "minecraft:stone", "Demo:Block_{next}"]}},
                    "minecraft:trail": {{"block_type": "demo:none_{i}"}},
                    "minecraft:transformation": {{"into": "demo:pet_{next}"}},
                    "minecraft:shooter": {{"def": "demo:ghost_{i}"}}
                }}}}}}"#
            )),
        ));
    }
    let files: Vec<(&'static str, Option<&str>)> = files
        .iter()
        .map(|(path, text)| (*path, text.as_deref()))
        .collect();
    let set = set_of(
        vec![pack(1, "data", &files)],
        Vec::new(),
        &LoadOptions::default(),
    );
    // The load is not the pass's: both collections are read before the clock starts.
    assert_eq!(set.behavior.entities().len(), n);
    #[cfg(feature = "block")]
    assert_eq!(set.behavior.blocks().len(), n);
    set
}

/// What a pass over the world of `n` entities tells.
fn told(n: usize) -> usize {
    let per_entity = if cfg!(feature = "block") { 3 } else { 1 };
    n * per_entity
}

#[test]
fn a_pass_over_n_entities_with_n_references_costs_n() {
    crate::test_cost::assert_linear_at(
        "link entity group: n entities, n blocks",
        700,
        world,
        |set: &crate::pack::StackSet| {
            let found = link_with_rules(set, LinkOptions::default(), &[super::run]);
            assert_eq!(found.len(), told(set.behavior.entities().len()));
            found.len()
        },
    );
}

/// The pass over the small world, in full: each finding once, in document order.
#[test]
fn a_pass_tells_each_reference_once() {
    let set = world(2);
    let found = link_with_rules(&set, LinkOptions::default(), &[super::run]);
    let said: Vec<String> = all_said(&found)
        .into_iter()
        .map(|(_, _, at, kind, id)| format!("{kind} {id} {at}"))
        .collect();
    let c = "minecraft:entity/components/minecraft:";
    let mut expected = Vec::new();
    #[cfg(feature = "block")]
    for i in 0..2 {
        let next = (i + 1) % 2;
        expected.push(format!(
            "block Demo:Block_{next} {c}break_blocks/breakable_blocks/2"
        ));
        expected.push(format!("block demo:none_{i} {c}trail/block_type"));
    }
    for i in 0..2 {
        expected.push(format!("entity demo:ghost_{i} {c}shooter/def"));
    }
    assert_eq!(said, expected);
}
