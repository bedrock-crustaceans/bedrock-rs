use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum AuthoritativeMovementMode {
    Client = 0,
    Server = 1,
    ServerWithRewind = 2,
}