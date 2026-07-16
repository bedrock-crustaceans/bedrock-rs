use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 52)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CraftingDataPacket<V: ProtoVersion> {
    pub shaped_recipes: Vec<V::ShapedRecipe>,
    pub shapeless_recipes: Vec<V::ShapelessRecipe>,
    pub multi_recipes: Vec<V::MultiRecipe>,
    pub user_data_shapeless_recipes: Vec<V::ShapelessRecipe>,
    pub shapeless_chemistry_recipes: Vec<V::ShapelessRecipe>,
    pub shaped_chemistry_recipes: Vec<V::ShapedRecipe>,
    pub smithing_transform_recipes: Vec<V::SmithingTransformRecipe>,
    pub smithing_trim_recipes: Vec<V::SmithingTrimRecipe>,
    pub potion_mixes: Vec<V::PotionMixDataEntry>,
    pub container_mixes: Vec<V::ContainerMixDataEntry>,
    pub material_reducers: Vec<V::MaterialReducerDataEntry>,
    pub clear_recipes: bool,
}
