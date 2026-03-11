use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};
use bedrockrs_proto_core::ProtoCodec;
use bedrockrs_proto_core::error::ProtoCodecError;
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};
use std::mem::size_of;

#[gamepacket(id = 97)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BookEditPacket<V: ProtoVersion> {
    #[endianness(var)]
    pub book_slot: i32,
    pub action: V::BookEditAction,
}
