use bedrock_level::serde_helpers::deserialize_bool;

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Beehive {
    /// Whether the beehive should spawn bees.
    #[serde(deserialize_with = "deserialize_bool")]
    pub should_spawn_bees: bool,
}
