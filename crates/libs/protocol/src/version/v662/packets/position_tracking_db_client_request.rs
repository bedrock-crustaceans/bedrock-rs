use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 154)]
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
