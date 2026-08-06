use bedrock_level::types::ItemStack;
use facet::Facet;

#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct BrewingStand {
    pub cook_time: i16,
    pub fuel_amount: i16,
    pub fuel_total: i16,
    /// The items currently contained in the brewing stand.
    pub items: Vec<ItemStack>,
}
