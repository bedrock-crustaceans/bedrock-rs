use super::super::enums::BookEditAction;
use bedrockrs_macros::gamepacket;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::ProtoCodec;
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};
use std::mem::size_of;

#[gamepacket(id = 97)]
#[derive(Clone, Debug)]
pub struct BookEditPacket {
    pub action: BookEditAction,
    pub book_slot: i8,
}

impl ProtoCodec for BookEditPacket {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        let mut action_stream: Vec<u8> = Vec::new();
        <BookEditAction as ProtoCodec>::proto_serialize(&self.action, &mut action_stream)?;
        let mut action_cursor = Cursor::new(action_stream.as_slice());

        stream.write_i8(action_cursor.read_i8()?)?;
        <i8 as ProtoCodec>::proto_serialize(&self.book_slot, stream)?;
        action_cursor.read_to_end(stream)?;

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let mut action_stream: Vec<u8> = Vec::new();

        action_stream.write_i8(stream.read_i8()?)?;
        let book_slot = <i8 as ProtoCodec>::proto_deserialize(stream)?;
        stream.read_to_end(&mut action_stream)?;

        let mut action_cursor = Cursor::new(action_stream.as_slice());
        let action = <BookEditAction as ProtoCodec>::proto_deserialize(&mut action_cursor)?;

        Ok(Self { action, book_slot })
    }

    fn get_size_prediction(&self) -> usize {
        self.action.get_size_prediction() + size_of::<i8>()
    }
}

// VERIFY: ProtoCodec impl
