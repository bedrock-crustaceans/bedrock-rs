use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, packet};

#[packet(id = 50)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct InventorySlotPacket<V: ProtoVersion> {
    #[endianness(var)]
    pub container_id: u32,
    #[endianness(var)]
    pub slot: u32,
    pub container_name_data: V::FullContainerName,
    #[endianness(var)]
    pub dynamic_container_size: u32,
    pub item: V::NetworkItemStackDescriptor,
}
