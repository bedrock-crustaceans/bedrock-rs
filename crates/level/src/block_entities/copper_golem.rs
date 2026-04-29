#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CopperGolemActor {
    #[serde(rename = "ActorIdentifier")]
    pub identifier: String,
    #[serde(rename = "SaveData")]
    pub save_data: nbtx::Value, // TODO: entity data
}

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde_repr::Serialize_repr,
    serde_repr::Deserialize_repr,
)]
#[repr(i32)]
pub enum CopperGolemPose {
    Standing = 0,
    Sitting = 1,
    Running = 2,
    Star = 3,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CopperGolemStatue {
    pub actor: CopperGolemActor,
    pub pose: CopperGolemPose,
}
