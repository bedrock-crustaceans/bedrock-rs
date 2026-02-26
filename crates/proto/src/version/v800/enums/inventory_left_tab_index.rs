use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum InventoryLeftTabIndex {
    None = 0,
    RecipeConstruction = 1,
    RecipeEquipment = 2,
    RecipeItems = 3,
    RecipeNature = 4,
    RecipeSearch = 5,
    Survival = 6,
    Count = 7,
}