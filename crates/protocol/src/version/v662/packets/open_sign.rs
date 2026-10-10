use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 303, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct OpenSignPacket<V: ProtoVersion> {
    pub pos: V::NetworkBlockPosition,
    pub is_front: bool,
}
