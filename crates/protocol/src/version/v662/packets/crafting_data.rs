use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 52, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CraftingDataPacket<V: ProtoVersion> {
    pub crafting_entries: Vec<V::CraftingDataEntry>,
    pub potion_mixes: Vec<V::PotionMixDataEntry>,
    pub container_mixes: Vec<V::ContainerMixDataEntry>,
    pub material_reducers: Vec<V::MaterialReducerDataEntry>,
    pub clear_recipes: bool,
}
