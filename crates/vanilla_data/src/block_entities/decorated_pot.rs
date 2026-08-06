use bedrock_level::types::ItemStack;
use facet::Facet;

// Known sherds are: minecraft:brick, minecraft:guster_pottery_sherd

#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct DecoratedPot {
    pub loot_table: Option<String>,
    pub loot_table_seed: Option<i32>,
    #[facet(rename = "animation")]
    pub animation: i8,
    #[facet(rename = "item")]
    pub item: ItemStack,
    #[facet(rename = "sherds")]
    pub sherds: Option<Vec<String>>,
}
