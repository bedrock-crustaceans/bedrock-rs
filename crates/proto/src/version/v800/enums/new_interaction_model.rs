use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum NewInteractionModel {
    Touch = 0,
    Crosshair = 1,
    Classic = 2,
    Count = 3,
}