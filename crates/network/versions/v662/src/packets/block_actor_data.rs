use std::collections::HashMap;
use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 56)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BlockActorDataPacket<V: ProtoVersion> {
    pub block_position: V::NetworkBlockPosition,
    #[nbt]
    pub actor_data_tags: HashMap<String, nbtx::Value>,
}
