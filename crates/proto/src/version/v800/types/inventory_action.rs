use super::super::types::{InventorySource, NetworkItemStackDescriptor};
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct InventoryAction {
    pub source: InventorySource,
    #[endianness(var)]
    pub slot: u32,
    pub from_item_descriptor: NetworkItemStackDescriptor,
    pub to_item_descriptor: NetworkItemStackDescriptor,
}
