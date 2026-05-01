use crate::ProtoCodec;
use crate::error::ProtoCodecError;
use std::io::{Read, Write};
use std::mem::size_of;
use xuid::Xuid;

impl ProtoCodec for Xuid {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        self.to_string().serialize(stream)?;
        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        Ok(Xuid::try_from(String::deserialize(stream)?)?)
    }

    fn size_hint(&self) -> usize {
        // 20 = u64::MAX as String
        size_of::<u32>() + 20
    }
}
