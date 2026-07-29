use bedrock_level::types::ItemStack;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Cauldron {
    pub potion_type: i16,
    pub potion_id: i16,
    pub items: Vec<ItemStack>,
}
