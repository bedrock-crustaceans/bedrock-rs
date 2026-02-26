use super::super::types::PositionTrackingId;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 153)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PositionTrackingDBServerBroadcastPacket {
    pub action: Action,
    pub id: PositionTrackingId,
    #[nbt]
    pub position_tracking_data: nbtx::Value, // TODO: NBT Structure
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum Action {
    Update = 0,
    Destroy = 1,
    NotFound = 2,
}
