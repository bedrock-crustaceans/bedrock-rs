use crate::deserialize_bool;
use crate::subchunk::BlockDef;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct ItemStack {
    pub block: Option<BlockDef>,
    pub can_destroy: Option<Vec<String>>,
    pub can_place_on: Option<Vec<String>>,
    pub count: i8,
    pub damage: i16,
    pub name: String,
    #[serde(rename = "tag")]
    pub tag: Option<nbtx::Value>,
    #[serde(deserialize_with = "deserialize_bool")]
    pub was_picked_up: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct ItemSlot {
    pub slot: i8,
    #[serde(flatten)]
    pub stack: ItemStack,
}
