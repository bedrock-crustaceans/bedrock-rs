extern crate core;

use std::io::{Cursor, Read, Write};

use crate::error::ProtoCodecError;

mod endian;
use crate::sub_client::SubClientID;
pub use endian::*;

pub mod error;
pub mod sub_client;
pub mod types;

pub trait ProtoCodec: Sized {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError>;

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError>;

    fn size_hint(&self) -> usize;
}

pub trait GamePacket: Sized + ProtoCodec {
    const ID: u16;
    const COMPRESS: bool;
    const ENCRYPT: bool;

    #[inline]
    fn size_hint(&self) -> usize {
        <Self as ProtoCodec>::size_hint(self)
    }
}

pub trait GamePacketsAll: Sized {
    fn id(&self) -> u16;
    fn compress(&self) -> bool;
    fn encrypt(&self) -> bool;

    fn pk_serialize(
        &self,
        stream: &mut Vec<u8>,
        subclient_sender_id: SubClientID,
        subclient_target_id: SubClientID,
    ) -> Result<(), ProtoCodecError>;
    fn pk_deserialize(
        stream: &mut Cursor<&[u8]>,
    ) -> Result<(Self, SubClientID, SubClientID), ProtoCodecError>;

    fn size_hint(&self) -> usize;
}
