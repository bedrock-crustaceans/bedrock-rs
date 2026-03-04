use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 31)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MobEquipmentPacket<V: ProtoVersion> {
    pub target_runtime_id: V::ActorRuntimeID,
    pub item: V::NetworkItemStackDescriptor,
    pub slot: i8,
    pub selected_slot: i8,
    pub container_id: V::ContainerID,
}