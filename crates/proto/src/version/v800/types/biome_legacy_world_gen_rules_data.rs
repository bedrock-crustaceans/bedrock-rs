use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeLegacyWorldGenRulesData<V: ProtoVersion> {
    pub legacy_pre_hills: Vec<V::BiomeConditionalTransformationData>,
}
