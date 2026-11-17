use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 155, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct DebugInfoPacket<V: ProtoVersion> {
    pub actor_id: V::ActorUniqueID,
    pub data: Vec<u8>,
}
