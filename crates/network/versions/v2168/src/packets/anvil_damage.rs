use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{ProtoCodec, packet};

#[packet(id = 141, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AnvilDamagePacket<V: ProtoVersion> {
    pub block_position: V::NetworkBlockPosition,
}
