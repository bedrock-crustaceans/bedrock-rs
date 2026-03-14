use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};
use std::mem::size_of;

use crate::ProtoCodec;
use crate::error::ProtoCodecError;

impl ProtoCodec for bool {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError>
    where
        Self: Sized,
    {
        match self {
            true => stream.write_u8(1)?,
            false => stream.write_u8(0)?,
        };

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError>
    where
        Self: Sized,
    {
        Ok(!matches!(stream.read_u8()?, 0))
    }

    fn size_hint(&self) -> usize {
        size_of::<u8>()
    }
}
