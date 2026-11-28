use bedrock_level::types::ItemStack;
use facet::Facet;

// Known sherds are: minecraft:brick, minecraft:guster_pottery_sherd

#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(
    not(feature = "deny-unknown-fields"),
    facet(nbtx::allow_unknown_fields)
)]
pub struct DecoratedPot {
    pub loot_table: Option<String>,
    pub loot_table_seed: Option<i32>,
    #[facet(rename = "animation")]
    pub animation: i8,
    /// Absent on 16 of 61 real records — the same ones that also omit
    /// [`Self::loot_table`]/[`Self::loot_table_seed`], a pot whose contents
    /// come from a loot table rather than a fixed placed item.
    #[facet(rename = "item")]
    pub item: Option<ItemStack>,
    #[facet(rename = "sherds")]
    pub sherds: Option<Vec<String>>,
}
