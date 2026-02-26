use super::super::types::{NetworkItemInstanceDescriptor, RecipeIngredient};
use bedrockrs_macros::ProtoCodec;
use uuid::Uuid;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ShapedChemistryRecipe {
    pub recipe_id: String,
    #[endianness(var)]
    pub width: i32,
    #[endianness(var)]
    pub height: i32,
    pub ingredient: RecipeIngredient,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub result_items: Vec<NetworkItemInstanceDescriptor>,
    pub id: Uuid,
    pub tag: String,
    #[endianness(var)]
    pub priority: i32,
    pub assume_symmetry: bool,
}