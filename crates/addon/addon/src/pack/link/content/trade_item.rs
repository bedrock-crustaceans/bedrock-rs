//! The items a trade table wants and gives.

use bedrock_addon_core::diagnostic::{JsonPath, Severity};

use crate::pack::link::prelude::*;
use crate::families::trade::{ItemList, TradeItem};

/// The `item` of every trade item (`name` or `name:aux`) is an item of the stack or
/// of the game. The table loads whatever the name; the trade of a missing item cannot be
/// made.
///
/// The finding is at the member the file has. A table of the older layout is upgraded as it
/// loads: its reader leaves out what the game drops (a trade without `wants` or `gives`, an
/// entry that is no object) and the upgrade rebuilds what the newest format writes otherwise
/// (a tier's own `trades` become a group, a group without a trade is not kept, an item with
/// the function `trader_material_type` becomes five); the document keeps where each member
/// it holds is in the file.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    for (_, document) in cx.behavior().trade_tables() {
        let missing = |out: &mut Findings, held: JsonPath, traded: &TradeItem| {
            let Target::Missing(hint) = cx.item(&traded.item) else {
                return;
            };
            let mut path = document.diagnostics().place_of(&held);
            path.push("item");
            // The finding's breadcrumb starts at the root (`tiers[0] -> groups[0] -> …`).
            // The linked-asset check's grading; the severity here is read by nothing else.
            let told = At::new(document.origin(), path, Severity::Error).validator(
                Validator::Missing,
                0,
                traded.item.as_str(),
            );
            out.push(unresolved(told, kinds::ITEM, traded.item.as_str(), hint));
        };
        for (tier_index, tier) in document.value().tiers.iter().enumerate() {
            for (group_index, group) in tier.groups.iter().enumerate() {
                for (trade_index, trade) in group.trades.iter().enumerate() {
                    let trade_at = at![
                        "tiers",
                        tier_index,
                        "groups",
                        group_index,
                        "trades",
                        trade_index
                    ];
                    for (key, lists) in [("wants", &trade.wants), ("gives", &trade.gives)] {
                        for (index, list) in lists.iter().enumerate() {
                            let entry = trade_at.join(&at![key, index]);
                            match list {
                                ItemList::Item(traded) => missing(out, entry, traded),
                                ItemList::Choice { choice } => {
                                    for (choice_index, traded) in choice.iter().enumerate() {
                                        let member = entry.join(&at!["choice", choice_index]);
                                        missing(out, member, traded);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
