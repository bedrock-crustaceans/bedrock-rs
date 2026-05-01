#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct BrushableBlock {
    pub brush_count: i32,
    pub brush_direction: i8,
    #[serde(rename = "LootTableSeed")]
    pub loot_table_seed: i32,
    #[serde(rename = "LootTable")]
    pub loot_table: String,
    /// Types are (presumably) `minecraft:suspicious_gravel` and `minecraft:suspicious_sand`.
    #[serde(rename = "type")]
    pub ty: String,
}
