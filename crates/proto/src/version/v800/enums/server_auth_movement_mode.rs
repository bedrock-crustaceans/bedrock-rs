use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum ServerAuthMovementMode {
    ClientAuthoritative = 0,
    ServerAuthoritative = 1,
    ServerAuthoritativeWithRewind = 2,
}