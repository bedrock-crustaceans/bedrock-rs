use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackRequestNetworkItemInstanceDescriptor<V: ProtoVersion> {
    pub ingredient: V::RecipeIngredient,
    #[endianness(var)]
    pub block_runtime_id: u32,
    pub user_data_buffer: Vec<u8>,
}
