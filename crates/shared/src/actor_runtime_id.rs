use bedrockrs_proto_core::ProtoCodec;
use bedrockrs_proto_core::error::ProtoCodecError;
use std::io::{Read, Write};
use std::mem::size_of;
use varint_rs::{VarintReader, VarintWriter};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ActorRuntimeID(pub u64);

// ProtoCodec
impl ProtoCodec for ActorRuntimeID {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        Ok(stream.write_u64_varint(self.0)?)
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        Ok(Self(stream.read_u64_varint()?))
    }

    fn size_hint(&self) -> usize {
        size_of::<u64>()
    }
}
