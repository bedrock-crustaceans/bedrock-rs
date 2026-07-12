use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct RecipeIngredient<V: ProtoVersion> {
    pub item_descriptor: V::ItemDescriptorType,
    #[endianness(le)]
    pub stack_size: i16,
}
