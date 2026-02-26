use super::super::types::{ActorRuntimeID, DataItem, PropertySyncData};
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 39)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetActorDataPacket {
    pub target_runtime_id: ActorRuntimeID,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub actor_data: Vec<DataItem>, // VERIFY: vec_repr & vec_endianness
    pub synced_properties: PropertySyncData,
    #[endianness(var)]
    pub tick: u64,
}