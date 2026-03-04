use crate::version::proto_version::ProtoVersion;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecVAR};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};

#[derive(Clone, Debug)]
pub struct ItemStackResponseInfo<V: ProtoVersion> {
    pub result: V::ItemStackNetResult,
    pub client_request_id: i32,
}

impl<V: ProtoVersion> ProtoCodec for ItemStackResponseInfo<V> {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        let mut result_stream: Vec<u8> = Vec::new();

        self.result.proto_serialize(&mut result_stream)?;
        let mut result_cursor = Cursor::new(result_stream.as_slice());

        stream.write_i8(result_cursor.read_i8()?)?;
        <i32 as ProtoCodecVAR>::proto_serialize(&self.client_request_id, stream)?;
        result_cursor.read_to_end(stream)?;

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let mut result_stream: Vec<u8> = Vec::new();

        result_stream.write_i8(stream.read_i8()?)?;
        let client_request_id = <i32 as ProtoCodecVAR>::proto_deserialize(stream)?;
        stream.read_to_end(&mut result_stream)?;

        let mut result_cursor = Cursor::new(result_stream.as_slice());
        let result = V::ItemStackNetResult::proto_deserialize(&mut result_cursor)?;

        Ok(Self {
            result,
            client_request_id,
        })
    }

    fn get_size_prediction(&self) -> usize {
        self.result.get_size_prediction()
            + <i32 as ProtoCodecVAR>::get_size_prediction(&self.client_request_id)
    }
}
