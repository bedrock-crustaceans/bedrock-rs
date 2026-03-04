use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 49)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct InventoryContentPacket<V: ProtoVersion> {
    #[endianness(var)]
    pub inventory_id: u32,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub slots: Vec<V::NetworkItemStackDescriptor>,
    pub container_name_data: V::FullContainerName,
    #[endianness(var)]
    pub dynamic_container_size: u32,
}
