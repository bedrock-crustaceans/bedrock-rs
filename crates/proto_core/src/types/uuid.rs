use crate::ProtoCodec;
use crate::error::ProtoCodecError;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;
use std::mem::size_of;
use uuid::Uuid;

impl ProtoCodec for Uuid {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        let (upper, lower) = self.as_u64_pair();

        stream.write_u64::<LittleEndian>(upper)?;
        stream.write_u64::<LittleEndian>(lower)?;

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        Ok(Uuid::from_u64_pair(
            stream.read_u64::<LittleEndian>()?,
            stream.read_u64::<LittleEndian>()?,
        ))
    }

    fn get_size_prediction(&self) -> usize {
        size_of::<u64>() + size_of::<u64>()
    }
}
