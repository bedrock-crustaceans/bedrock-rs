use crate::version::versions::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeSurfaceMaterialAdjustmentData<V: ProtoVersion> {
    pub biome_elements: Vec<V::BiomeElementData>,
}
