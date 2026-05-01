use crate::types::ItemSlot;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Dispenser {
    pub loot_table: String,
    pub items: Vec<ItemSlot>,
    pub loot_table_seed: i32,
}
