//! Documents stamped 1.21.30 or later: the strict cereal slice (fixtures t07, t08).

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_family_trade::{ItemList, Quantity};

use super::common::{above_info, fixture, found_at, loads, rejects, trade};

#[test]
fn t08_the_strict_slice_rejects_flat_trades_biomes_and_a_missing_total_exp_required() {
    let d = rejects(&fixture("t08_butcher_trades_1_26_10_strict.json"));
    found_at(&d, "UnrecognizedField", Severity::Error, "tiers/0/trades");
    found_at(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "tiers/0/groups/0/trades/0/wants/0/choice/0/biomes",
    );
    found_at(
        &d,
        "MissingRequiredField",
        Severity::Error,
        "tiers/1/total_exp_required",
    );
    assert_eq!(above_info(&d).len(), 3, "{d:#?}");
}

#[test]
fn t08_the_same_json_without_format_version_loads_through_the_legacy_parser() {
    let stamped = fixture("t08_butcher_trades_1_26_10_strict.json");
    let unstamped = stamped.replace(r#""format_version": "1.26.10","#, "");
    assert_ne!(stamped, unstamped);
    let document = loads(&unstamped);
    assert_eq!(document.version().parsed_as, FormatVersion::new(0, 0, 0));
    let table = document.value();
    // Tier 0 keeps its two groups; its flat `trades` is not read (ours: a warning).
    assert_eq!(table.tiers[0].groups.len(), 2);
    found_at(
        document.diagnostics(),
        "DroppedField",
        Severity::Info,
        "tiers/0/trades",
    );
    assert!(above_info(document.diagnostics()).is_empty());
    // The missing `total_exp_required` is the tier's index; `biomes` is read by the gate.
    assert_eq!(table.tiers[1].total_exp_required, 1);
    // A written one is kept.
    assert_eq!(table.tiers[2].total_exp_required, 70);
    assert!(
        trade(table, 0, 0, 0).wants[0].items()[0]
            .legacy_filters
            .is_some()
    );
}

#[test]
fn t07_vanillas_1_26_10_butcher_table_loads_with_no_diagnostic() {
    let document = loads(&fixture("t07_butcher_trades_1_26_10.json"));
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 21, 30));
    assert_eq!(
        document.version().written,
        Some(FormatVersion::new(1, 26, 10))
    );
    assert!(!document.version().upgraded);
    let table = document.value();
    assert_eq!(table.tiers.len(), 5);
    assert_eq!(table.tiers[2].total_exp_required, 70);
    // Every entry is `{choice: [...]}`, every item has `functions: []`.
    for tier in &table.tiers {
        for group in &tier.groups {
            for trade in &group.trades {
                for list in trade.wants.iter().chain(&trade.gives) {
                    let ItemList::Choice { choice } = list else {
                        panic!("a bare item: {list:?}");
                    };
                    for item in choice {
                        assert_eq!(item.functions.as_deref(), Some(&[][..]));
                    }
                }
            }
        }
    }
    // Tier 0, group 1, trade 0 writes `max_uses` only: `trader_exp` reads 1.
    let first = trade(table, 0, 1, 0);
    assert_eq!(first.trader_exp, None);
    assert_eq!(first.trader_exp(), 1);
    assert_eq!(first.max_uses(), 12);
    let chicken = &trade(table, 0, 0, 0).wants[0].items()[0];
    assert_eq!(chicken.item.as_str(), "minecraft:chicken");
    assert_eq!(chicken.quantity_range(), (14, 14));
    assert_eq!(chicken.price_multiplier, Some(0.05));
    assert_eq!(table.tiers[0].groups[0].num_to_select(), 1);
}

#[test]
fn t07_omitted_trade_members_read_their_defaults_and_max_only_reads_one_to_max() {
    let document = loads(&fixture("t07_cleric_trades_1_26_10.json"));
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    let table = document.value();
    let plain = trade(table, 0, 0, 0);
    assert_eq!((plain.trader_exp, plain.max_uses), (None, None));
    assert_eq!((plain.trader_exp(), plain.max_uses()), (1, 7));
    assert!(plain.reward_exp());
    assert_eq!(plain.weight(), 1);
    let dye = &trade(table, 1, 0, 1).gives[0].items()[0];
    assert_eq!(dye.item.as_str(), "minecraft:dye:4");
    assert_eq!(
        dye.quantity,
        Some(Quantity::Range {
            min: None,
            max: Some(2)
        })
    );
    assert_eq!(dye.quantity_range(), (1, 2));
}
