use bedrock_protocol_version::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeSurfaceMaterialAdjustmentData<V: ProtoVersion> {
    pub biome_elements: Vec<V::BiomeElementData>,
}
