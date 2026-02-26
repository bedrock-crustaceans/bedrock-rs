use super::super::types::{NetworkItemInstanceDescriptor, ShapedChemistryRecipe, ShapedRecipe, ShapelessRecipe, ShulkerBoxRecipe, SmithingTransformRecipe, SmithingTrimRecipe};
use bedrockrs_macros::ProtoCodec;
use uuid::Uuid;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum CraftingDataEntryType {
    ShapelessRecipe {
        shapeless_recipe: ShapelessRecipe,
        #[endianness(var)]
        net_id: i32,
    } = 0,
    ShapedRecipe {
        shaped_recipe: ShapedRecipe,
        #[endianness(var)]
        net_id: i32,
    } = 1,
    FurnaceRecipe {
        #[endianness(var)]
        item_data: i32,
        result_item: NetworkItemInstanceDescriptor,
        recipe_tag: String,
    } = 2,
    FurnaceAuxRecipe {
        #[endianness(var)]
        item_data: i32,
        #[endianness(var)]
        auxiliary_item_data: i32,
        result_item: NetworkItemInstanceDescriptor,
        recipe_tag: String,
    } = 3,
    MultiRecipe {
        multi_recipe: Uuid,
        #[endianness(var)]
        net_id: i32,
    } = 4,
    ShulkerBoxRecipe {
        shulker_box_recipe: ShulkerBoxRecipe,
        #[endianness(var)]
        net_id: i32,
    } = 5,
    ShapelessChemistryRecipe {
        shapeless_chemistry_recipe: ShapelessRecipe,
        #[endianness(var)]
        net_id: i32,
    } = 6,
    ShapedChemistryRecipe {
        shaped_chemistry_recipe: ShapedChemistryRecipe,
        #[endianness(var)]
        net_id: i32,
    } = 7,
    SmithingTransformRecipe {
        smithing_transform_recipe: SmithingTransformRecipe,
        #[endianness(var)]
        net_id: i32,
    } = 8,
    SmithingTrimRecipe {
        smithing_trim_recipe: SmithingTrimRecipe,
        #[endianness(var)]
        net_id: i32,
    } = 9,
}
