//! The published biome dump's own JSON shape
//! (`metadata/vanilladata_modules/mojang-biomes.json` in a `bedrock-samples` release),
//! transcribed field for field -- same `deny_unknown_fields` discipline as
//! `codegen::palette::input`, for the same reason: a field this generator does not
//! recognize must fail parsing loudly, not vanish silently.
//!
//! Only the three pinned tags whose dump carries a numeric `id` are vendored (see
//! `xtask/dumps/bedrock-samples/COVERAGE.md`'s biome section), so unlike the block dump's
//! `raw_id`/`serialization_id` (present on some tags, absent on others, modelled as
//! optional), `id` here is required -- every vendored biome dump has one on every entry.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBiomeDump {
    pub data_items: Vec<RawBiomeItem>,
    /// `"vanilla_data"` on every vendored biome dump. Recorded so a genuinely new value
    /// is visible in a diff rather than silently ignored, not otherwise interpreted --
    /// same treatment as `RawDump::module_type` in the block-dump parser.
    pub module_type: String,
    pub name: String,
    #[serde(default)]
    pub vanilla_data_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBiomeItem {
    pub id: i32,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unknown_top_level_field_is_rejected() {
        let json = r#"{
            "data_items": [],
            "module_type": "vanilla_data",
            "name": "mojang-biome",
            "a_field_this_generator_has_never_seen": true
        }"#;
        let err = serde_json::from_str::<RawBiomeDump>(json).unwrap_err();
        assert!(
            err.to_string()
                .contains("a_field_this_generator_has_never_seen"),
            "expected the error to name the unknown field, got: {err}"
        );
    }

    #[test]
    fn a_known_shape_parses() {
        let json = r#"{
            "data_items": [{"id": 1, "name": "minecraft:plains"}],
            "module_type": "vanilla_data",
            "name": "mojang-biome",
            "vanilla_data_type": "biome"
        }"#;
        let raw: RawBiomeDump = serde_json::from_str(json).unwrap();
        assert_eq!(raw.data_items.len(), 1);
        assert_eq!(raw.data_items[0].id, 1);
    }
}
