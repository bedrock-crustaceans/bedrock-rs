use crate::block_entities::ItemBlock;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct FlowerPot {
    pub plant_block: Option<ItemBlock>,
}
