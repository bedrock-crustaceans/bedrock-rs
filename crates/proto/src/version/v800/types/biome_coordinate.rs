use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone, Copy)]
pub struct BiomeCoordinateData {
    #[endianness(le)]
    pub min_value_type: i32,
    #[endianness(le)]
    pub min_value: i32,
    #[endianness(le)]
    pub max_value_type: i32,
    #[endianness(le)]
    pub max_value: i32,
    #[endianness(le)]
    pub grid_offset: i64,
    #[endianness(le)]
    pub grid_step_size: i64,
    #[endianness(le)]
    pub distribution: i32,
}