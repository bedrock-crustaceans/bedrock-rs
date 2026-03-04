use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 50)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct InventorySlotPacket<V: ProtoVersion> {
    pub container_id: V::ContainerID,
    #[endianness(var)]
    pub slot: u32,
    pub item: V::NetworkItemStackDescriptor,
}