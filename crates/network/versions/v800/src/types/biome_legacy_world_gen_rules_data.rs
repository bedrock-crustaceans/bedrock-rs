use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeLegacyWorldGenRulesData<V: ProtoVersion> {
    pub legacy_pre_hills: Vec<V::BiomeConditionalTransformationData>,
}
