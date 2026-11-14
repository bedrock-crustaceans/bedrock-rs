use bedrock_protocol_version::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 34, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BlockPickRequestPacket<V: ProtoVersion> {
    pub position: V::BlockPos,
    pub with_data: bool,
    pub max_slots: i8,
}
