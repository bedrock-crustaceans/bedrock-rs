use facet::Facet;


/// A command block tile entity.
#[derive(Debug, Clone, PartialEq, Eq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct CommandBlock {
    pub execute_on_first_tick: bool,
    #[facet(rename = "powered")]
    pub powered: bool,
    pub success_count: i32,
    // Yes this really is "Condional"
    #[facet(rename = "LPCondionalMode")]
    pub conditional_mode: bool,
    pub last_execution: i64,
    #[facet(rename = "LPCommandMode")]
    pub command_mode: i32,
    pub command: String,
    pub last_output_params: Vec<String>,
    #[facet(rename = "conditionMet")]
    pub condition_met: bool,
    pub track_output: bool,
    #[facet(rename = "auto")]
    pub auto: bool,
    pub last_output: String,
    pub version: i32,
    pub tick_delay: i32,
    pub custom_name: String,
    #[facet(rename = "LPRedstoneMode")]
    pub redstone_mode: bool,
}
