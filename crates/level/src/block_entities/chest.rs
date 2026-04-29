#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename = "PascalCase")]
pub struct ItemBlock {}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename = "PascalCase")]
pub struct ItemStack {
    pub block: Option<ItemBlock>,
    pub can_destroy: Option<Vec<String>>,
    pub can_place_on: Option<Vec<String>>,
    pub count: i8,
    pub damage: i16,
    pub name: String,
    #[serde(rename = "tag")]
    pub tag: nbtx::Value,
    pub was_picked_up: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename = "PascalCase")]
pub struct ItemSlot {
    pub slot: i8,
    #[serde(flatten)]
    pub stack: ItemStack,
}

// TODO: TEST ITEMS AND PAIRING
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Chest {
    pub findable: i32,
    #[serde(rename = "forceunpair")]
    pub force_unpair: Option<bool>,
    pub loot_table: Option<String>,
    pub loot_table_seed: Option<i32>,
    #[serde(rename = "pairlead")]
    pub pair_lead: Option<i8>,
    #[serde(rename = "pairx")]
    pub pair_x: Option<i32>,
    #[serde(rename = "pairz")]
    pub pair_z: Option<i32>,
    pub items: Vec<ItemStack>,
}
