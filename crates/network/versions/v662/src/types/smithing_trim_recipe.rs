use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct SmithingTrimRecipe<V: ProtoVersion> {
    pub recipe_id: String,
    pub template_ingredient: V::RecipeIngredient,
    pub base_ingredient: V::RecipeIngredient,
    pub addition_ingredient: V::RecipeIngredient,
    pub tag: String,
    #[endianness(var)]
    pub network_id: u32,
}
