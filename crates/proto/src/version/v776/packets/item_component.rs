use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 162)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemComponentPacket<V: ProtoVersion> {
    pub items: Vec<ItemsEntry<V>>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemsEntry<V: ProtoVersion> {
    pub component_item_name: String,
    #[endianness(le)]
    pub runtime_id: i16,
    pub is_component_based: bool,
    pub version: V::ItemVersion,
    #[nbt]
    pub component_data: nbtx::Value,
}
