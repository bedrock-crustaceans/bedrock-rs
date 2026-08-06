use facet::Facet;

#[derive(Debug, Clone, PartialEq, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct BrushableBlock {
    pub brush_count: i32,
    pub brush_direction: i8,
    #[facet(rename = "LootTableSeed")]
    pub loot_table_seed: i32,
    #[facet(rename = "LootTable")]
    pub loot_table: String,
    /// Types are (presumably) `minecraft:suspicious_gravel` and `minecraft:suspicious_sand`.
    #[facet(rename = "type")]
    pub ty: String,
}
