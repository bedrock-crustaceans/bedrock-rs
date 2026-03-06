use crate::ProtoCodec;
use crate::error::ProtoCodecError;
use std::io::Cursor;
use std::mem::size_of;
use xuid::Xuid;

impl ProtoCodec for Xuid {
    fn proto_serialize(&self, buf: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        self.to_string().proto_serialize(buf)?;
        Ok(())
    }

    fn proto_deserialize(buf: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        Ok(Xuid::try_from(String::proto_deserialize(buf)?)?)
    }

    fn get_size_prediction(&self) -> usize {
        // 20 = u64::MAX as String
        size_of::<u32>() + 20
    }
}
