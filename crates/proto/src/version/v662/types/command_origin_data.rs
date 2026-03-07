use crate::version::versions::ProtoVersion;
use bedrockrs_proto_core::ProtoCodec;
use bedrockrs_proto_core::error::ProtoCodecError;
use std::io::{Cursor, Read, Write, copy};
use uuid::Uuid;
use varint_rs::{VarintReader, VarintWriter};

#[derive(Clone, Debug)]
pub struct CommandOriginData<V: ProtoVersion> {
    pub command_type: V::CommandOriginType,
    pub command_uuid: Uuid,
    pub request_id: String,
}

impl<V: ProtoVersion> ProtoCodec for CommandOriginData<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        let mut type_stream: Vec<u8> = Vec::new();
        self.command_type.serialize(&mut type_stream)?;
        let mut type_cursor = Cursor::new(type_stream.as_slice());

        stream.write_u32_varint(type_cursor.read_u32_varint()?)?;
        self.command_uuid.serialize(stream)?;
        self.request_id.serialize(stream)?;
        copy(&mut type_cursor, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let mut type_stream: Vec<u8> = Vec::new();
        type_stream.write_u32_varint(stream.read_u32_varint()?)?;

        let command_uuid = Uuid::deserialize(stream)?;
        let request_id = String::deserialize(stream)?;
        stream.read_to_end(&mut type_stream)?;
        let mut type_cursor = Cursor::new(type_stream.as_slice());
        let command_type = V::CommandOriginType::deserialize(&mut type_cursor)?;

        Ok(Self {
            command_type,
            command_uuid,
            request_id,
        })
    }

    fn size_hint(&self) -> usize {
        self.command_type.size_hint() + self.command_uuid.size_hint() + self.request_id.size_hint()
    }
}
