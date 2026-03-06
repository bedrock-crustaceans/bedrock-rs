use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 162)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemComponentPacket {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub items: Vec<ItemsEntry>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemsEntry {
    pub component_item_name: String,
    #[nbt]
    pub component_data: nbtx::Value, // TODO: NBT Structure
}
