#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
pub struct Skull {
    pub mouth_moving: bool,
    pub mouth_tick_count: i32,
    pub rotation: f32,
    pub skull_type: SkullType,
}
