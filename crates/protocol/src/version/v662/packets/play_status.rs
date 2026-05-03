use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 2)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayStatusPacket<V: ProtoVersion> {
    pub status: V::PlayStatus,
}
