use bedrockrs_macros::ProtoCodec;

/// UNUSED
#[derive(ProtoCodec, Clone, Debug)]
#[enum_endianness(var)]
#[enum_repr(u32)]
#[repr(u32)]
pub enum UIProfile {
    Classic = 0,
    Pocket = 1,
    None = 2,
    Count = 3,
}
