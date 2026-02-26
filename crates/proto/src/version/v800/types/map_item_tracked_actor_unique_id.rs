use super::super::types::{ActorUniqueID, NetworkBlockPosition};
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct MapItemTrackedActorUniqueID {
    pub unique_id_type: MapItemTrackedActorType,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(le)]
#[repr(i32)]
pub enum MapItemTrackedActorType {
    Entity(ActorUniqueID) = 0,
    BlockEntity(NetworkBlockPosition) = 1,
    Other = 2,
}
