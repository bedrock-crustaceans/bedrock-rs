use bedrock_protocol_core::ProtoCodecVAR;
use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::ProtoCodec;
use std::io::{Read, Write};
use std::mem::size_of;

#[derive(Clone, Debug)]
pub struct ActorUniqueID(pub i64); // TODO: consider removing this type and using primitive types directly

impl ProtoCodec for ActorUniqueID {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        <i64 as ProtoCodecVAR>::serialize(&self.0, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        Ok(Self(<i64 as ProtoCodecVAR>::deserialize(stream)?))
    }

    fn size_hint(&self) -> usize {
        size_of::<i64>()
    }
}
