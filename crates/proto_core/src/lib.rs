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

pub trait ProtoCodec: Sized {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError>;

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError>;

    fn size_hint(&self) -> usize;
}

pub trait Packet: Sized + ProtoCodec {
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
