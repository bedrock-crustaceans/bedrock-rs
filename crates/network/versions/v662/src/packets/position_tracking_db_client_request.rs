use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 154, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PositionTrackingDBClientRequestPacket<V: ProtoVersion> {
    pub action: PositionTrackingDBClientRequestAction,
    pub id: V::PositionTrackingId,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum PositionTrackingDBClientRequestAction {
    Query = 0,
}
