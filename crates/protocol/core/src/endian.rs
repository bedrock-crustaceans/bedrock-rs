use crate::error::ProtoCodecError;
use std::io::{Read, Write};

pub trait ProtoCodecLE: Sized {
    /// Writes the value to `stream` in this encoding.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `stream` fails or the value cannot be represented on the wire.
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError>;

    /// Reads a value from `stream` in this encoding.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from `stream` fails, the stream ends early, or the bytes are not a valid value.
    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError>;

    fn size_hint(&self) -> usize;
}

pub trait ProtoCodecBE: Sized {
    /// Writes the value to `stream` in this encoding.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `stream` fails or the value cannot be represented on the wire.
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError>;

    /// Reads a value from `stream` in this encoding.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from `stream` fails, the stream ends early, or the bytes are not a valid value.
    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError>;

    fn size_hint(&self) -> usize;
}

pub trait ProtoCodecVAR: Sized {
    /// Writes the value to `stream` in this encoding.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `stream` fails or the value cannot be represented on the wire.
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError>;

    /// Reads a value from `stream` in this encoding.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from `stream` fails, the stream ends early, or the bytes are not a valid value.
    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError>;

    fn size_hint(&self) -> usize;
}
