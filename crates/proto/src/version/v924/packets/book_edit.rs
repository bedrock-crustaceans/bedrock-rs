use crate::version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, packet};

#[packet(id = 97)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BookEditPacket<V: ProtoVersion> {
    #[endianness(var)]
    pub book_slot: i32,
    pub action: V::BookEditAction,
}
