use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum CodeBuilderCodeStatus {
    None = 0,
    NotStarted = 1,
    InProgress = 2,
    Paused = 3,
    Error = 4,
    Succeeded = 5,
}
