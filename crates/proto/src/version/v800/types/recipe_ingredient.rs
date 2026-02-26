use super::super::enums::ItemDescriptorType;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct RecipeIngredient {
    pub item_descriptor: ItemDescriptorType,
    #[endianness(var)]
    pub stack_size: i32,
}