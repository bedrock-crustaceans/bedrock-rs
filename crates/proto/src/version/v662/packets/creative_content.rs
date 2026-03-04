use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 145)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CreativeContentPacket<V: ProtoVersion> {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub write_entries: Vec<WriteEntry<V>>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct WriteEntry<V: ProtoVersion> {
    #[endianness(var)]
    pub creative_net_id: u32,
    pub item_instance: V::NetworkItemInstanceDescriptor,
}
