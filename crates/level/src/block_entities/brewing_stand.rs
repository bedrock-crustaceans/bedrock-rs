use crate::block_entities::ItemStack;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct BrewingStand {
    pub cook_time: i16,
    pub fuel_amount: i16,
    pub fuel_total: i16,
    pub items: Vec<ItemStack>,
}
