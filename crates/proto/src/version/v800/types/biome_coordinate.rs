use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone, Copy)]
pub struct BiomeCoordinateData {
    #[endianness(var)]
    pub min_value_type: i32,
    #[endianness(le)]
    pub min_value: u16,
    #[endianness(var)]
    pub max_value_type: i32,
    #[endianness(le)]
    pub max_value: u16,
    #[endianness(le)]
    pub grid_offset: u32,
    #[endianness(le)]
    pub grid_step_size: u32,
    #[endianness(var)]
    pub distribution: i32,
}
