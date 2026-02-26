use super::super::enums::ContainerID;
use super::super::types::{ActorRuntimeID, NetworkItemStackDescriptor};
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 31)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MobEquipmentPacket {
    pub target_runtime_id: ActorRuntimeID,
    pub item: NetworkItemStackDescriptor,
    pub slot: i8,
    pub selected_slot: i8,
    pub container_id: ContainerID,
}