use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum PlayerPositionMode {
    Normal = 0,
    Respawn = 1,
    Teleport {
        #[endianness(le)]
        teleportation_cause: i32,
        #[endianness(le)]
        source_actor_type: i32,
    } = 2,
    OnlyHeadRot = 3,
}
