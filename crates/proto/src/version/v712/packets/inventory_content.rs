use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 49)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct InventoryContentPacket<V: ProtoVersion> {
    #[endianness(var)]
    pub inventory_id: u32,

    pub slots: Vec<V::NetworkItemStackDescriptor>,
    #[endianness(var)]
    pub container_name_dynamic_id: i32,
}
