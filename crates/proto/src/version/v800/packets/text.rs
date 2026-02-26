use super::super::enums::TextPacketType;
use bedrockrs_macros::gamepacket;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::ProtoCodec;
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};

#[gamepacket(id = 9)]
#[derive(Clone, Debug)]
pub struct TextPacket {
    pub message_type: TextPacketType,
    pub localize: bool,
    pub sender_xuid: String,
    pub platform_id: String,
    pub filtered_message: String,
}

impl ProtoCodec for TextPacket {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        let mut message_type_stream: Vec<u8> = Vec::new();
        TextPacketType::proto_serialize(&self.message_type, &mut message_type_stream)?;
        let mut message_type_cursor = Cursor::new(message_type_stream.as_slice());

        stream.write_i8(message_type_cursor.read_i8()?)?;
        bool::proto_serialize(&self.localize, stream)?;
        message_type_cursor.read_to_end(stream)?;
        String::proto_serialize(&self.sender_xuid, stream)?;
        String::proto_serialize(&self.platform_id, stream)?;
        String::proto_serialize(&self.filtered_message, stream)?;

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let mut sub_stream: Vec<u8> = Vec::new();

        sub_stream.write_i8(stream.read_i8()?)?;
        let localize = bool::proto_deserialize(stream)?;
        stream.read_to_end(&mut sub_stream)?;
        let mut sub_cursor = Cursor::new(sub_stream.as_slice());
        let message_type = TextPacketType::proto_deserialize(&mut sub_cursor)?;
        let sender_xuid = String::proto_deserialize(&mut sub_cursor)?;
        let platform_id = String::proto_deserialize(&mut sub_cursor)?;
        let filtered_message = String::proto_deserialize(&mut sub_cursor)?;

        Ok(Self {
            message_type,
            localize,
            sender_xuid,
            platform_id,
            filtered_message,
        })
    }

    fn get_size_prediction(&self) -> usize {
        self.message_type.get_size_prediction()
            + self.localize.get_size_prediction()
            + self.sender_xuid.get_size_prediction()
            + self.platform_id.get_size_prediction()
    }
}

// VERIFY: ProtoCodec impl
