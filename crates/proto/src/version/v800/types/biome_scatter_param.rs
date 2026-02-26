use bedrockrs_macros::ProtoCodec;

use crate::v800::types::BiomeCoordinateData;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeScatterParamData {
    #[endianness(le)]
    pub coordinates: Vec<BiomeCoordinateData>,
    #[endianness(le)]
    pub eval_order: i32,
    #[endianness(le)]
    pub chance_percent_type: i32,
    #[endianness(le)]
    pub chance_percent: i32,
    #[endianness(le)]
    pub chance_numerator: i32,
    #[endianness(le)]
    pub change_denominator: i32,
    #[endianness(le)]
    pub iterations_type: i32,
    #[endianness(le)]
    pub iterations: i32,
}