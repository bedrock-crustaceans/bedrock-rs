//! `bp.trade_tables()`: the walk of `trading/**/*.json`, the files keyed by path, the top
//! pack's copy winning, and a failing top copy leaving its path without a table.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::id::TradeTablePath;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, PackStack};
use bedrock_addon::storage::PackPath;

use super::pack_common::bp;

const WHEAT: &[u8] = br#"{"tiers": [{"trades": [{"wants": [{"item": "minecraft:wheat"}], "gives": [{"item": "minecraft:emerald"}]}]}]}"#;
const CARROT: &[u8] = br#"{"format_version": "1.21.30", "tiers": [{"total_exp_required": 0, "groups": [{"trades": [{"wants": [{"item": "minecraft:carrot"}], "gives": [{"item": "minecraft:emerald"}]}]}]}]}"#;
const BROKEN: &[u8] = br#"{"format_version": "1.21.30", "tiers": [{"groups": []}]}"#;

fn path(text: &'static str) -> PackPath {
    PackPath::from_static(text)
}

/// The first wanted item of the table at `at`.
fn wanted(stack: &PackStack<Behavior>, at: &'static str) -> Option<String> {
    let document = stack.trade_tables().get(&path(at))?;
    let item = &document.tiers[0].groups[0].trades[0].wants[0].items()[0];
    Some(item.item.as_str().to_owned())
}

#[test]
fn tables_are_keyed_by_their_path_below_trading() {
    let pack = bp(1)
        .file("trading/farmer_trades.json", WHEAT)
        .file("trading/economy_trades/farmer_trades.json", CARROT)
        .file("trading/notes.txt", b"not a table")
        .file("loot_tables/farmer_trades.json", WHEAT)
        .open();
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default()).unwrap();
    let tables = stack.trade_tables();
    let paths: Vec<&str> = tables.iter().map(|(path, _)| path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "trading/economy_trades/farmer_trades.json",
            "trading/farmer_trades.json"
        ]
    );
    assert_eq!(
        wanted(&stack, "trading/farmer_trades.json").as_deref(),
        Some("minecraft:wheat")
    );
    assert_eq!(
        wanted(&stack, "trading/economy_trades/farmer_trades.json").as_deref(),
        Some("minecraft:carrot")
    );
    assert!(tables.rejected().is_empty());
    assert_eq!(tables.diagnostics().count(), 0);
    // An entity's `table` is such a path.
    let table = TradeTablePath::new("trading/economy_trades/farmer_trades.json").unwrap();
    assert!(
        tables
            .get(&PackPath::new(table.as_str()).unwrap())
            .is_some()
    );
    // The document knows its pack and file.
    let document = tables.get(&path("trading/farmer_trades.json")).unwrap();
    assert_eq!(
        document.origin().path.as_str(),
        "trading/farmer_trades.json"
    );
}

#[test]
fn the_top_packs_copy_of_a_path_wins() {
    let top = bp(1).file("trading/farmer_trades.json", CARROT).open();
    let bottom = bp(2)
        .file("trading/farmer_trades.json", WHEAT)
        .file("trading/butcher_trades.json", WHEAT)
        .open();
    let options = LoadOptions {
        keep_shadowed: true,
        ..LoadOptions::default()
    };
    let stack = PackStack::<Behavior>::of([top, bottom], options).unwrap();
    assert_eq!(
        wanted(&stack, "trading/farmer_trades.json").as_deref(),
        Some("minecraft:carrot")
    );
    assert_eq!(
        wanted(&stack, "trading/butcher_trades.json").as_deref(),
        Some("minecraft:wheat")
    );
    let tables = stack.trade_tables();
    assert_eq!(tables.iter().count(), 2);
    // The lower copy is kept as a layer, not as the table.
    assert_eq!(
        tables.layers(&path("trading/farmer_trades.json")).count(),
        2
    );
    assert_eq!(
        tables.layers(&path("trading/butcher_trades.json")).count(),
        1
    );
}

#[test]
fn a_failing_top_copy_leaves_the_path_without_a_table() {
    let top = bp(1).file("trading/farmer_trades.json", BROKEN).open();
    let bottom = bp(2).file("trading/farmer_trades.json", WHEAT).open();
    let stack = PackStack::<Behavior>::of([top, bottom], LoadOptions::default()).unwrap();
    let tables = stack.trade_tables();
    // Vanilla looks the path up once: the lower pack's valid copy does not stand in.
    assert!(tables.get(&path("trading/farmer_trades.json")).is_none());
    assert_eq!(tables.rejected().len(), 1);
    let errors: Vec<_> = tables
        .diagnostics()
        .filter(|f| f.severity == Severity::Error)
        .collect();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code.id(), "core/missing_required_field");
    assert_eq!(
        errors[0].location.path.as_ref().map(PackPath::as_str),
        Some("trading/farmer_trades.json")
    );
    assert!(errors[0].location.pack.is_some());
}

#[test]
fn a_failing_lower_copy_does_not_disturb_the_top_one() {
    let top = bp(1).file("trading/farmer_trades.json", WHEAT).open();
    let bottom = bp(2).file("trading/farmer_trades.json", BROKEN).open();
    let stack = PackStack::<Behavior>::of([top, bottom], LoadOptions::default()).unwrap();
    assert_eq!(
        wanted(&stack, "trading/farmer_trades.json").as_deref(),
        Some("minecraft:wheat")
    );
    assert_eq!(stack.trade_tables().rejected().len(), 1);
}

#[test]
fn the_determinism_fragment_is_two_valid_tables() {
    let pack = bp(1)
        .file(
            "trading/demo_trades.json",
            include_bytes!(
                "../fixtures/trade/trading/determinism/behavior_pack/trading/demo_trades.json"
            ),
        )
        .file(
            "trading/economy_trades/demo_legacy_trades.json",
            include_bytes!(
                "../fixtures/trade/trading/determinism/behavior_pack/trading/economy_trades/demo_legacy_trades.json"
            ),
        )
        .open();
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default()).unwrap();
    let tables = stack.trade_tables();
    assert_eq!(tables.iter().count(), 2);
    assert!(tables.rejected().is_empty());
    // The legacy table's material function becomes filters, which the newest format has
    // under a deprecated name: announced (`round_trip.rs`, `k107_`), and nothing else is told.
    let told: Vec<String> = tables
        .diagnostics()
        .map(|f| {
            format!(
                "{} {:?} at {}",
                f.code.id(),
                f.severity,
                f.location.json_path
            )
        })
        .collect();
    assert_eq!(
        told,
        ["core/value_rewritten Info at tiers/0/trades/0/wants/0/functions"]
    );
}

#[test]
fn a_stack_without_trading_has_no_tables() {
    let pack = bp(1).open();
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default()).unwrap();
    assert_eq!(stack.trade_tables().iter().count(), 0);
}
