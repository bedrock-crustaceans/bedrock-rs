use std::collections::HashMap;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 122, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BiomeDefinitionListPacket {
    #[nbt]
    pub biome_definition_data: HashMap<String, nbtx::Value>,
}
