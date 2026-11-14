use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum NewInteractionModel {
    Touch = 0,
    Crosshair = 1,
    Classic = 2,
    Count = 3,
}
