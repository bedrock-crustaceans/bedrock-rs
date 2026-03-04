use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeElementData<V: ProtoVersion> {
    #[endianness(le)]
    pub noise_frequency_scale: f32,
    #[endianness(le)]
    pub noise_lower_bound: f32,
    #[endianness(le)]
    pub noise_upper_bound: f32,
    #[endianness(var)]
    pub height_min_type: i32,
    #[endianness(le)]
    pub height_min: u16,
    #[endianness(var)]
    pub height_max_type: i32,
    #[endianness(le)]
    pub height_max: u16,
    pub adjusted_materials: V::BiomeSurfaceMaterialData,
}
