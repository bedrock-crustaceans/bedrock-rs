use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};
use vek::Vec3;

#[gamepacket(id = 15)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AddItemActorPacket<V: ProtoVersion> {
    pub target_actor_id: V::ActorUniqueID,
    pub target_runtime_id: V::ActorRuntimeID,
    pub item: V::NetworkItemStackDescriptor,
    #[endianness(le)]
    pub position: Vec3<f32>,
    #[endianness(le)]
    pub velocity: Vec3<f32>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub entity_data: Vec<V::DataItem>, // TODO: verify vec_repr & vec_endianness
    pub from_fishing: bool,
}
