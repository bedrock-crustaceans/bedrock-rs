use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 95)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AutomationClientConnectPacket<V: ProtoVersion> {
    pub web_socket_data: V::WebSocketPacketData,
}
