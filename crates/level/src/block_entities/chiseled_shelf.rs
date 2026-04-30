use crate::block_entities::ItemStack;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct ChiseledShelf {
    pub items: Option<Vec<ItemStack>>,
    pub last_interacted_slot: Option<i32>,
}
