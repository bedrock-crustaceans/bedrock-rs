use bedrock_protocol_core::ProtoCodec;
use bedrock_protocol_core::error::ProtoCodecError;
use std::io::{Read, Write};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ContainerID(pub i8);

impl ContainerID {
    pub const NONE: Self = Self(-1);
    pub const INVENTORY: Self = Self(0);
    pub const FIRST: Self = Self(1);
    pub const LAST: Self = Self(100);
    pub const OFFHAND: Self = Self(119);
    pub const ARMOR: Self = Self(120);
    pub const SELECTION_SLOTS: Self = Self(122);
    pub const PLAYER_ONLY_UI: Self = Self(124);
}

impl ProtoCodec for ContainerID {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        self.0.serialize(stream)
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        Ok(Self(i8::deserialize(stream)?))
    }

    fn size_hint(&self) -> usize {
        self.0.size_hint()
    }
}
