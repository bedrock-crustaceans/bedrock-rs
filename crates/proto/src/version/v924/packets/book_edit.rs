use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 97)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BookEditPacket<V: ProtoVersion> {
    #[endianness(var)]
    pub book_slot: i32,
    pub action: V::BookEditAction,
}
