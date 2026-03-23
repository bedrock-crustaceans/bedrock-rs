use std::collections::HashMap;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 122)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BiomeDefinitionListPacket {
    #[nbt]
    pub biome_definition_data: HashMap<String, nbtx::Value>,
}
