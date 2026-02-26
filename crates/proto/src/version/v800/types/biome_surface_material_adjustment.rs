use bedrockrs_macros::ProtoCodec;

use crate::v800::types::BiomeElementData;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeSurfaceMaterialAdjustmentData {
    #[endianness(le)]
    pub biome_elements: Vec<BiomeElementData>,
}