use super::super::types::{NetworkItemStackDescriptor, FullContainerName};
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 49)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct InventoryContentPacket {
    #[endianness(var)]
    pub inventory_id: u32,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub slots: Vec<NetworkItemStackDescriptor>,
    pub container_name_data: FullContainerName,
    pub storage_item: NetworkItemStackDescriptor,
}