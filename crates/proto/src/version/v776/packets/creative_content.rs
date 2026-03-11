use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 145)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CreativeContentPacket<V: ProtoVersion> {
    pub groups: Vec<CreativeItemGroup<V>>,
    pub contents: Vec<CreativeItemData<V>>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct CreativeItemData<V: ProtoVersion> {
    #[endianness(var)]
    pub creative_net_id: u32,
    pub item_instance: V::NetworkItemInstanceDescriptor,
    #[endianness(var)]
    pub group_id: u32,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct CreativeItemGroup<V: ProtoVersion> {
    pub category: CreativeItemCategory,
    pub name: String,
    pub icon: V::NetworkItemInstanceDescriptor,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(le)]
#[repr(i32)]
pub enum CreativeItemCategory {
    All = 0,
    Construction = 1,
    Nature = 2,
    Equipment = 3,
    Items = 4,
    ItemCommandOnly = 5,
    Undefined = 6,
}
