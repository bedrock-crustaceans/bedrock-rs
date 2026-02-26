use bedrockrs_macros::gamepacket;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use std::io::Cursor;
use std::mem::size_of;

#[gamepacket(id = 135)]
#[derive(Clone, Debug)]
pub struct ClientCacheBlobStatusPacket {
    pub missing_blobs: Vec<u64>,
    pub obtained_blobs: Vec<u64>,
}

impl ProtoCodec for ClientCacheBlobStatusPacket {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        <u32 as ProtoCodecVAR>::proto_serialize(&(self.missing_blobs.len() as u32), stream)?;
        <u32 as ProtoCodecVAR>::proto_serialize(&(self.obtained_blobs.len() as u32), stream)?;
        for i in &self.missing_blobs {
            <u64 as ProtoCodecLE>::proto_serialize(i, stream)?;
        }
        for i in &self.obtained_blobs {
            <u64 as ProtoCodecLE>::proto_serialize(i, stream)?;
        }

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let missing_blobs_len = <u32 as ProtoCodecVAR>::proto_deserialize(stream)?;
        let obtained_blobs_len = <u32 as ProtoCodecVAR>::proto_deserialize(stream)?;
        let mut missing_blobs = Vec::with_capacity(missing_blobs_len.try_into()?);
        let mut obtained_blobs = Vec::with_capacity(obtained_blobs_len.try_into()?);
        for _ in 0..missing_blobs_len {
            missing_blobs.push(<u64 as ProtoCodecLE>::proto_deserialize(stream)?);
        }
        for _ in 0..obtained_blobs_len {
            obtained_blobs.push(<u64 as ProtoCodecLE>::proto_deserialize(stream)?);
        }

        Ok(Self {
            missing_blobs,
            obtained_blobs,
        })
    }

    fn get_size_prediction(&self) -> usize {
        size_of::<u32>()
            + size_of::<u32>()
            + self.missing_blobs.len() * size_of::<u64>()
            + self.obtained_blobs.len() * size_of::<u64>()
    }
}

// VERIFY: ProtoCodec impl
