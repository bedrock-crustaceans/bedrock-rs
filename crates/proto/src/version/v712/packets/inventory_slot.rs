use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 50)]
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
