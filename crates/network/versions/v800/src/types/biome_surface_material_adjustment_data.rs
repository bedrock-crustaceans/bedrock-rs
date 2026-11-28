use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeSurfaceMaterialAdjustmentData<V: ProtoVersion> {
    pub biome_elements: Vec<V::BiomeElementData>,
}
