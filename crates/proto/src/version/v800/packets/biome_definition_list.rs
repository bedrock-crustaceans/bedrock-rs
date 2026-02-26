use super::super::types::{BiomeDefinition};
use bedrockrs_macros::{gamepacket, ProtoCodec};
use nbtx::Value;

#[derive(ProtoCodec, Clone, Debug)]
pub struct BiomeEntry {
    pub name: String,
    pub definition: BiomeDefinition,
}

#[gamepacket(id = 122)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BiomeDefinitionListPacket {
    #[nbt]
    pub biome_definitions: Value,

    #[vec_repr(i32)]
    #[vec_endianness(var)]
    pub biomes: Vec<BiomeEntry>,
}