use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 49)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct InventoryContentPacket<V: ProtoVersion> {
    #[endianness(var)]
    pub inventory_id: u32,
    pub slots: Vec<V::NetworkItemStackDescriptor>,
    pub container_name_data: V::FullContainerName,
    pub storage_item: V::NetworkItemStackDescriptor,
}
