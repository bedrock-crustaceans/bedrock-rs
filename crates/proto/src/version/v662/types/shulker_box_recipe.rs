use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;
use uuid::Uuid;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ShulkerBoxRecipe<V: ProtoVersion> {
    pub recipe_unique_id: String,

    pub ingredient_list: Vec<V::RecipeIngredient>,

    pub production_list: Vec<V::NetworkItemInstanceDescriptor>,
    pub recipe_id: Uuid,
    pub recipe_tag: String,
    #[endianness(var)]
    pub priority: i32,
    #[endianness(var)]
    pub network_id: i32,
}
