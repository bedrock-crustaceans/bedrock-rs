use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;
use uuid::Uuid;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ShapedRecipe<V: ProtoVersion> {
    pub recipe_unique_id: String,
    #[endianness(var)]
    pub width: i32,
    #[endianness(var)]
    pub height: i32,
    pub ingredients: Vec<V::CraftingRecipeIngredient>,
    pub production_list: Vec<V::NetworkItemInstanceDescriptor>,
    pub recipe_id: Uuid,
    pub recipe_tag: String,
    #[endianness(var)]
    pub priority: i32,
    pub assume_symmetry: bool,
    pub unlocking_requirement: Option<V::RecipeUnlockingRequirement>,
    #[endianness(var)]
    pub network_id: i32,
}
