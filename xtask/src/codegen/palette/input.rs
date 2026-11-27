//! The published dump's own JSON shape (`metadata/vanilladata_modules/mojang-blocks.json`
//! in a `bedrock-samples` release), transcribed field for field.
//!
//! Every struct here derives `deny_unknown_fields`: a field the dump carries that none of
//! these structs name is exactly the "input the generator does not understand" case the
//! schema-generator task requires failing loudly on, rather than silently dropping it and
//! emitting a table with a hole in it.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawDump {
    pub block_properties: Vec<RawBlockProperty>,
    pub data_items: Vec<RawDataItem>,
    /// Present and a real semver-ish string on early dumps, `null` on later ones -- not a
    /// dependable version source either way, since the file does not know which release
    /// tag it was read from. The generator takes the version from the pinned manifest
    /// entry instead; see `manifest.rs`.
    #[serde(default)]
    pub minecraft_version: Option<String>,
    /// `"block"` on early dumps, `"vanilla_data"` on later ones. Recorded so a
    /// genuinely new value here is visible in a diff rather than silently ignored, but
    /// not otherwise interpreted.
    pub module_type: String,
    pub name: String,
    /// Absent on early dumps, `"block"` (or `null`) on later ones. Same treatment as
    /// `module_type`.
    #[serde(default)]
    pub vanilla_data_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBlockProperty {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub values: Vec<RawPropertyValue>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPropertyValue {
    pub value: RawValue,
}

/// A property value as the dump writes it -- untagged because the JSON itself carries no
/// type tag beyond the value's own JSON type, which is exactly what
/// [`RawBlockProperty::ty`] is checked against during validation (`model.rs`).
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum RawValue {
    Bool(bool),
    Int(i64),
    Str(String),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawDataItem {
    pub name: String,
    pub properties: Vec<RawPropertyRef>,
    /// Present on later dumps (a numeric palette id), absent on early ones. Not part of
    /// the generated `BlockSpec` yet -- nothing in this crate's first-deliverable scope
    /// consumes it -- but modeled so its presence or absence does not trip
    /// `deny_unknown_fields`.
    #[serde(default)]
    pub raw_id: Option<u32>,
    /// Present on later dumps (an internal serialization tag, e.g. `"tile.acacia_button"`),
    /// absent on early ones. Same treatment as `raw_id`.
    #[serde(default)]
    pub serialization_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPropertyRef {
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A field none of these structs name is exactly the "unrecognized input" case
    /// `deny_unknown_fields` exists to catch -- this pins that a genuinely new top-level
    /// field on a future dump fails parsing loudly rather than being silently dropped.
    #[test]
    fn an_unknown_top_level_field_is_rejected() {
        let json = r#"{
            "block_properties": [],
            "data_items": [],
            "module_type": "vanilla_data",
            "name": "mojang-block",
            "a_field_this_generator_has_never_seen": true
        }"#;
        let err = serde_json::from_str::<RawDump>(json).unwrap_err();
        assert!(
            err.to_string()
                .contains("a_field_this_generator_has_never_seen"),
            "expected the error to name the unknown field, got: {err}"
        );
    }

    #[test]
    fn a_known_top_level_shape_parses() {
        let json = r#"{
            "block_properties": [],
            "data_items": [],
            "module_type": "vanilla_data",
            "name": "mojang-block"
        }"#;
        let raw: RawDump = serde_json::from_str(json).unwrap();
        assert_eq!(raw.module_type, "vanilla_data");
        assert!(raw.minecraft_version.is_none());
        assert!(raw.vanilla_data_type.is_none());
    }
}
