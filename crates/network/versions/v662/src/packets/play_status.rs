use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 2, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayStatusPacket<V: ProtoVersion> {
    pub status: V::PlayStatus,
}
