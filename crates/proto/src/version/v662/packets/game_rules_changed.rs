use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 72)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct GameRulesChangedPacket<V: ProtoVersion> {
    pub rules_data: V::GameRulesChangedPacketData,
}