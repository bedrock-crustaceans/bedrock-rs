//! The trade-table types, their inherent items, and the constructor of a trade item from its
//! identifier.

use bedrock_addon_core::id::ItemId;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::shared::Filter;

/// A villager trade table (`trading/**/*.json`), at the newest format version (1.21.30).
///
/// JSON: `{"format_version": "1.21.30", "tiers": [tier, …]}`. A trader unlocks the tiers in
/// order as its experience reaches each tier's `total_exp_required`; each tier offers trades
/// picked from its groups. Tables are looked up by path, so the top pack's file at a path wins.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet, Schema)]
#[doc(alias = "trade_table")]
pub struct TradeTable {
    /// The tiers, lowest first (JSON `tiers`). Vanilla accepts an empty list; the crate reports it.
    pub tiers: Vec<Tier>,
}

/// One tier of a trade table.
///
/// JSON: `{"total_exp_required": n, "groups": [group, …]}`, both required.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet, Schema)]
#[doc(alias = "TradeTier")]
pub struct Tier {
    /// The groups whose trades the tier offers (JSON `groups`).
    pub groups: Vec<Group>,
    /// The trader experience at which the tier unlocks (JSON `total_exp_required`).
    pub total_exp_required: u32,
}

/// A group of trades: the tier offers `num_to_select` of them, picked by
/// weight, or every one when `num_to_select` is -1.
///
/// JSON: `{"num_to_select"?: n, "trades": [trade, …]}`; `trades` is required and not empty.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet, Schema)]
#[doc(alias = "TradeGroup")]
pub struct Group {
    /// How many trades are picked; -1 picks every one (JSON `num_to_select`, default 0, see
    /// [`Group::num_to_select`]). It must not exceed the number of trades.
    #[facet(schema::fallback = 0)]
    pub num_to_select: Option<i32>,
    /// The trades (JSON `trades`).
    pub trades: Vec<Trade>,
}

/// One trade: what the trader wants and what it gives in return.
///
/// JSON: `{"wants": [list, …], "gives": [list, …], "trader_exp"?, "max_uses"?, "reward_exp"?,
/// "weight"?}`; `wants` and `gives` are required and not empty. An absent member reads as its
/// default through the accessor of the same name and is not written back.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet, Schema)]
pub struct Trade {
    /// The items the player pays, each a list one item is picked from (JSON `wants`).
    pub wants: Vec<ItemList>,
    /// The items the player receives (JSON `gives`).
    pub gives: Vec<ItemList>,
    /// The experience the trader gains from the trade (JSON `trader_exp`, default 1).
    #[facet(schema::fallback = 1)]
    pub trader_exp: Option<u32>,
    /// How often the trade can be used before it locks (JSON `max_uses`, default 7).
    #[facet(schema::fallback = 7)]
    pub max_uses: Option<i32>,
    /// Whether the player gains experience from the trade (JSON `reward_exp`, default `true`).
    #[facet(schema::fallback = true)]
    pub reward_exp: Option<bool>,
    /// The trade's weight when a group picks `num_to_select` trades (JSON `weight`, default 1,
    /// at least 1). It has no effect in a group that picks every trade.
    #[facet(schema::fallback = 1)]
    pub weight: Option<i32>,
}

/// One `wants` / `gives` entry: one item, or a list one item is picked
/// from at random when the trade is generated.
///
/// JSON: an item object, or `{"choice": [item, …]}`. The form is kept for writing.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(untagged)]
#[repr(u8)]
#[allow(
    clippy::large_enum_variant,
    reason = "the bare item is the common form; an entry is built once and stored"
)]
#[doc(alias = "TradeItemList")]
pub enum ItemList {
    /// `{"choice": [item, …]}`: one of the items, picked at random. Vanilla accepts an empty
    /// list.
    Choice {
        /// The items (JSON `choice`).
        choice: Vec<TradeItem>,
    },
    /// A bare item object: a list of one.
    Item(TradeItem),
}

/// One item of a trade.
///
/// JSON: `{"item": "<id>", "quantity"?, "functions"?, "filters"?, "price_multiplier"?}`. The
/// identifier may carry a legacy `:aux` suffix (`minecraft:wool:14`). An absent member reads as
/// its default and is not written back.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct TradeItem {
    /// How many of the item (JSON `quantity`, default 1, see [`TradeItem::quantity_range`]).
    pub quantity: Option<Quantity>,
    /// The item (JSON `item`, required).
    pub item: ItemId,
    /// Loot-table item functions applied to the item, kept as written (JSON `functions`).
    pub functions: Option<Vec<Value>>,
    /// The conditions under which the item is offered (JSON `filters`; default: always).
    pub filters: Option<Filter>,
    /// How much the price follows demand (JSON `price_multiplier`, default 0.0).
    #[facet(schema::fallback = 0.0)]
    pub price_multiplier: Option<f32>,
    /// The functions of a document upgraded from before 1.21.30, which keep their legacy
    /// evaluation (JSON `__DEP_fnctns`, a deprecated name: written by an author it rejects the
    /// table; a table holding one is written back as a 1.20.30 legacy document).
    #[facet(rename = "__DEP_fnctns")]
    pub legacy_functions: Option<Value>,
    /// The filters of a document upgraded from before 1.21.30, which keep their legacy
    /// evaluation (JSON `__DEP_fltrs`, a deprecated name: written by an author it rejects the
    /// table; a table holding one is written back as a 1.20.30 legacy document).
    #[facet(rename = "__DEP_fltrs")]
    pub legacy_filters: Option<Value>,
}

/// How many of an item a trade takes or gives.
///
/// JSON: a whole number (`3`: exactly three) or `{"min"?: n, "max"?: n}`, each bound at least 1
/// and 1 when absent (`{"max": 2}` is one or two). The form is kept for writing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, facet::Facet)]
#[facet(untagged)]
#[repr(u8)]
#[doc(alias = "Trade Quantity")]
pub enum Quantity {
    /// A whole number: exactly that many.
    Exact(u32),
    /// `{"min", "max"}`: between the two, inclusive.
    // The writer writes a `None` field of an untagged enum's struct variant as `null`, which
    // a struct leaves out (pinned in core's `json_write`,
    // `an_untagged_struct_variant_writes_a_none_field_as_null`): the two skips keep an absent
    // bound absent.
    Range {
        /// The least (JSON `min`, default 1).
        #[facet(skip_serializing_if = Option::is_none)]
        min: Option<u32>,
        /// The most (JSON `max`, default 1).
        #[facet(skip_serializing_if = Option::is_none)]
        max: Option<u32>,
    },
}

// The two forms of an entry and of a quantity: leaves of the derived walks (an untagged enum
// holds no derived bound, trigger or Molang member).
bedrock_addon_core::impl_schema_leaf!(ItemList, Quantity);
bedrock_addon_core::impl_no_molang!(ItemList, Quantity);

impl ItemList {
    /// The items of the entry: the `choice` list, or the one bare item.
    pub fn items(&self) -> &[TradeItem] {
        match self {
            Self::Choice { choice } => choice,
            Self::Item(item) => std::slice::from_ref(item),
        }
    }
}

impl TradeItem {
    /// `(min, max)` of the quantity: 1..1 when absent, `n..n` for a whole number, each absent
    /// bound of the object form 1.
    pub fn quantity_range(&self) -> (u32, u32) {
        self.quantity.map_or((1, 1), Quantity::bounds)
    }
}

impl Quantity {
    /// `(min, max)`: `n..n` for a whole number, each absent bound 1.
    pub fn bounds(self) -> (u32, u32) {
        match self {
            Self::Exact(n) => (n, n),
            Self::Range { min, max } => (min.unwrap_or(1), max.unwrap_or(1)),
        }
    }
}

impl TradeItem {
    /// One of `item`, every other member absent.
    pub fn new(item: ItemId) -> Self {
        Self {
            quantity: None,
            item,
            functions: None,
            filters: None,
            price_multiplier: None,
            legacy_functions: None,
            legacy_filters: None,
        }
    }
}
