use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 50)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct InventorySlotPacket<V: ProtoVersion> {
    pub container_id: V::ContainerID,
    #[endianness(var)]
    pub slot: u32,
    pub item: V::NetworkItemStackDescriptor,
}
