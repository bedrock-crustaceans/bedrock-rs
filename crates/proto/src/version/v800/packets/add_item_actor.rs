use super::super::types::{ActorRuntimeID, ActorUniqueID, DataItem, NetworkItemStackDescriptor};
use bedrockrs_macros::{gamepacket, ProtoCodec};
use vek::Vec3;

#[gamepacket(id = 15)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AddItemActorPacket {
    pub target_actor_id: ActorUniqueID,
    pub target_runtime_id: ActorRuntimeID,
    pub item: NetworkItemStackDescriptor,
    #[endianness(le)]
    pub position: Vec3<f32>,
    #[endianness(le)]
    pub velocity: Vec3<f32>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub entity_data: Vec<DataItem>, // VERIFY: vec_repr & vec_endianness
    pub from_fishing: bool,
}