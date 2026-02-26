use super::super::types::BlockPos;
use bedrockrs_macros::ProtoCodec;
use vek::Vec3;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum DataItemType {
    Byte(i8) = 0,
    Short(#[endianness(le)] i16) = 1,
    Int(#[endianness(var)] i32) = 2,
    Float(#[endianness(le)] f32) = 3,
    String(String) = 4,
    NBT(#[nbt] nbtx::Value) = 5,
    Pos(BlockPos) = 6,
    Int64(#[endianness(var)] i64) = 7,
    Vec3(#[endianness(le)] Vec3<f32>) = 8,
}