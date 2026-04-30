use crate::types::ItemSlot;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Hopper {
    pub items: Vec<ItemSlot>,
    pub transfer_cooldown: i32,
}
