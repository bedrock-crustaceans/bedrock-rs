extern crate core;

use crate::error::{PacketCodecError, ProtoCodecError};
use std::any::Any;
use std::fmt::Debug;
use std::io::{Read, Write};

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

pub trait Packet: ProtoCodec + Debug + Send + Sync + Any + 'static {
    const ID: u16;
}

pub trait DynPacket: Debug + Send + Sync + Any + 'static {
    fn id(&self) -> u16;
}

impl<T: Packet> DynPacket for T {
    #[inline]
    fn id(&self) -> u16 {
        T::ID
    }
}

#[derive(Clone, Debug)]
pub struct UnknownPacket {
    pub id: u16,
    pub buf: Box<[u8]>,
}

impl DynPacket for UnknownPacket {
    fn id(&self) -> u16 {
        self.id
    }
}

pub trait Packets: DynPacket + Sized {
    fn serialize<W: Write>(
        &self,
        header: &PacketHeader,
        stream: &mut W,
    ) -> Result<(), PacketCodecError>;

    fn deserialize<R: Read>(stream: &mut R) -> Result<(Self, PacketHeader), PacketCodecError>;
    fn size_hint(&self, header: &PacketHeader) -> usize;

    fn inner(&self) -> &dyn DynPacket;
    fn into_inner(self) -> Box<dyn DynPacket>;
}
