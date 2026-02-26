use bedrockrs_macros::ProtoCodec;
use super::super::types::RecipeIngredient;

#[derive(ProtoCodec, Clone, Debug)]
pub struct RecipeUnlockingRequirement {
    context: UnlockingContext
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum UnlockingContext {
    None {
        #[vec_repr(i32)]
        #[vec_endianness(var)]
        unlocking_ingredients: Vec<RecipeIngredient>
    } = 0,
    AlwaysUnlocked = 1,
    PlayerInWater = 2,
    PlayerHasManyItems = 3,
}