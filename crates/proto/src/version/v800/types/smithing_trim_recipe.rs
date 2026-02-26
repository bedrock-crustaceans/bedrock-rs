use super::super::types::RecipeIngredient;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct SmithingTrimRecipe {
    pub recipe_id: String,
    pub template_ingredient: RecipeIngredient,
    pub base_ingredient: RecipeIngredient,
    pub addition_ingredient: RecipeIngredient,
    pub tag: String,
}