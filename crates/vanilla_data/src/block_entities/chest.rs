use facet::Facet;

pub use bedrock_level::types::ItemStack;

// TODO: TEST ITEMS AND PAIRING
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct Chest {
    pub findable: i32,
    #[facet(rename = "forceunpair")]
    pub force_unpair: Option<bool>,
    pub loot_table: Option<String>,
    pub loot_table_seed: Option<i32>,
    #[facet(rename = "pairlead")]
    pub pair_lead: Option<i8>,
    #[facet(rename = "pairx")]
    pub pair_x: Option<i32>,
    #[facet(rename = "pairz")]
    pub pair_z: Option<i32>,
    pub items: Vec<ItemStack>,
}
