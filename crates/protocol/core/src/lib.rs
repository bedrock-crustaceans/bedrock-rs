extern crate core;

use crate::error::{PacketCodecError, ProtoCodecError};
use std::any::Any;
use std::fmt::Debug;
use std::io::{Read, Write};

mod endian;
pub mod error;
mod header;
pub mod sub_client;
pub mod trace;
pub mod types;

pub use endian::*;
pub use header::*;

#[cfg(feature = "packet-meta")]
mod meta;
#[cfg(feature = "packet-meta")]
pub use meta::*;

mod unknown;
pub use unknown::*;

pub trait ProtoCodec: Sized {
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

pub trait Packet: ProtoCodec + Debug + Send + Sync + Any + 'static {
    const ID: u16;

    #[cfg(feature = "packet-meta")]
    const META: PacketMeta;
}

pub trait Packets: Sized {
    /// Writes the packet header followed by the packet body to `stream`.
    ///
    /// # Errors
    ///
    /// Returns an error if the header or the body cannot be written to `stream`.
    fn serialize<W: Write>(
        &self,
        header: &PacketHeader,
        stream: &mut W,
    ) -> Result<(), PacketCodecError>;

    /// Reads a packet header and the packet body it announces from `stream`.
    ///
    /// # Errors
    ///
    /// Returns an error if the header or the body cannot be read from `stream`.
    fn deserialize<R: Read>(stream: &mut R) -> Result<(Self, PacketHeader), PacketCodecError>;

    fn size_hint(&self, header: &PacketHeader) -> usize;

    fn id(&self) -> u16;
}

#[cfg(feature = "packet-dyn")]
pub trait PacketDyn: Debug + Send + Sync + Any + 'static {
    fn id(&self) -> u16;

    #[cfg(feature = "packet-meta")]
    fn meta(&self) -> &'static PacketMeta;
}

#[cfg(feature = "packet-dyn")]
impl<T: Packet> PacketDyn for T {
    #[inline]
    fn id(&self) -> u16 {
        T::ID
    }

    #[cfg(feature = "packet-meta")]
    #[inline]
    fn meta(&self) -> &'static PacketMeta {
        &T::META
    }
}
