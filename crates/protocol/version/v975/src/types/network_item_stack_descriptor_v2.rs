use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct NetworkItemStackDescriptorV2 {
    #[endianness(le)]
    pub id: i16,
    #[endianness(le)]
    pub stack_size: u16,
    #[endianness(var)]
    pub aux_value: u32,
    pub net_id: Option<ItemStackNetIdVariant>,
    #[endianness(var)]
    pub block_runtime_id: u32,
    pub user_data_buffer: Vec<u8>,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum ItemStackNetIdVariant {
    ItemStackNetId(#[endianness(var)] i32) = 0,
    ItemStackRequestId(#[endianness(var)] i32) = 1,
    ItemStackLegacyRequestId(#[endianness(var)] i32) = 2,
}
