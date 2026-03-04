use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 52)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CraftingDataPacket<V: ProtoVersion> {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub crafting_entries: Vec<V::CraftingDataEntry>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub potion_mixes: Vec<V::PotionMixDataEntry>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub container_mixes: Vec<V::ContainerMixDataEntry>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub material_reducers: Vec<V::MaterialReducerDataEntry>,
    pub clear_recipes: bool,
}
