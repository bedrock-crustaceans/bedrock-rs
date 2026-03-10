use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 52)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CraftingDataPacket<V: ProtoVersion> {
    pub crafting_entries: Vec<V::CraftingDataEntry>,

    pub potion_mixes: Vec<V::PotionMixDataEntry>,

    pub container_mixes: Vec<V::ContainerMixDataEntry>,

    pub material_reducers: Vec<V::MaterialReducerDataEntry>,
    pub clear_recipes: bool,
}
