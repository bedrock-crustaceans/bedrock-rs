use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct MapItemTrackedActorUniqueID<V: ProtoVersion> {
    pub unique_id_type: MapItemTrackedActorType,
    pub entity_id: Option<V::ActorUniqueID>,
    pub block_position: Option<V::NetworkBlockPosition>,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(le)]
#[repr(i32)]
pub enum MapItemTrackedActorType {
    Entity = 0,
    BlockEntity = 1,
    Other = 2,
}
