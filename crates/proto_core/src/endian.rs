use crate::error::ProtoCodecError;
use std::io::{Read, Write};

pub trait ProtoCodecLE: Sized {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError>;

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError>;

    fn size_hint(&self) -> usize;
}

pub trait ProtoCodecBE: Sized {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError>;

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError>;

    fn size_hint(&self) -> usize;
}

pub trait ProtoCodecVAR: Sized {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError>;

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError>;

    fn size_hint(&self) -> usize;
}
