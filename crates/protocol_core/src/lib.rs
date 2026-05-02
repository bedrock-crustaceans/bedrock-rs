extern crate core;

use std::any::Any;
use std::fmt::Debug;
use std::io::{Read, Write};

use crate::error::{PacketCodecError, ProtoCodecError};

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

pub trait Packet: Debug + Send + Sync + Sized + ProtoCodec {
    const ID: u16;
    const COMPRESS: bool;
    const ENCRYPT: bool;
}

pub trait PacketDyn: Debug + Send + Sync + Any + 'static {
    fn id(&self) -> u16;
    fn name(&self) -> &'static str;
}

impl<T: Packet + 'static> PacketDyn for T {
    fn id(&self) -> u16 {
        T::ID
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}

#[derive(Clone, Debug)]
pub struct UnknownPacket {
    pub id: u16,
    pub buf: Box<[u8]>,
}

impl PacketDyn for UnknownPacket {
    fn id(&self) -> u16 {
        self.id
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<UnknownPacket>()
    }
}

pub trait Packets: Sized {
    fn id(&self) -> u16;
    fn compress(&self) -> bool;
    fn encrypt(&self) -> bool;

    fn serialize<W: Write>(
        &self,
        header: &PacketHeader,
        stream: &mut W,
    ) -> Result<(), PacketCodecError>;

    fn deserialize<R: Read>(stream: &mut R) -> Result<(Self, PacketHeader), PacketCodecError>;

    fn size_hint(&self, header: &PacketHeader) -> usize;

    fn as_dyn(&self) -> &dyn PacketDyn;
    fn into_dyn(self) -> Box<dyn PacketDyn>;
}
