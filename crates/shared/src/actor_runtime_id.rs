use bedrock_protocol_core::ProtoCodec;
use bedrock_protocol_core::ProtoCodecVAR;
use bedrock_protocol_core::error::ProtoCodecError;
use std::io::{Read, Write};
use std::mem::size_of;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ActorRuntimeID(pub u64);

// ProtoCodec
impl ProtoCodec for ActorRuntimeID {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        <u64 as ProtoCodecVAR>::serialize(&self.0, stream)
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        Ok(Self(<u64 as ProtoCodecVAR>::deserialize(stream)?))
    }

    fn size_hint(&self) -> usize {
        size_of::<u64>()
    }
}
