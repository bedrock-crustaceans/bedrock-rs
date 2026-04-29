use crate::block_entities::ItemSlot;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Furnace {
    pub burn_duration: i16,
    pub burn_time: i16,
    pub cook_time: i16,
    pub items: Vec<ItemSlot>,
    #[serde(rename = "StoredXPInt")]
    pub stored_xp: i32,
}
