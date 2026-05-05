use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 72)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct GameRulesChangedPacket<V: ProtoVersion> {
    pub rules_data: V::GameRulesChangedPacketData,
}
