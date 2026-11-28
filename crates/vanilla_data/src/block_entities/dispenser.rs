use bedrock_level::types::ItemSlot;
use facet::Facet;

#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(
    not(feature = "deny-unknown-fields"),
    facet(nbtx::allow_unknown_fields)
)]
pub struct Dispenser {
    /// Absent together with [`Self::loot_table_seed`] on every real record
    /// with no loot table assigned (24 records across `Dispenser`/`Dropper`
    /// in the fixture) — the same pairing `Chest::loot_table` already models
    /// as optional.
    pub loot_table: Option<String>,
    pub items: Vec<ItemSlot>,
    pub loot_table_seed: Option<i32>,
}
