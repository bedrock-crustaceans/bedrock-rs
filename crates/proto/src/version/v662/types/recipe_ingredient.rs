use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct RecipeIngredient<V: ProtoVersion> {
    pub item_descriptor: V::ItemDescriptorType,
    #[endianness(var)]
    pub stack_size: i32,
}
