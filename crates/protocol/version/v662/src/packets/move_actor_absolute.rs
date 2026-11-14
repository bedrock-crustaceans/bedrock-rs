use bedrock_protocol_version::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 18, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MoveActorAbsolutePacket<V: ProtoVersion> {
    pub move_data: V::MoveActorAbsoluteData,
}
