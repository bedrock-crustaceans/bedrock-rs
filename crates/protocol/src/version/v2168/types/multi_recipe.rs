use bedrock_macros::ProtoCodec;
use uuid::Uuid;

#[derive(ProtoCodec, Clone, Debug)]
pub struct MultiRecipe {
    pub multi_recipe_id: Uuid,
    #[endianness(var)]
    pub network_id: i32,
}
