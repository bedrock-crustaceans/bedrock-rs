use bedrockrs_macros::ProtoCodec;
use crate::v800::types::BiomeScatterParamData;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeConsolidatedFeatureData {
    pub scatter: BiomeScatterParamData,
    #[endianness(le)]
    pub feature: u16,
    #[endianness(le)]
    pub identifier: u16,
    #[endianness(le)]
    pub pass: u16,
    pub internal_use: bool,
}

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeConsolidatedFeatureList {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub entries: Vec<BiomeConsolidatedFeatureData>,
}
