use facet::Facet;


/// Stored as a `Byte` tag holding the numeric type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Facet)]
#[facet(nbtx::variant_as(i8))]
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

#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct Skull {
    pub doing_animation: bool,
    pub mouth_tick_count: i32,
    pub rotation: f32,
    pub skull_type: i8, // TODO: Use SkullType enum
}
