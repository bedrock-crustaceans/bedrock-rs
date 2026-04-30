use crate::deserialize_bool;

/// A command block tile entity.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CommandBlock {
    #[serde(deserialize_with = "deserialize_bool")]
    pub execute_on_first_tick: bool,
    #[serde(deserialize_with = "deserialize_bool")]
    #[serde(rename = "powered")]
    pub powered: bool,
    pub success_count: i32,
    #[serde(deserialize_with = "deserialize_bool")]
    // Yes this really is "Condional"
    #[serde(rename = "LPCondionalMode")]
    pub conditional_mode: bool,
    pub last_execution: i64,
    #[serde(rename = "LPCommandMode")]
    pub command_mode: i32,
    pub command: String,
    pub last_output_params: Vec<String>,
    #[serde(deserialize_with = "deserialize_bool")]
    #[serde(rename = "conditionMet")]
    pub condition_met: bool,
    #[serde(deserialize_with = "deserialize_bool")]
    pub track_output: bool,
    #[serde(deserialize_with = "deserialize_bool")]
    #[serde(rename = "auto")]
    pub auto: bool,
    pub last_output: String,
    pub version: i32,
    pub tick_delay: i32,
    pub custom_name: String,
    #[serde(deserialize_with = "deserialize_bool")]
    #[serde(rename = "LPRedstoneMode")]
    pub redstone_mode: bool,
}
