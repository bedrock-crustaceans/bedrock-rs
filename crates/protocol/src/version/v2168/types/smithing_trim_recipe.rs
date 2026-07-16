use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct SmithingTrimRecipe<V: ProtoVersion> {
    pub recipe_id: String,
    pub template_ingredient: V::CraftingRecipeIngredient,
    pub base_ingredient: V::CraftingRecipeIngredient,
    pub addition_ingredient: V::CraftingRecipeIngredient,
    pub tag: String,
    #[endianness(var)]
    pub network_id: i32,
}
