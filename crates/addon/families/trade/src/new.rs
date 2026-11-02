//! Constructors: a trade item from its identifier.

use super::TradeItem;
use bedrock_addon_core::id::ItemId;

impl TradeItem {
    /// One of `item`, every other member absent.
    #[must_use]
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
