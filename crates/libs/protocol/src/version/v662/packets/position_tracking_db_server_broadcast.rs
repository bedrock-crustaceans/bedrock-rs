use std::collections::HashMap;
use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 153)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PositionTrackingDBServerBroadcastPacket<V: ProtoVersion> {
    pub action: PositionTrackingDBServerBroadcastAction,
    pub id: V::PositionTrackingId,
    #[nbt]
    pub position_tracking_data: HashMap<String, nbtx::Value>
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum PositionTrackingDBServerBroadcastAction {
    Update = 0,
    Destroy = 1,
    NotFound = 2,
}
