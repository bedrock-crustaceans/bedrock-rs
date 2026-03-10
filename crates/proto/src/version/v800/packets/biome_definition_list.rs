use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};
use nbtx::Value;

#[gamepacket(id = 122)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BiomeDefinitionListPacket<V: ProtoVersion> {
    #[nbt]
    pub biome_definitions: Value,

    pub biomes: Vec<BiomeEntry<V>>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct BiomeEntry<V: ProtoVersion> {
    #[endianness(le)]
    pub name_index: u16,
    pub definition: V::BiomeDefinition,
}
