use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct BiomeTagList {
    #[endianness(le)]
    pub tags: Vec<u16>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct BiomeDefinition<V: ProtoVersion> {
    #[endianness(le)]
    pub id: u16,
    #[endianness(le)]
    pub temperature: f32,
    #[endianness(le)]
    pub downfall: f32,
    #[endianness(le)]
    pub foliage_snow: f32,
    #[endianness(le)]
    pub depth: f32,
    #[endianness(le)]
    pub scale: f32,
    #[endianness(le)]
    pub map_water_color: i32,
    pub rain: bool,
    pub tags: Option<BiomeTagList>,
    pub chunk_gen_data: Option<V::BiomeDefinitionChunkGenData>,
}
