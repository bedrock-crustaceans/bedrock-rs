use nbtx::Value;
use super::super::types::BiomeDefinition;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[derive(ProtoCodec, Clone, Debug)]
pub struct BiomeEntry {
    #[endianness(le)]
    pub name_index: u16,
    pub definition: BiomeDefinition,
}

#[gamepacket(id = 122)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BiomeDefinitionListPacket {
    #[nbt]
    pub biome_definitions: Value,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub biomes: Vec<BiomeEntry>,
}