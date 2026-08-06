use bedrock_level::types::ItemSlot;
use facet::Facet;

#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct Hopper {
    pub items: Vec<ItemSlot>,
    pub transfer_cooldown: i32,
}
