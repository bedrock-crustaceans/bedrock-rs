use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum ItemVersion {
    Legacy = 0, 
    DataDriven = 1,
    None = 2,
}