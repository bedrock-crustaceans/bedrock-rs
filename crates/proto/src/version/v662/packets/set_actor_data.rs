use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 39)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetActorDataPacket<V: ProtoVersion> {
    pub target_runtime_id: V::ActorRuntimeID,

    pub actor_data: Vec<V::DataItem>, // VERIFY: vec_repr & vec_endianness
    pub synced_properties: V::PropertySyncData,
    #[endianness(var)]
    pub tick: u64,
}
