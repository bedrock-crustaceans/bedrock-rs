use std::option::Option;
use crate::v800::types::{BiomeDefinitionChunkGenData, Color};
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct BiomeDefinition {
    #[endianness(le)]
    pub id: Option<i32>,
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
    pub map_water_color: Color,
    pub rain: bool,
    #[endianness(le)]
    pub tags: Option<Vec<i32>>,
    #[endianness(le)]
    pub chunk_gen_data: Option<BiomeDefinitionChunkGenData>
}