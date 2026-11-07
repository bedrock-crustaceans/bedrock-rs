//! The trade family's codes: vanilla's content-log lines of trade tables, byte for byte, each a
//! definition beside the reader or rule that raises it. Each id is `trade/<name>`, the name its
//! constant's in lower case.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// The document root is JSON `null`.
pub static MUST_CONTAIN_TIERS: CodeDef<0> = CodeDef::new(
    "trade/must_contain_tiers",
    Kind::Envelope,
    Class::Error,
    "the trade table has no tiers",
)
.game("Trade table file must containt \"tiers\"");

/// `tiers` is missing.
pub static TIERS_MISSING: CodeDef<0> = CodeDef::new(
    "trade/tiers_missing",
    Kind::Envelope,
    Class::Error,
    "no tiers array",
)
.game("\"tiers\" must be an array");

/// `tiers` is not an array.
pub static TIERS_ARRAY: CodeDef<0> = CodeDef::new(
    "trade/tiers_array",
    Kind::WrongType,
    Class::Error,
    "tiers is not an array",
)
.game("\"tiers\" must be an array");

/// A tier's `groups` is present and not an array.
pub static GROUPS_ARRAY: CodeDef<0> = CodeDef::new(
    "trade/groups_array",
    Kind::WrongType,
    Class::Error,
    "groups is not an array",
)
.game("\"groups\" must be an array");

/// A group's `trades` (or a tier's flat `trades`) is missing; the group is dropped and the
/// rest of the table loads: a kept error.
pub static TRADES_MISSING: CodeDef<0> = CodeDef::new(
    "trade/trades_missing",
    Kind::MissingRequired,
    Class::KeptError,
    "no trades array",
)
.game("\"trades\" must be an array");

/// A group's `trades` (or a tier's flat `trades`) is not an array; the group is dropped: a
/// kept error.
pub static TRADES_ARRAY: CodeDef<0> = CodeDef::new(
    "trade/trades_array",
    Kind::WrongType,
    Class::KeptError,
    "trades is not an array",
)
.game("\"trades\" must be an array");

/// A group's `num_to_select` is above its number of trades; the arguments are the two
/// numbers.
pub static NUM_TO_SELECT: CodeDef<2> = CodeDef::new(
    "trade/num_to_select",
    Kind::InvalidValue,
    Class::Error,
    "num_to_select {} above the {} trades",
)
.game("num_to_select[{}] must not be greater than the number of trades[{}]");

/// A trade without `wants`; the argument is the trade as styled JSON text. The trade is
/// dropped: a kept error.
pub static NO_OFFERS: CodeDef<1> = CodeDef::new(
    "trade/no_offers",
    Kind::InvalidValue,
    Class::KeptError,
    "the trade {} wants nothing",
)
.game("Trade table {} doesn't contain any offers, at least one is required");

/// A trade without `gives`; the argument is the trade as styled JSON text. The trade is
/// dropped: a kept error.
pub static NO_RECEIVES: CodeDef<1> = CodeDef::new(
    "trade/no_receives",
    Kind::InvalidValue,
    Class::KeptError,
    "the trade {} gives nothing",
)
.game("Trade {} doesn't have any recieves, at least one is required");

/// A `weight` of 0 or less; the argument is the weight. The trade is dropped: a kept error.
pub static INVALID_WEIGHT: CodeDef<1> = CodeDef::new(
    "trade/invalid_weight",
    Kind::InvalidValue,
    Class::KeptError,
    "weight {} is not above 0",
)
.game("Invalid weight value: {}");

/// `weight` in a group that picks every trade (a warning of the legacy parser; the typed
/// document tells it as an information).
pub static WEIGHT_NO_EFFECT: CodeDef<0> = CodeDef::new(
    "trade/weight_no_effect",
    Kind::NeverApplies,
    Class::Warning,
    "weight in a group that picks every trade",
)
.game("\"weight\" key used for a trade without using \"groups\" key or with \"num_to_select\" set to -1. Weight value will have no effect as every trade will be chosen.");

/// Every definition of this table, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    MUST_CONTAIN_TIERS.meta(),
    TIERS_MISSING.meta(),
    TIERS_ARRAY.meta(),
    GROUPS_ARRAY.meta(),
    TRADES_MISSING.meta(),
    TRADES_ARRAY.meta(),
    NUM_TO_SELECT.meta(),
    NO_OFFERS.meta(),
    NO_RECEIVES.meta(),
    INVALID_WEIGHT.meta(),
    WEIGHT_NO_EFFECT.meta(),
];
