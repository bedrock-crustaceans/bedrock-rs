use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, packet};
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecVAR};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write, copy};
use std::mem::size_of;

#[packet(id = 79)]
#[derive(Clone, Debug)]
pub struct CommandOutputPacket<V: ProtoVersion> {
    pub origin_data: V::CommandOriginData,
    pub output_type: V::CommandOutputType,
    pub success_count: u32,
    pub output_messages: Vec<OutputMessagesEntry>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct OutputMessagesEntry {
    pub successful: bool,
    pub message_id: String,
    pub parameters: Vec<String>,
}

impl<V: ProtoVersion> ProtoCodec for CommandOutputPacket<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        let mut output_type_stream: Vec<u8> = Vec::new();
        <V::CommandOutputType as ProtoCodec>::serialize(
            &self.output_type,
            &mut output_type_stream,
        )?;
        let mut output_type_cursor = Cursor::new(output_type_stream.as_slice());

        <V::CommandOriginData as ProtoCodec>::serialize(&self.origin_data, stream)?;
        stream.write_i8(output_type_cursor.read_i8()?)?;
        <u32 as ProtoCodecVAR>::serialize(&self.success_count, stream)?;
        <u32 as ProtoCodecVAR>::serialize(&(self.output_messages.len() as u32), stream)?;
        for i in &self.output_messages {
            <OutputMessagesEntry as ProtoCodec>::serialize(i, stream)?;
        }
        copy(&mut output_type_cursor, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let mut output_type_stream: Vec<u8> = Vec::new();

        let origin_data = <V::CommandOriginData as ProtoCodec>::deserialize(stream)?;
        output_type_stream.write_i8(stream.read_i8()?)?;
        let success_count = <u32 as ProtoCodecVAR>::deserialize(stream)?;
        let output_messages = {
            let len = <u32 as ProtoCodecVAR>::deserialize(stream)?;
            let mut vec = Vec::with_capacity(len.try_into()?);
            for _ in 0..len {
                vec.push(<OutputMessagesEntry as ProtoCodec>::deserialize(stream)?);
            }
            vec
        };
        stream.read_to_end(&mut output_type_stream)?;

        let mut output_type_cursor = Cursor::new(output_type_stream.as_slice());
        let output_type =
            <V::CommandOutputType as ProtoCodec>::deserialize(&mut output_type_cursor)?;

        Ok(Self {
            origin_data,
            output_type,
            success_count,
            output_messages,
        })
    }

    fn size_hint(&self) -> usize {
        self.origin_data.size_hint()
            + self.output_type.size_hint()
            + self.success_count.size_hint()
            + size_of::<u32>()
            + self
                .output_messages
                .iter()
                .map(|i| i.size_hint())
                .sum::<usize>()
    }
}

// TODO: verify ProtoCodec impl
