use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum StructureBlockType {
    Data = 0,
    Save = 1,
    Load = 2,
    Corner = 3,
    Invalid = 4,
    Export = 5,
    Count = 6,
}