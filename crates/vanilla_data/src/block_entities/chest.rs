use facet::Facet;

pub use bedrock_level::types::ItemStack;

// TODO: TEST ITEMS AND PAIRING
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct Chest {
    /// A real chest record has been observed writing this as a `Byte` rather
    /// than the `Int` every other field's width would suggest.
    ///
    /// `lenient_width` is decode-only: a `Byte`-tagged record read and
    /// written back re-encodes as the declared `Int`, widening on write.
    #[facet(nbtx::lenient_width(i8))]
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
