use crate::version::versions::ProtoVersion;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecVAR};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write, copy};

#[derive(Clone, Debug)]
pub struct ItemStackResponseInfo<V: ProtoVersion> {
    pub result: V::ItemStackNetResult,
    pub client_request_id: i32,
}

impl<V: ProtoVersion> ProtoCodec for ItemStackResponseInfo<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        let mut result_stream: Vec<u8> = Vec::new();

        self.result.serialize(&mut result_stream)?;
        let mut result_cursor = Cursor::new(result_stream.as_slice());

        stream.write_i8(result_cursor.read_i8()?)?;
        <i32 as ProtoCodecVAR>::serialize(&self.client_request_id, stream)?;
        copy(&mut result_cursor, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let mut result_stream: Vec<u8> = Vec::new();

        result_stream.write_i8(stream.read_i8()?)?;
        let client_request_id = <i32 as ProtoCodecVAR>::deserialize(stream)?;
        stream.read_to_end(&mut result_stream)?;

        let mut result_cursor = Cursor::new(result_stream.as_slice());
        let result = V::ItemStackNetResult::deserialize(&mut result_cursor)?;

        Ok(Self {
            result,
            client_request_id,
        })
    }

    fn size_hint(&self) -> usize {
        self.result.size_hint() + <i32 as ProtoCodecVAR>::size_hint(&self.client_request_id)
    }
}
