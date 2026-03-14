use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::ProtoCodec;
use std::io::{Read, Write};

#[derive(Clone, Debug)]
pub struct BaseGameVersion(pub String);

impl ProtoCodec for BaseGameVersion {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        self.0.serialize(stream)?;
        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        Ok(Self(String::deserialize(stream)?))
    }

    fn size_hint(&self) -> usize {
        self.0.size_hint()
    }
}
