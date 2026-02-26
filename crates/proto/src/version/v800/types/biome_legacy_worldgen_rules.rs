use bedrockrs_macros::ProtoCodec;

use crate::v800::types::BiomeConditionalTransformationData;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeLegacyWorldGenRulesData {
    #[endianness(le)]
    pub legacy_pre_hills: Vec<BiomeConditionalTransformationData>,
}