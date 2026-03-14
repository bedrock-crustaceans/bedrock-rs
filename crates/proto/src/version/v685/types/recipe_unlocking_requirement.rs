use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct RecipeUnlockingRequirement<V: ProtoVersion> {
    context: UnlockingContext<V>,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum UnlockingContext<V: ProtoVersion> {
    None {
        #[vec_repr(i32)]
        #[vec_endianness(var)]
        unlocking_ingredients: Vec<V::RecipeIngredient>,
    } = 0,
    AlwaysUnlocked = 1,
    PlayerInWater = 2,
    PlayerHasManyItems = 3,
}
