//! The rules of trade tables. The two rules of a group,
//! which both parsers apply where they read it (so a finding is at the member's path in the
//! file as written), and `F::validate`: the rules on the loaded table.

use super::texts::{NUM_TO_SELECT, WEIGHT_NO_EFFECT, child, element, finding, formatted};
use super::{ItemList, TradeTable};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, JsonPath, Phase, Severity, VanillaText,
};
use bedrock_addon_core::versioning::ValidateCx;

/// `num_to_select` above the group's number of trades: vanilla's legacy parser logs the text
/// and returns an empty table; its cereal slice has no such check. The crate reports the same
/// error at both versions, and the file is not loaded.
pub(crate) fn selects_too_many(selects: i32, trades: usize, at: &JsonPath) -> Option<Diagnostic> {
    let too_many = usize::try_from(selects).is_ok_and(|n| n > trades);
    too_many.then(|| {
        finding(
            DiagnosticCode::InvalidValue {
                text: Some(formatted(NUM_TO_SELECT, &[&selects, &trades])),
            },
            Severity::Error,
            Phase::Validate,
            at.clone(),
        )
    })
}

/// A `weight` (above 0: the callers reject or drop any other first) on a trade whose group
/// selects every trade (`num_to_select` -1): it has no effect, and the trade is kept. Vanilla
/// warns from its legacy parser only, at the content log's warning level; the crate reports it
/// at both versions, at `severity`: the legacy parser's warning, or an information from the
/// typed document.
pub(crate) fn weight_without_effect(
    selects: i32,
    at: &JsonPath,
    severity: Severity,
) -> Option<Diagnostic> {
    (selects == -1).then(|| {
        finding(
            DiagnosticCode::NoEffect {
                text: Some(VanillaText::Fixed(WEIGHT_NO_EFFECT)),
            },
            severity,
            Phase::Validate,
            at.clone(),
        )
    })
}

/// The family rules on a loaded table: an empty `tiers` (vanilla loads an empty table with no
/// message; the crate reports an error), and each typed filter's own findings.
pub(crate) fn validate(table: &mut TradeTable, cx: &mut ValidateCx<'_>) {
    // `tiers` is a root member in both forms of the document (the payload key of a checked
    // envelope, a member of the document the legacy parser reads whole).
    let tiers_at = child(&JsonPath::new(), "tiers");
    if table.tiers.is_empty() {
        cx.diagnostics.push(finding(
            DiagnosticCode::InvalidValue { text: None },
            Severity::Error,
            Phase::Validate,
            tiers_at.clone(),
        ));
    }
    // Typed filters exist only in a document written at 1.21.30 or later, whose paths are the
    // loaded table's.
    for (t, tier) in table.tiers.iter().enumerate() {
        let groups_at = child(&element(&tiers_at, t), "groups");
        for (g, group) in tier.groups.iter().enumerate() {
            let list_at = child(&element(&groups_at, g), "trades");
            for (i, trade) in group.trades.iter().enumerate() {
                let trade_at = element(&list_at, i);
                for (key, lists) in [("wants", &trade.wants), ("gives", &trade.gives)] {
                    let side_at = child(&trade_at, key);
                    for (e, list) in lists.iter().enumerate() {
                        check_filters(list, &element(&side_at, e), cx);
                    }
                }
            }
        }
    }
}

/// The findings of the typed filters of one entry, at `at`.
fn check_filters(list: &ItemList, at: &JsonPath, cx: &mut ValidateCx<'_>) {
    let mut check = |filters: Option<&bedrock_addon_core::shared::Filter>, item_at: &JsonPath| {
        if let Some(filters) = filters {
            for finding in filters.check(&child(item_at, "filters")) {
                cx.diagnostics.push(finding);
            }
        }
    };
    match list {
        ItemList::Choice { choice } => {
            let choice_at = child(at, "choice");
            for (c, item) in choice.iter().enumerate() {
                check(item.filters.as_ref(), &element(&choice_at, c));
            }
        }
        ItemList::Item(item) => check(item.filters.as_ref(), at),
    }
}
