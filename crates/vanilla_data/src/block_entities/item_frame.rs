use bedrock_level::types::ItemStack;
use facet::Facet;

#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct ItemFrame {
    /// Written as a `Float` since 1.13; older records write it as a `Byte`.
    #[facet(nbtx::lenient_width(i8))]
    pub item_rotation: f32,
    pub item_drop_chance: f32,
    pub item: ItemStack,
}
