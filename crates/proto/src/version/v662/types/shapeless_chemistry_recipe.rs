use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;
use uuid::Uuid;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ShapelessChemistryRecipe<V: ProtoVersion> {
    pub recipe_id: String,

    pub ingredients: Vec<V::RecipeIngredient>,

    pub results: Vec<V::NetworkItemInstanceDescriptor>,
    pub id: Uuid,
    pub tag: String,
    #[endianness(var)]
    pub priority: i32,
    #[endianness(var)]
    pub network_id: i32,
}
