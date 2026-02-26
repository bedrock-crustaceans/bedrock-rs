use bedrockrs_macros::ProtoCodec;

use crate::v800::types::BiomeSurfaceMaterialData;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeElementData {
    #[endianness(le)]
    pub noise_frequency_scale: f32,
    #[endianness(le)]
    pub noise_lower_bound: f32,
    #[endianness(le)]
    pub noise_upper_bound: f32,
    #[endianness(le)]
    pub height_min_type: i32,
    #[endianness(le)]
    pub height_min: i32,
    #[endianness(le)]
    pub height_max_type: i32,
    #[endianness(le)]
    pub height_max: i32,
    #[endianness(le)]
    pub adjusted_materials: BiomeSurfaceMaterialData,
}