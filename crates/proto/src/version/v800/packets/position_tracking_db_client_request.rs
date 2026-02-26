use super::super::types::PositionTrackingId;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 154)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PositionTrackingDBClientRequestPacket {
    pub action: Action,
    pub id: PositionTrackingId,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum Action {
    Query = 0
}
