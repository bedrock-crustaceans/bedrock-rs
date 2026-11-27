//! 20 E13: the trade tables an entity's components name.

#![allow(unused_imports, reason = "each feature set compiles part of the tests")]

use bedrock_addon::pack::LinkOptions;

use super::super::common::{hosted, link, link_with};
use super::{FILE, pet, said_of, unresolved};

const TABLE: &str = r#"{"tiers": [{"trades": [{"wants": [{"item": "minecraft:wheat"}], "gives": [{"item": "minecraft:emerald"}]}]}]}"#;

const COMPONENTS: &str = r#"
    "minecraft:trade_table": {"table": "trading/none1.json", "display_name": "x"},
    "minecraft:economy_trade_table": {"table": "trading/economy_trades/none2.json", "display_name": "y"}
"#;

/// The game answers a missing table with null and no line, and both components
/// have only internal assertions for it: an information of ours.
#[cfg(feature = "trade")]
#[test]
fn e13_a_trade_table_nothing_defines_is_an_information() {
    let found = link(&[(FILE, &pet(COMPONENTS))], &[]);
    assert_eq!(
        said_of(&found, "trade table"),
        [
            unresolved(
                "components/minecraft:trade_table/table",
                "trade table",
                "trading/none1.json"
            ),
            unresolved(
                "components/minecraft:economy_trade_table/table",
                "trade table",
                "trading/economy_trades/none2.json"
            ),
        ]
    );
}

/// A table is its path in the pack, as written: a pack's file resolves by exactly that path,
/// and a path in another case or without the extension does not.
#[cfg(feature = "trade")]
#[test]
fn e13_a_table_a_pack_defines_resolves_by_its_path_as_written() {
    let components = r#"
        "minecraft:trade_table": {"table": "trading/pet.json"},
        "minecraft:economy_trade_table": {"table": "trading/Pet.json"}
    "#;
    let found = link(
        &[(FILE, &pet(components)), ("trading/pet.json", TABLE)],
        &[],
    );
    assert_eq!(
        said_of(&found, "trade table"),
        [unresolved(
            "components/minecraft:economy_trade_table/table",
            "trade table",
            "trading/Pet.json"
        )]
    );
}

/// The game's tables are the host's to answer for; a member that is absent or empty names
/// none.
#[cfg(feature = "trade")]
#[test]
fn e13_a_host_answers_for_the_games_tables_and_an_empty_member_names_none() {
    let components = r#"
        "minecraft:trade_table": {"display_name": "x"},
        "minecraft:economy_trade_table": {"table": "trading/economy_trades/farmer_trades.json"}
    "#;
    let files = [(FILE, &*pet(components))];
    assert_eq!(said_of(&link(&files, &[]), "trade table").len(), 1);
    let found = link_with(
        &files,
        &[],
        hosted(&[("trade table", "trading/economy_trades/farmer_trades.json")]),
        LinkOptions::default(),
    );
    assert!(found.is_empty(), "{found:?}");
}

#[cfg(not(feature = "trade"))]
#[test]
fn e13_without_the_trade_feature_no_trade_table_is_told() {
    let found = link(&[(FILE, &pet(COMPONENTS))], &[]);
    assert!(said_of(&found, "trade table").is_empty(), "{found:?}");
    let _ = (unresolved("", "", ""), hosted(&[]), TABLE);
    let _ = (LinkOptions::default(), link_with as fn(_, _, _, _) -> _);
}

/// A table whose file the pass could not judge (a baked binary document, which the game
/// reads and this crate does not) is not told; a table whose file the load refused is no
/// table, for the game either, and is told like any other miss.
#[cfg(feature = "trade")]
#[test]
fn e13_a_table_whose_file_could_not_be_read_is_not_told_and_a_refused_one_is() {
    use super::super::common::{behavior_pack_of, leak, stacks_of};
    let components = r#"
        "minecraft:trade_table": {"table": "trading/broken.json"},
        "minecraft:economy_trade_table": {"table": "trading/baked.json"}
    "#;
    let pack = behavior_pack_of(
        1,
        &[
            (FILE, leak(&pet(components))),
            ("trading/broken.json", b"{"),
            ("trading/baked.json", b"\x7FMCB\x01\x00\x00\x00"),
        ],
    );
    let set = stacks_of(
        vec![pack],
        Vec::new(),
        bedrock_addon::load::LoadOptions::default(),
    );
    let found = set.link();
    assert_eq!(
        said_of(&found, "trade table"),
        [unresolved(
            "components/minecraft:trade_table/table",
            "trade table",
            "trading/broken.json"
        )]
    );
}
