use uuid::Uuid;
use vek::Vec3;
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
    #[endianness(le)]
    pub texture: Option<u32>,
    #[endianness(le)]
    pub color: Option<i32>,
    pub client_authority: Option<bool>,
    #[endianness(var)]
    pub entity_unique_id: Option<i64>
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_endianness(le)]
#[enum_repr(u32)]
#[repr(u32)]
pub enum LocatorBarWaypointTexture {
    Square = 0,
    Circle = 1,
    SmallSquare = 2,
    SmallStar = 3,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct LocatorBarWaypointWorldPosition {
    #[endianness(le)]
    pub position: Vec3<f32>,
    #[endianness(var)]
    pub dimension: i32,
}