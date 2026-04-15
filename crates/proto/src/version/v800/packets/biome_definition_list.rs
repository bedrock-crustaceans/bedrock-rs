use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 122)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BiomeDefinitionListPacket<V: ProtoVersion> {
    pub biomes: Vec<BiomeEntry<V>>,
    pub strings: Vec<String>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct BiomeEntry<V: ProtoVersion> {
    #[endianness(le)]
    pub name_index: u16,
    pub definition: V::BiomeDefinition,
}
