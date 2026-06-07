use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;
use uuid::Uuid;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum CraftingDataEntryType<V: ProtoVersion> {
    ShapelessRecipe(V::ShapelessRecipe) = 0,
    ShapedRecipe(V::ShapedRecipe) = 1,
    FurnaceRecipe {
        #[endianness(var)]
        item_data: i32,
        result_item: V::NetworkItemInstanceDescriptor,
        recipe_tag: String,
    } = 2,
    FurnaceAuxRecipe {
        #[endianness(var)]
        item_data: i32,
        #[endianness(var)]
        auxiliary_item_data: i32,
        result_item: V::NetworkItemInstanceDescriptor,
        recipe_tag: String,
    } = 3,
    MultiRecipe {
        multi_recipe: Uuid,
        #[endianness(var)]
        net_id: u32,
    } = 4,
    ShulkerBoxRecipe(V::ShulkerBoxRecipe) = 5,
    ShapelessChemistryRecipe(V::ShapelessRecipe) = 6,
    ShapedChemistryRecipe(V::ShapedRecipe)= 7,
    SmithingTransformRecipe(V::SmithingTransformRecipe) = 8,
    SmithingTrimRecipe(V::SmithingTrimRecipe) = 9,
}
