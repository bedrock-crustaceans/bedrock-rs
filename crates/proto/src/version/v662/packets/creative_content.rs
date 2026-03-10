use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 145)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CreativeContentPacket<V: ProtoVersion> {
    pub write_entries: Vec<WriteEntry<V>>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct WriteEntry<V: ProtoVersion> {
    #[endianness(var)]
    pub creative_net_id: u32,
    pub item_instance: V::NetworkItemInstanceDescriptor,
}
