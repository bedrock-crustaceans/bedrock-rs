use crate::block_entities::ItemBlock;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FlowerPot {
    pub plant_block: Option<ItemBlock>,
}
