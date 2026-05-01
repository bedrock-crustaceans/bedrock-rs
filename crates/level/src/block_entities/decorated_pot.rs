use crate::types::ItemStack;

// Known sherds are: minecraft:brick, minecraft:guster_pottery_sherd

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct DecoratedPot {
    pub loot_table: Option<String>,
    pub loot_table_seed: Option<i32>,
    #[serde(rename = "animation")]
    pub animation: i8,
    #[serde(rename = "item")]
    pub item: ItemStack,
    #[serde(rename = "sherds")]
    pub sherds: Option<Vec<String>>,
}
