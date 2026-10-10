use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 72, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct GameRulesChangedPacket<V: ProtoVersion> {
    pub rules_data: V::GameRulesChangedPacketData,
}
