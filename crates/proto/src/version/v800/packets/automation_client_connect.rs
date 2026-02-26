use super::super::types::WebSocketPacketData;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 95)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AutomationClientConnectPacket {
    pub web_socket_data: WebSocketPacketData,
}