use crate::block_entities::ItemStack;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct ItemFrame {
    pub item_rotation: f32,
    pub item_drop_chance: f32,
    pub item: ItemStack,
}
