use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 124, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LevelEventGenericPacket<V: ProtoVersion> {
    pub event_id: V::LevelEvent,
    #[nbt]
    pub event_data: nbtx::Value
}
