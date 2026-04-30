pub use crate::types::ItemStack;

// TODO: TEST ITEMS AND PAIRING
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
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
