use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeLegacyWorldGenRulesData<V: ProtoVersion> {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub legacy_pre_hills: Vec<V::BiomeConditionalTransformationData>,
}
