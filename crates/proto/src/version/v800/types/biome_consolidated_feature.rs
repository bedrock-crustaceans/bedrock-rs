use bedrockrs_macros::ProtoCodec;

use crate::v800::types::BiomeScatterParamData;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeConsolidatedFeatureData {
    pub scatter: BiomeScatterParamData,
    #[endianness(le)]
    pub feature: i32,
    #[endianness(le)]
    pub identifier: i32,
    #[endianness(le)]
    pub pass: i32,
    pub internal_use: bool,
}