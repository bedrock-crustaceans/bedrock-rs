extern crate core;

use std::io::{Read, Write};

use crate::error::ProtoCodecError;

mod endian;
pub mod error;
mod header;
pub mod sub_client;
pub mod types;

pub use endian::*;
pub use header::*;

pub trait ProtoCodec<T: Sized> {
    fn serialize<W: Write>(value: &T, stream: &mut W) -> Result<(), ProtoCodecError>;

    fn deserialize<R: Read>(stream: &mut R) -> Result<T, ProtoCodecError>;

    fn size_hint(value: &T) -> usize;
}

pub trait Packet: Sized + ProtoCodec<Self> {
    const ID: u16;
    const COMPRESS: bool;
    const ENCRYPT: bool;
}

pub trait Packets: Sized {
    fn id(&self) -> u16;
    fn compress(&self) -> bool;
    fn encrypt(&self) -> bool;

    fn serialize<W: Write>(
        &self,
        header: &PacketHeader,
        stream: &mut W,
    ) -> Result<(), ProtoCodecError>;

    fn deserialize<R: Read>(stream: &mut R) -> Result<(Self, PacketHeader), ProtoCodecError>;

    fn size_hint(&self, header: &PacketHeader) -> usize;
}
