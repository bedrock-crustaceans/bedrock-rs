use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 95)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AutomationClientConnectPacket<V: ProtoVersion> {
    pub web_socket_data: V::WebSocketPacketData,
}
