use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 39)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetActorDataPacket<V: ProtoVersion> {
    pub target_runtime_id: V::ActorRuntimeID,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub actor_data: Vec<V::DataItem>, // VERIFY: vec_repr & vec_endianness
    pub synced_properties: V::PropertySyncData,
    #[endianness(var)]
    pub tick: u64,
}