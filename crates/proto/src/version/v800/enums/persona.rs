use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(le)]
#[repr(u32)]
pub enum AnimatedTextureType {
    None = 0,
    Face = 1,
    Body32x32 = 2,
    Body128x128 = 3,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(le)]
#[repr(u32)]
pub enum AnimationExpression {
    Linear = 0,
    Blinking = 1,
}