use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 162)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemComponentPacket {
    pub items: Vec<ItemsEntry>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemsEntry {
    pub component_item_name: String,
    #[nbt]
    pub component_data: nbtx::Value, // TODO: NBT Structure
}
