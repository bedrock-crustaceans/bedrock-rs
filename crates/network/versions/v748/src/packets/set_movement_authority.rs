use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 319, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetMovementAuthorityPacket<V: ProtoVersion> {
    pub movement_mode: V::AuthoritativeMovementMode,
}
