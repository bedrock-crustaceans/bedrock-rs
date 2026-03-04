use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;
use uuid::Uuid;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ShapedChemistryRecipe<V: ProtoVersion> {
    pub recipe_id: String,
    #[endianness(var)]
    pub width: i32,
    #[endianness(var)]
    pub height: i32,
    pub ingredient: V::RecipeIngredient,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub result_items: Vec<V::NetworkItemInstanceDescriptor>,
    pub id: Uuid,
    pub tag: String,
    #[endianness(var)]
    pub priority: i32,
}
