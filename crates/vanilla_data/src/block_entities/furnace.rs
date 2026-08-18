use bedrock_level::types::ItemSlot;
use facet::Facet;

#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct Furnace {
    pub burn_duration: i16,
    pub burn_time: i16,
    pub cook_time: i16,
    pub items: Vec<ItemSlot>,
    /// Absent on every real `BlastFurnace` and `Smoker` record in the
    /// fixture (0/8 each) and present on every real `Furnace` record (8/8) —
    /// the same struct backs all three ids (see `mod.rs`), and only the
    /// plain furnace writes this key.
    #[facet(rename = "StoredXPInt")]
    pub stored_xp: Option<i32>,
}
