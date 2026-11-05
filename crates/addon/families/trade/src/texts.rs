//! Vanilla's content-log texts of trade tables, and the findings built from them.

use bedrock_addon_core::diagnostic::{JsonPath, VanillaText};

/// The document root is JSON `null`.
pub(crate) const MUST_CONTAIN_TIERS: &str = "Trade table file must containt \"tiers\"";
/// `tiers` is missing or not an array.
pub(crate) const TIERS_ARRAY: &str = "\"tiers\" must be an array";
/// A tier's `groups` is present and not an array.
pub(crate) const GROUPS_ARRAY: &str = "\"groups\" must be an array";
/// A group's `trades` (or a tier's flat `trades`) is missing or not an array.
pub(crate) const TRADES_ARRAY: &str = "\"trades\" must be an array";
/// A group's `num_to_select` is above its number of trades.
pub(crate) const NUM_TO_SELECT: &str =
    "num_to_select[{}] must not be greater than the number of trades[{}]";
/// A trade without `wants`; `{}` is the trade as styled JSON text.
pub(crate) const NO_OFFERS: &str =
    "Trade table {} doesn't contain any offers, at least one is required";
/// A trade without `gives`.
pub(crate) const NO_RECEIVES: &str = "Trade {} doesn't have any recieves, at least one is required";
/// `weight` in a group that picks every trade.
pub(crate) const WEIGHT_NO_EFFECT: &str = "\"weight\" key used for a trade without using \"groups\" key or with \"num_to_select\" set to -1. Weight value will have no effect as every trade will be chosen.";
/// A `weight` of 0 or less.
pub(crate) const INVALID_WEIGHT: &str = "Invalid weight value: {}";

/// Every template with arguments and its number of arguments, for the arity test.
#[cfg(test)]
pub(crate) const FORMATTED: &[(&str, usize)] = &[
    (NUM_TO_SELECT, 2),
    (NO_OFFERS, 1),
    (NO_RECEIVES, 1),
    (INVALID_WEIGHT, 1),
];

/// `template` with `args` in its `{}` placeholders.
pub(crate) fn formatted(template: &'static str, args: &[&dyn std::fmt::Display]) -> VanillaText {
    VanillaText::Formatted {
        template,
        args: args.iter().map(ToString::to_string).collect(),
    }
}

/// `base` followed by `key`.
pub(crate) fn child(base: &JsonPath, key: &str) -> JsonPath {
    let mut path = base.clone();
    path.push(key);
    path
}

/// `base` followed by `index`.
pub(crate) fn element(base: &JsonPath, index: usize) -> JsonPath {
    let mut path = base.clone();
    path.push_index(index);
    path
}
