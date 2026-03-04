use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 153)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PositionTrackingDBServerBroadcastPacket<V: ProtoVersion> {
    pub action: Action,
    pub id: V::PositionTrackingId,
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