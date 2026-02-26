use super::super::types::NetworkItemInstanceDescriptor;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 145)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CreativeContentPacket {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub groups: Vec<CreativeItemGroup>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub contents: Vec<CreativeItemData>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct CreativeItemData {
    #[endianness(var)]
    pub creative_net_id: u32,
    pub item_instance: NetworkItemInstanceDescriptor,
    #[endianness(var)]
    pub group_id: i32,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct CreativeItemGroup {
    pub category: CreativeItemCategory,
    pub name: String,
    pub icon: NetworkItemInstanceDescriptor,
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
