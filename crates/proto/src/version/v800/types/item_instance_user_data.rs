use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemInstanceUserData {
    #[endianness(le)]
    pub serialization_marker: i16,
    pub serialization_version: i8,
    #[nbt]
    pub tags: nbtx::Value, // TODO: NBT Structure
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub can_place_on: Vec<String>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub can_break: Vec<String>,
}