use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct InventoryAction<V: ProtoVersion> {
    pub source: V::InventorySource,
    #[endianness(var)]
    pub slot: u32,
    pub from_item_descriptor: V::NetworkItemStackDescriptor,
    pub to_item_descriptor: V::NetworkItemStackDescriptor,
}
