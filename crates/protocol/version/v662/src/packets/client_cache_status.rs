use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 129, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientCacheStatusPacket {
    pub is_cache_supported: bool,
}
