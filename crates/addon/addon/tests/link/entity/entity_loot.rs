//! 20 E12: the loot tables an entity's components name (a lint).

#![allow(unused_imports, reason = "each feature set compiles part of the tests")]

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::pack::LinkOptions;

use super::super::common::{hosted, link, link_with};
use super::{FILE, miss, pet, said_of};

const TABLE: &str = r#"{"pools": []}"#;

/// Every member of E12, each naming a table nothing defines.
const COMPONENTS: &str = r#"
    "minecraft:loot": {"table": "loot_tables/none1.json"},
    "minecraft:equipment": {"table": "loot_tables/none2.json"},
    "minecraft:barter": {"barter_table": "loot_tables/none3.json", "cooldown_after_being_attacked": 1},
    "minecraft:interact": {"interactions": [
        {"add_items": {"table": "loot_tables/none4.json"}},
        {"spawn_items": {"table": "loot_tables/none5.json"}}]},
    "minecraft:behavior.drop_item_for": {"priority": 1, "loot_table": "loot_tables/none6.json"},
    "minecraft:behavior.eat_mob": {"priority": 1, "loot_table": "loot_tables/none7.json"},
    "minecraft:behavior.sneeze": {"priority": 1, "loot_table": "loot_tables/none8.json"},
    "minecraft:behavior.random_search_and_dig": {"priority": 1, "item_table": "loot_tables/none9.json"}
"#;

fn info(path: &str, id: &str) -> super::Said {
    miss(
        "core/unresolved_reference",
        Severity::Info,
        &format!("minecraft:entity/components/minecraft:{path}"),
        "loot table",
        id,
    )
}

#[cfg(feature = "loot")]
#[test]
fn e12_a_loot_table_nothing_defines_is_an_information() {
    let found = link(&[(FILE, &pet(COMPONENTS))], &[]);
    assert_eq!(
        said_of(&found, "loot table"),
        [
            info("loot/table", "loot_tables/none1.json"),
            info("equipment/table", "loot_tables/none2.json"),
            info("barter/barter_table", "loot_tables/none3.json"),
            info(
                "interact/interactions/0/add_items/table",
                "loot_tables/none4.json"
            ),
            info(
                "interact/interactions/1/spawn_items/table",
                "loot_tables/none5.json"
            ),
            info(
                "behavior.drop_item_for/loot_table",
                "loot_tables/none6.json"
            ),
            info("behavior.eat_mob/loot_table", "loot_tables/none7.json"),
            info("behavior.sneeze/loot_table", "loot_tables/none8.json"),
            info(
                "behavior.random_search_and_dig/item_table",
                "loot_tables/none9.json"
            ),
        ]
    );
}

/// The row is a lint: without the lints nothing of it is told.
#[cfg(feature = "loot")]
#[test]
fn e12_without_the_lints_no_loot_table_is_told() {
    let found = link_with(
        &[(FILE, &pet(COMPONENTS))],
        &[],
        bedrock_addon::load::LoadOptions::default(),
        LinkOptions {
            lints: false,
            ..Default::default()
        },
    );
    assert!(said_of(&found, "loot table").is_empty(), "{found:?}");
}

/// The path is looked up as the loot family does (20 L2): the text up to the last `:` is
/// dropped, the rest compared without case, with or without `.json`.
#[cfg(feature = "loot")]
#[test]
fn e12_a_table_a_pack_defines_resolves_by_the_loot_lookups_normalisation() {
    let components = r#"
        "minecraft:loot": {"table": "loot_tables/entities/pet.json"},
        "minecraft:equipment": {"table": "demo:Loot_Tables/Entities/PET"},
        "minecraft:behavior.sneeze": {"priority": 1, "loot_table": "loot_tables/entities/pets.json"}
    "#;
    let found = link(
        &[
            (FILE, &pet(components)),
            ("loot_tables/entities/pet.json", TABLE),
        ],
        &[],
    );
    assert_eq!(
        said_of(&found, "loot table"),
        [info(
            "behavior.sneeze/loot_table",
            "loot_tables/entities/pets.json"
        )]
    );
}

/// The game's tables are the host's to answer for; without a host a table no pack has is
/// missing.
#[cfg(feature = "loot")]
#[test]
fn e12_a_host_answers_for_the_games_tables() {
    let files = [(
        FILE,
        &*pet(r#""minecraft:loot": {"table": "loot_tables/entities/zombie.json"}"#),
    )];
    assert_eq!(said_of(&link(&files, &[]), "loot table").len(), 1);
    let found = link_with(
        &files,
        &[],
        hosted(&[("loot table", "loot_tables/entities/zombie.json")]),
        LinkOptions::default(),
    );
    assert!(found.is_empty(), "{found:?}");
}

#[cfg(not(feature = "loot"))]
#[test]
fn e12_without_the_loot_feature_no_loot_table_is_told() {
    let found = link(&[(FILE, &pet(COMPONENTS))], &[]);
    assert!(said_of(&found, "loot table").is_empty(), "{found:?}");
    let _ = (
        info("", ""),
        hosted(&[]),
        TABLE,
        link_with as fn(_, _, _, _) -> _,
    );
    let _ = LinkOptions::default();
}

/// A table whose file the pass could not judge (a baked binary document) is not told
/// (compared as the loot lookup compares); one whose file the load refused is no table and
/// is told.
#[cfg(feature = "loot")]
#[test]
fn e12_a_table_whose_file_could_not_be_read_is_not_told_and_a_refused_one_is() {
    use super::super::common::{behavior_pack_of, leak, stacks_of};
    let components = r#"
        "minecraft:loot": {"table": "loot_tables/Broken.json"},
        "minecraft:equipment": {"table": "loot_tables/Baked"}
    "#;
    let pack = behavior_pack_of(
        1,
        &[
            (FILE, leak(&pet(components))),
            ("loot_tables/broken.json", b"{"),
            ("loot_tables/baked.json", b"\x7FMCB\x01\x00\x00\x00"),
        ],
    );
    let set = stacks_of(
        vec![pack],
        Vec::new(),
        bedrock_addon::load::LoadOptions::default(),
    );
    let found = set.link();
    assert_eq!(
        said_of(&found, "loot table"),
        [info("loot/table", "loot_tables/Broken.json")]
    );
}
