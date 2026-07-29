use bedrock_level::serde_helpers::deserialize_bool;

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde_repr::Serialize_repr,
    serde_repr::Deserialize_repr,
)]
#[repr(i8)]
pub enum SkullType {
    Skeleton,
    WitherSkeleton,
    Zombie,
    Player,
    Creeper,
    EnderDragon,
    Piglin,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Skull {
    #[serde(deserialize_with = "deserialize_bool")]
    pub doing_animation: bool,
    pub mouth_tick_count: i32,
    pub rotation: f32,
    pub skull_type: i8, // TODO: Use SkullType enum
}
