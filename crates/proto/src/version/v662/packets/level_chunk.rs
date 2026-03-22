use crate::version::versions::ProtoVersion;
use bedrockrs_macros::packet;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use std::io::{Read, Write};

#[packet(id = 58)]
#[derive(Clone, Debug)]
pub struct LevelChunkPacket<V: ProtoVersion> {
    pub chunk_position: V::ChunkPos,
    pub dimension_id: i32,
    pub sub_chunk_count: u32,
    pub sub_chunk_limit: u16,
    pub cache_enabled: bool,
    pub cache_blobs: Vec<u64>,
    pub serialized_chunk_data: Vec<u8>,
}

impl <V: ProtoVersion> LevelChunkPacket<V> {
    pub const LIMITLESS: u32 = u32::MAX;
    pub const LIMITED: u32 = u32::MAX - 1;
}

impl <V: ProtoVersion> ProtoCodec for LevelChunkPacket<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        self.chunk_position.serialize(stream)?;
        <i32 as ProtoCodecVAR>::serialize(&self.dimension_id, stream)?;
        <u32 as ProtoCodecVAR>::serialize(&self.sub_chunk_count, stream)?;
        if self.sub_chunk_count == Self::LIMITED { <u16 as ProtoCodecLE>::serialize(&self.sub_chunk_limit, stream)? };
        self.cache_enabled.serialize(stream)?;
        if self.cache_enabled { <Vec<u64> as ProtoCodecLE>::serialize(&self.cache_blobs, stream)? };
        self.serialized_chunk_data.serialize(stream)?;
        
        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let chunk_position = V::ChunkPos::deserialize(stream)?;
        let dimension_id = <i32 as ProtoCodecVAR>::deserialize(stream)?;
        let sub_chunk_count = <u32 as ProtoCodecVAR>::deserialize(stream)?;
        let sub_chunk_limit = match sub_chunk_count {
            Self::LIMITED => <u16 as ProtoCodecLE>::deserialize(stream)?,
            _ => 0
        };
        let cache_enabled = bool::deserialize(stream)?;
        let cache_blobs = match cache_enabled {
            true => <Vec<u64> as ProtoCodecVAR>::deserialize(stream)?,
            _ => vec![],
        };
        let serialized_chunk_data = Vec::<u8>::deserialize(stream)?;
        
        Ok(Self {
            chunk_position,
            dimension_id,
            sub_chunk_count,
            sub_chunk_limit,
            cache_enabled,
            cache_blobs,
            serialized_chunk_data
        })
    }

    fn size_hint(&self) -> usize {
        self.chunk_position.size_hint()
        + <i32 as ProtoCodecVAR>::size_hint(&self.dimension_id)
        + <u32 as ProtoCodecVAR>::size_hint(&self.sub_chunk_count)
        + match self.sub_chunk_count {
            Self::LIMITED => <u16 as ProtoCodecLE>::size_hint(&self.sub_chunk_limit),
            _ => 0,
        }
        + self.cache_enabled.size_hint()
        + match self.cache_enabled {
            true => <Vec<u64> as ProtoCodecLE>::size_hint(&self.cache_blobs),
            _ => 0,
        }
        + self.serialized_chunk_data.size_hint()
    }
}
