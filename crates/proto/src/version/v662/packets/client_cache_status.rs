use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 129)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientCacheStatusPacket {
    pub is_cache_supported: bool,
}
