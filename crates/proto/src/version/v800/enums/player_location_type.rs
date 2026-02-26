use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
pub enum PlayerLocationType {
   Coordinates = 0,
   Hide = 1,
}