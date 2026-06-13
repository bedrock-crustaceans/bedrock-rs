use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct InventoryAction<V: ProtoVersion> {
    pub source: V::InventorySource,
    #[endianness(var)]
    pub slot: u32,
    pub from_item_descriptor: V::NetworkItemStackDescriptorV2,
    pub to_item_descriptor: V::NetworkItemStackDescriptorV2,
}
