use super::super::enums::CommandOutputType;
use super::super::types::CommandOriginData;
use bedrockrs_macros::{gamepacket, ProtoCodec};
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecVAR};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};
use std::mem::size_of;

#[gamepacket(id = 79)]
#[derive(Clone, Debug)]
pub struct CommandOutputPacket {
    pub origin_data: CommandOriginData,
    pub output_type: CommandOutputType,
    pub success_count: u32,
    pub output_messages: Vec<OutputMessagesEntry>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct OutputMessagesEntry {
    pub successful: bool,
    pub message_id: String,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub parameters: Vec<String>,
}

impl ProtoCodec for CommandOutputPacket {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        let mut output_type_stream: Vec<u8> = Vec::new();
        <CommandOutputType as ProtoCodec>::proto_serialize(
            &self.output_type,
            &mut output_type_stream,
        )?;
        let mut output_type_cursor = Cursor::new(output_type_stream.as_slice());

        <CommandOriginData as ProtoCodec>::proto_serialize(&self.origin_data, stream)?;
        stream.write_i8(output_type_cursor.read_i8()?)?;
        <u32 as ProtoCodecVAR>::proto_serialize(&self.success_count, stream)?;
        <u32 as ProtoCodecVAR>::proto_serialize(&(self.output_messages.len() as u32), stream)?;
        for i in &self.output_messages {
            <OutputMessagesEntry as ProtoCodec>::proto_serialize(i, stream)?;
        }
        output_type_cursor.read_to_end(stream)?;

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let mut output_type_stream: Vec<u8> = Vec::new();

        let origin_data = <CommandOriginData as ProtoCodec>::proto_deserialize(stream)?;
        output_type_stream.write_i8(stream.read_i8()?)?;
        let success_count = <u32 as ProtoCodecVAR>::proto_deserialize(stream)?;
        let output_messages = {
            let len = <u32 as ProtoCodecVAR>::proto_deserialize(stream)?;
            let mut vec = Vec::with_capacity(len.try_into()?);
            for _ in 0..len {
                vec.push(<OutputMessagesEntry as ProtoCodec>::proto_deserialize(
                    stream,
                )?);
            }
            vec
        };
        stream.read_to_end(&mut output_type_stream)?;

        let mut output_type_cursor = Cursor::new(output_type_stream.as_slice());
        let output_type =
            <CommandOutputType as ProtoCodec>::proto_deserialize(&mut output_type_cursor)?;

        Ok(Self {
            origin_data,
            output_type,
            success_count,
            output_messages,
        })
    }

    fn get_size_prediction(&self) -> usize {
        self.origin_data.get_size_prediction()
            + self.output_type.get_size_prediction()
            + self.success_count.get_size_prediction()
            + size_of::<u32>()
            + self
            .output_messages
            .iter()
            .map(|i| i.get_size_prediction())
            .sum::<usize>()
    }
}

// VERIFY: ProtoCodec impl
