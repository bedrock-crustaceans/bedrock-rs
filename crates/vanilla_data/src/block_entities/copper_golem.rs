use facet::Facet;

#[derive(Debug, Clone, PartialEq, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct CopperGolemActor {
    #[facet(rename = "ActorIdentifier")]
    pub identifier: String,
    #[facet(rename = "SaveData")]
    pub save_data: nbtx::Value, // TODO: entity data
}

/// Stored as an `Int` tag holding the numeric pose.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Facet)]
#[facet(nbtx::variant_as(i32))]
#[repr(i32)]
pub enum CopperGolemPose {
    Standing = 0,
    Sitting = 1,
    Running = 2,
    Star = 3,
}

#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct CopperGolemStatue {
    pub actor: CopperGolemActor,
    pub pose: CopperGolemPose,
}
