use crate::v800::types::BiomeDefinitionChunkGenData;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct BiomeTagList {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    #[endianness(le)]
    pub tags: Vec<u16>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct BiomeDefinition {
    #[endianness(le)]
    pub id: Option<u16>,
    #[endianness(le)]
    pub temperature: f32,
    #[endianness(le)]
    pub downfall: f32,
    #[endianness(le)]
    pub red_spore_density: f32,
    #[endianness(le)]
    pub blue_spore_density: f32,
    #[endianness(le)]
    pub ash_density: f32,
    #[endianness(le)]
    pub white_ash_density: f32,
    #[endianness(le)]
    pub depth: f32,
    #[endianness(le)]
    pub scale: f32,
    #[endianness(le)]
    pub map_water_color: i32,
    pub rain: bool,
    pub tags: Option<BiomeTagList>,
    pub chunk_gen_data: Option<BiomeDefinitionChunkGenData>,
}
