use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::ProtoCodec;
use std::io::{Read, Write};
use std::mem::size_of;
use varint_rs::{VarintReader, VarintWriter};

#[derive(Clone, Debug)]
pub struct ActorUniqueID(pub i64); // TODO: consider removing this type and using primitive types directly

impl ProtoCodec for ActorUniqueID {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        stream.write_i64_varint(self.0)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        Ok(Self(stream.read_i64_varint()?))
    }

    fn size_hint(&self) -> usize {
        size_of::<i64>()
    }
}
