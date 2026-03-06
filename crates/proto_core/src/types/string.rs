use std::convert::TryInto;
use std::io::{Cursor, Read, Write};
use varint_rs::{VarintReader, VarintWriter};

use crate::ProtoCodec;
use crate::error::ProtoCodecError;

impl ProtoCodec for String {
    fn proto_serialize(&self, buf: &mut Vec<u8>) -> Result<(), ProtoCodecError>
    where
        Self: Sized,
    {
        let len = self.len().try_into()?;

        buf.write_u32_varint(len)?;
        buf.write_all(self.as_bytes())?;

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError>
    where
        Self: Sized,
    {
        let len = stream.read_u32_varint()?.try_into()?;

        let mut string_buf = vec![0u8; len];
        stream.read_exact(&mut string_buf)?;

        Ok(String::from_utf8(string_buf)?)
    }

    fn get_size_prediction(&self) -> usize {
        // 4 = u32 String size
        self.len() + 4
    }
}
