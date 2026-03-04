use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};
use bedrockrs_proto_core::ProtoCodec;

#[gamepacket(id = 50)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct InventorySlotPacket<V: ProtoVersion> {
    #[endianness(var)]
    pub container_id: u32,
    #[endianness(var)]
    pub slot: u32,
    pub container_name_data: V::FullContainerName,
    pub storage_item: V::NetworkItemStackDescriptor,
    pub item: V::NetworkItemStackDescriptor,
}
