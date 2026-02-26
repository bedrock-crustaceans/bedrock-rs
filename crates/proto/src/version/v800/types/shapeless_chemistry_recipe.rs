use super::super::types::{NetworkItemInstanceDescriptor, RecipeIngredient};
use bedrockrs_macros::ProtoCodec;
use uuid::Uuid;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ShapelessChemistryRecipe {
    pub recipe_id: String,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub ingredients: Vec<RecipeIngredient>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub results: Vec<NetworkItemInstanceDescriptor>,
    pub id: Uuid,
    pub tag: String,
    #[endianness(var)]
    pub priority: i32,
}