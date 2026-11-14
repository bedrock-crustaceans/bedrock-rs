use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct NetworkItemInstanceDescriptor {
    #[endianness(var)]
    pub id: i32,
    #[endianness(le)]
    pub stack_size: u16,
    #[endianness(var)]
    pub aux_value: u32,
    #[endianness(var)]
    pub block_runtime_id: i32,
    pub user_data_buffer: Vec<u8>,
}
