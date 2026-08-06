use facet::Facet;

/// Stored as a `String` tag holding the joint name.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Facet)]
#[facet(nbtx::variant_as(str), rename_all = "snake_case")]
#[repr(u8)]
pub enum JigsawJoint {
    Rollable,
    Aligned,
}

/// A jigsaw block tile entity.
#[derive(Debug, Clone, PartialEq, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
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
