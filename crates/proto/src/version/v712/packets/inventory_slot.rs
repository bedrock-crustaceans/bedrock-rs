use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 50)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct InventorySlotPacket<V: ProtoVersion> {
    #[endianness(var)]
    pub container_id: i32,
    #[endianness(var)]
    pub slot: u32,
    #[endianness(var)]
    pub container_name_dynamic_id: i32,
    pub item: V::NetworkItemStackDescriptor,
}
