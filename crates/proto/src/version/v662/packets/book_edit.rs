use crate::version::versions::ProtoVersion;
use bedrockrs_macros::packet;
use bedrockrs_proto_core::ProtoCodec;
use bedrockrs_proto_core::error::ProtoCodecError;
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write, copy};
use std::mem::size_of;

#[packet(id = 97)]
#[derive(Clone, Debug)]
pub struct BookEditPacket<V: ProtoVersion> {
    pub action: V::BookEditAction,
    pub book_slot: i8,
}

impl<V: ProtoVersion> ProtoCodec for BookEditPacket<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        let mut action_stream: Vec<u8> = Vec::new();
        <V::BookEditAction as ProtoCodec>::serialize(&self.action, &mut action_stream)?;
        let mut action_cursor = Cursor::new(action_stream.as_slice());

        stream.write_i8(action_cursor.read_i8()?)?;
        <i8 as ProtoCodec>::serialize(&self.book_slot, stream)?;
        copy(&mut action_cursor, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let mut action_stream: Vec<u8> = Vec::new();

        action_stream.write_i8(stream.read_i8()?)?;
        let book_slot = <i8 as ProtoCodec>::deserialize(stream)?;
        stream.read_to_end(&mut action_stream)?;

        let mut action_cursor = Cursor::new(action_stream.as_slice());
        let action = <V::BookEditAction as ProtoCodec>::deserialize(&mut action_cursor)?;

        Ok(Self { action, book_slot })
    }

    fn size_hint(&self) -> usize {
        self.action.size_hint() + size_of::<i8>()
    }
}

// TODO: verify ProtoCodec impl
