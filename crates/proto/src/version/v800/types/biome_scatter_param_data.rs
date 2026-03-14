use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeScatterParamData<V: ProtoVersion> {
    pub coordinates: Vec<V::BiomeCoordinateData>,
    #[endianness(var)]
    pub eval_order: i32,
    #[endianness(var)]
    pub chance_percent_type: i32,
    #[endianness(le)]
    pub chance_percent: u16,
    #[endianness(le)]
    pub chance_numerator: i32,
    #[endianness(le)]
    pub change_denominator: i32,
    #[endianness(var)]
    pub iterations_type: i32,
    #[endianness(le)]
    pub iterations: u16,
}
