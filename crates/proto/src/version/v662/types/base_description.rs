use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct BaseDescription<V: ProtoVersion> {
    pub internal_item_descriptor: InternalItemDescriptor,
    pub molang_descriptor: MolangDescriptor<V>,
    pub item_tag_descriptor: ItemTagDescriptor,
    pub deferred_descriptor: DeferredDescriptor,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct InternalItemDescriptor {
    pub full_name: String,
    #[endianness(le)]
    pub aux_value: u16,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct MolangDescriptor<V: ProtoVersion> {
    pub full_name: String,
    pub molang_version: V::MolangVersion,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemTagDescriptor {
    pub item_tag: String,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct DeferredDescriptor {
    pub full_name: String,
    #[endianness(le)]
    pub aux_value: u16,
}
