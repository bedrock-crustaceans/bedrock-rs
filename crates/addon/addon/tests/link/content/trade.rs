//! The items of a trade table (20 T1) and the trade row that is not checked (T2).

#![allow(
    unused_imports,
    dead_code,
    reason = "which tests exist depends on the enabled features"
)]

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::LinkOptions;

use super::super::common::{hosted, stacks};
use super::said_of;
#[cfg(feature = "item")]
use super::{one, said, text};

const NO_LINTS: LinkOptions = LinkOptions {
    lints: false,
    client: true,
};
const FILE: &str = "trading/demo_trades.json";

/// A table of one trade that wants `wants` and gives `gives` (JSON arrays).
fn trades(wants: &str, gives: &str) -> String {
    format!(
        r#"{{"format_version": "1.21.30", "tiers": [{{"total_exp_required": 0, "groups": [{{"trades": [{{"wants": {wants}, "gives": {gives}}}]}}]}}]}}"#
    )
}

fn linked(
    behavior: &[(&'static str, &str)],
    load: LoadOptions,
    options: LinkOptions,
) -> bedrock_addon::diagnostic::Diagnostics {
    let set = stacks(behavior, &[], load);
    assert_eq!(
        set.behavior.trade_tables().iter().count(),
        1,
        "{:#?}",
        set.behavior
            .trade_tables()
            .diagnostics()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    set.link_with(&options)
}

#[cfg(feature = "item")]
mod item {
    use super::*;

    const KINDS: [&str; 1] = ["item"];

    #[test]
    fn t1_an_item_of_the_stack_or_of_the_game_is_no_finding_with_or_without_an_aux_value() {
        let gem = ("items/gem.json", &*super::super::item("demo:gem", ""));
        let table = trades(
            r#"[{"item": "minecraft:wheat"}, {"choice": [{"item": "minecraft:wool:14"}, {"item": "demo:gem"}]}]"#,
            // A name of an older version, as the game's own trade tables spell it.
            r#"[{"item": "demo:gem:2"}, {"item": "emerald"}, {"item": "minecraft:emptyMap"}, {"item": "minecraft:emptyMap:2"}]"#,
        );
        let found = linked(
            &[(FILE, &table), gem],
            LoadOptions::default(),
            LinkOptions::default(),
        );
        assert_eq!(said(&found, &KINDS), [], "{found:#?}");
        let host = hosted(&[("item", "host:coin")]);
        let table = trades(r#"[{"item": "host:coin"}]"#, r#"[{"item": "host:coin:1"}]"#);
        let found = linked(&[(FILE, &table)], host, LinkOptions::default());
        assert_eq!(said(&found, &KINDS), []);
    }

    // Without `block` a name outside `minecraft` may be a pack's block: not in view.
    /// A name of an older version is compared without case (the game's trade tables write
    /// `emptyMap`, its recipes `emptymap`); the fold is those names' alone.
    // K113: one list for every group, compared as the game does.
    #[test]
    fn t1_a_legacy_name_resolves_in_any_case_and_no_other_name_does() {
        let table = trades(
            r#"[{"item": "minecraft:wheat"}]"#,
            r#"[{"item": "minecraft:EmptyMap"}, {"item": "minecraft:WOOD"}, {"item": "minecraft:Wheat"}, {"item": "demo:emptymap"}]"#,
        );
        let found = linked(&[(FILE, &table)], LoadOptions::default(), NO_LINTS);
        let named: Vec<_> = said(&found, &KINDS)
            .into_iter()
            .map(|said| said.5)
            .collect();
        assert_eq!(named, ["minecraft:Wheat", "demo:emptymap"]);
    }

    /// A table of the older layout (no `format_version`) may write a tier's trades directly,
    /// without groups: the finding names the member as that file has it.
    #[test]
    fn t1_a_table_of_the_older_layout_is_told_at_the_member_the_file_has() {
        let flat = r#"{"tiers": [{"trades": [{"wants": [{"item": "minecraft:wheat", "quantity": 1}], "gives": [{"item": "minecraft:not_an_item"}]}]}]}"#;
        let found = linked(&[(FILE, flat)], LoadOptions::default(), NO_LINTS);
        let paths: Vec<_> = said(&found, &KINDS)
            .into_iter()
            .map(|said| said.3)
            .collect();
        assert_eq!(paths, ["tiers/0/trades/0/gives/0/item"]);
        // Groups, the first of them without a trade (the upgrade does not keep it): the
        // group's own index in the file.
        let grouped = r#"{"tiers": [{"groups": [{"trades": []}, {"num_to_select": 1, "trades": [{"wants": [{"item": "minecraft:not_an_item", "quantity": 1}], "gives": [{"item": "minecraft:emerald"}]}]}]}]}"#;
        let found = linked(&[(FILE, grouped)], LoadOptions::default(), NO_LINTS);
        let paths: Vec<_> = said(&found, &KINDS)
            .into_iter()
            .map(|said| said.3)
            .collect();
        assert_eq!(paths, ["tiers/0/groups/1/trades/0/wants/0/item"]);
    }

    /// The older layout's reader leaves out a trade without `wants` or `gives` or with a
    /// weight of 0, an entry or a `choice` element that is no object, an entry whose
    /// `choice` is empty, and a group whose `trades` is no list; the upgrade leaves out a
    /// group all of whose trades went. An item behind any of them is at the index the file
    /// has it at, at every level.
    #[test]
    fn t1_an_item_behind_what_the_older_reader_leaves_out_is_at_the_index_of_the_file() {
        let paths = |table: &str| -> Vec<String> {
            let found = linked(&[(FILE, table)], LoadOptions::default(), NO_LINTS);
            said(&found, &KINDS)
                .into_iter()
                .map(|said| said.3)
                .collect()
        };
        let grouped = r#"{"tiers": [{"groups": [
            {"trades": 7},
            {"trades": [{"wants": [{"item": "minecraft:wheat"}]}]},
            {"trades": [
                {"wants": [{"item": "minecraft:wheat"}]},
                {"wants": [{"item": "minecraft:wheat"}], "gives": [{"item": "minecraft:emerald"}], "weight": 0},
                {"wants": [7, {"item": "demo:none_a"}],
                 "gives": [{"choice": []}, {"choice": [7, {"item": "demo:none_b"}]}]}
            ]}
        ]}]}"#;
        assert_eq!(
            paths(grouped),
            [
                "tiers/0/groups/2/trades/2/wants/1/item",
                "tiers/0/groups/2/trades/2/gives/1/choice/1/item",
            ]
        );
        // The flat form: the tier's own `trades`.
        let flat = r#"{"tiers": [{"trades": [
            {"gives": [{"item": "minecraft:emerald"}]},
            {"wants": [{"item": "minecraft:wheat"}], "gives": [7, {"item": "demo:none"}]}
        ]}]}"#;
        assert_eq!(paths(flat), ["tiers/0/trades/1/gives/1/item"]);
    }

    /// Below 1.18.20 an item with the function `trader_material_type` becomes five items of
    /// a `choice`: the five are the one item of the file, and an item behind them in a
    /// `choice` is at the index the file has it at.
    #[test]
    fn t1_an_item_the_upgrade_makes_five_of_is_told_where_the_file_has_the_one() {
        let table = r#"{"tiers": [{"trades": [{
            "wants": [{"item": "demo:none_boat", "functions": [{"function": "trader_material_type"}]}],
            "gives": [{"choice": [
                {"item": "minecraft:boat", "functions": [{"function": "trader_material_type"}]},
                {"item": "demo:none"}]}]
        }]}]}"#;
        let found = linked(&[(FILE, table)], LoadOptions::default(), NO_LINTS);
        let told: Vec<_> = said(&found, &KINDS)
            .into_iter()
            .map(|said| (said.3, said.5))
            .collect();
        let wants = "tiers/0/trades/0/wants/0/item";
        let expected: Vec<(String, String)> = [
            (wants, "demo:none_boat:0"),
            (wants, "demo:none_boat:1"),
            (wants, "demo:none_boat:3"),
            (wants, "demo:none_boat:4"),
            (wants, "demo:none_boat:5"),
            ("tiers/0/trades/0/gives/0/choice/1/item", "demo:none"),
        ]
        .into_iter()
        .map(|(path, id)| (path.to_owned(), id.to_owned()))
        .collect();
        assert_eq!(told, expected, "{found:#?}");
    }

    #[cfg(feature = "block")]
    #[test]
    fn t1_an_item_nothing_defines_is_an_error_at_the_member_that_names_it() {
        const TRADE: &str = "tiers/0/groups/0/trades/0";

        let table = trades(
            r#"[{"item": "demo:none_a"}, {"choice": [{"item": "minecraft:wheat"}, {"item": "demo:none_b:3"}]}]"#,
            r#"[{"item": "demo:none_c"}]"#,
        );
        let found = linked(&[(FILE, &table)], LoadOptions::default(), NO_LINTS);
        let expected: Vec<_> = [
            ("wants/0/item", "demo:none_a"),
            ("wants/1/choice/1/item", "demo:none_b:3"),
            ("gives/0/item", "demo:none_c"),
        ]
        .into_iter()
        .map(|(path, id)| {
            one(
                "UnresolvedReference",
                Severity::Error,
                FILE,
                &format!("{TRADE}/{path}"),
                "item",
                id,
            )
        })
        .collect();
        assert_eq!(said(&found, &KINDS), expected, "{found:#?}");
        let finding = found
            .iter()
            .find(|finding| said_of(finding).4 == "item")
            .unwrap();
        assert_eq!(
            finding
                .location
                .pack
                .as_ref()
                .map(bedrock_addon::pack::PackRef::name),
            Some("bp 1")
        );
        // the root.
        assert_eq!(
            text(finding).as_deref(),
            Some(
                "-> tiers[0] -> groups[0] -> trades[0] -> wants[0] -> item:  Missing referenced asset demo:none_a"
            )
        );
    }
}

#[cfg(not(feature = "item"))]
#[test]
fn t1_without_the_item_feature_nothing_is_told() {
    let table = trades(r#"[{"item": "demo:none"}]"#, r#"[{"item": "demo:none"}]"#);
    let found = linked(&[(FILE, &table)], LoadOptions::default(), &NO_LINTS);
    assert_eq!(found.iter().map(said_of).collect::<Vec<_>>(), []);
}

/// 20 T2: the `functions` of a trade item are loot functions the game reads when the trade
/// is made; neither the load nor the pass looks into them.
#[test]
fn t2_the_functions_of_a_trade_item_are_not_checked() {
    let table = trades(
        r#"[{"item": "minecraft:wheat"}]"#,
        r#"[{"item": "minecraft:emerald", "functions": [{"function": "no_such_function"}, {"function": "set_actor_id", "id": "demo:nobody"}, {"function": "fill_container", "loot_table": "loot_tables/none.json"}, {"function": "set_armor_trim", "pattern": "spiral"}]}]"#,
    );
    let set = stacks(&[(FILE, &table)], &[], LoadOptions::default());
    let tables = set.behavior.trade_tables();
    assert_eq!(tables.iter().count(), 1);
    assert_eq!(
        tables
            .diagnostics()
            .filter(|finding| finding.severity >= Severity::Warning)
            .count(),
        0
    );
    let found = set.link();
    assert_eq!(found.iter().map(said_of).collect::<Vec<_>>(), []);
}
