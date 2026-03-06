use bedrockrs_macros::packet;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use std::io::{Read, Write};
use std::mem::size_of;

#[packet(id = 135)]
#[derive(Clone, Debug)]
pub struct ClientCacheBlobStatusPacket {
    pub missing_blobs: Vec<u64>,
    pub obtained_blobs: Vec<u64>,
}

impl ProtoCodec for ClientCacheBlobStatusPacket {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        <u32 as ProtoCodecVAR>::serialize(&(self.missing_blobs.len() as u32), stream)?;
        <u32 as ProtoCodecVAR>::serialize(&(self.obtained_blobs.len() as u32), stream)?;
        for i in &self.missing_blobs {
            <u64 as ProtoCodecLE>::serialize(i, stream)?;
        }
        for i in &self.obtained_blobs {
            <u64 as ProtoCodecLE>::serialize(i, stream)?;
        }

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let missing_blobs_len = <u32 as ProtoCodecVAR>::deserialize(stream)?;
        let obtained_blobs_len = <u32 as ProtoCodecVAR>::deserialize(stream)?;
        let mut missing_blobs = Vec::with_capacity(missing_blobs_len.try_into()?);
        let mut obtained_blobs = Vec::with_capacity(obtained_blobs_len.try_into()?);
        for _ in 0..missing_blobs_len {
            missing_blobs.push(<u64 as ProtoCodecLE>::deserialize(stream)?);
        }
        for _ in 0..obtained_blobs_len {
            obtained_blobs.push(<u64 as ProtoCodecLE>::deserialize(stream)?);
        }

        Ok(Self {
            missing_blobs,
            obtained_blobs,
        })
    }

    fn size_hint(&self) -> usize {
        size_of::<u32>()
            + size_of::<u32>()
            + self.missing_blobs.len() * size_of::<u64>()
            + self.obtained_blobs.len() * size_of::<u64>()
    }
}

// VERIFY: ProtoCodec impl
