use crate::ProtoCodec;
use crate::error::ProtoCodecError;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};
use std::mem::size_of;
use uuid::Uuid;

impl ProtoCodec for Uuid {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        let (upper, lower) = self.as_u64_pair();

        stream.write_u64::<LittleEndian>(upper)?;
        stream.write_u64::<LittleEndian>(lower)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        Ok(Uuid::from_u64_pair(
            stream.read_u64::<LittleEndian>()?,
            stream.read_u64::<LittleEndian>()?,
        ))
    }

    fn size_hint(&self) -> usize {
        size_of::<u64>() + size_of::<u64>()
    }
}
