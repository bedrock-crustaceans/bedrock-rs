use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct NetworkItemStackDescriptorV2 {
    #[endianness(le)]
    pub id: i16,
    #[endianness(le)]
    pub stack_size: u16,
    #[endianness(var)]
    pub aux_value: u32,
    #[endianness(var)]
    pub net_id: Option<i32>,
    #[endianness(var)]
    pub block_runtime_id: u32,
    pub user_data_buffer: Vec<u8>,
}
