use super::super::enums::CommandOriginType;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::ProtoCodec;
use std::io::{Cursor, Read};
use uuid::Uuid;
use varint_rs::{VarintReader, VarintWriter};

#[derive(Clone, Debug)]
pub struct CommandOriginData {
    pub command_type: CommandOriginType,
    pub command_uuid: Uuid,
    pub request_id: String,
}

impl ProtoCodec for CommandOriginData {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        let mut type_stream: Vec<u8> = Vec::new();
        self.command_type.proto_serialize(&mut type_stream)?;
        let mut type_cursor = Cursor::new(type_stream.as_slice());

        stream.write_u32_varint(type_cursor.read_u32_varint()?)?;
        self.command_uuid.proto_serialize(stream)?;
        self.request_id.proto_serialize(stream)?;
        type_cursor.read_to_end(stream)?;

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let mut type_stream: Vec<u8> = Vec::new();
        type_stream.write_u32_varint(stream.read_u32_varint()?)?;

        let command_uuid = Uuid::proto_deserialize(stream)?;
        let request_id = String::proto_deserialize(stream)?;
        stream.read_to_end(&mut type_stream)?;
        let mut type_cursor = Cursor::new(type_stream.as_slice());
        let command_type = CommandOriginType::proto_deserialize(&mut type_cursor)?;

        Ok(Self {
            command_type,
            command_uuid,
            request_id,
        })
    }

    fn get_size_prediction(&self) -> usize {
        self.command_type.get_size_prediction()
            + self.command_uuid.get_size_prediction()
            + self.request_id.get_size_prediction()
    }
}