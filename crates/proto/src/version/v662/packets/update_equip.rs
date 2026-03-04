use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 81)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateEquipPacket<V: ProtoVersion> {
    pub container_id: V::ContainerID,
    pub container_type: V::ContainerType,
    #[endianness(var)]
    pub size: i32,
    pub target_actor_id: V::ActorUniqueID,
    #[nbt]
    pub data_tags: nbtx::Value, // TODO: NBT Structure
}