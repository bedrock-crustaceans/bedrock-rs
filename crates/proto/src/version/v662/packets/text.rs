use crate::version::versions::ProtoVersion;
use bedrockrs_macros::packet;
use bedrockrs_proto_core::ProtoCodec;
use bedrockrs_proto_core::error::ProtoCodecError;
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write, copy};

#[packet(id = 9)]
#[derive(Clone, Debug)]
pub struct TextPacket<V: ProtoVersion> {
    pub message_type: V::TextPacketType,
    pub localize: bool,
    pub sender_xuid: String,
    pub platform_id: String,
}

impl<V: ProtoVersion> ProtoCodec for TextPacket<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        let mut message_type_stream: Vec<u8> = Vec::new();
        V::TextPacketType::serialize(&self.message_type, &mut message_type_stream)?;
        let mut message_type_cursor = Cursor::new(message_type_stream.as_slice());

        stream.write_i8(message_type_cursor.read_i8()?)?;
        bool::serialize(&self.localize, stream)?;
        copy(&mut message_type_cursor, stream)?;
        String::serialize(&self.sender_xuid, stream)?;
        String::serialize(&self.platform_id, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let mut sub_stream: Vec<u8> = Vec::new();

        sub_stream.write_i8(stream.read_i8()?)?;
        let localize = bool::deserialize(stream)?;
        stream.read_to_end(&mut sub_stream)?;
        let mut sub_cursor = Cursor::new(sub_stream.as_slice());
        let message_type = V::TextPacketType::deserialize(&mut sub_cursor)?;
        let sender_xuid = String::deserialize(&mut sub_cursor)?;
        let platform_id = String::deserialize(&mut sub_cursor)?;

        Ok(Self {
            message_type,
            localize,
            sender_xuid,
            platform_id,
        })
    }

    fn size_hint(&self) -> usize {
        self.message_type.size_hint()
            + self.localize.size_hint()
            + self.sender_xuid.size_hint()
            + self.platform_id.size_hint()
    }
}

// VERIFY: ProtoCodec impl
