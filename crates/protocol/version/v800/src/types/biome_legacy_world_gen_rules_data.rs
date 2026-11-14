use bedrock_protocol_version::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeLegacyWorldGenRulesData<V: ProtoVersion> {
    pub legacy_pre_hills: Vec<V::BiomeConditionalTransformationData>,
}
