#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub enum JigsawJoint {
    Rollable,
    Aligned,
}

/// A jigsaw block tile entity.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct JigsawBlock {
    /// The target name.
    pub target: String,
    /// The block that this jigsaw block will become.
    pub final_state: String,
    pub placement_priority: i32,
    pub selection_priority: i32,
    /// The joint option. See [`JigsawJoint`].
    pub joint: JigsawJoint,
    /// The jigsaw block's name.
    pub name: String,
    /// The target pool to select a structure from.
    pub target_pool: String,
}
