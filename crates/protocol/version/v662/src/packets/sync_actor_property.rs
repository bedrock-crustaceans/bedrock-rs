use std::collections::HashMap;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 165, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SyncActorPropertyPacket {
    #[nbt]
    pub property_data: HashMap<String, nbtx::Value>,
}
