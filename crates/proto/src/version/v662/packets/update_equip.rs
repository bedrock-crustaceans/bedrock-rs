use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 81)]
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
