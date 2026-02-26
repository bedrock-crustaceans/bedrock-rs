use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 129)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientCacheStatusPacket {
    pub is_cache_supported: bool,
}