use crate::block_entities::ItemStack;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Shelf {
    pub items: Option<Vec<ItemStack>>,
}
