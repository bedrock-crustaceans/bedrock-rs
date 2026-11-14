use bedrock_protocol_version::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 2, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayStatusPacket<V: ProtoVersion> {
    pub status: V::PlayStatus,
}
