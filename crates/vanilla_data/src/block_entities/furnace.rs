use bedrock_level::types::ItemSlot;
use facet::Facet;

#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct Furnace {
    pub burn_duration: i16,
    pub burn_time: i16,
    pub cook_time: i16,
    pub items: Vec<ItemSlot>,
    #[facet(rename = "StoredXPInt")]
    pub stored_xp: i32,
}
