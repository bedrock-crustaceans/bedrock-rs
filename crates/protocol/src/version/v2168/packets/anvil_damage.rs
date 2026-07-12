use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 141)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AnvilDamagePacket<V: ProtoVersion> {
    pub block_position: V::NetworkBlockPosition,
}
