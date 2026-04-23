/// A command block tile entity.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandBlock {
    /// Whether the command block activates without a redstone signal.
    pub auto: bool,
    pub conditional_mode: bool,
    /// If a conditional command block had its condition met the last time it was activated.
    ///
    /// This is true if the command block is not conditional.
    pub condition_met: bool,
    #[serde(rename = "LPConditionalMode")]
    pub lp_conditional_mode: i8,
    #[serde(rename = "LPRedstoneMode")]
    pub lp_redstone_mode: i8,
    #[serde(rename = "LPCommandMode")]
    pub lp_command_mode: i8,
    /// Whether the command block is currently powered by redstone.
    pub powered: bool,
}
