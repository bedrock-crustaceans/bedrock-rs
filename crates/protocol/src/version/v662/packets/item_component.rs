use std::collections::HashMap;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 162)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemComponentPacket {
    pub items: Vec<ItemsEntry>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemsEntry {
    pub component_item_name: String,
    #[nbt]
    pub component_data: HashMap<String, nbtx::Value>
}
