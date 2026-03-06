use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::Cursor;
use std::mem::size_of;

use crate::ProtoCodec;
use crate::error::ProtoCodecError;

impl ProtoCodec for bool {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError>
    where
        Self: Sized,
    {
        match self {
            true => stream.write_u8(1)?,
            false => stream.write_u8(0)?,
        };

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError>
    where
        Self: Sized,
    {
        Ok(!matches!(stream.read_u8()?, 0))
    }

    fn get_size_prediction(&self) -> usize {
        size_of::<u8>()
    }
}
