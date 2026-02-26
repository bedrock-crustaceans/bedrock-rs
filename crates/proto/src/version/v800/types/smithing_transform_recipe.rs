use super::super::types::{NetworkItemInstanceDescriptor, RecipeIngredient};
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct SmithingTransformRecipe {
    pub recipe_id: String,
    pub template_ingredient: RecipeIngredient,
    pub base_ingredient: RecipeIngredient,
    pub addition_ingredient: RecipeIngredient,
    pub result: NetworkItemInstanceDescriptor,
    pub tag: String,
}