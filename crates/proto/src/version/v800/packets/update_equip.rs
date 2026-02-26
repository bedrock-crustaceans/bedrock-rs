use super::super::enums::{ContainerID, ContainerType};
use super::super::types::ActorUniqueID;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 81)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateEquipPacket {
    pub container_id: ContainerID,
    pub container_type: ContainerType,
    #[endianness(var)]
    pub size: i32,
    pub target_actor_id: ActorUniqueID,
    #[nbt]
    pub data_tags: nbtx::Value, // TODO: NBT Structure
}