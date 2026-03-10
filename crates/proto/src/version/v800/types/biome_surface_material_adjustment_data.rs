use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeSurfaceMaterialAdjustmentData<V: ProtoVersion> {
    pub biome_elements: Vec<V::BiomeElementData>,
}
