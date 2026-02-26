use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum AnimationMode {
    None = 0,
    Layers = 1,
    Blocks = 2,
}