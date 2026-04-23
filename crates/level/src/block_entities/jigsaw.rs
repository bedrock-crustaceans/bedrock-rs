#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JigsawJoint {
    Rollable,
    Aligned,
}

/// A jigsaw block tile entity.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Jigsaw {
    /// The block that this jigsaw block will become.
    pub final_state: String,
    /// The joint option. See [`JigsawJoint`].
    pub joint: JigsawJoint,
    /// The jigsaw block's name.
    pub name: String,
    /// The target name.
    pub target: String,
    /// The target pool to select a structure from.
    pub target_pool: String,
}
