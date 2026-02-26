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
    // Encoded as VarInt where -1 means "no expression op".
    #[endianness(var)]
    pub height_min_type: i32,
    #[endianness(le)]
    pub height_min: u16,
    // Encoded as VarInt where -1 means "no expression op".
    #[endianness(var)]
    pub height_max_type: i32,
    #[endianness(le)]
    pub height_max: u16,
    pub adjusted_materials: BiomeSurfaceMaterialData,
}
