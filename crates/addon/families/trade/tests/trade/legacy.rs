//! Documents read by the legacy parser and upgraded by `v0 → v1_21_30` (the upgrade steps;
//! fixtures t01, t02, t04, t09, t10, t11 and the F.6.6 list).

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::document::JsonDocument;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_family_trade::{Tier, TradeTable};

use super::common::{doc, fixture, found_at, loads, rejects, trade};

/// One tier with one group of one trade, as in a 1.26.10 file.
const ONE_TRADE: &str = r#"[{"total_exp_required": 0, "groups": [{"num_to_select": -1, "trades": [
    {"wants": [{"item": "minecraft:wheat", "quantity": 20}], "gives": [{"item": "minecraft:emerald"}]}
]}]}]"#;

#[test]
fn t01_flat_trades_become_one_group_of_all_and_tiers_take_their_index() {
    let document = loads(&fixture("t01_flat_to_groups.json"));
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    let table = document.value();
    assert_eq!(table.tiers.len(), 2);
    for (index, tier) in table.tiers.iter().enumerate() {
        assert_eq!(tier.total_exp_required, u32::try_from(index).unwrap());
        assert_eq!(tier.groups.len(), 1);
        assert_eq!(tier.groups[0].num_to_select, Some(-1));
    }
    assert_eq!(table.tiers[0].groups[0].trades.len(), 2);
    let first = trade(table, 0, 0, 0);
    assert_eq!((first.trader_exp, first.max_uses), (Some(2), Some(16)));
    assert_eq!(
        trade(table, 1, 0, 0).gives[0].items()[0].item.as_str(),
        "minecraft:bread"
    );
}

#[test]
fn t01_a_stamped_file_relies_on_the_schema_defaults() {
    let document = loads(&doc(
        Some("1.21.30"),
        r#"[{"total_exp_required": 0, "groups": [{"trades": [
            {"wants": [{"item": "minecraft:wheat"}], "gives": [{"item": "minecraft:emerald"}]}
        ]}]}]"#,
    ));
    let table = document.value();
    let group = &table.tiers[0].groups[0];
    assert_eq!((group.num_to_select, group.num_to_select()), (None, 0));
    let traded = trade(table, 0, 0, 0);
    assert_eq!(
        (traded.trader_exp(), traded.max_uses(), traded.weight()),
        (1, 7, 1)
    );
}

#[test]
fn t01_a_stamped_tier_without_total_exp_required_is_an_error() {
    let d = rejects(&doc(
        Some("1.21.30"),
        r#"[{"groups": [{"trades": [
            {"wants": [{"item": "minecraft:wheat"}], "gives": [{"item": "minecraft:emerald"}]}
        ]}]}]"#,
    ));
    found_at(
        &d,
        "MissingRequiredField",
        Severity::Error,
        "tiers/0/total_exp_required",
    );
}

#[test]
fn t11_a_legacy_trade_without_its_scalars_reads_the_stamped_defaults() {
    let legacy = loads(&doc(None, ONE_TRADE));
    assert!(
        legacy.diagnostics().is_empty(),
        "no MissingFormatVersion: {:#?}",
        legacy.diagnostics()
    );
    assert_eq!(legacy.version().written, None);
    assert_eq!(legacy.version().parsed_as, FormatVersion::new(0, 0, 0));
    assert!(legacy.version().upgraded);
    let traded = trade(legacy.value(), 0, 0, 0);
    assert_eq!(
        (
            traded.trader_exp(),
            traded.max_uses(),
            traded.reward_exp(),
            traded.weight()
        ),
        (1, 7, true, 1)
    );
    let stamped = loads(&doc(Some("1.21.30"), ONE_TRADE));
    assert_eq!(legacy.value(), stamped.value());
}

#[test]
fn t09_four_flat_tiers_take_0_1_2_3_as_vanillas_restamp_of_farmer_trades_wrote() {
    let legacy = loads(&fixture("t09_farmer_trades_unstamped.json"));
    let restamped = loads(&fixture("t09_farmer_trades_1_26_10.json"));
    let (legacy, restamped) = (legacy.value(), restamped.value());
    let totals = |table: &TradeTable| -> Vec<u32> {
        table.tiers.iter().map(|t| t.total_exp_required).collect()
    };
    assert_eq!(totals(legacy), [0, 1, 2, 3]);
    assert_eq!(totals(legacy), totals(restamped));
    for (old, new) in legacy.tiers.iter().zip(&restamped.tiers) {
        assert_eq!(old.groups.len(), 1);
        assert_eq!(old.groups[0].num_to_select, new.groups[0].num_to_select);
        let items = |tier: &Tier| -> Vec<(String, (u32, u32))> {
            tier.groups[0]
                .trades
                .iter()
                .flat_map(|t| t.wants.iter().chain(&t.gives))
                .flat_map(bedrock_addon_family_trade::ItemList::items)
                .map(|i| (i.item.as_str().to_owned(), i.quantity_range()))
                .collect()
        };
        assert_eq!(items(old), items(new));
    }
}

#[test]
fn t02_legacy_functions_and_filters_move_to_the_deprecated_side_cars() {
    // Held under the deprecated names; written back in the legacy shape, since the game
    // rejects a document that writes those names.
    // the upgrade steps' example, with a legacy filter beside the function.
    let document = loads(
        r#"{"tiers":[{"trades":[{"wants":[{"item":"minecraft:wheat","quantity":20,
            "functions":[{"function":"set_data","data":0}],
            "filters":{"test":"is_daytime","value":true}}],
            "gives":[{"item":"minecraft:emerald"}]}]}]}"#,
    );
    let item = &trade(document.value(), 0, 0, 0).wants[0].items()[0];
    assert_eq!(
        (item.functions.as_ref(), item.filters.as_ref()),
        (None, None)
    );
    assert!(item.legacy_functions.is_some() && item.legacy_filters.is_some());
    let written: Value = facet_json::from_str(&document.to_json().unwrap()).unwrap();
    let expected: Value = facet_json::from_str(
        r#"{"format_version":"1.20.30","tiers":[{"total_exp_required":0,"groups":[{"num_to_select":-1,"trades":[{
            "wants":[{"quantity":20,"item":"minecraft:wheat",
                "functions":[{"function":"set_data","data":0}],
                "filters":{"test":"is_daytime","value":true}}],
            "gives":[{"item":"minecraft:emerald"}]}]}]}]}"#,
    )
    .unwrap();
    assert_eq!(written, expected);
}

/// A tier whose group has no `num_to_select`.
const GROUP_WITHOUT_SELECT: &str = r#"[{"total_exp_required": 0, "groups": [{"trades": [
    {"wants": [{"item": "minecraft:wheat"}], "gives": [{"item": "minecraft:emerald"}]},
    {"wants": [{"item": "minecraft:potato"}], "gives": [{"item": "minecraft:emerald"}]}
]}]}]"#;

#[test]
fn t10_a_legacy_group_without_num_to_select_is_written_as_minus_one() {
    let legacy = loads(&doc(None, GROUP_WITHOUT_SELECT));
    assert_eq!(legacy.value().tiers[0].groups[0].num_to_select, Some(-1));
    let stamped = loads(&doc(Some("1.21.30"), GROUP_WITHOUT_SELECT));
    let group = &stamped.value().tiers[0].groups[0];
    assert_eq!((group.num_to_select, group.num_to_select()), (None, 0));
}

/// A legacy trade that goes
/// for its `wants`, `gives` or `weight`, and a group that goes for its `trades`, is told at the
/// trade or the group too: the member's line does not say that the other members go with it.
#[test]
fn a_legacy_trade_or_group_that_goes_whole_for_one_member_is_told_where_it_is() {
    let document = loads(&doc(
        None,
        r#"[{"groups": [
            {"num_to_select": 1, "trades": [
                {"wants": [{"item": "minecraft:emerald"}], "gives": [{"item": "minecraft:bread"}]},
                {"wants": 7, "gives": [{"item": "minecraft:carrot"}]},
                {"wants": [{"item": "minecraft:emerald"}], "gives": "x"},
                {"wants": [{"item": "minecraft:emerald"}], "gives": [{"item": "minecraft:cookie"}], "weight": 0}]},
            {"num_to_select": 1, "trades": 7}]}]"#,
    ));
    let d = document.diagnostics();
    for at in [
        "tiers/0/groups/0/trades/1",
        "tiers/0/groups/0/trades/2",
        "tiers/0/groups/0/trades/3",
        "tiers/0/groups/1",
    ] {
        found_at(d, "DroppedField", Severity::Info, at);
    }
    assert_eq!(document.tiers[0].groups.len(), 1);
    assert_eq!(document.tiers[0].groups[0].trades.len(), 1);
    // A group left without a trade is not upgraded (ours): a warning at the group.
    let document = loads(&doc(
        None,
        r#"[{"groups": [{"num_to_select": 0, "trades": []}, {"num_to_select": 1, "trades": [
            {"wants": [{"item": "minecraft:emerald"}], "gives": [{"item": "minecraft:bread"}]}]}]}]"#,
    ));
    found_at(
        document.diagnostics(),
        "DroppedField",
        Severity::Info,
        "tiers/0/groups/0",
    );
}

/// a member of a `trader_material_type` function entry other than `function` goes
/// with the expansion below 1.18.20 and is told where it is.
#[test]
fn a_member_of_the_material_function_the_expansion_does_not_read_is_told() {
    let document = loads(&doc(
        None,
        r#"[{"trades": [{"wants": [{"item": "minecraft:emerald"}], "gives": [{"item": "minecraft:boat", "functions": [{"function": "trader_material_type", "zz": 3}]}]}]}]"#,
    ));
    found_at(
        document.diagnostics(),
        "UnrecognizedField",
        Severity::Info,
        "tiers/0/trades/0/gives/0/functions/0/zz",
    );
}

/// What the step tells of a member is at the index the file has it at, behind a trade, an
/// entry or a `choice` element the reader left out, and behind a group it left out.
// The quantities are booleans, whose finding no number has: the legacy judge reads these
// documents too and puts numbers where they are, and a number's finding must be one the
// document does not have already.
#[test]
fn a_finding_of_the_step_behind_what_the_reader_leaves_out_is_at_the_index_of_the_file() {
    let document = loads(
        r#"{"tiers": [{"groups": [
            {"trades": 7},
            {"trades": [{"wants": [{"item": "minecraft:wheat"}]}]},
            {"trades": [
                {"wants": [{"item": "minecraft:wheat"}]},
                {"wants": [7, {"item": "minecraft:wheat", "quantity": false}],
                 "gives": [{"choice": []}, {"choice": [7, {"item": "minecraft:emerald", "quantity": true}]}]}
            ]}
        ]}]}"#,
    );
    let d = document.diagnostics();
    let upgrade: Vec<String> = d
        .iter()
        .filter(|f| f.phase == bedrock_addon_core::diagnostic::Phase::Upgrade)
        .map(|f| format!("{} {}", f.code.name(), f.location.json_path))
        .collect();
    assert_eq!(
        upgrade,
        [
            "InvalidValue tiers/0/groups/1/trades",
            "DroppedField tiers/0/groups/1",
            "InvalidType tiers/0/groups/2/trades/1/wants/1/quantity",
            "InvalidType tiers/0/groups/2/trades/1/gives/1/choice/1/quantity",
        ],
        "{d:#?}"
    );
    // The flat form.
    let document = loads(
        r#"{"tiers": [{"trades": [
            {"gives": [{"item": "minecraft:emerald"}]},
            {"wants": [{"item": "minecraft:wheat", "quantity": false}], "gives": [{"item": "minecraft:emerald"}]}
        ]}]}"#,
    );
    found_at(
        document.diagnostics(),
        "InvalidType",
        Severity::Info,
        "tiers/0/trades/1/wants/0/quantity",
    );
}

/// The other door, `parse_as_written` and then `history::upgrade`, tells the step's findings
/// where the file has the members too: the document it is given carries what the reader
/// recorded.
#[test]
fn the_step_run_on_a_document_as_written_tells_its_findings_at_the_index_of_the_file() {
    use bedrock_addon_core::pack_identity::PackPath;
    let text = r#"{"tiers": [{"groups": [
        {"trades": 7},
        {"trades": [
            {"wants": [{"item": "minecraft:wheat"}]},
            {"wants": [7, {"item": "minecraft:wheat", "quantity": false}],
             "gives": [{"item": "minecraft:emerald"}]}
        ]}
    ]}]}"#;
    let path = PackPath::from_static("trading/test_trades.json");
    let cx = super::common::cx();
    let written = TradeTable::parse_as_written(&path, text.as_bytes(), &cx).expect("it is read");
    let upgraded = bedrock_addon_family_trade::history::upgrade(written, &cx).expect("it loads");
    let told = |d: &bedrock_addon_core::diagnostic::Diagnostics| -> Vec<String> {
        d.iter()
            .map(|f| format!("{} {}", f.code.name(), f.location.json_path))
            .collect()
    };
    assert_eq!(
        told(upgraded.diagnostics()),
        told(loads(text).diagnostics())
    );
    found_at(
        upgraded.diagnostics(),
        "InvalidType",
        Severity::Info,
        "tiers/0/groups/1/trades/1/wants/1/quantity",
    );
}
