use bedrockrs_macros::{gamepacket, ProtoCodec};
use super::super::enums::ItemVersion;

#[gamepacket(id = 162)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemComponentPacket {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub items: Vec<ItemsEntry>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemsEntry {
    pub component_item_name: String,
    #[endianness(le)]
    pub runtime_id: i16,
    pub is_component_based: bool,
    pub version: ItemVersion,
    #[nbt]
    pub component_data: nbtx::Value
}
