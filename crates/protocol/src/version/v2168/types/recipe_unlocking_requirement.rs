use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct RecipeUnlockingRequirement<V: ProtoVersion> {
    pub context: UnlockingContext,
    pub unlocking_ingredients: Option<Vec<V::CraftingRecipeIngredient>>,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum UnlockingContext {
    None = 0,
    AlwaysUnlocked = 1,
    PlayerInWater = 2,
    PlayerHasManyItems = 3,
}
