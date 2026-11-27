//! Parsing a trade table from format 1.21.30 on: vanilla's strict cereal slice
//! (`v1_21_30`). The types derive the reader (`versioning::read_payload_with`), the quantity
//! included.

use super::validate as rules;
use super::{ItemList, Tier, Trade, TradeItem, TradeTable};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::json::kit::Profile;
use bedrock_addon_core::schema::{Schema, SchemaCx};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{ParseCx, Reading, read_payload_with};

/// The cereal slice's version (`v1_21_30::TradeTableData::VERSION`).
pub(crate) const CEREAL: FormatVersion = known::V1_21_30;

/// Parses the `tiers` payload of a document from 1.21.30 on: the derived read of the tiers
/// (vanilla's validation reports every unknown member, also beside another failure), then the
/// rules the types do not state.
pub(crate) fn parse_table(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<TradeTable> {
    let tiers: Vec<Tier> =
        read_payload_with(payload, cx, d, |read| read.profile(Profile::UNKNOWN_FIRST))?;
    let mut table = TradeTable { tiers };
    let at = cx.at.clone();
    table.check(&at, cx.original_version >= CEREAL, d);
    Some(table)
}

impl TradeTable {
    /// The rules of the cereal slice: the bounds the types state (a `weight` of at least 1), then
    /// per group and trade the rules no bound states. A finding that is an error rejects the
    /// table. `cereal`: the document was written at 1.21.30 or later, read by the cereal reader
    /// itself (not a legacy table the steps brought here).
    fn check(&mut self, at: &JsonPath, cereal: bool, d: &mut Diagnostics) {
        // The derived check names the `tiers` member itself: it starts at the table's own path.
        let mut table_at = at.clone();
        table_at.pop();
        bounds(self, &table_at, d);
        for (t, tier) in self.tiers.iter().enumerate() {
            for (g, group) in tier.groups.iter().enumerate() {
                let group_at = at.index(t).child("groups").index(g);
                let list_at = group_at.child("trades");
                // The schema default is 0.
                let selects = group.num_to_select.unwrap_or(0);
                // The sequence must hold at least one element.
                if group.trades.is_empty() {
                    d.push(
                        Diagnostic::error(Code::from(&core_codes::INVALID_VALUE))
                            .at(list_at.clone()),
                    );
                }
                if let Some(finding) = rules::selects_too_many(
                    selects,
                    group.trades.len(),
                    &group_at.child("num_to_select"),
                ) {
                    d.push(finding);
                }
                for (i, trade) in group.trades.iter().enumerate() {
                    check_trade(trade, &list_at.index(i), selects, cereal, d);
                }
            }
        }
    }
}

/// The bind constraints `value` states as its bounds (`Schema::check`), at `at`: each member
/// outside its bound is an `InvalidValue` error, which rejects the table.
fn bounds<T: Schema>(value: &mut T, at: &JsonPath, d: &mut Diagnostics) {
    value.check(&mut SchemaCx::new(Reading::DOCUMENT, at.clone(), d));
}

/// The rules of one trade at `at` in a group that selects `selects` trades.
fn check_trade(trade: &Trade, at: &JsonPath, selects: i32, cereal: bool, d: &mut Diagnostics) {
    // `wants` and `gives` must hold at least one element.
    for (entries, key) in [(&trade.wants, "wants"), (&trade.gives, "gives")] {
        if entries.is_empty() {
            d.push(Diagnostic::error(Code::from(&core_codes::INVALID_VALUE)).at(at.child(key)));
        }
    }
    // A `weight` of at least 1 (the bound's finding is told) in a group that picks every trade
    // has no effect; the cereal slice has no such check, so the crate reports an information.
    if let Some(weight) = trade.weight.filter(|weight| *weight >= 1) {
        let _ = weight;
        if let Some(finding) =
            rules::weight_without_effect(selects, &at.child("weight"), Severity::Info)
        {
            d.push(finding);
        }
    }
    for (key, lists) in [("wants", &trade.wants), ("gives", &trade.gives)] {
        for (e, list) in lists.iter().enumerate() {
            let entry_at = at.child(key).index(e);
            match list {
                ItemList::Choice { choice } => {
                    for (c, item) in choice.iter().enumerate() {
                        check_item(item, &entry_at.child("choice").index(c), cereal, d);
                    }
                }
                ItemList::Item(item) => check_item(item, &entry_at, cereal, d),
            }
        }
    }
}

/// The deprecated names an item holds. The hidden names are read and deprecated by name:
/// written in a 1.21.30+ document they are an error under both masks that rejects the table.
/// Moved here by the legacy steps they carry the legacy evaluation and are announced with the
/// same text as an information (the upgrade's move logs nothing).
fn check_item(item: &TradeItem, at: &JsonPath, cereal: bool, d: &mut Diagnostics) {
    for (written, member) in [
        (item.legacy_functions.is_some(), "__DEP_fnctns"),
        (item.legacy_filters.is_some(), "__DEP_fltrs"),
    ] {
        if written {
            let mut finding = Diagnostic::deprecated_member(member, at.child(member));
            if !cereal {
                finding.severity = Severity::Info;
            }
            d.push(finding);
        }
    }
}
