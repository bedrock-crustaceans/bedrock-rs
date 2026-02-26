use bedrockrs_macros::ProtoCodec;
use crate::v800::types::BiomeElementData;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeSurfaceMaterialAdjustmentData {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub biome_elements: Vec<BiomeElementData>,
}
