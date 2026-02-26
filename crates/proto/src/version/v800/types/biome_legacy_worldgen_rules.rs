use bedrockrs_macros::ProtoCodec;
use crate::v800::types::BiomeConditionalTransformationData;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeLegacyWorldGenRulesData {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub legacy_pre_hills: Vec<BiomeConditionalTransformationData>,
}
