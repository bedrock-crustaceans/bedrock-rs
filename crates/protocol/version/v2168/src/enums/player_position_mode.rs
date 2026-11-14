use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum PlayerPositionMode {
    Normal = 0,
    Respawn = 1,
    Teleport = 2,
    OnlyHeadRot = 3,
}
