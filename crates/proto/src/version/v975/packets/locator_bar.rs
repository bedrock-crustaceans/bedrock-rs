use uuid::Uuid;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 341)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LocatorBarPacket {
    pub waypoints: Vec<LocatorBarPayload>
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct LocatorBarPayload {
    pub group_handle: Uuid,
    pub waypoint: LocatorBarWaypoint,
    pub action: LocatorBarPayloadAction
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u8)]
#[repr(u8)]
pub enum LocatorBarPayloadAction {
    None = 0,
    Add = 1,
    Remove = 2,
    Update = 3,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct LocatorBarWaypoint {
    #[endianness(le)]
    pub update_flag: u32,
    pub visible: Option<bool>,
    pub world_position: Option<LocatorBarWaypointWorldPosition>,
    pub texture_path: String,
    #[endianness(le)]
    pub icon_size: (f32, f32),
    #[endianness(le)]
    pub color: Option<i32>,
    pub client_authority: Option<bool>,
    #[endianness(var)]
    pub entity_unique_id: Option<i64>
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct LocatorBarWaypointWorldPosition {
    #[endianness(le)]
    pub position: (f32, f32, f32),
    #[endianness(var)]
    pub dimension: i32,
}