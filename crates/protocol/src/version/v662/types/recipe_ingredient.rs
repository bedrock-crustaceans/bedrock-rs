use crate::version::versions::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct RecipeIngredient<V: ProtoVersion> {
    pub item_descriptor: V::ItemDescriptorType,
    #[endianness(var)]
    pub stack_size: i32,
}
