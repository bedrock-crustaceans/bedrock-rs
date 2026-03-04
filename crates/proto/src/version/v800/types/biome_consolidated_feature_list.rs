use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeConsolidatedFeatureData<V: ProtoVersion> {
    pub scatter: V::BiomeScatterParamData,
    #[endianness(le)]
    pub feature: u16,
    #[endianness(le)]
    pub identifier: u16,
    #[endianness(le)]
    pub pass: u16,
    pub internal_use: bool,
}

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeConsolidatedFeatureList<V: ProtoVersion> {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub entries: Vec<BiomeConsolidatedFeatureData<V>>,
}
