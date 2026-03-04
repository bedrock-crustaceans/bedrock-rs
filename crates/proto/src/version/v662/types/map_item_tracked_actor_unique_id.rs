use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct MapItemTrackedActorUniqueID<V: ProtoVersion> {
    pub unique_id_type: MapItemTrackedActorType<V>,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(le)]
#[repr(i32)]
pub enum MapItemTrackedActorType<V: ProtoVersion> {
    Entity(V::ActorUniqueID) = 0,
    BlockEntity(V::NetworkBlockPosition) = 1,
    Other = 2,
}
