use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 50, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct InventorySlotPacket<V: ProtoVersion> {
    pub container_id: V::ContainerID,
    #[endianness(var)]
    pub slot: u32,
    pub item: V::NetworkItemStackDescriptor,
}
