use bedrockrs_macros::ProtoCodec;
use crate::v800::types::BiomeCoordinateData;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeScatterParamData {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub coordinates: Vec<BiomeCoordinateData>,
    // Encoded as VarInt enum ordinal.
    #[endianness(var)]
    pub eval_order: i32,
    // Encoded as VarInt where -1 means "no expression op".
    #[endianness(var)]
    pub chance_percent_type: i32,
    #[endianness(le)]
    pub chance_percent: u16,
    #[endianness(le)]
    pub chance_numerator: i32,
    #[endianness(le)]
    pub change_denominator: i32,
    // Encoded as VarInt where -1 means "no expression op".
    #[endianness(var)]
    pub iterations_type: i32,
    #[endianness(le)]
    pub iterations: u16,
}
